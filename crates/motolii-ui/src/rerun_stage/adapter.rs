use std::sync::Arc;
use std::time::Instant;

use egui::{
    Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect, TouchPhase, Vec2,
};
use glam::EulerRot;
use re_chunk::Chunk;
use re_log_types::{EntityPath, TimePoint};
use transform_gizmo::{
    Gizmo,
    math::{DQuat, DVec3},
};

use crate::stage_app_geometry::{AppStageGeometry, AppStageTransformEdit};

use super::document_camera::document_camera;
use super::host_mesh::{
    hidden_layer_mesh, host_layer_fill_is_visible, host_layer_mesh_paths, ingest_host_layer_meshes,
    ingest_mesh,
};
use super::{DOCUMENT_FRAME_ENTITY, STAGE_HOST_ERASE_COLOR};

// なぜここに定義があるか: 原文は native-renderer の `renderer_core::{PointerPhase,
// StagePointerButton}` を使っていた。素の enum なので rerun_stage 側へ定義ごと移す。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagePointerButton {
    Primary,
    Secondary,
    Middle,
}

impl StagePointerButton {
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(Self::Primary),
            1 => Some(Self::Secondary),
            2 => Some(Self::Middle),
            _ => None,
        }
    }
}

/// 自前 egui 一式。**`render()`(egui をここで1周回す経路)だけが使う。**
///
/// ホストが既に egui の場合は `show_in` を使い、この一式は持たない(`None`)。
/// 1つの窓の中で egui を二重に回さないため。
pub(super) struct OwnEgui {
    ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
}

