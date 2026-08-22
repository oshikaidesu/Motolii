//! 作業範囲(work area / In-Out 点)の**意味の純関数**(B18 第1切片、正典
//! `next/reference/timeline-grammar.md` §5「ループ帯」)。map 採用予定
//! 719/720/721(Clear In/Out/両方)・724/727(Mark Clip/Selection)・
//! 725/726(Mark In/Out)・296/297(Set Work Area In/Out)・1064(作業範囲の
//! 先頭/末尾へ)・1082/1083(Loop/Unloop)・1099(Play In To Out)の意味の家。
//!
//! ## 型の置き場(発注 2026-08-22 B21+B18 第1切片)
//! 正典 §5.5 は scroll/zoom/fold を Session 状態と定めており、作業範囲も同格
//! (Document に乗せない・undo の対象にしない)が、このレーンの write-set は
//! `next/ui/motolii-timeline-pane/` のみで `motolii-shell-state`(Session の
//! 家)に触れない。そのため状態の実体は [`crate::PaneState`](Shell が保持し
//! フレームを跨いで生きる)に置き、`Session` への昇格は supervisor 判断に
//! 委ねる(RETURN の逸脱報告参照)。このモジュールは状態を持たない — 全部
//! 「値を受けて値を返す」だけ(`nav.rs` と同じ型)。
//!
//! ## 区間の約束
//! [`WorkArea`] は**半開区間 `[start, end)`**(clip の `start..start+duration`
//! と同じ言葉遣い — 2つの区間表現を持たない)。最短1フレーム(`end >= start+1`、
//! 正典 §5「最短1フレーム保証」)。再生対象の最終フレームは `end - 1`
//! ([`WorkArea::last_frame`])。

use crate::nav;

/// 作業範囲(In-Out)。半開区間 `[start, end)`、`end >= start + 1` 不変
/// (このモジュールの関数だけが作るので、手組みしない限り破れない)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkArea {
    /// In 点(含む)。
    pub start: i64,
    /// Out 点(含まない — clip の `start + duration` と同じ約束)。
    pub end: i64,
}

impl WorkArea {
    /// 作業範囲の先頭フレーム(map 1064「作業範囲の先頭へ」・正典 §8.1
    /// `JumpToLoopStart` の着地点)。
    pub fn first_frame(&self) -> i64 {
        self.start
    }

    /// 作業範囲の最終フレーム(map 1064「作業範囲の末尾へ」・`JumpToLoopEnd`)。
    /// 半開区間なので `end - 1`。
    pub fn last_frame(&self) -> i64 {
        (self.end - 1).max(self.start)
    }

    /// フレーム数(最短1)。
    pub fn len(&self) -> i64 {
        (self.end - self.start).max(1)
    }

    /// `frame` が範囲内か(半開)。ループ折り返し([`advanced_playhead`])の
    /// 「今 範囲の中を再生しているか」判定。
    pub fn contains(&self, frame: i64) -> bool {
        frame >= self.start && frame < self.end
    }
}

/// ループ帯端の掴み許容(px)。正典 §1 `LOOP_GRAB` と同値(`TRIM_EDGE` と同じ
/// 「正典の文法定数はコード定数」の型)。
pub const LOOP_GRAB: f32 = 8.0;

/// ループ帯(ルーラ最上段)の中のどこを押したか。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopBandPart {
    /// In 端 ±8px。リサイズ(反対端=Out は掴んだ瞬間の値で固定)。
    EdgeIn,
    /// Out 端 ±8px。**同距離なら Out 優先**(正典 §1 — 伸ばす操作が多い)。
    EdgeOut,
    /// 帯の中(両端の間)。平行移動。
    Body,
    /// 帯の外(または帯が無い)。ドラッグで新規作成(左右どちらから引いても
    /// 同じ、正典 §5)。
    Blank,
}

/// `x`(時間場ローカル px)がループ帯のどこに当たるか。`span_x` は現在の
/// 作業範囲の `(start_x, end_x)` — 無ければ全面 [`LoopBandPart::Blank`]。
/// 端は中心 ±[`LOOP_GRAB`] の許容(bar の `TRIM_EDGE`(内側のみ)と違い、
/// 帯端は線なので両側に開く)。**同距離なら Out 優先**(正典 §1)。
pub fn classify_loop_band(x: f32, span_x: Option<(f32, f32)>) -> LoopBandPart {
    let Some((start_x, end_x)) = span_x else {
        return LoopBandPart::Blank;
    };
    let d_in = (x - start_x).abs();
    let d_out = (x - end_x).abs();
    if d_out <= LOOP_GRAB && d_out <= d_in {
        return LoopBandPart::EdgeOut;
    }
    if d_in <= LOOP_GRAB {
        return LoopBandPart::EdgeIn;
    }
    if x > start_x && x < end_x {
        LoopBandPart::Body
    } else {
        LoopBandPart::Blank
    }
}

