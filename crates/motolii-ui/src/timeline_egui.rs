//! egui版Timelineのskin。
//!
//! ui/motolii-rn/src/web/TimelineSkiaCanvas.tsx の現行Skia skinを表示面へ移植する。
//! Document投影と編集意味は既存のtimeline_projection／Hostが所有する。

use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Shape, Stroke, StrokeKind, Ui, Vec2,
};
use motolii_core::RationalTime;
use motolii_doc::{Document, KeyframeId, LayerId};

use crate::timeline_projection::{
    TimelineMetrics, TimelineProjection, TimelineProjectionError, TimelineViewport,
    project_timeline,
};

const OVERVIEW_H: f32 = 22.0;
const RULER_H: f32 = 20.0;
const LOCATOR_H: f32 = 16.0;
const DESKTOP: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
const SURFACE: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
const SURFACE_HI: Color32 = Color32::from_rgb(0x46, 0x46, 0x46);
const SURFACE_LO: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
const CONTRAST: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const DIM: Color32 = Color32::from_rgb(0x75, 0x75, 0x75);
const RULER: Color32 = Color32::from_rgb(0x91, 0x91, 0x91);
const ACCENT: Color32 = Color32::from_rgb(0xff, 0xad, 0x56);
const INK: Color32 = Color32::from_rgb(0xd6, 0xd6, 0xd6);
const BAR_INK: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
const PALETTE: [Color32; 6] = [
    Color32::from_rgb(0x96, 0xaa, 0xdb),
    Color32::from_rgb(0x6f, 0xb9, 0xc1),
    Color32::from_rgb(0xbf, 0xa9, 0x73),
    Color32::from_rgb(0x89, 0xb9, 0x92),
    Color32::from_rgb(0xd6, 0x9a, 0x8b),
    Color32::from_rgb(0xc3, 0x9b, 0xc5),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelinePointerPhase {
    Down,
    Drag,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineCommand {
    Escape,
    Delete,
    Backspace,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Duplicate,
    SelectAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineWheelAction {
    Zoom,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EguiTimelineHit {
    Key { layer: LayerId, key: KeyframeId },
    Body { layer: LayerId },
    Left { layer: LayerId },
    Right { layer: LayerId },
    None,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TimelineIntent {
    Pointer {
        phase: TimelinePointerPhase,
        position: Pos2,
        time: Option<RationalTime>,
        hit: EguiTimelineHit,
        modifiers: egui::Modifiers,
    },
    Command {
        command: TimelineCommand,
        modifiers: egui::Modifiers,
    },
    Wheel {
        action: TimelineWheelAction,
        delta: Vec2,
        anchor: Option<Pos2>,
        modifiers: egui::Modifiers,
    },
}

#[derive(Debug)]
pub(crate) struct TimelineFrameOutput {
    pub(crate) response: egui::Response,
    pub(crate) intents: Vec<TimelineIntent>,
}

impl Default for TimelineTheme {
    fn default() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
struct TimelineRow {
    layer: LayerId,
    label: String,
    property: Option<&'static str>,
    start: Option<f32>,
    end: Option<f32>,
    keys: Vec<f32>,
    palette_slot: usize,
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

    painter.rect_filled(rect, CornerRadius::ZERO, DESKTOP);

    let sidebar_width = (rect.width() * 0.255).clamp(140.0, 204.0);
    let sidebar_width = sidebar_width.min((rect.width() - 1.0).max(1.0));
    let surface_width = (rect.width() - sidebar_width).max(1.0);
    let x_at =
        |fraction: f32| rect.left() + sidebar_width + fraction.clamp(0.0, 1.0) * surface_width;
    let rows = rows_from_projection(document, projection);
    let row_height = ((rect.height() - 72.0) / rows.len().max(1) as f32).clamp(20.0, 24.0);
    let rows_top = rect.top() + OVERVIEW_H + RULER_H + LOCATOR_H + 1.0;

    painter.rect_filled(
        Rect::from_min_max(
            rect.min,
            Pos2::new(rect.left() + sidebar_width, rect.bottom()),
        ),
        CornerRadius::ZERO,
        SURFACE,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(rect.left() + sidebar_width, rect.top()),
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
            let x0 = x_at(start);
            let x1 = x_at(end);
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
            Pos2::new(rect.left() + sidebar_width + 1.0, rect.top() + 1.0),
            Pos2::new(rect.right() - 2.0, rect.top() + OVERVIEW_H - 2.0),
        ),
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(0xd8, 0xd8, 0xd8)),
        StrokeKind::Inside,
    );

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
        let x = x_at(index as f32 / 10.0);
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
        let x = x_at(fraction);
        painter.line_segment(
            [
                Pos2::new(x, rect.top() + OVERVIEW_H + RULER_H + 1.0),
                Pos2::new(x, rows_top - 1.0),
            ],
            Stroke::new(1.0, Color32::from_rgb(0x8a, 0x8a, 0x8a)),
        );
        painter.text(
            Pos2::new(x + 7.0, rows_top - 12.0),
            Align2::LEFT_TOP,
            label,
            FontId::proportional(8.0),
            Color32::from_rgb(0xa8, 0xa8, 0xa8),
        );
    }

    let selected_index = primary.as_ref().and_then(|layer| {
        rows.iter()
            .position(|row| row.property.is_none() && row.layer == *layer)
    });
    for (index, row) in rows.iter().enumerate() {
        let y = rows_top + index as f32 * row_height;
        if y >= rect.bottom() {
            break;
        }
        let cy = y + row_height * 0.5;
        let selected = row.property.is_none() && selected_index == Some(index);
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), y),
                Pos2::new(rect.right(), (y + row_height - 1.0).min(rect.bottom())),
            ),
            CornerRadius::ZERO,
            if selected {
                Color32::from_rgb(0x41, 0x41, 0x41)
            } else {
                SURFACE
            },
        );
        for grid_index in 0..=10 {
            let x = x_at(grid_index as f32 / 10.0);
            painter.line_segment(
                [
                    Pos2::new(x, y),
                    Pos2::new(x, (y + row_height - 1.0).min(rect.bottom())),
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
                Pos2::new(rect.left(), y + row_height - 1.0),
                Pos2::new(rect.right(), y + row_height - 1.0),
            ],
            Stroke::new(1.0, CONTRAST),
        );

        if row.property.is_some() {
            painter.rect_filled(
                Rect::from_min_size(Pos2::new(rect.left() + 11.0, cy - 1.0), Vec2::splat(2.0)),
                CornerRadius::ZERO,
                Color32::from_rgb(0x92, 0x92, 0x92),
            );
        } else {
            draw_disclosure(&painter, rect.left() + 10.0, cy, true);
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
            draw_toggle(&painter, rect.left() + sidebar_width - 48.0, cy, "M", false);
            draw_toggle(&painter, rect.left() + sidebar_width - 31.0, cy, "S", false);
        }

        if let (Some(start), Some(end)) = (row.start, row.end) {
            let x0 = x_at(start);
            let x1 = x_at(end);
            let bar_rect = Rect::from_min_max(
                Pos2::new(x0, y + 1.0),
                Pos2::new(x1.max(x0 + 1.0), (y + row_height - 3.0).min(rect.bottom())),
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
                        Pos2::new(x1.max(x0 + 1.0) - 0.5, y + row_height - 2.5),
                    ),
                    CornerRadius::ZERO,
                    Stroke::new(1.0, Color32::WHITE),
                    StrokeKind::Inside,
                );
            }
        }
        for (key_index, key) in row.keys.iter().enumerate() {
            let x = x_at(*key);
            draw_diamond(&painter, x, cy, selected && key_index == 0);
        }
    }

    painter.line_segment(
        [
            Pos2::new(rect.left() + sidebar_width, rect.top()),
            Pos2::new(rect.left() + sidebar_width, rect.bottom()),
        ],
        Stroke::new(1.0, CONTRAST),
    );

    let duration = document.composition.duration.as_seconds_f64();
    let fraction = if duration.is_finite() && duration > 0.0 {
        (playhead.as_seconds_f64() / duration).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };
    let playhead_x = x_at(fraction);
    painter.line_segment(
        [
            Pos2::new(playhead_x, rect.top() + OVERVIEW_H + RULER_H),
            Pos2::new(
                playhead_x,
                (rows_top + rows.len() as f32 * row_height).min(rect.bottom()),
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

    if let Some(error) = projection_error {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("Timeline unavailable: {error}"),
            FontId::proportional(9.0),
            DIM,
        );
    }

    if let Some(pointer) = response.interact_pointer_pos() {
        let in_surface = pointer.x >= rect.left() + sidebar_width;
        let in_time_header = pointer.y < rows_top;
        if in_surface && (response.clicked() || response.dragged()) && in_time_header {
            let fraction =
                ((pointer.x - rect.left() - sidebar_width) / surface_width).clamp(0.0, 1.0);
            let millis = (f64::from(fraction) * duration.max(0.0) * 1000.0).round() as i64;
            if let Ok(time) = RationalTime::try_new(millis, 1000) {
                *playhead = time;
            }
        } else if response.clicked() && pointer.y >= rows_top {
            let index = ((pointer.y - rows_top) / row_height).floor() as usize;
            if let Some(row) = rows.get(index).filter(|row| row.property.is_none()) {
                *primary = Some(row.layer);
            }
        }
    }

    let intents = collect_input_intents(
        ui,
        &response,
        rect,
        sidebar_width,
        surface_width,
        rows_top,
        row_height,
        duration,
        &rows,
        projection,
    );

    TimelineFrameOutput { response, intents }
}

fn collect_input_intents(
    ui: &Ui,
    response: &egui::Response,
    rect: Rect,
    sidebar_width: f32,
    surface_width: f32,
    rows_top: f32,
    row_height: f32,
    duration: f64,
    rows: &[TimelineRow],
    projection: Option<&TimelineProjection>,
) -> Vec<TimelineIntent> {
    let events = ui.input(|input| input.events.clone());
    let modifiers = ui.input(|input| input.modifiers);
    let mut intents = Vec::new();

    for event in events {
        match event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers,
            } if rect.contains(pos) || (!pressed && response.dragged()) => {
                intents.push(TimelineIntent::Pointer {
                    phase: if pressed {
                        TimelinePointerPhase::Down
                    } else {
                        TimelinePointerPhase::Up
                    },
                    position: pos,
                    time: timeline_time_at(pos, rect, sidebar_width, surface_width, duration),
                    hit: timeline_hit_at(
                        pos,
                        rect,
                        sidebar_width,
                        surface_width,
                        rows_top,
                        row_height,
                        rows,
                        projection,
                    ),
                    modifiers,
                });
            }
            egui::Event::PointerMoved(pos) if response.dragged() && rect.contains(pos) => {
                intents.push(TimelineIntent::Pointer {
                    phase: TimelinePointerPhase::Drag,
                    position: pos,
                    time: timeline_time_at(pos, rect, sidebar_width, surface_width, duration),
                    hit: timeline_hit_at(
                        pos,
                        rect,
                        sidebar_width,
                        surface_width,
                        rows_top,
                        row_height,
                        rows,
                        projection,
                    ),
                    modifiers,
                });
            }
            egui::Event::MouseWheel {
                delta, modifiers, ..
            } if response.hovered() => {
                let anchor = response.hover_pos();
                intents.push(TimelineIntent::Wheel {
                    action: if modifiers.command || modifiers.ctrl {
                        TimelineWheelAction::Zoom
                    } else {
                        TimelineWheelAction::Pan
                    },
                    delta,
                    anchor,
                    modifiers,
                });
            }
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                if let Some(command) = timeline_command_for_key(key, modifiers) {
                    intents.push(TimelineIntent::Command { command, modifiers });
                }
            }
            _ => {}
        }
    }

    intents
}

