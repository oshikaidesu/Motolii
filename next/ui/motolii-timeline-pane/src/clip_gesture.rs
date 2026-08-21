//! 単一クリップの move/trim ジェスチャの**意味関数**(第2波T2、正典
//! `next/reference/timeline-grammar.md` §2)。ここに在るのは全部**純関数**で、
//! `motolii_store::LayerTiming` を直接は持たない — `i64`(フレーム)と
//! `f32`(px/frame)だけを受けて答えを返す。呼び手(`Shell::update`、
//! `lib.rs`)が結果を `LayerTiming` へ詰め直す。
//!
//! この形にしてある理由は `super::projection`/`super::hit` と同じ:
//! Document 型を持ち込まないことで、`timeline/` の外(motolii-store)を
//! 一切 import せずに単体で試験できる(このファイルは自己完結 — mod.rs は
//! `mod clip_gesture;` の結線1行だけ)。
//!
//! ## 移動/trim の共通の形
//!
//! - **掴んだ瞬間の値(`origin_*`)を基準に絶対値で出し直す**(delta 蓄積禁止、
//!   正典 §2)。呼び手は毎 move で `origin_*` をそのまま渡し直すこと —
//!   前回の結果を次の入力に混ぜない
//! - スナップは**画面距離**(`px_per_frame` で frame へ換算、正典 §1 `SNAP_PX`)。
//!   候補外・`px_per_frame <= 0.0` の時は素通り(吸着なし)
//! - clamp は常にスナップの**後**(スナップ結果が画面外・尺の外に出ても
//!   comp/1フレーム最小の床は破らない)

use motolii_store::LayerId;

use super::projection::RowProjection;

/// 吸着の画面距離しきい値(px)。正典 §1 `SNAP_PX` と同値。
pub const SNAP_PX: f32 = 7.0;

/// 吸着候補: 0秒・終端・playhead・**他 clip**(grabbed 自身は除く)の start/end
/// (正典 §2「スナップ対象」)。キー時刻はキー行がまだ無いので対象外
/// (`next/DECISIONS.md` 裁定151 の T2 発注書 KNOWN)。
pub fn snap_candidates(
    rows: &[RowProjection],
    grabbed: LayerId,
    playhead: i64,
    duration_frames: i64,
) -> Vec<i64> {
    let mut out = vec![0, duration_frames.max(0), playhead];
    for row in rows {
        if row.id == grabbed {
            continue;
        }
        out.push(row.start);
        out.push(row.start + row.duration);
    }
    out
}

/// `frame` を最も近い候補へ吸着する。**間合いは画面距離**
/// (`px_per_frame` で frame へ換算 — ズームしても同じ「指の許容」になる、
/// 正典 §1)。窓の外・`px_per_frame <= 0.0`(0除算防止、M16)は素通り。
pub fn snap_frame(frame: i64, candidates: &[i64], px_per_frame: f32) -> i64 {
    if px_per_frame <= 0.0 {
        return frame;
    }
    let window = (SNAP_PX / px_per_frame).ceil() as i64;
    candidates
        .iter()
        .copied()
        .filter(|candidate| (candidate - frame).abs() <= window)
        .min_by_key(|candidate| (candidate - frame).abs())
        .unwrap_or(frame)
}

/// move(本体ドラッグ)の新しい start。**尺は変わらない** — comp の外へは出さない
/// (`[0, comp_duration - origin_duration]` へ clamp、負の余地は0に潰す)。
#[allow(clippy::too_many_arguments)]
pub fn moved_start(
    origin_start: i64,
    origin_duration: i64,
    grab_at_frame: i64,
    current_frame: i64,
    comp_duration: i64,
    candidates: &[i64],
    px_per_frame: f32,
    snap_enabled: bool,
) -> i64 {
    let raw = origin_start + (current_frame - grab_at_frame);
    let target = if snap_enabled {
        snap_frame(raw, candidates, px_per_frame)
    } else {
        raw
    };
    let max_start = (comp_duration - origin_duration).max(0);
    target.clamp(0, max_start)
}

