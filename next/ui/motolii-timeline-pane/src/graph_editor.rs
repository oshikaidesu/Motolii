//! 選択キーの Bezier 補間を視覚的に編集する Graph Editor。
//!
//! この component は、選択キーから現在の4制御値を投影し、canvas のハンドルを
//! x/y へ戻し、数値欄と同じ GraphControlInput へ収束させる。Document への
//! 書き込みは PaneState::commit_graph_editor が既存の SetKeyInterp へ委譲する。

use iced::widget::canvas;
use iced::widget::{button, canvas as canvas_widget, column, container, row, text, text_input};
use iced::{Background, Element, Length, Point, Rectangle, Size};
use motolii_store::{Interp, StoreView};

use crate::state::Session;
use crate::tokens::Colors;
use crate::{Message, TimelinePane};

/* motolii-component
id = "timeline.graph_editor"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["project", "update_control", "commit_graph_editor"]
meaning = ["GraphControlInput", "GraphHandleDragged", "GraphHandleReleased"]
evaluation = ["parse_control", "bezier_point", "committing_graph_control_inputs_writes_bezier"]
render = ["view", "GraphCanvas"]
observable = ["graph_handle_drag_updates_the_control_point", "committing_graph_control_inputs_writes_bezier"]
*/

const DEFAULT_CONTROLS: [f32; 4] = [0.333, 0.0, 0.667, 1.0];
const PLOT_PADDING: f32 = 16.0;
const HANDLE_HIT: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphControl {
    X1,
    Y1,
    X2,
    Y2,
}

impl GraphControl {
    pub const ALL: [Self; 4] = [Self::X1, Self::Y1, Self::X2, Self::Y2];

