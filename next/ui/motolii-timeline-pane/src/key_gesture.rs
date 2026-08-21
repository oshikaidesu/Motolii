//! キー(菱形)の時刻編集の**意味関数**(第2波T4、正典
//! `next/reference/timeline-grammar.md` §3・§8.1・裁定146)。`clip_gesture.rs`
//! と対になる純関数だけの置き場 — ここも `motolii_store::Document`/`StoreView`
//! を一切持たない。`i64`(フレーム)と `f32`(px/frame)だけを受けて答えを返し、
//! `motolii_store::KeyframeTrack` への詰め直しは呼び手(`Shell::update`、
//! `lib.rs`)の仕事(`clip_gesture.rs` モジュール doc と同じ役割分担)。
//!
//! ## この波で足す3つの動詞
//!
//! - **時刻ドラッグ**(§3): 単一/複数選択どちらも「掴んだ瞬間の値を基準に
//!   絶対値で出し直す」(delta 蓄積禁止)。複数選択は同じ delta を全員へ適用して
//!   相対間隔を保つ(§8.1「LottieFiles と同型」)。集合の誰かが clip 範囲の端に
//!   当たったら全員そこで止まる([`clamp_group_delta`] — `clip_gesture` の
//!   「塊制約」と同じ形)
//! - **NudgeKeyframe**(§8.1): 固定 delta 版の同じ塊制約。キーボード入口なので
//!   スナップは無い(画面距離という概念自体が無い)
//! - **RetimeSelection**(裁定146・§7-2): 固定端(anchor)からの比例伸縮。
//!   反転しない — anchor を跨ぐ入力は0長(anchor そのもの)へ clamp する
//!
//! ## 同時刻衝突(§3)
//!
//! 「複数キー移動は移動方向でソートしてから書く(delta≥0 なら遅いキーから)」の
//! 決定的な書き込み順を返すのが [`key_write_order`]。呼び手はこの順で
//! `KeyframeTrack::insert` を叩く。

use super::projection::RowProjection;
use crate::clip_gesture::snap_frame;

/// キーのスナップ候補(正典 §3「スナップ…は clip と同じ対象」)。0秒・終端・
/// playhead・**全 clip の start/end**。`clip_gesture::snap_candidates` と違い、
/// 自分の layer を除外しない — キーを掴んでいるのであって clip を掴んでいる
/// のではないので、その clip 自身の start/end も有効な吸着先(むしろ「クリップの
/// 頭に揃える」用途の本命)。
pub fn key_snap_candidates(rows: &[RowProjection], playhead: i64, duration_frames: i64) -> Vec<i64> {
    let mut out = vec![0, duration_frames.max(0), playhead];
    for row in rows {
        out.push(row.start);
        out.push(row.start + row.duration);
    }
    out
}

/// 塊制約(`clip_gesture` の move と同じ思想): 集合の誰か(`min_origin`/
/// `max_origin`)が `[clip_start, clip_end]` の外へ出そうになったら、**全員が
/// そこで止まる delta へ丸め直す**(個別 clamp では相対間隔が壊れる)。
pub fn clamp_group_delta(min_origin: i64, max_origin: i64, delta: i64, clip_start: i64, clip_end: i64) -> i64 {
    let mut delta = delta;
    if min_origin + delta < clip_start {
        delta = clip_start - min_origin;
    }
    if max_origin + delta > clip_end {
        delta = clip_end - max_origin;
    }
    delta
}

/// 複数選択キーの一括時刻ドラッグ(正典 §3・§8.1)。**掴んだキー
/// (`grabbed_origin_frame`)の絶対値を基準に delta を出し直し**(delta 蓄積
/// 禁止)、同じ delta を選択集合全員へ適用して相対間隔を保つ。スナップは
/// 掴んだキーの目標位置にだけ効く(全員を個別にスナップすると間隔が崩れる)。
#[allow(clippy::too_many_arguments)]
pub fn moved_key_group(
    origin_frames: &[i64],
    grabbed_origin_frame: i64,
    grab_at_frame: i64,
    current_frame: i64,
    clip_start: i64,
    clip_end: i64,
    candidates: &[i64],
    px_per_frame: f32,
    snap_enabled: bool,
) -> Vec<i64> {
    if origin_frames.is_empty() {
        return Vec::new();
    }
    let raw_target = grabbed_origin_frame + (current_frame - grab_at_frame);
    let snapped_target = if snap_enabled {
        snap_frame(raw_target, candidates, px_per_frame)
    } else {
        raw_target
    };
    let delta = snapped_target - grabbed_origin_frame;

    let min_origin = origin_frames.iter().copied().min().unwrap_or(grabbed_origin_frame);
    let max_origin = origin_frames.iter().copied().max().unwrap_or(grabbed_origin_frame);
    let delta = clamp_group_delta(min_origin, max_origin, delta, clip_start, clip_end);

    origin_frames.iter().map(|f| f + delta).collect()
}

