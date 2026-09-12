//! 効果の札の見本。席ごとに 1 つの小さな書類を組み、Stage と同じ Engine で 1 フレーム描く。
//! 全部の札が同じ見本なので、差は隣の札と見比べて読める。作者の絵(manifest の `THUMBNAIL`)はそのまま返す。
use crate::doc::store::{
    Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
    PropertyId, RationalTime, Value, property,
};
use crate::render::compositor::EffectStage;
use crate::render::engine::{EffectDescriptor, EffectThumbnail, Engine};
use super::create::{self, NewKind};
use base64::Engine as _;
use serde_json::{json, Value as J};

/// 札の寸法。見本は 3 倍で描いて縮める(px の既定値が並の大きさで効き、縁も滑らか)。
pub(crate) const WIDTH: u32 = 160;
pub(crate) const HEIGHT: u32 = 120;
const SCALE: u32 = 3;
const FPS: i64 = 30;
/// 見本は 1 秒動く。時間で効く効果(残像・窓)がその中に尾を引く。
const DURATION_FRAMES: i64 = 30;

/// 平らな席の見本: 明部(glow・radiance)・硬い縁(blur)・階調(gain)・色(blend)が 1 枚で読める絵。
fn sample_picture() -> Result<String, String> {
    let (w, h) = (112 * SCALE, 84 * SCALE);
    let mut image = image::RgbaImage::new(w, h);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let t = y as f32 / h as f32;
        let mut rgb = [0.10 + 0.25 * t, 0.12 + 0.25 * t, 0.18 + 0.28 * t];
        let (x, y) = (x / SCALE, y / SCALE);
        let (dx, dy) = (x as f32 - 34.0, y as f32 - 42.0);
        if dx * dx + dy * dy < 14.0 * 14.0 { rgb = [1.0, 0.98, 0.92]; }
        if (62..68).contains(&x) { rgb = [0.0, 0.0, 0.0]; }
        if (80..100).contains(&x) && (50..70).contains(&y) { rgb = [0.92, 0.55, 0.22]; }
        *pixel = image::Rgba([to8(rgb[0]), to8(rgb[1]), to8(rgb[2]), 255]);
    }
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image).write_to(&mut png, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    create::builtin("effect-sample-v1.png", &png.into_inner())
}

fn to8(v: f32) -> u8 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 }

fn track(from: Value, to: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe { t: RationalTime::ZERO, value: from, interp: Interp::Linear, spatial: None });
    track.insert(Keyframe { t: RationalTime::try_new(1, 1).expect("1s"), value: to, interp: Interp::Linear, spatial: None });
    track
}

/// 席の見本(効果なし)と、効果を載せる層。
pub(crate) fn document(stage: EffectStage) -> Result<(Document, LayerId), String> {
    let fps = Fps::try_new(FPS, 1).map_err(|e| e.to_string())?;
    let comp = ((WIDTH * SCALE) as f64, (HEIGHT * SCALE) as f64);
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition { width: WIDTH * SCALE, height: HEIGHT * SCALE, fps, duration_frames: DURATION_FRAMES, background: [0.0; 4] })).map_err(|e| e.to_string())?;
    let place = |layer: LayerId, order: i16, kind: NewKind| create::new_layer_intents(layer, order, 0, DURATION_FRAMES, fps, comp, kind, None);
    let subject = LayerId(1);
    let intents = match stage {
        EffectStage::Pass | EffectStage::Warp => {
            let path = sample_picture()?;
            let mut out = place(subject, 0, NewKind::Media { path, name: "Sample".into() });
            let centered = out.iter().find_map(|i| match i { Intent::SetConstant { property, value: Value::Vec2(p), .. } if property.name() == property::POSITION => Some(*p), _ => None }).unwrap_or([0.0, 0.0]);
            out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, track: track(Value::Vec2([centered[0] - 90.0, centered[1]]), Value::Vec2(centered)) });
            out
        }
        EffectStage::Placement | EffectStage::Path | EffectStage::Solid => {
            let mut out = place(subject, 0, NewKind::Star);
            if stage == EffectStage::Placement {
                // Repeater の既定は 100px 刻みで 3 つ。1 歩ぶん左へ寄せて、複製の群れが枠の中に並ぶ。
                let centered = out.iter().find_map(|i| match i { Intent::SetConstant { property, value: Value::Vec2(p), .. } if property.name() == property::POSITION => Some(*p), _ => None }).unwrap_or([0.0, 0.0]);
                out.push(Intent::SetConstant { layer: subject, property: PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, value: Value::Vec2([centered[0] - 100.0, centered[1]]) });
            }
            if stage == EffectStage::Solid {
                // 立体は傾けて側面と縁を見せる(2.5D の板のまま回すと奥行きは出ない)。
                out.push(Intent::SetAttrs { layer: subject, patch: crate::doc::store::LayerAttrsPatch { projection: Some(crate::doc::store::LayerProjection::ThreeD), ..Default::default() } });
                out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::ROTATION_Y).map_err(|e| e.to_string())?, track: track(Value::F64(-20.0), Value::F64(-50.0)) });
            }
            out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::ROTATION).map_err(|e| e.to_string())?, track: track(Value::F64(0.0), Value::F64(45.0)) });
            out
        }
        EffectStage::Surface | EffectStage::Field | EffectStage::Clip => {
            let mut out = Vec::new();
            if stage == EffectStage::Surface {
                out.extend(place(LayerId(2), 0, create::background("photo-studio")?));
            }
            out.extend(place(subject, 1, create::primitive("sphere")?));
            out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::ROTATION_Y).map_err(|e| e.to_string())?, track: track(Value::F64(0.0), Value::F64(60.0)) });
            out
        }
    };
    doc.apply_all(intents).map_err(|e| e.to_string())?;
    Ok((doc, subject))
}

