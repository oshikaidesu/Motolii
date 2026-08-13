use std::time::Instant;

use crate::rerun_stage::{EmbeddedSpatialStage, StageGizmoAction, StageTransformProjection};
use crate::timeline_skia::{TimelinePointerPhase, TimelineScene, TimelineSession};
use motolii_gpu::GpuCtx;
use motolii_render::RenderSession;
use motolii_ui::{AppStageFrame, AppStageTransformEdit, host_render_frame_for_app};

const SET_TIME_THROTTLE_MS: u64 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScrubPointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug)]
struct ScrubTimePump {
    down_frame: Option<i64>,
    last_dispatch_ms: Option<u64>,
    sent_since_down: bool,
}

impl ScrubTimePump {
    fn new() -> Self {
        Self {
            down_frame: None,
            last_dispatch_ms: None,
            sent_since_down: false,
        }
    }

    fn is_active(&self) -> bool {
        self.down_frame.is_some()
    }

    fn should_send_throttled(&self, now_ms: u64) -> bool {
        self.last_dispatch_ms
            .is_none_or(|last| now_ms.saturating_sub(last) >= SET_TIME_THROTTLE_MS)
    }

    fn next_frame(
        &mut self,
        phase: ScrubPointerPhase,
        bar: f64,
        now_ms: u64,
        fps_num: i64,
        fps_den: i64,
    ) -> Option<i64> {
        if fps_num <= 0 || fps_den <= 0 {
            return None;
        }
        let frame = crate::host_bridge::frame_from_scrub_bar(bar, fps_num, fps_den);
        match phase {
            ScrubPointerPhase::Down => {
                self.down_frame = Some(frame);
                self.last_dispatch_ms = Some(now_ms);
                self.sent_since_down = true;
                Some(frame)
            }
            ScrubPointerPhase::Move => {
                if !self.should_send_throttled(now_ms) {
                    return None;
                }
                self.last_dispatch_ms = Some(now_ms);
                self.sent_since_down = true;
                Some(frame)
            }
            ScrubPointerPhase::Up => {
                self.last_dispatch_ms = Some(now_ms);
                self.sent_since_down = true;
                self.down_frame = None;
                Some(frame)
            }
            ScrubPointerPhase::Cancel => {
                let dispatch_frame = self.down_frame;
                self.down_frame = None;
                if self.sent_since_down {
                    self.sent_since_down = false;
                    return dispatch_frame;
                }
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum HostStageGeometryCommand {
    Apply(crate::host_bridge::HostStageGeometry),
    Clear,
    Noop,
}

fn host_stage_geometry_command(
    previous: Option<&crate::host_bridge::HostStageGeometry>,
    projection: Option<&crate::host_bridge::HostTimelineProjection>,
) -> HostStageGeometryCommand {
    let next = projection.and_then(|next| next.stage_geometry.as_ref());
    match (previous, next) {
        (Some(current), Some(next)) if current == next => HostStageGeometryCommand::Noop,
        (Some(_), Some(next)) => HostStageGeometryCommand::Apply(next.clone()),
        (None, Some(next)) => HostStageGeometryCommand::Apply(next.clone()),
        (Some(_), None) => HostStageGeometryCommand::Clear,
        (None, None) => HostStageGeometryCommand::Noop,
    }
}

fn stage_selection_commit(
    selected_entity_path: Option<&str>,
) -> crate::timeline_skia::TimelineSelectionCommit {
    selected_entity_path
        .and_then(crate::rerun_stage::host_layer_id_from_entity_path)
        .map_or(
            crate::timeline_skia::TimelineSelectionCommit::ClearSelection,
            |layer_id| crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
                layer_id: layer_id.to_owned(),
            },
        )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SceneKind {
    Stage,
    Timeline,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub frame_count: u64,
    pub last_cpu_us: u64,
    pub max_cpu_us: u64,
    pub vertex_bytes: u64,
    pub overlay_uploads: u64,
    pub overlay_last_us: u64,
    pub pointer_downs: u64,
    pub pointer_moves: u64,
    pub pointer_ups: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StagePointerButton {
    Primary,
    Secondary,
    Middle,
}

impl StagePointerButton {
    pub(crate) fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(Self::Primary),
            1 => Some(Self::Secondary),
            2 => Some(Self::Middle),
            _ => None,
        }
    }
}

struct TimelineResources {
    surface_texture: wgpu::Texture,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,
    pixels: Vec<u8>,
    dirty: bool,
}

struct StageResources {
    rerun: EmbeddedSpatialStage,
    preview_active: bool,
    gpu: GpuCtx,
    session: RenderSession,
    frame: Option<AppStageFrame>,
}

pub(crate) struct RendererCore {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    stage: Option<StageResources>,
    timeline: Option<TimelineResources>,
    timeline_session: Option<TimelineSession>,
    /// Host snapshotのrevision。変化時だけsceneを差し替える。
    host_revision: Option<String>,
    /// set_timeはrevisionを進めないため、playhead追従はgenerationで見る。
    host_projection_generation: Option<String>,
    host_stage_geometry: Option<crate::host_bridge::HostStageGeometry>,
    /// host 投影メッシュ適用時の viewport（aspect 再適用判定）。
    host_stage_viewport: Option<(u32, u32)>,
    host_fps: Option<(i64, i64)>,
    stage_gizmo_pointer_active: bool,
    scrubbing: bool,
    scrub_time_pump: ScrubTimePump,
    scrub_clock_start: Instant,
    force_next_host_snapshot: bool,
    /// F9: 前回読んだhost stamp。(revision, generation)。未取得はNone。
    host_projection_stamp: Option<(u64, u64)>,
    /// F11: mount時warm-upを1回だけ先払いしたか。resizeでは再実行しない。
    mount_warmup_done: bool,
    scene: SceneKind,
    selected_object_index: i32,
    playhead: f64,
    frame: u64,
    stats: RenderStats,
}

fn timeline_scene_from_projection(
    existing_scene: &TimelineScene,
    projection: &crate::host_bridge::HostTimelineProjection,
) -> TimelineScene {
    let fallback_song_bars = (10.0f64 / crate::timeline_skia::SECONDS_PER_BAR) as f32;
    let song_bars = projection
        .timeline_duration
        .and_then(|(num, den)| {
            if den <= 0 || num < 0 {
                None
            } else {
                Some((num as f64 / den as f64 / crate::timeline_skia::SECONDS_PER_BAR) as f32)
            }
        })
        .filter(|bars| bars.is_finite())
        .unwrap_or(fallback_song_bars);
    let layers = crate::host_bridge::snapshot_layers_from_projection(projection);
    let mut scene = TimelineScene::from_snapshot_with_song_bars(
        &layers,
        projection.primary_layer_id.as_deref(),
        song_bars,
    );
    if let Some((num, den)) = projection.fps {
        scene = scene.with_fps(num, den);
    }
    if let Some(timeline_layers) = &projection.timeline_layers {
        scene.apply_layer_mute_solo(
            timeline_layers
                .iter()
                .map(|layer| (layer.visible, layer.solo, layer.effects.len())),
        );
    }
    // real同士の差し替えではlocal viewを維持。fixture→real初回はfrom_snapshotの0..song_bars。
    if existing_scene.real {
        scene.view_a = existing_scene.view_a;
        scene.view_b = existing_scene.view_b;
        let span = scene.view_b - scene.view_a;
        if scene.view_a < 0.0 {
            scene.view_a = 0.0;
            scene.view_b = span.min(scene.song_bars);
        }
        if scene.view_b > scene.song_bars {
            scene.view_b = scene.song_bars;
            scene.view_a = (scene.song_bars - span).max(0.0);
        }
        // revision再投影でkeyのselが落ちるとDeleteがlayer削除へ化ける。key_id一致で引き継ぐ。
        if let Some((layer_id, key_id)) = crate::timeline_skia::selected_real_key(existing_scene) {
            if projection.primary_layer_id.as_deref() == Some(layer_id.as_str()) {
                let _ = crate::timeline_skia::restore_key_selection(
                    &mut scene,
                    layer_id.as_str(),
                    key_id,
                );
            }
        }
    }
    scene
}

fn timeline_projection_selected_flat(
    projection: &crate::host_bridge::HostTimelineProjection,
) -> i32 {
    let Some(primary) = projection.primary_layer_id.as_deref() else {
        return -1;
    };
    let position = projection.timeline_layers.as_ref().map_or_else(
        || {
            projection
                .bounds
                .iter()
                .position(|(layer_id, _)| layer_id == primary)
        },
        |layers| layers.iter().position(|layer| layer.layer_id == primary),
    );
    position
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}

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
        match crate::host_bridge::try_commit_stage_transform(expected_revision, &layer_id, edit) {
            Ok(()) => {
                self.force_next_host_snapshot = true;
                true
            }
            Err(error) => {
                self.restore_stage_preview(&error);
                false
            }
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
        let result =
            crate::host_bridge::try_commit_stage_transform(expected_revision, layer_id, edit);
        match &result {
            Ok(()) => {
                if let Some(stage) = self.stage.as_mut() {
                    stage.preview_active = false;
                    stage
                        .rerun
                        .set_feedback("Transform applied · Undo available", false);
                }
                self.force_next_host_snapshot = true;
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

    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        let Some(session) = &self.timeline_session else {
            return None;
        };
        crate::timeline_skia::hit_test(&session.scene, self.config.width, self.config.height, x, y)
    }

    /// hover位置のhit種→cursor。clip drag中はclosedHand。
    pub(crate) fn timeline_hover_cursor(&self, x: f64, y: f64) -> i32 {
        let Some(session) = &self.timeline_session else {
            return crate::timeline_skia::CursorKind::Arrow.as_i32();
        };
        let hit = crate::timeline_skia::timeline_hover_hit(
            &session.scene,
            self.playhead,
            self.config.width,
            self.config.height,
            x,
            y,
        );
        let clip_dragging = matches!(
            session.gesture_kind_for_cursor(),
            Some(crate::timeline_skia::CursorDragKind::Clip)
        );
        crate::timeline_skia::cursor_for_timeline_hover(hit, clip_dragging).as_i32()
    }

    /// Stage上のlayer hover → cursor。x/yはtimeline同様の物理座標。
    pub(crate) fn stage_hover_cursor(&self, x: f64, y: f64) -> i32 {
        let dragging = self.stage_gizmo_pointer_active;
        let over = self
            .stage
            .as_ref()
            .is_some_and(|stage| stage.rerun.gizmo_wants_pointer(x, y));
        crate::timeline_skia::cursor_for_stage_hover(over, dragging).as_i32()
    }

    /// Timeline pointer。戻り値trueはselection/playhead変化(feedback対象)。
    pub(crate) fn timeline_pointer(
        &mut self,
        phase: PointerPhase,
        x: f64,
        y: f64,
        modifiers: u32,
    ) -> Option<(i32, f64)> {
        // 並行gesture防御: Downで進行中scrub pumpを先に解除してから新gestureへ。
        if matches!(phase, PointerPhase::Down) {
            self.scrub_time_pump = ScrubTimePump::new();
            self.scrubbing = false;
        }
        let tl_phase = match phase {
            PointerPhase::Down => {
                self.stats.pointer_downs += 1;
                TimelinePointerPhase::Down
            }
            PointerPhase::Move => {
                self.stats.pointer_moves += 1;
                TimelinePointerPhase::Move
            }
            PointerPhase::Up => {
                self.stats.pointer_ups += 1;
                TimelinePointerPhase::Up
            }
            PointerPhase::Cancel => TimelinePointerPhase::Cancel,
        };
        let width = self.config.width;
        let height = self.config.height;
        let (is_real, outcome) = {
            let Some(session) = &mut self.timeline_session else {
                return None;
            };
            let is_real = session.scene.real;
            let outcome = session.pointer(
                &mut self.selected_object_index,
                &mut self.playhead,
                width,
                height,
                tl_phase,
                x,
                y,
                modifiers,
            );
            (is_real, outcome)
        };
        if outcome.dirty {
            if let Some(timeline) = &mut self.timeline {
                timeline.dirty = true;
            }
        }
        if is_real {
            let maybe_scrub_playhead =
                outcome
                    .scrub_playhead
                    .or(if matches!(phase, PointerPhase::Cancel) {
                        if self.scrub_time_pump.is_active() {
                            Some(self.playhead)
                        } else {
                            None
                        }
                    } else {
                        None
                    });
            if let Some(scrub_playhead) = maybe_scrub_playhead {
                self.dispatch_set_time_for_scrub(
                    match phase {
                        PointerPhase::Down => ScrubPointerPhase::Down,
                        PointerPhase::Move => ScrubPointerPhase::Move,
                        PointerPhase::Up => ScrubPointerPhase::Up,
                        PointerPhase::Cancel => ScrubPointerPhase::Cancel,
                    },
                    scrub_playhead,
                );
            }
            if matches!(phase, PointerPhase::Up) && outcome.edit_commit.is_some() {
                self.force_next_host_snapshot = true;
            }
            if let Some(commit) = outcome.edit_commit {
                let accepted = crate::host_bridge::try_dispatch_timeline_edit(&commit);
                if !accepted {
                    // 拒否時は次snapshotで幾何を戻す。dirty再描画だけ先に立てる。
                    if let Some(timeline) = &mut self.timeline {
                        timeline.dirty = true;
                    }
                    self.force_next_host_snapshot = true;
                }
            }
            if let Some(commit) = outcome.selection_commit {
                let _ = Self::dispatch_timeline_selection(&commit);
            }
            self.scrubbing = self.scrub_time_pump.is_active();
        }
        if matches!(phase, PointerPhase::Down) {
            crate::host_bridge::set_timeline_interacting(true);
        } else if matches!(phase, PointerPhase::Up | PointerPhase::Cancel) {
            crate::host_bridge::set_timeline_interacting(false);
            self.scrubbing = false;
            self.scrub_time_pump = ScrubTimePump::new();
        }
        outcome
            .feedback
            .then_some((self.selected_object_index, self.playhead))
    }

    fn now_ms(&self) -> u64 {
        self.scrub_clock_start.elapsed().as_millis() as u64
    }

    fn dispatch_set_time_for_scrub(&mut self, phase: ScrubPointerPhase, playhead: f64) {
        let Some((fps_num, fps_den)) = self.host_fps else {
            return;
        };
        if fps_num <= 0 || fps_den <= 0 {
            return;
        }
        let song_bars = self
            .timeline_session
            .as_ref()
            .map(|session| session.scene.song_bars)
            .unwrap_or(crate::timeline_skia::SONG_BARS);
        let bar = playhead.clamp(0.0, 1.0) * f64::from(song_bars);
        let Some(frame) =
            self.scrub_time_pump
                .next_frame(phase, bar, self.now_ms(), fps_num, fps_den)
        else {
            return;
        };
        let _ = crate::host_bridge::try_dispatch_set_time(frame);
    }

    /// Timeline scroll/pinch。戻り値trueは視覚変化(dirty)。feedbackなし。
    pub(crate) fn timeline_scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        let Some(session) = &mut self.timeline_session else {
            return false;
        };
        let dirty = session.scroll(
            self.config.width,
            self.config.height,
            delta_x,
            delta_y,
            magnification,
            modifiers,
            x,
            y,
        );
        if dirty {
            if let Some(timeline) = &mut self.timeline {
                timeline.dirty = true;
            }
        }
        dirty
    }

    /// Timeline Delete/Backspace: 選択keyがあれば remove、なければ layer削除。
    pub(crate) fn timeline_keymap_delete(&self) -> bool {
        let Some(session) = &self.timeline_session else {
            return crate::host_bridge::try_dispatch_keymap("delete_layer");
        };
        crate::host_bridge::try_timeline_keymap_delete(&session.scene)
    }

    pub(crate) fn stats(&self) -> RenderStats {
        self.stats
    }

    pub(crate) fn stage_pointer(
        &mut self,
        phase: PointerPhase,
        button: StagePointerButton,
        modifiers: u32,
        x: f64,
        y: f64,
    ) {
        let gizmo_hit = button == StagePointerButton::Primary
            && matches!(phase, PointerPhase::Down)
            && self
                .stage
                .as_ref()
                .is_some_and(|stage| stage.rerun.gizmo_wants_pointer(x, y));
        if (self.stage_gizmo_pointer_active && button == StagePointerButton::Primary) || gizmo_hit {
            self.stage_gizmo_pointer_active =
                !matches!(phase, PointerPhase::Up | PointerPhase::Cancel);
            if let Some(stage) = &mut self.stage {
                // gizmo capture中のprimaryはRerun camera/pickingへ二重配送しない。
                stage.rerun.gizmo_pointer(phase, x, y);
            }
        } else if let Some(stage) = &mut self.stage {
            stage.rerun.pointer(phase, button, modifiers, x, y);
        } else {
            return;
        }
        match phase {
            PointerPhase::Down => self.stats.pointer_downs += 1,
            PointerPhase::Move => self.stats.pointer_moves += 1,
            PointerPhase::Up => self.stats.pointer_ups += 1,
            PointerPhase::Cancel => {}
        }
    }

    pub(crate) fn stage_scroll(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        magnification: f64,
        modifiers: u32,
        x: f64,
        y: f64,
    ) -> bool {
        self.stage.as_mut().is_some_and(|stage| {
            stage
                .rerun
                .scroll(delta_x, delta_y, magnification, modifiers, x, y)
        })
    }

    fn process_stage_gizmo_action(&mut self) {
        let action = self
            .stage
            .as_mut()
            .and_then(|stage| stage.rerun.take_gizmo_action());
        let Some(action) = action else {
            return;
        };
        let Some(expected_revision) = self
            .host_revision
            .as_deref()
            .and_then(|revision| revision.parse::<u64>().ok())
        else {
            self.restore_stage_preview("The live Document revision is unavailable");
            return;
        };

        match action {
            StageGizmoAction::Preview { layer_id, edit } => {
                let _ = self.preview_stage_transform_from_app(expected_revision, &layer_id, edit);
            }
            StageGizmoAction::Commit { layer_id, edit } => {
                let _ = self.commit_stage_transform_from_app(expected_revision, &layer_id, edit);
            }
            StageGizmoAction::Cancel => {
                let _ = self.cancel_stage_transform_from_app();
            }
        }
    }

    fn restore_stage_preview(&mut self, message: &str) {
        let Some(stage) = self.stage.as_mut() else {
            return;
        };
        stage.preview_active = false;
        if let Some(geometry) = self.host_stage_geometry.as_ref() {
            let _ = stage.rerun.apply_host_stage_geometry(
                geometry,
                self.config.width,
                self.config.height,
            );
        }
        stage.rerun.set_feedback(message, true);
    }

    pub(crate) fn render(&mut self) -> Result<(), String> {
        let started = Instant::now();
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err("surface lost".into()),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface validation failure".into());
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        match self.scene {
            SceneKind::Stage => {
                self.process_stage_gizmo_action();
                self.present_stage(&view)?;
            }
            SceneKind::Timeline => {
                // 可視frameでもhost snapshotを載せる。warmup未達・stamp更新の両方を拾う。
                self.sync_host_timeline_projection();
                self.render_timeline(&view);
            }
        }

        output.present();
        self.frame = self.frame.wrapping_add(1);
        let cpu_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        self.stats.frame_count = self.stats.frame_count.wrapping_add(1);
        self.stats.last_cpu_us = cpu_us;
        self.stats.max_cpu_us = self.stats.max_cpu_us.max(cpu_us);
        self.stats.vertex_bytes = 0;
        Ok(())
    }

    fn dispatch_timeline_selection(commit: &crate::timeline_skia::TimelineSelectionCommit) -> bool {
        crate::host_bridge::try_dispatch_timeline_selection(commit)
    }

    /// F9: stamp不変かつforceでなく初回でもないtickはfull JSON読みを飛ばす。
    /// 引数 `current` はtick冒頭で1回だけ採るstamp。gateと保存で同じ値を使う。
    pub(crate) fn host_snapshot_read_needed(
        last: Option<(u64, u64)>,
        current: Option<(u64, u64)>,
        force: bool,
    ) -> bool {
        if force || last.is_none() {
            return true;
        }
        match current {
            Some(stamp) => last != Some(stamp),
            // stamp取得失敗時は従来どおりfull読みへ落とす(挙動維持)。
            None => true,
        }
    }

    /// mount時1回だけ。resize/scene切替では呼ばない。
    fn run_mount_warmup(&mut self) {
        if self.mount_warmup_done {
            return;
        }
        self.mount_warmup_done = true;
        let started = Instant::now();
        let ok = match self.scene {
            SceneKind::Stage => self.warmup_stage_offscreen(),
            SceneKind::Timeline => self.warmup_timeline_skia(),
        };
        let warmup_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        // telemetry 1行。ObjC ABIは触らない。
        eprintln!(
            "[MotoliiRenderProbe] warmup_us={warmup_us} scene={:?} ok={}",
            self.scene,
            u8::from(ok),
        );
    }

    fn warmup_stage_offscreen(&mut self) -> bool {
        let width = self.config.width.max(1);
        let height = self.config.height.max(1);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Motolii stage mount warmup"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // 既存Stage描画経路をpresent無しで1回(submit+poll)。
        let ok = self.present_stage(&view).is_ok();
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        ok
    }

    fn warmup_timeline_skia(&mut self) -> bool {
        // 初回Skia画素をhost snapshotにする。未syncだとempty/fixtureが製品mountに残る。
        self.sync_host_timeline_projection();
        let width = self.config.width.max(1);
        let height = self.config.height.max(1);
        let scene = self
            .timeline_session
            .as_ref()
            .map(|session| session.scene.clone())
            .unwrap_or_else(TimelineScene::empty_host);
        let playhead = self.playhead;
        let selected = self.selected_object_index;
        let Some(timeline) = self.timeline.as_mut() else {
            return true;
        };
        // font/typeface/surface初期化を実サイズbufferへ先払い。
        crate::timeline_skia::draw_timeline(
            &scene,
            &mut timeline.pixels,
            width,
            height,
            playhead,
            selected,
        );
        // 次の可視frameでも再rasterしてよい(dirty維持)。warm-upは初期化コスト払いが目的。
        timeline.dirty = true;
        true
    }

    fn sync_host_timeline_projection(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let read_stamp = crate::host_bridge::try_read_projection_stamp();
            let force_scene = self.force_next_host_snapshot;
            if !Self::host_snapshot_read_needed(self.host_projection_stamp, read_stamp, force_scene)
            {
                return;
            }
            let Some(projection) = crate::host_bridge::try_read_timeline_projection() else {
                self.host_projection_stamp = None;
                return;
            };
            self.host_projection_stamp = read_stamp;
            self.host_fps = projection.fps;
            let revision_changed =
                self.host_revision.as_deref() != Some(projection.revision.as_str());
            let generation_changed = self.host_projection_generation.as_deref()
                != Some(projection.projection_generation.as_str());
            let primary_changed =
                self.selected_object_index != timeline_projection_selected_flat(&projection);
            let has_active_gesture = self
                .timeline_session
                .as_ref()
                .is_some_and(TimelineSession::has_active_gesture);
            let should_reproject =
                revision_changed || primary_changed || (force_scene && !has_active_gesture);
            if should_reproject {
                let Some(session) = &mut self.timeline_session else {
                    return;
                };
                // 進行中gestureは復元せず破棄。古いband indexでのpanicを防ぐ。
                let gesture_dirty = session.discard_active_gesture();
                if gesture_dirty {
                    self.scrub_time_pump = ScrubTimePump::new();
                    self.scrubbing = false;
                    crate::host_bridge::set_timeline_interacting(false);
                }
                let scene = timeline_scene_from_projection(&session.scene, &projection);
                self.selected_object_index = scene.selected_flat;
                session.scene = scene;
                if revision_changed {
                    self.host_revision = Some(projection.revision);
                }
                if let Some(timeline) = &mut self.timeline {
                    timeline.dirty = true;
                }
                if force_scene {
                    self.force_next_host_snapshot = false;
                }
            }
            if !self.scrubbing && (generation_changed || revision_changed || force_scene) {
                let song_bars = self
                    .timeline_session
                    .as_ref()
                    .map(|session| session.scene.song_bars)
                    .unwrap_or(crate::timeline_skia::SONG_BARS);
                let next = crate::host_bridge::playhead_from_current_time(
                    projection.current_time.0,
                    projection.current_time.1,
                    song_bars,
                );
                if (self.playhead - next).abs() > f64::EPSILON {
                    self.playhead = next;
                    if let Some(timeline) = &mut self.timeline {
                        timeline.dirty = true;
                    }
                }
            }
            self.host_projection_generation = Some(projection.projection_generation);
        }
    }

    fn present_stage(&mut self, view: &wgpu::TextureView) -> Result<(), String> {
        self.sync_host_stage_geometry();
        self.sync_host_stage_frame();
        let width = self.config.width;
        let height = self.config.height;
        let Some(stage) = self.stage.as_mut() else {
            return Ok(());
        };
        let StageResources {
            rerun, frame, gpu, ..
        } = stage;
        let selected_entity_path = rerun.render(
            &gpu.device,
            &gpu.queue,
            view,
            width,
            height,
            frame.as_ref().map(|frame| &frame.texture),
        )?;
        if let Some(selected_entity_path) = selected_entity_path {
            let commit = stage_selection_commit(selected_entity_path.as_deref());
            let _ = Self::dispatch_timeline_selection(&commit);
        }
        Ok(())
    }

    fn sync_host_stage_frame(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let Some(stage) = self.stage.as_mut() else {
                return;
            };
            if stage.preview_active {
                return;
            }
            let Some(handle) = crate::host_bridge::try_host_handle() else {
                return;
            };
            let _ =
                host_render_frame_for_app(handle, &stage.gpu, &mut stage.session, &mut stage.frame);
        }
    }

    fn sync_host_stage_geometry(&mut self) {
        #[cfg(target_os = "macos")]
        {
            if self.stage.is_none() {
                return;
            }
            if self
                .stage
                .as_ref()
                .is_some_and(|stage| stage.preview_active)
            {
                return;
            }
            let viewport = (self.config.width, self.config.height);
            // stampゲートはstage/timeline共通。forceと初回はfull読み。
            let read_stamp = crate::host_bridge::try_read_projection_stamp();
            let force = self.force_next_host_snapshot;
            let read_needed =
                Self::host_snapshot_read_needed(self.host_projection_stamp, read_stamp, force);
            let projection = if read_needed {
                let projection = crate::host_bridge::try_read_timeline_projection();
                if projection.is_some() {
                    self.host_projection_stamp = read_stamp;
                    if force {
                        // Stageはforceをscene再投影に使わないが、消費して無限full読みを防ぐ。
                        self.force_next_host_snapshot = false;
                    }
                } else {
                    self.host_projection_stamp = None;
                }
                projection
            } else {
                None
            };
            let stage = self.stage.as_mut().expect("stage present");
            let command = match projection {
                Some(ref projection) => {
                    self.host_revision = Some(projection.revision.clone());
                    self.host_projection_generation =
                        Some(projection.projection_generation.clone());
                    stage
                        .rerun
                        .set_host_primary_layer_id(projection.primary_layer_id.clone());
                    host_stage_geometry_command(self.host_stage_geometry.as_ref(), Some(projection))
                }
                None if read_needed => {
                    stage.rerun.set_host_primary_layer_id(None);
                    host_stage_geometry_command(self.host_stage_geometry.as_ref(), None)
                }
                None => {
                    // stamp不変: 既存geometryのviewport再適用だけ。
                    HostStageGeometryCommand::Noop
                }
            };
            match command {
                HostStageGeometryCommand::Apply(geometry) => {
                    if stage
                        .rerun
                        .apply_host_stage_geometry(&geometry, viewport.0, viewport.1)
                    {
                        self.host_stage_geometry = Some(geometry);
                        self.host_stage_viewport = Some(viewport);
                    }
                }
                HostStageGeometryCommand::Clear => {
                    if stage.rerun.clear_host_projection() {
                        self.host_stage_geometry = None;
                        self.host_stage_viewport = None;
                        stage.preview_active = false;
                    }
                }
                HostStageGeometryCommand::Noop => {
                    // geometry不変でもviewport aspectが変わったら再投影する。
                    if self.host_stage_viewport != Some(viewport) {
                        if let Some(geometry) = self.host_stage_geometry.clone() {
                            if stage
                                .rerun
                                .apply_host_stage_geometry(&geometry, viewport.0, viewport.1)
                            {
                                self.host_stage_viewport = Some(viewport);
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_timeline(&mut self, view: &wgpu::TextureView) {
        let needs_raster = self.timeline.as_ref().is_some_and(|t| t.dirty);
        if needs_raster {
            let scene = self
                .timeline_session
                .as_ref()
                .expect("timeline session")
                .scene
                .clone();
            let width = self.config.width;
            let height = self.config.height;
            let playhead = self.playhead;
            let selected = self.selected_object_index;
            let timeline = self.timeline.as_mut().expect("timeline resources");
            let raster_started = Instant::now();
            crate::timeline_skia::draw_timeline(
                &scene,
                &mut timeline.pixels,
                width,
                height,
                playhead,
                selected,
            );
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &timeline.surface_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &timeline.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            timeline.dirty = false;
            self.stats.overlay_uploads += 1;
            self.stats.overlay_last_us = raster_started.elapsed().as_micros() as u64;
        }

        let timeline = self.timeline.as_mut().expect("timeline resources");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Motolii Skia timeline"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Skia timeline blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
            pass.set_pipeline(&timeline.blit_pipeline);
            pass.set_bind_group(0, &timeline.blit_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
    }
}

fn create_timeline_resources(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> TimelineResources {
    let surface_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Motolii Skia timeline raster"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Motolii Skia timeline blit shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @group(0) @binding(0) var raster:texture_2d<f32>; @group(0) @binding(1) var samp:sampler;
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn fs(i:O)->@location(0) vec4<f32> { return vec4(textureSample(raster,samp,i.uv).rgb,1.); }
        "#
            .into(),
        ),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Motolii Skia timeline blit layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Motolii Skia timeline blit pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &surface_texture.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    TimelineResources {
        surface_texture,
        blit_pipeline,
        blit_bind_group,
        pixels: vec![0; width as usize * height as usize * 4],
        dirty: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_bridge::frame_from_scrub_bar;

    #[test]
    fn stage_pointer_buttons_accept_only_rerun_standard_buttons() {
        assert_eq!(
            StagePointerButton::from_raw(0),
            Some(StagePointerButton::Primary)
        );
        assert_eq!(
            StagePointerButton::from_raw(1),
            Some(StagePointerButton::Secondary)
        );
        assert_eq!(
            StagePointerButton::from_raw(2),
            Some(StagePointerButton::Middle)
        );
        assert_eq!(StagePointerButton::from_raw(3), None);
    }

    #[test]
    fn rerun_entity_selection_remaps_to_existing_document_selection_intent() {
        assert_eq!(
            stage_selection_commit(Some("motolii/document/layers/42/fill")),
            crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
                layer_id: "42".into()
            }
        );
        assert_eq!(
            stage_selection_commit(Some("motolii/document/layers/42/path")),
            crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
                layer_id: "42".into()
            }
        );
        assert_eq!(
            stage_selection_commit(Some("motolii/document/frame")),
            crate::timeline_skia::TimelineSelectionCommit::ClearSelection
        );
        assert_eq!(
            stage_selection_commit(None),
            crate::timeline_skia::TimelineSelectionCommit::ClearSelection
        );
    }

    #[test]
    fn timeline_view_is_preserved_when_revision_changes() {
        let mut scene = TimelineScene::from_snapshot(
            &[crate::timeline_skia::SnapshotLayerInput {
                layer_id: "L1".into(),
                display_name: "Layer 1".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![],
            }],
            Some("L1"),
        );
        scene.view_a = 1.0;
        scene.view_b = 4.0;
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "r1".into(),
            projection_generation: "0".into(),
            primary_layer_id: Some("L2".into()),
            current_time: (0, 1),
            timeline_duration: Some((10, 1)),
            fps: None,
            bounds: vec![
                ("L1".into(), "Layer 1".into()),
                ("L2".into(), "Layer 2".into()),
            ],
            timeline_layers: None,
            stage_geometry: None,
        };
        let rebuilt = timeline_scene_from_projection(&scene, &projection);
        assert_eq!(rebuilt.view_a, 1.0);
        assert_eq!(rebuilt.view_b, 4.0);
        assert_eq!(rebuilt.selected_flat, 1);
        assert_eq!(timeline_projection_selected_flat(&projection), 1);
        let expected_song_bars = 10.0_f32 / crate::timeline_skia::SECONDS_PER_BAR as f32;
        assert!((rebuilt.song_bars - expected_song_bars).abs() < 1e-6);

        let mut missing = projection.clone();
        missing.primary_layer_id = Some("outside-truncated-projection".into());
        assert_eq!(timeline_projection_selected_flat(&missing), -1);
    }

    #[test]
    fn timeline_projection_scales_song_bars_from_duration_10_and_40_seconds() {
        let existing = TimelineScene::default();

        for (duration_num, duration_den, expected_song_bars) in
            [(10_i64, 1_i64, 10.0_f32), (40_i64, 1_i64, 40.0_f32)]
        {
            let projection = crate::host_bridge::HostTimelineProjection {
                revision: "r0".into(),
                projection_generation: "0".into(),
                primary_layer_id: Some("L1".into()),
                current_time: (0, 1),
                timeline_duration: Some((duration_num, duration_den)),
                fps: Some((30, 1)),
                bounds: vec![("L1".to_string(), "Layer 1".to_string())],
                timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
                    layer_id: "L1".into(),
                    display_name: "Layer 1".into(),
                    start_secs: 0.0,
                    duration_secs: duration_num as f64 / duration_den as f64,
                    position_keys: vec![],
                    param_keys: vec![],
                    effects: vec![],
                    effects_truncated: false,
                    source_params: vec![],
                    source_params_truncated: false,
                    visible: true,
                    solo: false,
                }]),
                stage_geometry: None,
            };

            let rebuilt = timeline_scene_from_projection(&existing, &projection);
            assert_eq!(rebuilt.selected_flat, 0);
            assert!((rebuilt.song_bars - expected_song_bars).abs() < 1e-6);
            assert!((rebuilt.view_a - 0.0).abs() < 1e-6);
            assert!((rebuilt.view_b - rebuilt.song_bars).abs() < 1e-6);
        }
    }

    #[test]
    fn timeline_projection_short_duration_does_not_panic_and_sets_span_to_duration() {
        let existing = TimelineScene::default();
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "r2".into(),
            projection_generation: "0".into(),
            primary_layer_id: Some("L1".into()),
            current_time: (0, 1),
            timeline_duration: Some((2, 1)),
            fps: Some((30, 1)),
            bounds: vec![("L1".into(), "Layer 1".into())],
            timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
                layer_id: "L1".into(),
                display_name: "Layer 1".into(),
                start_secs: 0.0,
                duration_secs: 2.0,
                position_keys: vec![],
                param_keys: vec![],
                effects: vec![],
                effects_truncated: false,
                source_params: vec![],
                source_params_truncated: false,
                visible: true,
                solo: false,
            }]),
            stage_geometry: None,
        };

        let rebuilt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            timeline_scene_from_projection(&existing, &projection)
        }));
        assert!(rebuilt.is_ok(), "short duration must not panic");
        let rebuilt = rebuilt.expect("short duration must not panic");
        assert!((rebuilt.song_bars - 2.0).abs() < 1e-6);
        assert_eq!(rebuilt.view_a, 0.0);
        assert_eq!(rebuilt.view_b, 2.0);
    }

    #[test]
    fn fixture_to_real_projection_resets_view_to_song_bars() {
        let existing = TimelineScene::default();
        assert!(!existing.real);
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "r1".into(),
            projection_generation: "0".into(),
            primary_layer_id: Some("L1".into()),
            current_time: (0, 1),
            fps: Some((30, 1)),
            timeline_duration: Some((10, 1)),
            bounds: vec![("L1".into(), "Layer 1".into())],
            timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
                layer_id: "L1".into(),
                display_name: "Layer 1".into(),
                start_secs: 0.0,
                duration_secs: 10.0,
                position_keys: vec![],
                param_keys: vec![],
                effects: vec![],
                effects_truncated: false,
                source_params: vec![],
                source_params_truncated: false,
                visible: true,
                solo: false,
            }]),
            stage_geometry: None,
        };
        let rebuilt = timeline_scene_from_projection(&existing, &projection);
        assert!(rebuilt.real);
        assert!((rebuilt.view_a - 0.0).abs() < 1e-6);
        assert!((rebuilt.view_b - rebuilt.song_bars).abs() < 1e-6);
        assert!((rebuilt.song_bars - 10.0).abs() < 1e-6);
    }

    #[test]
    fn product_timeline_session_starts_empty_host_not_fixture() {
        let session = TimelineSession::host_product();
        assert!(session.scene.real);
        assert_eq!(session.scene.band_count(), 0);
        let fixture = TimelineScene::default();
        assert!(!fixture.real);
        assert!(fixture.band_count() > 0);
    }

    #[test]
    fn empty_host_projection_clears_fixture_bands() {
        let existing = TimelineScene::default();
        assert!(!existing.real);
        assert!(existing.band_count() > 0);
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "0".into(),
            projection_generation: "0".into(),
            primary_layer_id: None,
            current_time: (0, 1),
            fps: Some((30, 1)),
            timeline_duration: Some((10, 1)),
            bounds: vec![],
            timeline_layers: Some(vec![]),
            stage_geometry: Some(crate::host_bridge::HostStageGeometry {
                layers: vec![],
                layers_truncated: false,
            }),
        };
        let rebuilt = timeline_scene_from_projection(&existing, &projection);
        assert!(rebuilt.real);
        assert_eq!(rebuilt.band_count(), 0);
        assert!(rebuilt.clip0_layer_id(0).is_none());
    }

    #[test]
    fn host_projection_exposes_the_same_layer_id() {
        let existing = TimelineScene::empty_host();
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "1".into(),
            projection_generation: "1".into(),
            primary_layer_id: Some("42".into()),
            current_time: (0, 1),
            fps: Some((30, 1)),
            timeline_duration: Some((10, 1)),
            bounds: vec![("42".into(), "Rectangle".into())],
            timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
                layer_id: "42".into(),
                display_name: "Rectangle".into(),
                start_secs: 0.0,
                duration_secs: 10.0,
                position_keys: vec![],
                param_keys: vec![],
                effects: vec![],
                effects_truncated: false,
                source_params: vec![],
                source_params_truncated: false,
                visible: true,
                solo: false,
            }]),
            stage_geometry: Some(crate::host_bridge::HostStageGeometry {
                layers: vec![crate::host_bridge::HostStageGeometryLayer {
                    layer_id: "42".into(),
                    corners: [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
                    position: [0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                }],
                layers_truncated: false,
            }),
        };
        let rebuilt = timeline_scene_from_projection(&existing, &projection);
        assert!(rebuilt.real);
        assert_eq!(rebuilt.band_count(), 1);
        assert_eq!(rebuilt.clip0_layer_id(0), Some("42"));
        assert_eq!(rebuilt.selected_flat, 0);
        let apply = host_stage_geometry_command(None, Some(&projection));
        match apply {
            HostStageGeometryCommand::Apply(geometry) => {
                assert_eq!(geometry.layers.len(), 1);
                assert_eq!(geometry.layers[0].layer_id, "42");
            }
            other => panic!("expected Apply, got {other:?}"),
        }
    }

    #[test]
    fn empty_host_stage_geometry_applies_instead_of_leaving_fixture() {
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "0".into(),
            projection_generation: "0".into(),
            primary_layer_id: None,
            current_time: (0, 1),
            fps: None,
            timeline_duration: Some((10, 1)),
            bounds: vec![],
            timeline_layers: Some(vec![]),
            stage_geometry: Some(crate::host_bridge::HostStageGeometry {
                layers: vec![],
                layers_truncated: false,
            }),
        };
        let apply = host_stage_geometry_command(None, Some(&projection));
        match apply {
            HostStageGeometryCommand::Apply(geometry) => {
                assert!(geometry.layers.is_empty());
            }
            other => panic!("empty host must Apply empty geometry, got {other:?}"),
        }
    }

    #[test]
    fn discard_gesture_on_scene_rebuild_leaves_no_active_gesture() {
        let mut session = TimelineSession::default();
        session.scene = TimelineScene::from_snapshot(
            &[crate::timeline_skia::SnapshotLayerInput {
                layer_id: "clip-real".into(),
                display_name: "clip".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![],
            }],
            None,
        );
        let mut selected = -1;
        // clip中心(bar1)からplayheadを離す — F1でplayhead優先になると選択がscrubになる。
        let mut playhead = 0.0;
        let x = 202.0 + (1.0f64 / 5.0) * (1240.0 - 202.0 - 6.0);
        let y = 66.5;
        let down = session.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert!(down.selection_commit.is_some() || selected >= 0);
        assert!(session.discard_active_gesture());
        // 差し替え後にUpしてもdispatchなし(gesture無し)。
        let up = session.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            x,
            y,
            0,
        );
        assert!(up.edit_commit.is_none());
    }

    #[test]
    fn scrub_time_pump_throttle_moves_and_always_dispatches_release() {
        let mut pump = ScrubTimePump::new();
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Down, 4.0, 0, 30, 1),
            Some(frame_from_scrub_bar(4.0, 30, 1))
        );
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Move, 8.0, 16, 30, 1),
            None
        );
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Move, 8.0, 31, 30, 1),
            None
        );
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Move, 8.0, 32, 30, 1),
            Some(frame_from_scrub_bar(8.0, 30, 1))
        );
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Up, 24.0, 40, 30, 1),
            Some(frame_from_scrub_bar(24.0, 30, 1))
        );
    }

    #[test]
    fn scrub_time_pump_restores_down_frame_only_after_dispatch() {
        let mut pump = ScrubTimePump::new();
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Down, 11.0, 0, 30, 1),
            Some(frame_from_scrub_bar(11.0, 30, 1))
        );
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 10, 30, 1),
            Some(frame_from_scrub_bar(11.0, 30, 1))
        );
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 20, 30, 1),
            None
        );
        let mut pump = ScrubTimePump::new();
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 20, 30, 1),
            None
        );
    }

    #[test]
    fn real_clip_down_dispatches_selection_once_via_renderer_path() {
        let mut real_session = TimelineSession::default();
        real_session.scene = TimelineScene::from_snapshot(
            &[crate::timeline_skia::SnapshotLayerInput {
                layer_id: "clip-real".into(),
                display_name: "clip".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![],
            }],
            None,
        );
        let mut selected = -1;
        let mut playhead = 0.27;
        let x = 202.0 + (3.0f64 / 5.0) * (1240.0 - 202.0 - 6.0);
        let y = 66.5;
        crate::host_bridge::test_reset_timeline_selection_dispatch_count();
        crate::host_bridge::test_clear_host_slot();
        let down = real_session.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert!(down.selection_commit.is_some());
        if let Some(commit) = down.selection_commit {
            assert!(!RendererCore::dispatch_timeline_selection(&commit));
        }
        assert_eq!(
            crate::host_bridge::test_timeline_selection_dispatch_count(),
            1
        );
    }

    #[test]
    fn fixture_down_or_trim_down_does_not_dispatch_selection() {
        let mut session = TimelineSession::default();
        let down_x = 202.0 + (3.0f64 / 48.0) * (1240.0 - 202.0 - 6.0);
        let y = 66.5;
        crate::host_bridge::test_reset_timeline_selection_dispatch_count();
        crate::host_bridge::test_clear_host_slot();
        let mut selected = -1;
        let mut playhead = 0.27;
        let clip_down = session.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            down_x,
            y,
            0,
        );
        assert!(clip_down.selection_commit.is_none());

        let trim_down = session.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            202.0 + (14.0f64 / 48.0) * (1240.0 - 202.0 - 6.0),
            y,
            0,
        );
        assert!(trim_down.selection_commit.is_none());

        if let Some(commit) = clip_down.selection_commit {
            assert!(!RendererCore::dispatch_timeline_selection(&commit));
        }
        if let Some(commit) = trim_down.selection_commit {
            assert!(!RendererCore::dispatch_timeline_selection(&commit));
        }
        assert_eq!(
            crate::host_bridge::test_timeline_selection_dispatch_count(),
            0
        );
    }

    #[test]
    fn host_stage_geometry_command_transitions_apply_and_clear() {
        let geometry_a = crate::host_bridge::HostStageGeometry {
            layers: vec![crate::host_bridge::HostStageGeometryLayer {
                layer_id: "L1".into(),
                corners: [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
                position: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            }],
            layers_truncated: false,
        };
        let geometry_b = crate::host_bridge::HostStageGeometry {
            layers: vec![crate::host_bridge::HostStageGeometryLayer {
                layer_id: "L1".into(),
                corners: [[-0.4, -0.4], [0.4, -0.4], [0.4, 0.4], [-0.4, 0.4]],
                position: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            }],
            layers_truncated: false,
        };
        let projection_a = crate::host_bridge::HostTimelineProjection {
            revision: "r".into(),
            projection_generation: "0".into(),
            primary_layer_id: None,
            current_time: (0, 1),
            fps: None,
            timeline_duration: Some((10, 1)),
            bounds: vec![],
            timeline_layers: None,
            stage_geometry: Some(geometry_a.clone()),
        };
        let projection_b = crate::host_bridge::HostTimelineProjection {
            revision: "r".into(),
            projection_generation: "0".into(),
            primary_layer_id: None,
            current_time: (0, 1),
            fps: None,
            timeline_duration: Some((10, 1)),
            bounds: vec![],
            timeline_layers: None,
            stage_geometry: Some(geometry_b.clone()),
        };

        let mut cached = None;
        let apply = host_stage_geometry_command(cached.as_ref(), Some(&projection_a));
        assert_eq!(apply, HostStageGeometryCommand::Apply(geometry_a.clone()));
        if let HostStageGeometryCommand::Apply(next) = apply {
            cached = Some(next);
        }
        assert_eq!(cached.as_ref(), Some(&geometry_a));

        let noop = host_stage_geometry_command(cached.as_ref(), Some(&projection_a));
        assert_eq!(noop, HostStageGeometryCommand::Noop);

        let apply = host_stage_geometry_command(cached.as_ref(), Some(&projection_b));
        assert_eq!(apply, HostStageGeometryCommand::Apply(geometry_b.clone()));
        if let HostStageGeometryCommand::Apply(next) = apply {
            cached = Some(next);
        }
        assert_eq!(cached.as_ref(), Some(&geometry_b));

        let clear = host_stage_geometry_command(cached.as_ref(), None);
        assert_eq!(clear, HostStageGeometryCommand::Clear);
    }

    #[test]
    fn timeline_keymap_delete_branches_on_selected_real_key() {
        crate::host_bridge::test_reset_keymap_dispatch_counts();
        let mut scene = TimelineScene::from_snapshot(
            &[crate::timeline_skia::SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![crate::timeline_skia::SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                }],
            }],
            None,
        );
        let _ = crate::host_bridge::try_timeline_keymap_delete(&scene);
        assert_eq!(
            crate::host_bridge::test_keymap_remove_position_key_count(),
            0
        );
        assert_eq!(crate::host_bridge::test_keymap_delete_layer_count(), 1);

        crate::timeline_skia::test_select_first_real_key(&mut scene);
        let _ = crate::host_bridge::try_timeline_keymap_delete(&scene);
        assert_eq!(
            crate::host_bridge::test_keymap_remove_position_key_count(),
            1
        );
        assert_eq!(crate::host_bridge::test_keymap_delete_layer_count(), 1);
    }

    #[test]
    fn key_selection_survives_revision_reproject_so_delete_removes_key() {
        crate::host_bridge::test_reset_keymap_dispatch_counts();
        let mut scene = TimelineScene::from_snapshot(
            &[crate::timeline_skia::SnapshotLayerInput {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                interval_secs: Some((0.0, 10.0)),
                keys: vec![
                    crate::timeline_skia::SnapshotKeyInput {
                        key_id: 7,
                        time_secs: 4.0,
                    },
                    crate::timeline_skia::SnapshotKeyInput {
                        key_id: 8,
                        time_secs: 6.0,
                    },
                ],
            }],
            Some("11"),
        );
        crate::timeline_skia::test_select_first_real_key(&mut scene);
        assert_eq!(
            crate::timeline_skia::selected_real_key(&scene),
            Some(("11".into(), 7))
        );

        let mut projection = crate::host_bridge::HostTimelineProjection {
            revision: "r2".into(),
            projection_generation: "1".into(),
            primary_layer_id: Some("11".into()),
            current_time: (0, 1),
            timeline_duration: Some((10, 1)),
            fps: None,
            bounds: vec![("11".into(), "keyed".into())],
            timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
                layer_id: "11".into(),
                display_name: "keyed".into(),
                start_secs: 0.0,
                duration_secs: 10.0,
                position_keys: vec![
                    crate::host_bridge::HostTimelineKey {
                        key_id: 7,
                        time_secs: 4.0,
                        value: None,
                    },
                    crate::host_bridge::HostTimelineKey {
                        key_id: 8,
                        time_secs: 6.0,
                        value: None,
                    },
                ],
                param_keys: vec![],
                effects: vec![],
                effects_truncated: false,
                source_params: vec![],
                source_params_truncated: false,
                visible: true,
                solo: false,
            }]),
            stage_geometry: None,
        };
        let rebuilt = timeline_scene_from_projection(&scene, &projection);
        assert_eq!(
            crate::timeline_skia::selected_real_key(&rebuilt),
            Some(("11".into(), 7))
        );
        let _ = crate::host_bridge::try_timeline_keymap_delete(&rebuilt);
        assert_eq!(
            crate::host_bridge::test_keymap_remove_position_key_count(),
            1
        );
        assert_eq!(crate::host_bridge::test_keymap_delete_layer_count(), 0);

        projection.primary_layer_id = None;
        let primary_cleared = timeline_scene_from_projection(&scene, &projection);
        assert_eq!(
            crate::timeline_skia::selected_real_key(&primary_cleared),
            None
        );
    }

    #[test]
    fn move_preview_geometry_translates_only_target_layer() {
        let geometry = crate::host_bridge::HostStageGeometry {
            layers: vec![
                crate::host_bridge::HostStageGeometryLayer {
                    layer_id: "A".into(),
                    corners: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                    position: [0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                },
                crate::host_bridge::HostStageGeometryLayer {
                    layer_id: "B".into(),
                    corners: [[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0]],
                    position: [0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                },
            ],
            layers_truncated: false,
        };
        let preview = crate::rerun_stage::apply_move_preview_to_geometry(
            &geometry,
            Some(&("A".into(), [0.5, -0.25])),
        );
        assert_eq!(
            preview.layers[0].corners,
            [[0.5, -0.25], [1.5, -0.25], [1.5, 0.75], [0.5, 0.75]]
        );
        assert_eq!(preview.layers[1].corners, geometry.layers[1].corners);
        let restored = crate::rerun_stage::apply_move_preview_to_geometry(&geometry, None);
        assert_eq!(restored, geometry);
    }

    #[test]
    fn host_snapshot_read_needed_force_and_first_read_and_missing_stamp() {
        // force / 初回(None)は読む。host不在でstamp取得失敗もfull読みへ落とす。
        assert!(RendererCore::host_snapshot_read_needed(
            None,
            Some((1, 2)),
            false
        ));
        assert!(RendererCore::host_snapshot_read_needed(
            Some((1, 2)),
            Some((1, 2)),
            true
        ));
        assert!(RendererCore::host_snapshot_read_needed(
            Some((1, 2)),
            Some((0, 3)),
            false
        ));
        assert!(!RendererCore::host_snapshot_read_needed(
            Some((1, 2)),
            Some((1, 2)),
            false
        ));
        assert!(RendererCore::host_snapshot_read_needed(
            Some((1, 2)),
            None,
            false
        ));
    }
}
