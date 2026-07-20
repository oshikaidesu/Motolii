use crate::{fixture::BARS, theme};
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Id, Rect, Response, Sense, Stroke, StrokeKind,
};

const HEADER_HEIGHT: f32 = 28.0;
const KEY_TOOLS_WIDTH: f32 = 200.0;
const BAND_RAIL_WIDTH: f32 = 54.0;
const RULER_HEIGHT: f32 = 23.0;
const BAND_COUNT: usize = 5;
const DEFAULT_BAND_HEIGHT: f32 = 34.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyToolsMode {
    Keys,
    Layers,
}

#[derive(Debug)]
pub(crate) struct TimelineState {
    pub(crate) selected_bar: &'static str,
    pub(crate) status: String,
    key_tools_open: bool,
    key_tools_mode: KeyToolsMode,
    key_scope: usize,
    key_section: usize,
    layer_section: usize,
    solo: [bool; BAND_COUNT],
    muted: [bool; BAND_COUNT],
    playhead: f32,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            selected_bar: "pulse",
            status: "Pulse rings · Timeline selection · mock-local".into(),
            key_tools_open: true,
            key_tools_mode: KeyToolsMode::Keys,
            key_scope: 0,
            key_section: 0,
            layer_section: 0,
            solo: [false; BAND_COUNT],
            muted: [false; BAND_COUNT],
            playhead: 0.887,
        }
    }
}

pub(crate) fn timeline_ui(ui: &mut egui::Ui, state: &mut TimelineState) {
    let desired = ui.available_size();
    let (whole, _) = ui.allocate_exact_size(desired, Sense::hover());
    let painter = ui.painter_at(whole);
    painter.rect_filled(whole, 0.0, theme::PANEL);

    let header = Rect::from_min_max(whole.min, pos2(whole.right(), whole.top() + HEADER_HEIGHT));
    paint_header(ui, header, state);

    let body = Rect::from_min_max(pos2(whole.left(), header.bottom()), whole.max);
    if body.height() <= RULER_HEIGHT {
        return;
    }

    let tools_width = if state.key_tools_open {
        KEY_TOOLS_WIDTH.min((body.width() - 180.0).max(0.0))
    } else {
        0.0
    };
    if state.key_tools_open {
        let tools = Rect::from_min_max(body.min, pos2(body.left() + tools_width, body.bottom()));
        paint_key_tools(ui, tools, state);
    } else {
        let open = Rect::from_min_size(body.min + vec2(1.0, 1.0), vec2(22.0, 21.0));
        if flat_button(ui, open, "K", false, "timeline-key-tools-open").clicked() {
            state.key_tools_open = true;
        }
    }

    let rail = Rect::from_min_max(
        pos2(body.left() + tools_width, body.top()),
        pos2(
            (body.left() + tools_width + BAND_RAIL_WIDTH).min(body.right()),
            body.bottom(),
        ),
    );
    paint_action_rail(ui, rail, state);

    let plane = Rect::from_min_max(pos2(rail.right(), body.top()), body.max);
    if plane.width() > 8.0 {
        paint_time_plane(ui, plane, state);
    }

    painter.rect_filled(
        Rect::from_min_size(
            pos2(whole.left(), body.top() + 21.0),
            vec2(whole.width(), 1.0),
        ),
        0.0,
        theme::TEXT,
    );
    painter.rect_filled(
        Rect::from_min_size(
            pos2(plane.left(), body.top() + 22.0),
            vec2(plane.width(), 2.0),
        ),
        0.0,
        theme::PANEL,
    );
}

