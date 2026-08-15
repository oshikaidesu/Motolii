//! `timeline_egui/geometry.rs` の**写し**。座標をここで決めないため、
//! 式・定数・関数名を1対1で保つ。
//!
//! 違いは `egui::Rect` を取らず原点(0,0)の幅高で受ける点だけ。
//! Blitz文書は自分の左上が原点であり、`rect.left()`/`rect.top()` は常に 0 になる。

use motolii_core::RationalTime;

use super::theme::{LOCATOR_H, OVERVIEW_H, ROW_H, RULER_H};

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
            // **行高は固定・最小 20px**(2026-08-08決定, 利用者裁定 2026-08-16 で適用)。
            // 理由は「コンポジットのタイムラインは縦が情報を持たない」ため。
            // 移植元(`timeline_egui/geometry.rs:27`, 撤去済み)は面の高さで
            // `clamp(20.0, 24.0)` と伸縮させており、**決定に反していた**。
            // `row_count` はもう使わないが、呼び手の引数は変えない
            // (行数は他の寸法計算で使う可能性があるため、ここでの不使用に留める)。
            row_height: ROW_H,
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
        // 行高は面の高さにも行数にも依らず 20px 固定(2026-08-08決定(3))。
        // 移植元は clamp(20,24) で伸縮させていたが、決定に反していた。
        assert_eq!(geometry.row_height, 20.0);
        assert_eq!(
            TimelineGeometry::new(1000.0, 460.0, 40, 10.0).row_height,
            20.0
        );
        assert_eq!(
            TimelineGeometry::new(1000.0, 2000.0, 2, 10.0).row_height,
            20.0
        );
    }
}
