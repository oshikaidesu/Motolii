use std::{sync::Arc, time::Instant};

use egui::{
    Align2, Color32, Event, FontId, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect,
    TouchPhase, Vec2,
    epaint::{Mesh, Vertex as EguiVertex},
};
use glam::EulerRot;
use lyon_path::{Path as LyonPath, math::point};
use lyon_tessellation::{
    FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
    geometry_builder::{VertexBuffers, simple_builder},
};
use motolii_doc::{
    CompositeOrder, LineJoin, PointType, TrimMode,
    pathgeom::{self, Contour, Path, Point, ResolvedPathOp, ResolvedTransform, Vertex},
};
use motolii_ui::AppStageTransformEdit;
use re_chunk::Chunk;
use re_log_types::TimePoint;
use re_sdk_types::archetypes::Mesh3D;
use transform_gizmo::{
    Gizmo, GizmoConfig, GizmoInteraction, GizmoMode, GizmoOrientation, GizmoResult,
    math::{DMat4, DQuat, DVec3, Pos2 as GizmoPos2, Rect as GizmoRect, Transform},
};

use crate::host_bridge::HostStageGeometry;
use crate::renderer_core::{PointerPhase, StagePointerButton};

const FIXTURE_RECT_FILL_COLOR: u32 = 0xE9_8C_6AFF;
const FIXTURE_RECT_STROKE_COLOR: u32 = 0xEC_D8_FFFF;
const DOCUMENT_RECT_FILL_COLOR: u32 = 0xFFFF_FFFF;
const STAGE_HOST_ERASE_COLOR: u32 = 0x0000_0000;
const DOCUMENT_FRAME_ENTITY: &str = "motolii/document/frame";

/// Owns only the adapter from the native Stage surface to Rerun's Spatial View.
///
/// Product chrome, persistence, and Document projection stay outside this adapter.
pub(crate) struct EmbeddedSpatialStage {
    egui_ctx: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    spatial_stage: re_view_spatial::SpatialStage,
    input_events: Vec<Event>,
    input_modifiers: Modifiers,
    started_at: Instant,
    gizmo: Gizmo,
    /// Host `stage_geometry` 適用済み。
    host_geometry_active: bool,
    host_layer_ids: Vec<String>,
    /// 直近の host 投影（move preview の復元元）。
    host_geometry: Option<HostStageGeometry>,
    /// primary 選択（outline用）。
    host_primary_layer_id: Option<String>,
    /// move drag 中の world delta preview（対象 layer のみ）。
    move_preview: Option<(String, [f64; 2])>,
    gizmo_gesture: Option<(String, AppStageTransformEdit)>,
    pending_gizmo_action: Option<StageGizmoAction>,
    gizmo_cancel_requested: bool,
    gizmo_pointer_position: Pos2,
    gizmo_pointer_down: bool,
    gizmo_pointer_pressed: bool,
    gizmo_pointer_released: bool,
    feedback: Option<(String, bool)>,
    /// 評価済み Document frame を Image visualizer へ載せているか。
    evaluated_frame_active: bool,
    host_viewport: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum StageGizmoAction {
    Preview {
        layer_id: String,
        edit: AppStageTransformEdit,
    },
    Commit {
        layer_id: String,
        edit: AppStageTransformEdit,
    },
    Cancel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StageTransformProjection {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) rotation_x: f64,
    pub(crate) rotation_y: f64,
    pub(crate) rotation_z: f64,
}

impl EmbeddedSpatialStage {
    pub(crate) fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let mut egui_renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );
        let render_ctx = re_renderer::RenderContext::new(
            adapter,
            device.clone(),
            queue.clone(),
            surface_format,
            re_renderer::RenderConfig::best_for_device_caps,
        )
        .map_err(|error| format!("create Rerun render context: {error}"))?;
        egui_renderer.callback_resources.insert(render_ctx);

