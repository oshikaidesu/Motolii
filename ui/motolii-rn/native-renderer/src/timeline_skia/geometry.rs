//! 時間面の座標・view・snap。paint/hit/sessionが同じ写像を使うため。

use super::layout::{
    LOC_H, MIN_CLIP_BARS, MIN_VIEW_SPAN, OVER_H, ROW, RULER_H, SNAP_THRESHOLD_LOGICAL_PX, SURF_X,
    TIME_H, W,
};
use super::scene::TimelineScene;

/// 範囲が逆転していてもpanicしないclamp(逆転時はminへ寄せる)。
/// gesture中のscene差し替え等でclipが隙間より長い縮退があり得るため、
/// `f64::clamp`のmin>max panicを構造的に排除する。
pub(super) fn clamp_ordered(v: f64, lo: f64, hi: f64) -> f64 {
    if !v.is_finite() || !lo.is_finite() || !hi.is_finite() {
        return if lo.is_finite() { lo } else { 0.0 };
    }
    if hi < lo {
        lo
    } else {
        v.clamp(lo, hi)
    }
}

pub(super) fn clamp_ordered_f32(v: f32, lo: f32, hi: f32) -> f32 {
    clamp_ordered(f64::from(v), f64::from(lo), f64::from(hi)) as f32
}

pub(super) fn surface_width() -> f64 {
    f64::from(W - SURF_X - 6.0)
}

pub(super) fn bx(scene: &TimelineScene, b: f32) -> f32 {
    SURF_X + (b - scene.view_a) / (scene.view_b - scene.view_a) * (W - SURF_X - 6.0)
}

pub(super) fn bar_at_lx(scene: &TimelineScene, lx: f64) -> f64 {
    f64::from(scene.view_a)
        + (lx - f64::from(SURF_X)) / surface_width() * f64::from(scene.view_b - scene.view_a)
}

pub(super) fn overview_bar_at_lx(scene: &TimelineScene, lx: f64) -> f64 {
    ((lx - f64::from(SURF_X)) / surface_width() * f64::from(scene.song_bars))
        .clamp(0.0, f64::from(scene.song_bars))
}

pub(super) fn time_at_lx(scene: &TimelineScene, lx: f64) -> f64 {
    (bar_at_lx(scene, lx) / f64::from(scene.song_bars)).clamp(0.0, 1.0)
}

pub(super) fn clamp_view_translate(scene: &mut TimelineScene) {
    let span = (scene.view_b - scene.view_a)
        .max(0.0)
        .min(scene.song_bars.max(0.0));
    if scene.view_a < 0.0 {
        scene.view_a = 0.0;
        scene.view_b = span;
    }
    if scene.view_b > scene.song_bars {
        scene.view_b = scene.song_bars;
        scene.view_a = (scene.song_bars - span).max(0.0);
    }
    if scene.view_a < 0.0 || scene.view_b < scene.view_a {
        scene.view_a = 0.0;
        scene.view_b = span;
    }
}

pub(super) fn center_view_on(scene: &mut TimelineScene, center: f32) -> bool {
    let span = (scene.view_b - scene.view_a)
        .max(0.0)
        .min(scene.song_bars.max(0.0));
    let mut a = center - span * 0.5;
    let mut b = a + span;
    if a < 0.0 {
        a = 0.0;
        b = span;
    }
    if b > scene.song_bars {
        b = scene.song_bars;
        a = scene.song_bars - span;
    }
    if (scene.view_a - a).abs() > f32::EPSILON || (scene.view_b - b).abs() > f32::EPSILON {
        scene.view_a = a;
        scene.view_b = b;
        true
    } else {
        false
    }
}

pub(super) fn zoom_at(scene: &mut TimelineScene, lx: f64, span_factor: f64) {
    let va = f64::from(scene.view_a);
    let vb = f64::from(scene.view_b);
    let span = vb - va;
    if span <= f64::EPSILON {
        return;
    }
    let anchor = bar_at_lx(scene, lx);
    let min_span = f64::from(scene.song_bars.min(MIN_VIEW_SPAN));
    let new_span =
        (span * span_factor).clamp(min_span, f64::from(scene.song_bars.max(MIN_VIEW_SPAN)));
    let t = ((anchor - va) / span).clamp(0.0, 1.0);
    let mut new_a = anchor - t * new_span;
    let mut new_b = new_a + new_span;
    if new_a < 0.0 {
        new_a = 0.0;
        new_b = new_span;
    }
    if new_b > f64::from(scene.song_bars) {
        new_b = f64::from(scene.song_bars);
        new_a = new_b - new_span;
    }
    scene.view_a = new_a as f32;
    scene.view_b = new_b as f32;
}