    pub(crate) fn index(self) -> usize {
        match self {
            Self::X1 => 0,
            Self::Y1 => 1,
            Self::X2 => 2,
            Self::Y2 => 3,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::X1 => "x1",
            Self::Y1 => "y1",
            Self::X2 => "x2",
            Self::Y2 => "y2",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEditorProjection {
    pub controls: [f32; 4],
    pub has_bezier_key: bool,
}

/// 選択中の最初のキーの補間を読む。複数選択で補間が異なる場合は、
/// 既存の一括編集と同じく先頭キーを編集の顔にする。
pub fn project(store: &StoreView<'_>, session: &Session) -> GraphEditorProjection {
    let Some(key) = session.selected_keys.first() else {
        return GraphEditorProjection { controls: DEFAULT_CONTROLS, has_bezier_key: false };
    };
    let Some(fps) = store.composition().ok().flatten().map(|comp| comp.fps) else {
        return GraphEditorProjection { controls: DEFAULT_CONTROLS, has_bezier_key: false };
    };
    let Ok(Some(track)) = store.track(key.layer, &key.property) else {
        return GraphEditorProjection { controls: DEFAULT_CONTROLS, has_bezier_key: false };
    };
    let Some(found) = track
        .keys()
        .iter()
        .find(|candidate| candidate.t.try_to_frame_round(fps).ok() == Some(key.frame))
    else {
        return GraphEditorProjection { controls: DEFAULT_CONTROLS, has_bezier_key: false };
    };
    match found.interp {
        Interp::Bezier { x1, y1, x2, y2 } => GraphEditorProjection {
            controls: [x1, y1, x2, y2],
            has_bezier_key: true,
        },
        _ => GraphEditorProjection { controls: DEFAULT_CONTROLS, has_bezier_key: false },
    }
}

pub fn parse_control(input: &str) -> Result<f32, &'static str> {
    let value = input
        .trim()
        .parse::<f32>()
        .map_err(|_| "Bezier の制御値は数値で入力してください")?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err("Bezier の制御値は0から1の範囲で入力してください");
    }
    Ok(value)
}

pub fn update_control(mut controls: [f32; 4], control: GraphControl, value: f32) -> [f32; 4] {
    controls[control.index()] = value.clamp(0.0, 1.0);
    controls
}

pub fn bezier_point(controls: [f32; 4], t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let b0 = u * u * u;
    let b1 = 3.0 * u * u * t;
    let b2 = 3.0 * u * t * t;
    let b3 = t * t * t;
    (
        b1 * controls[0] + b2 * controls[2] + b3,
        b1 * controls[1] + b2 * controls[3] + b3,
    )
}

pub(crate) fn view(pane: &TimelinePane) -> Element<'static, Message> {
    let Some(projection) = pane.graph_editor.as_ref() else {
        return container(text("Graph Editor — select a keyframe")).into();
    };
    let values = projection.controls;
    let graph = canvas_widget(GraphCanvas { values, colors: pane.colors })
        .width(Length::Fill)
        .height(Length::Fixed(180.0));
    let mut controls = Vec::with_capacity(GraphControl::ALL.len());
    for control in GraphControl::ALL {
        let index = control.index();
        let displayed = pane.graph_drafts[index]
            .clone()
            .unwrap_or_else(|| format!("{:.3}", values[index]));
        controls.push(
            row![
                text(control.label()).width(Length::Fixed(20.0)),
                text_input("0.000", displayed)
                    .on_input(move |input| Message::GraphControlInput(control, input))
                    .on_submit(Message::GraphCommit)
                    .width(Length::Fixed(pane.dims.inspector_value_width))
                    .padding([0.0, pane.dims.spacing_xs]),
            ]
            .spacing(pane.dims.spacing_xs)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
        );
    }
    let state_label = if projection.has_bezier_key {
        "Selected Bezier key"
    } else {
        "Selected key: commit to create Bezier"
    };
    container(column![
        row![
            text("GRAPH EDITOR").size(pane.dims.caption_text),
            text(state_label).size(pane.dims.caption_text).color(pane.colors.text_muted),
            button(text("Hide")).on_press(Message::ToggleGraphEditor),
        ]
        .spacing(pane.dims.spacing_s)
        .align_y(iced::alignment::Vertical::Center),
        graph,
        row(controls).spacing(pane.dims.spacing_m),
        text("Drag the handles or enter x1/y1/x2/y2, then press Enter.")
            .size(pane.dims.caption_text)
            .color(pane.colors.text_muted),
    ])
    .padding([pane.dims.spacing_s, pane.dims.spacing_m])
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(pane.colors.surface_panel)),
        ..container::Style::default()
    })
    .into()
}

struct GraphCanvas {
    values: [f32; 4],
    colors: Colors,
}

#[derive(Default)]
struct GraphCanvasState {
    active: Option<GraphControl>,
}

impl GraphCanvas {
    fn plot_size(bounds: Rectangle) -> (f32, f32) {
        (
            (bounds.width - PLOT_PADDING * 2.0).max(1.0),
            (bounds.height - PLOT_PADDING * 2.0).max(1.0),
        )
    }

    fn point(bounds: Rectangle, controls: [f32; 4], control: GraphControl) -> Point {
        let (width, height) = Self::plot_size(bounds);
        let (x, y) = match control {
            GraphControl::X1 | GraphControl::Y1 => (controls[0], controls[1]),
            GraphControl::X2 | GraphControl::Y2 => (controls[2], controls[3]),
        };
        Point::new(PLOT_PADDING + x * width, PLOT_PADDING + (1.0 - y) * height)
    }

    fn normalized(bounds: Rectangle, position: Point) -> (f32, f32) {
        let (width, height) = Self::plot_size(bounds);
        (
            ((position.x - PLOT_PADDING) / width).clamp(0.0, 1.0),
            (1.0 - (position.y - PLOT_PADDING) / height).clamp(0.0, 1.0),
        )
    }

    fn hit(bounds: Rectangle, values: [f32; 4], position: Point) -> Option<GraphControl> {
        GraphControl::ALL.into_iter().find(|&control| {
            Self::point(bounds, values, control).distance(position) <= HANDLE_HIT
        })
    }
}

