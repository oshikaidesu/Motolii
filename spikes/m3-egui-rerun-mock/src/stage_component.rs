use crate::theme;
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    Vec2,
};

const HEADER_HEIGHT: f32 = 29.0;
const TRANSPORT_HEIGHT: f32 = 30.0;

#[derive(Debug, Clone)]
pub(crate) struct StageState {
    pub(crate) playing: bool,
    pub(crate) fit_requested: bool,
    pub(crate) previous_key_requested: bool,
    pub(crate) next_key_requested: bool,
}

impl Default for StageState {
    fn default() -> Self {
        Self {
            playing: false,
            fit_requested: false,
            previous_key_requested: false,
            next_key_requested: false,
        }
    }
}

pub(crate) fn stage_ui(ui: &mut egui::Ui, state: &mut StageState) {
    ui.spacing_mut().item_spacing.y = 0.0;
    state.fit_requested = false;
    state.previous_key_requested = false;
    state.next_key_requested = false;

    ui.painter().rect_filled(ui.max_rect(), 0.0, theme::PANEL);

    let content_width = ui.available_width();
    let header_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(content_width, HEADER_HEIGHT));
    ui.painter().rect_filled(header_rect, 0.0, theme::APP);
    ui.allocate_ui_with_layout(
        Vec2::new(content_width, HEADER_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            ui.add_space(8.0);
            if quiet_button(ui, "Fit").clicked() {
                state.fit_requested = true;
            }
            let _ = quiet_button(ui, "100%");

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(9.0);
                ui.label(RichText::new("Stage").strong().color(theme::TEXT));
                let (marker, _) = ui.allocate_exact_size(Vec2::new(6.0, 14.0), Sense::hover());
                ui.painter().rect_filled(marker, 0.0, theme::WAY_STAGE);
            });
        },
    );

    let separator_y = ui.min_rect().top() + HEADER_HEIGHT;
    ui.painter().line_segment(
        [
            Pos2::new(ui.max_rect().left(), separator_y),
            Pos2::new(ui.max_rect().right(), separator_y),
        ],
        Stroke::new(1.0, theme::BORDER),
    );

    let canvas_size = Vec2::new(
        ui.available_width(),
        (ui.available_height() - TRANSPORT_HEIGHT).max(80.0),
    );
    let (canvas, _) = ui.allocate_exact_size(canvas_size, Sense::click());
    paint_canvas(ui.painter(), canvas);

    let transport_top = ui.cursor().top();
    let transport_rect = Rect::from_min_size(
        Pos2::new(ui.max_rect().left(), transport_top),
        Vec2::new(ui.available_width(), TRANSPORT_HEIGHT),
    );
    ui.painter().rect_filled(transport_rect, 0.0, theme::APP);
    ui.painter().line_segment(
        [
            Pos2::new(ui.max_rect().left(), transport_top),
            Pos2::new(ui.max_rect().right(), transport_top),
        ],
        Stroke::new(1.0, theme::BORDER),
    );
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), TRANSPORT_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            ui.add_space(8.0);
            if quiet_button(ui, "|‹")
                .on_hover_text("Previous key")
                .clicked()
            {
                state.previous_key_requested = true;
            }
            let play_glyph = if state.playing { "■" } else { "▶" };
            if quiet_button(ui, play_glyph)
                .on_hover_text(if state.playing { "Pause" } else { "Play" })
                .clicked()
            {
                state.playing = !state.playing;
            }
            if quiet_button(ui, "›|").on_hover_text("Next key").clicked() {
                state.next_key_requested = true;
            }
            ui.label(
                RichText::new("00:54.2")
                    .strong()
                    .monospace()
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new("120 BPM · SNAP BEAT")
                    .size(10.0)
                    .color(theme::TEXT_SECONDARY),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("DRAFT · FP16 · 1/2")
                        .monospace()
                        .size(9.0)
                        .color(theme::TEXT_MUTED),
                );
            });
        },
    );
}

fn quiet_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add_sized(
        [25.0_f32.max(label.len() as f32 * 7.0 + 14.0), 25.0],
        egui::Button::new(RichText::new(label).size(10.0).color(theme::TEXT_SECONDARY))
            .fill(theme::PANEL)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(2.0),
    )
}

fn paint_canvas(painter: &egui::Painter, canvas: Rect) {
    painter.rect_filled(canvas, 0.0, Color32::from_rgb(9, 9, 9));

    let frame = output_frame(canvas);
    paint_output_background(painter, frame);
    paint_output_grid(painter, frame);
    paint_selection(painter, frame);
    paint_ring_preview(painter, frame);

    painter.text(
        frame.left_top() + Vec2::new(8.0, 8.0),
        Align2::LEFT_TOP,
        "OUTPUT FRAME",
        FontId::monospace(8.0),
        theme::TEXT_MUTED,
    );
    painter.text(
        frame.center_bottom() - Vec2::new(0.0, 12.0),
        Align2::CENTER_BOTTOM,
        "TEXTURE → ECHO BLOOM → TEXTURE",
        FontId::monospace(8.0),
        theme::TEXT_SECONDARY,
    );
    painter.rect_stroke(
        frame,
        0.0,
        Stroke::new(1.0, theme::BORDER_STRONG),
        StrokeKind::Inside,
    );
}

