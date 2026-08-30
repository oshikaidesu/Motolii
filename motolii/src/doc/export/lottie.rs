
use std::collections::HashSet;

use crate::doc::core::{Fps, RationalTimeError};
use crate::render::media::is_point_cloud_path;
use crate::doc::store::{
    property, EffectInstance, LayerAttrs, LayerId, LayerMeta, LayerSource, PropertyId, StoreError,
    StoreView, Value,
};

mod properties;
mod shapes;
mod text;
mod enums;

use enums::{blend_mode_to_int, matte_mode_to_int};
use properties::{build_markers, build_masks, build_slots, scalar_property, vector_property};
use shapes::shape_node_to_json;
use text::build_text_data;

#[derive(Debug, Clone, PartialEq)]
pub struct UnsupportedForLottie {
    pub layer: Option<LayerId>,
    pub category: &'static str,
    pub detail: String,
}

impl std::fmt::Display for UnsupportedForLottie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.layer {
            Some(layer) => write!(f, "[{}] layer {}: {}", self.category, layer.0, self.detail),
            None => write!(f, "[{}] {}", self.category, self.detail),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LottieExportError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Time(#[from] RationalTimeError),
    #[error("comp の設定が Document に無い")]
    NoComposition,
    #[error("property `{0}` の値の型が期待と違う: {1:?}")]
    TypeMismatch(String, Value),
    #[error(
        "補間型 `{0}` は Lottie に写せない(ベジェへ焼くか拡張として持つかが未決 —\
         `docs/reviews/2026-08-28-current-position.md` の裁定待ち)"
    )]
    UnrepresentableEasing(&'static str),
}

pub struct LottieExport {
    pub json: serde_json::Value,
    pub unsupported: Vec<UnsupportedForLottie>,
}

struct Ctx<'a, 'b> {
    view: &'a StoreView<'b>,
    fps: Fps,
    duration_frames: i64,
}

pub fn export_lottie(view: &StoreView<'_>) -> Result<LottieExport, LottieExportError> {
    let composition = view.composition()?.ok_or(LottieExportError::NoComposition)?;
    let fps = composition.fps;
    let ctx = Ctx {
        view,
        fps,
        duration_frames: composition.duration_frames,
    };
    let mut unsupported = Vec::new();

    check_camera(&ctx, &mut unsupported)?;

    let layers = view.layers();

    let mut matte_sources: HashSet<LayerId> = HashSet::new();
    for &layer in &layers {
        if let Some(matte) = attrs_of(view, layer)?.matte {
            matte_sources.insert(matte.layer);
        }
    }

    let mut assets = Vec::new();
    let mut out_layers = Vec::new();
    for &layer in &layers {
        let is_matte_source = matte_sources.contains(&layer);
        let value = build_layer(&ctx, layer, is_matte_source, &mut assets, &mut unsupported)?;
        out_layers.push(value);
    }

    let markers = build_markers(&ctx)?;
    let slots = build_slots(&ctx, &mut unsupported)?;

    let mut root = serde_json::json!({
        "v": "5.7.4",
        "fr": fps.as_f64(),
        "ip": 0.0,
        "op": ctx.duration_frames as f64,
        "w": composition.width,
        "h": composition.height,
        "layers": out_layers,
        "assets": assets,
        "markers": markers,
    });
    if !slots.is_empty() {
        root["slots"] = serde_json::Value::Object(slots);
    }

    Ok(LottieExport {
        json: root,
        unsupported,
    })
}

fn attrs_of(view: &StoreView<'_>, layer: LayerId) -> Result<LayerAttrs, LottieExportError> {
    Ok(view.attrs(layer)?.unwrap_or_default())
}

fn meta_of(view: &StoreView<'_>, layer: LayerId) -> Result<LayerMeta, LottieExportError> {
    view.meta(layer)?.ok_or_else(|| {
        LottieExportError::TypeMismatch(format!("layer {} に meta が無い", layer.0), Value::Bool(false))
    })
}

