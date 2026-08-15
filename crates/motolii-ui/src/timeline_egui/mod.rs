//! egui版Timelineのskin。
//!
//! 責任は各moduleが持ち、ここは組み立てだけを行う。
//! ui/motolii-rn/src/web/TimelineSkiaCanvas.tsx の現行Skia skinを表示面へ移植する。
//! Document投影と編集意味は既存のtimeline_projection／Hostが所有する。

mod clip_band;
mod geometry;
mod input;
mod rows;
mod ruler;
mod theme;

use egui::{Align2, CornerRadius, FontId, Pos2, Rect, Ui, Vec2};
use motolii_core::RationalTime;
use motolii_doc::{Document, LayerId};

use crate::timeline_projection::{
    project_timeline, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
};
use geometry::TimelineGeometry;
use rows::TimelineRow;
use theme::{DESKTOP, DIM, SURFACE};

pub(crate) use clip_band::classify_bar_edge;
pub(crate) use input::{
    timeline_command_for_key, EguiTimelineHit, TimelineCommand, TimelineIntent,
    TimelinePointerPhase, TimelineWheelAction,
};
pub(crate) use theme::TimelineTheme;

#[derive(Debug)]
pub(crate) struct TimelineFrameOutput {
    pub(crate) response: egui::Response,
    pub(crate) intents: Vec<TimelineIntent>,
}

pub(crate) fn project_for_egui(
    document: &Document,
) -> Result<TimelineProjection, TimelineProjectionError> {
    let duration = document.composition.duration;
    let duration_seconds = duration.as_seconds_f64();
    project_timeline(
        document,
        &TimelineMetrics {
            band_height: 1.0,
            units_per_second: duration_seconds.recip(),
            key_half_extent: 1.0,
        },
        &TimelineViewport {
            start: RationalTime::ZERO,
            end: duration,
        },
    )
}

pub(crate) fn paint_timeline(
    ui: &mut Ui,
    document: &Document,
    projection: Option<&TimelineProjection>,
    projection_error: Option<&str>,
    primary: &mut Option<LayerId>,
    playhead: &mut RationalTime,
) -> TimelineFrameOutput {
    let available = ui.available_size();
    let size = Vec2::new(available.x.max(1.0), available.y.max(1.0));
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    let _theme = TimelineTheme;

    let rows = rows::rows_from_projection(document, projection);
    let geometry = TimelineGeometry::new(
        rect,
        rows.len(),
        document.composition.duration.as_seconds_f64(),
    );

    painter.rect_filled(rect, CornerRadius::ZERO, DESKTOP);
    painter.rect_filled(
        Rect::from_min_max(rect.min, Pos2::new(geometry.surface_left(), rect.bottom())),
        CornerRadius::ZERO,
        SURFACE,
    );

    ruler::paint_overview(&painter, &geometry, &rows);
    ruler::paint_ruler(&painter, &geometry);
    ruler::paint_locators(&painter, &geometry);
    clip_band::paint_rows(&painter, &geometry, &rows, *primary);
    ruler::paint_playhead(&painter, &geometry, *playhead, rows.len());

    if let Some(error) = projection_error {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("Timeline unavailable: {error}"),
            FontId::proportional(9.0),
            DIM,
        );
    }

    apply_direct_manipulation(&response, &geometry, &rows, primary, playhead);
    let intents = input::collect_intents(ui, &response, &geometry, &rows, projection);

    TimelineFrameOutput { response, intents }
}

/// 時間ヘッダのscrubと行選択。Documentを経由せず席の見え方だけを進める。
fn apply_direct_manipulation(
    response: &egui::Response,
    geometry: &TimelineGeometry,
    rows: &[TimelineRow],
    primary: &mut Option<LayerId>,
    playhead: &mut RationalTime,
) {
    let Some(pointer) = response.interact_pointer_pos() else {
        return;
    };
    let in_surface = geometry.in_surface(pointer);
    let in_time_header = pointer.y < geometry.rows_top;
    if in_surface && (response.clicked() || response.dragged()) && in_time_header {
        if let Some(time) = geometry.time_at(pointer) {
            *playhead = time;
        }
    } else if response.clicked() && pointer.y >= geometry.rows_top {
        if let Some(row) = geometry
            .row_index_at(pointer.y)
            .and_then(|index| rows.get(index))
            .filter(|row| row.property.is_none())
        {
            *primary = Some(row.layer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paints_the_document_timeline_skin_into_an_egui_surface() {
        let context = egui::Context::default();
        let document = Document::new_current();
        let projection = project_for_egui(&document).expect("current document timeline");
        let mut response_rect = None;
        let output = context.run_ui(Default::default(), |ui| {
            let mut primary = None;
            let mut playhead = RationalTime::ZERO;
            response_rect = Some(
                paint_timeline(
                    ui,
                    &document,
                    Some(&projection),
                    None,
                    &mut primary,
                    &mut playhead,
                )
                .response
                .rect,
            );
        });
        let rect = response_rect.expect("timeline response");
        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
        assert!(!output.shapes.is_empty());
    }
}