        let stage = Self {
            egui_ctx: egui::Context::default(),
            egui_renderer,
            spatial_stage: re_view_spatial::SpatialStage::new(re_log_types::ApplicationId::from(
                "motolii-rn-stage",
            ))
            .map_err(|error| format!("create Rerun spatial stage: {error}"))?,
            input_events: Vec::new(),
            input_modifiers: Modifiers::NONE,
            started_at: Instant::now(),
            gizmo: Gizmo::default(),
            host_geometry_active: false,
            host_layer_ids: Vec::new(),
            host_geometry: None,
            host_primary_layer_id: None,
            move_preview: None,
            gizmo_gesture: None,
            pending_gizmo_action: None,
            gizmo_cancel_requested: false,
            gizmo_pointer_position: Pos2::ZERO,
            gizmo_pointer_down: false,
            gizmo_pointer_pressed: false,
            gizmo_pointer_released: false,
            feedback: None,
            evaluated_frame_active: false,
            host_viewport: None,
        };
        // 製品マウントでは fixture rect を出さない。host snapshot / 評価フレームが正本。
        Ok(stage)
    }

    pub(crate) fn pointer(
        &mut self,
        phase: PointerPhase,
        button: StagePointerButton,
        modifiers: u32,
        x: f64,
        y: f64,
    ) {
        let position = Pos2::new(x as f32, y as f32);
        let modifiers = stage_modifiers(modifiers);
        self.input_modifiers = modifiers;
        self.input_events.push(Event::PointerMoved(position));

        match phase {
            PointerPhase::Down => self.input_events.push(Event::PointerButton {
                pos: position,
                button: egui_pointer_button(button),
                pressed: true,
                modifiers,
            }),
            PointerPhase::Up => self.input_events.push(Event::PointerButton {
                pos: position,
                button: egui_pointer_button(button),
                pressed: false,
                modifiers,
            }),
            PointerPhase::Cancel => self.input_events.push(Event::PointerGone),
            PointerPhase::Move => {}
        }
    }

    pub(crate) fn gizmo_pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
        self.gizmo_pointer_position = Pos2::new(x as f32, y as f32);
        match phase {
            PointerPhase::Down => {
                self.gizmo_pointer_pressed = true;
                self.gizmo_pointer_down = true;
            }
            PointerPhase::Move => {}
            PointerPhase::Up => {
                self.gizmo_pointer_down = false;
                self.gizmo_pointer_released = true;
            }
            PointerPhase::Cancel => {
                self.gizmo_pointer_down = false;
                self.gizmo_cancel_requested = true;
            }
        }
    }

    pub(crate) fn scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        let modifiers = stage_modifiers(modifiers);
        let Some(events) =
            stage_navigation_events(delta_x, delta_y, magnification, modifiers, x, y)
        else {
            return false;
        };
        self.input_modifiers = modifiers;
        self.input_events.extend(events);
        true
    }

    pub(crate) fn gizmo_wants_pointer(&self, x: f64, y: f64) -> bool {
        self.selected_gizmo_transform().is_some() && self.gizmo.pick_preview((x as f32, y as f32))
    }

    pub(crate) fn take_gizmo_action(&mut self) -> Option<StageGizmoAction> {
        self.pending_gizmo_action.take()
    }

    pub(crate) fn set_feedback(&mut self, message: impl Into<String>, rejected: bool) {
        self.feedback = Some((message.into(), rejected));
    }

    /// 選択中 Document layer の TRS。未選択は零。Inspector 正本ではない。
    pub(crate) fn transform_projection(&self) -> StageTransformProjection {
        let Some((_, transform)) = self.selected_gizmo_transform() else {
            return StageTransformProjection::default();
        };
        let (rotation_x, rotation_y, rotation_z) =
            DQuat::from(transform.rotation).to_euler(EulerRot::XYZ);
        let translation = DVec3::from(transform.translation);
        StageTransformProjection {
            x: translation.x,
            y: translation.y,
            z: translation.z,
            rotation_x: rotation_x.to_degrees(),
            rotation_y: rotation_y.to_degrees(),
            rotation_z: rotation_z.to_degrees(),
        }
    }

    /// RN props echo。Document 書きは gizmo → host_preview/commit。
    pub(crate) fn set_transform_projection(
        &mut self,
        projection: StageTransformProjection,
    ) -> bool {
        let _ = projection;
        true
    }

    pub(crate) fn set_created_item(&mut self, item_id: &str) -> bool {
        let _ = item_id;
        true
    }

    pub(crate) fn clear_host_projection(&mut self) -> bool {
        let _ = self.spatial_stage.clear_gpu_image(DOCUMENT_FRAME_ENTITY);
        self.evaluated_frame_active = false;
        self.host_viewport = None;
        if !self.host_geometry_active {
            return true;
        }
        let host_layers = std::mem::take(&mut self.host_layer_ids);
        for old_id in &host_layers {
            if !ingest_mesh(
                &mut self.spatial_stage,
                old_id,
                hidden_layer_mesh(),
                STAGE_HOST_ERASE_COLOR,
            ) {
                self.host_layer_ids = host_layers;
                return false;
            }
        }
        self.host_geometry_active = false;
        self.host_geometry = None;
        self.host_primary_layer_id = None;
        self.move_preview = None;
        self.gizmo_gesture = None;
        self.pending_gizmo_action = None;
        self.gizmo_cancel_requested = false;
        // host が空なら Stage も空。fixture を製品へ戻さない。
        true
    }

    pub(crate) fn set_host_primary_layer_id(&mut self, primary: Option<String>) {
        self.host_primary_layer_id = primary;
    }

    pub(crate) fn host_primary_layer_id(&self) -> Option<&str> {
        self.host_primary_layer_id.as_deref()
    }

    /// Host snapshot の stage_geometry を tessellate して載せる。
    /// `viewport_width/height` は host 投影の aspect 写像に使う（正方近似を避ける）。
    pub(crate) fn apply_host_stage_geometry(
        &mut self,
        geometry: &HostStageGeometry,
        viewport_width: u32,
        viewport_height: u32,
    ) -> bool {
        if viewport_width == 0 || viewport_height == 0 {
            return false;
        }

        let next_ids: Vec<String> = geometry
            .layers
            .iter()
            .flat_map(|layer| host_layer_mesh_paths(&layer.layer_id))
            .collect();
        for old_id in &self.host_layer_ids {
            if next_ids.iter().any(|id| id == old_id) {
                continue;
            }
            let _ = ingest_mesh(
                &mut self.spatial_stage,
                old_id,
                hidden_layer_mesh(),
                STAGE_HOST_ERASE_COLOR,
            );
        }

        let previewing = self.gizmo_gesture.is_some();
        for layer in &geometry.layers {
            if !ingest_host_layer_meshes(
                &mut self.spatial_stage,
                &layer.layer_id,
                layer.corners,
                !host_layer_fill_is_visible(layer.corners, self.evaluated_frame_active, previewing),
            ) {
                return false;
            }
        }
        self.host_layer_ids = next_ids;
        self.host_geometry = Some(geometry.clone());
        self.host_viewport = Some((viewport_width, viewport_height));
        self.host_geometry_active = true;
        self.move_preview = None;
        true
    }

    /// 対象 layer の corners だけを world delta で仮表示する。他 layer は不変。
    pub(crate) fn set_move_preview(
        &mut self,
        layer_id: &str,
        delta: [f64; 2],
        viewport_width: u32,
        viewport_height: u32,
    ) -> bool {
        if viewport_width == 0 || viewport_height == 0 {
            return false;
        }
        let Some(base) = self.host_geometry.clone() else {
            return false;
        };
        self.move_preview = Some((layer_id.to_owned(), delta));
        let preview = apply_move_preview_to_geometry(&base, self.move_preview.as_ref());
        let Some(layer) = preview
            .layers
            .iter()
            .find(|layer| layer.layer_id == layer_id)
        else {
            self.move_preview = None;
            return false;
        };
        if !ingest_host_layer_meshes(
            &mut self.spatial_stage,
            layer_id,
            layer.corners,
            !host_layer_fill_is_visible(layer.corners, self.evaluated_frame_active, true),
        ) {
            self.move_preview = None;
            return false;
        }
        true
    }

    pub(crate) fn clear_move_preview(&mut self, viewport_width: u32, viewport_height: u32) -> bool {
        let Some((_, _)) = self.move_preview.take() else {
            return true;
        };
        let Some(base) = self.host_geometry.clone() else {
            return true;
        };
        self.apply_host_stage_geometry(&base, viewport_width, viewport_height)
    }

    pub(crate) fn fit_view(&mut self, viewport_width: u32, viewport_height: u32) -> bool {
        if viewport_width == 0 || viewport_height == 0 {
            return false;
        }
        self.spatial_stage.reset_view();
        true
    }

    pub(crate) fn set_one_to_one(&mut self, viewport_width: u32, viewport_height: u32) -> bool {
        if viewport_width == 0 || viewport_height == 0 {
            return false;
        }
        // 100% は2D composition固有の倍率。Rerun標準cameraを迂回する固定Eyeへ戻さない。
        self.set_feedback("100% is not available with the standard Stage camera", true);
        false
    }

    pub(crate) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
        evaluated_frame: Option<&wgpu::Texture>,
    ) -> Result<Option<Option<String>>, String> {
        // Rerunのcallback command bufferより先にsurfaceを初期化する。
        // 後からClearすると、Rerunが同じsurfaceへ描いたMeshまで消してしまう。
        let mut clear_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii Rerun Spatial Stage clear"),
        });
        {
            let _clear_pass = clear_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Rerun Spatial Stage clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.035,
                            g: 0.041,
                            b: 0.050,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        queue.submit([clear_encoder.finish()]);

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(width as f32, height as f32),
            )),
            time: Some(self.started_at.elapsed().as_secs_f64()),
            modifiers: self.input_modifiers,
            events: std::mem::take(&mut self.input_events),
            ..Default::default()
        };

        let egui_ctx = self.egui_ctx.clone();
        let mut stage_error = None;
        let full_output = egui_ctx.run_ui(raw_input, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    let mut render_ctx = self
                        .egui_renderer
                        .callback_resources
                        .remove::<re_renderer::RenderContext>()
                        .expect("Rerun render context is registered for the native Stage");
                    if let Some(texture) = evaluated_frame {
                        self.present_evaluated_frame(&render_ctx, texture);
                    }
                    let result = self.spatial_stage.show(ui, &mut render_ctx);
                    self.egui_renderer.callback_resources.insert(render_ctx);
                    if let Err(error) = result {
                        stage_error = Some(error.to_string());
                    }
                    self.show_transform_gizmo(ui);
                    self.show_feedback(ui);
                });
        });
        if let Some(error) = stage_error {
            return Err(format!("run Rerun spatial stage: {error}"));
        }

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(device, queue, *id, delta);
        }
        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii Rerun Spatial Stage"),
        });
        let callback_command_buffers = self.egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Rerun Spatial Stage present"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }
        queue.submit(
            callback_command_buffers
                .into_iter()
                .chain([encoder.finish()]),
        );

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        Ok(self.spatial_stage.take_selected_entity_path())
    }

    fn present_evaluated_frame(
        &mut self,
        render_ctx: &re_renderer::RenderContext,
        texture: &wgpu::Texture,
    ) {
        if self
            .spatial_stage
            .copy_gpu_image(render_ctx, DOCUMENT_FRAME_ENTITY, texture)
            .is_err()
        {
            return;
        }
        if self.evaluated_frame_active {
            return;
        }
        self.evaluated_frame_active = true;
        if let (Some(geometry), Some((width, height))) =
            (self.host_geometry.clone(), self.host_viewport)
        {
            let _ = self.apply_host_stage_geometry(&geometry, width, height);
        }
    }
}

