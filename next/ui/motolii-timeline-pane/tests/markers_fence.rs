//! マーカーレーン(B19 第1切片)の柵。**未実行**(裁定189: 検収線は
//! `cargo check --tests -p motolii-timeline-pane` 緑まで — テストは書くが
//! 走らせない。実行は supervisor / 後続レーン)。
//!
//! オラクル(発注やること2+正典 §5「locator」):
//! - (a) 縦の家割りが既存梯子だけから導出されている(裁定167 — ループ帯と
//!   隣接・小目盛りと衝突しない)
//! - (b) store 投影の純関数(表示・ラベル) — fps 無しでは1枚も出さない
//! - (c) M タップ=playhead へ即置き(無名・尺0・同一フレーム連打は畳む)
//! - (d) ドラッグ移動は絶対値で出し直し(delta 蓄積禁止)・未移動 release は
//!   no-op・確定は `Vec<Marker>` 丸ごと1回(`Intent::SetMarkers` へ)
//! - (e) click で jump / 右クリック削除の意味(map 722/723・718 の UI 入口)
//! - (f) 当たり: ±8px・範囲ロケータは span 内=距離0・同距離は後の添字

use motolii_store::{Fps, Marker, RationalTime};
use motolii_timeline_pane::markers::{
    added_at_playhead, in_locator_lane, locator_at, locator_head_height, locator_head_top,
    moved, project, removed, LocatorProjection, MarkerDrag, MarkerMessage, LOCATOR_GRAB,
};
use motolii_timeline_pane::{loop_band_height, major_tick_length, minor_tick_length, ruler_height};

/// 1px=1frame 縮尺(`hit.rs`/`input.rs` の既存試験と同じ約束 — 新しい縮尺を
/// 発明しない)。
const WIDTH: f32 = 300.0;
const DURATION: i64 = 300;

fn fps30() -> Fps {
    Fps::try_new(30, 1).expect("30/1 は正の既約 fps")
}

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, fps30()).expect("フレーム→時刻変換")
}

fn marker(name: &str, frame: i64, duration: i64) -> Marker {
    Marker {
        name: name.to_owned(),
        time: t(frame),
        duration: t(duration),
    }
}

// ---------------------------------------------------------------------------
// (a) 縦の家割り — 梯子からの導出のみ(裁定167)
// ---------------------------------------------------------------------------

/// 頭部はループ帯の直下に隙間も重なりも無く住む(面の住み分け)。
#[test]
fn the_locator_head_sits_flush_under_the_loop_band() {
    let h = ruler_height(26.0); // 裁定172 §2: 0.846×行高 = 22px。
    assert_eq!(locator_head_top(h), loop_band_height(h), "頭部上端=ループ帯下端");
}

/// 頭部の高さは「大目盛り長 − 小目盛り長」= 小目盛りが届かない中間帯。
/// 頭部下端は小目盛りの上端にちょうど接する(目盛り梯子と衝突しない)。
#[test]
fn the_locator_head_fills_exactly_the_tick_free_middle_band() {
    let h = ruler_height(26.0);
    assert_eq!(
        locator_head_height(h),
        major_tick_length(h) - minor_tick_length(h),
        "頭部高=大目盛り−小目盛り"
    );
    assert_eq!(
        locator_head_top(h) + locator_head_height(h),
        h - minor_tick_length(h),
        "頭部下端=小目盛り上端(衝突ゼロ)"
    );
}

/// 縦域: 頭部上端〜ルーラ下端(柄も掴める — Q0)。ループ帯とは排他。
#[test]
fn the_locator_lane_spans_head_top_to_ruler_bottom_and_excludes_the_loop_band() {
    let h = ruler_height(26.0);
    let top = locator_head_top(h);
    assert!(in_locator_lane(top, h), "頭部上端は縦域内");
    assert!(in_locator_lane(h - 1.0, h), "柄の下端近くも縦域内");
    assert!(!in_locator_lane(top - 1.0, h), "ループ帯側は縦域外(排他)");
    assert!(!in_locator_lane(h, h), "ルーラ下端(クリップ面)は縦域外");
}

// ---------------------------------------------------------------------------
// (b) store 投影の純関数(map 743 の表示・ラベル)
// ---------------------------------------------------------------------------