/// trim In(頭側の端)の新しい start。**1フレーム未満に潰さない・0未満へは
/// 出ない**(`end` は固定 — 正典 §2「trim(端8px)…単独 clip の start/end のみ
/// 動かす」)。
pub fn trimmed_in_start(
    end: i64,
    current_frame: i64,
    candidates: &[i64],
    px_per_frame: f32,
    snap_enabled: bool,
) -> i64 {
    let target = if snap_enabled {
        snap_frame(current_frame, candidates, px_per_frame)
    } else {
        current_frame
    };
    target.clamp(0, (end - 1).max(0))
}

/// trim Out(尻側の端)の新しい end。**1フレーム未満に潰さない・comp の外へは
/// 出ない**(`start` は固定)。
pub fn trimmed_out_end(
    start: i64,
    current_frame: i64,
    comp_duration: i64,
    candidates: &[i64],
    px_per_frame: f32,
    snap_enabled: bool,
) -> i64 {
    let target = if snap_enabled {
        snap_frame(current_frame, candidates, px_per_frame)
    } else {
        current_frame
    };
    target.clamp(start + 1, comp_duration.max(start + 1))
}

/// speed 変更に伴う duration の再計算(SP1 第一波= Inspector 数値欄、第二波=
/// Shift+bar端 rate stretch が共有する純関数、δ 採択理由・supervisor 決定4)。
/// **start・source_in はここでは触らない**(呼び手が不変に保つ責務、
/// `motolii-inspector-pane`/第二波どちらの呼び手も同じ契約)。
///
/// `new_duration = round(old_duration × old_speed / new_speed)`、**最低1フレーム**
/// (0以下・尺なしへは潰さない、M13)。speed は `motolii_store::Speed` の
/// `(num, den)` をそのまま tuple で受ける — `motolii-store` を import しない
/// 自己完結を保つため(ファイル冒頭 doc の理由と同じ)。`den` は
/// `Speed::try_new` の不変式により本来は常に正だが、ここでは呼び手の契約違反
/// (0)を安全側で受け止め、0除算はせず `old_duration` をそのまま返す。
///
/// 丸めは四捨五入(round-half-away-from-zero、`f64` を経由しない — `i128` の
/// 整数演算のみ、Speed 型自身の「f64 を経由しない」設計と合わせる)。
pub fn retimed_duration(old_duration: i64, old_speed: (i64, i64), new_speed: (i64, i64)) -> i64 {
    let (old_num, old_den) = old_speed;
    let (new_num, new_den) = new_speed;
    if old_den == 0 || new_den == 0 || new_num == 0 {
        return old_duration.max(1);
    }
    let numerator = old_duration as i128 * old_num as i128 * new_den as i128;
    let denominator = old_den as i128 * new_num as i128;
    let rounded = round_div_i128(numerator, denominator);
    rounded.clamp(1, i64::MAX as i128) as i64
}