fn stage_navigation_events(
    delta_x: f64,
    delta_y: f64,
    magnification: f64,
    modifiers: Modifiers,
    x: f64,
    y: f64,
) -> Option<[Event; 2]> {
    if ![delta_x, delta_y, magnification, x, y]
        .into_iter()
        .all(f64::is_finite)
    {
        return None;
    }
    let navigation = if magnification == 0.0 {
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(delta_x as f32, delta_y as f32),
            phase: TouchPhase::Move,
            modifiers,
        }
    } else {
        let zoom = (magnification as f32).exp();
        if !zoom.is_finite() {
            return None;
        }
        Event::Zoom(zoom)
    };
    Some([
        Event::PointerMoved(Pos2::new(x as f32, y as f32)),
        navigation,
    ])
}

fn stage_modifiers(bits: u32) -> Modifiers {
    Modifiers {
        shift: bits & 1 != 0,
        ctrl: bits & 2 != 0,
        alt: bits & 4 != 0,
        mac_cmd: bits & 8 != 0,
        command: bits & 8 != 0,
    }
}

fn egui_pointer_button(button: StagePointerButton) -> PointerButton {
    match button {
        StagePointerButton::Primary => PointerButton::Primary,
        StagePointerButton::Secondary => PointerButton::Secondary,
        StagePointerButton::Middle => PointerButton::Middle,
    }
}