fn paint_header(ui: &mut egui::Ui, rect: Rect, state: &mut TimelineState) {
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, theme::RAISED);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::BORDER),
    );
    let marker = Rect::from_min_size(rect.min + vec2(9.0, 10.0), vec2(8.0, 8.0));
    painter.rect_filled(marker, 0.0, theme::WAY_TIMELINE);
    let depth = Rect::from_min_size(rect.min + vec2(25.0, 3.0), vec2(24.0, 22.0));
    let _ = flat_button(ui, depth, "", false, "timeline-depth");
    paint_depth_icon(&painter, depth, theme::TEXT_MUTED);
    painter.text(
        pos2(depth.right() + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        "譜面 / Timeline",
        FontId::proportional(11.0),
        theme::TEXT,
    );
    painter.text(
        pos2(rect.right() - 73.0, rect.center().y),
        Align2::RIGHT_CENTER,
        "00:54.2",
        FontId::monospace(8.0),
        theme::TEXT_MUTED,
    );

    let timeline_button = Rect::from_min_size(
        pos2(rect.right() - 66.0, rect.top() + 3.0),
        vec2(27.0, 22.0),
    );
    let graph_button = timeline_button.translate(vec2(29.0, 0.0));
    let _ = flat_button(ui, timeline_button, "", true, "timeline-view");
    paint_timeline_view_icon(&painter, timeline_button, theme::TEXT);
    let graph = flat_button(ui, graph_button, "", false, "graph-view");
    paint_graph_icon(&painter, graph_button, theme::TEXT_MUTED);
    if graph.clicked() {
        state.status = "Graph View · prototype switch".into();
    }
}

fn paint_key_tools(ui: &mut egui::Ui, rect: Rect, state: &mut TimelineState) {
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, theme::APP);
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0, theme::BORDER_STRONG),
    );

    let mode_bar = Rect::from_min_size(rect.min, vec2(rect.width(), RULER_HEIGHT));
    painter.rect_filled(mode_bar, 0.0, theme::PANEL);
    painter.line_segment(
        [mode_bar.left_bottom(), mode_bar.right_bottom()],
        Stroke::new(1.0, theme::BORDER_STRONG),
    );
    let close_width = 22.0;
    let tab_width = (mode_bar.width() - close_width - 6.0) * 0.5;
    let keys = Rect::from_min_size(mode_bar.min + vec2(2.0, 2.0), vec2(tab_width, 18.0));
    let layers = keys.translate(vec2(tab_width + 2.0, 0.0));
    let close = Rect::from_min_size(
        pos2(mode_bar.right() - close_width - 2.0, mode_bar.top() + 2.0),
        vec2(close_width, 18.0),
    );
    if tab_button(
        ui,
        keys,
        "KEYS",
        state.key_tools_mode == KeyToolsMode::Keys,
        "timeline-mode-keys",
    )
    .clicked()
    {
        state.key_tools_mode = KeyToolsMode::Keys;
    }
    if tab_button(
        ui,
        layers,
        "LAYERS",
        state.key_tools_mode == KeyToolsMode::Layers,
        "timeline-mode-layers",
    )
    .clicked()
    {
        state.key_tools_mode = KeyToolsMode::Layers;
    }
    if flat_button(ui, close, "×", false, "timeline-key-tools-close").clicked() {
        state.key_tools_open = false;
        return;
    }

    let content = rect.shrink2(vec2(6.0, 0.0));
    let mut y = mode_bar.bottom() + 7.0;
    let icon = if state.key_tools_mode == KeyToolsMode::Keys {
        "◆ 0"
    } else {
        "L 1"
    };
    painter.text(
        pos2(content.left(), y + 8.0),
        Align2::LEFT_CENTER,
        icon,
        FontId::monospace(8.0),
        theme::ACCENT,
    );

    if state.key_tools_mode == KeyToolsMode::Keys {
        let scope_labels = [("O", "Object"), (":", "Channel"), ("A", "All")];
        for (index, (label, hint)) in scope_labels.into_iter().enumerate() {
            let button = Rect::from_min_size(
                pos2(content.right() - 70.0 + index as f32 * 24.0, y),
                vec2(22.0, 18.0),
            );
            if outlined_button(
                ui,
                button,
                label,
                state.key_scope == index,
                ("key-scope", index),
            )
            .on_hover_text(hint)
            .clicked()
            {
                state.key_scope = index;
            }
        }
    }
    y += 25.0;
    separator(ui, content.left(), content.right(), y);
    y += 5.0;

    let sections = match state.key_tools_mode {
        KeyToolsMode::Keys => [("", "Align"), ("", "Stagger"), ("", "Stretch")],
        KeyToolsMode::Layers => [
            ("|L|", "Layer Align"),
            ("L/L", "Layer Stagger"),
            ("<L>", "Layer Shift"),
        ],
    };
    let section_width = (content.width() - 4.0) / 3.0;
    for (index, (label, hint)) in sections.into_iter().enumerate() {
        let button = Rect::from_min_size(
            pos2(content.left() + index as f32 * (section_width + 2.0), y),
            vec2(section_width, 28.0),
        );
        let selected = match state.key_tools_mode {
            KeyToolsMode::Keys => state.key_section == index,
            KeyToolsMode::Layers => state.layer_section == index,
        };
        let response = outlined_button(ui, button, label, selected, ("key-section", index))
            .on_hover_text(hint);
        if state.key_tools_mode == KeyToolsMode::Keys {
            paint_key_section_icon(&painter, button, index, selected);
        }
        if response.clicked() {
            match state.key_tools_mode {
                KeyToolsMode::Keys => state.key_section = index,
                KeyToolsMode::Layers => state.layer_section = index,
            }
        }
    }
    y += 33.0;
    separator(ui, content.left(), content.right(), y);
    y += 5.0;

    let section = match state.key_tools_mode {
        KeyToolsMode::Keys => state.key_section,
        KeyToolsMode::Layers => state.layer_section,
    };
    let section_title = ["ALIGN", "STAGGER", "STRETCH / SHIFT"][section];
    painter.text(
        pos2(content.left(), y),
        Align2::LEFT_TOP,
        section_title,
        FontId::monospace(6.0),
        theme::TEXT_MUTED,
    );
    y += 13.0;

    let action_labels = match (state.key_tools_mode, section) {
        (KeyToolsMode::Keys, 0) => ["", "", ""],
        (KeyToolsMode::Keys, 1) => ["K..K", "<>", "->"],
        (KeyToolsMode::Keys, _) => ["80%", "100%", "120%"],
        (KeyToolsMode::Layers, 0) => ["|L", "L|L", "L|"],
        (KeyToolsMode::Layers, 1) => ["L..L", "<>", "->"],
        (KeyToolsMode::Layers, _) => ["<<", "0", ">>"],
    };
    let action_width = (content.width() - 6.0) / 3.0;
    for (index, label) in action_labels.into_iter().enumerate() {
        let button = Rect::from_min_size(
            pos2(content.left() + index as f32 * (action_width + 3.0), y),
            vec2(action_width, 22.0),
        );
        let response = outlined_button(ui, button, label, false, ("key-action", index));
        if state.key_tools_mode == KeyToolsMode::Keys && section == 0 {
            paint_key_action_icon(&painter, button, index);
        }
        if response.clicked() {
            state.status = format!("{section_title} action {} · preview only", index + 1);
        }
    }
}