pub(super) fn body_top() -> f64 {
    f64::from(OVER_H + 1.0 + RULER_H + LOC_H + 1.0)
}

pub(super) fn body_bottom(scene: &TimelineScene) -> f64 {
    body_top() + f64::from(scene.bands.len() as f32 * ROW + empty_real_guide_rows(scene))
}

pub(super) fn band_index_at_ly(scene: &TimelineScene, ly: f64) -> Option<usize> {
    let top = body_top();
    if ly < top {
        return None;
    }
    let index = ((ly - top) / f64::from(ROW)) as usize;
    if index < scene.bands.len() {
        Some(index)
    } else {
        None
    }
}

pub(super) fn neighbors(scene: &TimelineScene, band: usize, clip_idx: usize) -> (f32, f32) {
    let clips = &scene.bands[band].clips;
    let prev_b = if clip_idx == 0 {
        0.0
    } else {
        clips[clip_idx - 1].b
    };
    let next_a = if clip_idx + 1 >= clips.len() {
        scene.song_bars
    } else {
        clips[clip_idx + 1].a
    };
    (prev_b, next_a)
}

pub(super) fn snap_threshold_bars(scene: &TimelineScene) -> f64 {
    let span = f64::from(scene.view_b - scene.view_a);
    SNAP_THRESHOLD_LOGICAL_PX / surface_width() * span
}

pub(super) fn min_clip_units(scene: &TimelineScene) -> f32 {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        return MIN_CLIP_BARS;
    }
    scene.fps_den as f32 / scene.fps_num as f32
}

pub(super) fn snap_to_frame(scene: &TimelineScene, raw: f32) -> f32 {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        return raw.round();
    }
    let frame = (f64::from(raw) * scene.fps_num as f64 / scene.fps_den as f64).round();
    (frame * scene.fps_den as f64 / scene.fps_num as f64) as f32
}

/// フレーム格子・同band他clip端・playheadへ、画面6論理px閾値で吸着。Cmd中は無効。
pub(super) fn snap_bar(
    scene: &TimelineScene,
    playhead: f64,
    band: usize,
    exclude_clip: Option<usize>,
    raw: f32,
    modifiers: u32,
) -> f32 {
    snap_bar_with_guide(scene, playhead, band, exclude_clip, raw, modifiers).0
}

pub(super) fn snap_bar_with_guide(
    scene: &TimelineScene,
    playhead: f64,
    band: usize,
    exclude_clip: Option<usize>,
    raw: f32,
    modifiers: u32,
) -> (f32, Option<f32>) {
    if modifiers & 1 != 0 {
        return (raw, None);
    }
    let threshold = snap_threshold_bars(scene) as f32;
    let mut best_dist = threshold;
    let mut best = raw;
    let mut consider = |candidate: f32| {
        let dist = (candidate - raw).abs();
        if dist <= best_dist {
            best_dist = dist;
            best = candidate;
        }
    };
    consider(snap_to_frame(scene, raw));
    if let Some(band_ref) = scene.bands.get(band) {
        for (idx, clip) in band_ref.clips.iter().enumerate() {
            if Some(idx) == exclude_clip {
                continue;
            }
            consider(clip.a);
            consider(clip.b);
        }
    }
    let playhead_bar = (playhead.clamp(0.0, 1.0) as f32) * scene.song_bars;
    consider(playhead_bar);
    if (best - raw).abs() <= threshold {
        (best, Some(best))
    } else {
        (raw, None)
    }
}

/// test用: snap候補の閾値判定を直接検証する。
#[cfg(test)]
pub(super) fn test_snap_bar(
    scene: &TimelineScene,
    playhead: f64,
    band: usize,
    exclude_clip: Option<usize>,
    raw: f32,
    modifiers: u32,
) -> f32 {
    snap_bar(scene, playhead, band, exclude_clip, raw, modifiers)
}

pub(super) fn empty_real_guide_rows(scene: &TimelineScene) -> f32 {
    if scene.real && scene.bands.is_empty() {
        ROW
    } else {
        0.0
    }
}

/// 描画に使う論理座標系の高さ。
pub(super) fn logical_height(scene: &TimelineScene) -> f32 {
    OVER_H
        + RULER_H
        + LOC_H
        + 1.0
        + scene.bands.len() as f32 * ROW
        + empty_real_guide_rows(scene)
        + TIME_H
        + 2.0
}
