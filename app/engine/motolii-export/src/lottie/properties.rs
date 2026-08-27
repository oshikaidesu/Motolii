//! Lottie の property/track 焼き込み — `scalar_property`/`vector_property`/
//! `bezier_property`(出処の解決)・`bake_property`(link のサンプル焼き)・
//! `encode_*_track`(キーフレーム列の JSON 化)・mask/marker/slot の JSON 化。
//! `next/engine/motolii-export/src/lottie.rs` から移送(SP-7、2026-08-23、
//! 中身は変えていない——移送のみ)。呼び手は `super`(`build_layer`/
//! `export_lottie`)と `super::text`(`build_text_data`)。

use motolii_core::RationalTime;
use motolii_store::{
    LayerId, Mask, Marker, Path as BezierPath, PropertyBase, PropertyId, PropertySource, Slot,
    SlotId, Value,
};

use super::enums::mask_mode_to_str;
use super::{report_out_of_range, Ctx, LottieExportError, UnsupportedForLottie};

/// property の出処(Track/Slot/Link)を Lottie の scalar-property JSON へ。
/// `scale` は Lottie の慣習(opacity/scale は 0..100)への換算係数。`bounds` は
/// **スケール後の単位**での Lottie 有効域(`Some` の property だけ検査する
/// ——opacity 系のみ、rotation/skew/expansion 等は無制限)、外れたら
/// [`report_out_of_range`] が `unsupported` へ積む(値はそのまま書く、
/// 裁定213/`slot.rs` doc「範囲外に出た時」参照)。
pub(crate) fn scalar_property(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    name: &str,
    scale: f64,
    default: f64,
    bounds: Option<(f64, f64)>,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    match resolve(ctx, layer, name, unsupported)? {
        Resolved::None => Ok(serde_json::json!({ "a": 0, "k": default })),
        Resolved::SlotRef(sid) => Ok(serde_json::json!({ "sid": sid })),
        Resolved::Track(track) => {
            encode_scalar_track(ctx, Some(layer), name, &track, scale, bounds, unsupported)
        }
    }
}

pub(crate) fn vector_property(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    name: &str,
    scale: f64,
    default: [f64; 2],
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    match resolve(ctx, layer, name, unsupported)? {
        Resolved::None => Ok(serde_json::json!({ "a": 0, "k": [default[0], default[1]] })),
        Resolved::SlotRef(sid) => Ok(serde_json::json!({ "sid": sid })),
        Resolved::Track(track) => encode_vector_track(ctx, name, &track, scale, /* spatial */ false),
    }
}

/// mask/`p`(shape)専用。位置に spatial tangent(`ti`/`to`)が乗る唯一の property なので
/// [`vector_property`] とは分けてある——実際には position のみが該当するが、mask 形状は
/// bezier 専用口([`bezier_property`])を使うのでここは position 用の分岐そのまま。
fn scalar_property_percent0_100(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    name: &str,
    default: f64,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    // mask opacity も layer opacity と同じ有効域 0..100(% 換算後)。
    scalar_property(ctx, layer, name, 100.0, default, Some((0.0, 100.0)), unsupported)
}

fn bezier_property(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    name: &str,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    match resolve(ctx, layer, name, unsupported)? {
        Resolved::None => Ok(serde_json::json!({ "a": 0, "k": bezier_to_json(&BezierPath::default()) })),
        Resolved::SlotRef(sid) => Ok(serde_json::json!({ "sid": sid })),
        Resolved::Track(track) => encode_bezier_track(ctx, name, &track),
    }
}

enum Resolved {
    None,
    SlotRef(String),
    Track(motolii_store::KeyframeTrack),
}

