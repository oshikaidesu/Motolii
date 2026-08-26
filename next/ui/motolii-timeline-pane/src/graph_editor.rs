//! 選択キーの Bezier 補間を視覚的に編集する Graph Editor。
//!
//! この component は、選択キーから現在の4制御値を投影し、canvas のハンドルを
//! x/y へ戻し、数値欄と同じ GraphControlInput へ収束させる。Document への
//! 書き込みは PaneState::commit_graph_editor が既存の SetKeyInterp へ委譲する。
//!
//! Graph Editor は Timeline の代替画面ではなく、Timeline body の下に開く Detail
//! View である。選択キー・playhead・時間軸は TimelinePane の同じ投影を読む。

use iced::widget::canvas;
use iced::widget::{button, canvas as canvas_widget, column, container, row, text, text_input};
use iced::{Background, Element, Length, Point, Rectangle, Size};
use motolii_store::{Interp, StoreView};

use crate::state::Session;
use crate::tokens::{Colors, Dimensions, TimelineValues};
use crate::{Message, TimelinePane};

/* motolii-component
id = "timeline.graph_editor"
kind = "semantic"
weight = "core_edit"
maps = [519]
entry = ["project", "detail_tab", "detail_header", "view", "update_control", "commit_graph_editor"]
meaning = ["GraphControlInput", "GraphHandleDragged", "GraphHandleReleased"]
evaluation = ["parse_control", "bezier_point", "committing_graph_control_inputs_writes_bezier"]
render = ["detail_header", "view", "GraphCanvas"]
observable = ["detail_tab_keeps_timeline_body_visible", "graph_handle_drag_updates_the_control_point", "committing_graph_control_inputs_writes_bezier"]
*/

const DEFAULT_CONTROLS: [f32; 4] = [0.333, 0.0, 0.667, 1.0];

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

/// Timeline の主画面を残したまま、下部 Detail View がどの顔を出すか。
/// `Timeline` は空の詳細ではなく「Graph Editorを閉じた状態」を表す。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DetailTab {
    Timeline,
    GraphEditor,
}

pub(crate) fn detail_tab(graph_editor_open: bool) -> DetailTab {
    if graph_editor_open {
        DetailTab::GraphEditor
    } else {
        DetailTab::Timeline
    }
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
            controls: [x1 as f32, y1 as f32, x2 as f32, y2 as f32],
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
    let b1 = 3.0 * u * u * t;
    let b2 = 3.0 * u * t * t;
    let b3 = t * t * t;
    (
        b1 * controls[0] + b2 * controls[2] + b3,
        b1 * controls[1] + b2 * controls[3] + b3,
    )
}

/// Detail View のタブヘッダー。Transport は再生と時間移動だけを所有し、
/// Graph Editor の入口はここへ集約する。
pub(crate) fn detail_header(pane: &TimelinePane) -> Element<'static, Message> {
    let dims: Dimensions = pane.dims;
    let colors = pane.colors;
    let active = detail_tab(pane.graph_editor_open);
    let timeline = detail_tab_button("Timeline", active == DetailTab::Timeline, pane);
    let graph = detail_tab_button("Graph Editor", active == DetailTab::GraphEditor, pane);
    container(
        row![text("DETAIL").size(dims.theme().text.caption), timeline, graph]
            .spacing(dims.theme().space.xs)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding([0.0, dims.theme().space.s])
    .width(Length::Fill)
    .height(Length::Fixed(dims.theme().size.panel_header))
    .style(move |_theme| container::Style {
        background: Some(Background::Color(colors.surface_panel)),
        ..container::Style::default()
    })
    .into()
}

fn detail_tab_button(label: &'static str, active: bool, pane: &TimelinePane) -> Element<'static, Message> {
    let tab = button(text(label).size(pane.dims.theme().text.caption))
        .padding([0.0, pane.dims.theme().space.s]);
    if active {
        tab.into()
    } else {
        tab.on_press(Message::ToggleGraphEditor).into()
    }
}

/// Graph Editor の内容。タイトルと閉じる操作は [`detail_header`] が持つため、
/// ここには別の Hide 入口を置かない。
pub(crate) fn view(pane: &TimelinePane) -> Element<'static, Message> {
    let dims: Dimensions = pane.dims;
    let colors = pane.colors;
    let Some(projection) = pane.graph_editor.as_ref() else {
        return container(text("Graph Editor — select a keyframe")).into();
    };
    let values = projection.controls;
    let graph = canvas_widget(GraphCanvas {
        values,
        colors,
        layout: dims.components.timeline,
    })
        .width(Length::Fill)
        .height(Length::Fixed(dims.graph_editor_plot_height));
    let mut controls = Vec::with_capacity(GraphControl::ALL.len());
    for control in GraphControl::ALL {
        let index = control.index();
        let displayed = pane.graph_drafts[index]
            .clone()
            .unwrap_or_else(|| format!("{:.3}", values[index]));
        controls.push(
            row![
                text(control.label()).width(Length::Fixed(dims.graph_control_label_width)),
                text_input("0.000", displayed)
                    .on_input(move |input| Message::GraphControlInput(control, input))
                    .on_submit(Message::GraphCommit)
                    .width(Length::Fixed(dims.inspector_value_width))
                    .padding([0.0, dims.theme().space.xs]),
            ]
            .spacing(dims.theme().space.xs)
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
        text(state_label).size(dims.theme().text.caption).color(colors.text_muted),
        graph,
        row(controls).spacing(dims.theme().space.m),
        text("Drag the handles or enter x1/y1/x2/y2, then press Enter.")
            .size(dims.theme().text.caption)
            .color(colors.text_muted),
    ])
    .padding([dims.theme().space.s, dims.theme().space.m])
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(colors.surface_panel)),
        ..container::Style::default()
    })
    .into()
}