impl EmbeddedSpatialStage {
    fn show_transform_gizmo(&mut self, ui: &egui::Ui) {
        let cursor_pos = self.gizmo_pointer_position;
        let drag_started = std::mem::take(&mut self.gizmo_pointer_pressed);
        let dragging = self.gizmo_pointer_down;
        let released = std::mem::take(&mut self.gizmo_pointer_released);
        let viewport = ui.clip_rect();
        if viewport.width() <= 0.0 || viewport.height() <= 0.0 {
            return;
        }

        let Some((selected_layer, target_transform)) = self.selected_gizmo_transform() else {
            return;
        };

        // なぜ: authoring gizmoもRerun標準cameraが実際に評価したEyeをそのまま使う。
        let Some(eye) = self.spatial_stage.last_eye() else {
            return;
        };
        let eye_translation = eye.world_from_rub_view.translation();
        let eye_rotation = eye.world_from_rub_view.rotation();
        let world_from_view = DMat4::from_rotation_translation(
            DQuat::from_xyzw(
                f64::from(eye_rotation.x),
                f64::from(eye_rotation.y),
                f64::from(eye_rotation.z),
                f64::from(eye_rotation.w),
            ),
            DVec3::new(
                f64::from(eye_translation.x),
                f64::from(eye_translation.y),
                f64::from(eye_translation.z),
            ),
        );
        let view_matrix = world_from_view.inverse();
        let fov_y = f64::from(eye.fov_y.unwrap_or(re_view_spatial::Eye::DEFAULT_FOV_Y));
        let projection_matrix = DMat4::perspective_infinite_rh(
            fov_y,
            f64::from(viewport.width() / viewport.height()),
            0.01,
        );
        self.gizmo.update_config(GizmoConfig {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            viewport: GizmoRect::from_min_max(
                GizmoPos2::new(viewport.min.x, viewport.min.y),
                GizmoPos2::new(viewport.max.x, viewport.max.y),
            ),
            // なぜ: all_* は RotateX/Y/View が None、ScaleZ が XY noop、Uniform が RotateView と排他。
            modes: GizmoMode::TranslateX
                | GizmoMode::TranslateY
                | GizmoMode::TranslateXY
                | GizmoMode::RotateZ
                | GizmoMode::ScaleX
                | GizmoMode::ScaleY
                | GizmoMode::ScaleUniform,
            orientation: GizmoOrientation::Local,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..Default::default()
        });

        let interaction = GizmoInteraction {
            cursor_pos: (cursor_pos.x, cursor_pos.y),
            hovered: viewport.contains(cursor_pos),
            drag_started,
            dragging,
        };
        if let Some((result, _transforms)) = self.gizmo.update(interaction, &[target_transform]) {
            if let Some(edit) = stage_transform_edit(result, DQuat::from(target_transform.rotation))
                && !stage_transform_edit_is_noop(edit)
            {
                self.gizmo_gesture = Some((selected_layer.clone(), edit));
                self.pending_gizmo_action = Some(StageGizmoAction::Preview {
                    layer_id: selected_layer.clone(),
                    edit,
                });
            }
        }

        if self.gizmo_cancel_requested {
            self.gizmo_cancel_requested = false;
            if self.gizmo_gesture.take().is_some() {
                self.pending_gizmo_action = Some(StageGizmoAction::Cancel);
            }
        } else if released && let Some((layer_id, edit)) = self.gizmo_gesture.take() {
            self.pending_gizmo_action = Some(StageGizmoAction::Commit { layer_id, edit });
        }

        let draw_data = self.gizmo.draw();
        ui.painter().add(Mesh {
            indices: draw_data.indices,
            vertices: draw_data
                .vertices
                .into_iter()
                .zip(draw_data.colors)
                .map(|(position, [r, g, b, a])| EguiVertex {
                    pos: Pos2::new(position[0], position[1]),
                    uv: Pos2::ZERO,
                    color: egui::Rgba::from_rgba_premultiplied(r, g, b, a).into(),
                })
                .collect(),
            ..Default::default()
        });
    }

    fn selected_gizmo_transform(&self) -> Option<(String, Transform)> {
        let layer_id = self.host_primary_layer_id.as_ref()?;
        let layer = self
            .host_geometry
            .as_ref()?
            .layers
            .iter()
            .find(|layer| &layer.layer_id == layer_id)?;
        if layer.scale[0].abs() <= f64::EPSILON || layer.scale[1].abs() <= f64::EPSILON {
            return None;
        }
        Some((
            layer_id.clone(),
            Transform::from_scale_rotation_translation(
                DVec3::new(layer.scale[0], layer.scale[1], 1.0),
                DQuat::from_rotation_z(layer.rotation),
                DVec3::new(layer.position[0], layer.position[1], 0.0),
            ),
        ))
    }

    fn show_feedback(&self, ui: &egui::Ui) {
        let Some((message, rejected)) = &self.feedback else {
            return;
        };
        let color = if *rejected {
            Color32::from_rgb(255, 145, 145)
        } else {
            Color32::from_rgb(170, 240, 196)
        };
        ui.painter().text(
            ui.clip_rect().left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            message,
            FontId::proportional(13.0),
            color,
        );
    }
}

fn stage_transform_edit(
    result: GizmoResult,
    local_rotation: DQuat,
) -> Option<AppStageTransformEdit> {
    match result {
        GizmoResult::Translation { total, .. } => {
            // なぜ: Local の total はローカル。host の TranslateWorld は world delta を要求する。
            let world = local_rotation * DVec3::from(total);
            Some(AppStageTransformEdit::TranslateWorld([world.x, world.y]))
        }
        GizmoResult::Rotation { total, axis, .. } if axis.z.abs() >= 0.5 => {
            // なぜ: gizmo の applied は from_axis_angle(axis, -total)。Document +Z へ合わせる。
            Some(AppStageTransformEdit::RotateZ(-total * axis.z.signum()))
        }
        GizmoResult::Scale { total } => Some(AppStageTransformEdit::Scale([total.x, total.y])),
        GizmoResult::Rotation { .. } | GizmoResult::Arcball { .. } => None,
    }
}

fn stage_transform_edit_is_noop(edit: AppStageTransformEdit) -> bool {
    match edit {
        AppStageTransformEdit::TranslateWorld(delta) => {
            delta[0].abs() <= f64::EPSILON && delta[1].abs() <= f64::EPSILON
        }
        AppStageTransformEdit::RotateZ(delta) => delta.abs() <= f64::EPSILON,
        AppStageTransformEdit::Scale(scale) => {
            (scale[0] - 1.0).abs() <= f64::EPSILON && (scale[1] - 1.0).abs() <= f64::EPSILON
        }
    }
}

/// Preview fixtureだけの既定値。DocumentのPathOp評価・保存値をここへ持ち込まない。
fn preview_path_operation(id: &str) -> Option<ResolvedPathOp> {
    Some(match id {
        // Puckerは頂点だけでなくハンドルも変形するため、Bezierを経由する表示確認になる。
        "pucker-bloat" => ResolvedPathOp::PuckerBloat { amount: -0.42 },
        "zig-zag" => ResolvedPathOp::ZigZag {
            amount: 0.035,
            ridges: 3.0,
            point_type: PointType::Smooth,
        },
        "offset" => ResolvedPathOp::Offset {
            distance: 0.035,
            line_join: LineJoin::Round,
            miter_limit: 4.0,
        },
        "round-corners" => ResolvedPathOp::RoundCorners { radius: 0.055 },
        "trim" => ResolvedPathOp::Trim {
            start: 0.08,
            end: 0.72,
            offset: 0.0,
            mode: TrimMode::Parallel,
        },
        "twist" => ResolvedPathOp::Twist {
            angle: std::f64::consts::FRAC_PI_4,
            center: Point::ZERO,
        },
        "wiggle" => ResolvedPathOp::Wiggle {
            amp: 0.035,
            freq: 1.0,
            seed: 7,
        },
        "repeater" => ResolvedPathOp::Repeater {
            copies: 3.0,
            offset: 0.0,
            transform: ResolvedTransform {
                position: Point { x: 0.075, y: 0.055 },
                anchor: Point::ZERO,
                scale: Point { x: 0.88, y: 0.88 },
                rotation: 0.12,
            },
            composite: CompositeOrder::Above,
            start_opacity: 1.0,
            end_opacity: 1.0,
        },
        _ => return None,
    })
}