/// property の出処を読み、`Link` はその場で**焼く**(裁定206 の実地検証、モジュール
/// doc 参照)。`Track`/`Slot` は素通し。
fn resolve(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    name: &str,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<Resolved, LottieExportError> {
    let property = PropertyId::new(name)?;
    match ctx.view.property_source(layer, &property)? {
        None => Ok(Resolved::None),
        // modulator が無ければ今までどおり base を素通しする(裁定213で
        // `PropertySource` が enum から base+modulators の struct へ変わっただけで、
        // base 無し・modulator 無しの組み合わせは元々作れない=旧 `None` 相当)。
        Some(PropertySource {
            base,
            modulators,
        }) if modulators.is_empty() => match base {
            None => Ok(Resolved::None),
            Some(PropertyBase::Track(track)) => Ok(Resolved::Track(track)),
            Some(PropertyBase::Slot(SlotId(id))) => Ok(Resolved::SlotRef(id)),
        },
        // modulator が1本でもあれば(旧 `Link` 相当の base無し1本も、base+modulator
        // の和も)その場で**焼く**(裁定206 の実地検証、モジュール doc 参照) ——
        // 焼く経路は `StoreView::value_at` を直接サンプルするので、base の有無や
        // modulator の本数を問わず同じ1本の経路で正しい。
        Some(_) => {
            let baked = bake_property(ctx, layer, &property)?;
            let _ = unsupported; // link は焼けるので unsupported に積まない(裁定206)
            Ok(Resolved::Track(baked))
        }
    }
}

/// **link を焼く**——`StoreView::value_at`(link/slot 解決込みの評価器そのもの)を
/// フレーム単位でサンプルし、値が変わった時だけ Hold キーフレームを打つ普通の
/// `KeyframeTrack` を組み立てる。焼いた後は [`encode_scalar_track`] 等、Track と
/// 全く同じ経路を通る——「焼けば KeyframeTrack と区別がつかない」という裁定206 の
/// 主張をコードで実演する関数。
pub(crate) fn bake_property(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    property: &PropertyId,
) -> Result<motolii_store::KeyframeTrack, LottieExportError> {
    use motolii_store::{Interp, Keyframe};
    let mut track = motolii_store::KeyframeTrack::new();
    let mut last: Option<Value> = None;
    for frame in 0..ctx.duration_frames.max(1) {
        let t = RationalTime::try_from_frame(frame, ctx.fps)?;
        let value = ctx.view.value_at(layer, property, t)?;
        let Some(value) = value else { continue };
        if last.as_ref() != Some(&value) {
            track.insert(Keyframe {
                t,
                value: value.clone(),
                interp: Interp::Hold,
                spatial: None,
            });
            last = Some(value);
        }
    }
    if track.keys().is_empty() {
        // 一度も値が取れなかった(link 先が終始無値) — 0.0 を1本だけ持たせて、
        // 呼び手側の encode_* が空トラックの特別扱いを要らないようにする。
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.0),
            interp: Interp::Hold,
            spatial: None,
        });
    }
    Ok(track)
}

fn time_to_frame(ctx: &Ctx<'_, '_>, t: RationalTime) -> Result<f64, LottieExportError> {
    Ok(t.try_to_frame_round(ctx.fps)? as f64)
}

fn interp_easing(interp: motolii_store::Interp) -> Option<(serde_json::Value, serde_json::Value)> {
    use motolii_store::Interp;
    match interp {
        Interp::Hold => None,
        Interp::Linear => Some((
            serde_json::json!({ "x": [0.0], "y": [0.0] }),
            serde_json::json!({ "x": [1.0], "y": [1.0] }),
        )),
        Interp::Bezier { x1, y1, x2, y2 } => Some((
            serde_json::json!({ "x": [x1], "y": [y1] }),
            serde_json::json!({ "x": [x2], "y": [y2] }),
        )),
    }
}

/// `bounds`(スケール後の単位)が `Some` なら、書く各値をその場で検査し、
/// 外れていれば [`report_out_of_range`] へ積む(値そのものは clamp せず
/// そのまま書く——`slot.rs` doc「範囲外に出た時」参照)。`layer` は
/// `UnsupportedForLottie::layer`(comp 単位の呼び手 = slot は `None`)。
pub(crate) fn encode_scalar_track(
    ctx: &Ctx<'_, '_>,
    layer: Option<LayerId>,
    name: &str,
    track: &motolii_store::KeyframeTrack,
    scale: f64,
    bounds: Option<(f64, f64)>,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    let keys = track.keys();
    if keys.len() <= 1 {
        let v = keys
            .first()
            .map(|k| &k.value)
            .and_then(Value::as_f64)
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), keys[0].value.clone()))?;
        let scaled = v * scale;
        if let Some(bounds) = bounds {
            report_out_of_range(unsupported, layer, name, scaled, bounds);
        }
        return Ok(serde_json::json!({ "a": 0, "k": scaled }));
    }
    let mut out = Vec::with_capacity(keys.len());
    for (i, key) in keys.iter().enumerate() {
        let v = key
            .value
            .as_f64()
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), key.value.clone()))?;
        let scaled = v * scale;
        if let Some(bounds) = bounds {
            report_out_of_range(unsupported, layer, name, scaled, bounds);
        }
        let mut obj = serde_json::json!({
            "t": time_to_frame(ctx, key.t)?,
            "s": [scaled],
        });
        if i + 1 < keys.len() {
            match interp_easing(key.interp) {
                None => obj["h"] = serde_json::json!(1),
                Some((o, in_)) => {
                    obj["o"] = o;
                    obj["i"] = in_;
                }
            }
        }
        out.push(obj);
    }
    Ok(serde_json::json!({ "a": 1, "k": out }))
}

