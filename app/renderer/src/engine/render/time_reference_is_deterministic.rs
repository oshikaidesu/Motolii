use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;

/// 中身が時刻で変わる素材(動画)が要る。1 コマごとに色が変わる小さな mp4 を作る。
fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let out = dir.join("ramp.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=s=64x64:d=2:r=10",
               "-pix_fmt", "yuv420p", "-c:v", "libx264", "-y"])
        .arg(&out)
        .status()
        .ok()?;
    status.success().then_some(out)
}

fn document(path: &std::path::Path) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: Fps::try_new(10, 1).unwrap(), duration_frames: 20, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 20) } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.time_difference".into() }] },
    ]).unwrap();
    doc
}

/// 先に素の層(効果なし)で測る。ここが揺れるなら、揺れているのは復号であって時間参照ではない。
#[test]
fn a_plain_clip_is_already_the_same_however_you_got_there() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let mut doc = document(&path);
    doc.apply(Intent::SetEffects { layer: LayerId(1), effects: vec![] }).unwrap();
    let at = RationalTime::try_from_frame(12, Fps::try_new(10, 1).unwrap()).unwrap();
    let mut engine = Engine::new().unwrap();
    let jumped = engine.render_frame(&doc.view(), at).unwrap();
    for frame in 0..12 {
        let t = RationalTime::try_from_frame(frame, Fps::try_new(10, 1).unwrap()).unwrap();
        engine.render_frame(&doc.view(), t).unwrap();
    }
    let scrubbed = engine.render_frame(&doc.view(), at).unwrap();
    assert_eq!(jumped, scrubbed, "効果なしでも、たどり着き方で絵が変わっている");
}

#[test]
fn the_same_time_gives_the_same_picture_however_you_got_there() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let doc = document(&path);
    let at = RationalTime::try_from_frame(12, Fps::try_new(10, 1).unwrap()).unwrap();
    let mut engine = Engine::new().unwrap();

    // いきなりその時刻へ飛ぶ。
    let jumped = engine.render_frame(&doc.view(), at).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());

    // 頭から順に辿ってから、同じ時刻へ。
    for frame in 0..12 {
        let t = RationalTime::try_from_frame(frame, Fps::try_new(10, 1).unwrap()).unwrap();
        engine.render_frame(&doc.view(), t).unwrap();
    }
    let scrubbed = engine.render_frame(&doc.view(), at).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());

    assert_eq!(jumped, scrubbed, "同じ時刻の絵が、たどり着き方で変わった");
    // 素材が時刻で変わる物であることの確認(変わらない素材なら試験になっていない)。
    let other = RationalTime::try_from_frame(4, Fps::try_new(10, 1).unwrap()).unwrap();
    assert_ne!(jumped, engine.render_frame(&doc.view(), other).unwrap(), "時刻で絵が変わる素材で測っている");
}