/// 上限(clamp の cap)。comp が無い(`duration_frames <= 0`)時は上限を作れない
/// (`nav::step_playhead` と同じ前提)。
fn cap(duration_frames: i64) -> i64 {
    if duration_frames > 0 { duration_frames } else { i64::MAX }
}

/// 固定端 `anchor` から `current` まで引いた区間(新規作成とリサイズの共通式)。
/// - **新規**: `anchor` = 押した瞬間のフレーム(左右どちらへ引いても同じ —
///   正典 §5「空白=新規」)
/// - **リサイズ**: `anchor` = 反対端の掴んだ瞬間の値(正典 §5「反対端は掴んだ
///   瞬間の値で固定 — 追い越しで区間が畳まれない」: 端を追い越したら区間は
///   固定端との min/max で張り直され、0 幅に潰れない)
///
/// clamp は `[0, duration]`・最短1フレーム保証。
pub fn dragged_area(anchor: i64, current: i64, duration_frames: i64) -> WorkArea {
    let cap = cap(duration_frames);
    let lo = anchor.min(current).clamp(0, cap - 1);
    let hi = anchor.max(current).clamp(lo + 1, cap);
    WorkArea { start: lo, end: hi }
}

/// 帯の平行移動(正典 §5「中=平行移動」)。長さを保ったまま、掴んだ瞬間の
/// 値(`origin`)基準の**絶対値で出し直す**(delta 蓄積禁止 — 正典 §2 の思想を
/// 帯へ延長)。両端は `[0, duration]` で clamp(長さ優先 — 端に当たったら
/// そこで止まる)。
pub fn moved_area(origin: WorkArea, grab_at_frame: i64, current: i64, duration_frames: i64) -> WorkArea {
    let cap = cap(duration_frames);
    let len = origin.len().min(cap);
    let start = (origin.start + current - grab_at_frame).clamp(0, cap - len);
    WorkArea { start, end: start + len }
}

/// Mark In / Set Work Area In(map 725・296): In 点を `playhead` へ。既存の
/// Out がそのまま使えれば保ち(反対端固定の思想)、Out が無い・playhead が
/// Out 以右なら Out は comp 終端へ落ちる(In だけ打った状態 = 「ここから最後
/// まで」— Premiere/Resolve の In 単独打ちと同じ読み)。
pub fn with_in(area: Option<WorkArea>, playhead: i64, duration_frames: i64) -> WorkArea {
    let cap = cap(duration_frames);
    let start = playhead.clamp(0, cap - 1);
    let end = match area {
        Some(a) if a.end > start => a.end.min(cap),
        _ => {
            if duration_frames > 0 {
                duration_frames
            } else {
                start + 1
            }
        }
    };
    WorkArea { start, end: end.max(start + 1) }
}

/// Mark Out / Set Work Area Out(map 726・297): Out 点を `playhead` へ
/// (半開なので `end = playhead` — Out 端の線が playhead に乗る、AE の N と
/// 同じ絵)。In が使えれば保ち、無ければ 0(「最初からここまで」)。
pub fn with_out(area: Option<WorkArea>, playhead: i64, duration_frames: i64) -> WorkArea {
    let end = playhead.clamp(1, cap(duration_frames));
    let start = match area {
        Some(a) if a.start < end => a.start,
        _ => 0,
    };
    WorkArea { start, end }
}

/// Clear In(map 719): In 点だけ解除 = 先頭(0)へ開く。帯は残る。
pub fn cleared_in(area: WorkArea) -> WorkArea {
    WorkArea { start: 0, end: area.end }
}

/// Clear Out(map 721): Out 点だけ解除 = comp 終端へ開く。comp が無ければ
/// 開く先が無いので現状維持(黙って誤った終端をでっち上げない — M13/M16)。
pub fn cleared_out(area: WorkArea, duration_frames: i64) -> WorkArea {
    if duration_frames > 0 {
        WorkArea { start: area.start.min(duration_frames - 1), end: duration_frames }
    } else {
        area
    }
}

