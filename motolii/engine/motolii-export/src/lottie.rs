//! Document → Lottie JSON 書き出し。
//!
//! **なぜこれが要るか**(`next/DECISIONS.md` 裁定191/裁定206): 裁定191 は Lottie
//! スキーマを「意味の正本」に置いたが、裁定206 はその先へ進んで製品の基準を
//! 「**その機構の結果を Lottie へ無損失で書けるか**」に置いた。書き出す経路が無いと、
//! この基準は口だけの標語で終わる——本モジュールはそれを `cargo test` で機械的に
//! 判定できる形にする。
//!
//! ## スコープ(`next/reference/lottie-coverage.tsv` の**採用済**集合が対象)
//!
//! 書く対象は採用済語彙だけであって Lottie 全体ではない(発注書どおり)。落ちる物は
//! **黙って落とさない** — [`UnsupportedForLottie`] に理由つきで積む。既存資産の調査
//! (RETURN 参照): `next/` に Lottie **読み込み**側は無い(逆写像を使う近道は無かった)。
//! crates.io の `lottie-data`(Bodymovin JSON 用の型定義 crate)も調べたが、
//! (a) `Transform` に `sk`/`sa`(skew、裁定58 で採用済)が無い、(b) `text` 系が
//! 旧世代の animator スキーマで、この地図が採用した `text-range`/`text-style` 系
//! (`next/reference/lottie.schema.json` そのまま)と噛み合わない、という2点で
//! 実地に不適合が見つかったため使わず、スキーマを直接引ける `serde_json::Value` で
//! 自前構築する(発注書「cargo tree と crates.io を確認してから自前定義を決める」
//! の検討結果)。
//!
//! ## 裁定206 の実地検証(このモジュールの中心)
//!
//! [`PropertySource::Link`](motolii_store::PropertySource) と `LayerAttrs::matte`
//! (裁定66)は、どちらも「Motolii の編集機構としては Lottie の素の語彙と違う形」
//! だが、**評価結果は普通の Lottie 語彙で書ける**という裁定206 の主張そのものの
//! 実例になっている:
//! - **link**: [`bake_property`] が `StoreView::value_at`(評価器そのもの)を
//!   フレーム単位でサンプルし、変化した時だけ Hold キーフレームを打つ普通の
//!   `KeyframeTrack` へ焼く。焼いた後は Track と区別が付かない一本の経路を通る。
//! - **matte**: `LayerAttrs.matte`(1フィールド)は Lottie の `tt`/`tp`/`td`
//!   3フィールドへ**明示展開**して書く(裁定66 が拒んだのは「`tp` 省略時の暗黙
//!   隣接参照」という Motolii 内部の意味論であって、`tp` を明示すること自体は
//!   拒んでいない——スキーマの `tp` の note が「省略時は1つ上の layer」と書いており、
//!   常に明示すれば暗黙参照には一度も頼らない)。
//!
//! ## 対応外(構造的に Lottie 語彙が無い)
//!
//! カメラ(`property::CAMERA_*`)は `layers/camera-layer` が **不採用**
//! (`lottie-coverage.tsv` 行229-231、裁定65: 3D 系は不採用)——Motolii のカメラは
//! そもそも Lottie に対応する語彙を持たない。カメラ track が実在する Document は
//! [`UnsupportedForLottie`] を1件返す(comp 単位、layer 非依存)。

use std::collections::HashSet;

use motolii_core::{Fps, RationalTimeError};
use motolii_store::{
    property, EffectInstance, LayerAttrs, LayerId, LayerMeta, LayerSource, PropertyId, StoreError,
    StoreView, Value,
};

/// Lottie の property/track 焼き込み(`scalar_property`/`vector_property`/
/// `bake_property`/mask・marker・slot の JSON 化)。SP-7(2026-08-23)で
/// `lottie.rs` から移送——module doc(`lottie/properties.rs`)参照。
mod properties;
/// Lottie の shape 語彙の焼き込み。SP-7(2026-08-23)で `lottie.rs` から移送——
/// module doc(`lottie/shapes.rs`)参照。
mod shapes;
/// Lottie の text 語彙の焼き込み。SP-7(2026-08-23)で `lottie.rs` から移送——
/// module doc(`lottie/text.rs`)参照。
mod text;
/// 語彙の変換(BlendMode/MatteMode/MaskMode/shape 系 enum → Lottie の整数コード)。
/// SP-7(2026-08-23)で `lottie.rs` から移送——module doc(`lottie/enums.rs`)参照。
mod enums;

use enums::{blend_mode_to_int, matte_mode_to_int};
use properties::{build_markers, build_masks, build_slots, scalar_property, vector_property};
use shapes::shape_node_to_json;
use text::build_text_data;

/// Lottie へ書けなかった物の一覧。**黙って落とさない**——空であることが
/// 「裁定206 の基準を満たす」ことの機械的な証拠になる(発注書の中心)。
#[derive(Debug, Clone, PartialEq)]
pub struct UnsupportedForLottie {
    /// どの layer の話か。comp 全体に関わる話(camera 等)は `None`。
    pub layer: Option<LayerId>,
    /// 短い分類名(grep しやすいように固定文字列にしてある)。
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
    /// Lottie が持っていない補間型(`Bounce` / `Elastic` / `Steps`)。
    ///
    /// **黙って焼かない。** Lottie のキーが持てるのはベジェの接線
    /// (`i`/`o`)と離散フラグ(`h`)だけで、パラメトリックなバウンス/バネ/
    /// 段階移動はどれにも当たらない。ベジェへ近似すれば書けはするが、
    /// それは**不可逆な劣化を無告知で行う**ことであり、`unsupported` へ
    /// 積んで「無損失で書けた」の証拠を汚すのとも意味が違う
    /// (どちらが正しいかは利用者の裁定待ち)。決まるまでは失敗する —
    /// `motolii-store` の速度積算が Bezier 区間で `Err` を返し
    /// 「黙って近似しない」と書いてあるのと同じ規律。
    #[error(
        "補間型 `{0}` は Lottie に写せない(ベジェへ焼くか拡張として持つかが未決 —\
         `docs/reviews/2026-08-28-current-position.md` の裁定待ち)"
    )]
    UnrepresentableEasing(&'static str),
}

