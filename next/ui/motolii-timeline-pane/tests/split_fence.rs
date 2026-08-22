//! Split(レイヤー分割、B39)の落ちるテスト先行(発注: 裁定189 dispatch
//! 「Split(レイヤー分割)— 高 freq の未実装動詞(B39)」)。**書くだけ・
//! 実行しない**(裁定189 の規律 — `cargo check --tests` が緑になることだけを
//! この発注の検収線とする)。
//!
//! オラクル:
//! - (a) 分割後2レイヤーの in/out(`split_plan` が正しい `LayerTiming` を組む)
//! - (b) track 保存(キーフレームが時刻不変で両半分に残る、`split.rs` モジュール
//!   doc の「AE 型」)
//! - (c) undo 1段(`apply_all` 1回 = undo 1回で split 前へ完全に戻る)
//! - (d) 端 no-op(`at_frame == in`/`== out` は `Err`、Document は無傷のまま)
//! - (e) locked 拒否(理由文つき `Err`)
//! - (f) 複数選択(`split_selected_plan` — 割れる layer だけ割れる・1 apply_all
//!   = 1 undo・複製 id が衝突しない)

use motolii_store::{
    Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Speed, Value,
};
use motolii_timeline_pane::split::{split_plan, split_selected_plan, Message as SplitMessage};

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, fps30()).expect("frame は 30fps で表せる")
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .expect("comp 設定");
    doc
}

/// `id` に、`[start, start+duration)` を占める Solid layer を新規配置する。
fn place_layer(doc: &mut Document, id: LayerId, start: i64, duration: i64, source_in: i64) {
    doc.apply_all([
        Intent::AddLayer(id),
        Intent::SetMeta {
            layer: id,
            meta: LayerMeta {
                source: LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 },
                order: 0,
                timing: LayerTiming { start, duration, source_in, speed: Speed::NORMAL },
            },
        },
    ])
    .expect("layer 配置");
}

fn opacity() -> PropertyId {
    PropertyId::new("opacity").expect("opacity は予約語ではない")
}

/// `frames` の位置に Hold キーを打った track(値はフレーム番号そのもの —
/// どのキーがどれかテストから見分けやすくするため)。
fn track_with_keys(frames: &[i64]) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    for &frame in frames {
        track.insert(Keyframe {
            t: t(frame),
            value: Value::F64(frame as f64),
            interp: Interp::Hold,
            spatial: None,
        });
    }
    track
}

// ---------------------------------------------------------------------------
// (a) 分割後2レイヤーの in/out
// ---------------------------------------------------------------------------

#[test]
fn split_produces_two_layers_with_correct_in_out() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_layer(&mut doc, layer, 10, 90, 5); // [10, 100) 、source_in=5

    let intents = split_plan(&doc.view(), layer, 40).expect("40 は [10,100) の内側");
    doc.apply_all(intents).expect("split の apply_all");

    let mut layers = doc.view().layers();
    layers.sort();
    assert_eq!(layers, vec![LayerId(1), LayerId(2)], "分割後は2レイヤーになる");

    let original = doc.view().meta(layer).unwrap().unwrap().timing;
    assert_eq!(
        original,
        LayerTiming { start: 10, duration: 30, source_in: 5, speed: Speed::NORMAL },
        "元レイヤーは前半 [10,40) だけ残る(source_in/speed 不変)"
    );

    let duplicate = doc.view().meta(LayerId(2)).unwrap().unwrap().timing;
    assert_eq!(
        duplicate,
        LayerTiming { start: 40, duration: 60, source_in: 35, speed: Speed::NORMAL },
        "複製レイヤーは後半 [40,100) — source_in は 5+(40-10)=35"
    );
}

/// 等速でない speed でも `source_frame` 写像を使う(裁定63 の 1:1 決め打ちを
/// 再現しないことの検分)。2倍速(2/1): comp 30フレーム進む間に素材60フレーム進む。
#[test]
fn split_uses_speed_aware_source_mapping_for_the_duplicate() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid { rgba: [0, 255, 0, 255], width: 64, height: 64 },
                order: 0,
                timing: LayerTiming {
                    start: 0,
                    duration: 100,
                    source_in: 0,
                    speed: Speed::try_new(2, 1).expect("2倍速"),
                },
            },
        },
    ])
    .expect("2倍速 layer 配置");

    let intents = split_plan(&doc.view(), layer, 30).expect("30 は [0,100) の内側");
    doc.apply_all(intents).expect("split の apply_all");

    let duplicate = doc.view().meta(LayerId(2)).unwrap().unwrap().timing;
    assert_eq!(
        duplicate.source_in, 60,
        "2倍速なので comp 30フレーム分 = 素材60フレーム分進んでいる"
    );
}

// ---------------------------------------------------------------------------
// (b) track 保存(キーフレームは時刻不変で両半分に残る)
// ---------------------------------------------------------------------------

#[test]
fn split_preserves_keyframe_track_values_on_both_halves() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_layer(&mut doc, layer, 10, 90, 0); // [10, 100)

    let track = track_with_keys(&[20, 80]);
    doc.apply(Intent::SetTrack { layer, property: opacity(), track: track.clone() })
        .expect("opacity track 設置");

    let intents = split_plan(&doc.view(), layer, 40).expect("40 は内側");
    doc.apply_all(intents).expect("split の apply_all");

    let original_track = doc.view().track(layer, &opacity()).unwrap().unwrap();
    assert_eq!(original_track, track, "元レイヤーの track は split で変わらない(cut/trim しない)");

    let duplicate_track = doc.view().track(LayerId(2), &opacity()).unwrap().unwrap();
    assert_eq!(
        duplicate_track, track,
        "複製レイヤーは同じ track をそのまま複製(キーフレームは時刻不変、AE 型)"
    );
}

