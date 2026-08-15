//! Timeline席の座標。描画・hit・入力が同じ一本の変換を共有するため。

use egui::{Pos2, Rect};
use motolii_core::RationalTime;

use super::theme::{LOCATOR_H, OVERVIEW_H, RULER_H};

#[derive(Debug, Clone, Copy)]
pub(super) struct TimelineGeometry {
    pub(super) rect: Rect,
    pub(super) sidebar_width: f32,
    pub(super) surface_width: f32,
    pub(super) rows_top: f32,
    pub(super) row_height: f32,
    pub(super) duration: f64,
}

impl TimelineGeometry {
    pub(super) fn new(rect: Rect, row_count: usize, duration: f64) -> Self {
        let sidebar_width = (rect.width() * 0.255).clamp(140.0, 204.0);
        let sidebar_width = sidebar_width.min((rect.width() - 1.0).max(1.0));
        Self {
            rect,
            sidebar_width,
            surface_width: (rect.width() - sidebar_width).max(1.0),
            rows_top: rect.top() + OVERVIEW_H + RULER_H + LOCATOR_H + 1.0,
            row_height: ((rect.height() - 72.0) / row_count.max(1) as f32).clamp(20.0, 24.0),
            duration,
        }
    }

    pub(super) fn surface_left(&self) -> f32 {
        self.rect.left() + self.sidebar_width
    }

    pub(super) fn x_at(&self, fraction: f32) -> f32 {
        self.surface_left() + fraction.clamp(0.0, 1.0) * self.surface_width
    }

    pub(super) fn in_surface(&self, pos: Pos2) -> bool {
        pos.x >= self.surface_left()
    }

    pub(super) fn row_index_at(&self, y: f32) -> Option<usize> {
        if self.row_height <= 0.0 {
            return None;
        }
        Some(((y - self.rows_top) / self.row_height).floor() as usize)
    }

    /// surface上のxをcomposition時刻へ。sidebar上と非有限durationでは時刻を作らない。
    pub(super) fn time_at(&self, pos: Pos2) -> Option<RationalTime> {
        if !self.in_surface(pos) || self.surface_width <= 0.0 || !self.duration.is_finite() {
            return None;
        }
        let fraction = ((pos.x - self.surface_left()) / self.surface_width).clamp(0.0, 1.0);
        let millis = (f64::from(fraction) * self.duration.max(0.0) * 1000.0).round() as i64;
        RationalTime::try_new(millis, 1000).ok()
    }

    pub(super) fn playhead_fraction(&self, playhead: RationalTime) -> f32 {
        if self.duration.is_finite() && self.duration > 0.0 {
            (playhead.as_seconds_f64() / self.duration).clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }
}
