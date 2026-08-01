//! G0-9受入済みmodelを通常製品windowから開くprivate native wgpu popup。

use std::sync::Arc;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use motolii_eval::Interp;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId, WindowLevel},
};

use crate::easing_popup_model::{
    build_popup_scene, curve_point_from_graph, hit_action, hit_handle, hit_preset,
    hit_preset_index, place_popup, Bezier, Handle, LogicalRect, PhysicalRect, PopupAction,
    PopupSession, PopupVisualState, Primitive, SpikePresetStore, TextPrimitive, POPUP_HEIGHT,
    POPUP_WIDTH,
};
use crate::stage_chrome_host_runtime::OpenEasingRequest;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    size: [f32; 2],
    _padding: [f32; 2],
}

struct PreparedText {
    buffer: Buffer,
    left: f32,
    top: f32,
    bounds: TextBounds,
    color: Color,
}

struct PopupGfx {
    surface: wgpu::Surface<'static>,
    gpu: Arc<motolii_gpu::GpuCtx>,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    screen_buffer: wgpu::Buffer,
    primitive_buffer: wgpu::Buffer,
    primitive_count: u32,
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    texts: Vec<PreparedText>,
}

impl PopupGfx {
    fn new(
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        gpu: Arc<motolii_gpu::GpuCtx>,
        window: &Arc<Window>,
        session: &PopupSession,
        store: &SpikePresetStore,
        visual: PopupVisualState,
    ) -> Result<Self, EasingPopupError> {
        let surface = instance.create_surface(Arc::clone(window))?;
        if !adapter.is_surface_supported(&surface) {
            return Err(EasingPopupError::SurfaceUnsupported);
        }
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(EasingPopupError::SurfaceUnsupported)?;
        let alpha_mode = capabilities
            .alpha_modes
            .first()
            .copied()
            .ok_or(EasingPopupError::SurfaceUnsupported)?;
        let size = window.inner_size();
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
        surface.configure(&gpu.device, &config);

        let screen_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-easing-popup-screen"),
            size: std::mem::size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let primitive_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-easing-popup-primitives"),
            size: (std::mem::size_of::<Primitive>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("motolii-easing-popup-bind-group-layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-easing-popup-bind-group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("motolii-easing-popup-shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("easing_popup.wgsl").into()),
            });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("motolii-easing-popup-pipeline-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("motolii-easing-popup-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Primitive>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Uint32,
                                offset: 48,
                                shader_location: 3,
                            },
                        ],
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
            });
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&gpu.device);
        let viewport = Viewport::new(&gpu.device, &cache);
        let mut atlas = TextAtlas::new(&gpu.device, &gpu.queue, &cache, format);
        let text_renderer = TextRenderer::new(
            &mut atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );
        let mut gfx = Self {
            surface,
            gpu,
            config,
            pipeline,
            bind_group,
            screen_buffer,
            primitive_buffer,
            primitive_count: 0,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            texts: Vec::new(),
        };
        gfx.update_scene(session, store, visual)?;
        Ok(gfx)
    }

    fn update_scene(
        &mut self,
        session: &PopupSession,
        store: &SpikePresetStore,
        visual: PopupVisualState,
    ) -> Result<(), EasingPopupError> {
        let scene = build_popup_scene(session, store, visual);
        if scene.primitives.len() > 1024 {
            return Err(EasingPopupError::PrimitiveCapacity);
        }
        let scale_x = self.config.width as f32 / POPUP_WIDTH as f32;
        let scale_y = self.config.height as f32 / POPUP_HEIGHT as f32;
        let scaled = scene
            .primitives
            .iter()
            .copied()
            .map(|mut primitive| {
                primitive.bounds[0] *= scale_x;
                primitive.bounds[1] *= scale_y;
                primitive.bounds[2] *= scale_x;
                primitive.bounds[3] *= scale_y;
                if primitive.shape == 3 {
                    primitive.extra[0] *= scale_x;
                    primitive.extra[1] *= scale_y;
                    primitive.extra[2] *= scale_x.min(scale_y);
                }
                primitive
            })
            .collect::<Vec<_>>();
        self.primitive_count = scaled.len() as u32;
        self.gpu
            .queue
            .write_buffer(&self.primitive_buffer, 0, bytemuck::cast_slice(&scaled));
        self.gpu.queue.write_buffer(
            &self.screen_buffer,
            0,
            bytemuck::bytes_of(&ScreenUniform {
                size: [self.config.width as f32, self.config.height as f32],
                _padding: [0.0; 2],
            }),
        );
        self.viewport.update(
            &self.gpu.queue,
            Resolution {
                width: self.config.width,
                height: self.config.height,
            },
        );
        self.texts = scene
            .texts
            .into_iter()
            .map(|text| {
                prepare_text(
                    &mut self.font_system,
                    text,
                    scale_x,
                    scale_y,
                    self.config.width,
                    self.config.height,
                )
            })
            .collect();
        let areas = self.texts.iter().map(|text| TextArea {
            buffer: &text.buffer,
            left: text.left,
            top: text.top,
            scale: 1.0,
            bounds: text.bounds,
            default_color: text.color,
            custom_glyphs: &[],
        });
        self.text_renderer
            .prepare(
                &self.gpu.device,
                &self.gpu.queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|_| EasingPopupError::TextPrepare)
    }

    fn render(&mut self) -> Result<(), EasingPopupError> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(())
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.gpu.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => return Err(EasingPopupError::SurfaceFrame),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-easing-popup-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-easing-popup-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0103,
                            g: 0.0103,
                            b: 0.0103,
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.primitive_buffer.slice(..));
            pass.draw(0..6, 0..self.primitive_count);
            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut pass)
                .map_err(|_| EasingPopupError::TextRender)?;
        }
        self.gpu.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }
}

