use std::time::Instant;

use crate::rerun_stage::{EmbeddedSpatialStage, StageTransformProjection};
use motolii_gpu::GpuCtx;
use motolii_render::RenderSession;
use motolii_ui::AppStageTransformEdit;

use super::RendererCore;
use super::host::HostTerminalLatch;
use super::present::create_timeline_resources;
use super::scrub::ScrubTimePump;
use super::types::{RenderStats, SceneKind, StageResources};

impl RendererCore {
    pub(crate) fn new(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
        scene: SceneKind,
    ) -> Result<Self, String> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|error| format!("request adapter: {error}"))?;

        let adapter_limits = adapter.limits();
        let max_texture_dimension_2d = adapter_limits.max_texture_dimension_2d;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Motolii RN native-component Stage"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter_limits,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .map_err(|error| format!("request device: {error}"))?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.clamp(1, max_texture_dimension_2d),
            height: height.clamp(1, max_texture_dimension_2d),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let stage = (scene == SceneKind::Stage)
            .then(|| {
                EmbeddedSpatialStage::new(&adapter, &device, &queue, config.format).map(|rerun| {
                    let gpu = GpuCtx::from_device_queue(device.clone(), queue.clone());
                    let session = RenderSession::new(&gpu);
                    StageResources {
                        rerun,
                        preview_active: false,
                        gpu,
                        session,
                        frame: None,
                    }
                })
            })
            .transpose()?;
        let timeline = (scene == SceneKind::Timeline)
            .then(|| create_timeline_resources(&device, format, config.width, config.height));

        let mut core = Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            config,
            stage,
            timeline,
            // 製品初期はfixture defaultではない。host空ならempty_host、warmup/presentがsnapshotを載せる。
            timeline_session: (scene == SceneKind::Timeline).then(TimelineSession::host_product),
            host_revision: None,
            host_handle: None,
            host_projection_generation: None,
            host_stage_geometry: None,
            host_stage_viewport: None,
            host_fps: None,
            stage_gizmo_pointer_active: false,
            scrubbing: false,
            scrub_time_pump: ScrubTimePump::new(),
            scrub_clock_start: Instant::now(),
            force_next_host_snapshot: false,
            host_projection_stamp: None,
            mount_warmup_done: false,
            host_terminal_latch: HostTerminalLatch::default(),
            scene,
            selected_object_index: -1,
            playhead: 0.0,
            frame: 0,
            stats: RenderStats::default(),
        };
        // mount完了直後・初回present前にshader/Skia初期化を先払い(F11 / B6)。
        core.run_mount_warmup();
        Ok(core)
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        let width = width.clamp(1, max_dimension);
        let height = height.clamp(1, max_dimension);
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        if self.scene == SceneKind::Timeline {
            self.timeline = Some(create_timeline_resources(
                &self.device,
                self.config.format,
                width,
                height,
            ));
        }
    }

    pub(crate) fn set_timeline_state(&mut self, selected_object_index: i32, playhead: f64) {
        let real = self
            .timeline_session
            .as_ref()
            .is_some_and(|session| session.scene.real);
        // realの選択正本はhost primary。RN props echoがnative選択を押し戻さない。
        if !real {
            self.selected_object_index = selected_object_index.max(-1);
        }
        // real sceneのplayhead正本はhost current_time。scrub中だけRN echoを受ける。
        if !real || self.scrubbing {
            self.playhead = playhead.clamp(0.0, 1.0);
        }
        if let Some(timeline) = &mut self.timeline {
            timeline.dirty = true;
        }
    }

    pub(crate) fn set_created_item(&mut self, item_id: &str) -> bool {
        self.stage
            .as_mut()
            .is_some_and(|stage| stage.rerun.set_created_item(item_id))
    }

    pub(crate) fn fit_stage_view(&mut self) -> bool {
        let Some(stage) = self.stage.as_mut() else {
            return false;
        };
        stage.rerun.fit_view(self.config.width, self.config.height)
    }

    pub(crate) fn set_stage_one_to_one(&mut self) -> bool {
        let Some(stage) = self.stage.as_mut() else {
            return false;
        };
        stage
            .rerun
            .set_one_to_one(self.config.width, self.config.height)
    }

    pub(crate) fn stage_transform_projection(&self) -> Option<StageTransformProjection> {
        self.stage
            .as_ref()
            .map(|stage| stage.rerun.transform_projection())
    }

    pub(crate) fn set_stage_transform_projection(
        &mut self,
        projection: StageTransformProjection,
    ) -> bool {
        let Some(stage) = self.stage.as_mut() else {
            return false;
        };
        let Some(layer_id) = stage.rerun.host_primary_layer_id().map(str::to_owned) else {
            return stage.rerun.set_transform_projection(projection);
        };
        let current = stage.rerun.transform_projection();
        let delta = [projection.x - current.x, projection.y - current.y];
        let rotate = (projection.rotation_z - current.rotation_z).to_radians();
        let edit = if delta[0].abs() > f64::EPSILON || delta[1].abs() > f64::EPSILON {
            AppStageTransformEdit::TranslateWorld(delta)
        } else if rotate.abs() > f64::EPSILON {
            AppStageTransformEdit::RotateZ(rotate)
        } else {
            return true;
        };
        let Some(expected_revision) = self
            .host_revision
            .as_deref()
            .and_then(|revision| revision.parse::<u64>().ok())
        else {
            self.restore_stage_preview("The live Document revision is unavailable");
            return false;
        };
        let result =
            crate::host_bridge::dispatch_commit_stage_transform(expected_revision, &layer_id, edit);
        let accepted = result.accepted;
        let message = result
            .feedback()
            .unwrap_or("Stage transform rejected")
            .to_owned();
        self.apply_terminal_stage_result(result);
        if accepted {
            true
        } else {
            self.restore_stage_preview(&message);
            false
        }
    }

    pub(crate) fn preview_stage_transform_from_app(
        &mut self,
        expected_revision: u64,
        layer_id: &str,
        edit: AppStageTransformEdit,
    ) -> Result<(), String> {
        let result = (|| {
            let geometry =
                crate::host_bridge::try_preview_stage_transform(expected_revision, layer_id, edit)?;
            let stage = self
                .stage
                .as_mut()
                .ok_or_else(|| "Stage renderer is unavailable".to_owned())?;
            if !stage.rerun.apply_host_stage_geometry(
                &geometry,
                self.config.width,
                self.config.height,
            ) {
                return Err("The preview path could not be projected".to_owned());
            }
            stage.preview_active = true;
            stage
                .rerun
                .set_feedback("Previewing Document transform", false);
            Ok(())
        })();
        if let Err(error) = &result {
            self.restore_stage_preview(error);
        }
        result
    }

    pub(crate) fn commit_stage_transform_from_app(
        &mut self,
        expected_revision: u64,
        layer_id: &str,
        edit: AppStageTransformEdit,
    ) -> Result<(), String> {
        let terminal =
            crate::host_bridge::dispatch_commit_stage_transform(expected_revision, layer_id, edit);
        let accepted = terminal.accepted;
        let message = terminal
            .feedback()
            .unwrap_or("Stage transform rejected")
            .to_owned();
        self.apply_terminal_stage_result(terminal);
        let result = if accepted { Ok(()) } else { Err(message) };
        match &result {
            Ok(()) => {
                if let Some(stage) = self.stage.as_mut() {
                    stage.preview_active = false;
                    stage
                        .rerun
                        .set_feedback("Transform applied · Undo available", false);
                }
            }
            Err(error) => self.restore_stage_preview(error),
        }
        result
    }

    pub(crate) fn cancel_stage_transform_from_app(&mut self) -> Result<(), String> {
        if self.stage.is_none() {
            return Err("Stage renderer is unavailable".to_owned());
        }
        self.restore_stage_preview("Transform cancelled · Document unchanged");
        Ok(())
    }
}
