use crate::theme;
use eframe::egui::{
    self, pos2, Align, Color32, FontId, Layout, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ComponentTokens {
    pub panel_header_height: f32,
    pub tab_height: f32,
    pub section_header_height: f32,
    pub property_row_height: f32,
    pub control_height: f32,
    pub inset: f32,
    pub gap: f32,
    pub corner_radius: f32,
}

pub(crate) const TOKENS: ComponentTokens = ComponentTokens {
    panel_header_height: 29.0,
    tab_height: 29.0,
    section_header_height: 23.0,
    property_row_height: 27.0,
    control_height: 25.0,
    inset: 9.0,
    gap: 5.0,
    corner_radius: 2.0,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Block {
    pub height: f32,
    pub fill: Color32,
    pub border_top: bool,
    pub border_bottom: bool,
    pub inset_x: f32,
}

impl Block {
    pub(crate) fn show<R>(
        self,
        ui: &mut egui::Ui,
        layout: Layout,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let rect = Rect::from_min_size(
            ui.cursor().min,
            Vec2::new(ui.available_width(), self.height),
        );
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, self.fill);
        if self.border_top {
            painter.line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(1.0, theme::BORDER),
            );
        }
        if self.border_bottom {
            painter.line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.0, theme::BORDER),
            );
        }
        ui.allocate_ui_with_layout(rect.size(), layout, |ui| {
            ui.add_space(self.inset_x);
            add_contents(ui)
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ToolIcon {
    Select,
    Hand,
    Shape,
    Text,
    Connect,
    Relative,
    Camera,
}

pub(crate) fn tool_button(ui: &mut egui::Ui, icon: ToolIcon, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(29.0, 24.0), Sense::click());
    let painter = ui.painter();
    if selected || response.hovered() {
        painter.rect_filled(
            rect,
            0.0,
            if selected {
                theme::RAISED
            } else {
                theme::HOVER
            },
        );
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, if selected { theme::TEXT } else { theme::BORDER }),
            StrokeKind::Inside,
        );
    }
    let c = rect.center();
    let stroke = Stroke::new(1.15, theme::TEXT_SECONDARY);
    match icon {
        ToolIcon::Select => {
            painter.line_segment([c + Vec2::new(-5.0, -5.0), c + Vec2::new(4.0, 4.0)], stroke);
            painter.line_segment(
                [c + Vec2::new(-5.0, -5.0), c + Vec2::new(-4.0, 1.0)],
                stroke,
            );
            painter.line_segment(
                [c + Vec2::new(-5.0, -5.0), c + Vec2::new(1.0, -4.0)],
                stroke,
            );
        }
        ToolIcon::Hand => {
            painter.line_segment([c + Vec2::new(-5.0, 0.0), c + Vec2::new(5.0, 0.0)], stroke);
            painter.line_segment([c + Vec2::new(0.0, -5.0), c + Vec2::new(0.0, 5.0)], stroke);
            painter.line_segment([c + Vec2::new(-3.5, -3.5), c + Vec2::new(3.5, 3.5)], stroke);
            painter.line_segment([c + Vec2::new(3.5, -3.5), c + Vec2::new(-3.5, 3.5)], stroke);
        }
        ToolIcon::Shape => {
            painter.add(egui::Shape::closed_line(
                vec![
                    c + Vec2::new(0.0, -5.0),
                    c + Vec2::new(5.0, 0.0),
                    c + Vec2::new(0.0, 5.0),
                    c + Vec2::new(-5.0, 0.0),
                ],
                stroke,
            ));
        }
        ToolIcon::Text => {
            painter.text(
                c,
                egui::Align2::CENTER_CENTER,
                "T",
                FontId::proportional(11.0),
                theme::TEXT_SECONDARY,
            );
        }
        ToolIcon::Connect => {
            painter.line_segment(
                [c + Vec2::new(-5.0, 3.0), c + Vec2::new(-1.0, -3.0)],
                stroke,
            );
            painter.line_segment([c + Vec2::new(-1.0, -3.0), c + Vec2::new(4.0, 2.0)], stroke);
            painter.circle_filled(c + Vec2::new(-5.0, 3.0), 1.5, theme::TEXT_SECONDARY);
            painter.circle_filled(c + Vec2::new(4.0, 2.0), 1.5, theme::TEXT_SECONDARY);
        }
        ToolIcon::Relative => {
            painter.add(egui::Shape::closed_line(
                vec![
                    c + Vec2::new(0.0, -5.0),
                    c + Vec2::new(5.0, 5.0),
                    c + Vec2::new(-5.0, 5.0),
                ],
                stroke,
            ));
        }
        ToolIcon::Camera => {
            painter.rect_stroke(
                Rect::from_center_size(c, Vec2::new(10.0, 8.0)),
                0.0,
                stroke,
                StrokeKind::Inside,
            );
            painter.rect_filled(
                Rect::from_center_size(c, Vec2::new(4.0, 3.0)),
                0.0,
                theme::TEXT_SECONDARY,
            );
        }
    };
    response
}

