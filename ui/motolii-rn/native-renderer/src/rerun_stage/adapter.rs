use std::time::Instant;

use egui::{
    Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect, TouchPhase, Vec2,
};
use glam::EulerRot;
use motolii_ui::AppStageTransformEdit;
use transform_gizmo::{
    Gizmo,
    math::{DQuat, DVec3},
};

use crate::host_bridge::HostStageGeometry;
use crate::renderer_core::{PointerPhase, StagePointerButton};

use super::host_mesh::{
    apply_move_preview_to_geometry, hidden_layer_mesh, host_layer_fill_is_visible,
    host_layer_mesh_paths, ingest_host_layer_meshes, ingest_mesh,
};
use super::{DOCUMENT_FRAME_ENTITY, STAGE_HOST_ERASE_COLOR};

/// Owns only the adapter from the native Stage surface to Rerun's Spatial View.
///
/// Product chrome, persistence, and Document projection stay outside this adapter.
pub(crate) struct EmbeddedSpatialStage {
    pub(super) egui_ctx: egui::Context,
    pub(super) egui_renderer: egui_wgpu::Renderer,
    pub(super) spatial_stage: re_view_spatial::SpatialStage,
    pub(super) input_events: Vec<Event>,
    pub(super) input_modifiers: Modifiers,
    pub(super) started_at: Instant,
    pub(super) gizmo: Gizmo,
    /// Host `stage_geometry` 適用済み。
    pub(super) host_geometry_active: bool,
    pub(super) host_layer_ids: Vec<String>,
    /// 直近の host 投影（move preview の復元元）。
    pub(super) host_geometry: Option<HostStageGeometry>,
    /// primary 選択（outline用）。
    pub(super) host_primary_layer_id: Option<String>,
    /// move drag 中の world delta preview（対象 layer のみ）。
    pub(super) move_preview: Option<(String, [f64; 2])>,
    pub(super) gizmo_gesture: Option<(String, AppStageTransformEdit)>,
    pub(super) pending_gizmo_action: Option<StageGizmoAction>,
    pub(super) gizmo_cancel_requested: bool,
    pub(super) gizmo_pointer_position: Pos2,
    pub(super) gizmo_pointer_down: bool,
    pub(super) gizmo_pointer_pressed: bool,
    pub(super) gizmo_pointer_released: bool,
    pub(super) feedback: Option<(String, bool)>,
    /// 評価済み Document frame を Image visualizer へ載せているか。
    pub(super) evaluated_frame_active: bool,
    pub(super) host_viewport: Option<(u32, u32)>,
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

pub(super) fn stage_navigation_events(
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

pub(super) fn stage_modifiers(bits: u32) -> Modifiers {
    Modifiers {
        shift: bits & 1 != 0,
        ctrl: bits & 2 != 0,
        alt: bits & 4 != 0,
        mac_cmd: bits & 8 != 0,
        command: bits & 8 != 0,
    }
}

pub(super) fn egui_pointer_button(button: StagePointerButton) -> PointerButton {
    match button {
        StagePointerButton::Primary => PointerButton::Primary,
        StagePointerButton::Secondary => PointerButton::Secondary,
        StagePointerButton::Middle => PointerButton::Middle,
    }
}