fn rectangle_path(center_x: f32, center_y: f32) -> Path {
    let half_width = 0.14;
    let half_height = 0.10;
    Path {
        contours: vec![Contour {
            vertices: vec![
                Vertex::corner(Point {
                    x: f64::from(center_x - half_width),
                    y: f64::from(center_y - half_height),
                }),
                Vertex::corner(Point {
                    x: f64::from(center_x + half_width),
                    y: f64::from(center_y - half_height),
                }),
                Vertex::corner(Point {
                    x: f64::from(center_x + half_width),
                    y: f64::from(center_y + half_height),
                }),
                Vertex::corner(Point {
                    x: f64::from(center_x - half_width),
                    y: f64::from(center_y + half_height),
                }),
            ],
            closed: true,
        }],
    }
}

fn path_from_canonical_corners(corners: [[f64; 2]; 4]) -> Path {
    Path {
        contours: vec![Contour {
            vertices: corners
                .into_iter()
                .map(|[x, y]| Vertex::corner(Point { x, y }))
                .collect(),
            closed: true,
        }],
    }
}

fn path_meshes_from_canonical_corners(
    corners: [[f64; 2]; 4],
) -> Result<(MeshData, MeshData), String> {
    tessellate_path(&path_from_canonical_corners(corners), Transform::default())
}

fn path_stroke_from_canonical_corners(corners: [[f64; 2]; 4]) -> Result<MeshData, String> {
    path_meshes_from_canonical_corners(corners).map(|(_, stroke)| stroke)
}

fn host_layer_path(layer_id: &str) -> String {
    format!("motolii/document/layers/{layer_id}/path")
}

fn host_layer_fill_path(layer_id: &str) -> String {
    format!("motolii/document/layers/{layer_id}/fill")
}

fn host_layer_mesh_paths(layer_id: &str) -> [String; 2] {
    [host_layer_fill_path(layer_id), host_layer_path(layer_id)]
}

pub(crate) fn host_layer_id_from_entity_path(entity_path: &str) -> Option<&str> {
    let layer = entity_path
        .strip_prefix('/')
        .unwrap_or(entity_path)
        .strip_prefix("motolii/document/layers/")?;
    let (layer_id, visualizer_leaf) = layer.rsplit_once('/')?;
    (!layer_id.is_empty() && matches!(visualizer_leaf, "fill" | "path")).then_some(layer_id)
}

/// Image 未着なら fill を出す。gizmo preview 中は stale Image の上に fill を残す。
pub(crate) fn host_layer_fill_is_visible(
    corners: [[f64; 2]; 4],
    evaluated_frame_active: bool,
    previewing: bool,
) -> bool {
    if evaluated_frame_active && !previewing {
        return false;
    }
    path_meshes_from_canonical_corners(corners)
        .map(|(fill, _)| !fill.indices.is_empty() && fill.vertices != hidden_layer_mesh().vertices)
        .unwrap_or(false)
}

fn ingest_host_layer_meshes(
    stage: &mut re_view_spatial::SpatialStage,
    layer_id: &str,
    corners: [[f64; 2]; 4],
    hide_fill: bool,
) -> bool {
    let Ok((fill, stroke)) = path_meshes_from_canonical_corners(corners) else {
        return false;
    };
    let (fill_mesh, fill_color) = if hide_fill {
        (hidden_layer_mesh(), STAGE_HOST_ERASE_COLOR)
    } else {
        (fill, DOCUMENT_RECT_FILL_COLOR)
    };
    ingest_mesh(
        stage,
        &host_layer_fill_path(layer_id),
        fill_mesh,
        fill_color,
    ) && ingest_mesh(
        stage,
        &host_layer_path(layer_id),
        stroke,
        FIXTURE_RECT_STROKE_COLOR,
    )
}

fn lyon_path(path: &Path) -> LyonPath {
    let mut builder = LyonPath::builder();
    for contour in &path.contours {
        let Some(first) = contour.vertices.first() else {
            continue;
        };
        builder.begin(point(first.point.x as f32, first.point.y as f32));
        for pair in contour.vertices.windows(2) {
            add_segment(&mut builder, pair[0], pair[1]);
        }
        if contour.closed && contour.vertices.len() > 1 {
            add_segment(
                &mut builder,
                *contour.vertices.last().unwrap_or(first),
                *first,
            );
        }
        builder.end(contour.closed);
    }
    builder.build()
}

fn add_segment(builder: &mut lyon_path::path::Builder, from: Vertex, to: Vertex) {
    let control1 = Point {
        x: from.point.x + from.out_tangent.x,
        y: from.point.y + from.out_tangent.y,
    };
    let control2 = Point {
        x: to.point.x + to.in_tangent.x,
        y: to.point.y + to.in_tangent.y,
    };
    if from.out_tangent == Point::ZERO && to.in_tangent == Point::ZERO {
        builder.line_to(point(to.point.x as f32, to.point.y as f32));
    } else {
        builder.cubic_bezier_to(
            point(control1.x as f32, control1.y as f32),
            point(control2.x as f32, control2.y as f32),
            point(to.point.x as f32, to.point.y as f32),
        );
    }
}