fn paint_key_section_icon(painter: &egui::Painter, rect: Rect, index: usize, selected: bool) {
    let color = if selected {
        theme::TEXT
    } else {
        theme::TEXT_SECONDARY
    };
    let center = rect.center();
    match index {
        0 => {
            for x in [center.x - 13.0, center.x + 13.0] {
                painter.line_segment(
                    [pos2(x, center.y - 6.0), pos2(x, center.y + 6.0)],
                    Stroke::new(1.0, color),
                );
            }
            paint_diamond(painter, center, 3.5, color, true);
        }
        1 => {
            paint_diamond(painter, center + vec2(-11.0, -4.0), 3.2, color, true);
            paint_diamond(painter, center + vec2(11.0, 4.0), 3.2, color, true);
            for step in 0..4 {
                let t = (step as f32 + 0.5) / 4.0;
                let point = pos2(
                    egui::lerp(center.x - 7.0..=center.x + 7.0, t),
                    egui::lerp(center.y - 3.0..=center.y + 3.0, t),
                );
                painter.circle_filled(point, 0.8, color);
            }
        }
        _ => {
            painter.line_segment(
                [center + vec2(-13.0, 0.0), center + vec2(13.0, 0.0)],
                Stroke::new(1.0, color),
            );
            paint_arrow_head(painter, center + vec2(-13.0, 0.0), false, color);
            paint_arrow_head(painter, center + vec2(13.0, 0.0), true, color);
            paint_diamond(painter, center, 3.4, color, true);
        }
    }
}