struct GraphCanvas {
    values: [f32; 4],
    colors: Colors,
    layout: TimelineValues,
}

#[derive(Default)]
struct GraphCanvasState {
    active: Option<GraphControl>,
}

impl GraphCanvas {
    fn plot_size(bounds: Rectangle, padding: f32) -> (f32, f32) {
        (
            (bounds.width - padding * 2.0).max(1.0),
            (bounds.height - padding * 2.0).max(1.0),
        )
    }

    fn point(
        bounds: Rectangle,
        controls: [f32; 4],
        control: GraphControl,
        padding: f32,
    ) -> Point {
        let (width, height) = Self::plot_size(bounds, padding);
        let (x, y) = match control {
            GraphControl::X1 | GraphControl::Y1 => (controls[0], controls[1]),
            GraphControl::X2 | GraphControl::Y2 => (controls[2], controls[3]),
        };
        Point::new(padding + x * width, padding + (1.0 - y) * height)
    }

    fn normalized(bounds: Rectangle, position: Point, padding: f32) -> (f32, f32) {
        let (width, height) = Self::plot_size(bounds, padding);
        (
            ((position.x - padding) / width).clamp(0.0, 1.0),
            (1.0 - (position.y - padding) / height).clamp(0.0, 1.0),
        )
    }

    fn hit(
        &self,
        bounds: Rectangle,
        values: [f32; 4],
        position: Point,
    ) -> Option<GraphControl> {
        GraphControl::ALL.into_iter().find(|&control| {
            Self::point(bounds, values, control, self.layout.graph_plot_padding)
                .distance(position)
                <= self.layout.graph_handle_hit
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
                let control = self.hit(bounds, self.values, position)?;
                state.active = Some(control);
                Some(canvas::Action::publish(Message::GraphHandleGrabbed(control)).and_capture())
            }
            iced::mouse::Event::CursorMoved { .. } => {
                let control = state.active?;
                let position = cursor.position_in(bounds)?;
                let (x, y) = Self::normalized(bounds, position, self.layout.graph_plot_padding);
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
        let padding = self.layout.graph_plot_padding;
        let (width, height) = Self::plot_size(bounds, padding);
        let origin = Point::new(padding, padding + height);
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(bounds.width, bounds.height),
            self.colors.surface_app,
        );
        for fraction in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let x = padding + fraction * width;
            let y = padding + (1.0 - fraction) * height;
            frame.stroke(
                &canvas::Path::line(
                    Point::new(x, padding),
                    Point::new(x, padding + height),
                ),
                canvas::Stroke::default()
                    .with_color(self.colors.timeline_grid_minor)
                    .with_width(self.layout.graph_grid_line_width),
            );
            frame.stroke(
                &canvas::Path::line(
                    Point::new(padding, y),
                    Point::new(padding + width, y),
                ),
                canvas::Stroke::default()
                    .with_color(self.colors.timeline_grid_minor)
                    .with_width(self.layout.graph_grid_line_width),
            );
        }
        let curve = canvas::Path::new(|builder| {
            for index in 0..=32 {
                let t = index as f32 / 32.0;
                let (x, y) = bezier_point(self.values, t);
                let point = Point::new(padding + x * width, padding + (1.0 - y) * height);
                if index == 0 {
                    builder.move_to(point);
                } else {
                    builder.line_to(point);
                }
            }
        });
        frame.stroke(
            &curve,
            canvas::Stroke::default()
                .with_color(self.colors.action_active)
                .with_width(self.layout.graph_curve_line_width),
        );
        let first = Self::point(bounds, self.values, GraphControl::X1, padding);
        let second = Self::point(bounds, self.values, GraphControl::X2, padding);
        frame.stroke(
            &canvas::Path::line(origin, first),
            canvas::Stroke::default()
                .with_color(self.colors.text_muted)
                .with_width(self.layout.graph_control_line_width),
        );
        frame.stroke(
            &canvas::Path::line(Point::new(padding + width, padding), second),
            canvas::Stroke::default()
                .with_color(self.colors.text_muted)
                .with_width(self.layout.graph_control_line_width),
        );
        for point in [first, second] {
            frame.fill(
                &canvas::Path::circle(point, self.layout.graph_handle_radius),
                self.colors.action_active,
            );
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

    #[test]
    fn detail_tab_keeps_timeline_body_visible() {
        assert_eq!(detail_tab(false), DetailTab::Timeline);
        assert_eq!(detail_tab(true), DetailTab::GraphEditor);
    }
}