fn check_camera(
    ctx: &Ctx<'_, '_>,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<(), LottieExportError> {
    let names = [property::CAMERA_CENTER, property::CAMERA_ZOOM, property::CAMERA_ROLL];
    for name in names {
        let property = PropertyId::camera(name)?;
        if ctx.view.camera_property_source(&property)?.is_some() {
            unsupported.push(UnsupportedForLottie {
                layer: None,
                category: "camera",
                detail: format!(
                    "comp のカメラ property `{name}` が使われているが、Lottie に \
                     camera-layer 相当の語彙が無い(layers/camera-layer は不採用、裁定65)"
                ),
            });
            return Ok(());
        }
    }
    Ok(())
}

fn build_layer(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    is_matte_source: bool,
    assets: &mut Vec<serde_json::Value>,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    let view = ctx.view;
    let meta = meta_of(view, layer)?;
    let attrs = attrs_of(view, layer)?;

    let mut out = serde_json::json!({
        "ddd": 0,
        "ind": layer.0,
        "nm": attrs.name,
        "hd": attrs.hidden,
        "ip": meta.timing.start as f64,
        "op": (meta.timing.start + meta.timing.duration) as f64,
        "st": (meta.timing.start - meta.timing.source_in) as f64,
        "sr": meta.timing.speed.num() as f64 / meta.timing.speed.den() as f64,
        "ao": if attrs.auto_orient { 1 } else { 0 },
        "bm": blend_mode_to_int(attrs.blend_mode),
        "ks": build_transform(ctx, layer, unsupported)?,
    });

    if let Some(parent) = attrs.parent {
        if view.has_layer(parent) {
            out["parent"] = serde_json::json!(parent.0);
        }
    }

    if let Some(matte) = attrs.matte {
        out["tt"] = serde_json::json!(matte_mode_to_int(matte.mode));
        out["tp"] = serde_json::json!(matte.layer.0);
    }
    if is_matte_source {
        out["td"] = serde_json::json!(1);
    }

    let masks = build_masks(ctx, layer, unsupported)?;
    if !masks.is_empty() {
        out["masksProperties"] = serde_json::Value::Array(masks);
    }

    for effect in view.effects(layer)? {
        unsupported.push(effect_unsupported(layer, &effect));
    }

    if attrs.pinned {
        unsupported.push(UnsupportedForLottie {
            layer: Some(layer),
            category: "pinned",
            detail: "LayerAttrs.pinned(カメラ非追従、裁定113)に対応する Lottie 語彙が無い"
                .to_owned(),
        });
    }

    let time_remap = PropertyId::new(property::TIME_REMAP)?;
    if view.property_source(layer, &time_remap)?.is_some() {
        unsupported.push(UnsupportedForLottie {
            layer: Some(layer),
            category: "time-remap",
            detail: "`property::TIME_REMAP` が使われているが、Lottie の `tm` は \
                     precomposition-layer 専用で、Motolii に対応する LayerSource が無い"
                .to_owned(),
        });
    }

    match &meta.source {
        LayerSource::Solid { rgba, width, height } => {
            out["ty"] = serde_json::json!(1);
            out["sw"] = serde_json::json!(width);
            out["sh"] = serde_json::json!(height);
            out["sc"] = serde_json::json!(rgb_hex(rgba));
            if rgba[3] != 255 {
                unsupported.push(UnsupportedForLottie {
                    layer: Some(layer),
                    category: "solid-alpha",
                    detail: format!(
                        "solid の alpha={} だが Lottie の solid-layer(`sc`)は \
                         `#RRGGBB` のみで alpha を運べない — 不透明として書いた",
                        rgba[3]
                    ),
                });
            }
        }
        LayerSource::File { path, .. } if is_point_cloud_path(path) => {
            out["ty"] = serde_json::json!(3);
            unsupported.push(UnsupportedForLottie {
                layer: Some(layer),
                category: "point-cloud",
                detail: format!(
                    "点群 layer({path})に対応する Lottie layer type が無いため \
                     Null layer(`ty: 3`)として書いた——絵は出ない"
                ),
            });
        }
        LayerSource::File { path, .. } => {
            let (ty, asset) = build_media_asset(layer, path, unsupported);
            let ref_id = asset["id"].as_str().unwrap().to_owned();
            assets.push(asset);
            out["ty"] = serde_json::json!(ty);
            out["refId"] = serde_json::json!(ref_id);
            if ty == 6 {
                out["au"] = serde_json::json!({});
                check_audio_settings_unsupported(view, layer, unsupported)?;
            }
        }
        LayerSource::Null => {
            out["ty"] = serde_json::json!(3);
        }
        LayerSource::Group => {
            out["ty"] = serde_json::json!(3);
        }
        LayerSource::Shape => {
            out["ty"] = serde_json::json!(4);
            let shapes = view.shapes(layer)?;
            out["shapes"] = serde_json::Value::Array(
                shapes.iter().map(shape_node_to_json).collect::<Vec<_>>(),
            );
        }
        LayerSource::Text => {
            out["ty"] = serde_json::json!(5);
            out["t"] = build_text_data(ctx, layer, unsupported)?;
        }
    }

    Ok(out)
}

fn effect_unsupported(layer: LayerId, effect: &EffectInstance) -> UnsupportedForLottie {
    UnsupportedForLottie {
        layer: Some(layer),
        category: "effect",
        detail: format!(
            "effect `{}`(id={}): plugin_id は Motolii の拡張名前空間の文字列であって \
             Lottie の `ty`(組込み effect の数値 id)ではないため、対応する数値へ \
             機械的に写せない(裁定70: 閉じた int registry にしない設計そのものが \
             ここでは export の壁になる)",
            effect.plugin_id, effect.id
        ),
    }
}

pub(crate) fn report_out_of_range(
    unsupported: &mut Vec<UnsupportedForLottie>,
    layer: Option<LayerId>,
    field: &str,
    value: f64,
    bounds: (f64, f64),
) {
    if value < bounds.0 || value > bounds.1 {
        unsupported.push(UnsupportedForLottie {
            layer,
            category: "value-out-of-range",
            detail: format!(
                "`{field}` に Lottie の有効域 [{}, {}] を外れた値 {value} が焼かれた \
                 (加算 modulator の和・またはベジェイージングの overshoot——store は \
                 意図して clamp しない設計、`slot.rs` doc 参照)。値はそのまま書いた \
                 ——黙って clamp すると裁定206 の基準を測れなくなるため報告する",
                bounds.0, bounds.1
            ),
        });
    }
}

fn check_audio_settings_unsupported(
    view: &StoreView<'_>,
    layer: LayerId,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<(), LottieExportError> {
    let names = [
        property::LEVEL,
        property::PAN,
        property::FADE_IN,
        property::FADE_OUT,
    ];
    let mut used = Vec::new();
    for name in names {
        let id = PropertyId::new(name)?;
        if view.property_source(layer, &id)?.is_some() {
            used.push(name);
        }
    }
    if !used.is_empty() {
        unsupported.push(UnsupportedForLottie {
            layer: Some(layer),
            category: "audio-settings",
            detail: format!(
                "property {used:?} が使われているが、`au`(audio-settings)へ写す \
                 語彙をまだ実装していない(空の `au` を書いた)"
            ),
        });
    }
    Ok(())
}

fn rgb_hex(rgba: &[u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2])
}

fn build_media_asset(
    layer: LayerId,
    path: &str,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> (u8, serde_json::Value) {
    const AUDIO_EXT: &[&str] = &["mp3", "wav", "aac", "flac", "ogg", "m4a", "opus"];
    const IMAGE_OR_VIDEO_EXT: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "tga", "exr", "tiff", "tif", "mp4", "mov",
        "webm", "mkv", "avi", "gif",
    ];
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    let ty = if AUDIO_EXT.contains(&ext.as_str()) {
        6u8
    } else {
        if !IMAGE_OR_VIDEO_EXT.contains(&ext.as_str()) {
            unsupported.push(UnsupportedForLottie {
                layer: Some(layer),
                category: "media-kind",
                detail: format!(
                    "path `{path}` の拡張子 `{ext}` から image/video/audio を判定できない \
                     (LayerSource::File は種別を型で持たないので拡張子で当てるしかない) \
                     — image-layer(ty=2)へ倒した"
                ),
            });
        }
        2u8
    };

    let id = format!("media_{}", layer.0);
    let file_name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_owned();
    let asset = serde_json::json!({
        "id": id,
        "u": "",
        "p": file_name,
        "e": 0,
    });
    (ty, asset)
}

fn build_transform(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    Ok(serde_json::json!({
        "a": vector_property(ctx, layer, property::ANCHOR, 1.0, [0.0, 0.0], unsupported)?,
        "p": build_position(ctx, layer, unsupported)?,
        "s": vector_property(ctx, layer, property::SCALE, 100.0, [100.0, 100.0], unsupported)?,
        "r": scalar_property(ctx, layer, property::ROTATION, 1.0, 0.0, None, unsupported)?,
        "o": scalar_property(ctx, layer, property::OPACITY, 100.0, 100.0, Some((0.0, 100.0)), unsupported)?,
        "sk": scalar_property(ctx, layer, property::SKEW, 1.0, 0.0, None, unsupported)?,
        "sa": scalar_property(ctx, layer, property::SKEW_AXIS, 1.0, 0.0, None, unsupported)?,
    }))
}

fn build_position(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    let position = PropertyId::new(property::POSITION)?;
    if ctx.view.property_source(layer, &position)?.is_some() {
        return vector_property(ctx, layer, property::POSITION, 1.0, [0.0, 0.0], unsupported);
    }
    let x_id = PropertyId::new(property::POSITION_X)?;
    let y_id = PropertyId::new(property::POSITION_Y)?;
    let has_split = ctx.view.property_source(layer, &x_id)?.is_some()
        || ctx.view.property_source(layer, &y_id)?.is_some();
    if !has_split {
        return vector_property(ctx, layer, property::POSITION, 1.0, [0.0, 0.0], unsupported);
    }
    let x = scalar_property(ctx, layer, property::POSITION_X, 1.0, 0.0, None, unsupported)?;
    let y = scalar_property(ctx, layer, property::POSITION_Y, 1.0, 0.0, None, unsupported)?;
    Ok(serde_json::json!({ "s": true, "x": x, "y": y }))
}
