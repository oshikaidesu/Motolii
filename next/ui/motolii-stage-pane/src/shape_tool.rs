//! Stage の shape tool 部品。
//!
//! この部品は工具の選択、Stage 上の座標変換、ドラッグ/ペンの一時状態だけを持つ。
//! Document は知らず、確定した意味は [`Message`] として親へ返す。`gizmo`/`marquee`
//! と同じ「canvas は gesture の翻訳、書き込みは Shell」の境界である。

use glam::{Affine2, Vec2};
use iced::widget::{button, canvas, row, text};
use iced::{keyboard, mouse, Background, Border, Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_tokens_rs::{Colors, Dimensions};

use crate::observation_as_resolved;

/* motolii-component
id = "stage.shape_tool"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["ShapeTool", "ShapeToolOverlay"]
meaning = ["Select", "Create", "CreatePen"]
evaluation = ["comp_from_screen", "screen_from_comp"]
render = ["toolbar", "preview"]
observable = ["shape_tool_draws_shape"]
*/

/// Stage 上で選べる工具。`Select` は既存の marquee/gizmo へイベントを素通しする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShapeTool {
    #[default]
    Select,
    Rectangle,
    Ellipse,
    Pen,
}

impl ShapeTool {
    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "選択",
            Self::Rectangle => "矩形",
            Self::Ellipse => "楕円",
            Self::Pen => "ペン",
        }
    }

    fn draws_drag(self) -> bool {
        matches!(self, Self::Rectangle | Self::Ellipse)
    }
}

/// shape tool の選択/確定事象。親は `Create`/`CreatePen` を Document へ写す。
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Select(ShapeTool),
    Create {
        tool: ShapeTool,
        start: [f32; 2],
        end: [f32; 2],
    },
    CreatePen { points: Vec<[f32; 2]> },
    Cancel,
}

/// Stage 上縁へ置く工具列。選択状態も同じ `ShapeTool` を読むため二重管理しない。
pub fn toolbar(tool: ShapeTool, dims: Dimensions, colors: Colors) -> iced::Element<'static, Message> {
    row([
        tool_button(ShapeTool::Select, tool, dims, colors),
        tool_button(ShapeTool::Rectangle, tool, dims, colors),
        tool_button(ShapeTool::Ellipse, tool, dims, colors),
        tool_button(ShapeTool::Pen, tool, dims, colors),
    ])
    .spacing(dims.spacing_xs)
    .padding([dims.spacing_xs, dims.spacing_m])
    .into()
}

fn tool_button(
    value: ShapeTool,
    current: ShapeTool,
    dims: Dimensions,
    colors: Colors,
) -> iced::Element<'static, Message> {
    let active = value == current;
    button(text(value.label()).size(dims.body_text))
        .on_press(Message::Select(value))
        .padding([dims.spacing_xs, dims.spacing_m])
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let background = if active {
                Some(Background::Color(colors.state_selected))
            } else if hovered {
                Some(Background::Color(colors.surface_hover))
            } else {
                None
            };
            button::Style {
                background,
                text_color: if active {
                    colors.text_primary
                } else if hovered {
                    colors.text_secondary
                } else {
                    colors.text_muted
                },
                border: Border::default(),
                ..button::Style::default()
            }
        })
        .into()
}

/// Shape tool の canvas overlay。`Select` 中は一切 capture しないので既存の
/// marquee/gizmo/観測カメラの責任を奪わない。
#[derive(Clone, Copy)]
pub struct ShapeToolOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    tool: ShapeTool,
    dims: Dimensions,
    colors: Colors,
}

impl ShapeToolOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        tool: ShapeTool,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            tool,
            dims,
            colors,
        }
    }

    pub fn view<'a>(self) -> iced::Element<'a, Message> {
        canvas(self)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }

    fn camera(&self) -> ResolvedCamera {
        self.observation
            .map(observation_as_resolved)
            .unwrap_or(self.render_camera)
    }

    fn screen_from_comp(&self, bounds: Rectangle) -> Option<Affine2> {
        let local_bounds = Rectangle::new(Point::ORIGIN, bounds.size());
        let rect = crate::letterboxed_rect(local_bounds, self.comp)?;
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return None;
        }
        let letterbox = Affine2::from_translation(Vec2::new(rect.x, rect.y))
            * Affine2::from_scale(Vec2::new(
                rect.width / self.comp.width as f32,
                rect.height / self.comp.height as f32,
            ));
        let affine = letterbox * camera_screen_from_world_z0(self.comp, self.camera());
        affine.is_finite().then_some(affine)
    }

    fn comp_from_screen(&self, bounds: Rectangle, point: Point) -> Option<[f32; 2]> {
        let screen_from_comp = self.screen_from_comp(bounds)?;
        let determinant = screen_from_comp.matrix2.determinant();
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let comp_point = screen_from_comp
            .inverse()
            .transform_point2(Vec2::new(point.x, point.y));
        comp_point.is_finite().then_some([comp_point.x, comp_point.y])
    }
}