fn paint_key_action_icon(painter: &egui::Painter, rect: Rect, index: usize) {
    let color = theme::TEXT_SECONDARY;
    let center = rect.center();
    match index {
        0 => {
            painter.line_segment(
                [center + vec2(-6.0, -6.0), center + vec2(-6.0, 6.0)],
                Stroke::new(1.0, color),
            );
            paint_diamond(painter, center + vec2(3.0, 0.0), 3.2, color, true);
        }
        1 => {
            paint_diamond(painter, center + vec2(-10.0, 0.0), 3.0, color, true);
            painter.line_segment(
                [center + vec2(0.0, -6.0), center + vec2(0.0, 6.0)],
                Stroke::new(1.0, color),
            );
            paint_diamond(painter, center + vec2(10.0, 0.0), 3.0, color, true);
        }
        _ => {
            paint_diamond(painter, center + vec2(-3.0, 0.0), 3.2, color, true);
            painter.line_segment(
                [center + vec2(6.0, -6.0), center + vec2(6.0, 6.0)],
                Stroke::new(1.0, color),
            );
        }
    }
}

fn paint_diamond(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
    fill: bool,
) {
    let points = vec![
        center + vec2(0.0, -radius),
        center + vec2(radius, 0.0),
        center + vec2(0.0, radius),
        center + vec2(-radius, 0.0),
    ];
    if fill {
        painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    } else {
        painter.add(egui::Shape::closed_line(points, Stroke::new(1.0, color)));
    }
}

fn paint_arrow_head(painter: &egui::Painter, tip: egui::Pos2, points_right: bool, color: Color32) {
    let direction = if points_right { -1.0 } else { 1.0 };
    painter.line_segment(
        [tip, tip + vec2(4.0 * direction, -3.0)],
        Stroke::new(1.0, color),
    );
    painter.line_segment(
        [tip, tip + vec2(4.0 * direction, 3.0)],
        Stroke::new(1.0, color),
    );
}

fn paint_action_rail(ui: &mut egui::Ui, rect: Rect, state: &mut TimelineState) {
    let painter = ui.painter().clone();
    painter.rect_filled(rect, 0.0, theme::APP);
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0, theme::BORDER_STRONG),
    );
    painter.line_segment(
        [rect.left_top(), rect.left_bottom()],
        Stroke::new(1.0, theme::BORDER),
    );

    let head = Rect::from_min_size(rect.min, vec2(rect.width(), RULER_HEIGHT));
    painter.line_segment(
        [head.left_bottom(), head.right_bottom()],
        Stroke::new(1.0, theme::BORDER_STRONG),
    );
    painter.text(
        pos2(head.left() + 19.0, head.center().y),
        Align2::CENTER_CENTER,
        "S",
        FontId::monospace(6.0),
        theme::TEXT_MUTED,
    );
    painter.text(
        pos2(head.left() + 40.0, head.center().y),
        Align2::CENTER_CENTER,
        "M",
        FontId::monospace(6.0),
        theme::TEXT_MUTED,
    );

    let band_height = ((rect.height() - RULER_HEIGHT) / BAND_COUNT as f32).min(DEFAULT_BAND_HEIGHT);
    for band in 0..BAND_COUNT {
        let top = head.bottom() + band as f32 * band_height;
        let row = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), band_height));
        painter.line_segment(
            [row.left_bottom(), row.right_bottom()],
            Stroke::new(1.0, theme::BORDER),
        );
        let solo = Rect::from_min_size(row.min + vec2(8.0, 7.0), vec2(18.0, 18.0));
        let mute = solo.translate(vec2(21.0, 0.0));
        if rail_button(ui, solo, "S", state.solo[band], false, ("band-solo", band)).clicked() {
            state.solo[band] = !state.solo[band];
        }
        if rail_button(ui, mute, "M", state.muted[band], true, ("band-mute", band)).clicked() {
            state.muted[band] = !state.muted[band];
        }
        let grip_y = row.bottom() - 4.0;
        for offset in [0.0, 3.0] {
            painter.line_segment(
                [
                    pos2(row.center().x - 7.0, grip_y + offset),
                    pos2(row.center().x + 7.0, grip_y + offset),
                ],
                Stroke::new(1.0, theme::BORDER_STRONG),
            );
        }
    }
}