pub(crate) fn quiet_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add_sized(
        [
            TOKENS.control_height.max(label.len() as f32 * 7.0 + 14.0),
            TOKENS.control_height,
        ],
        egui::Button::new(RichText::new(label).size(10.0).color(theme::TEXT_SECONDARY))
            .fill(theme::PANEL)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(TOKENS.corner_radius),
    )
}

pub(crate) fn panel_header(ui: &mut egui::Ui, title: &str, detail: &str, marker_color: Color32) {
    Block {
        height: TOKENS.panel_header_height,
        fill: theme::RAISED,
        border_top: false,
        border_bottom: true,
        inset_x: TOKENS.inset,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = TOKENS.gap;
        let (marker, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
        ui.painter().rect_filled(marker, 0.0, marker_color);
        ui.label(RichText::new(title).strong().color(theme::TEXT));
        if !detail.is_empty() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(TOKENS.inset);
                ui.label(
                    RichText::new(detail)
                        .monospace()
                        .size(7.0)
                        .color(theme::TEXT_MUTED),
                );
            });
        }
    });
}

pub(crate) fn tabs(
    ui: &mut egui::Ui,
    labels: &[&str],
    selected: usize,
    accent: Color32,
) -> Option<usize> {
    let width = ui.available_width();
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        for (index, label) in labels.iter().enumerate() {
            let response = ui.add_sized(
                [width / labels.len() as f32, TOKENS.tab_height],
                egui::Button::new(RichText::new(*label).size(10.0))
                    .fill(theme::PANEL)
                    .stroke(Stroke::NONE)
                    .corner_radius(0.0),
            );
            if index == selected {
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        pos2(response.rect.left(), response.rect.bottom() - 2.0),
                        Vec2::new(response.rect.width(), 2.0),
                    ),
                    0.0,
                    accent,
                );
            }
            if response.clicked() {
                clicked = Some(index);
            }
        }
    });
    clicked
}

pub(crate) fn section_header(
    ui: &mut egui::Ui,
    left: &str,
    right: &str,
    right_color: Option<Color32>,
) {
    Block {
        height: TOKENS.section_header_height,
        fill: theme::APP,
        border_top: false,
        border_bottom: false,
        inset_x: TOKENS.inset,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        ui.label(
            RichText::new(left)
                .monospace()
                .size(8.0)
                .color(theme::TEXT_MUTED),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(TOKENS.inset);
            ui.label(
                RichText::new(right)
                    .monospace()
                    .size(8.0)
                    .color(right_color.unwrap_or(theme::TEXT_SECONDARY)),
            );
        });
    });
}

pub(crate) fn property_row(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    horizontal_rule(ui);
    Block {
        height: TOKENS.property_row_height,
        fill: Color32::TRANSPARENT,
        border_top: false,
        border_bottom: false,
        inset_x: TOKENS.inset,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        contents(ui);
        ui.add_space(3.0);
    });
}

pub(crate) fn property_label(ui: &mut egui::Ui, label: &str) {
    ui.add_sized(
        [92.0, 21.0],
        egui::Label::new(RichText::new(label).size(10.0).color(theme::TEXT_SECONDARY)),
    );
}

pub(crate) fn value_box(ui: &mut egui::Ui, value: &str) {
    let width = (ui.available_width() - 61.0).max(70.0);
    egui::Frame::NONE
        .fill(theme::APP)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(TOKENS.corner_radius)
        .inner_margin(egui::Margin::symmetric(5, 2))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.label(
                RichText::new(value)
                    .monospace()
                    .size(9.0)
                    .color(theme::TEXT),
            );
        });
}

