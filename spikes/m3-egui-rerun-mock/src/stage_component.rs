use crate::theme;
use eframe::egui::{
    self, pos2, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, Vec2,
};

const HEADER_HEIGHT: f32 = 30.0;
const TRANSPORT_HEIGHT: f32 = 32.0;

#[derive(Debug, Clone, Default)]
pub(crate) struct StageState {
    pub(crate) playing: bool,
    pub(crate) fit_requested: bool,
    pub(crate) previous_key_requested: bool,
    pub(crate) next_key_requested: bool,
}

pub(crate) fn stage_ui(ui: &mut egui::Ui, state: &mut StageState) {
    ui.spacing_mut().item_spacing.y = 0.0;
    state.fit_requested = false;
    state.previous_key_requested = false;
    state.next_key_requested = false;

    ui.painter().rect_filled(ui.max_rect(), 0.0, theme::PANEL);

    let content_width = ui.available_width();
    let header_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(content_width, HEADER_HEIGHT));
    ui.painter().rect_filled(header_rect, 0.0, theme::PANEL);
    ui.allocate_ui_with_layout(
        Vec2::new(content_width, HEADER_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            ui.add_space(8.0);
            let (marker, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
            ui.painter().rect_filled(marker, 0.0, theme::WAY_STAGE);
            if stage_tool_button(ui, "Fit", false).clicked() {
                state.fit_requested = true;
            }
            let _ = stage_tool_button(ui, "100%", false);
            let _ = stage_grid_button(ui);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(9.0);
                ui.label(
                    RichText::new("ECHO BLOOM")
                        .monospace()
                        .size(8.0)
                        .strong()
                        .color(theme::ACCENT),
                );
                ui.label(
                    RichText::new("STAGE /")
                        .monospace()
                        .size(8.0)
                        .color(theme::TEXT_MUTED),
                );
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
    ui.painter().rect_filled(transport_rect, 0.0, theme::RAISED);
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
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.add_space(8.0);
            if stage_tool_button(ui, "|‹", false)
                .on_hover_text("Previous key")
                .clicked()
            {
                state.previous_key_requested = true;
            }
            let play_glyph = if state.playing { "■" } else { "▶" };
            if stage_tool_button(ui, play_glyph, false)
                .on_hover_text(if state.playing { "Pause" } else { "Play" })
                .clicked()
            {
                state.playing = !state.playing;
            }
            if stage_tool_button(ui, "›|", false)
                .on_hover_text("Next key")
                .clicked()
            {
                state.next_key_requested = true;
            }
            let _ = interval_easing_button(ui).on_hover_text("Open Interval Easing Editor");
            ui.label(
                RichText::new("00:54.2")
                    .strong()
                    .monospace()
                    .color(theme::TEXT),
            );
            ui.label(
                RichText::new("BAR 54.2.00")
                    .monospace()
                    .size(9.0)
                    .color(theme::TEXT_SECONDARY),
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

fn paint_canvas(painter: &egui::Painter, canvas: Rect) {
    painter.rect_filled(canvas, 0.0, Color32::from_rgb(6, 6, 6));

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
    let painter = painter.with_clip_rect(frame);
    let mut mesh = egui::Mesh::default();
    let base = mesh.vertices.len() as u32;
    for (pos, color) in [
        (frame.left_top(), Color32::from_rgb(23, 19, 21)),
        (frame.right_top(), Color32::from_rgb(11, 10, 10)),
        (frame.right_bottom(), Color32::from_rgb(9, 9, 9)),
        (frame.left_bottom(), Color32::from_rgb(13, 12, 12)),
    ] {
        mesh.colored_vertex(pos, color);
    }
    mesh.add_triangle(base, base + 1, base + 2);
    mesh.add_triangle(base, base + 2, base + 3);
    painter.add(egui::Shape::mesh(mesh));

    let radial_center = Pos2::new(
        frame.left() + frame.width() * 0.63,
        frame.top() + frame.height() * 0.37,
    );
    for step in (1..=18).rev() {
        let amount = step as f32 / 18.0;
        let (from, to, weight) = if amount <= 0.49 {
            (
                [38.0_f32, 49.0, 58.0],
                [20.0_f32, 25.0, 28.0],
                amount / 0.49,
            )
        } else {
            (
                [20.0_f32, 25.0, 28.0],
                [13.0_f32, 12.0, 12.0],
                (amount - 0.49) / 0.51,
            )
        };
        let color = Color32::from_rgb(
            egui::lerp(from[0]..=to[0], weight) as u8,
            egui::lerp(from[1]..=to[1], weight) as u8,
            egui::lerp(from[2]..=to[2], weight) as u8,
        );
        painter.circle_filled(radial_center, frame.width() * 0.51 * amount, color);
    }

    let veil_center = Pos2::new(
        frame.left() + frame.width() * 0.70,
        frame.top() + frame.height() * 0.46,
    );
    for step in (1..=12).rev() {
        let amount = step as f32 / 12.0;
        let alpha = ((1.0 - amount) * 4.0) as u8;
        painter.circle_filled(
            veil_center,
            frame.width() * 0.33 * amount,
            Color32::from_rgba_unmultiplied(216, 181, 116, alpha),
        );
    }
}

fn paint_output_grid(painter: &egui::Painter, frame: Rect) {
    let grid_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(51, 51, 51, 43));
    for index in 1..8 {
        let x = egui::lerp(frame.x_range(), index as f32 / 8.0);
        painter.line_segment(
            [Pos2::new(x, frame.top()), Pos2::new(x, frame.bottom())],
            grid_stroke,
        );
    }
    for index in 1..4 {
        let y = egui::lerp(frame.y_range(), index as f32 / 4.0);
        painter.line_segment(
            [Pos2::new(frame.left(), y), Pos2::new(frame.right(), y)],
            grid_stroke,
        );
    }
}

fn paint_selection(painter: &egui::Painter, frame: Rect) {
    let selection = Rect::from_min_size(
        frame.left_top() + Vec2::new(frame.width() * 0.07, frame.height() * 0.13),
        Vec2::new(frame.width() * 0.39, frame.height() * 0.31),
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
        let handle = Rect::from_center_size(corner, Vec2::splat(7.0));
        painter.rect_filled(handle, 0.0, theme::APP);
        painter.rect_stroke(
            handle,
            0.0,
            Stroke::new(1.0, theme::TEXT),
            StrokeKind::Inside,
        );
    }
    let title_size = 36.0;
    paint_spaced_text(
        painter,
        selection.left_top() + Vec2::new(7.0, -2.0),
        "NIGHT",
        title_size,
        3.0,
        theme::TEXT_SECONDARY,
    );
    paint_spaced_text(
        painter,
        selection.left_top() + Vec2::new(7.0, 36.0),
        "DRIVE",
        title_size,
        3.0,
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

fn paint_spaced_text(
    painter: &egui::Painter,
    origin: Pos2,
    text: &str,
    size: f32,
    spacing: f32,
    color: Color32,
) {
    let font = theme::display_font(size);
    let mut x = origin.x;
    for character in text.chars() {
        let galley = painter.layout_no_wrap(character.to_string(), font.clone(), color);
        let advance = galley.size().x;
        painter.galley(pos2(x, origin.y), galley, color);
        x += advance + spacing;
    }
}

fn paint_ring_preview(painter: &egui::Painter, frame: Rect) {
    let center = frame.center() + Vec2::new(frame.width() * 0.17, 0.0);
    for (index, radius) in [0.373, 0.291, 0.209, 0.127].into_iter().enumerate() {
        let color = Color32::from_rgba_unmultiplied(170, 170, 170, 170 - index as u8 * 22);
        if index % 2 == 0 {
            paint_dashed_circle(painter, center, frame.height() * radius, color);
        } else {
            painter.circle_stroke(center, frame.height() * radius, Stroke::new(1.0, color));
        }
    }
    let pulse_radius = frame.height() * 0.082;
    for step in (1..=10).rev() {
        let amount = step as f32 / 10.0;
        painter.circle_filled(
            center,
            pulse_radius + 24.0 * amount,
            Color32::from_rgba_unmultiplied(183, 211, 213, ((1.0 - amount) * 10.0) as u8),
        );
    }
    painter.circle_filled(center, pulse_radius, Color32::from_rgb(221, 221, 221));
}

fn paint_dashed_circle(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let segments = 96;
    for segment in (0..segments).step_by(2) {
        let start = std::f32::consts::TAU * segment as f32 / segments as f32;
        let end = std::f32::consts::TAU * (segment + 1) as f32 / segments as f32;
        painter.line_segment(
            [
                center + Vec2::angled(start) * radius,
                center + Vec2::angled(end) * radius,
            ],
            Stroke::new(1.0, color),
        );
    }
}

fn stage_tool_button(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    ui.add_sized(
        [if label.len() > 3 { 40.0 } else { 29.0 }, 23.0],
        egui::Button::selectable(selected, RichText::new(label).size(10.0)),
    )
}

fn stage_grid_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(29.0, 23.0), Sense::click());
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0, theme::TEXT), StrokeKind::Inside);
    let grid = Rect::from_center_size(rect.center(), Vec2::splat(10.0));
    ui.painter().rect_stroke(
        grid,
        0.0,
        Stroke::new(1.0, theme::TEXT_SECONDARY),
        StrokeKind::Inside,
    );
    ui.painter().line_segment(
        [
            pos2(grid.center().x, grid.top()),
            pos2(grid.center().x, grid.bottom()),
        ],
        Stroke::new(1.0, theme::TEXT_SECONDARY),
    );
    ui.painter().line_segment(
        [
            pos2(grid.left(), grid.center().y),
            pos2(grid.right(), grid.center().y),
        ],
        Stroke::new(1.0, theme::TEXT_SECONDARY),
    );
    response
}

fn interval_easing_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(28.0, 22.0), Sense::click());
    ui.painter().rect_filled(rect, 0.0, theme::APP);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, theme::ACCENT),
        StrokeKind::Inside,
    );
    let left = rect.left_bottom() + Vec2::new(5.0, -4.0);
    let right = rect.right_top() + Vec2::new(-5.0, 4.0);
    let points = (0..=12)
        .map(|index| {
            let t = index as f32 / 12.0;
            let eased = t * t * (3.0 - 2.0 * t);
            Pos2::new(
                egui::lerp(left.x..=right.x, t),
                egui::lerp(left.y..=right.y, eased),
            )
        })
        .collect();
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(1.2, theme::ACCENT)));
    ui.painter().circle_filled(left, 1.5, theme::ACCENT);
    ui.painter().circle_filled(right, 1.5, theme::ACCENT);
    response
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
