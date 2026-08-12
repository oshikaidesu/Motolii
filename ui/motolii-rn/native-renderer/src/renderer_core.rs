use std::time::Instant;

use crate::rerun_stage::{EmbeddedSpatialStage, StageTransformProjection};
use crate::timeline_skia::{TimelinePointerPhase, TimelineScene, TimelineSession};

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
    let mut scene = TimelineScene::from_snapshot(
        &projection.bounds,
        projection.primary_layer_id.as_deref(),
    );
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
        self.playhead = playhead.clamp(0.0, 1.0);
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
        let Some(session) = &mut self.timeline_session else {
            return None;
        };
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
        let outcome = session.pointer(
            &mut self.selected_object_index,
            &mut self.playhead,
            self.config.width,
            self.config.height,
            tl_phase,
            x,
            y,
        );
        if outcome.dirty {
            if let Some(timeline) = &mut self.timeline {
                timeline.dirty = true;
            }
        }
        outcome
            .feedback
            .then_some((self.selected_object_index, self.playhead))
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
            SceneKind::Stage => self.stage.as_mut().expect("stage resources").rerun.render(
                &self.device,
                &self.queue,
                &view,
                self.config.width,
                self.config.height,
            )?,
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
            if self.host_revision.as_deref() == Some(projection.revision.as_str()) {
                return;
            }
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

    #[test]
    fn timeline_view_is_preserved_when_revision_changes() {
        let mut scene = TimelineScene::default();
        scene.view_a = 12.0;
        scene.view_b = 48.0;
        let projection = crate::host_bridge::HostTimelineProjection {
            revision: "r1".into(),
            primary_layer_id: Some("L2".into()),
            bounds: vec![
                ("L1".into(), "Layer 1".into()),
                ("L2".into(), "Layer 2".into()),
            ],
        };
        let rebuilt = timeline_scene_from_projection(&scene, &projection);
        assert_eq!(rebuilt.view_a, 12.0);
        assert_eq!(rebuilt.view_b, 48.0);
        assert_eq!(rebuilt.selected_flat, 1);
    }
}
