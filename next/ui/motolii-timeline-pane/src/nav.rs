//! playhead ナビゲーション動詞束(U2、正典 `timeline-grammar.md` §5・§8.1)の
//! **意味の純関数**。`Session`/`StoreView` を読まない・書かない — 呼び出し側
//! (`Shell::update`)が `composition()`/`timeline_property_rows()`/`markers()`/
//! `timeline_rows()` から材料を集め、ここへ渡すだけ。
//!
//! キー→`Message` の解決(`!modifiers.command()` ガード・text 入力中の
//! 横取り防止)は `lib.rs`(`resolve_navigation_key`、既存の `EscapePressed`/
//! `NudgeKeyframe` の並び)側の役目 — ここには置かない(このファイルは
//! `iced::keyboard` 型を一切知らない)。

/// Step Forward/Back(正典 §5「矢印キー」)。`delta` は符号つき(素で ±1、
/// Shift で ±10 — 符号と歩幅は呼び出し側のキー解決が決める、既存の
/// `NudgeKeyframe(i64)` と同じ役割分担)。**選択も clip も動かさない**
/// (AE/Premiere 同型、playhead だけを動かす)。
///
/// `duration_frames <= 0`(comp 無し)では上限を作れないので、下端 0 だけを
/// 守る(`Message::ScrubTo` の `frame.max(0)` と同じ「上限は comp が要る」
/// 前提を踏襲)。
pub fn step_playhead(current: i64, delta: i64, duration_frames: i64) -> i64 {
    let last = if duration_frames > 0 { duration_frames - 1 } else { i64::MAX };
    (current + delta).clamp(0, last)
}

/// Go to End(正典 §8.1 `JumpToCompStart/End` の End 側)。comp が無ければ
/// Start と同じ既定値 0。Start 側は `0` そのものなので専用関数を持たない
/// (`Shell::update` が `Message::JumpPlayheadToStart` で直に代入する)。
pub fn comp_end_frame(duration_frames: i64) -> i64 {
    (duration_frames - 1).max(0)
}

/// JumpPrev/NextMeaningPoint(正典 §8.1)の向き。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpDirection {
    Next,
    Prev,
}

/// JumpPrev/NextMeaningPoint(正典 §8.1)。`points` は意味点(表示中 property
/// 行のキー菱形時刻・locator 等)の集合 — 昇順である必要はない(ここで
/// フィルタ+min/max するので、呼び出し側は集めて渡すだけでよい)。
///
/// `current` と同じ frame の点は対象外(**渡る**のが役目であって、その場に
/// 留まるのではない — AE の `]`/`[` と同型)。渡る先が無ければ `None`
/// (呼び出し側は playhead を動かさない = no-op)。
pub fn nearest_meaning_point(points: &[i64], current: i64, direction: JumpDirection) -> Option<i64> {
    match direction {
        JumpDirection::Next => points.iter().copied().filter(|&f| f > current).min(),
        JumpDirection::Prev => points.iter().copied().filter(|&f| f < current).max(),
    }
}

/// JumpToClipIn/Out(正典 §8.1)の対象端。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipEdge {
    In,
    Out,
}

/// 選択 clip の In/Out フレーム。**「その線ちょうど」を含む半開区間ではない**
/// (`Shell::nudge_keyframe` の `clip_end = row.start + row.duration` と同じ
/// 約束 — clip 終端ちょうどにキーを置ける実データがこの解釈を裏付けている、
/// `next/shell/motolii-shell/src/fixture.rs` のタイトルロゴ opacity 参照)。
pub fn clip_edge_frame(start: i64, duration: i64, edge: ClipEdge) -> i64 {
    match edge {
        ClipEdge::In => start,
        ClipEdge::Out => start + duration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_playhead_moves_by_delta() {
        assert_eq!(step_playhead(900, 1, 1800), 901);
        assert_eq!(step_playhead(900, -1, 1800), 899);
        assert_eq!(step_playhead(900, 10, 1800), 910);
        assert_eq!(step_playhead(900, -10, 1800), 890);
    }

    #[test]
    fn step_playhead_clamps_at_the_composition_bounds() {
        assert_eq!(step_playhead(0, -1, 1800), 0, "0未満へ行かない");
        assert_eq!(step_playhead(1799, 1, 1800), 1799, "尺の終端を超えない");
    }

    #[test]
    fn step_playhead_without_a_composition_only_floors_at_zero() {
        assert_eq!(step_playhead(0, -5, 0), 0);
        assert_eq!(step_playhead(5, 5, 0), 10, "comp 無しは上限を作れない");
    }

    #[test]
    fn comp_end_frame_is_the_last_frame_index() {
        assert_eq!(comp_end_frame(1800), 1799);
        assert_eq!(comp_end_frame(0), 0, "comp 無しは 0");
    }

    #[test]
    fn nearest_meaning_point_finds_the_closest_point_past_current_in_time_order() {
        let points = [0, 20, 70, 90, 150, 510, 1200];
        assert_eq!(nearest_meaning_point(&points, 100, JumpDirection::Next), Some(150));
        assert_eq!(nearest_meaning_point(&points, 100, JumpDirection::Prev), Some(90));
    }

    #[test]
    fn nearest_meaning_point_returns_none_when_there_is_nothing_further_in_that_direction() {
        let points = [0, 20, 70, 90];
        assert_eq!(nearest_meaning_point(&points, 100, JumpDirection::Next), None);
        assert_eq!(nearest_meaning_point(&points, 0, JumpDirection::Prev), None);
    }

    #[test]
    fn nearest_meaning_point_skips_a_point_sitting_exactly_on_current() {
        let points = [90, 150];
        assert_eq!(
            nearest_meaning_point(&points, 90, JumpDirection::Next),
            Some(150),
            "自分の真上の点には留まらず、渡った先を返すはず"
        );
    }

    #[test]
    fn clip_edge_frame_returns_start_and_start_plus_duration() {
        assert_eq!(clip_edge_frame(0, 90, ClipEdge::In), 0);
        assert_eq!(clip_edge_frame(0, 90, ClipEdge::Out), 90);
        assert_eq!(clip_edge_frame(510, 270, ClipEdge::In), 510);
        assert_eq!(clip_edge_frame(510, 270, ClipEdge::Out), 780);
    }
}
