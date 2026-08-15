//! hit優先順位とcursor写像。gesture開始とhoverが同じ判定を共有するため。

use super::geometry::{bar_at_lx, body_bottom, body_top, bx};
use super::layout::{
    scale_for, INBOX_W, KEY_HIT_PX, OVER_H, PLAYHEAD_HIT_PX, ROW, RULER_H, SURF_X, TIME_H,
    TRIM_EDGE_MAX_PX, TRIM_EDGE_MIN_CLIP_H, TRIM_EDGE_MIN_CLIP_W,
};
use super::scene::TimelineScene;

#[derive(Clone, Copy, Debug)]
pub(super) enum HitKind {
    Overview,
    /// playhead頭/線。Scrubと同じ文法へ落とす。
    Playhead,
    Scrub,
    Key {
        band: usize,
        clip_idx: usize,
        key_idx: usize,
        flat: i32,
    },
    TrimStart {
        band: usize,
        clip_idx: usize,
    },
    TrimEnd {
        band: usize,
        clip_idx: usize,
    },
    Clip {
        band: usize,
        clip_idx: usize,
        flat: i32,
    },
    EmptyBar,
    Mute {
        band: usize,
    },
    Solo {
        band: usize,
    },
}

/// hover/cursor用の粗いhit種。gesture内部indexは持たない。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TimelineHoverHit {
    None,
    PlayheadOrRuler,
    Key,
    Trim,
    Clip,
}

/// OSカーソル写像の閉集合。ObjC側は薄い分岐だけ。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CursorKind {
    Arrow = 0,
    ResizeLeftRight = 1,
    OpenHand = 2,
    ClosedHand = 3,
    PointingHand = 4,
}