fn tessellate_path(path: &Path, transform: Transform) -> Result<(MeshData, MeshData), String> {
    // 正準座標での誤差上限。固定頂点数ではなく曲率に応じて分割する。
    const TOLERANCE: f32 = 0.000_1;
    let path = lyon_path(path);
    let mut fill = VertexBuffers::<_, u16>::new();
    let mut stroke = VertexBuffers::<_, u16>::new();
    FillTessellator::new()
        .tessellate_path(
            &path,
            &FillOptions::default().with_tolerance(TOLERANCE),
            &mut simple_builder(&mut fill),
        )
        .map_err(|error| format!("tessellate path fill: {error}"))?;
    StrokeTessellator::new()
        .tessellate_path(
            &path,
            &StrokeOptions::default()
                .with_line_width(0.008)
                .with_tolerance(TOLERANCE),
            &mut simple_builder(&mut stroke),
        )
        .map_err(|error| format!("tessellate path stroke: {error}"))?;
    Ok((
        MeshData::from(fill).transformed(transform),
        MeshData::from(stroke).transformed(transform),
    ))
}

#[derive(Debug, Clone)]
struct MeshData {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

impl From<VertexBuffers<lyon_path::math::Point, u16>> for MeshData {
    fn from(buffers: VertexBuffers<lyon_path::math::Point, u16>) -> Self {
        Self {
            vertices: buffers
                .vertices
                .into_iter()
                .map(|point| [point.x, point.y, 0.0])
                .collect(),
            indices: buffers
                .indices
                .chunks_exact(3)
                .map(|triangle| {
                    [
                        u32::from(triangle[0]),
                        u32::from(triangle[1]),
                        u32::from(triangle[2]),
                    ]
                })
                .collect(),
        }
    }
}

impl MeshData {
    fn transformed(mut self, transform: Transform) -> Self {
        let translation = DVec3::from(transform.translation);
        let rotation = DQuat::from(transform.rotation);
        let scale = DVec3::from(transform.scale);
        for vertex in &mut self.vertices {
            let point = rotation
                * (DVec3::new(
                    f64::from(vertex[0]),
                    f64::from(vertex[1]),
                    f64::from(vertex[2]),
                ) * scale)
                + translation;
            *vertex = [point.x as f32, point.y as f32, point.z as f32];
        }
        self
    }
}

fn hidden_layer_mesh() -> MeshData {
    MeshData {
        vertices: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        indices: vec![[0, 1, 2]],
    }
}

fn ingest_mesh(
    stage: &mut re_view_spatial::SpatialStage,
    entity_path: &str,
    mesh: MeshData,
    color: u32,
) -> bool {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return true;
    }
    let normals = vec![[0.0, 0.0, 1.0]; mesh.vertices.len()];
    let colors = vec![color; mesh.vertices.len()];
    let mesh = Mesh3D::new(mesh.vertices)
        .with_triangle_indices(mesh.indices)
        .with_vertex_normals(normals)
        .with_vertex_colors(colors);
    let Ok(chunk) = Chunk::builder(entity_path)
        .with_archetype_auto_row(TimePoint::default(), &mesh)
        .build()
    else {
        return false;
    };
    stage.ingest_chunk(Arc::new(chunk)).is_ok()
}

/// content を viewport に aspect-fit した NDC 矩形 `[x, y, w, h]`（Y-up、中心原点）。
pub(crate) fn aspect_fit_ndc_rect(
    content_w: f32,
    content_h: f32,
    viewport_w: f32,
    viewport_h: f32,
) -> [f32; 4] {
    let scale = (viewport_w / content_w).min(viewport_h / content_h);
    let fitted_w = content_w * scale;
    let fitted_h = content_h * scale;
    let ndc_w = 2.0 * fitted_w / viewport_w;
    let ndc_h = 2.0 * fitted_h / viewport_h;
    [-0.5 * ndc_w, -0.5 * ndc_h, ndc_w, ndc_h]
}

/// canonical (Y-up, height=1, width=aspect) → aspect-fit NDC。
pub(crate) fn canonical_to_fit_ndc(cx: f64, cy: f64, aspect: f64, fit: [f32; 4]) -> [f32; 2] {
    let u = (cx / aspect + 0.5) as f32;
    let v = (cy + 0.5) as f32;
    [fit[0] + u * fit[2], fit[1] + v * fit[3]]
}

pub(crate) fn rgba_from_u32(color: u32) -> [f32; 4] {
    let r = ((color >> 24) & 0xff) as f32 / 255.0;
    let g = ((color >> 16) & 0xff) as f32 / 255.0;
    let b = ((color >> 8) & 0xff) as f32 / 255.0;
    let a = (color & 0xff) as f32 / 255.0;
    [r, g, b, a]
}

pub(crate) fn fixture_rect_fill_rgba() -> [f32; 4] {
    rgba_from_u32(FIXTURE_RECT_FILL_COLOR)
}

pub(crate) fn fixture_rect_stroke_rgba() -> [f32; 4] {
    rgba_from_u32(FIXTURE_RECT_STROKE_COLOR)
}

/// canonical corners → fixture と同じ mesh 空間（host 投影経路）。
/// `nx = 0.5 + cx*(h/w)`, `ny = 0.5 - cy` のあと fixture の `(n - 0.5)` 写像。
/// fixture 単独経路は呼ばない（正方近似を維持）。
pub(crate) fn mesh_vertices_from_canonical_corners(
    corners: [[f64; 2]; 4],
    viewport_width: u32,
    viewport_height: u32,
) -> [[f32; 3]; 4] {
    let w = f64::from(viewport_width.max(1));
    let h = f64::from(viewport_height.max(1));
    let aspect = h / w;
    corners.map(|[cx, cy]| {
        let nx = 0.5 + cx * aspect;
        let ny = 0.5 - cy;
        [nx as f32 - 0.5, ny as f32 - 0.5, 0.0]
    })
}