/// [`export_lottie`] の結果。`unsupported` が空なら、この Document の採用済語彙は
/// **全部** Lottie へ無損失で書けた、という機械的な証拠になる。
pub struct LottieExport {
    pub json: serde_json::Value,
    pub unsupported: Vec<UnsupportedForLottie>,
}

struct Ctx<'a, 'b> {
    view: &'a StoreView<'b>,
    fps: Fps,
    duration_frames: i64,
}

/// Document(の現在の edit 時点)を Lottie JSON へ書き出す。
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

    // matte の参照先(source)集合。**参照される側**に `td:1` を立てるための下ごしらえ
    // (2パス目を避けるため先に1回だけ全 layer の attrs を読む)。
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

/// カメラは Lottie に対応する語彙が無い(`layers/camera-layer` 不採用、裁定65)。
/// カメラ property が一度でも触られていれば comp 単位の unsupported を1件積む。
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

// ---------------------------------------------------------------------------
// layer
// ---------------------------------------------------------------------------

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

    // `layers/precomposition-layer/tm`(Time Remap)相当。Motolii は
    // `property::TIME_REMAP` をレイヤ種別を問わず一般化しているが(view.rs 参照)、
    // Lottie の `tm` は precomposition-layer 専用フィールド——Motolii に precomp
    // という LayerSource variant が無い(Group は Null layer へ寄せている、上記参照)
    // ため、書ける layer 型が無い。
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
        LayerSource::Media { path, .. } => {
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
            // Lottie に「印だけの group layer」という variant は無い。裁定173(c)の
            // 「絵を持たない印」に一番近い実在の語彙は Null layer(絵を持たず
            // transform だけ持つ、`layers/null-layer` 採用済)なので、それへ寄せる。
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
        LayerSource::PointCloud { path, .. } => {
            // Lottie は 3D 点群という概念自体を持たない(ベクタ/ラスタのみ)——
            // `Group` が Null(`ty: 3`)へ寄せるのと同じ形で書くが、こちらは
            // 「絵を持たない印」ではなく本来絵を持つ layer が欠落するので、
            // `Group` と違い明示的に `unsupported` へ積む(黙って空にしない)。
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

/// **加算接続子(裁定213)の範囲外検出**——`motolii-store`/`motolii-eval` は
/// 意図して clamp しない(`slot.rs` モジュール doc「2026-08-23」節「1. 範囲外に
/// 出た時」— `Value::lerp` の `Color` 実装も clamp していないので `add` だけ
/// 特別扱いすると一貫しない挙動になる、という判断)。厳しい側(拒否・報告)は
/// export 境界の仕事として明示的にここへ持ち込む——**黙って clamp して出さない**
/// (裁定206 の基準を測る道具としての価値が消えるため)。`effect_unsupported`/
/// `check_audio_settings_unsupported` と同じ「報告はするが値はそのまま書く」
/// 作法(呼び手は clamp 済みの値を渡さない——このまま Lottie の JSON へ焼く)。
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

/// `layers/audio-settings lv`(Level)相当。`property::LEVEL`/`PAN`/`FADE_IN`/
/// `FADE_OUT` は `motolii-audio` が実際に mix する層単位の property だが、
/// Lottie の `au`(Audio Settings)へ写す語彙をまだ持たない——**空の `au` を
/// 書いて黙って落とす**ことは避け、実際に触られている物があれば報告する。
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

/// solid の RGBA を Lottie の `#RRGGBB` へ(alpha は Lottie の solid-layer に無いので
/// 落ちる — solid layer の透明度は別途 layer の `o`(opacity)/mask で表現すること)。
fn rgb_hex(rgba: &[u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2])
}

/// Media 由来の Lottie asset を1つ作る。ファイル種別は拡張子で当てるしかない
/// (`LayerSource::Media` は image/video/audio を型で区別しない、地図の note どおり) —
/// 未知の拡張子は image 扱いへフォールバックしつつ、判定できないことを報告する。
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
                     (LayerSource::Media は種別を型で持たないので拡張子で当てるしかない) \
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

// ---------------------------------------------------------------------------
// transform
// ---------------------------------------------------------------------------

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
        // opacity は Lottie の有効域 0..100(% 換算後)——加算 modulator の和が
        // それを外れたら報告する(`report_out_of_range` doc 参照)。
        "o": scalar_property(ctx, layer, property::OPACITY, 100.0, 100.0, Some((0.0, 100.0)), unsupported)?,
        "sk": scalar_property(ctx, layer, property::SKEW, 1.0, 0.0, None, unsupported)?,
        "sa": scalar_property(ctx, layer, property::SKEW_AXIS, 1.0, 0.0, None, unsupported)?,
    }))
}

/// `position`(単一 Vec2)を優先し、無ければ split(x/y 別 track)を試す(裁定61 と
/// 同じ優先順位、`StoreView::resolve_position` を export 版に書き直したもの)。
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

