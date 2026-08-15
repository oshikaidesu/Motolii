//! 時間ヘッダ(overview／ruler／locator)とplayhead。行の中身には触れない。

use egui::{Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};
use motolii_core::RationalTime;

use super::geometry::TimelineGeometry;
use super::rows::TimelineRow;
use super::theme::{
    DIM, LOCATOR_H, OVERVIEW_H, PALETTE, RULER, RULER_H, SURFACE, SURFACE_HI, SURFACE_LO,
};

pub(super) fn paint_overview(painter: &Painter, geometry: &TimelineGeometry, rows: &[TimelineRow]) {
    let rect = geometry.rect;
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(geometry.surface_left(), rect.top()),
            Pos2::new(rect.right(), rect.top() + OVERVIEW_H),
        ),
        CornerRadius::ZERO,
        SURFACE_LO,
    );
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 4.0),
        Align2::LEFT_TOP,
        "overview",
        FontId::proportional(9.0),
        DIM,
    );

    for (index, row) in rows.iter().enumerate() {
        if row.property.is_some() {
            continue;
        }
        if let (Some(start), Some(end)) = (row.start, row.end) {
            let x0 = geometry.x_at(start);
            let x1 = geometry.x_at(end);
            if x1 > x0 {
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(x0, rect.top() + 4.0 + index as f32 * 4.0),
                        Pos2::new(x1, rect.top() + 7.0 + index as f32 * 4.0),
                    ),
                    CornerRadius::ZERO,
                    PALETTE[row.palette_slot],
                );
            }
        }
    }
    painter.rect_stroke(
        Rect::from_min_max(
            Pos2::new(geometry.surface_left() + 1.0, rect.top() + 1.0),
            Pos2::new(rect.right() - 2.0, rect.top() + OVERVIEW_H - 2.0),
        ),
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(0xd8, 0xd8, 0xd8)),
        StrokeKind::Inside,
    );
}

pub(super) fn paint_ruler(painter: &Painter, geometry: &TimelineGeometry) {
    let rect = geometry.rect;
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + OVERVIEW_H + 1.0),
            Pos2::new(rect.right(), rect.top() + OVERVIEW_H + 1.0 + RULER_H),
        ),
        CornerRadius::ZERO,
        SURFACE_HI,
    );
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + OVERVIEW_H + 3.0),
        Align2::LEFT_TOP,
        "Inbox",
        FontId::proportional(9.0),
        Color32::from_rgb(0xc0, 0xc0, 0xc0),
    );
    for index in 0..=10 {
        let x = geometry.x_at(index as f32 / 10.0);
        painter.line_segment(
            [
                Pos2::new(x, rect.top() + OVERVIEW_H + RULER_H - 5.0),
                Pos2::new(x, rect.top() + OVERVIEW_H + RULER_H),
            ],
            Stroke::new(1.0, Color32::from_rgb(0x6a, 0x6a, 0x6a)),
        );
        painter.text(
            Pos2::new(x + 3.0, rect.top() + OVERVIEW_H + 3.0),
            Align2::LEFT_TOP,
            format!("{index}s"),
            FontId::proportional(8.0),
            RULER,
        );
    }
}

pub(super) fn paint_locators(painter: &Painter, geometry: &TimelineGeometry) {
    let rect = geometry.rect;
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + OVERVIEW_H + RULER_H + 1.0),
            Pos2::new(
                rect.right(),
                rect.top() + OVERVIEW_H + RULER_H + 1.0 + LOCATOR_H,
            ),
        ),
        CornerRadius::ZERO,
        SURFACE,
    );
    for (fraction, label) in [(0.0, "intro"), (0.4, "middle"), (0.8, "out")] {
        let x = geometry.x_at(fraction);
        painter.line_segment(
            [
                Pos2::new(x, rect.top() + OVERVIEW_H + RULER_H + 1.0),
                Pos2::new(x, geometry.rows_top - 1.0),
            ],
            Stroke::new(1.0, Color32::from_rgb(0x8a, 0x8a, 0x8a)),
        );
        painter.text(
            Pos2::new(x + 7.0, geometry.rows_top - 12.0),
            Align2::LEFT_TOP,
            label,
            FontId::proportional(8.0),
            Color32::from_rgb(0xa8, 0xa8, 0xa8),
        );
    }
}

pub(super) fn paint_playhead(
    painter: &Painter,
    geometry: &TimelineGeometry,
    playhead: RationalTime,
    row_count: usize,
) {
    let rect = geometry.rect;
    let playhead_x = geometry.x_at(geometry.playhead_fraction(playhead));
    painter.line_segment(
        [
            Pos2::new(playhead_x, rect.top() + OVERVIEW_H + RULER_H),
            Pos2::new(
                playhead_x,
                (geometry.rows_top + row_count as f32 * geometry.row_height).min(rect.bottom()),
            ),
        ],
        Stroke::new(1.0, Color32::from_rgb(0xe7, 0xe7, 0xe7)),
    );
    painter.add(Shape::convex_polygon(
        vec![
            Pos2::new(playhead_x - 4.0, rect.top() + OVERVIEW_H + RULER_H - 6.0),
            Pos2::new(playhead_x + 5.0, rect.top() + OVERVIEW_H + RULER_H - 6.0),
            Pos2::new(playhead_x + 0.5, rect.top() + OVERVIEW_H + RULER_H),
        ],
        Color32::from_rgb(0xe7, 0xe7, 0xe7),
        Stroke::NONE,
    ));
}