fn paint_time_plane(ui: &mut egui::Ui, rect: Rect, state: &mut TimelineState) {
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 0.0, theme::PANEL);
    let ruler = Rect::from_min_size(rect.min, vec2(rect.width(), RULER_HEIGHT));
    painter.rect_filled(ruler, 0.0, theme::APP);
    painter.line_segment(
        [ruler.left_bottom(), ruler.right_bottom()],
        Stroke::new(1.0, theme::BORDER_STRONG),
    );

    for index in 0..=16 {
        let x = egui::lerp(rect.x_range(), index as f32 / 16.0);
        let major = index % 4 == 0;
        painter.line_segment(
            [
                pos2(x, ruler.bottom() - if major { 8.0 } else { 4.0 }),
                pos2(x, rect.bottom()),
            ],
            Stroke::new(
                if major { 1.0 } else { 0.5 },
                if major {
                    theme::BORDER
                } else {
                    theme::BORDER.gamma_multiply(0.55)
                },
            ),
        );
        if index % 2 == 0 {
            let beat = 52.0 + index as f32 * 0.25;
            let label = if beat.fract() == 0.0 {
                format!("{beat:.0}")
            } else {
                format!("{:.1}", beat)
            };
            painter.text(
                pos2(x + 4.0, ruler.top() + 5.0),
                Align2::LEFT_TOP,
                label,
                FontId::monospace(7.0),
                if major {
                    theme::TEXT_SECONDARY
                } else {
                    theme::TEXT_MUTED
                },
            );
        }
    }
    painter.rect_filled(
        Rect::from_min_size(ruler.min + vec2(5.0, 4.0), vec2(55.0, 12.0)),
        0.0,
        theme::RAISED,
    );
    painter.text(
        ruler.min + vec2(8.0, 6.0),
        Align2::LEFT_TOP,
        "TIME / BEAT",
        FontId::monospace(6.0),
        theme::ACCENT,
    );

    let band_height = ((rect.height() - RULER_HEIGHT) / BAND_COUNT as f32).min(DEFAULT_BAND_HEIGHT);
    let row_two = Rect::from_min_size(
        pos2(rect.left(), ruler.bottom() + 2.0 * band_height),
        vec2(rect.width(), band_height),
    );
    painter.rect_filled(row_two, 0.0, theme::SHAPE.gamma_multiply(0.07));
    for band in 0..BAND_COUNT {
        let y = ruler.bottom() + (band + 1) as f32 * band_height;
        painter.line_segment(
            [pos2(rect.left(), y), pos2(rect.right(), y)],
            Stroke::new(1.0, theme::BORDER),
        );
    }

    for bar in BARS {
        let top = ruler.bottom() + bar.row as f32 * band_height + 4.0;
        let bar_rect = Rect::from_min_max(
            pos2(egui::lerp(rect.x_range(), bar.start), top),
            pos2(
                egui::lerp(rect.x_range(), bar.end),
                top + (band_height - 7.0).max(20.0),
            ),
        );
        let selected = state.selected_bar == bar.id;
        let dimmed =
            state.muted[bar.row] || (state.solo.iter().any(|value| *value) && !state.solo[bar.row]);
        let base_fill = if bar.id == "city" {
            Color32::from_rgb(142, 166, 141)
        } else {
            bar.color
        };
        let fill = if dimmed {
            base_fill.gamma_multiply(0.28)
        } else {
            base_fill
        };
        painter.rect_filled(bar_rect, 0.0, fill);
        painter.rect_stroke(
            bar_rect,
            0.0,
            Stroke::new(1.0, Color32::from_rgb(55, 55, 55)),
            StrokeKind::Inside,
        );
        painter.line_segment(
            [
                pos2(bar_rect.left(), bar_rect.top() - 1.0),
                pos2(bar_rect.right(), bar_rect.top() - 1.0),
            ],
            Stroke::new(1.0, mix_with_black(fill, 0.48)),
        );
        painter.line_segment(
            [bar_rect.left_top(), bar_rect.right_top()],
            Stroke::new(1.0, mix_with_white(fill, 0.15)),
        );
        if selected {
            for offset in [-4.0, -3.0] {
                painter.line_segment(
                    [
                        pos2(bar_rect.left(), bar_rect.top() + offset),
                        pos2(bar_rect.right(), bar_rect.top() + offset),
                    ],
                    Stroke::new(1.0, theme::TEXT),
                );
            }
            painter.line_segment(
                [
                    pos2(bar_rect.left(), bar_rect.bottom() + 1.0),
                    pos2(bar_rect.right(), bar_rect.bottom() + 1.0),
                ],
                Stroke::new(1.0, theme::TEXT),
            );
        }
        painter.rect_filled(
            Rect::from_min_size(bar_rect.min, vec2(4.0, bar_rect.height())),
            0.0,
            Color32::from_rgb(58, 58, 58),
        );
        paint_bar_contents(&painter, bar_rect, bar.id, bar.kind, bar.label);

        let response = ui.interact(bar_rect, Id::new(("timeline-bar", bar.id)), Sense::click());
        if response.hovered() {
            painter.rect_stroke(
                bar_rect.shrink(1.0),
                0.0,
                Stroke::new(1.0, theme::ACCENT),
                StrokeKind::Inside,
            );
        }
        if response.clicked() {
            state.selected_bar = bar.id;
            state.status = format!("{} · Timeline selection · mock-local", bar.label);
        }
    }

    let playhead_x = egui::lerp(rect.x_range(), state.playhead);
    painter.line_segment(
        [
            pos2(playhead_x, ruler.top()),
            pos2(playhead_x, rect.bottom()),
        ],
        Stroke::new(1.0, theme::TEXT),
    );
    painter.add(egui::Shape::convex_polygon(
        vec![
            pos2(playhead_x - 4.0, ruler.top()),
            pos2(playhead_x + 4.0, ruler.top()),
            pos2(playhead_x, ruler.top() + 7.0),
        ],
        theme::TEXT,
        Stroke::NONE,
    ));
}