/// 再生 clock の1歩(B21「作業範囲へループ」の意味接続そのもの — map
/// 1082/1083 Loop・1099 Play In To Out の折り返し部)。
///
/// - ループ on・範囲あり・**今その中を再生している**時だけ、範囲内で折り返す
///   (`rem_euclid` — 順再生で `end` を踏み越えたら `start` へ、逆再生で
///   `start` を割ったら `last_frame` へ。1 tick に複数フレーム進む倍速
///   (シャトル)でも range 長で正しく畳まれる)
/// - 範囲の外にいる時は捕まえない(ループを跨いで普通に通過できる — 罠に
///   しない)。ループ off も同様。どちらも `nav::step_playhead` と同じ
///   `[0, duration)` clamp へ落ちる
pub fn advanced_playhead(
    current: i64,
    delta: i64,
    area: Option<WorkArea>,
    loop_enabled: bool,
    duration_frames: i64,
) -> i64 {
    if loop_enabled {
        if let Some(area) = area {
            if area.end > area.start && area.contains(current) {
                return area.start + (current + delta - area.start).rem_euclid(area.len());
            }
        }
    }
    nav::step_playhead(current, delta, duration_frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- classify(端8px・Out優先・帯なし全面Blank) ----

    #[test]
    fn without_a_band_every_x_is_blank() {
        assert_eq!(classify_loop_band(50.0, None), LoopBandPart::Blank);
    }

    #[test]
    fn edges_grab_within_loop_grab_px_on_both_sides() {
        let span = Some((100.0, 200.0));
        assert_eq!(classify_loop_band(92.0, span), LoopBandPart::EdgeIn, "In端の外側8px");
        assert_eq!(classify_loop_band(108.0, span), LoopBandPart::EdgeIn, "In端の内側8px");
        assert_eq!(classify_loop_band(192.0, span), LoopBandPart::EdgeOut, "Out端の内側8px");
        assert_eq!(classify_loop_band(208.0, span), LoopBandPart::EdgeOut, "Out端の外側8px");
        assert_eq!(classify_loop_band(150.0, span), LoopBandPart::Body);
        assert_eq!(classify_loop_band(90.0, span), LoopBandPart::Blank);
        assert_eq!(classify_loop_band(210.0, span), LoopBandPart::Blank);
    }

    /// 正典 §1「同距離なら Out 優先(伸ばす操作が多い)」— 幅16px 以下の帯は
    /// 中央で両端が等距離になる。
    #[test]
    fn out_wins_the_tie_when_both_edges_are_in_reach() {
        let span = Some((100.0, 110.0));
        assert_eq!(classify_loop_band(105.0, span), LoopBandPart::EdgeOut);
    }

    // ---- dragged_area(新規/リサイズ共通式) ----

    #[test]
    fn dragging_left_or_right_from_the_anchor_makes_the_same_area() {
        assert_eq!(dragged_area(100, 40, 300), WorkArea { start: 40, end: 100 });
        assert_eq!(dragged_area(40, 100, 300), WorkArea { start: 40, end: 100 });
    }

    #[test]
    fn a_zero_width_drag_still_keeps_one_frame() {
        assert_eq!(dragged_area(50, 50, 300), WorkArea { start: 50, end: 51 });
    }

    #[test]
    fn dragged_area_clamps_into_the_composition() {
        assert_eq!(dragged_area(100, -50, 300), WorkArea { start: 0, end: 100 });
        assert_eq!(dragged_area(100, 999, 300), WorkArea { start: 100, end: 300 });
        // 終端ちょうどを掴んでも最短1フレームが残る。
        assert_eq!(dragged_area(300, 300, 300), WorkArea { start: 299, end: 300 });
    }

    /// 正典 §5「追い越しで区間が畳まれない」: In 端を Out の先まで引くと、
    /// 固定端(旧 Out)との min/max で張り直され 0 幅に潰れない。
    #[test]
    fn crossing_the_fixed_end_reopens_instead_of_collapsing() {
        // In端リサイズ(anchor = 旧 Out = 200)で 250 まで追い越し。
        assert_eq!(dragged_area(200, 250, 300), WorkArea { start: 200, end: 250 });
    }

    // ---- moved_area(平行移動) ----

    #[test]
    fn moving_preserves_length_and_stops_at_both_walls() {
        let origin = WorkArea { start: 100, end: 160 };
        assert_eq!(moved_area(origin, 120, 140, 300), WorkArea { start: 120, end: 180 });
        assert_eq!(moved_area(origin, 120, -500, 300), WorkArea { start: 0, end: 60 }, "左壁で停止");
        assert_eq!(moved_area(origin, 120, 900, 300), WorkArea { start: 240, end: 300 }, "右壁で停止");
    }

    // ---- Mark In/Out・Clear In/Out(map 725/726・719/721) ----

    #[test]
    fn mark_in_keeps_a_usable_out_and_otherwise_opens_to_the_end() {
        let area = Some(WorkArea { start: 50, end: 200 });
        assert_eq!(with_in(area, 80, 300), WorkArea { start: 80, end: 200 }, "Out は保つ");
        assert_eq!(with_in(None, 80, 300), WorkArea { start: 80, end: 300 }, "帯なし= ここから最後まで");
        assert_eq!(
            with_in(area, 250, 300),
            WorkArea { start: 250, end: 300 },
            "Out を追い越したら終端へ開く"
        );
    }

    #[test]
    fn mark_out_keeps_a_usable_in_and_otherwise_opens_to_the_start() {
        let area = Some(WorkArea { start: 50, end: 200 });
        assert_eq!(with_out(area, 120, 300), WorkArea { start: 50, end: 120 }, "In は保つ");
        assert_eq!(with_out(None, 120, 300), WorkArea { start: 0, end: 120 }, "帯なし= 最初からここまで");
        assert_eq!(with_out(area, 30, 300), WorkArea { start: 0, end: 30 }, "In を割ったら先頭へ開く");
    }

    #[test]
    fn clear_in_and_clear_out_open_one_side_each() {
        let area = WorkArea { start: 50, end: 200 };
        assert_eq!(cleared_in(area), WorkArea { start: 0, end: 200 });
        assert_eq!(cleared_out(area, 300), WorkArea { start: 50, end: 300 });
        assert_eq!(cleared_out(area, 0), area, "comp なしでは終端をでっち上げない");
    }

    // ---- ループ折り返し(B21「作業範囲へループ」— 発注オラクル) ----

    #[test]
    fn playback_wraps_forward_at_the_loop_end() {
        let area = Some(WorkArea { start: 100, end: 160 });
        // 最終フレーム 159 の次は start へ。
        assert_eq!(advanced_playhead(159, 1, area, true, 300), 100);
        assert_eq!(advanced_playhead(150, 1, area, true, 300), 151, "範囲内は素直に進む");
    }

    #[test]
    fn playback_wraps_backward_at_the_loop_start() {
        let area = Some(WorkArea { start: 100, end: 160 });
        assert_eq!(advanced_playhead(100, -1, area, true, 300), 159, "逆再生は last_frame へ");
    }

    /// シャトル倍速(1 tick に複数フレーム)でも範囲長で正しく畳む。
    #[test]
    fn playback_folds_multi_frame_steps_with_rem_euclid() {
        let area = Some(WorkArea { start: 100, end: 160 });
        assert_eq!(advanced_playhead(158, 8, area, true, 300), 106);
        assert_eq!(advanced_playhead(101, -8, area, true, 300), 153);
    }

    /// 範囲の外にいる playhead は捕まえない(罠にしない)・ループ off は素通し。
    #[test]
    fn the_loop_only_captures_a_playhead_already_inside() {
        let area = Some(WorkArea { start: 100, end: 160 });
        assert_eq!(advanced_playhead(50, 1, area, true, 300), 51, "外は普通に通過");
        assert_eq!(advanced_playhead(159, 1, area, false, 300), 160, "off は素通し");
        assert_eq!(advanced_playhead(299, 1, area, false, 300), 299, "comp 終端 clamp は既存どおり");
        assert_eq!(advanced_playhead(159, 1, None, true, 300), 160, "帯なしも素通し");
    }

    #[test]
    fn first_and_last_frame_are_the_jump_targets() {
        let area = WorkArea { start: 100, end: 160 };
        assert_eq!(area.first_frame(), 100, "map 1064: 作業範囲の先頭へ");
        assert_eq!(area.last_frame(), 159, "map 1064: 作業範囲の末尾へ(半開の -1)");
    }
}
