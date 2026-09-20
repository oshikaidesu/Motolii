use super::*;
use motolii_edit::{Document, Intent};
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;

fn png(dir: &std::path::Path, name: &str, size: u32, rgba: [u8; 4]) -> std::path::PathBuf {
    let path = dir.join(name);
    let pixels: Vec<u8> = rgba.into_iter().cycle().take((size * size * 4) as usize).collect();
    image::save_buffer(&path, &pixels, size, size, image::ColorType::Rgba8).unwrap();
    path
}

fn pixel(frame: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * SIZE + x) * 4) as usize;
    [frame[i], frame[i + 1], frame[i + 2]]
}

/// 形の縁は下へ滑らかに溶け、**黒くならない**。乗算済みと非乗算を取り違えると、縁の画素が
/// 内側とも外側とも違う暗さに沈む(2026-09-13、実写の旗の縁で発覚)。
#[test]
fn a_blurred_copy_fades_at_its_edge_without_going_dark() {
    let dir = tempfile::tempdir().unwrap();
    let white = png(dir.path(), "white.png", SIZE, [255, 255, 255, 255]);
    let grey = png(dir.path(), "grey.png", 32, [128, 128, 128, 255]);
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    for (id, path, order, at) in [(1u64, &white, 0i16, 0.0), (2, &grey, 1, 16.0)] {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([at, at]) },
        ]).unwrap();
    }
    doc.apply_all([
        Intent::SetEffects { layer: LayerId(2), effects: vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.background_copy".into() },
            EffectInstance { id: EffectId(1), plugin_id: "motolii.blur".into() },
        ] },
        Intent::SetConstant { layer: LayerId(2), property: PropertyId::effect_param(EffectId(1), "radius").unwrap(), value: Value::F64(8.0) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let frame = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    // 白の上に「白の写し」をぼかして重ねた。中も外も白なら、縁も白でなければならない。
    let row: Vec<u8> = (8..40).map(|x| pixel(&frame, x, 32)[0]).collect();
    let darkest = row.iter().copied().min().unwrap();
    assert!(darkest >= 235, "縁が沈んでいる: x=8..40 の R = {row:?}");
}

/// 同じ縁の問いを、下を読まない普通の道(層の絵へ焼く)で。白い板を白の上でぼかす。
#[test]
fn a_blurred_layer_fades_at_its_edge_without_going_dark_on_the_baked_path() {
    let dir = tempfile::tempdir().unwrap();
    let white = png(dir.path(), "white.png", SIZE, [255, 255, 255, 255]);
    let square = png(dir.path(), "square.png", 32, [255, 255, 255, 255]);
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    for (id, path, order, at) in [(1u64, &white, 0i16, 0.0), (2, &square, 1, 16.0)] {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([at, at]) },
        ]).unwrap();
    }
    doc.apply_all([
        Intent::SetEffects { layer: LayerId(2), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
        Intent::SetConstant { layer: LayerId(2), property: PropertyId::effect_param(EffectId(0), "radius").unwrap(), value: Value::F64(8.0) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let frame = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let row: Vec<u8> = (8..40).map(|x| pixel(&frame, x, 32)[0]).collect();
    let darkest = row.iter().copied().min().unwrap();
    assert!(darkest >= 235, "焼く道でも縁が沈んでいる: x=8..40 の R = {row:?}");
}

#[test]
fn background_copy_then_gain_brightens_what_is_below_and_hides_the_layer() {
    let dir = tempfile::tempdir().unwrap();
    let blue = png(dir.path(), "blue.png", SIZE, [0, 0, 100, 255]);
    let red = png(dir.path(), "red.png", 32, [255, 0, 0, 255]);
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    for (id, path, order, at) in [(1u64, &blue, 0i16, 0.0), (2, &red, 1, 16.0)] {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([at, at]) },
        ]).unwrap();
    }
    doc.apply_all([
        Intent::SetEffects { layer: LayerId(2), effects: vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.background_copy".into() },
            EffectInstance { id: EffectId(1), plugin_id: "motolii.gain".into() },
        ] },
        Intent::SetConstant { layer: LayerId(2), property: PropertyId::effect_param(EffectId(1), "gain").unwrap(), value: Value::F64(2.0) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let frame = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let outside = pixel(&frame, 4, 4);
    let inside = pixel(&frame, 32, 32);
    assert!(outside[2] > 60 && outside[0] < 10, "外は青のまま: {outside:?}");
    assert!(inside[0] < 10, "赤は消える(自分の絵は出ない): {inside:?}");
    // gain は線形で 2 倍(列の規約)。sRGB の 100 は線形 0.127 → 0.254 → sRGB ≈ 139。
    assert!(inside[2] > outside[2] + 20, "中は下の青を線形で 2 倍にした青: 中 {inside:?} 外 {outside:?}");
}