fn output_frame(canvas: Rect) -> Rect {
    let max_width = (canvas.width() * 0.82).min(720.0);
    let max_height = canvas.height() * 0.80;
    let width = max_width.min(max_height * 16.0 / 9.0).max(160.0);
    Rect::from_center_size(canvas.center(), Vec2::new(width, width * 9.0 / 16.0))
}

fn paint_output_background(painter: &egui::Painter, frame: Rect) {
    painter.rect_filled(frame, 0.0, Color32::from_rgb(7, 8, 11));
    let center = frame.center();
    let max_radius = frame.width() * 0.52;
    for step in (1..=18).rev() {
        let amount = step as f32 / 18.0;
        let glow = 1.0 - amount;
        let color = Color32::from_rgb(
            (7.0 + 35.0 * glow) as u8,
            (8.0 + 21.0 * glow) as u8,
            (11.0 + 41.0 * glow) as u8,
        );
        painter.circle_filled(center, max_radius * amount, color);
    }
}

fn paint_output_grid(painter: &egui::Painter, frame: Rect) {
    let grid_stroke = Stroke::new(0.5, theme::BORDER.gamma_multiply(0.32));
    for index in 1..4 {
        let x = egui::lerp(frame.x_range(), index as f32 / 4.0);
        painter.line_segment(
            [Pos2::new(x, frame.top()), Pos2::new(x, frame.bottom())],
            grid_stroke,
        );
    }
    for index in 1..3 {
        let y = egui::lerp(frame.y_range(), index as f32 / 3.0);
        painter.line_segment(
            [Pos2::new(frame.left(), y), Pos2::new(frame.right(), y)],
            grid_stroke,
        );
    }
}

fn paint_selection(painter: &egui::Painter, frame: Rect) {
    let selection = Rect::from_min_size(
        frame.left_top() + Vec2::new(frame.width() * 0.07, frame.height() * 0.14),
        Vec2::new(frame.width() * 0.39, frame.height() * 0.30),
    );
    painter.rect_stroke(
        selection,
        0.0,
        Stroke::new(1.0, theme::ACCENT),
        StrokeKind::Inside,
    );
    for corner in [
        selection.left_top(),
        selection.right_top(),
        selection.left_bottom(),
        selection.right_bottom(),
    ] {
        painter.rect_filled(
            Rect::from_center_size(corner, Vec2::splat(6.0)),
            0.0,
            theme::TEXT_SECONDARY,
        );
    }
    let title_size = (frame.height() * 0.112).clamp(20.0, 43.0);
    painter.text(
        selection.left_top() + Vec2::new(7.0, 5.0),
        Align2::LEFT_TOP,
        "NIGHT",
        FontId::proportional(title_size),
        theme::TEXT_SECONDARY,
    );
    painter.text(
        selection.left_top() + Vec2::new(7.0, 43.0),
        Align2::LEFT_TOP,
        "DRIVE",
        FontId::proportional(title_size),
        theme::TEXT_SECONDARY,
    );
    painter.text(
        selection.left_bottom() + Vec2::new(7.0, -8.0),
        Align2::LEFT_BOTTOM,
        "54.2 / CITY SIGNAL",
        FontId::monospace(7.0),
        theme::TEXT_MUTED,
    );
}

fn paint_ring_preview(painter: &egui::Painter, frame: Rect) {
    let center = frame.center() + Vec2::new(frame.width() * 0.17, 0.0);
    for (radius, color) in [
        (frame.height() * 0.37, theme::SHAPE),
        (frame.height() * 0.29, theme::TEXT_SECONDARY),
        (frame.height() * 0.20, theme::SHAPE),
    ] {
        painter.circle_stroke(center, radius, Stroke::new(2.0, color.gamma_multiply(0.70)));
    }
    for step in (1..=8).rev() {
        let amount = step as f32 / 8.0;
        painter.circle_filled(
            center,
            frame.height() * 0.14 * amount,
            theme::TEXT.gamma_multiply((1.0 - amount) * 0.55 + 0.08),
        );
    }
    painter.circle_filled(center, frame.height() * 0.055, theme::TEXT_SECONDARY);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_frame_keeps_the_react_fixture_aspect_ratio() {
        let frame = output_frame(Rect::from_min_size(Pos2::ZERO, Vec2::new(830.0, 479.0)));
        assert!((frame.width() / frame.height() - 16.0 / 9.0).abs() < 0.001);
        assert!(frame.width() <= 720.0);
    }
}