/// Owns only the adapter from the native Stage surface to Rerun's Spatial View.
///
/// Product chrome, persistence, and Document projection stay outside this adapter.
pub struct EmbeddedSpatialStage {
    /// `new()` で作った場合だけ在る。`new_in_host()` では `None`
    /// — `re_renderer::RenderContext` はホスト側の `callback_resources` に住む。
    pub(super) own_egui: Option<OwnEgui>,
    pub(super) spatial_stage: re_view_spatial::SpatialStage,
    pub(super) input_events: Vec<Event>,
    pub(super) input_modifiers: Modifiers,
    pub(super) started_at: Instant,
    pub(super) gizmo: Gizmo,
    /// Host `stage_geometry` 適用済み。
    pub(super) host_geometry_active: bool,
    pub(super) host_layer_ids: Vec<String>,
    /// 直近の host 投影（move preview の復元元）。
    pub(super) host_geometry: Option<AppStageGeometry>,
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
    /// composition の縦横比。document camera の横合わせに使う。
    /// `None` は「まだ聞いていない」— 画枠と同じ形と見なす。
    pub(super) composition_aspect: Option<f32>,
    /// Stage 面の縦横比(物理px)。同上。
    pub(super) stage_viewport_aspect: Option<f32>,
    /// 絵と幾何を載せる時に落ちた理由。
    ///
    /// `show_in` は `Err` を返さない — 合成フレームが載らなくても幾何だけの絵は
    /// 出るし、出すべきだからである。だが**黙って出さない**のは別の話で、
    /// 理由はここへ溜めて `take_failures` で呼び手(pane)へ渡す
    /// (2026-08-18 外部診断 F-08)。帯へ出すのは pane 側で、同じ理由を
    /// 毎フレーム言わない latch もそちらに在る。
    pub(super) failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageGizmoAction {
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
pub struct StageTransformProjection {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub rotation_x: f64,
    pub rotation_y: f64,
    pub rotation_z: f64,
}

impl EmbeddedSpatialStage {
    /// 自前 egui を1周回す経路(`render()`)用。ホストが egui でない窓
    /// (RN の native surface 等)へ埋め込むときの口。
    ///
    /// ホストが既に egui なら `new_in_host` + `show_in` を使う。
    pub fn new(
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

        Self::with_own_egui(Some(OwnEgui {
            ctx: egui::Context::default(),
            renderer: egui_renderer,
        }))
    }

    /// ホストの egui へ直接挿す経路(`show_in`)用。**2つ目の egui を立てない。**
    ///
    /// `re_renderer::RenderContext` は **ホストの** `render_state.renderer` の
    /// `callback_resources` へ入れる。`SpatialStage` が積む paint callback は
    /// 塗る側の `egui_wgpu::Renderer` からそれを引くので、置き場所を間違えると
    /// callback が黙って何も描かずに戻る(絵が出ない)。
    /// Rerun 自身の eframe 版も同じ場所へ入れている
    /// (`re_viewer/src/lib.rs` `customize_eframe_and_setup_renderer`)。
    pub(crate) fn new_in_host(render_state: &eframe::egui_wgpu::RenderState) -> Result<Self, String> {
        {
            let mut renderer = render_state.renderer.write();
            if renderer
                .callback_resources
                .get::<re_renderer::RenderContext>()
                .is_none()
            {
                let render_ctx = re_renderer::RenderContext::new(
                    &render_state.adapter,
                    render_state.device.clone(),
                    render_state.queue.clone(),
                    render_state.target_format,
                    re_renderer::RenderConfig::best_for_device_caps,
                )
                .map_err(|error| format!("create Rerun render context: {error}"))?;
                renderer.callback_resources.insert(render_ctx);
            }
        }
        Self::with_own_egui(None)
    }

    fn with_own_egui(own_egui: Option<OwnEgui>) -> Result<Self, String> {
        let mut stage = Self {
            own_egui,
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
            composition_aspect: None,
            stage_viewport_aspect: None,
            failures: Vec::new(),
        };
        // 既定カメラを comp 平面の正面側へ置く。**Rerun の camera 機構の中で**決める
        // (独自 camera を持たない)ので、scene の向きを先に伝える。
        if !ingest_scene_view_coordinates(&mut stage.spatial_stage) {
            return Err("ingest Rerun scene view coordinates".to_string());
        }
        // 既定表示は document camera(正対)。**最初の1フレーム目から**置く
        // — Rerun の既定 eye が一度でも出ると、開いた瞬間だけ斜めの絵が見える。
        // 面の大きさも comp の縦横比もまだ聞いていないので、ここでは
        // 「画枠と comp が同じ形」の縦合わせになる(`document_camera` の注記)。
        stage.place_document_camera();
        // 製品マウントでは fixture rect を出さない。host snapshot / 評価フレームが正本。
        Ok(stage)
    }

    /// 今分かっている comp / 画枠の縦横比で document camera を組み直して置く。
    ///
    /// 片方しか分かっていないときは**もう片方も同じ形**と見なす。その仮定では
    /// 縦合わせと横合わせが一致するので、既知の側だけで距離が決まる
    /// (推測した縦横比で切ってしまうことがない)。
    fn place_document_camera(&mut self) {
        let (comp_aspect, viewport_aspect) =
            match (self.composition_aspect, self.stage_viewport_aspect) {
                (Some(comp), Some(viewport)) => (comp, viewport),
                (Some(comp), None) => (comp, comp),
                (None, Some(viewport)) => (viewport, viewport),
                (None, None) => (1.0, 1.0),
            };
        self.spatial_stage
            .set_camera(document_camera(comp_aspect, viewport_aspect));
    }

    /// composition の縦横比(幅/高さ)を伝える。**変わった時だけカメラを組み直す。**
    ///
    /// 出所は Document の `Composition`(正準では高さ1.0固定なので縦横比が寸法の全部)。
    /// 解像度が変われば縦横比も変わるので、ここが「composition 解像度変更時」の口である。
    /// 毎フレーム同じ値で呼ばれても、カメラは動かない(再生中に視点が跳ねない)。
    pub fn set_composition_aspect(&mut self, aspect: f32) -> bool {
        if !aspect.is_finite() || aspect <= 0.0 {
            return false;
        }
        if self.composition_aspect == Some(aspect) {
            return true;
        }
        self.composition_aspect = Some(aspect);
        self.place_document_camera();
        true
    }

    pub fn pointer(
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

    pub fn gizmo_pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
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

    pub fn scroll(
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

    pub fn gizmo_wants_pointer(&self, x: f64, y: f64) -> bool {
        self.selected_gizmo_transform().is_some() && self.gizmo.pick_preview((x as f32, y as f32))
    }

    pub fn take_gizmo_action(&mut self) -> Option<StageGizmoAction> {
        self.pending_gizmo_action.take()
    }

    pub fn set_feedback(&mut self, message: impl Into<String>, rejected: bool) {
        self.feedback = Some((message.into(), rejected));
    }

    /// 選択中 Document layer の TRS。未選択は零。Inspector 正本ではない。
    pub fn transform_projection(&self) -> StageTransformProjection {
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
    pub fn set_transform_projection(&mut self, projection: StageTransformProjection) -> bool {
        let _ = projection;
        true
    }

    pub fn clear_host_projection(&mut self) -> bool {
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

    pub fn set_host_primary_layer_id(&mut self, primary: Option<String>) {
        self.host_primary_layer_id = primary;
    }

    pub fn host_primary_layer_id(&self) -> Option<&str> {
        self.host_primary_layer_id.as_deref()
    }

    /// Host snapshot の stage_geometry を tessellate して載せる。
    /// `viewport_width/height` は host 投影の aspect 写像に使う（正方近似を避ける）。
    pub fn apply_host_stage_geometry(
        &mut self,
        geometry: &AppStageGeometry,
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
        // 消えた layer の板は先に潰す。id を先に写すのは、消しながら
        // `note_failure` を呼べるようにするため(自分の field を跨いで借りない)。
        let stale: Vec<String> = self
            .host_layer_ids
            .iter()
            .filter(|old_id| !next_ids.iter().any(|id| id == *old_id))
            .cloned()
            .collect();
        for old_id in &stale {
            if !ingest_mesh(
                &mut self.spatial_stage,
                old_id,
                hidden_layer_mesh(),
                STAGE_HOST_ERASE_COLOR,
            ) {
                // 消し損ねると**消えた layer の板が残り続ける**。絵だけ見ても
                // 「消したのに残っている」としか分からないので言う。
                self.note_failure(format!("消した layer の板を Stage から取り除けない: {old_id}"));
            }
        }

        let previewing = self.gizmo_gesture.is_some();
        for layer in &geometry.layers {
            if !ingest_host_layer_meshes(
                &mut self.spatial_stage,
                &layer.layer_id,
                layer.corners,
                !host_layer_fill_is_visible(layer.corners, self.evaluated_frame_active, previewing),
            ) {
                let layer_id = layer.layer_id.clone();
                self.note_failure(format!("layer の枠を Stage へ積めない: {layer_id}"));
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

    /// 面の大きさに合わせて document camera を掛け直す。
    ///
    /// **`SpatialStage::reset_view()` は呼ばない。** あれは Rerun 自身の framing
    /// (斜め見下ろしの既定 eye)へ戻す口で、置いた camera も落としてしまう
    /// — 面を広げるたびに正対が壊れることになる。
    pub fn fit_view(&mut self, viewport_width: u32, viewport_height: u32) -> bool {
        if viewport_width == 0 || viewport_height == 0 {
            return false;
        }
        let aspect = f64::from(viewport_width) / f64::from(viewport_height);
        self.stage_viewport_aspect = Some(aspect as f32);
        self.place_document_camera();
        true
    }

    /// ホストの `egui::Ui` の中へ直接描く。**2つ目の egui context を立てない。**
    ///
    /// `SpatialStage::show` は元から egui のウィジェットなので、ホストが egui なら
    /// `RawInput` の合成も `tessellate` も自前の render pass も要らない
    /// — `render()` にあるそれらは、ホストが egui でない窓のための足場。
    ///
    /// `re_renderer::RenderContext` はホストの `callback_resources` から借り出し、
    /// **この関数を抜けるまでに必ず返す**。返し忘れると、egui が塗る時点で
    /// callback が context を引けず、Stage が何も描かないまま戻る。
    ///
    /// 戻り値は `render()` と同じく `SpatialStage` が拾った選択 entity path。
    ///
    /// `evaluated_frame` は playhead 時刻の合成フレーム(`stage_frame_seat`)。
    /// `render()` 経路と**同じ** `present_evaluated_frame` を通し、Rerun 側の
    /// `GridMap` entity として comp 平面へ載せる。`None` なら幾何だけの従来表示
    /// (fixture 展示)。
    pub fn show_in(
        &mut self,
        ui: &mut egui::Ui,
        render_state: &eframe::egui_wgpu::RenderState,
        evaluated_frame: Option<&wgpu::Texture>,
    ) -> Result<Option<Option<String>>, String> {
        let Some(mut render_ctx) = render_state
            .renderer
            .write()
            .callback_resources
            .remove::<re_renderer::RenderContext>()
        else {
            return Err(
                "Rerun render context がホストの callback_resources に無い\
                 (new_in_host を通していない)"
                    .to_string(),
            );
        };

        // 絵は `show` より先に載せる。同じフレームの中で幾何(fill の隠し)まで
        // 追随させるため。
        if let Some(texture) = evaluated_frame {
            self.present_evaluated_frame(&render_ctx, texture);
        }

        let result = self.spatial_stage.show(ui, &mut render_ctx);

        // 借り物は成否にかかわらず返す。
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(render_ctx);

        if let Err(error) = result {
            return Err(format!("run Rerun spatial stage: {error}"));
        }

        // 順序は `render()` と同じ。gizmo と feedback は Stage の絵の上に乗る。
        self.show_transform_gizmo(ui);
        self.show_feedback(ui);
        Ok(self.spatial_stage.take_selected_entity_path())
    }

    /// **自前の egui を1周回す経路。** ホストが egui の場合は `show_in` を使う。
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
        evaluated_frame: Option<&wgpu::Texture>,
    ) -> Result<Option<Option<String>>, String> {
        // `own_egui` を丸ごと借り出す。`self` の他の場所と同時に触るため
        // (closure が `self.spatial_stage` を要る)、field 参照のままだと借用が割れない。
        let Some(mut own) = self.own_egui.take() else {
            return Err(
                "render() は自前 egui 経路。new_in_host で作った Stage は show_in を使う"
                    .to_string(),
            );
        };

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

        let egui_ctx = own.ctx.clone();
        let mut stage_error = None;
        let full_output = egui_ctx.run_ui(raw_input, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    let mut render_ctx = own
                        .renderer
                        .callback_resources
                        .remove::<re_renderer::RenderContext>()
                        .expect("Rerun render context is registered for the native Stage");
                    if let Some(texture) = evaluated_frame {
                        self.present_evaluated_frame(&render_ctx, texture);
                    }
                    let result = self.spatial_stage.show(ui, &mut render_ctx);
                    own.renderer.callback_resources.insert(render_ctx);
                    if let Err(error) = result {
                        stage_error = Some(error.to_string());
                    }
                    self.show_transform_gizmo(ui);
                    self.show_feedback(ui);
                });
        });
        if let Some(error) = stage_error {
            self.own_egui = Some(own);
            return Err(format!("run Rerun spatial stage: {error}"));
        }

        for (id, delta) in &full_output.textures_delta.set {
            own.renderer.update_texture(device, queue, *id, delta);
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
        let callback_command_buffers = own.renderer.update_buffers(
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
            own.renderer.render(
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
            own.renderer.free_texture(id);
        }
        self.own_egui = Some(own);
        Ok(self.spatial_stage.take_selected_entity_path())
    }

    /// 溜まっている失敗を引き取る(引き取ったら空になる)。
    ///
    /// 呼び手は shell の pane で、`ShellTranscript` へ写して帯と
    /// `--status-log` に出す。呼ばれない席(RN 経路・単体)では従来どおり
    /// 絵が出ないだけだが、**理由は捨てていない**。
    pub fn take_failures(&mut self) -> Vec<String> {
        std::mem::take(&mut self.failures)
    }

    /// 合成フレームを comp 平面へ載せる。
    ///
    /// 落ちても `show_in` は続ける — 幾何だけの絵は出るし、出すべきである。
    /// **ただし黙って出さない**: 理由は `failures` へ積んで呼び手が引き取る
    /// (2026-08-18 外部診断 F-08。以前はここが無言 `return` だった)。
    fn present_evaluated_frame(
        &mut self,
        render_ctx: &re_renderer::RenderContext,
        texture: &wgpu::Texture,
    ) {
        if let Err(error) =
            self.spatial_stage
                .copy_gpu_image(render_ctx, DOCUMENT_FRAME_ENTITY, texture)
        {
            self.note_failure(format!("合成フレームを Stage へ載せられない: {error}"));
            return;
        }
        if self.evaluated_frame_active {
            return;
        }
        self.evaluated_frame_active = true;
        if let (Some(geometry), Some((width, height))) =
            (self.host_geometry.clone(), self.host_viewport)
        {
            // 絵が載った初回は fill を隠すために幾何を積み直す。ここが落ちると
            // **絵の上に不透明な板が残る** — 見た目だけでは原因が読めないので言う。
            if !self.apply_host_stage_geometry(&geometry, width, height) {
                self.note_failure(
                    "合成フレームに合わせて幾何を積み直せない(layer の板が絵を覆ったままになる)"
                        .to_owned(),
                );
            }
        }
    }

    /// 失敗を1つ控える。**同じ理由を溜め込まない**(帯は1行しか出さないので、
    /// 毎フレーム落ちる経路で `Vec` が伸び続けるほうが害になる)。
    pub(super) fn note_failure(&mut self, text: String) {
        if self.failures.iter().any(|known| *known == text) {
            return;
        }
        self.failures.push(text);
    }
}

/// Motolii の正準座標(原点中央 / Y-up / 高さ1.0、comp 平面は XY・法線 +Z)を
/// Rerun の scene 座標系として root へ積む。**これが既定カメラの向きを決める。**
///
/// なぜ必要か: Rerun の既定 eye は blueprint の `EyeControls3D` が空のときに
/// fallback で決まり、その fallback は scene の view coordinates から軸を引く
/// (`re_view_spatial/src/view_3d.rs:206-233`)。何も積まないと
/// `unwrap_or_default()` が効いて right=+X / up=+Z / forward=+Y、つまり **Z-up の
/// 世界**とみなされる。comp 平面は XY にあるので、既定カメラはそれを地面すれすれの
/// 角度から見る絵になる(= 合成フレームが斜めに潰れる)。
///
/// `RIGHT_HAND_Y_UP`(X=Right, Y=Up, Z=Back)を積むと同じ fallback が
/// up=+Y / forward=-Z を引き、既定 eye は comp 平面の **正面側(+Z)** に立ち、
/// 上下も Document と揃う。orbit / zoom は Rerun 側のまま触らない。
pub(super) const STAGE_SCENE_VIEW_COORDINATES: re_sdk_types::components::ViewCoordinates =
    re_sdk_types::components::ViewCoordinates::RIGHT_HAND_Y_UP;

fn scene_view_coordinates_chunk() -> Option<Arc<Chunk>> {
    Chunk::builder(EntityPath::root())
        .with_archetype_auto_row(
            TimePoint::STATIC,
            &re_sdk_types::archetypes::ViewCoordinates::new(STAGE_SCENE_VIEW_COORDINATES),
        )
        .build()
        .ok()
        .map(Arc::new)
}

fn ingest_scene_view_coordinates(stage: &mut re_view_spatial::SpatialStage) -> bool {
    let Some(chunk) = scene_view_coordinates_chunk() else {
        return false;
    };
    stage.ingest_chunk(chunk).is_ok()
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

#[cfg(test)]
mod tests {
    use re_sdk_types::view_coordinates::{Axis3, Sign, SignedAxis3};

    use super::*;

    /// `SignedAxis3` → 単位ベクトル。`glam` は Motolii(0.32)と Rerun(0.30)で
    /// 別crateなので `From` が繋がらない。ここは素の配列で持つ。
    fn unit(axis: SignedAxis3) -> [f32; 3] {
        let sign = match axis.sign {
            Sign::Positive => 1.0,
            Sign::Negative => -1.0,
        };
        match axis.axis {
            Axis3::X => [sign, 0.0, 0.0],
            Axis3::Y => [0.0, sign, 0.0],
            Axis3::Z => [0.0, 0.0, sign],
        }
    }

    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    /// Rerun の既定 eye 方向の算出をそのまま写した審判。
    /// 出所は `re_view_spatial/src/view_3d.rs:206-233`
    /// (`EyeControls3D::position` の fallback provider)。
    /// **ここで式を変えない** — 変わったら Rerun 追随として別に扱う。
    fn rerun_default_eye_dir(coordinates: re_sdk_types::components::ViewCoordinates) -> [f32; 3] {
        let right = unit(coordinates.right().unwrap_or(SignedAxis3::POSITIVE_X));
        let eye_up = unit(coordinates.up().unwrap_or(SignedAxis3::POSITIVE_Z));
        let fwd = cross(eye_up, right);

        let dir = [
            0.75 * fwd[0] + 0.25 * right[0] - 0.25 * eye_up[0],
            0.75 * fwd[1] + 0.25 * right[1] - 0.25 * eye_up[1],
            0.75 * fwd[2] + 0.25 * right[2] - 0.25 * eye_up[2],
        ];
        let length = dot(dir, dir).sqrt();
        [dir[0] / length, dir[1] / length, dir[2] / length]
    }

    /// comp 平面の正面方向。canonical 座標では平面は XY にあり法線は ±Z。
    const COMP_PLANE_FRONT: [f32; 3] = [0.0, 0.0, -1.0];

    /// 何も積まないときの既定(`ViewCoordinates::default()` = RFU)は、
    /// comp 平面をほとんど真横から見る。これが「斜めに見える」の実体。
    #[test]
    fn rerun_default_scene_coordinates_look_at_the_comp_plane_edge_on() {
        let dir = rerun_default_eye_dir(re_sdk_types::components::ViewCoordinates::default());
        assert!(
            dot(dir, COMP_PLANE_FRONT) < 0.35,
            "既定 scene 座標系では comp 平面を正面から見ていない: {dir:?}"
        );
    }

    /// Stage が積む scene 座標系では、既定 eye は comp 平面の正面側に立つ。
    #[test]
    fn stage_scene_coordinates_put_the_default_eye_in_front_of_the_comp_plane() {
        let coordinates = STAGE_SCENE_VIEW_COORDINATES;
        assert_eq!(coordinates.describe_short(), "RUB");

        // Document と同じ上下。
        assert_eq!(
            coordinates.up().map(unit),
            Some([0.0, 1.0, 0.0]),
            "Stage の上は Document の上(+Y)"
        );

        // eye は comp 平面の正面(-Z 方向)を向く。
        let dir = rerun_default_eye_dir(coordinates);
        assert!(
            dot(dir, COMP_PLANE_FRONT) > 0.9,
            "既定 eye が comp 平面の正面を向いていない: {dir:?}"
        );
        // `eye_pos = center - radius * dir` なので、+Z 側に立つ。
        assert!(-dir[2] > 0.0, "既定 eye が comp 平面の裏側に立っている");
    }

    // ───────── document camera(既定=正対) ─────────
    //
    // 「編集時の既定表示も document camera(未定義なら正対)とし、現在の斜め固定視点は
    // view camera の初期値バグとして扱う」(2026-08-18 利用者裁定,
    // `docs/reviews/2026-08-18-rerun-as-composition-foundation.md`)。
    //
    // 下の審判は**画素を見ない**。`SpatialStage::camera()` が返す姿勢を、透視投影の
    // 定義だけで検算する。

    /// 合成フレーム面の半分の高さ。
    ///
    /// `SpatialStage::copy_gpu_image` は `cell_size = 1/height` で置くので、
    /// comp 平面は世界の中で必ず高さ1.0・幅 aspect になる(fork の
    /// `spatial_stage.rs` `copy_gpu_image`)。
    const COMPOSITION_HALF_HEIGHT: f32 = 0.5;

    /// カメラが正対しているか。位置と注視点が x/y で一致し、注視点より手前(+Z)に立つ。
    fn assert_head_on(camera: &re_view_spatial::StageCamera, what: &str) {
        assert_eq!(
            camera.up,
            [0.0, 1.0, 0.0],
            "{what}: Stage の上が Document の上(+Y)でない"
        );
        assert_eq!(
            (camera.position[0], camera.position[1]),
            (camera.look_target[0], camera.look_target[1]),
            "{what}: 視線が comp 平面の法線からずれている(斜めに見ている)"
        );
        assert!(
            camera.position[2] > camera.look_target[2],
            "{what}: カメラが comp 平面の裏側に立っている: {camera:?}"
        );
    }

    /// comp 平面の位置で画枠に収まる**半分の高さ**。
    ///
    /// `d * tan(fov_y/2)` は透視投影の定義そのもので、adapter の距離の決め方は
    /// 通らない。よって「定数を直書きして絵を合わせた」camera はここで落ちる。
    fn half_visible_height(camera: &re_view_spatial::StageCamera) -> f32 {
        let distance = camera.position[2] - camera.look_target[2];
        let fov_y = camera
            .fov_y_radians
            .expect("document camera が垂直画角を決めていない");
        distance * (fov_y * 0.5).tan()
    }

    /// **構築直後**の Stage には document camera が置かれている。
    ///
    /// ここが落ちている間、製品の Stage は Rerun の既定 eye(斜め見下ろし)で開く。
    #[test]
    fn a_freshly_built_stage_already_faces_the_composition_head_on() {
        let stage =
            EmbeddedSpatialStage::with_own_egui(None).expect("自前 egui 無しの Stage を作る");
        let camera = stage
            .spatial_stage
            .camera()
            .expect("構築直後の Stage に document camera が置かれていない");

        assert_head_on(&camera, "構築直後");

        let fov_y = camera
            .fov_y_radians
            .expect("document camera が垂直画角を決めていない");
        assert!(
            fov_y > 0.0 && fov_y < std::f32::consts::PI,
            "画角が画角になっていない: {fov_y}"
        );

        // 画枠の縦を comp の高さ1.0がちょうど満たす。
        // 距離は画角から逆算されているはずなので、ここは 0.5 に一致する。
        let half = half_visible_height(&camera);
        assert!(
            (half - COMPOSITION_HALF_HEIGHT).abs() < 1.0e-4,
            "comp の高さが画枠に収まっていない: 可視半高 {half} (期待 {COMPOSITION_HALF_HEIGHT})"
        );
    }

    /// 面の大きさが変わっても正対のまま。**Rerun の既定 eye へ戻さない。**
    #[test]
    fn fitting_the_view_keeps_the_document_camera_head_on() {
        let mut stage =
            EmbeddedSpatialStage::with_own_egui(None).expect("自前 egui 無しの Stage を作る");
        assert!(stage.fit_view(1920, 1080), "fit_view が失敗した");

        let camera = stage
            .spatial_stage
            .camera()
            .expect("fit_view の後に document camera が無い(Rerun 既定へ戻っている)");
        assert_head_on(&camera, "fit_view の後");
    }

    /// 面の形と comp の形を両方伝えたら、comp 全体が画枠へ収まる。
    ///
    /// 画枠より横長の comp を正方の面へ出す場合で見る。縦だけ合わせる実装は
    /// 左右がはみ出してここで落ちる。
    #[test]
    fn a_wide_composition_still_fits_a_square_stage() {
        let mut stage =
            EmbeddedSpatialStage::with_own_egui(None).expect("自前 egui 無しの Stage を作る");
        let comp_aspect = 16.0 / 9.0;
        assert!(stage.set_composition_aspect(comp_aspect));
        assert!(stage.fit_view(1000, 1000));

        let camera = stage.spatial_stage.camera().expect("document camera が無い");
        assert_head_on(&camera, "正方の面");

        let half_visible_h = half_visible_height(&camera);
        // 面が正方なので可視半幅 = 可視半高。
        let half_visible_w = half_visible_h;
        let half_comp_w = COMPOSITION_HALF_HEIGHT * comp_aspect;
        assert!(
            half_visible_w >= half_comp_w - 1.0e-5,
            "comp の左右が画枠からはみ出している: 可視半幅 {half_visible_w} < comp 半幅 {half_comp_w}"
        );
        assert!(
            (half_visible_w - half_comp_w).abs() < 1.0e-4,
            "横合わせなのに画枠に接していない: 可視半幅 {half_visible_w} / comp 半幅 {half_comp_w}"
        );
    }

    /// 同じ comp でも面の形が変われば距離が変わる。**定数を置いた実装はここで落ちる。**
    #[test]
    fn the_camera_moves_when_the_stage_changes_shape() {
        let comp_aspect = 16.0 / 9.0;
        let distance_for = |viewport: (u32, u32)| {
            let mut stage =
                EmbeddedSpatialStage::with_own_egui(None).expect("自前 egui 無しの Stage を作る");
            assert!(stage.set_composition_aspect(comp_aspect));
            assert!(stage.fit_view(viewport.0, viewport.1));
            let camera = stage.spatial_stage.camera().expect("document camera が無い");
            camera.position[2] - camera.look_target[2]
        };

        let matching = distance_for((1600, 900));
        let square = distance_for((1000, 1000));
        assert!(
            square > matching * 1.5,
            "正方の面でも同じ距離のまま(画枠の形を見ていない): 同形 {matching} / 正方 {square}"
        );
    }

    /// 同じ縦横比を毎フレーム伝えてもカメラは動かない(再生中に視点が跳ねない)。
    #[test]
    fn repeating_the_composition_shape_leaves_the_camera_alone() {
        let mut stage =
            EmbeddedSpatialStage::with_own_egui(None).expect("自前 egui 無しの Stage を作る");
        assert!(stage.set_composition_aspect(16.0 / 9.0));
        assert!(stage.fit_view(1600, 900));
        let before = stage.spatial_stage.camera().expect("document camera が無い");

        for _ in 0..8 {
            assert!(stage.set_composition_aspect(16.0 / 9.0));
        }
        let after = stage.spatial_stage.camera().expect("document camera が無い");
        assert_eq!(before, after, "同じ形を伝え直しただけでカメラが動いた");
    }

    /// 意味を成さない縦横比は受け取らない(壊れた値でカメラを潰さない)。
    #[test]
    fn a_broken_composition_shape_is_refused() {
        let mut stage =
            EmbeddedSpatialStage::with_own_egui(None).expect("自前 egui 無しの Stage を作る");
        let before = stage.spatial_stage.camera().expect("document camera が無い");
        for broken in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(
                !stage.set_composition_aspect(broken),
                "縦横比 {broken} を受け取ってしまった"
            );
        }
        let after = stage.spatial_stage.camera().expect("document camera が無い");
        assert_eq!(before, after, "壊れた縦横比でカメラが動いた");
    }

    /// 積むものが root の ViewCoordinates 1本であること。
    /// `query_view_coordinates_at_closest_ancestor` は view origin(`/`)から
    /// 親を辿るので、root に無いと fallback へ落ちる。
    #[test]
    fn scene_view_coordinates_chunk_sits_at_the_recording_root() {
        let chunk = scene_view_coordinates_chunk().expect("scene view coordinates chunk");
        assert_eq!(chunk.entity_path(), &EntityPath::root());
        assert!(
            chunk.components().keys().any(|component| {
                *component == re_sdk_types::archetypes::ViewCoordinates::descriptor_xyz().component
            }),
            "root chunk が ViewCoordinates を運んでいない"
        );
    }
}