fn encode_vector_track(
    ctx: &Ctx<'_, '_>,
    name: &str,
    track: &motolii_store::KeyframeTrack,
    scale: f64,
    with_spatial: bool,
) -> Result<serde_json::Value, LottieExportError> {
    let keys = track.keys();
    if keys.len() <= 1 {
        let v = keys
            .first()
            .map(|k| &k.value)
            .and_then(Value::as_vec2)
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), keys[0].value.clone()))?;
        return Ok(serde_json::json!({ "a": 0, "k": [v[0] * scale, v[1] * scale] }));
    }
    let mut out = Vec::with_capacity(keys.len());
    for (i, key) in keys.iter().enumerate() {
        let v = key
            .value
            .as_vec2()
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), key.value.clone()))?;
        let mut obj = serde_json::json!({
            "t": time_to_frame(ctx, key.t)?,
            "s": [v[0] * scale, v[1] * scale],
        });
        if with_spatial {
            if let Some(spatial) = &key.spatial {
                obj["to"] = serde_json::json!(spatial.out_tangent);
                obj["ti"] = serde_json::json!(spatial.in_tangent);
            }
        }
        if i + 1 < keys.len() {
            match interp_easing(key.interp) {
                None => obj["h"] = serde_json::json!(1),
                Some((o, in_)) => {
                    obj["o"] = o;
                    obj["i"] = in_;
                }
            }
        }
        out.push(obj);
    }
    Ok(serde_json::json!({ "a": 1, "k": out }))
}

fn encode_bezier_track(
    ctx: &Ctx<'_, '_>,
    name: &str,
    track: &motolii_store::KeyframeTrack,
) -> Result<serde_json::Value, LottieExportError> {
    let keys = track.keys();
    if keys.len() <= 1 {
        let p = keys
            .first()
            .map(|k| &k.value)
            .and_then(Value::as_path)
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), keys[0].value.clone()))?;
        return Ok(serde_json::json!({ "a": 0, "k": bezier_to_json(p) }));
    }
    let mut out = Vec::with_capacity(keys.len());
    for (i, key) in keys.iter().enumerate() {
        let p = key
            .value
            .as_path()
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), key.value.clone()))?;
        let mut obj = serde_json::json!({
            "t": time_to_frame(ctx, key.t)?,
            "s": [bezier_to_json(p)],
        });
        if i + 1 < keys.len() {
            match interp_easing(key.interp) {
                None => obj["h"] = serde_json::json!(1),
                Some((o, in_)) => {
                    obj["o"] = o;
                    obj["i"] = in_;
                }
            }
        }
        out.push(obj);
    }
    Ok(serde_json::json!({ "a": 1, "k": out }))
}

fn bezier_to_json(path: &BezierPath) -> serde_json::Value {
    let v: Vec<[f64; 2]> = path.vertices.iter().map(|p| p.point).collect();
    let i: Vec<[f64; 2]> = path.vertices.iter().map(|p| p.in_tangent).collect();
    let o: Vec<[f64; 2]> = path.vertices.iter().map(|p| p.out_tangent).collect();
    serde_json::json!({ "c": path.closed, "v": v, "i": i, "o": o })
}

// ---------------------------------------------------------------------------
// masks
// ---------------------------------------------------------------------------

