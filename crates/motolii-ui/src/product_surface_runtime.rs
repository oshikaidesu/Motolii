struct ProductSurface {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    gpu: Arc<GpuCtx>,
    config: wgpu::SurfaceConfiguration,
    preview_pipeline: wgpu::RenderPipeline,
    preview_bind_group: wgpu::BindGroup,
    native_timeline_renderer: NativeTimelineRenderer,
    last_timeline_scene_trace: Option<(u64, usize, usize, usize, usize)>,
    place_overlay_pipeline: wgpu::RenderPipeline,
    place_overlay_vertices: wgpu::Buffer,
    occluded: bool,
}

struct ProductGpuParts {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
}

struct ProductSurfaceFrame<'a> {
    layout: NativeHostLayout,
    window: &'a Window,
    document: &'a motolii_doc::Document,
    timeline_projection: &'a ProductTimelineProjection,
    primary: Option<LayerId>,
    playhead: RationalTime,
    interval_preview: Option<TimelineIntervalPreview>,
    place_overlay: Option<&'a RectanglePlaceOverlay>,
}

impl ProductSurface {
    fn new(
        window: &Arc<Window>,
        parts: ProductGpuParts,
        gpu: &Arc<GpuCtx>,
        preview: &StaticPreview,
    ) -> Result<Self, ProductRuntimeError> {
        let surface = parts.instance.create_surface(Arc::clone(window))?;
        if !parts.adapter.is_surface_supported(&surface) {
            return Err(ProductRuntimeError::SurfaceUnsupported);
        }
        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&parts.adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let alpha_mode = capabilities
            .alpha_modes
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&parts.device, &config);
        let (preview_pipeline, preview_bind_group) =
            create_preview_pipeline(&parts.device, format, preview.slot().view());
        let native_timeline_renderer = NativeTimelineRenderer::new(
            &parts.device,
            &gpu.queue,
            format,
            size.width,
            size.height,
        )?;
        let place_overlay_pipeline = create_place_overlay_pipeline(&parts.device, format);
        let place_overlay_vertices = parts.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-product-place-overlay-vertices"),
            size: 48,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            instance: parts.instance,
            adapter: parts.adapter,
            surface,
            gpu: Arc::clone(gpu),
            config,
            preview_pipeline,
            preview_bind_group,
            native_timeline_renderer,
            last_timeline_scene_trace: None,
            place_overlay_pipeline,
            place_overlay_vertices,
            occluded: false,
        })
    }

    fn configure(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.native_timeline_renderer
            .resize(&self.gpu.device, width, height);
        self.reconfigure();
    }

    fn reconfigure(&self) {
        self.surface.configure(&self.gpu.device, &self.config);
    }

    fn render(&mut self, input: ProductSurfaceFrame<'_>) -> Result<(), ProductSurfaceError> {
        let ProductSurfaceFrame {
            layout,
            window,
            document,
            timeline_projection,
            primary,
            playhead,
            interval_preview,
            place_overlay,
        } = input;
        if self.occluded || self.config.width == 0 || self.config.height == 0 {
            return Err(ProductSurfaceError::Skip);
        }
        let timeline_stats = self.native_timeline_renderer.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            TimelinePrepareInput {
                layout,
                document,
                projection: &timeline_projection.projection,
                primary,
                playhead,
                interval_preview,
            },
        )?;
        let trace_key = (
            layout.epoch,
            timeline_stats.rows,
            timeline_stats.bars,
            timeline_stats.keys,
            timeline_stats.text_runs,
        );
        if self.last_timeline_scene_trace != Some(trace_key) {
            if let Some(timeline) = layout.timeline_physical {
                crate::ui_numeric_trace::emit(format_args!(
                    "kind=timeline-scene layout_epoch={} rows={} bars={} keys={} text_runs={} \
                     physical_x={} physical_y={} physical_width={} physical_height={}",
                    layout.epoch,
                    timeline_stats.rows,
                    timeline_stats.bars,
                    timeline_stats.keys,
                    timeline_stats.text_runs,
                    timeline.x,
                    timeline.y,
                    timeline.width,
                    timeline.height,
                ));
                self.last_timeline_scene_trace = Some(trace_key);
            }
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(ProductSurfaceError::Recover);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ProductSurfaceError::Fatal(
                    "native product Surface validation failed".to_owned(),
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-product-native-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-product-stage-timeline"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.020,
                            b: 0.024,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            draw_rect(
                &mut pass,
                layout.stage_physical,
                &self.preview_pipeline,
                Some(&self.preview_bind_group),
            );
            if let Some(place_overlay) = place_overlay {
                let bytes = place_overlay.vertex_bytes();
                self.gpu
                    .queue
                    .write_buffer(&self.place_overlay_vertices, 0, &bytes);
                pass.set_pipeline(&self.place_overlay_pipeline);
                pass.set_vertex_buffer(0, self.place_overlay_vertices.slice(..));
                pass.set_viewport(
                    layout.stage_physical.x as f32,
                    layout.stage_physical.y as f32,
                    layout.stage_physical.width as f32,
                    layout.stage_physical.height as f32,
                    0.0,
                    1.0,
                );
                pass.set_scissor_rect(
                    layout.stage_physical.x,
                    layout.stage_physical.y,
                    layout.stage_physical.width,
                    layout.stage_physical.height,
                );
                pass.draw(0..6, 0..1);
            }
            self.native_timeline_renderer.composite(&mut pass);
        }
        self.gpu.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        frame.present();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RectanglePlaceOverlay {
    vertices: [[f32; 2]; 6],
}

impl RectanglePlaceOverlay {
    fn vertex_bytes(&self) -> [u8; 48] {
        let mut bytes = [0_u8; 48];
        for (index, component) in self.vertices.iter().flatten().enumerate() {
            let start = index * 4;
            bytes[start..start + 4].copy_from_slice(&component.to_ne_bytes());
        }
        bytes
    }
}

fn rectangle_place_overlay(
    camera: motolii_core::CompCamera,
    ndc: [f64; 2],
) -> Option<RectanglePlaceOverlay> {
    let center = canonical_drop_from_ndc(camera, ndc)?;
    let corners = [
        CanonicalPoint {
            x: center[0] - 0.1,
            y: center[1] - 0.1,
        },
        CanonicalPoint {
            x: center[0] + 0.1,
            y: center[1] - 0.1,
        },
        CanonicalPoint {
            x: center[0] + 0.1,
            y: center[1] + 0.1,
        },
        CanonicalPoint {
            x: center[0] - 0.1,
            y: center[1] + 0.1,
        },
    ];
    let mut projected = [[0.0_f32; 2]; 4];
    for (target, corner) in projected.iter_mut().zip(corners) {
        let (x, y) = camera.world_to_ndc(corner).ok()?;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        *target = [x as f32, y as f32];
    }
    Some(RectanglePlaceOverlay {
        vertices: [
            projected[0],
            projected[1],
            projected[2],
            projected[0],
            projected[2],
            projected[3],
        ],
    })
}

fn draw_rect<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    rect: PhysicalRect,
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: Option<&'a wgpu::BindGroup>,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = bind_group {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.set_viewport(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);
    pass.draw(0..3, 0..1);
}