/// 宣言順を保ち、frame/尺/名前を写す(px は持たない — 換算は draw/hit 時)。
#[test]
fn projection_keeps_declared_order_and_maps_frames() {
    let markers = vec![marker("Chorus", 30, 15), marker("", 10, 0)];
    let projected = project(&markers, Some(fps30()));
    assert_eq!(
        projected,
        vec![
            LocatorProjection { index: 0, frame: 30, duration_frames: 15, name: "Chorus".into() },
            LocatorProjection { index: 1, frame: 10, duration_frames: 0, name: String::new() },
        ],
        "投影が宣言順・frame・尺・名前を保っていない"
    );
}

/// fps が引けない(comp が無い)なら1枚も出さない — 黙って誤った位置に
/// 描くより描かない(M13、既存 `TimelinePane::marker_frame` と同じ理由)。
#[test]
fn without_fps_no_locator_is_projected() {
    let markers = vec![marker("x", 10, 0)];
    assert!(project(&markers, None).is_empty(), "fps 無しで投影が出ている");
}

// ---------------------------------------------------------------------------
// (c) M タップ=playhead へ即置き(map 717/728、正典 §5)
// ---------------------------------------------------------------------------

/// 即置きは無名・尺0で末尾へ append(名前入力に入らない・宣言順を乱さない)。
#[test]
fn a_tap_drops_an_unnamed_point_locator_at_the_playhead() {
    let markers = vec![marker("a", 10, 0)];
    let next = added_at_playhead(&markers, 42, fps30()).expect("追加できるはず");
    assert_eq!(next.len(), 2);
    assert_eq!(next[1].name, "", "即置きは無名(名前入力に入らない — 正典 §5)");
    assert_eq!(next[1].time, t(42), "playhead 位置に置かれていない");
    assert_eq!(next[1].duration, t(0), "即置きは尺0(単発ロケータ)");
    assert_eq!(next[0], markers[0], "既存マーカーが動いている");
}

/// 同一フレーム連打は1つに畳む(正典 §5 の明文 — `None` は M13 違反ではない)。
#[test]
fn tapping_the_same_frame_twice_collapses_to_one() {
    let markers = vec![marker("", 42, 0)];
    assert!(
        added_at_playhead(&markers, 42, fps30()).is_none(),
        "同一フレームへの連打が畳まれていない"
    );
}

// ---------------------------------------------------------------------------
// (d) ドラッグ移動(map 728/738/739 の時刻 modify、正典 §2 の delta 蓄積禁止)
// ---------------------------------------------------------------------------

/// 時刻だけ差し替え、名前・尺は保つ。comp 内へ clamp。
#[test]
fn moving_changes_only_the_time_and_clamps_into_the_comp() {
    let markers = vec![marker("Verse", 100, 8)];
    let next = moved(&markers, 0, 120, fps30(), DURATION).expect("移動できるはず");
    assert_eq!(next[0].time, t(120));
    assert_eq!(next[0].name, "Verse", "移動で名前が落ちた");
    assert_eq!(next[0].duration, t(8), "移動で尺が落ちた");

    let clamped = moved(&markers, 0, 9_999, fps30(), DURATION).expect("clamp されるはず");
    assert_eq!(clamped[0].time, t(DURATION - 1), "comp 終端へ clamp されていない");
    let clamped = moved(&markers, 0, -50, fps30(), DURATION).expect("clamp されるはず");
    assert_eq!(clamped[0].time, t(0), "先頭へ clamp されていない");
}

/// 外れた添字は `None`(壊さない — 安全側)。
#[test]
fn moving_or_removing_an_out_of_range_index_is_refused() {
    let markers = vec![marker("a", 10, 0)];
    assert!(moved(&markers, 5, 20, fps30(), DURATION).is_none());
    assert!(removed(&markers, 5).is_none());
}

/// ドラッグは掴んだ瞬間の値を基準に**絶対値で出し直す**(delta 蓄積禁止):
/// 130 まで引いてから 105 へ戻すと、プレビューは 95(origin 90 + 15)であって
/// 「120 からさらに −25」の 95 と偶然一致ではない — 中間で3回動かしても同じ。
#[test]
fn a_drag_recomputes_from_the_grab_origin_not_by_accumulating_deltas() {
    let markers = vec![marker("", 90, 0)];
    let mut drag = MarkerDrag::start(&markers, 0, 100, fps30()).expect("掴めるはず");

    drag.dragged(130, fps30(), DURATION);
    assert_eq!(drag.preview()[0].time, t(120), "origin 90 + (130−100)");
    drag.dragged(160, fps30(), DURATION);
    drag.dragged(105, fps30(), DURATION);
    assert_eq!(drag.preview()[0].time, t(95), "origin 90 + (105−100) — 蓄積していない");
}