pub(crate) fn build_masks(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<Vec<serde_json::Value>, LottieExportError> {
    let mut out = Vec::new();
    for mask in ctx.view.masks(layer)? {
        out.push(build_mask(ctx, layer, &mask, unsupported)?);
    }
    Ok(out)
}

fn build_mask(
    ctx: &Ctx<'_, '_>,
    layer: LayerId,
    mask: &Mask,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    let shape_id = PropertyId::mask_shape(mask.id).name().to_owned();
    let opacity_id = PropertyId::mask_opacity(mask.id).name().to_owned();
    let expansion_id = PropertyId::mask_expansion(mask.id).name().to_owned();

    Ok(serde_json::json!({
        "inv": mask.inverted,
        "mode": mask_mode_to_str(mask.mode),
        "pt": bezier_property(ctx, layer, &shape_id, unsupported)?,
        "o": scalar_property_percent0_100(ctx, layer, &opacity_id, 100.0, unsupported)?,
        "x": scalar_property(ctx, layer, &expansion_id, 1.0, 0.0, None, unsupported)?,
    }))
}

// ---------------------------------------------------------------------------
// markers / slots
// ---------------------------------------------------------------------------

pub(crate) fn build_markers(ctx: &Ctx<'_, '_>) -> Result<Vec<serde_json::Value>, LottieExportError> {
    let mut out = Vec::new();
    for marker in ctx.view.markers()? {
        out.push(marker_to_json(ctx, &marker)?);
    }
    Ok(out)
}

fn marker_to_json(ctx: &Ctx<'_, '_>, marker: &Marker) -> Result<serde_json::Value, LottieExportError> {
    Ok(serde_json::json!({
        "cm": marker.name,
        "tm": time_to_frame(ctx, marker.time)?,
        "dr": (marker.duration.try_to_frame_round(ctx.fps)?) as f64,
    }))
}

/// comp の Slots 表。値の型が「静止/線形補間で意味を持つ」もの(F64/Vec2/Color/Path)
/// だけを書く——Bool/Enum/LayerId 値のスロットは Lottie のスロット型に対応する
/// 語彙が無い(スロットは property の**値**をそのまま差し替えるだけなので、値の型が
/// Lottie property として書けない場合はスロット自体も書けない)。
pub(crate) fn build_slots(
    ctx: &Ctx<'_, '_>,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Map<String, serde_json::Value>, LottieExportError> {
    let mut out = serde_json::Map::new();
    for slot in ctx.view.slots()? {
        match slot_property_value(ctx, &slot, unsupported) {
            Ok(value) => {
                out.insert(slot.id.0.clone(), serde_json::json!({ "p": value }));
            }
            Err(reason) => unsupported.push(UnsupportedForLottie {
                layer: None,
                category: "slot",
                detail: format!("slot `{}`: {reason}", slot.id.0),
            }),
        }
    }
    Ok(out)
}

fn slot_property_value(
    ctx: &Ctx<'_, '_>,
    slot: &Slot,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, String> {
    let keys = slot.track.keys();
    let first = keys.first().ok_or("track が空(値が無い)")?;
    match &first.value {
        Value::F64(_) => encode_scalar_track(ctx, None, "slot", &slot.track, 1.0, None, unsupported)
            .map_err(|e| e.to_string()),
        Value::Vec2(_) => encode_vector_track(ctx, "slot", &slot.track, 1.0, false)
            .map_err(|e| e.to_string()),
        Value::Color(color) => {
            let _ = color;
            encode_color_track(ctx, None, "slot", &slot.track, unsupported).map_err(|e| e.to_string())
        }
        Value::Path(_) => encode_bezier_track(ctx, "slot", &slot.track).map_err(|e| e.to_string()),
        other => Err(format!(
            "値の型 {other:?} は Lottie のスロットに書ける型(F64/Vec2/Color/Path)ではない"
        )),
    }
}

/// 色成分(r/g/b)は Lottie/`motolii_eval::Value::Color` とも「各成分
/// 0.0–1.0」が有効域(`value.rs` の型 doc)——加算 modulator の和がそこを外れたら
/// [`report_out_of_range`] へ積む(値はそのまま clamp せず書く)。
fn encode_color_track(
    ctx: &Ctx<'_, '_>,
    layer: Option<LayerId>,
    name: &str,
    track: &motolii_store::KeyframeTrack,
    unsupported: &mut Vec<UnsupportedForLottie>,
) -> Result<serde_json::Value, LottieExportError> {
    const UNIT: (f64, f64) = (0.0, 1.0);
    let check = |unsupported: &mut Vec<UnsupportedForLottie>, c: [f64; 4]| {
        report_out_of_range(unsupported, layer, &format!("{name}.r"), c[0], UNIT);
        report_out_of_range(unsupported, layer, &format!("{name}.g"), c[1], UNIT);
        report_out_of_range(unsupported, layer, &format!("{name}.b"), c[2], UNIT);
    };
    let keys = track.keys();
    if keys.len() <= 1 {
        let c = keys
            .first()
            .map(|k| &k.value)
            .and_then(Value::as_color)
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), keys[0].value.clone()))?;
        check(unsupported, c);
        return Ok(serde_json::json!({ "a": 0, "k": [c[0], c[1], c[2]] }));
    }
    let mut out = Vec::with_capacity(keys.len());
    for (i, key) in keys.iter().enumerate() {
        let c = key
            .value
            .as_color()
            .ok_or_else(|| LottieExportError::TypeMismatch(name.to_owned(), key.value.clone()))?;
        check(unsupported, c);
        let mut obj = serde_json::json!({
            "t": time_to_frame(ctx, key.t)?,
            "s": [c[0], c[1], c[2]],
        });
        if i + 1 < keys.len() {
            match interp_easing(key.interp) {
                None => obj["h"] = serde_json::json!(1),
                Some((o, in_)) => {
                    obj["o"] = o;
                    obj["i"] = in_;
                }
            }
        }
        out.push(obj);
    }
    Ok(serde_json::json!({ "a": 1, "k": out }))
}