fn mix_with_black(color: Color32, amount: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 * amount) as u8,
        (color.g() as f32 * amount) as u8,
        (color.b() as f32 * amount) as u8,
    )
}

fn mix_with_white(color: Color32, amount: f32) -> Color32 {
    Color32::from_rgb(
        egui::lerp(color.r() as f32..=255.0, amount) as u8,
        egui::lerp(color.g() as f32..=255.0, amount) as u8,
        egui::lerp(color.b() as f32..=255.0, amount) as u8,
    )
}

fn paint_bar_contents(painter: &egui::Painter, rect: Rect, id: &str, kind: &str, label: &str) {
    let ink = Color32::from_rgb(17, 17, 17);
    let muted = Color32::from_rgb(36, 36, 36);
    let mut x = rect.left() + 8.0;
    let kind_left = x;
    x = bar_cell(
        painter,
        rect,
        x,
        15.0,
        if id == "audio" { "" } else { kind },
        false,
    );
    if id == "audio" {
        paint_music_note(painter, bar_cell_rect(rect, kind_left, 15.0), ink);
    }

    match id {
        "audio" => {
            x = bar_cell(painter, rect, x + 3.0, 18.0, "S", true);
            x = bar_cell(painter, rect, x + 2.0, 18.0, "M", true);
            x = bar_cell(painter, rect, x + 2.0, 29.0, "◆ 1", true);
            painter.text(
                pos2(x + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::monospace(8.0),
                ink,
            );
            painter.text(
                pos2(x + 92.0, rect.center().y),
                Align2::LEFT_CENTER,
                "╱╲╱▁╲╱╲▁╱╲",
                FontId::monospace(8.0),
                muted,
            );
        }
        "pulse" => {
            let fold_left = x + 3.0;
            x = bar_cell(painter, rect, fold_left, 28.0, "", true);
            paint_group_fold(painter, bar_cell_rect(rect, fold_left, 28.0), ink);
            x = bar_cell(painter, rect, x + 2.0, 18.0, "S", true);
            x = bar_cell(painter, rect, x + 2.0, 18.0, "M", true);
            x = bar_cell(painter, rect, x + 2.0, 29.0, "◆ 3", true);
            x = bar_cell(painter, rect, x + 5.0, 62.0, label, false);
            painter.text(
                pos2(x + 10.0, rect.center().y),
                Align2::LEFT_CENTER,
                "IN  →  Echo Bloom  →  OUT",
                FontId::monospace(7.0),
                ink,
            );
            let depth = bar_cell(painter, rect, x + 150.0, 18.0, "", true);
            paint_depth_icon(
                painter,
                Rect::from_center_size(
                    pos2(depth - 9.0, rect.center().y),
                    vec2(18.0, (rect.height() - 8.0).max(16.0)),
                ),
                ink,
            );
        }
        "city" => {
            x = bar_cell(painter, rect, x + 3.0, 18.0, "S", true);
            x = bar_cell(painter, rect, x + 2.0, 18.0, "M", true);
            x = bar_cell(painter, rect, x + 2.0, 29.0, "◆ 1", true);
            x = bar_cell(painter, rect, x + 5.0, 76.0, "Pulse rings", false);
            painter.text(
                pos2(x + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::monospace(8.0),
                ink,
            );
            let depth = bar_cell(painter, rect, x + 70.0, 18.0, "", true);
            paint_depth_icon(
                painter,
                Rect::from_center_size(
                    pos2(depth - 9.0, rect.center().y),
                    vec2(18.0, (rect.height() - 8.0).max(16.0)),
                ),
                ink,
            );
        }
        "title" => {
            x = bar_cell(painter, rect, x + 3.0, 18.0, "S", true);
            x = bar_cell(painter, rect, x + 2.0, 18.0, "M", true);
            x = bar_cell(painter, rect, x + 2.0, 29.0, "◆ 1", true);
            painter.text(
                pos2(x + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::monospace(8.0),
                ink,
            );
            let depth = bar_cell(painter, rect, x + 93.0, 18.0, "", true);
            paint_depth_icon(
                painter,
                Rect::from_center_size(
                    pos2(depth - 9.0, rect.center().y),
                    vec2(18.0, (rect.height() - 8.0).max(16.0)),
                ),
                ink,
            );
        }
        _ => {
            x = bar_cell(painter, rect, x + 3.0, 18.0, "S", true);
            x = bar_cell(painter, rect, x + 2.0, 18.0, "M", true);
            x = bar_cell(painter, rect, x + 2.0, 29.0, "◇+", true);
            painter.text(
                pos2(x + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::monospace(8.0),
                ink,
            );
            let depth = bar_cell(painter, rect, x + 94.0, 18.0, "", true);
            paint_depth_icon(
                painter,
                Rect::from_center_size(
                    pos2(depth - 9.0, rect.center().y),
                    vec2(18.0, (rect.height() - 8.0).max(16.0)),
                ),
                ink,
            );
        }
    }
}

fn bar_cell_rect(bar: Rect, x: f32, width: f32) -> Rect {
    Rect::from_center_size(
        pos2(x + width * 0.5, bar.center().y),
        vec2(width, (bar.height() - 8.0).max(16.0)),
    )
}

fn paint_music_note(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stem_x = rect.center().x + 2.0;
    painter.line_segment(
        [
            pos2(stem_x, rect.center().y - 5.0),
            pos2(stem_x, rect.center().y + 3.0),
        ],
        Stroke::new(1.0, color),
    );
    painter.line_segment(
        [
            pos2(stem_x, rect.center().y - 5.0),
            pos2(stem_x + 4.0, rect.center().y - 4.0),
        ],
        Stroke::new(1.0, color),
    );
    painter.circle_filled(pos2(stem_x - 2.0, rect.center().y + 4.0), 2.0, color);
}

fn paint_group_fold(painter: &egui::Painter, rect: Rect, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![
            rect.center() + vec2(-8.0, -2.0),
            rect.center() + vec2(-2.0, -2.0),
            rect.center() + vec2(-5.0, 2.0),
        ],
        color,
        Stroke::NONE,
    ));
    painter.circle_filled(rect.center() + vec2(6.0, 0.0), 3.0, color);
}

fn paint_depth_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    for offset in [-3.0, 0.0, 3.0] {
        painter.line_segment(
            [
                pos2(rect.center().x - 5.0, rect.center().y + offset),
                pos2(rect.center().x + 5.0, rect.center().y + offset),
            ],
            Stroke::new(1.0, color),
        );
    }
}

fn paint_timeline_view_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let icon = Rect::from_center_size(rect.center(), vec2(10.0, 9.0));
    painter.rect_stroke(icon, 0.0, Stroke::new(1.0, color), StrokeKind::Inside);
    for y in [icon.top() + 3.0, icon.top() + 6.0] {
        painter.line_segment(
            [pos2(icon.left() + 2.0, y), pos2(icon.right() - 2.0, y)],
            Stroke::new(1.0, color),
        );
    }
}

