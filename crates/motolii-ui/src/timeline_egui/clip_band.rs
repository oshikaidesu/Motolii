//! clip帯の描画とhit。行の見た目と掴み位置だけをここで決める。

use egui::{Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};
use motolii_doc::LayerId;

use super::geometry::TimelineGeometry;
use super::input::EguiTimelineHit;
use super::rows::TimelineRow;
use super::theme::{ACCENT, BAR_INK, CONTRAST, DIM, INK, PALETTE, SURFACE};
use crate::timeline_projection::TimelineProjection;

pub(super) fn paint_rows(
    painter: &Painter,
    geometry: &TimelineGeometry,
    rows: &[TimelineRow],
    primary: Option<LayerId>,
) {
    let rect = geometry.rect;
    let selected_index = primary.as_ref().and_then(|layer| {
        rows.iter()
            .position(|row| row.property.is_none() && row.layer == *layer)
    });
    for (index, row) in rows.iter().enumerate() {
        let y = geometry.rows_top + index as f32 * geometry.row_height;
        if y >= rect.bottom() {
            break;
        }
        let cy = y + geometry.row_height * 0.5;
        let selected = row.property.is_none() && selected_index == Some(index);
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), y),
                Pos2::new(
                    rect.right(),
                    (y + geometry.row_height - 1.0).min(rect.bottom()),
                ),
            ),
            CornerRadius::ZERO,
            if selected {
                Color32::from_rgb(0x41, 0x41, 0x41)
            } else {
                SURFACE
            },
        );
        for grid_index in 0..=10 {
            let x = geometry.x_at(grid_index as f32 / 10.0);
            painter.line_segment(
                [
                    Pos2::new(x, y),
                    Pos2::new(x, (y + geometry.row_height - 1.0).min(rect.bottom())),
                ],
                Stroke::new(
                    1.0,
                    if grid_index % 2 == 0 {
                        Color32::from_rgb(0x26, 0x26, 0x26)
                    } else {
                        Color32::from_rgb(0x30, 0x30, 0x30)
                    },
                ),
            );
        }
        painter.line_segment(
            [
                Pos2::new(rect.left(), y + geometry.row_height - 1.0),
                Pos2::new(rect.right(), y + geometry.row_height - 1.0),
            ],
            Stroke::new(1.0, CONTRAST),
        );

        if row.property.is_some() {
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.left() + 11.0, cy - 1.0),
                    egui::Vec2::splat(2.0),
                ),
                CornerRadius::ZERO,
                Color32::from_rgb(0x92, 0x92, 0x92),
            );
        } else {
            draw_disclosure(painter, rect.left() + 10.0, cy, true);
        }
        painter.text(
            Pos2::new(
                rect.left() + if row.property.is_some() { 31.0 } else { 20.0 },
                cy - 5.0,
            ),
            Align2::LEFT_TOP,
            row.label.as_str(),
            FontId::proportional(9.0),
            if row.property.is_some() { DIM } else { INK },
        );

        if row.property.is_none() {
            draw_toggle(painter, geometry.surface_left() - 48.0, cy, "M", false);
            draw_toggle(painter, geometry.surface_left() - 31.0, cy, "S", false);
        }

        if let (Some(start), Some(end)) = (row.start, row.end) {
            let x0 = geometry.x_at(start);
            let x1 = geometry.x_at(end);
            let bar_rect = Rect::from_min_max(
                Pos2::new(x0, y + 1.0),
                Pos2::new(
                    x1.max(x0 + 1.0),
                    (y + geometry.row_height - 3.0).min(rect.bottom()),
                ),
            );
            painter.rect_filled(bar_rect, CornerRadius::ZERO, PALETTE[row.palette_slot]);
            painter.text(
                Pos2::new(x0 + 6.0, cy - 5.0),
                Align2::LEFT_TOP,
                row.label.as_str(),
                FontId::proportional(9.0),
                BAR_INK,
            );
            if selected {
                painter.rect_stroke(
                    Rect::from_min_max(
                        Pos2::new(x0 + 0.5, y + 1.5),
                        Pos2::new(x1.max(x0 + 1.0) - 0.5, y + geometry.row_height - 2.5),
                    ),
                    CornerRadius::ZERO,
                    Stroke::new(1.0, Color32::WHITE),
                    StrokeKind::Inside,
                );
            }
        }
        for (key_index, key) in row.keys.iter().enumerate() {
            let x = geometry.x_at(*key);
            draw_diamond(painter, x, cy, selected && key_index == 0);
        }
    }

    painter.line_segment(
        [
            Pos2::new(geometry.surface_left(), rect.top()),
            Pos2::new(geometry.surface_left(), rect.bottom()),
        ],
        Stroke::new(1.0, CONTRAST),
    );
}

