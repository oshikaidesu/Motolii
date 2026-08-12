use std::time::Instant;

use crate::rerun_stage::{EmbeddedSpatialStage, StageTransformProjection};
use crate::timeline_skia::{TimelinePointerPhase, TimelineScene, TimelineSession};

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

struct TimelineResources {
    surface_texture: wgpu::Texture,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,
    pixels: Vec<u8>,
    dirty: bool,
}

struct StageResources {
    rerun: EmbeddedSpatialStage,
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
    scrubbing: bool,
    scrub_time_pump: ScrubTimePump,
    scrub_clock_start: Instant,
    force_next_host_snapshot: bool,
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
    let layers = crate::host_bridge::snapshot_layers_from_projection(projection);
    let mut scene = TimelineScene::from_snapshot(&layers, projection.primary_layer_id.as_deref());
    scene.view_a = existing_scene.view_a;
    scene.view_b = existing_scene.view_b;
    scene
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
                EmbeddedSpatialStage::new(&adapter, &device, &queue, config.format)
                    .map(|rerun| StageResources { rerun })
            })
            .transpose()?;
        let timeline = (scene == SceneKind::Timeline)
            .then(|| create_timeline_resources(&device, format, config.width, config.height));

        Ok(Self {
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            queue,
            config,
            stage,
            timeline,
            timeline_session: (scene == SceneKind::Timeline).then(TimelineSession::default),
            host_revision: None,
            host_projection_generation: None,
            host_stage_geometry: None,
            host_stage_viewport: None,
            host_fps: None,
            scrubbing: false,
            scrub_time_pump: ScrubTimePump::new(),
            scrub_clock_start: Instant::now(),
            force_next_host_snapshot: false,
            scene,
            selected_object_index: 1,
            playhead: 0.27,
            frame: 0,
            stats: RenderStats::default(),
        })
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
        self.selected_object_index = selected_object_index.max(-1);
        let real = self
            .timeline_session
            .as_ref()
            .is_some_and(|session| session.scene.real);
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

    pub(crate) fn stage_transform_projection(&self) -> Option<StageTransformProjection> {
        self.stage
            .as_ref()
            .map(|stage| stage.rerun.transform_projection())
    }

    pub(crate) fn set_stage_transform_projection(&mut self, projection: StageTransformProjection) -> bool {
        self.stage
            .as_mut()
            .is_some_and(|stage| stage.rerun.set_transform_projection(projection))
    }

    pub(crate) fn timeline_hit_test(&self, x: f64, y: f64) -> Option<(i32, f64)> {
        let Some(session) = &self.timeline_session else {
            return None;
        };
        crate::timeline_skia::hit_test(
            &session.scene,
            self.config.width,
            self.config.height,
            x,
            y,
        )
    }

    /// Timeline pointer。戻り値trueはselection/playhead変化(feedback対象)。
    pub(crate) fn timeline_pointer(
        &mut self,
        phase: PointerPhase,
        x: f64,
        y: f64,
    ) -> Option<(i32, f64)> {
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
            );
            (is_real, outcome)
        };
        if outcome.dirty {
            if let Some(timeline) = &mut self.timeline {
                timeline.dirty = true;
            }
        }
        if is_real {
            let maybe_scrub_playhead = outcome.scrub_playhead.or(if matches!(phase, PointerPhase::Cancel) {
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
            if matches!(phase, PointerPhase::Up | PointerPhase::Cancel) {
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
            self.scrubbing = self.scrub_time_pump.is_active();
        } else if matches!(phase, PointerPhase::Up | PointerPhase::Cancel) {
            self.scrubbing = false;
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
        let bar = playhead.clamp(0.0, 1.0) * f64::from(crate::timeline_skia::SONG_BARS);
        let Some(frame) = self.scrub_time_pump.next_frame(
            phase,
            bar,
            self.now_ms(),
            fps_num,
            fps_den,
        ) else {
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

    pub(crate) fn stats(&self) -> RenderStats {
        self.stats
    }

    pub(crate) fn stage_pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
        // host 投影が正本の間は fixture gizmo probe へ送らない。
        if self.host_stage_geometry.is_some() {
            return;
        }
        let Some(stage) = &mut self.stage else { return };
        match phase {
            PointerPhase::Down => self.stats.pointer_downs += 1,
            PointerPhase::Move => self.stats.pointer_moves += 1,
            PointerPhase::Up => self.stats.pointer_ups += 1,
            PointerPhase::Cancel => {}
        }
        stage.rerun.pointer(phase, x, y);
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
                self.sync_host_stage_geometry();
                self.stage.as_mut().expect("stage resources").rerun.render(
                    &self.device,
                    &self.queue,
                    &view,
                    self.config.width,
                    self.config.height,
                )?
            }
            SceneKind::Timeline => {
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

    fn sync_host_timeline_projection(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let Some(projection) = crate::host_bridge::try_read_timeline_projection() else {
                return;
            };
            self.host_fps = projection.fps;
            let revision_changed =
                self.host_revision.as_deref() != Some(projection.revision.as_str());
            let generation_changed = self.host_projection_generation.as_deref()
                != Some(projection.projection_generation.as_str());
            if revision_changed {
                let Some(session) = &mut self.timeline_session else {
                    return;
                };
                let scene = timeline_scene_from_projection(&session.scene, &projection);
                self.selected_object_index = scene.selected_flat;
                session.scene = scene;
                self.host_revision = Some(projection.revision);
                if let Some(timeline) = &mut self.timeline {
                    timeline.dirty = true;
                }
            }
            if !self.scrubbing
                && (generation_changed || revision_changed || self.force_next_host_snapshot)
            {
                let next = crate::host_bridge::playhead_from_current_time(
                    projection.current_time.0,
                    projection.current_time.1,
                );
                if (self.playhead - next).abs() > f64::EPSILON {
                    self.playhead = next;
                    if let Some(timeline) = &mut self.timeline {
                        timeline.dirty = true;
                    }
                }
                self.force_next_host_snapshot = false;
            }
            self.host_projection_generation = Some(projection.projection_generation);
        }
    }

    fn sync_host_stage_geometry(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let Some(stage) = self.stage.as_mut() else {
                return;
            };
            let viewport = (self.config.width, self.config.height);
            let command = match crate::host_bridge::try_read_timeline_projection() {
                Some(projection) => {
                    host_stage_geometry_command(self.host_stage_geometry.as_ref(), Some(&projection))
                }
                None => host_stage_geometry_command(self.host_stage_geometry.as_ref(), None),
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
                    }
                }
                HostStageGeometryCommand::Noop => {
                    // geometry不変でもviewport aspectが変わったら再投影する。
                    if self.host_stage_viewport != Some(viewport) {
                        if let Some(geometry) = self.host_stage_geometry.clone() {
                            if stage.rerun.apply_host_stage_geometry(
                                &geometry,
                                viewport.0,
                                viewport.1,
                            ) {
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
    fn timeline_view_is_preserved_when_revision_changes() {
        let mut scene = TimelineScene::default();
        scene.view_a = 12.0;
        scene.view_b = 48.0;
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "r1".into(),
            projection_generation: "0".into(),
            primary_layer_id: Some("L2".into()),
            current_time: (0, 1),
            fps: None,
            bounds: vec![
                ("L1".into(), "Layer 1".into()),
                ("L2".into(), "Layer 2".into()),
            ],
            timeline_layers: None,
            stage_geometry: None,
        };
        let rebuilt = timeline_scene_from_projection(&scene, &projection);
        assert_eq!(rebuilt.view_a, 12.0);
        assert_eq!(rebuilt.view_b, 48.0);
        assert_eq!(rebuilt.selected_flat, 1);
    }

    #[test]
    fn scrub_time_pump_throttle_moves_and_always_dispatches_release() {
        let mut pump = ScrubTimePump::new();
        assert_eq!(
            pump.next_frame(ScrubPointerPhase::Down, 4.0, 0, 30, 1),
            Some(frame_from_scrub_bar(4.0, 30, 1))
        );
        assert_eq!(pump.next_frame(ScrubPointerPhase::Move, 8.0, 16, 30, 1), None);
        assert_eq!(pump.next_frame(ScrubPointerPhase::Move, 8.0, 31, 30, 1), None);
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
        assert_eq!(pump.next_frame(ScrubPointerPhase::Down, 11.0, 0, 30, 1), Some(frame_from_scrub_bar(11.0, 30, 1)));
        assert_eq!(pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 10, 30, 1), Some(frame_from_scrub_bar(11.0, 30, 1)));
        assert_eq!(pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 20, 30, 1), None);
        let mut pump = ScrubTimePump::new();
        assert_eq!(pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 20, 30, 1), None);
    }

    #[test]
    fn host_stage_geometry_command_transitions_apply_and_clear() {
        let geometry_a = crate::host_bridge::HostStageGeometry {
            layers: vec![crate::host_bridge::HostStageGeometryLayer {
                layer_id: "L1".into(),
                corners: [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
            }],
            layers_truncated: false,
        };
        let geometry_b = crate::host_bridge::HostStageGeometry {
            layers: vec![crate::host_bridge::HostStageGeometryLayer {
                layer_id: "L1".into(),
                corners: [[-0.4, -0.4], [0.4, -0.4], [0.4, 0.4], [-0.4, 0.4]],
            }],
            layers_truncated: false,
        };
        let projection_a = crate::host_bridge::HostTimelineProjection {
            revision: "r".into(),
            projection_generation: "0".into(),
            primary_layer_id: None,
            current_time: (0, 1),
            fps: None,
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
}
