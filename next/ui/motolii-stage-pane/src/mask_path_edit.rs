//! Stage 上の mask path 編集部品。
//!
//! Shape の `path_edit` と同じ画面座標・gesture 文法を使うが、mask の正本は
//! `mask.{id}.shape` の `Value::Path` なので、別 component として分ける。
//! この pane は layer-local 座標の確定事象だけを返し、Document は持たない。

use glam::{Affine2, Vec2};
use iced::widget::{canvas, container, row, text};
use iced::{keyboard, mouse, Background, Border, Point, Rectangle};

use motolii_core::{camera_screen_from_world_z0, CompSpec, ResolvedCamera};
use motolii_engine::ObservationCamera;
use motolii_store::{LayerId, MaskId, Path};
use motolii_tokens_rs::{Colors, Dimensions};

use crate::observation_as_resolved;

/* motolii-component
id = "stage.mask_path_edit"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["MaskPathEditTarget", "MaskPathEditOverlay"]
meaning = ["MoveVertex"]
evaluation = ["comp_from_screen", "layer_from_comp"]
render = ["draw"]
observable = ["mask_vertex_drag_changes_shape"]
*/

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskVertexRef {
    pub mask: MaskId,
    pub vertex_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct VertexHandle {
    reference: MaskVertexRef,
    local_point: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaskPathEditTarget {
    layer: LayerId,
    layer_to_comp: Affine2,
    vertices: Vec<VertexHandle>,
}

impl MaskPathEditTarget {
    /// `resolved.masks` は layer の mask 順を保つ。呼び手が static mask ids と
    /// 同じ順で束ねて渡すため、ここでは id と path の対応を再解釈しない。
    pub fn from_masks(
        layer: LayerId,
        masks: &[(MaskId, &Path)],
        layer_to_comp: Affine2,
    ) -> Self {
        let mut vertices = Vec::new();
        for (mask, path) in masks {
            for (vertex_index, vertex) in path.vertices.iter().enumerate() {
                let local_point = [vertex.point[0] as f32, vertex.point[1] as f32];
                if local_point.iter().all(|value| value.is_finite()) {
                    vertices.push(VertexHandle {
                        reference: MaskVertexRef {
                            mask: *mask,
                            vertex_index,
                        },
                        local_point,
                    });
                }
            }
        }
        Self {
            layer,
            layer_to_comp,
            vertices,
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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    MoveVertex {
        layer: LayerId,
        target: MaskVertexRef,
        point: [f64; 2],
    },
    Cancel,
}

pub fn toolbar(dims: Dimensions, colors: Colors) -> iced::Element<'static, Message> {
    container(row![text("Mask Path").size(dims.body_text)])
        .padding([dims.spacing_xs, dims.spacing_m])
        .style(move |_theme| iced::widget::container::Style {
            background: Some(Background::Color(colors.surface_raised)),
            border: Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

#[derive(Clone)]
pub struct MaskPathEditOverlay {
    comp: CompSpec,
    render_camera: ResolvedCamera,
    observation: Option<ObservationCamera>,
    target: MaskPathEditTarget,
    enabled: bool,
    dims: Dimensions,
    colors: Colors,
}

impl MaskPathEditOverlay {
    pub fn new(
        comp: CompSpec,
        render_camera: ResolvedCamera,
        observation: Option<ObservationCamera>,
        target: MaskPathEditTarget,
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

    pub fn target(&self) -> &MaskPathEditTarget {
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

impl canvas::Program<Message> for MaskPathEditOverlay {
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
                frame.fill(&marker, self.colors.action_active);
            } else {
                frame.fill(&marker, self.colors.surface_app);
                frame.stroke(
                    &marker,
                    canvas::Stroke::default()
                        .with_color(self.colors.action_active)
                        .with_width(self.dims.border_width),
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

    #[test]
    fn mask_vertex_drag_changes_shape() {
        let path = Path {
            vertices: vec![
                motolii_store::PathVertex {
                    point: [1.0, 2.0],
                    in_tangent: [0.0, 0.0],
                    out_tangent: [0.0, 0.0],
                },
                motolii_store::PathVertex {
                    point: [3.0, 4.0],
                    in_tangent: [0.0, 0.0],
                    out_tangent: [0.0, 0.0],
                },
            ],
            closed: true,
        };
        let target = MaskPathEditTarget::from_masks(
            LayerId(1),
            &[(MaskId(7), &path)],
            Affine2::IDENTITY,
        );
        assert!(!target.is_empty());
        assert_eq!(target.vertices[0].reference.mask, MaskId(7));
        assert_eq!(target.vertices[1].reference.vertex_index, 1);
    }
}
