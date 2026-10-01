use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;

fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let out = dir.join("ramp.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=s=64x64:d=3:r=10", "-pix_fmt", "yuv420p", "-c:v", "libx264", "-y"])
        .arg(&out).status().ok()?;
    status.success().then_some(out)
}
fn fps() -> Fps { Fps::try_new(10, 1).unwrap() }
fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

/// 下 = 動画、上 = 白い板(comp 全面)に効果。`top` が None なら下だけ。
fn document(clip: &std::path::Path, top: Option<(&std::path::Path, &str)>) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: fps(), duration_frames: 30, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let below = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(below),
        Intent::SetMeta { layer: below, meta: LayerMeta { source: LayerSource::File { path: clip.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 30) } },
        Intent::SetConstant { layer: below, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
    ]).unwrap();
    if let Some((white, effect)) = top {
        let above = LayerId(2);
        doc.apply_all([
            Intent::AddLayer(above),
            Intent::SetMeta { layer: above, meta: LayerMeta { source: LayerSource::File { path: white.to_string_lossy().into_owned(), fingerprint: None }, order: 1, timing: LayerTiming::place(0, None, 30) } },
            Intent::SetConstant { layer: above, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
            Intent::SetEffects { layer: above, effects: vec![EffectInstance { id: EffectId(0), plugin_id: effect.into() }] },
            Intent::SetConstant { layer: above, property: PropertyId::effect_param(EffectId(0), "offset").unwrap(), value: Value::F64(-0.5) },
        ]).unwrap();
    }
    doc
}

fn close(a: &[u8], b: &[u8]) -> usize {
    a.chunks_exact(4).zip(b.chunks_exact(4)).filter(|(a, b)| a[..3].iter().zip(&b[..3]).any(|(x, y)| x.abs_diff(*y) > 6)).count()
}

/// TIME_AT: 「層の入点から moment 秒の絵」を host が渡す。いつ見ても同じ絵(静止フレーム)で、
/// 層の入点を動かせば絵も付いて来る。
#[test]
fn a_held_moment_is_the_same_picture_at_every_time_and_follows_the_in_point() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let mut doc = document(&path, None);
    doc.apply_all([
        Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.hold".into() }] },
        Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "moment").unwrap(), value: Value::F64(0.5) },
    ]).unwrap();
    let plain = document(&path, None);
    let mut engine = Engine::new().unwrap();
    let reference = engine.render_frame(&plain.view(), at(5)).unwrap();
    for frame in [3, 12, 20] {
        let held = engine.render_frame(&doc.view(), at(frame)).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        assert!(close(&held, &reference) < 40, "frame {frame}: 0.5 秒の絵と違う: {} px", close(&held, &reference));
        if frame != 5 { assert!(close(&held, &engine.render_frame(&plain.view(), at(frame)).unwrap()) > 200, "frame {frame}: 今の絵と同じ = 止まっていない"); }
    }
    // 入点を 4 コマ後ろへ: moment 0.5 秒 = comp の 9 コマ目の素材(素材の頭から 0.5 秒)。
    let mut later = document(&path, None);
    later.apply_all([
        Intent::SetTiming { layer: LayerId(1), timing: LayerTiming::place(4, None, 26) },
        Intent::SetEffects { layer: LayerId(1), effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.hold".into() }] },
        Intent::SetConstant { layer: LayerId(1), property: PropertyId::effect_param(EffectId(0), "moment").unwrap(), value: Value::F64(0.5) },
    ]).unwrap();
    let held = engine.render_frame(&later.view(), at(15)).unwrap();
    assert!(close(&held, &reference) < 40, "入点が動いても、素材の頭から 0.5 秒の絵: {} px", close(&held, &reference));
}

#[test]
fn the_background_a_moment_ago_is_the_layers_below_rendered_then() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let white = dir.path().join("white.png");
    image::save_buffer(&white, &vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE, image::ColorType::Rgba8).unwrap();
    let delayed = document(&path, Some((&white, "motolii.background_delay")));
    let below_only = document(&path, None);
    let mut engine = Engine::new().unwrap();
    let seen = engine.render_frame(&delayed.view(), at(12)).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    // 12 フレーム目に見えるのは、下の層だけを 7 フレーム目(0.5 秒前)に描いた絵。
    let then = engine.render_frame(&below_only.view(), at(7)).unwrap();
    let now = engine.render_frame(&below_only.view(), at(12)).unwrap();
    assert!(close(&seen, &then) < 40, "0.5 秒前の下の合成と違う: {} px", close(&seen, &then));
    assert!(close(&seen, &then) < close(&seen, &now), "今の下の合成の方に近い = ずれていない");
    // 飛んでも辿っても同じ。
    let mut fresh = Engine::new().unwrap();
    for f in 0..12 { fresh.render_frame(&delayed.view(), at(f)).unwrap(); }
    assert_eq!(fresh.render_frame(&delayed.view(), at(12)).unwrap(), seen);
}
