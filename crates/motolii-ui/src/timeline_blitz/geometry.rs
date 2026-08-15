//! `timeline_egui/geometry.rs` の**写し**。座標をここで決めないため、
//! 式・定数・関数名を1対1で保つ。
//!
//! 違いは `egui::Rect` を取らず原点(0,0)の幅高で受ける点だけ。
//! Blitz文書は自分の左上が原点であり、`rect.left()`/`rect.top()` は常に 0 になる。

use motolii_core::RationalTime;

use super::theme::{LOCATOR_H, OVERVIEW_H, RULER_H};

#[derive(Debug, Clone, Copy)]
pub(super) struct TimelineGeometry {
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) sidebar_width: f64,
    pub(super) surface_width: f64,
    pub(super) rows_top: f64,
    pub(super) row_height: f64,
    pub(super) duration: f64,
}

impl TimelineGeometry {
    /// timeline_egui/geometry.rs:19-30 の写し。
    pub(super) fn new(width: f64, height: f64, row_count: usize, duration: f64) -> Self {
        let sidebar_width = (width * 0.255).clamp(140.0, 204.0);
        let sidebar_width = sidebar_width.min((width - 1.0).max(1.0));
        Self {
            width,
            height,
            sidebar_width,
            surface_width: (width - sidebar_width).max(1.0),
            rows_top: OVERVIEW_H + RULER_H + LOCATOR_H + 1.0,
            row_height: ((height - 72.0) / row_count.max(1) as f64).clamp(20.0, 24.0),
            duration,
        }
    }

    /// timeline_egui/geometry.rs:32-34 の写し。
    pub(super) fn surface_left(&self) -> f64 {
        self.sidebar_width
    }

    /// timeline_egui/geometry.rs:36-38 の写し。
    pub(super) fn x_at(&self, fraction: f64) -> f64 {
        self.surface_left() + fraction.clamp(0.0, 1.0) * self.surface_width
    }

    /// timeline_egui/geometry.rs:61-67 の写し。
    pub(super) fn playhead_fraction(&self, playhead: RationalTime) -> f64 {
        if self.duration.is_finite() && self.duration > 0.0 {
            (playhead.as_seconds_f64() / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ui_mock.rs:37 が SIDEBAR=204 を「clamp(width*0.255, 140, 204)」の結果として
    /// 書いている。同じ入力で同じ値が出ることを確かめる。
    #[test]
    fn reproduces_the_ui_mock_sidebar_and_row_height() {
        let geometry = TimelineGeometry::new(1000.0, 460.0, 11, 10.0);
        assert_eq!(geometry.sidebar_width, 204.0);
        assert_eq!(geometry.surface_width, 796.0);
        // ui_mock.rs:36 ROWS_TOP=59 と一致。
        assert_eq!(geometry.rows_top, 59.0);
        // 行高はmockの固定値22.0ではなく geometry.rs:27 の式が出所。
        // (460-72)/11 = 35.3 なので上端24へclampされる。
        assert_eq!(geometry.row_height, 24.0);
        assert_eq!(
            TimelineGeometry::new(1000.0, 460.0, 40, 10.0).row_height,
            20.0
        );
    }
}