/// `n / d` を四捨五入(round-half-away-from-zero)。`d == 0` は呼び手が既に
/// 弾いている(`retimed_duration` 参照)。
fn round_div_i128(n: i128, d: i128) -> i128 {
    let d_abs = d.abs();
    let n_abs = n.abs();
    let q = (n_abs + d_abs / 2) / d_abs;
    if (n < 0) != (d < 0) {
        -q
    } else {
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u64, start: i64, duration: i64) -> RowProjection {
        RowProjection {
            id: LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked: false,
            label_color: None,
            start,
            duration,
            selected: false,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    #[test]
    fn candidates_exclude_the_grabbed_clip_itself() {
        let rows = vec![row(1, 10, 20), row(2, 200, 30)];
        let candidates = snap_candidates(&rows, LayerId(1), 5, 300);
        assert!(!candidates.contains(&10), "掴んでいる自分の start を候補に含めている");
        assert!(!candidates.contains(&30), "掴んでいる自分の end を候補に含めている");
        assert!(candidates.contains(&0));
        assert!(candidates.contains(&300));
        assert!(candidates.contains(&5), "playhead が候補に無い");
        assert!(candidates.contains(&200));
        assert!(candidates.contains(&230));
    }

    /// **間合いは画面距離**: 縮尺が変われば同じ frame 差でも吸着可否が変わる。
    #[test]
    fn snapping_uses_screen_distance_not_frame_distance() {
        let candidates = vec![100];
        // 1px == 1frame: 7frame以内は吸着。
        assert_eq!(snap_frame(105, &candidates, 1.0), 100);
        assert_eq!(snap_frame(108, &candidates, 1.0), 108, "窓の外まで吸着している");
        // 0.5px == 1frame(ズームアウト): 同じ7frame差でも画面距離は3.5pxなので吸着。
        assert_eq!(snap_frame(107, &candidates, 0.5), 100);
    }

    #[test]
    fn zero_scale_never_divides_by_zero_and_never_snaps() {
        assert_eq!(snap_frame(103, &[100], 0.0), 103, "px_per_frame<=0 は素通りのはず");
    }

    /// 正典 §2: 掴んだ瞬間の値を基準に絶対値で出し直す — 同じ `current_frame`
    /// に2回呼んでも結果は同じ(delta が蓄積しない)。
    #[test]
    fn moved_start_recomputes_from_origin_each_call() {
        let first = moved_start(100, 50, 120, 140, 300, &[], 0.0, false);
        let second = moved_start(100, 50, 120, 140, 300, &[], 0.0, false);
        assert_eq!(first, second);
        assert_eq!(first, 100 + (140 - 120));
    }

    #[test]
    fn moved_start_clamps_to_the_composition() {
        // 尺50、comp300、左端に寄せようとしても0未満にはならない。
        assert_eq!(moved_start(10, 50, 100, 0, 300, &[], 0.0, false), 0);
        // 尺50、comp300、右へ寄せても comp の外(250 超)へは出ない。
        assert_eq!(moved_start(10, 50, 100, 1000, 300, &[], 0.0, false), 250);
    }

    #[test]
    fn trim_in_keeps_the_end_fixed_and_never_crosses_it() {
        assert_eq!(trimmed_in_start(200, 150, &[], 0.0, false), 150);
        // end(200)の右へは出られない(最短1フレーム保証)。
        assert_eq!(trimmed_in_start(200, 250, &[], 0.0, false), 199);
        assert_eq!(trimmed_in_start(200, -50, &[], 0.0, false), 0);
    }

    #[test]
    fn trim_out_keeps_the_start_fixed_and_never_crosses_it() {
        assert_eq!(trimmed_out_end(100, 150, 300, &[], 0.0, false), 150);
        // start(100)の左へは出られない(最短1フレーム保証)。
        assert_eq!(trimmed_out_end(100, 50, 300, &[], 0.0, false), 101);
        // comp の外へは出ない。
        assert_eq!(trimmed_out_end(100, 1000, 300, &[], 0.0, false), 300);
    }

    // -----------------------------------------------------------------------
    // SP1 第一波: speed 変更に伴う duration 再計算(ORACLE (a))
    // -----------------------------------------------------------------------

    #[test]
    fn retimed_duration_halves_at_double_speed() {
        assert_eq!(retimed_duration(100, (1, 1), (2, 1)), 50);
    }

    #[test]
    fn retimed_duration_doubles_at_half_speed() {
        assert_eq!(retimed_duration(100, (1, 1), (1, 2)), 200);
    }

    #[test]
    fn retimed_duration_rounds_to_nearest_frame() {
        // 5 frame を 1→3倍速(300%)にすると 5/3 = 1.666… → 2 frame に丸まる。
        assert_eq!(retimed_duration(5, (1, 1), (3, 1)), 2);
    }

    #[test]
    fn retimed_duration_clamps_to_a_minimum_of_one_frame() {
        // 1 frame をさらに速くしても 0 以下には潰れない。
        assert_eq!(retimed_duration(1, (1, 1), (100, 1)), 1);
    }

    #[test]
    fn retimed_duration_is_a_no_op_at_the_same_speed() {
        assert_eq!(retimed_duration(123, (1, 1), (1, 1)), 123);
    }

    #[test]
    fn retimed_duration_never_divides_by_zero() {
        // 契約違反(den=0/new_num=0)は起こらないはずだが、安全側で old_duration を返す。
        assert_eq!(retimed_duration(50, (1, 0), (2, 1)), 50);
        assert_eq!(retimed_duration(50, (1, 1), (0, 1)), 50);
        assert_eq!(retimed_duration(50, (1, 1), (2, 0)), 50);
    }
}