pub(crate) fn tag(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::NONE
        .stroke(Stroke::new(1.0, color))
        .corner_radius(TOKENS.corner_radius)
        .inner_margin(egui::Margin::symmetric(5, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().size(7.0).color(color));
        });
}

pub(crate) fn automation_mark(ui: &mut egui::Ui, active: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
    let color = if active {
        theme::ACCENT
    } else {
        theme::TEXT_MUTED
    };
    ui.painter().rect_filled(rect, 0.0, theme::APP);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, if active { theme::ACCENT } else { theme::BORDER }),
        StrokeKind::Inside,
    );
    let center = rect.center();
    let points = vec![
        center + Vec2::new(0.0, -4.0),
        center + Vec2::new(4.0, 0.0),
        center + Vec2::new(0.0, 4.0),
        center + Vec2::new(-4.0, 0.0),
    ];
    if active {
        ui.painter().rect_stroke(
            rect.shrink(2.0),
            0.0,
            Stroke::new(1.0, theme::ACCENT.gamma_multiply(0.25)),
            StrokeKind::Inside,
        );
    }
    ui.painter()
        .add(egui::Shape::closed_line(points, Stroke::new(1.0, color)));
    response
}

pub(crate) fn scrub_control(ui: &mut egui::Ui, value: &mut f32, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::click_and_drag());
    if response.dragged() {
        let delta = ui.input(|input| input.pointer.delta().x);
        *value = scrubbed_value(*value, delta);
    }

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, theme::APP);
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(
            1.0,
            if response.dragged() {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    let dial = Rect::from_min_max(
        rect.left_top() + Vec2::new(4.0, 3.0),
        rect.right_bottom() - Vec2::new(43.0, 3.0),
    );
    let shift = (*value * 200.0) % 50.0;
    let mut x = dial.left() - 50.0 + shift;
    while x <= dial.right() + 50.0 {
        painter.line_segment(
            [pos2(x, dial.top()), pos2(x, dial.bottom())],
            Stroke::new(1.0, theme::ACCENT),
        );
        for minor in 1..5 {
            let minor_x = x + minor as f32 * 10.0;
            painter.line_segment(
                [pos2(minor_x, dial.center().y), pos2(minor_x, dial.bottom())],
                Stroke::new(1.0, theme::BORDER_STRONG),
            );
        }
        x += 50.0;
    }
    painter.line_segment(
        [
            pos2(dial.center().x, dial.top()),
            pos2(dial.center().x, dial.bottom()),
        ],
        Stroke::new(1.0, theme::TEXT),
    );
    let value_rect = Rect::from_min_max(
        pos2(rect.right() - 43.0, rect.top() + 1.0),
        rect.right_bottom() - Vec2::splat(1.0),
    );
    painter.rect_filled(value_rect, 0.0, theme::APP);
    painter.text(
        value_rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.0}%", *value * 100.0),
        FontId::monospace(8.0),
        theme::ACCENT,
    );
    response.on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
}

pub(crate) fn horizontal_rule(ui: &egui::Ui) {
    let y = ui.cursor().top();
    ui.painter().line_segment(
        [
            pos2(ui.max_rect().left(), y),
            pos2(ui.max_rect().right(), y),
        ],
        Stroke::new(1.0, theme::BORDER),
    );
}

pub(crate) fn scrubbed_value(value: f32, pointer_delta: f32) -> f32 {
    (value + pointer_delta / 100.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_dimensions_match_the_react_tokens() {
        assert_eq!(TOKENS.panel_header_height, 29.0);
        assert_eq!(TOKENS.section_header_height, 23.0);
        assert_eq!(TOKENS.property_row_height, 27.0);
        assert_eq!(TOKENS.control_height, 25.0);
        assert_eq!(TOKENS.corner_radius, 2.0);
        assert_eq!(TOKENS.gap, 5.0);
    }

    #[test]
    fn scrub_is_horizontal_and_clamped() {
        assert_eq!(scrubbed_value(0.64, 10.0), 0.74);
        assert_eq!(scrubbed_value(0.95, 20.0), 1.0);
        assert_eq!(scrubbed_value(0.05, -20.0), 0.0);
    }
}
