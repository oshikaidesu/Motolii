use super::*;
use motolii_edit::{Document, Intent};
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;

fn document(path: &std::path::Path, radius: f64) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([16.0, 16.0]) },
        Intent::SetConstant { layer, property: PropertyId::new(property::SCALE).unwrap(), value: Value::Vec2([16.0, 16.0]) },
        Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.blur".into() }] },
        Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "radius").unwrap(), value: Value::F64(radius) },
    ]).unwrap();
    doc
}

fn drawn(frame: &[u8], x: u32, y: u32) -> u8 {
    let i = ((y * SIZE + x) * 4) as usize;
    frame[i..i + 3].iter().copied().max().unwrap()
}

#[test]
fn a_blur_on_a_mesh_bleeds_past_its_edge() {
    let dir = tempfile::tempdir().unwrap();
    let obj = dir.path().join("quad.obj");
    // 32×32 を (16,16) に置くので、絵は x,y が 16..48。外側の 8 px は素の網なら真っ黒。
    std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 0 -1\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
    let mut engine = Engine::new().unwrap();

    let sharp = engine.render_frame(&document(&obj, 0.0).view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    assert!(drawn(&sharp, 32, 32) > 0, "網は描かれている");
    assert_eq!(drawn(&sharp, 32, 10), 0, "ぼかさなければ縁の外は黒");

    let soft = engine.render_frame(&document(&obj, 12.0).view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    assert!(drawn(&soft, 32, 10) > 0, "ブラーが網の縁の外へ滲む");
}
