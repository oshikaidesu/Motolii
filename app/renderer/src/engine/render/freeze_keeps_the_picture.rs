use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;
fn fps() -> Fps { Fps::try_new(10, 1).unwrap() }
fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }
fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let out = dir.join("ramp.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=s=64x64:d=3:r=10", "-pix_fmt", "yuv420p", "-c:v", "libx264", "-y"])
        .arg(&out).status().ok()?;
    status.success().then_some(out)
}
fn document(path: &std::path::Path) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: fps(), duration_frames: 30, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(2, None, 26) } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        // feedback(残像)+ 通常の pass: 凍れば両方が cache の絵になる。
        Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.rgb_trail".into() }, EffectInstance { id: EffectId(1), plugin_id: "motolii.gain".into() }] },
        Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(1), "gain").unwrap(), value: Value::F64(1.5) },
    ]).unwrap();
    doc
}
fn close(a: &[u8], b: &[u8]) -> usize {
    a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x.abs_diff(*y) > 2)).count()
}

#[test]
fn a_plain_shape_freezes_to_material_space_without_changing_its_picture() {
    let dir = tempfile::tempdir().unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    let layer = LayerId(1);
    doc.apply_all([
        Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: fps(), duration_frames: 1, background: [0.0, 0.0, 0.0, 1.0] }),
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 1) } },
        Intent::SetShapes { layer, shapes: vec![crate::doc::store::rect_shape([255; 4], [16.0, 16.0])] },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([16.0, 16.0]) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let live = engine.render_frame(&doc.view(), at(0)).unwrap();
    doc.apply(Intent::Freeze { group: layer }).unwrap();
    let recording = doc.flattened().unwrap().into_recording();
    engine.set_cache_root(Some(dir.path().to_owned()));
    assert!(engine.freeze_bake_frame(&recording.view(), layer, 0).unwrap(), "a planar shape must produce a cached frame");
    assert_eq!(engine.frozen_frames_on_disk(layer), 1);
    let frozen = engine.render_frame(&doc.view(), at(0)).unwrap();
    assert!(close(&live, &frozen) < 30);
}

#[test]
fn a_frozen_layer_draws_the_same_picture_from_its_cache_and_thaws_back() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let mut doc = document(&path);
    let mut engine = Engine::new().unwrap();
    // 生の絵(順に辿る: feedback の履歴込み)。
    let mut live = Vec::new();
    for f in 0..16 { live.push(engine.render_frame(&doc.view(), at(f)).unwrap()); }
    // 焼く(cache は書類の隣に見立てた dir)。
    let root = dir.path().join("cache");
    engine.set_cache_root(Some(root.clone()));
    doc.apply(Intent::Freeze { group: LayerId(1) }).unwrap();
    let mut baked = 0;
    for f in 0..16 { if engine.freeze_bake_frame(&doc.view(), LayerId(1), f).unwrap() { baked += 1; } }
    assert_eq!(baked, 14, "入点 2 から 16 コマ目の手前まで、層が居るコマだけ焼ける");
    assert_eq!(engine.frozen_frames_on_disk(LayerId(1)), 14);
    // 凍っても絵は同じ(half float の丸めだけ)。飛んで来ても辿り直しが要らない。
    for f in [3, 9, 15] {
        let frozen = engine.render_frame(&doc.view(), at(f)).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(close(&frozen, &live[f as usize]) < 30, "frame {f}: 凍った絵が生の絵と違う: {} px", close(&frozen, &live[f as usize]));
    }
    // 凍っている間、中は触れない。
    assert!(doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(1), "gain").unwrap(), value: Value::F64(3.0) }).is_err(), "凍った層の欄は拒む");
    // 別の engine(再起動)でも disk の cache から同じ絵。
    let mut fresh = Engine::new().unwrap();
    fresh.set_cache_root(Some(root.clone()));
    let again = fresh.render_frame(&doc.view(), at(9)).unwrap();
    assert!(close(&again, &live[9]) < 30, "再起動後の cache の絵が違う: {} px", close(&again, &live[9]));
    assert_eq!(fresh.frozen_frames_resident(LayerId(1)), 1);
    // Unfreeze: cache を捨て、生に戻る。書類の欄も触れる。
    doc.apply(Intent::Unfreeze { group: LayerId(1) }).unwrap();
    engine.forget_frozen(LayerId(1));
    assert_eq!(engine.frozen_frames_on_disk(LayerId(1)), 0);
    let thawed = engine.render_frame(&doc.view(), at(9)).unwrap();
    assert!(close(&thawed, &live[9]) < 30, "戻した絵が生の絵と違う: {} px", close(&thawed, &live[9]));
    doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(1), "gain").unwrap(), value: Value::F64(3.0) }).unwrap();
}