#[derive(Default)]
pub struct Interaction {
    tool: ShapeTool,
    drag_start: Option<[f32; 2]>,
    drag_current: Option<[f32; 2]>,
    pen_points: Vec<[f32; 2]>,
}

impl canvas::Program<Message> for ShapeToolOverlay {
    type State = Interaction;

    fn update(
        &self,
        state: &mut Interaction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if state.tool != self.tool {
            state.tool = self.tool;
            state.drag_start = None;
            state.drag_current = None;
            state.pen_points.clear();
        }

        match event {
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                let had_pending = state.drag_start.take().is_some() || !state.pen_points.is_empty();
                state.drag_current = None;
                state.pen_points.clear();
                had_pending.then(|| canvas::Action::publish(Message::Cancel).and_capture())
            }
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Enter),
                ..
            }) if self.tool == ShapeTool::Pen => {
                if state.pen_points.len() < 2 {
                    state.pen_points.clear();
                    return Some(canvas::Action::request_redraw().and_capture());
                }
                let points = std::mem::take(&mut state.pen_points);
                Some(canvas::Action::publish(Message::CreatePen { points }).and_capture())
            }
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    let position = cursor.position_in(bounds)?;
                    let point = self.comp_from_screen(bounds, position)?;
                    match self.tool {
                        ShapeTool::Rectangle | ShapeTool::Ellipse => {
                            state.drag_start = Some(point);
                            state.drag_current = Some(point);
                            Some(canvas::Action::capture())
                        }
                        ShapeTool::Pen => {
                            state.pen_points.push(point);
                            Some(canvas::Action::request_redraw().and_capture())
                        }
                        ShapeTool::Select => None,
                    }
                }
                mouse::Event::CursorMoved { .. } if self.tool.draws_drag() => {
                    state.drag_start.as_ref()?;
                    let absolute = cursor.position()?;
                    state.drag_current = self.comp_from_screen(
                        bounds,
                        Point::new(absolute.x - bounds.x, absolute.y - bounds.y),
                    );
                    Some(canvas::Action::request_redraw().and_capture())
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) if self.tool.draws_drag() => {
                    let start = state.drag_start.take()?;
                    state.drag_current = None;
                    let absolute = cursor.position()?;
                    let end = self.comp_from_screen(
                        bounds,
                        Point::new(absolute.x - bounds.x, absolute.y - bounds.y),
                    )?;
                    Some(
                        canvas::Action::publish(Message::Create {
                            tool: self.tool,
                            start,
                            end,
                        })
                        .and_capture(),
                    )
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) if self.tool == ShapeTool::Pen => {
                    Some(canvas::Action::request_redraw().and_capture())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Interaction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let Some(screen_from_comp) = self.screen_from_comp(bounds) else {
            return Vec::new();
        };
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let accent = self.colors.action_active;

        if let Some(start) = state.drag_start {
            if let Some(current) = state.drag_current {
                let start = screen_from_comp.transform_point2(Vec2::new(start[0], start[1]));
                let current = screen_from_comp.transform_point2(Vec2::new(current[0], current[1]));
                let rect = Rectangle {
                    x: start.x.min(current.x),
                    y: start.y.min(current.y),
                    width: (start.x - current.x).abs(),
                    height: (start.y - current.y).abs(),
                };
                let path = canvas::Path::rectangle(
                    Point::new(rect.x, rect.y),
                    iced::Size::new(rect.width, rect.height),
                );
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(accent)
                        .with_width(self.dims.border_width),
                );
            }
        }

        if state.pen_points.len() >= 2 {
            let path = canvas::Path::new(|builder| {
                let first = screen_from_comp.transform_point2(Vec2::new(
                    state.pen_points[0][0],
                    state.pen_points[0][1],
                ));
                builder.move_to(Point::new(first.x, first.y));
                for point in &state.pen_points[1..] {
                    let point = screen_from_comp.transform_point2(Vec2::new(point[0], point[1]));
                    builder.line_to(Point::new(point.x, point.y));
                }
            });
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(accent)
                    .with_width(self.dims.border_width),
            );
            for point in &state.pen_points {
                let point = screen_from_comp.transform_point2(Vec2::new(point[0], point[1]));
                let marker = canvas::Path::circle(
                    Point::new(point.x, point.y),
                    self.dims.gizmo_handle_size * 0.3,
                );
                frame.fill(&marker, accent);
            }
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Interaction,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.tool.draws_drag() || self.tool == ShapeTool::Pen || state.drag_start.is_some() {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_tool_draws_shape() {
        assert_eq!(ShapeTool::Rectangle.label(), "矩形");
        assert_eq!(ShapeTool::Ellipse.label(), "楕円");
        assert_eq!(ShapeTool::Pen.label(), "ペン");
    }

    #[test]
    fn shape_tools_are_distinct_and_only_draw_tools_capture_drag() {
        assert!(!ShapeTool::Select.draws_drag());
        assert!(ShapeTool::Rectangle.draws_drag());
        assert!(ShapeTool::Ellipse.draws_drag());
        assert!(!ShapeTool::Pen.draws_drag());
    }
}