pub(crate) struct EasingPopupRuntime {
    window: Arc<Window>,
    gfx: PopupGfx,
    session: PopupSession,
    store: SpikePresetStore,
    target: OpenEasingRequest,
    cursor: [f32; 2],
    drag_token: Option<u64>,
    next_drag_token: u64,
    visual: PopupVisualState,
    received_focus: bool,
}

pub(crate) struct EasingPopupOpen<'a> {
    pub(crate) event_loop: &'a ActiveEventLoop,
    pub(crate) host: &'a Arc<Window>,
    pub(crate) instance: &'a wgpu::Instance,
    pub(crate) adapter: &'a wgpu::Adapter,
    pub(crate) gpu: Arc<motolii_gpu::GpuCtx>,
    pub(crate) target: OpenEasingRequest,
    pub(crate) anchor: LogicalRect,
    pub(crate) interp: Interp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EasingPopupInput {
    CloseRequested,
    Focused(bool),
    Resized { width: u32, height: u32 },
    RedrawRequested,
    CursorMoved { physical_x: f64, physical_y: f64 },
    PrimaryPressed,
    PrimaryReleased,
    EscapePressed,
    TabPressed,
    ArrowPressed(EasingPopupArrow),
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EasingPopupArrow {
    Left,
    Right,
    Up,
    Down,
}

impl EasingPopupRuntime {
    pub(crate) fn open(open: EasingPopupOpen<'_>) -> Result<Self, EasingPopupError> {
        let EasingPopupOpen {
            event_loop,
            host,
            instance,
            adapter,
            gpu,
            target,
            anchor,
            interp,
        } = open;
        let scale = host.scale_factor();
        let host_origin = host.outer_position().unwrap_or(PhysicalPosition::new(0, 0));
        let monitor = host.current_monitor().or_else(|| host.primary_monitor());
        let (monitor_position, monitor_size) = monitor
            .map(|monitor| (monitor.position(), monitor.size()))
            .unwrap_or((PhysicalPosition::new(0, 0), host.inner_size()));
        let placement = place_popup(
            [host_origin.x, host_origin.y],
            anchor,
            scale,
            PhysicalRect {
                x: monitor_position.x,
                y: monitor_position.y,
                width: monitor_size.width,
                height: monitor_size.height,
            },
        )
        .ok_or(EasingPopupError::Placement)?;
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Motolii · Interval Easing")
                    .with_inner_size(LogicalSize::new(POPUP_WIDTH, POPUP_HEIGHT))
                    .with_position(PhysicalPosition::new(placement.x, placement.y))
                    .with_decorations(false)
                    .with_resizable(false)
                    .with_window_level(WindowLevel::AlwaysOnTop),
            )?,
        );
        let curve = interp_to_bezier(interp);
        let session = PopupSession::new(curve, target.projection_generation, target.layout_epoch);
        let visual = PopupVisualState {
            focused_handle: Some(Handle::Start),
            ..Default::default()
        };
        let store = SpikePresetStore::default();
        let accessibility_count = session.bounded_accessibility().len();
        crate::ui_numeric_trace::emit(format_args!(
            "kind=easing-popup state=opening accessibility_nodes={} readbacks={} hot_resource_creations={}",
            accessibility_count,
            session.readback_count,
            session.hot_resource_creation_count,
        ));
        let mut gfx = PopupGfx::new(instance, adapter, gpu, &window, &session, &store, visual)?;
        gfx.render()?;
        window.focus_window();
        window.request_redraw();
        Ok(Self {
            window,
            gfx,
            session,
            store,
            target,
            cursor: [0.0; 2],
            drag_token: None,
            next_drag_token: 1,
            visual,
            received_focus: false,
        })
    }

    pub(crate) fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub(crate) fn target(&self) -> OpenEasingRequest {
        self.target
    }

    pub(crate) fn handle_input(
        &mut self,
        input: EasingPopupInput,
    ) -> Result<EasingPopupOutcome, EasingPopupError> {
        match input {
            EasingPopupInput::CloseRequested => return Ok(EasingPopupOutcome::Closed),
            EasingPopupInput::Focused(focused) => {
                if focused {
                    self.received_focus = true;
                } else if self.received_focus {
                    self.session.cancel();
                    return Ok(EasingPopupOutcome::Closed);
                }
            }
            EasingPopupInput::Resized { width, height } if width > 0 && height > 0 => {
                self.gfx.config.width = width;
                self.gfx.config.height = height;
                self.gfx
                    .surface
                    .configure(&self.gfx.gpu.device, &self.gfx.config);
                self.refresh()?;
            }
            EasingPopupInput::RedrawRequested => self.gfx.render()?,
            EasingPopupInput::CursorMoved {
                physical_x,
                physical_y,
            } => {
                let scale = self.window.scale_factor() as f32;
                self.cursor = [physical_x as f32 / scale, physical_y as f32 / scale];
                self.visual.hovered_preset = hit_preset_index(self.cursor);
                self.visual.hovered_handle = hit_handle(self.session.curve(), self.cursor);
                if self.drag_token.is_some() {
                    self.session
                        .update_drag(curve_point_from_graph(self.cursor));
                }
                self.refresh()?;
            }
            EasingPopupInput::PrimaryPressed => {
                if let Some(handle) = hit_handle(self.session.curve(), self.cursor) {
                    let token = self.next_drag_token;
                    self.next_drag_token = self
                        .next_drag_token
                        .checked_add(1)
                        .ok_or(EasingPopupError::DragTokenExhausted)?;
                    if self.session.begin_drag(handle, token) {
                        self.drag_token = Some(token);
                        self.visual.focused_handle = Some(handle);
                    }
                    self.refresh()?;
                } else if let Some(curve) = hit_preset(self.cursor) {
                    self.session.apply_preset(curve);
                    return Ok(EasingPopupOutcome::Commit(bezier_to_interp(curve)));
                } else if let Some(action) = hit_action(self.cursor) {
                    match action {
                        PopupAction::Close => return Ok(EasingPopupOutcome::Closed),
                        PopupAction::SavePreset | PopupAction::FavoriteLatest => {
                            crate::ui_numeric_trace::emit(format_args!(
                                "kind=easing-popup state=user-settings-unavailable"
                            ));
                        }
                    }
                }
            }
            EasingPopupInput::PrimaryReleased => {
                if let Some(token) = self.drag_token.take() {
                    if let Some(commit) = self.session.release(
                        token,
                        self.target.projection_generation,
                        self.target.layout_epoch,
                    ) {
                        return Ok(EasingPopupOutcome::Commit(bezier_to_interp(commit.curve)));
                    }
                    return Ok(EasingPopupOutcome::Closed);
                }
            }
            EasingPopupInput::EscapePressed => return Ok(EasingPopupOutcome::Closed),
            EasingPopupInput::TabPressed => {
                self.visual.focused_handle = Some(match self.visual.focused_handle {
                    Some(Handle::Start) => Handle::End,
                    Some(Handle::End) | None => Handle::Start,
                });
                self.refresh()?;
            }
            EasingPopupInput::ArrowPressed(arrow) => {
                let curve = self.session.curve();
                let handle = self.visual.focused_handle.unwrap_or(Handle::Start);
                let mut point = match handle {
                    Handle::Start => [curve.x1, curve.y1],
                    Handle::End => [curve.x2, curve.y2],
                };
                match arrow {
                    EasingPopupArrow::Left => point[0] -= 0.01,
                    EasingPopupArrow::Right => point[0] += 0.01,
                    EasingPopupArrow::Up => point[1] += 0.01,
                    EasingPopupArrow::Down => point[1] -= 0.01,
                }
                let next = curve.with_handle(handle, point);
                return Ok(EasingPopupOutcome::Commit(bezier_to_interp(next)));
            }
            EasingPopupInput::Resized { .. } | EasingPopupInput::Ignored => {}
        }
        Ok(EasingPopupOutcome::Handled)
    }

    fn refresh(&mut self) -> Result<(), EasingPopupError> {
        self.gfx
            .update_scene(&self.session, &self.store, self.visual)?;
        self.gfx.render()?;
        self.window.request_redraw();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EasingPopupOutcome {
    Handled,
    Closed,
    Commit(Interp),
}

fn interp_to_bezier(interp: Interp) -> Bezier {
    match interp {
        Interp::Bezier { x1, y1, x2, y2 } => {
            Bezier::new(x1 as f32, y1 as f32, x2 as f32, y2 as f32)
        }
        Interp::Hold | Interp::Linear => Bezier::LINEAR,
    }
}

fn bezier_to_interp(curve: Bezier) -> Interp {
    Interp::Bezier {
        x1: curve.x1 as f64,
        y1: curve.y1 as f64,
        x2: curve.x2 as f64,
        y2: curve.y2 as f64,
    }
}

fn prepare_text(
    font_system: &mut FontSystem,
    text: TextPrimitive,
    scale_x: f32,
    scale_y: f32,
    width: u32,
    height: u32,
) -> PreparedText {
    let size = text.size * scale_y;
    let mut buffer = Buffer::new(font_system, Metrics::new(size, size * 1.25));
    buffer.set_size(font_system, Some(text.width * scale_x), Some(size * 1.5));
    buffer.set_text(
        font_system,
        &text.text,
        &Attrs::new().family(if text.monospace {
            Family::Monospace
        } else {
            Family::SansSerif
        }),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(font_system, false);
    let left = text.left * scale_x;
    let top = text.top * scale_y;
    PreparedText {
        buffer,
        left,
        top,
        bounds: TextBounds {
            left: left.floor() as i32,
            top: top.floor() as i32,
            right: ((text.left + text.width) * scale_x)
                .ceil()
                .min(width as f32) as i32,
            bottom: (top + size * 1.5).ceil().min(height as f32) as i32,
        },
        color: Color::rgba(text.color[0], text.color[1], text.color[2], text.color[3]),
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum EasingPopupError {
    #[error("Easing popup window failed")]
    Window(#[from] winit::error::OsError),
    #[error("Easing popup surface failed")]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error("Easing popup surface is unsupported")]
    SurfaceUnsupported,
    #[error("Easing popup surface frame failed")]
    SurfaceFrame,
    #[error("Easing popup placement failed")]
    Placement,
    #[error("Easing popup primitive capacity was exceeded")]
    PrimitiveCapacity,
    #[error("Easing popup text preparation failed")]
    TextPrepare,
    #[error("Easing popup text rendering failed")]
    TextRender,
    #[error("Easing popup drag token exhausted")]
    DragTokenExhausted,
}
