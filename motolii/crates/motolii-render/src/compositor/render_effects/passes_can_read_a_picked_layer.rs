use super::*;
use motolii_edit::{Document, Intent};
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;

fn pixel(frame: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * SIZE + x) * 4) as usize;
    [frame[i], frame[i + 1], frame[i + 2]]
}

#[test]
fn set_matte_cuts_by_the_layer_the_user_picked() {
    let dir = tempfile::tempdir().unwrap();
    let blue = dir.path().join("blue.png");
    image::save_buffer(&blue, &[0u8, 0, 100, 255].repeat((SIZE * SIZE) as usize), SIZE, SIZE, image::ColorType::Rgba8).unwrap();
    // 左半分が不透明な白、右半分が透明。
    let matte = dir.path().join("matte.png");
    let mut half = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for _y in 0..SIZE { for x in 0..SIZE { half.extend_from_slice(if x < SIZE / 2 { &[255, 255, 255, 255] } else { &[0, 0, 0, 0] }); } }
    image::save_buffer(&matte, &half, SIZE, SIZE, image::ColorType::Rgba8).unwrap();
    let red = dir.path().join("red.png");
    image::save_buffer(&red, &[255u8, 0, 0, 255].repeat((SIZE * SIZE) as usize), SIZE, SIZE, image::ColorType::Rgba8).unwrap();

    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    for (id, path, order) in [(1u64, &blue, 0i16), (2, &matte, 1), (3, &red, 2)] {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order, timing: LayerTiming::place(0, None, 1) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        ]).unwrap();
    }
    doc.apply_all([
        Intent::SetEffects { layer: LayerId(3), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.set_matte".into() }] },
        Intent::SetConstant { layer: LayerId(3), property: PropertyId::effect_param(EffectId(0), "layer").unwrap(), value: Value::LayerId(2) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let frame = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    let left = pixel(&frame, 16, 32);
    let right = pixel(&frame, 48, 32);
    assert!(left[0] > 200 && left[2] < 30, "相手の α が 1 の所は赤が残る: {left:?}");
    assert!(right[0] < 30, "相手の α が 0 の所は赤が切れて下が見える: {right:?}");

    // 自分自身を指したら断る(黙って今の絵で代用しない)。
    doc.apply(Intent::SetConstant { layer: LayerId(3), property: PropertyId::effect_param(EffectId(0), "layer").unwrap(), value: Value::LayerId(3) }).unwrap();
    engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
    assert!(engine.layer_failures().iter().any(|f| f.contains("指した層")), "{:?}", engine.layer_failures());
}