pub(crate) fn classify_bar_edge(
    bar_x_start: f32,
    bar_x_end: f32,
    fraction: f32,
    surface_width: f32,
    layer: LayerId,
) -> EguiTimelineHit {
    let bar_width_px = (bar_x_end - bar_x_start) * surface_width;
    if !bar_width_px.is_finite() || bar_width_px < 25.0 {
        return EguiTimelineHit::Body { layer };
    }
    let edge_width = 15.0_f32.min(bar_width_px / 4.0);
    let local_x = (fraction - bar_x_start) * surface_width;
    if !local_x.is_finite() || !edge_width.is_finite() {
        EguiTimelineHit::Body { layer }
    } else if local_x <= edge_width {
        EguiTimelineHit::Left { layer }
    } else if local_x >= bar_width_px - edge_width {
        EguiTimelineHit::Right { layer }
    } else {
        EguiTimelineHit::Body { layer }
    }
}

pub(super) fn hit_at(
    pos: Pos2,
    geometry: &TimelineGeometry,
    rows: &[TimelineRow],
    projection: Option<&TimelineProjection>,
) -> EguiTimelineHit {
    let Some(projection) = projection else {
        return EguiTimelineHit::None;
    };
    if !geometry.in_surface(pos) || pos.y < geometry.rows_top || geometry.row_height <= 0.0 {
        return EguiTimelineHit::None;
    }
    let index = ((pos.y - geometry.rows_top) / geometry.row_height).floor() as usize;
    let Some(row) = rows.get(index).filter(|row| row.property.is_none()) else {
        return EguiTimelineHit::None;
    };
    let fraction = ((pos.x - geometry.surface_left()) / geometry.surface_width).clamp(0.0, 1.0);
    let key_hit = projection
        .keys()
        .iter()
        .filter(|key| key.layer == row.layer)
        .find(|key| (key.center_x as f32 - fraction).abs() <= 0.012);
    if let Some(key) = key_hit {
        return EguiTimelineHit::Key {
            layer: key.layer,
            key: key.key,
        };
    }
    let Some(bar) = projection.bars().iter().find(|bar| {
        bar.layer == row.layer && fraction >= bar.x_start as f32 && fraction < bar.x_end as f32
    }) else {
        return EguiTimelineHit::None;
    };
    classify_bar_edge(
        bar.x_start as f32,
        bar.x_end as f32,
        fraction,
        geometry.surface_width,
        row.layer,
    )
}

fn draw_disclosure(painter: &Painter, x: f32, y: f32, open: bool) {
    let points = if open {
        vec![
            Pos2::new(x - 4.0, y - 2.0),
            Pos2::new(x + 4.0, y - 2.0),
            Pos2::new(x, y + 3.0),
        ]
    } else {
        vec![
            Pos2::new(x - 2.0, y - 4.0),
            Pos2::new(x + 3.0, y),
            Pos2::new(x - 2.0, y + 4.0),
        ]
    };
    painter.add(Shape::convex_polygon(
        points,
        Color32::from_rgb(0xa5, 0xa5, 0xa5),
        Stroke::NONE,
    ));
}

fn draw_toggle(painter: &Painter, x: f32, y: f32, label: &str, active: bool) {
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(x, y - 7.0), Pos2::new(x + 14.0, y + 7.0)),
        CornerRadius::ZERO,
        if active {
            ACCENT
        } else {
            Color32::from_rgb(0x5d, 0x5d, 0x5d)
        },
    );
    painter.text(
        Pos2::new(x + 3.0, y - 5.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(8.0),
        if active {
            BAR_INK
        } else {
            Color32::from_rgb(0xd0, 0xd0, 0xd0)
        },
    );
}

fn draw_diamond(painter: &Painter, x: f32, y: f32, selected: bool) {
    let color = if selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(0xc5, 0xb8, 0x6c)
    };
    painter.add(Shape::convex_polygon(
        vec![
            Pos2::new(x, y - 4.0),
            Pos2::new(x + 4.0, y),
            Pos2::new(x, y + 4.0),
            Pos2::new(x - 4.0, y),
        ],
        if selected {
            Color32::WHITE
        } else {
            Color32::TRANSPARENT
        },
        if selected {
            Stroke::NONE
        } else {
            Stroke::new(1.0, color)
        },
    ));
}