/// NudgeKeyframe(正典 §8.1「選択キーを1(Shift で10)フレーム前後へ」)。
/// キーボード入口なのでスナップは無い — 固定 `delta` を塊制約付きで適用する
/// だけ([`moved_key_group`] と同じ clamp、ポインタ操作固有の部分だけを除いた形)。
pub fn nudge_key_group(origin_frames: &[i64], delta: i64, clip_start: i64, clip_end: i64) -> Vec<i64> {
    if origin_frames.is_empty() {
        return Vec::new();
    }
    let min_origin = origin_frames.iter().copied().min().unwrap();
    let max_origin = origin_frames.iter().copied().max().unwrap();
    let delta = clamp_group_delta(min_origin, max_origin, delta, clip_start, clip_end);
    origin_frames.iter().map(|f| f + delta).collect()
}

/// RetimeSelection(裁定146)の1キー分。`anchor_frame` は固定端、`edge_frame` は
/// 掴んだ端の掴んだ瞬間の値(スケール1.0の基準)、`new_edge_frame` はドラッグ中の
/// 目標(スナップ・clamp 済み)。**反転しない**(§7-2) — `new_edge_frame` が
/// `anchor_frame` を跨いだら0長(全キーが anchor へ収束)に clamp する。
pub fn retimed_key_frame(frame: i64, anchor_frame: i64, edge_frame: i64, new_edge_frame: i64) -> i64 {
    let span = edge_frame - anchor_frame;
    if span == 0 {
        return anchor_frame; // 起こらないはず(edge==anchor)だが安全側。
    }
    let clamped_new_edge = if span > 0 {
        new_edge_frame.max(anchor_frame)
    } else {
        new_edge_frame.min(anchor_frame)
    };
    let scale = (clamped_new_edge - anchor_frame) as f64 / span as f64;
    anchor_frame + ((frame - anchor_frame) as f64 * scale).round() as i64
}

/// [`retimed_key_frame`] を選択集合全員へ適用する。
pub fn retimed_key_group(frames: &[i64], anchor_frame: i64, edge_frame: i64, new_edge_frame: i64) -> Vec<i64> {
    frames
        .iter()
        .map(|&f| retimed_key_frame(f, anchor_frame, edge_frame, new_edge_frame))
        .collect()
}