/// 未移動 release は no-op(`None` — 履歴を汚さない)。動いていれば
/// `Intent::SetMarkers` へ渡す1回分の `Vec<Marker>` 丸ごと(1 gesture = 1 undo)。
#[test]
fn releasing_without_movement_is_a_no_op_and_a_real_move_commits_once() {
    let markers = vec![marker("", 90, 0)];
    let drag = MarkerDrag::start(&markers, 0, 100, fps30()).expect("掴めるはず");
    assert!(drag.finish().is_none(), "未移動 release が no-op でない");

    let mut drag = MarkerDrag::start(&markers, 0, 100, fps30()).expect("掴めるはず");
    drag.dragged(110, fps30(), DURATION);
    let committed = drag.finish().expect("移動したら確定するはず");
    assert_eq!(committed[0].time, t(100), "確定形が最後のプレビューと違う");
}

// ---------------------------------------------------------------------------
// (e) click jump / 右クリック削除(map 722/723・718 の UI 入口)
// ---------------------------------------------------------------------------

/// 削除は名指した1枚だけ落とす(宣言順の残りは無傷)。
#[test]
fn removal_drops_exactly_the_named_locator() {
    let markers = vec![marker("a", 10, 0), marker("b", 20, 0), marker("c", 30, 0)];
    let next = removed(&markers, 1).expect("消せるはず");
    assert_eq!(next.len(), 2);
    assert_eq!(next[0].name, "a");
    assert_eq!(next[1].name, "c", "隣の locator まで巻き込んだ");
}

/// Message の腕が揃っている(独立 enum の柵 — supervisor 統合時に腕を
/// 落とすとここが折れる)。jump はフレームを運ぶだけ(playhead の書き込みは
/// shell の ScrubTo 系へ写す)。
#[test]
fn the_marker_message_arms_cover_the_canonical_locator_verbs() {
    let arms: Vec<MarkerMessage> = vec![
        MarkerMessage::AddAtPlayhead,
        MarkerMessage::Grabbed { index: 0, at_frame: 10 },
        MarkerMessage::DragMoved { at_frame: 12 },
        MarkerMessage::DragReleased,
        MarkerMessage::DragCancelled,
        MarkerMessage::JumpTo(42),
        MarkerMessage::Remove(0),
    ];
    assert_eq!(arms.len(), 7, "正典 §5 の locator 動詞(置く/動かす/跳ぶ/消す)+ドラッグ3態");
}

// ---------------------------------------------------------------------------
// (f) 当たり(±8px・span 内=0・同距離は後の添字)
// ---------------------------------------------------------------------------

#[test]
fn hits_land_within_the_grab_tolerance_and_miss_outside_it() {
    let locators = project(&vec![marker("", 100, 0)], Some(fps30()));
    assert_eq!(locator_at(100.0 + LOCATOR_GRAB, WIDTH, DURATION, &locators), Some(0));
    assert_eq!(locator_at(100.0 - LOCATOR_GRAB, WIDTH, DURATION, &locators), Some(0));
    assert_eq!(
        locator_at(100.0 + LOCATOR_GRAB + 1.0, WIDTH, DURATION, &locators),
        None,
        "許容の外を掴んでいる(scrub を奪う)"
    );
}

/// 範囲ロケータは span 内どこでも距離0で当たる。
#[test]
fn a_duration_locator_is_hit_anywhere_inside_its_span() {
    let locators = project(&vec![marker("", 100, 20)], Some(fps30()));
    assert_eq!(locator_at(115.0, WIDTH, DURATION, &locators), Some(0));
    assert_eq!(locator_at(120.0 + LOCATOR_GRAB, WIDTH, DURATION, &locators), Some(0), "span 右端の外側許容");
}

/// 同距離なら後の添字(後から描いた方が上 — 絵と当たりの一致)。
#[test]
fn the_later_locator_wins_a_distance_tie() {
    let locators = project(
        &vec![marker("", 96, 0), marker("", 104, 0)],
        Some(fps30()),
    );
    assert_eq!(locator_at(100.0, WIDTH, DURATION, &locators), Some(1));
}