/// 効果を見本に載せる。姿勢は manifest、無ければ HERO の欄を範囲の 75% に。
fn dress(doc: &mut Document, layer: LayerId, effect: &EffectDescriptor, pose: &[(String, f64)]) -> Result<(), String> {
    let id = EffectId(0);
    doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id, plugin_id: effect.plugin_id.clone() }] }).map_err(|e| e.to_string())?;
    let hero = effect.params.iter().find(|p| p.hero).and_then(|p| p.range.map(|(min, max)| (p.name.clone(), min + (max - min) * 0.75)));
    let pose: Vec<(String, f64)> = if pose.is_empty() { hero.into_iter().collect() } else { pose.to_vec() };
    for (name, value) in pose {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_param(id, &name).map_err(|e| e.to_string())?, value: Value::F64(value) }).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 札の PNG。`before` は同じ席の素の見本。
pub(crate) fn png(engine: &mut Engine, effect: &EffectDescriptor, before: bool) -> Result<Vec<u8>, String> {
    let (pose, seconds): (&[(String, f64)], f64) = match &effect.thumbnail {
        EffectThumbnail::Picture(bytes) if !before => return Ok(bytes.to_vec()),
        EffectThumbnail::Picture(_) => (&[], EffectThumbnail::DEFAULT_TIME),
        EffectThumbnail::Rendered { pose, time, .. } => (pose, *time),
    };
    let (mut doc, layer) = document(effect.stage)?;
    if !before { dress(&mut doc, layer, effect, pose)?; }
    let at = RationalTime::try_new((seconds * FPS as f64).round() as i64, FPS).map_err(|e| e.to_string())?;
    let mut pixels = engine.render_frame(&doc.view(), at).map_err(|e| e.to_string())?;
    for pixel in pixels.chunks_exact_mut(4) {
        let a = pixel[3] as u32;
        if a > 0 { for c in &mut pixel[..3] { *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8; } }
    }
    let image = image::RgbaImage::from_raw(WIDTH * SCALE, HEIGHT * SCALE, pixels).ok_or("Sample frame has the wrong size")?;
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image).thumbnail(WIDTH, HEIGHT).write_to(&mut png, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(png.into_inner())
}

/// `{"op":"visualSample","kind":"effect","id":plugin_id,"before":bool}` の返事。
pub(crate) fn reply(engine: &mut Engine, j: &J) -> Result<J, String> {
    let id = j["id"].as_str().ok_or("Missing effect id")?;
    let catalog = crate::render::engine::known_effects();
    let effect = catalog.iter().find(|d| d.plugin_id == id).ok_or("Unknown effect")?;
    let png = png(engine, effect, j["before"] == true)?;
    Ok(json!({"image": base64::engine::general_purpose::STANDARD.encode(png)}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(png: &[u8]) -> image::RgbaImage { image::load_from_memory(png).unwrap().to_rgba8() }
    fn differs(a: &image::RgbaImage, b: &image::RgbaImage) -> bool {
        // A single edge sample can change coverage when equivalent contours tessellate differently.
        a.pixels().zip(b.pixels()).filter(|(p, q)| p.0.iter().zip(q.0.iter()).any(|(x, y)| x.abs_diff(*y) > 8)).take(2).count() == 2
    }

    /// 席ごとの見本は空でなく、棚の全部の札が素の見本と違う(姿勢が効いている)。
    /// 向きを変えるだけの Reverse Path だけは見た目を持たない。作者の絵はそのまま。
    #[test]
    fn every_stage_has_a_sample_and_the_dressed_tile_differs_from_it() {
        let mut engine = Engine::new().unwrap();
        let catalog = crate::render::engine::known_effects();
        let mut unmarked = Vec::new();
        for effect in catalog.iter() {
            let stage = effect.stage;
            let plain_png = png(&mut engine, effect, true).unwrap();
            let dressed_png = png(&mut engine, effect, false).unwrap();
            // 目で見る時: MOTOLII_SAMPLE_OUT=dir で札を書き出す。
            if let Some(dir) = std::env::var_os("MOTOLII_SAMPLE_OUT") {
                let dir = std::path::PathBuf::from(dir);
                std::fs::write(dir.join(format!("{stage:?}-before.png")), &plain_png).unwrap();
                std::fs::write(dir.join(format!("{}.png", effect.plugin_id)), &dressed_png).unwrap();
            }
            let plain = decode(&plain_png);
            assert!(plain.pixels().any(|p| p[3] > 0), "{stage:?}: the sample is empty");
            let dressed = decode(&dressed_png);
            if !differs(&plain, &dressed) { unmarked.push(effect.plugin_id.as_str()); }
        }
        // 輪郭を変えない演算(向き・頂点の数・閉じた形の端)だけが素の見本のまま。
        use crate::doc::store::pathop;
        unmarked.sort();
        let mut expected = [pathop::EXTEND_PATHS, pathop::REVERSE_PATH, pathop::SUBDIVIDE];
        expected.sort();
        assert_eq!(unmarked, expected, "these effects leave no mark on the sample");
        let gain = catalog.iter().find(|d| d.plugin_id == "motolii.gain").expect("Gain is on the shelf");
        let mut pictured = gain.clone();
        pictured.thumbnail = EffectThumbnail::Picture(std::sync::Arc::from(&b"png?"[..]));
        assert_eq!(png(&mut engine, &pictured, false).unwrap(), b"png?", "the author's picture is served as is");
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    }
}
