use super::*;
use crate::doc::store::{property, Composition, EffectId, EffectInstance, Fps, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::render::engine::Engine;

const SIZE: u32 = 64;
const FRAMES: i64 = 50;

fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let out = dir.join("ramp.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=s=64x64:d=5:r=10", "-pix_fmt", "yuv420p", "-c:v", "libx264", "-y"])
        .arg(&out).status().ok()?;
    status.success().then_some(out)
}

fn fps() -> Fps { Fps::try_new(10, 1).unwrap() }
fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

fn document(path: &std::path::Path, with_trail: bool) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: fps(), duration_frames: FRAMES, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, FRAMES) } },
        Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
    ]).unwrap();
    if with_trail {
        doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "import.ceil_trail".into() }] }).unwrap();
    }
    doc
}

fn walked_to(doc: &Document, frame: i64) -> (Engine, Vec<u8>) {
    let mut engine = Engine::new().unwrap();
    let mut last = Vec::new();
    for f in 0..=frame { last = engine.render_frame(&doc.view(), at(f)).unwrap(); }
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    (engine, last)
}

#[test]
fn the_same_frame_is_the_same_picture_however_you_got_there() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let doc = document(&path, true);
    let (mut engine, walked) = walked_to(&doc, 12);
    // 同じフレームをもう一度(2 つ目の窓が同じ時刻を描く形)。
    assert_eq!(engine.render_frame(&doc.view(), at(12)).unwrap(), walked, "同じフレームの描き直しで絵が変わった");
    // いきなり飛ぶ(別の engine = 状態が無い)。
    let mut fresh = Engine::new().unwrap();
    let jumped = fresh.render_frame(&doc.view(), at(12)).unwrap();
    assert!(fresh.layer_failures().is_empty(), "{:?}", fresh.layer_failures());
    assert_eq!(jumped, walked, "飛んで来た絵が、辿った絵と違う");
    // 効果が効いている(残像 ≠ 素の絵)、時刻で違う。
    let plain = Engine::new().unwrap().render_frame(&document(&path, false).view(), at(12)).unwrap();
    assert_ne!(walked, plain, "feedback が絵を変えていない");
    assert_ne!(walked, engine.render_frame(&doc.view(), at(4)).unwrap(), "時刻で絵が変わる素材で測っている");
}

#[test]
fn going_back_and_forward_replays_from_the_nearest_checkpoint() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let doc = document(&path, true);
    let (_, straight) = walked_to(&doc, 45);
    // 40 まで辿ってから(checkpoint 30 が焼けている)45 へ飛ぶ。
    let (mut engine, _) = walked_to(&doc, 40);
    assert_eq!(engine.render_frame(&doc.view(), at(45)).unwrap(), straight, "checkpoint から辿り直した絵が違う");
    // 戻る(35 ← 45): checkpoint 30 から 5 歩。
    let (_, back) = walked_to(&doc, 35);
    assert_eq!(engine.render_frame(&doc.view(), at(35)).unwrap(), back, "戻った絵が違う");
    // 入点より前は状態が無く、入点(0)は初期条件。
    assert_eq!(engine.render_frame(&doc.view(), at(0)).unwrap(), walked_to(&doc, 0).1);
}

/// 静止した素材に残像を掛けると、history は 0.2 ずつ入力へ寄る(1 − 0.8ⁿ)。前のフレームを本当に
/// 読んでいれば 12 フレーム目は 0 フレーム目より明るい。毎フレーム初期条件なら同じ明るさのまま。
#[test]
fn history_accumulates_over_frames() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("white.png");
    image::save_buffer(&path, &vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE, image::ColorType::Rgba8).unwrap();
    let doc = document(&path, true);
    let mean = |px: &[u8]| px.chunks_exact(4).map(|p| p[0] as f64).sum::<f64>() / (SIZE * SIZE) as f64;
    let (_, first) = walked_to(&doc, 0);
    let (_, later) = walked_to(&doc, 12);
    assert!(mean(&later) > mean(&first) * 2.0, "history が積もっていない: {} → {}", mean(&first), mean(&later));
}

/// 下の合成を読む列(Background Copy → 残像)は板に焼けず、画面の道で効く。それでも飛んで来た絵は
/// 辿った絵と同じ(窓ごとの状態を、フレームを丸ごと辿り直して作る)。
#[test]
fn a_chain_that_reads_below_is_also_a_recurrence() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let mut doc = document(&path, false);
    doc.apply(Intent::SetEffects { layer: LayerId(1), effects: vec![
        EffectInstance { id: EffectId(0), plugin_id: "motolii.background_copy".into() },
        EffectInstance { id: EffectId(1), plugin_id: "import.ceil_trail".into() },
    ] }).unwrap();
    let (mut engine, walked) = walked_to(&doc, 12);
    assert_eq!(engine.render_frame(&doc.view(), at(12)).unwrap(), walked, "同じフレームの描き直しで絵が変わった");
    let mut fresh = Engine::new().unwrap();
    let jumped = fresh.render_frame(&doc.view(), at(12)).unwrap();
    assert!(fresh.layer_failures().is_empty(), "{:?}", fresh.layer_failures());
    assert_eq!(jumped, walked, "画面の道の feedback が、飛んで来ると違う絵になる");
    // 戻る(45 まで辿ってから 35 へ): checkpoint 30 からフレームを丸ごと 5 歩。
    let (mut far, _) = walked_to(&doc, 45);
    assert_eq!(far.render_frame(&doc.view(), at(35)).unwrap(), walked_to(&doc, 35).1, "戻った絵が違う");
}

#[test]
fn editing_the_document_restarts_the_history() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let mut doc = document(&path, true);
    let (mut engine, before) = walked_to(&doc, 12);
    // 書類を変えると(位置を動かす)、履歴は新しい書類で入点から: 新しい engine の絵と一致する。
    doc.apply(Intent::SetConstant { layer: LayerId(1), property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([6.0, 0.0]) }).unwrap();
    let edited = engine.render_frame(&doc.view(), at(12)).unwrap();
    assert_ne!(edited, before);
    assert_eq!(edited, Engine::new().unwrap().render_frame(&doc.view(), at(12)).unwrap(), "編集後の絵が、入点からの絵と違う");
}