fn paint_graph_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let left = rect.left() + 6.0;
    let right = rect.right() - 6.0;
    let points = vec![
        pos2(left, rect.center().y + 3.0),
        pos2(left + (right - left) * 0.34, rect.center().y - 2.0),
        pos2(left + (right - left) * 0.67, rect.center().y + 1.0),
        pos2(right, rect.center().y - 4.0),
    ];
    painter.add(egui::Shape::line(points, Stroke::new(1.0, color)));
}

fn bar_cell(
    painter: &egui::Painter,
    bar: Rect,
    x: f32,
    width: f32,
    label: &str,
    filled: bool,
) -> f32 {
    let rect = bar_cell_rect(bar, x, width);
    if filled {
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(8, 8, 8, 34));
    }
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, Color32::from_rgb(48, 48, 48)),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(if width > 40.0 { 7.0 } else { 8.0 }),
        Color32::from_rgb(20, 20, 20),
    );
    rect.right()
}

fn flat_button(
    ui: &mut egui::Ui,
    rect: Rect,
    label: &str,
    selected: bool,
    id: impl std::hash::Hash + std::fmt::Debug,
) -> Response {
    let response = ui.interact(rect, Id::new(id), Sense::click());
    let fill = if selected || response.hovered() {
        theme::RAISED
    } else {
        theme::PANEL
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(
            1.0,
            if selected {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    if selected {
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - 2.0), rect.right_bottom()),
            0.0,
            theme::ACCENT,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(8.0),
        if selected {
            theme::TEXT
        } else {
            theme::TEXT_MUTED
        },
    );
    response
}

fn tab_button(
    ui: &mut egui::Ui,
    rect: Rect,
    label: &str,
    selected: bool,
    id: impl std::hash::Hash + std::fmt::Debug,
) -> Response {
    let response = ui.interact(rect, Id::new(id), Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(rect, 0.0, theme::RAISED);
    }
    if selected {
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - 2.0), rect.right_bottom()),
            0.0,
            theme::ACCENT,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(7.0),
        if selected {
            theme::ACCENT
        } else {
            theme::TEXT_MUTED
        },
    );
    response
}