/// 同時刻衝突(正典 §3): 複数キー移動は**移動方向でソートしてから書く**
/// (`delta >= 0` なら遅いキー=元 frame の大きい方から)。返る値は
/// `origin_frames` への添字の書き込み順 — 呼び手はこの順で
/// `KeyframeTrack::insert` を叩く。同じ `frame` へ2本のキーが向かっても、
/// 挿入の衝突は「後に書いた方が残る」という `KeyframeTrack::insert` の規則の
/// 上でこの順序が決定的に効く(panic はしない — `insert` 自体が上書きで
/// 衝突を吸収する)。
pub fn key_write_order(origin_frames: &[i64], delta: i64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..origin_frames.len()).collect();
    if delta >= 0 {
        order.sort_by(|&a, &b| origin_frames[b].cmp(&origin_frames[a]));
    } else {
        order.sort_by(|&a, &b| origin_frames[a].cmp(&origin_frames[b]));
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u64, start: i64, duration: i64) -> RowProjection {
        RowProjection {
            id: motolii_store::LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked: false,
            start,
            duration,
            selected: false,
            dragging: false,
        }
    }

    #[test]
    fn key_snap_candidates_include_all_clip_edges_without_exclusion() {
        let rows = vec![row(1, 10, 20), row(2, 200, 30)];
        let candidates = key_snap_candidates(&rows, 5, 300);
        assert!(candidates.contains(&0));
        assert!(candidates.contains(&300));
        assert!(candidates.contains(&5));
        assert!(candidates.contains(&10), "自分の clip の start も候補のはず(キーはclipを掴んでいない)");
        assert!(candidates.contains(&30));
        assert!(candidates.contains(&200));
        assert!(candidates.contains(&230));
    }

    /// 正典 §3: 掴んだ瞬間の値を基準に絶対値で出し直す — 同じ入力に2回呼んでも
    /// 結果は同じ(delta が蓄積しない、`clip_gesture::moved_start` と同じ形)。
    #[test]
    fn moved_key_group_recomputes_from_origin_each_call() {
        let origins = vec![100];
        let first = moved_key_group(&origins, 100, 120, 140, 0, 300, &[], 0.0, false);
        let second = moved_key_group(&origins, 100, 120, 140, 0, 300, &[], 0.0, false);
        assert_eq!(first, second);
        assert_eq!(first, vec![100 + (140 - 120)]);
    }

    /// 複数選択は同じ delta を全員へ適用して相対間隔を保つ(§8.1)。
    #[test]
    fn moved_key_group_preserves_relative_spacing() {
        let origins = vec![100, 120, 150];
        let moved = moved_key_group(&origins, 100, 100, 110, 0, 1000, &[], 0.0, false);
        assert_eq!(moved, vec![110, 130, 160], "delta が全員に同じだけ効いていない");
    }

    /// 塊制約: 誰かが端に当たったら全員そこで止まる(相対間隔は保ったまま)。
    #[test]
    fn moved_key_group_stops_the_whole_group_at_the_clip_edge() {
        let origins = vec![10, 20, 30];
        // clip範囲[0,25]、右のキー(30)を右へ大きく動かそうとしても
        // 「誰か(30)が端(25)を超える」ので全員 delta=-5 に丸められる。
        let moved = moved_key_group(&origins, 30, 30, 100, 0, 25, &[], 0.0, false);
        assert_eq!(moved, vec![5, 15, 25], "端で止めても相対間隔が崩れている");
    }

    #[test]
    fn nudge_key_group_applies_a_fixed_delta_with_the_same_clamp() {
        let origins = vec![5, 10];
        assert_eq!(nudge_key_group(&origins, 1, 0, 300), vec![6, 11]);
        assert_eq!(nudge_key_group(&origins, -10, 0, 300), vec![0, 5], "0未満へ出ている(clip 範囲の下限を破っている)");
    }

    /// RetimeSelection: anchor は不動、edge は目標値そのもの、中間は比例位置。
    #[test]
    fn retimed_key_group_scales_proportionally_from_the_anchor() {
        // anchor=0, edge=100 → edge を 50 へ(半分に縮める)。中間の25は12.5→13(四捨五入)。
        let frames = vec![0, 25, 50, 100];
        let retimed = retimed_key_group(&frames, 0, 100, 50);
        assert_eq!(retimed, vec![0, 13, 25, 50]);
    }

    /// **反転しない**(§7-2): edge が anchor を跨ごうとしたら0長(全員 anchor)に
    /// clamp する。
    #[test]
    fn retimed_key_group_clamps_to_zero_length_instead_of_reversing() {
        let frames = vec![0, 50, 100];
        // edge(100)を anchor(0)の左(-20)へ持っていこうとしても反転しない。
        let retimed = retimed_key_group(&frames, 0, 100, -20);
        assert_eq!(retimed, vec![0, 0, 0], "反転して負の frame へ出ている");
    }

    /// 同時刻衝突: 移動方向でソート — delta>=0 なら元frameの大きい方(遅いキー)
    /// から書く順序を返す。
    #[test]
    fn key_write_order_sorts_the_later_key_first_when_moving_forward() {
        let origins = vec![10, 30, 20];
        let order = key_write_order(&origins, 5);
        // frame の大きい順: 30(idx1), 20(idx2), 10(idx0)。
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn key_write_order_sorts_the_earlier_key_first_when_moving_backward() {
        let origins = vec![10, 30, 20];
        let order = key_write_order(&origins, -5);
        assert_eq!(order, vec![0, 2, 1]);
    }

    /// **ORACLE (f)**: 2キーが同じフレームへ向けて動いても panic しない —
    /// `key_write_order` はどんな入力でも常に全添字を返す決定的な並べ替えでしか
    /// ない(衝突の実際の吸収は呼び手の `KeyframeTrack::insert` が担う)。
    #[test]
    fn key_write_order_never_panics_when_two_keys_converge_on_the_same_target() {
        let origins = vec![10, 12];
        // 両方とも frame 50 へ向けて動く状況を模す(delta は代表値だけ渡す)。
        let order = key_write_order(&origins, 40);
        assert_eq!(order.len(), 2);
        assert!(order.contains(&0) && order.contains(&1));
    }
}