pub(crate) fn timeline_command_for_key(
    key: egui::Key,
    modifiers: egui::Modifiers,
) -> Option<TimelineCommand> {
    let has_no_modifier =
        !(modifiers.command || modifiers.ctrl || modifiers.alt || modifiers.shift);
    match key {
        egui::Key::Escape if has_no_modifier => Some(TimelineCommand::Escape),
        egui::Key::Delete if has_no_modifier => Some(TimelineCommand::Delete),
        egui::Key::Backspace if has_no_modifier => Some(TimelineCommand::Backspace),
        egui::Key::Z if modifiers.command => Some(if modifiers.shift {
            TimelineCommand::Redo
        } else {
            TimelineCommand::Undo
        }),
        egui::Key::C if modifiers.command => Some(TimelineCommand::Copy),
        egui::Key::X if modifiers.command => Some(TimelineCommand::Cut),
        egui::Key::V if modifiers.command => Some(TimelineCommand::Paste),
        egui::Key::D if modifiers.command => Some(TimelineCommand::Duplicate),
        egui::Key::A if modifiers.command => Some(TimelineCommand::SelectAll),
        _ => None,
    }
}

fn timeline_time_at(
    pos: Pos2,
    rect: Rect,
    sidebar_width: f32,
    surface_width: f32,
    duration: f64,
) -> Option<RationalTime> {
    if pos.x < rect.left() + sidebar_width || surface_width <= 0.0 || !duration.is_finite() {
        return None;
    }
    let fraction = ((pos.x - rect.left() - sidebar_width) / surface_width).clamp(0.0, 1.0);
    let millis = (f64::from(fraction) * duration.max(0.0) * 1000.0).round() as i64;
    RationalTime::try_new(millis, 1000).ok()
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

fn timeline_hit_at(
    pos: Pos2,
    rect: Rect,
    sidebar_width: f32,
    surface_width: f32,
    rows_top: f32,
    row_height: f32,
    rows: &[TimelineRow],
    projection: Option<&TimelineProjection>,
) -> EguiTimelineHit {
    let Some(projection) = projection else {
        return EguiTimelineHit::None;
    };
    if pos.x < rect.left() + sidebar_width || pos.y < rows_top || row_height <= 0.0 {
        return EguiTimelineHit::None;
    }
    let index = ((pos.y - rows_top) / row_height).floor() as usize;
    let Some(row) = rows.get(index).filter(|row| row.property.is_none()) else {
        return EguiTimelineHit::None;
    };
    let fraction = ((pos.x - rect.left() - sidebar_width) / surface_width).clamp(0.0, 1.0);
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
        surface_width,
        row.layer,
    )
}