fn outlined_button(
    ui: &mut egui::Ui,
    rect: Rect,
    label: &str,
    selected: bool,
    id: impl std::hash::Hash + std::fmt::Debug,
) -> Response {
    let response = ui.interact(rect, Id::new(id), Sense::click());
    ui.painter().rect_filled(
        rect,
        0.0,
        if response.hovered() {
            theme::HOVER
        } else {
            theme::PANEL
        },
    );
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(
            1.0,
            if selected || response.hovered() {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    if selected {
        ui.painter().rect_filled(
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - 2.0), rect.right_bottom()),
            0.0,
            theme::ACCENT,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(if rect.height() > 24.0 { 9.0 } else { 7.0 }),
        if selected {
            theme::TEXT
        } else {
            theme::TEXT_SECONDARY
        },
    );
    response
}

fn rail_button(
    ui: &mut egui::Ui,
    rect: Rect,
    label: &str,
    selected: bool,
    warning: bool,
    id: impl std::hash::Hash + std::fmt::Debug,
) -> Response {
    let response = ui.interact(rect, Id::new(id), Sense::click());
    ui.painter().rect_filled(
        rect,
        0.0,
        if selected {
            theme::RAISED
        } else {
            theme::PANEL
        },
    );
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(
            1.0,
            if selected {
                if warning {
                    theme::WARNING
                } else {
                    theme::ACCENT
                }
            } else if response.hovered() {
                theme::TEXT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(7.0),
        if selected && warning {
            theme::WARNING
        } else if selected {
            theme::ACCENT
        } else {
            theme::TEXT_SECONDARY
        },
    );
    response
}

fn separator(ui: &egui::Ui, left: f32, right: f32, y: f32) {
    ui.painter().line_segment(
        [pos2(left, y), pos2(right, y)],
        Stroke::new(1.0, theme::BORDER),
    );
}