pub(crate) fn apply_move_preview_to_geometry(
    geometry: &HostStageGeometry,
    preview: Option<&(String, [f64; 2])>,
) -> HostStageGeometry {
    let Some((layer_id, delta)) = preview else {
        return geometry.clone();
    };
    let mut next = geometry.clone();
    for layer in &mut next.layers {
        if layer.layer_id == *layer_id {
            for corner in &mut layer.corners {
                corner[0] += delta[0];
                corner[1] += delta[1];
            }
            layer.position[0] += delta[0];
            layer.position[1] += delta[1];
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_navigation_maps_wheel_and_pinch_to_egui_events() {
        let [
            Event::PointerMoved(position),
            Event::MouseWheel {
                unit,
                delta,
                modifiers,
                ..
            },
        ] = stage_navigation_events(3.0, -4.0, 0.0, stage_modifiers(5), 20.0, 30.0)
            .expect("wheel event")
        else {
            panic!("wheel must include pointer position and MouseWheel");
        };
        assert_eq!(position, Pos2::new(20.0, 30.0));
        assert_eq!(unit, MouseWheelUnit::Point);
        assert_eq!(delta, Vec2::new(3.0, -4.0));
        assert!(modifiers.shift && modifiers.alt);

        let [Event::PointerMoved(_), Event::Zoom(zoom)] =
            stage_navigation_events(0.0, 0.0, 0.2, Modifiers::NONE, 20.0, 30.0)
                .expect("pinch event")
        else {
            panic!("pinch must include pointer position and Zoom");
        };
        assert!((zoom - 0.2_f32.exp()).abs() < f32::EPSILON);
        assert!(stage_navigation_events(f64::NAN, 0.0, 0.0, Modifiers::NONE, 0.0, 0.0).is_none());
        let modifiers = stage_modifiers(1 | 2 | 4 | 8);
        assert!(modifiers.shift && modifiers.ctrl && modifiers.alt);
        assert!(modifiers.mac_cmd && modifiers.command);
        assert_eq!(
            egui_pointer_button(StagePointerButton::Primary),
            PointerButton::Primary
        );
        assert_eq!(
            egui_pointer_button(StagePointerButton::Secondary),
            PointerButton::Secondary
        );
        assert_eq!(
            egui_pointer_button(StagePointerButton::Middle),
            PointerButton::Middle
        );
    }

    #[test]
    fn pucker_preview_is_tessellated_as_curved_fill_and_stroke() {
        let path = pathgeom::apply(
            &rectangle_path(0.0, 0.0),
            &preview_path_operation("pucker-bloat").expect("fixture operation"),
            0.0,
        )
        .expect("fixture operation evaluates");
        let (fill, stroke) =
            tessellate_path(&path, Transform::default()).expect("Bezier path tessellates");

        assert!(
            fill.vertices.len() > 4,
            "curve uses adaptive vertices, not a rectangle fan"
        );
        assert!(!fill.indices.is_empty());
        assert!(
            stroke.vertices.len() > 8,
            "stroke follows the same curved path"
        );
    }

    #[test]
    fn every_visible_path_operation_has_a_preview_evaluation() {
        for id in [
            "pucker-bloat",
            "zig-zag",
            "offset",
            "round-corners",
            "trim",
            "twist",
            "wiggle",
            "repeater",
        ] {
            let path = pathgeom::apply(
                &rectangle_path(0.0, 0.0),
                &preview_path_operation(id).expect("known path operation"),
                0.0,
            )
            .expect("fixture operation evaluates");
            let (_, stroke) =
                tessellate_path(&path, Transform::default()).expect("evaluated path tessellates");
            assert!(
                !stroke.indices.is_empty(),
                "{id} has a visible Stage outline"
            );
        }
    }

    #[test]
    fn performance_gizmo_transform_moves_and_rotates_fixture_vertices() {
        let mesh = MeshData {
            vertices: vec![[1.0, 0.0, 0.0]],
            indices: vec![],
        }
        .transformed(Transform::from_scale_rotation_translation(
            DVec3::ONE,
            DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2),
            DVec3::new(0.5, -0.25, 0.75),
        ));

        let [x, y, z] = mesh.vertices[0];
        assert!((x - 0.5).abs() < 0.000_1);
        assert!((y - 0.75).abs() < 0.000_1);
        assert!((z - 0.75).abs() < 0.000_1);
    }

    #[test]
    fn canonical_corners_map_to_fixture_mesh_space() {
        // seed unit rect: center(0,0) size(1,1) / 正方 viewport では旧写像と一致
        let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
        let verts = mesh_vertices_from_canonical_corners(corners, 1, 1);
        // (nx,ny)=(cx+0.5, 0.5-cy) → (n-0.5) = (cx, -cy)
        assert_eq!(verts[0], [-0.5, 0.5, 0.0]);
        assert_eq!(verts[1], [0.5, 0.5, 0.0]);
        assert_eq!(verts[2], [0.5, -0.5, 0.0]);
        assert_eq!(verts[3], [-0.5, -0.5, 0.0]);
    }

    #[test]
    fn canonical_to_normalized_uses_viewport_aspect() {
        // w=2h → h/w=0.5。cx=0.5 → nx=0.5+0.25=0.75
        let corners = [[0.5, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];
        let verts = mesh_vertices_from_canonical_corners(corners, 2, 1);
        let nx = verts[0][0] as f64 + 0.5;
        assert!((nx - 0.75).abs() < 1e-9);
    }

    #[test]
    fn aspect_fit_ndc_rect_for_16x9_viewports() {
        // 同一比率 → 全面
        assert_eq!(
            aspect_fit_ndc_rect(1920.0, 1080.0, 1920.0, 1080.0),
            [-1.0, -1.0, 2.0, 2.0]
        );
        // 正方: 横フル、縦 1080/1920 * 2
        assert_eq!(
            aspect_fit_ndc_rect(1920.0, 1080.0, 1920.0, 1920.0),
            [-1.0, -0.5625, 2.0, 1.125]
        );
        // ウルトラワイド: 縦フル、横 1920/3840 * 2
        assert_eq!(
            aspect_fit_ndc_rect(1920.0, 1080.0, 3840.0, 1080.0),
            [-0.5, -1.0, 1.0, 2.0]
        );
        // 縦長 9:16 viewport: scale=1080/1920
        let tall = aspect_fit_ndc_rect(1920.0, 1080.0, 1080.0, 1920.0);
        assert_eq!(tall[0], -1.0);
        assert_eq!(tall[2], 2.0);
        assert!((tall[3] - 0.632_812_5).abs() < 1e-6);
        assert!((tall[1] + 0.316_406_25).abs() < 1e-6);
    }

    #[test]
    fn host_path_stroke_vertices_follow_projected_corners() {
        let before = path_stroke_from_canonical_corners([
            [-0.5, -0.5],
            [0.5, -0.5],
            [0.5, 0.5],
            [-0.5, 0.5],
        ])
        .expect("baseline path");
        let after = path_stroke_from_canonical_corners([
            [-0.4, -0.5],
            [0.6, -0.5],
            [0.6, 0.5],
            [-0.4, 0.5],
        ])
        .expect("translated path");
        assert_ne!(
            before.vertices, after.vertices,
            "Stage path mesh must move when Document corners move"
        );
    }

    #[test]
    fn host_path_fill_and_stroke_tessellate_from_corners() {
        let (fill, stroke) = path_meshes_from_canonical_corners([
            [-0.5, -0.5],
            [0.5, -0.5],
            [0.5, 0.5],
            [-0.5, 0.5],
        ])
        .expect("rect path tessellates");
        assert!(
            !fill.indices.is_empty(),
            "Stage layer fill must reach Mesh3D"
        );
        assert!(
            !stroke.indices.is_empty(),
            "Stage layer stroke must reach Mesh3D"
        );
    }

    #[test]
    fn rerun_layer_entity_paths_remap_to_document_layer_identity() {
        assert_eq!(
            host_layer_id_from_entity_path("motolii/document/layers/42/fill"),
            Some("42")
        );
        assert_eq!(
            host_layer_id_from_entity_path("motolii/document/layers/42/path"),
            Some("42")
        );
        assert_eq!(
            host_layer_id_from_entity_path("/motolii/document/layers/42/path"),
            Some("42")
        );
        assert_eq!(
            host_layer_id_from_entity_path("motolii/document/frame"),
            None
        );
        assert_eq!(
            host_layer_id_from_entity_path("motolii/document/layers/42/other"),
            None
        );
    }

    #[test]
    fn evaluated_frame_hides_opaque_fill_so_image_is_visible() {
        let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
        let (fill, _) = path_meshes_from_canonical_corners(corners).expect("fill");
        let hidden = hidden_layer_mesh();
        assert!(
            host_layer_fill_is_visible(corners, false, false),
            "place must keep fill until evaluated Image"
        );
        assert!(
            !host_layer_fill_is_visible(corners, true, false),
            "evaluated frame must hide the opaque fill in front of the Image"
        );
        assert!(
            host_layer_fill_is_visible(corners, true, true),
            "gizmo preview must keep fill while the evaluated Image is stale"
        );
        assert_ne!(
            fill.vertices, hidden.vertices,
            "evaluated frame must not keep the opaque fill mesh in front of the Image"
        );
        assert_eq!(DOCUMENT_FRAME_ENTITY, "motolii/document/frame");
    }

    #[test]
    fn stage_transform_edit_maps_translate_rotate_z_and_scale() {
        assert_eq!(
            stage_transform_edit(
                GizmoResult::Translation {
                    delta: DVec3::ZERO.into(),
                    total: DVec3::new(0.1, -0.2, 0.9).into(),
                },
                DQuat::IDENTITY,
            ),
            Some(AppStageTransformEdit::TranslateWorld([0.1, -0.2]))
        );
        assert_eq!(
            stage_transform_edit(
                GizmoResult::Rotation {
                    axis: DVec3::Z.into(),
                    delta: 0.0,
                    total: 0.25,
                    is_view_axis: false,
                },
                DQuat::IDENTITY,
            ),
            Some(AppStageTransformEdit::RotateZ(-0.25))
        );
        assert_eq!(
            stage_transform_edit(
                GizmoResult::Scale {
                    total: DVec3::new(1.5, 0.5, 3.0).into(),
                },
                DQuat::IDENTITY,
            ),
            Some(AppStageTransformEdit::Scale([1.5, 0.5]))
        );
    }

    #[test]
    fn stage_transform_edit_maps_local_translation_to_world() {
        let rotated = DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2);
        let Some(AppStageTransformEdit::TranslateWorld(delta)) = stage_transform_edit(
            GizmoResult::Translation {
                delta: DVec3::ZERO.into(),
                total: DVec3::new(0.1, 0.0, 0.0).into(),
            },
            rotated,
        ) else {
            panic!("local X translation must map to world XY");
        };
        assert!(delta[0].abs() < 1e-12);
        assert!((delta[1] - 0.1).abs() < 1e-12);
    }

    #[test]
    fn stage_transform_edit_rejects_unsupported_rotation() {
        assert_eq!(
            stage_transform_edit(
                GizmoResult::Rotation {
                    axis: DVec3::X.into(),
                    delta: 0.1,
                    total: 0.4,
                    is_view_axis: false,
                },
                DQuat::IDENTITY,
            ),
            None
        );
        assert_eq!(
            stage_transform_edit(
                GizmoResult::Rotation {
                    axis: DVec3::Y.into(),
                    delta: 0.1,
                    total: 0.4,
                    is_view_axis: false,
                },
                DQuat::IDENTITY,
            ),
            None
        );
        assert_eq!(
            stage_transform_edit(
                GizmoResult::Arcball {
                    delta: DQuat::IDENTITY.into(),
                    total: DQuat::IDENTITY.into(),
                },
                DQuat::IDENTITY,
            ),
            None
        );
    }

    #[test]
    fn stage_transform_edit_filters_noop() {
        let translate = stage_transform_edit(
            GizmoResult::Translation {
                delta: DVec3::ZERO.into(),
                total: DVec3::ZERO.into(),
            },
            DQuat::IDENTITY,
        )
        .expect("zero translation still maps");
        assert!(stage_transform_edit_is_noop(translate));

        let rotate = stage_transform_edit(
            GizmoResult::Rotation {
                axis: DVec3::Z.into(),
                delta: 0.0,
                total: 0.0,
                is_view_axis: false,
            },
            DQuat::IDENTITY,
        )
        .expect("zero rotation still maps");
        assert!(stage_transform_edit_is_noop(rotate));

        let scale = stage_transform_edit(
            GizmoResult::Scale {
                total: DVec3::ONE.into(),
            },
            DQuat::IDENTITY,
        )
        .expect("identity scale still maps");
        assert!(stage_transform_edit_is_noop(scale));
    }
}