fn rows_from_projection(
    document: &Document,
    projection: Option<&TimelineProjection>,
) -> Vec<TimelineRow> {
    let Some(projection) = projection else {
        return Vec::new();
    };
    let mut layers = Vec::<LayerId>::new();
    for bar in projection.bars() {
        if !layers.contains(&bar.layer) {
            layers.push(bar.layer);
        }
    }
    for key in projection.keys() {
        if !layers.contains(&key.layer) {
            layers.push(key.layer);
        }
    }
    layers.sort();

    layers
        .into_iter()
        .flat_map(|layer| {
            let bars = projection
                .bars()
                .iter()
                .filter(|bar| bar.layer == layer)
                .collect::<Vec<_>>();
            let start = bars.iter().map(|bar| bar.x_start as f32).reduce(f32::min);
            let end = bars.iter().map(|bar| bar.x_end as f32).reduce(f32::max);
            let keys = projection
                .keys()
                .iter()
                .filter(|key| key.layer == layer)
                .map(|key| key.center_x as f32)
                .collect::<Vec<_>>();
            let label = document
                .layers
                .display_name(layer)
                .unwrap_or("layer")
                .to_owned();
            vec![
                TimelineRow {
                    layer,
                    label: label.clone(),
                    property: None,
                    start,
                    end,
                    keys: Vec::new(),
                    palette_slot: bars
                        .first()
                        .map_or(0, |bar| bar.band as usize % PALETTE.len()),
                },
                TimelineRow {
                    layer,
                    label: "Position".to_owned(),
                    property: Some("Position"),
                    start: None,
                    end: None,
                    keys,
                    palette_slot: bars
                        .first()
                        .map_or(0, |bar| bar.band as usize % PALETTE.len()),
                },
                TimelineRow {
                    layer,
                    label: "Parameters".to_owned(),
                    property: Some("Parameters"),
                    start: None,
                    end: None,
                    keys: Vec::new(),
                    palette_slot: bars
                        .first()
                        .map_or(0, |bar| bar.band as usize % PALETTE.len()),
                },
            ]
        })
        .collect()
}

fn draw_disclosure(painter: &egui::Painter, x: f32, y: f32, open: bool) {
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

fn draw_toggle(painter: &egui::Painter, x: f32, y: f32, label: &str, active: bool) {
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

fn draw_diamond(painter: &egui::Painter, x: f32, y: f32, selected: bool) {
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