impl CursorKind {
    pub(crate) fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Timeline hover hit → cursor。clip drag中はhitを外れてもclosedHand維持(gesture active優先)。
pub(super) fn cursor_for_timeline_hover(hit: TimelineHoverHit, clip_dragging: bool) -> CursorKind {
    if clip_dragging {
        return CursorKind::ClosedHand;
    }
    match hit {
        TimelineHoverHit::None => CursorKind::Arrow,
        TimelineHoverHit::PlayheadOrRuler | TimelineHoverHit::Trim => CursorKind::ResizeLeftRight,
        TimelineHoverHit::Key => CursorKind::PointingHand,
        TimelineHoverHit::Clip => CursorKind::OpenHand,
    }
}

/// Stage上の選択可能物hover → cursor。drag中は物体を外れてもclosedHand維持。
pub(super) fn cursor_for_stage_hover(over_layer: bool, dragging: bool) -> CursorKind {
    if dragging {
        return CursorKind::ClosedHand;
    }
    if over_layer {
        CursorKind::OpenHand
    } else {
        CursorKind::Arrow
    }
}

/// 短いbarは左右を推測分割せず、既存body moveへ縮退させる。
pub(super) fn trim_edge_width(clip_width: f64, clip_height: f64) -> Option<f64> {
    if !clip_width.is_finite()
        || !clip_height.is_finite()
        || clip_width < TRIM_EDGE_MIN_CLIP_W
        || clip_height < TRIM_EDGE_MIN_CLIP_H
    {
        return None;
    }
    Some(TRIM_EDGE_MAX_PX.min(clip_width / 4.0))
}

pub(super) fn hit_gesture(
    scene: &TimelineScene,
    playhead: f64,
    lx: f64,
    ly: f64,
) -> Option<HitKind> {
    // 0. overview帯
    if ly < f64::from(OVER_H) && lx >= f64::from(SURF_X) {
        return Some(HitKind::Overview);
    }

    // 1. playhead(頭+線) — ruler/key/trim/clipより先
    if playhead_hit(scene, playhead, lx, ly) {
        return Some(HitKind::Playhead);
    }

    let ry = f64::from(OVER_H + 1.0);
    let time_y0 = body_bottom(scene);
    // 2. ruler帯
    if ((ly >= ry && ly < ry + f64::from(RULER_H))
        || (ly >= time_y0 && ly < time_y0 + f64::from(TIME_H)))
        && lx >= f64::from(SURF_X)
    {
        return Some(HitKind::Scrub);
    }

    let top = body_top();
    let bottom = body_bottom(scene);
    if ly >= top && ly < bottom {
        let band_index = ((ly - top) / f64::from(ROW)) as usize;
        if band_index < scene.bands.len() {
            let cy = top + (band_index as f64 + 0.5) * f64::from(ROW) - 0.5;
            if tog_hit(lx, ly, f64::from(INBOX_W) + 5.0, cy) {
                return Some(HitKind::Mute { band: band_index });
            }
            if tog_hit(lx, ly, f64::from(INBOX_W) + 21.0, cy) {
                return Some(HitKind::Solo { band: band_index });
            }
        }
    }
    if ly < top || ly >= bottom || lx < f64::from(SURF_X) {
        return None;
    }

    let band_index = ((ly - top) / f64::from(ROW)) as usize;
    if band_index >= scene.bands.len() {
        // 空realのガイド帯など、band無しのbodyはEmptyBar扱いにしない(選択解除のみ)。
        return if scene.real && scene.bands.is_empty() {
            Some(HitKind::EmptyBar)
        } else {
            None
        };
    }
    let band = &scene.bands[band_index];
    let row_cy = top + (band_index as f64 + 0.5) * f64::from(ROW) - 0.5;
    let bar = bar_at_lx(scene, lx);

    let mut flat_before = 0i32;
    for (bi, b) in scene.bands.iter().enumerate() {
        if bi == band_index {
            break;
        }
        flat_before += b.clips.len() as i32;
    }

    // 3. key中心±KEY_HIT_PX
    for (clip_idx, clip) in band.clips.iter().enumerate() {
        for (key_idx, (kt, _, _, _)) in clip.keys.iter().enumerate() {
            if *kt < clip.a || *kt > clip.b {
                continue;
            }
            let kx = f64::from(bx(scene, *kt));
            if (lx - kx).abs() <= KEY_HIT_PX && (ly - row_cy).abs() <= KEY_HIT_PX {
                return Some(HitKind::Key {
                    band: band_index,
                    clip_idx,
                    key_idx,
                    flat: flat_before + clip_idx as i32,
                });
            }
        }
    }

    // 4. trim端 / 5. clip本体
    for (clip_idx, clip) in band.clips.iter().enumerate() {
        let ax = f64::from(bx(scene, clip.a));
        let bx_ = f64::from(bx(scene, clip.b));
        if lx >= ax && lx <= bx_ {
            let clip_width = bx_ - ax;
            if let Some(edge_width) = trim_edge_width(clip_width, f64::from(ROW - 1.0)) {
                if lx - ax <= edge_width {
                    return Some(HitKind::TrimStart {
                        band: band_index,
                        clip_idx,
                    });
                }
                if bx_ - lx <= edge_width {
                    return Some(HitKind::TrimEnd {
                        band: band_index,
                        clip_idx,
                    });
                }
            }
            if bar >= f64::from(clip.a) && bar < f64::from(clip.b) {
                return Some(HitKind::Clip {
                    band: band_index,
                    clip_idx,
                    flat: flat_before + clip_idx as i32,
                });
            }
        }
    }

    // 6. 空き面
    Some(HitKind::EmptyBar)
}

pub(super) fn playhead_hit(scene: &TimelineScene, playhead: f64, lx: f64, ly: f64) -> bool {
    if lx < f64::from(SURF_X) {
        return false;
    }
    let bar = (playhead.clamp(0.0, 1.0) as f32) * scene.song_bars;
    if bar < scene.view_a || bar > scene.view_b {
        return false;
    }
    let px = f64::from(bx(scene, bar));
    if (lx - px).abs() > PLAYHEAD_HIT_PX {
        return false;
    }
    let ry = f64::from(OVER_H + 1.0);
    let tri_top = ry + f64::from(RULER_H) - 6.0;
    let time_y0 = body_bottom(scene);
    let line_bottom = time_y0; // 描画線は分秒ruler上端まで
    ly >= tri_top && ly < line_bottom.max(tri_top + 1.0)
}

/// hover位置のhit種。gesture判定と同じ優先順位をpureに返す。
pub(super) fn timeline_hover_hit(
    scene: &TimelineScene,
    playhead: f64,
    width: u32,
    height: u32,
    x: f64,
    y: f64,
) -> TimelineHoverHit {
    if width == 0 || height == 0 {
        return TimelineHoverHit::None;
    }
    let scale = f64::from(scale_for(width));
    let lx = x / scale;
    let ly = y / scale;
    match hit_gesture(scene, playhead, lx, ly) {
        Some(HitKind::Playhead) | Some(HitKind::Scrub) => TimelineHoverHit::PlayheadOrRuler,
        Some(HitKind::Key { .. }) => TimelineHoverHit::Key,
        Some(HitKind::TrimStart { .. }) | Some(HitKind::TrimEnd { .. }) => TimelineHoverHit::Trim,
        Some(HitKind::Clip { .. }) => TimelineHoverHit::Clip,
        Some(HitKind::Mute { .. }) | Some(HitKind::Solo { .. }) => TimelineHoverHit::Clip,
        Some(HitKind::Overview) | Some(HitKind::EmptyBar) | None => TimelineHoverHit::None,
    }
}

pub(super) fn tog_hit(lx: f64, ly: f64, x: f64, cy: f64) -> bool {
    lx >= x && lx < x + 14.0 && ly >= cy - 6.5 && ly < cy + 6.5
}

/// physical pointerを(平坦化clip index, 表示範囲内の正規化時間)へ写す。
///
/// 時間面の外(Inbox / rail / ruler)は`None`。
/// clipに当たらなければ`(-1, time)`を返し、呼び出し側でselection解除に使える。
pub(super) fn hit_test(
    scene: &TimelineScene,
    width: u32,
    height: u32,
    x: f64,
    y: f64,
) -> Option<(i32, f64)> {
    if width == 0 || height == 0 {
        return None;
    }
    let scale = f64::from(scale_for(width));
    let lx = x / scale;
    let ly = y / scale;
    if lx < f64::from(SURF_X) {
        return None;
    }
    let top = body_top();
    let bottom = body_bottom(scene);
    if ly < top || ly >= bottom {
        return None;
    }
    let bar = bar_at_lx(scene, lx);
    let time = time_at_lx(scene, lx);
    let band_index = ((ly - top) / f64::from(ROW)) as usize;

    let mut flat = 0i32;
    for (index, band) in scene.bands.iter().enumerate() {
        for c in &band.clips {
            if index == band_index && bar >= f64::from(c.a) && bar < f64::from(c.b) {
                return Some((flat, time));
            }
            flat += 1;
        }
    }
    Some((-1, time))
}