fn create_preview_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    view: &wgpu::TextureView,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("motolii-product-preview-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("motolii-product-preview-layout"),
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
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("motolii-product-preview-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    (
        create_pipeline(device, format, Some(&layout), PREVIEW_SHADER),
        bind_group,
    )
}

fn create_place_overlay_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-place-overlay-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(PLACE_OVERLAY_SHADER)),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-place-overlay-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-place-overlay-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: Option<&wgpu::BindGroupLayout>,
    source: &'static str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-native-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
    });
    let layouts: Vec<_> = bind_group_layout.into_iter().map(Some).collect();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-native-pipeline-layout"),
        bind_group_layouts: &layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-native-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

const PREVIEW_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var uvs = array<vec2<f32>, 3>(vec2(0.0,1.0), vec2(2.0,1.0), vec2(0.0,-1.0));
    var out: VertexOut; out.position = vec4(positions[index],0.0,1.0); out.uv = uvs[index]; return out;
}
@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@fragment fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.uv);
}
"#;
const PLACE_OVERLAY_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32> }
@vertex fn vs_main(@location(0) position: vec2<f32>) -> VertexOut {
    var out: VertexOut; out.position = vec4(position, 0.0, 1.0); return out;
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.8, 0.58431375, 0.5294118, 0.42);
}
"#;

#[derive(Debug, thiserror::Error)]
enum ProductSurfaceError {
    #[error("native product Surface must be reconfigured")]
    Recover,
    #[error("native product Surface frame must be retried")]
    Retry,
    #[error("native product Surface frame is skipped")]
    Skip,
    #[error("native product Surface failed: {0}")]
    Fatal(String),
    #[error(transparent)]
    NativeTimeline(#[from] NativeTimelineRendererError),
}