impl canvas::Program<Message> for GraphCanvas {
    type State = GraphCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let canvas::Event::Mouse(mouse_event) = event else {
            return None;
        };
        match mouse_event {
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                let position = cursor.position_in(bounds)?;
                let control = Self::hit(bounds, self.values, position)?;
                state.active = Some(control);
                Some(canvas::Action::publish(Message::GraphHandleGrabbed(control)).and_capture())
            }
            iced::mouse::Event::CursorMoved { .. } => {
                let control = state.active?;
                let position = cursor.position_in(bounds)?;
                let (x, y) = Self::normalized(bounds, position);
                Some(
                    canvas::Action::publish(Message::GraphHandleDragged { control, x, y })
                        .and_capture(),
                )
            }
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                let control = state.active.take()?;
                Some(canvas::Action::publish(Message::GraphHandleReleased(control)).and_capture())
            }
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) => {
                state.active.take().map(|_| {
                    canvas::Action::publish(Message::GraphHandleCancelled).and_capture()
                })
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let (width, height) = Self::plot_size(bounds);
        let origin = Point::new(PLOT_PADDING, PLOT_PADDING + height);
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(bounds.width, bounds.height),
            self.colors.surface_app,
        );
        for fraction in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let x = PLOT_PADDING + fraction * width;
            let y = PLOT_PADDING + (1.0 - fraction) * height;
            frame.stroke(
                &canvas::Path::line(
                    Point::new(x, PLOT_PADDING),
                    Point::new(x, PLOT_PADDING + height),
                ),
                canvas::Stroke::default()
                    .with_color(self.colors.timeline_grid_minor)
                    .with_width(1.0),
            );
            frame.stroke(
                &canvas::Path::line(
                    Point::new(PLOT_PADDING, y),
                    Point::new(PLOT_PADDING + width, y),
                ),
                canvas::Stroke::default()
                    .with_color(self.colors.timeline_grid_minor)
                    .with_width(1.0),
            );
        }
        let mut curve = canvas::Path::builder();
        for index in 0..=32 {
            let t = index as f32 / 32.0;
            let (x, y) = bezier_point(self.values, t);
            let point = Point::new(PLOT_PADDING + x * width, PLOT_PADDING + (1.0 - y) * height);
            if index == 0 {
                curve.move_to(point);
            } else {
                curve.line_to(point);
            }
        }
        frame.stroke(
            &curve.build(),
            canvas::Stroke::default()
                .with_color(self.colors.action_active)
                .with_width(2.0),
        );
        let first = Self::point(bounds, self.values, GraphControl::X1);
        let second = Self::point(bounds, self.values, GraphControl::X2);
        frame.stroke(
            &canvas::Path::line(origin, first),
            canvas::Stroke::default()
                .with_color(self.colors.text_muted)
                .with_width(1.0),
        );
        frame.stroke(
            &canvas::Path::line(Point::new(PLOT_PADDING + width, PLOT_PADDING), second),
            canvas::Stroke::default()
                .with_color(self.colors.text_muted)
                .with_width(1.0),
        );
        for point in [first, second] {
            frame.fill(&canvas::Path::circle(point, 5.0), self.colors.action_active);
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if state.active.is_some() {
            iced::mouse::Interaction::Grabbing
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_point_keeps_the_curve_endpoints() {
        let controls = [0.25, 0.1, 0.75, 0.9];
        assert_eq!(bezier_point(controls, 0.0), (0.0, 0.0));
        assert_eq!(bezier_point(controls, 1.0), (1.0, 1.0));
    }

    #[test]
    fn graph_handle_drag_updates_the_control_point() {
        let values = update_control(DEFAULT_CONTROLS, GraphControl::X1, 1.5);
        assert_eq!(values[0], 1.0);
        let values = update_control(values, GraphControl::Y1, -1.0);
        assert_eq!(values[1], 0.0);
    }

    #[test]
    fn parse_control_rejects_values_outside_the_graph() {
        assert!(parse_control("0.75").is_ok());
        assert!(parse_control("1.5").is_err());
        assert!(parse_control("nan").is_err());
    }
}
