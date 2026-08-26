//! Stage 上の Bezier path 編集部品。
//!
//! `ShapeTool` が新しい shape を作るのに対し、ここは選択中の Bezier shape の
//! 既存頂点を動かす。Document は持たず、screen/comp の投影と drag の翻訳だけを
//! 担当する。書き込みは親の Shell が `SetShapes` 一回へ畳む。

use glam::{Affine2, Vec2};
use iced::widget::{button, canvas, row, text};
use iced::{keyboard, mouse, Background, Border, Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_store::{LayerId, PathSource, ShapeNode};
use motolii_tokens_rs::{Colors, Dimensions};

use crate::observation_as_resolved;

/* motolii-component
id = "stage.path_edit"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["PathEditTarget", "PathEditOverlay"]
meaning = ["MoveVertex", "ClosePath", "OpenPath"]
evaluation = ["comp_from_screen", "layer_from_comp"]
render = ["toolbar", "draw"]
observable = ["path_vertex_drag_changes_shape"]
*/

/// Stage が扱う一頂点の参照。shape/group の構造を UI 側で複製せず、書き戻しに
/// 必要な index だけを保持する。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexRef {
    pub shape_index: usize,
    pub contour_index: usize,
    pub vertex_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct VertexHandle {
    reference: VertexRef,
    local_point: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContourRef {
    shape_index: usize,
    contour_index: usize,
    closed: bool,
}

/// 選択中 layer の Bezier leaf から作る、描画用の使い捨て投影。
#[derive(Clone, Debug, PartialEq)]
pub struct PathEditTarget {
    layer: LayerId,
    layer_to_comp: Affine2,
    vertices: Vec<VertexHandle>,
    contours: Vec<ContourRef>,
}

impl PathEditTarget {
    /// group はまだ path vertex の直接編集対象にしない。親子変換を UI 側で再実装
    /// せず、flat な `ShapeNode::Leaf` だけを安全に編集可能として投影する。
    pub fn from_shapes(layer: LayerId, shapes: &[ShapeNode], layer_to_comp: Affine2) -> Self {
        let mut vertices = Vec::new();
        let mut contours = Vec::new();
        for (shape_index, node) in shapes.iter().enumerate() {
            let ShapeNode::Leaf(shape) = node else {
                continue;
            };
            let PathSource::Bezier(path) = &shape.source else {
                continue;
            };
            for (contour_index, contour) in path.iter().enumerate() {
                contours.push(ContourRef {
                    shape_index,
                    contour_index,
                    closed: contour.closed,
                });
                for (vertex_index, vertex) in contour.vertices.iter().enumerate() {
                    let local_point = [vertex.point.x as f32, vertex.point.y as f32];
                    if local_point.iter().all(|value| value.is_finite()) {
                        vertices.push(VertexHandle {
                            reference: VertexRef {
                                shape_index,
                                contour_index,
                                vertex_index,
                            },
                            local_point,
                        });
                    }
                }
            }
        }
        Self {
            layer,
            layer_to_comp,
            vertices,
            contours,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn layer(&self) -> LayerId {
        self.layer
    }

    fn comp_point(&self, handle: VertexHandle) -> Option<Vec2> {
        let point = self
            .layer_to_comp
            .transform_point2(Vec2::new(handle.local_point[0], handle.local_point[1]));
        point.is_finite().then_some(point)
    }

    fn primary_contour(&self) -> Option<ContourRef> {
        self.contours.first().copied()
    }
}

/// 頂点編集の確定事象。`point` は layer-local 座標で、Shell がそのまま
/// `motolii_vector::Point` へ戻す。
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    MoveVertex {
        layer: LayerId,
        target: VertexRef,
        point: [f64; 2],
    },
    ClosePath {
        layer: LayerId,
        shape_index: usize,
        contour_index: usize,
    },
    OpenPath {
        layer: LayerId,
        shape_index: usize,
        contour_index: usize,
    },
    Cancel,
}

/// 現在の先頭 contour を操作する toolbar。構造が複数 contour の時も、頂点は
/// 全て見せるが、開閉は先頭 contour に限定し、対象を曖昧にしない。
pub fn toolbar(
    target: &PathEditTarget,
    dims: Dimensions,
    colors: Colors,
) -> Option<iced::Element<'static, Message>> {
    let contour = target.primary_contour()?;
    let (label, message) = if contour.closed {
        (
            "パスを開く",
            Message::OpenPath {
                layer: target.layer,
                shape_index: contour.shape_index,
                contour_index: contour.contour_index,
            },
        )
    } else {
        (
            "パスを閉じる",
            Message::ClosePath {
                layer: target.layer,
                shape_index: contour.shape_index,
                contour_index: contour.contour_index,
            },
        )
    };
    Some(
        row![
            button(text(label).size(dims.theme().text.body))
                .on_press(message)
                .padding([dims.theme().space.xs, dims.theme().space.m])
                .style(move |_theme, status| {
                    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
                    button::Style {
                        background: hovered.then_some(Background::Color(colors.surface_hover)),
                        text_color: colors.text_secondary,
                        border: Border::default(),
                        ..button::Style::default()
                    }
                })
        ]
        .spacing(dims.theme().space.xs)
        .padding([dims.theme().space.xs, dims.theme().space.m])
        .into(),
    )
}

/// Bezier 頂点の hit-test と preview 描画を持つ canvas overlay。
#[derive(Clone)]
pub struct PathEditOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    target: PathEditTarget,
    enabled: bool,
    dims: Dimensions,
    colors: Colors,
}

impl PathEditOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        target: PathEditTarget,
        enabled: bool,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        Self {
            comp,
            render_camera,
            observation,
            target,
            enabled,
            dims,
            colors,
        }
    }

    pub fn target(&self) -> &PathEditTarget {
        &self.target
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

    fn comp_from_screen(&self, bounds: Rectangle, point: Point) -> Option<Vec2> {
        let screen_from_comp = self.screen_from_comp(bounds)?;
        let determinant = screen_from_comp.matrix2.determinant();
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let comp_point = screen_from_comp
            .inverse()
            .transform_point2(Vec2::new(point.x, point.y));
        comp_point.is_finite().then_some(comp_point)
    }

    fn layer_from_comp(&self, point: Vec2) -> Option<Vec2> {
        let determinant = self.target.layer_to_comp.matrix2.determinant();
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let local = self
            .target
            .layer_to_comp
            .inverse()
            .transform_point2(point);
        local.is_finite().then_some(local)
    }

    fn nearest_vertex(&self, bounds: Rectangle, screen: Point) -> Option<usize> {
        let screen_from_comp = self.screen_from_comp(bounds)?;
        let pointer = Vec2::new(screen.x, screen.y);
        self.target
            .vertices
            .iter()
            .enumerate()
            .filter_map(|(index, handle)| {
                let point = self.target.comp_point(*handle)?;
                let screen_point = screen_from_comp.transform_point2(point);
                let distance = screen_point.distance(pointer);
                (distance <= self.dims.gizmo_hit_radius).then_some((index, distance))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
    }
}

#[derive(Default)]
pub struct Interaction {
    active_vertex: Option<usize>,
    preview_comp: Option<Vec2>,
}

impl canvas::Program<Message> for PathEditOverlay {
    type State = Interaction;

    fn update(
        &self,
        state: &mut Interaction,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if !self.enabled {
            state.active_vertex = None;
            state.preview_comp = None;
            return None;
        }
        match event {
            canvas::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) if state.active_vertex.take().is_some() => {
                state.preview_comp = None;
                Some(canvas::Action::publish(Message::Cancel).and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let point = cursor.position_in(bounds)?;
                let vertex = self.nearest_vertex(bounds, point)?;
                state.active_vertex = Some(vertex);
                state.preview_comp = self.comp_from_screen(bounds, point);
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. })
                if state.active_vertex.is_some() =>
            {
                let point = cursor.position_in(bounds)?;
                state.preview_comp = self.comp_from_screen(bounds, point);
                Some(canvas::Action::request_redraw().and_capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let vertex_index = state.active_vertex.take()?;
                let point = cursor.position_in(bounds)?;
                let comp = self.comp_from_screen(bounds, point)?;
                state.preview_comp = None;
                let handle = self.target.vertices.get(vertex_index)?;
                let local = self.layer_from_comp(comp)?;
                Some(
                    canvas::Action::publish(Message::MoveVertex {
                        layer: self.target.layer,
                        target: handle.reference,
                        point: [f64::from(local.x), f64::from(local.y)],
                    })
                    .and_capture(),
                )
            }
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
        if !self.enabled {
            return Vec::new();
        }
        let Some(screen_from_comp) = self.screen_from_comp(bounds) else {
            return Vec::new();
        };
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let accent = self.colors.action_active;
        for (index, handle) in self.target.vertices.iter().enumerate() {
            let point = if state.active_vertex == Some(index) {
                state
                    .preview_comp
                    .or_else(|| self.target.comp_point(*handle))
            } else {
                self.target.comp_point(*handle)
            };
            let Some(point) = point else { continue };
            let point = screen_from_comp.transform_point2(point);
            let marker = canvas::Path::circle(
                Point::new(point.x, point.y),
                self.dims.gizmo_handle_size * 0.3,
            );
            if state.active_vertex == Some(index) {
                frame.fill(&marker, accent);
            } else {
                frame.fill(&marker, self.colors.surface_app);
                frame.stroke(
                    &marker,
                    canvas::Stroke::default()
                        .with_color(accent)
                        .with_width(self.dims.theme().stroke.hairline),
                );
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
        if self.enabled && state.active_vertex.is_some() {
            mouse::Interaction::Grabbing
        } else if self.enabled {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::Shape;
    use motolii_vector::{Contour, Point as VectorPoint};

    #[test]
    fn path_target_projects_bezier_vertices_without_projecting_primitives_or_groups() {
        let layer = LayerId(7);
        let path = vec![Contour::closed([
            VectorPoint { x: 10.0, y: 20.0 },
            VectorPoint { x: 30.0, y: 20.0 },
        ])];
        let shapes = vec![
            ShapeNode::Leaf(Shape::new(PathSource::Bezier(path))),
            ShapeNode::Leaf(Shape::new(PathSource::Rectangle {
                size: VectorPoint { x: 10.0, y: 10.0 },
            })),
        ];
        let target = PathEditTarget::from_shapes(layer, &shapes, Affine2::IDENTITY);
        assert_eq!(target.layer(), layer);
        assert_eq!(target.vertices.len(), 2);
        assert_eq!(target.contours.len(), 1);
        assert!(target.primary_contour().is_some_and(|contour| contour.closed));
    }

    #[test]
    fn path_edit_does_not_inverse_a_singular_layer_transform() {
        let target = PathEditTarget::from_shapes(
            LayerId(1),
            &[],
            Affine2::from_scale(Vec2::ZERO),
        );
        let overlay = PathEditOverlay::new(
            CompSpec { width: 100, height: 100 },
            ResolvedCamera::default(),
            None,
            target,
            true,
            Dimensions::default(),
            Colors::default(),
        );
        assert!(overlay.layer_from_comp(Vec2::new(1.0, 1.0)).is_none());
    }
}
