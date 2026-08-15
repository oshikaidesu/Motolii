// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release), `src/tree.rs`.
//
// なぜこのファイルか: 移植元は `egui::Id`/`Context` で egui のメモリストアへ
// ドラッグ状態(`smooth_preview_rect` 等)を置いていた。C4 capsule が言う「HashMap 一つ」がこれ。
// 元と同じく **永続化しない**(`Tree` 側で `serde(skip)`)。毎フレーム `Tree` を作り直しても
// プレビューが滑らかに続く、という元の性質もそのまま残る。

use std::collections::HashMap;

use super::geom::{exponential_smooth_factor, Pos2, Rect};
use super::interaction::SplitterId;
use super::TileId;

/// Transient drag-and-drop state, in place of egui's memory store.
#[derive(Clone, Debug, Default)]
pub struct DragState {
    /// The tile the user is currently dragging, if any.
    pub dragged_tile: Option<TileId>,

    /// The splitter the user is currently dragging, if any.
    pub active_splitter: Option<SplitterId>,

    /// Where the primary button went down, for telling a click from the start of a drag.
    ///
    /// 元は egui の interaction state が持っていたもの。
    pub press_origin: Option<Pos2>,

    /// The smoothed drop-preview rectangle, per dragged tile.
    ///
    /// 元は `egui::Id::new((dragged_tile_id, "smoothed_preview_rect"))` を鍵にした temp storage。
    smooth_preview_rect: HashMap<TileId, Rect>,

    /// Set when a smoothing step is still in flight, so the host knows to draw another frame.
    ///
    /// 元の `ctx.request_repaint()` の置き換え。
    pub needs_repaint: bool,
}

impl DragState {
    /// Is this tile the one being dragged?
    #[inline]
    pub fn is_being_dragged(&self, tile_id: TileId) -> bool {
        self.dragged_tile == Some(tile_id)
    }

    pub fn clear_smooth_preview_rect(&mut self, dragged_tile_id: TileId) {
        self.smooth_preview_rect.remove(&dragged_tile_id);
    }

    /// Take the preview rectangle and smooth it over time.
    pub fn smooth_preview_rect(
        &mut self,
        dragged_tile_id: TileId,
        new_rect: Rect,
        stable_dt: f32,
    ) -> Rect {
        let dt = stable_dt.min(0.1);

        let mut requires_repaint = false;

        let smoothed = {
            let smoothed = self
                .smooth_preview_rect
                .entry(dragged_tile_id)
                .or_insert(new_rect);

            let t = exponential_smooth_factor(0.9, 0.05, dt);

            *smoothed = smoothed.lerp_towards(&new_rect, t);

            let diff = smoothed.min.distance(new_rect.min) + smoothed.max.distance(new_rect.max);
            if diff < 0.5 {
                *smoothed = new_rect;
            } else {
                requires_repaint = true;
            }
            *smoothed
        };

        if requires_repaint {
            self.needs_repaint = true;
        }

        smoothed
    }
}