// ---------------------------------------------------------------------------
// (c) undo 1段
// ---------------------------------------------------------------------------

#[test]
fn split_is_a_single_undo_step() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_layer(&mut doc, layer, 10, 90, 0);

    let intents = split_plan(&doc.view(), layer, 40).expect("40 は内側");
    doc.apply_all(intents).expect("split の apply_all");
    assert_eq!(doc.view().layers().len(), 2);

    assert!(doc.can_undo());
    assert!(doc.undo(), "split の undo は1回で戻るはず");
    assert_eq!(doc.view().layers(), vec![layer], "undo 1回で split 前(1レイヤー)に戻る");
    assert_eq!(
        doc.view().meta(layer).unwrap().unwrap().timing,
        LayerTiming { start: 10, duration: 90, source_in: 0, speed: Speed::NORMAL },
        "undo 後は元の timing に完全に戻る"
    );

    assert!(doc.redo());
    assert_eq!(doc.view().layers().len(), 2, "redo で split 後の状態に戻る");
}

// ---------------------------------------------------------------------------
// (d) 端 no-op
// ---------------------------------------------------------------------------

#[test]
fn split_at_the_in_point_is_rejected() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_layer(&mut doc, layer, 10, 90, 0); // [10, 100)

    let result = split_plan(&doc.view(), layer, 10);
    assert!(result.is_err(), "at == in は端なので分割できない");
    assert_eq!(doc.view().layers(), vec![layer], "Err なので Document は無傷");
}

#[test]
fn split_at_the_out_point_is_rejected() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_layer(&mut doc, layer, 10, 90, 0); // [10, 100)

    let result = split_plan(&doc.view(), layer, 100);
    assert!(result.is_err(), "at == out は端なので分割できない");
    assert_eq!(doc.view().layers(), vec![layer], "Err なので Document は無傷");
}

// ---------------------------------------------------------------------------
// (e) locked 拒否
// ---------------------------------------------------------------------------

#[test]
fn split_on_a_locked_layer_is_rejected_with_a_reason() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_layer(&mut doc, layer, 10, 90, 0);
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch { locked: Some(true), ..Default::default() },
    })
    .expect("locked 設定");

    let result = split_plan(&doc.view(), layer, 40);
    let error = result.expect_err("locked な layer は分割できない");
    assert!(error.contains("ロック"), "拒否理由に locked である旨が書かれていない: {error}");
}

// ---------------------------------------------------------------------------
// (f) 複数選択(1 apply_all = 1 undo・割れる layer だけ割れる・id 衝突なし)
// ---------------------------------------------------------------------------

#[test]
fn split_selected_plan_splits_only_the_layers_that_can_split_in_one_undo() {
    let mut doc = doc_with_comp();
    let splittable = LayerId(1);
    let out_of_range = LayerId(2);
    let locked = LayerId(3);

    place_layer(&mut doc, splittable, 10, 90, 0); // [10,100) — 40 の内側
    place_layer(&mut doc, out_of_range, 200, 50, 0); // [200,250) — 40 の外側
    place_layer(&mut doc, locked, 0, 300, 0); // [0,300) — 40 の内側だが locked
    doc.apply(Intent::SetAttrs {
        layer: locked,
        patch: LayerAttrsPatch { locked: Some(true), ..Default::default() },
    })
    .expect("locked 設定");

    let intents = split_selected_plan(&doc.view(), &[splittable, out_of_range, locked], 40)
        .expect("splittable だけでも1本 split できるので Ok");
    doc.apply_all(intents).expect("bulk split の apply_all");

    let mut layers = doc.view().layers();
    layers.sort();
    assert_eq!(
        layers,
        vec![LayerId(1), LayerId(2), LayerId(3), LayerId(4)],
        "splittable だけ複製され(新id=4)、out_of_range/locked は複製されない"
    );
    assert_eq!(
        doc.view().meta(out_of_range).unwrap().unwrap().timing,
        LayerTiming { start: 200, duration: 50, source_in: 0, speed: Speed::NORMAL },
        "区間外の layer は無傷"
    );
    assert_eq!(
        doc.view().meta(locked).unwrap().unwrap().timing,
        LayerTiming { start: 0, duration: 300, source_in: 0, speed: Speed::NORMAL },
        "locked な layer は無傷"
    );
    assert_eq!(
        doc.view().meta(splittable).unwrap().unwrap().timing.duration,
        30,
        "splittable は前半 [10,40) に縮んでいる"
    );

    assert!(doc.can_undo());
    assert!(doc.undo(), "bulk split も undo 1回で戻るはず(1 apply_all = 1 undo)");
    let mut layers_after_undo = doc.view().layers();
    layers_after_undo.sort();
    assert_eq!(
        layers_after_undo,
        vec![LayerId(1), LayerId(2), LayerId(3)],
        "undo 1回で bulk split 前(3レイヤー)に完全に戻る"
    );
}

#[test]
fn split_selected_plan_errs_when_nothing_in_the_selection_can_split() {
    let mut doc = doc_with_comp();
    let out_of_range = LayerId(1);
    place_layer(&mut doc, out_of_range, 200, 50, 0); // [200,250) — 40 の外側

    let result = split_selected_plan(&doc.view(), &[out_of_range], 40);
    assert!(result.is_err(), "選択の誰も分割できないなら Err");
}

// ---------------------------------------------------------------------------
// Message(次波統合待ち)の smoke test — 宣言だけで終わらないことの検分。
// ---------------------------------------------------------------------------

#[test]
fn split_at_playhead_message_is_declared_for_next_wave_integration() {
    let message = SplitMessage::SplitAtPlayhead;
    assert_eq!(message, SplitMessage::SplitAtPlayhead);
}
