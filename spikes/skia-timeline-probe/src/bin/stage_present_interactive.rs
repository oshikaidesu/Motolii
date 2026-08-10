use skia_safe::{AlphaType, Color, ColorType, ImageInfo, Paint, PaintStyle, Rect, surfaces};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    preview: wgpu::Texture,
    overlay: wgpu::Texture,
    preview_pipe: wgpu::RenderPipeline,
    composite_pipe: wgpu::RenderPipeline,
    composite_bg: wgpu::BindGroup,
    overlay_pixels: Vec<u8>,
}

impl Gpu {
    fn new(window: &Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(Arc::clone(window))
            .expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("adapter");
        let info = adapter.get_info();
        println!("adapter={} backend={:?}", info.name, info.backend);
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("device");
        let caps = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let preview_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("preview-shader"), source: wgpu::ShaderSource::Wgsl(r#"
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn fs(i:O)->@location(0) vec4<f32> { let q=i.uv*2.-1.; let glow=.12/max(length(q-vec2(.18,-.12)),.12); let checker=select(.025,.055,((u32(i.uv.x*24.)+u32(i.uv.y*14.))&1u)==1u); return vec4(.035+checker+glow*.22,.045+checker+glow*.10,.065+checker+glow*.28,1.); }
        "#.into()) });
        let preview_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let preview_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("preview-pipe"),
            layout: Some(&preview_layout),
            vertex: wgpu::VertexState {
                module: &preview_shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &preview_shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
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
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                tex_entry(0),
                tex_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("composite-shader"), source: wgpu::ShaderSource::Wgsl(r#"
            @group(0) @binding(0) var preview:texture_2d<f32>; @group(0) @binding(1) var overlay:texture_2d<f32>; @group(0) @binding(2) var samp:sampler;
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn fs(i:O)->@location(0) vec4<f32> { let b=textureSample(preview,samp,i.uv); let o=textureSample(overlay,samp,i.uv); return vec4(o.rgb+b.rgb*(1.-o.a),1.); }
        "#.into()) });
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let composite_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite-pipe"),
            layout: Some(&composite_layout),
            vertex: wgpu::VertexState {
                module: &composite_shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &composite_shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
        let (preview, overlay, composite_bg, overlay_pixels) =
            make_targets(&device, &bgl, &sampler, config.width, config.height);
        Self {
            surface,
            device,
            queue,
            config,
            preview,
            overlay,
            preview_pipe,
            composite_pipe,
            composite_bg,
            overlay_pixels,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        let sampler = self
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let bgl = self.composite_pipe.get_bind_group_layout(0);
        (
            self.preview,
            self.overlay,
            self.composite_bg,
            self.overlay_pixels,
        ) = make_targets(&self.device, &bgl, &sampler, size.width, size.height);
    }

    fn draw_overlay(
        &mut self,
        scale: f64,
        gizmo: [f32; 2],
        focused: bool,
        outside_drag: bool,
    ) -> Duration {
        let started = Instant::now();
        let info = ImageInfo::new(
            (self.config.width as i32, self.config.height as i32),
            ColorType::RGBA8888,
            AlphaType::Premul,
            None,
        );
        let mut surface = surfaces::wrap_pixels(
            &info,
            &mut self.overlay_pixels,
            Some(self.config.width as usize * 4),
            None,
        )
        .unwrap();
        let c = surface.canvas();
        c.clear(Color::TRANSPARENT);
        c.scale((scale as f32, scale as f32));
        let lw = self.config.width as f32 / scale as f32;
        let lh = self.config.height as f32 / scale as f32;
        let mut p = Paint::default();
        p.set_anti_alias(true);
        p.set_style(PaintStyle::Stroke);
        for x in (0..lw as u32).step_by(64) {
            p.set_color(if x % 256 == 0 {
                Color::from_argb(85, 110, 180, 235)
            } else {
                Color::from_argb(34, 110, 180, 235)
            });
            p.set_stroke_width(1.0);
            c.draw_line((x as f32, 0.0), (x as f32, lh), &p);
        }
        for y in (0..lh as u32).step_by(64) {
            p.set_color(if y % 256 == 0 {
                Color::from_argb(85, 110, 180, 235)
            } else {
                Color::from_argb(34, 110, 180, 235)
            });
            c.draw_line((0.0, y as f32), (lw, y as f32), &p);
        }
        let r = Rect::from_xywh(gizmo[0], gizmo[1], 220.0, 130.0);
        p.set_color(if focused {
            Color::from_argb(245, 255, 207, 75)
        } else {
            Color::from_argb(180, 150, 150, 150)
        });
        p.set_stroke_width(2.0);
        c.draw_rect(r, &p);
        p.set_style(PaintStyle::Fill);
        for &(x, y) in &[
            (r.left, r.top),
            (r.center_x(), r.top),
            (r.right, r.top),
            (r.left, r.center_y()),
            (r.right, r.center_y()),
            (r.left, r.bottom),
            (r.center_x(), r.bottom),
            (r.right, r.bottom),
        ] {
            c.draw_circle((x, y), 4.5, &p);
        }
        p.set_style(PaintStyle::Stroke);
        p.set_color(if outside_drag {
            Color::from_argb(245, 255, 80, 105)
        } else {
            Color::from_argb(225, 75, 220, 245)
        });
        c.draw_line((r.center_x(), 0.0), (r.center_x(), lh), &p);
        c.draw_line((0.0, r.center_y()), (lw, r.center_y()), &p);
        let elapsed = started.elapsed();
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.overlay,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.overlay_pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.config.width * 4),
                rows_per_image: Some(self.config.height),
            },
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
        );
        elapsed
    }

    fn present(&mut self) -> Result<(), &'static str> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Err("recover");
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Err("retry");
            }
            wgpu::CurrentSurfaceTexture::Validation => return Err("validation"),
        };
        let pv = self.preview.create_view(&Default::default());
        let fv = frame.texture.create_view(&Default::default());
        let mut e = self.device.create_command_encoder(&Default::default());
        {
            let attachment = [Some(wgpu::RenderPassColorAttachment {
                view: &pv,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut p = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &attachment,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&self.preview_pipe);
            p.draw(0..3, 0..1);
        }
        {
            let attachment = [Some(wgpu::RenderPassColorAttachment {
                view: &fv,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut p = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &attachment,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&self.composite_pipe);
            p.set_bind_group(0, &self.composite_bg, &[]);
            p.draw(0..3, 0..1);
        }
        self.queue.submit([e.finish()]);
        frame.present();
        Ok(())
    }
}

fn tex_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}
fn texture(
    device: &wgpu::Device,
    label: &str,
    w: u32,
    h: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    })
}
fn make_targets(
    device: &wgpu::Device,
    bgl: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    w: u32,
    h: u32,
) -> (wgpu::Texture, wgpu::Texture, wgpu::BindGroup, Vec<u8>) {
    let preview = texture(
        device,
        "preview",
        w,
        h,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let overlay = texture(
        device,
        "overlay",
        w,
        h,
        wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &preview.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &overlay.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    (preview, overlay, bg, vec![0; w as usize * h as usize * 4])
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    scale: f64,
    cursor: [f32; 2],
    gizmo: [f32; 2],
    drag_offset: [f32; 2],
    dragging: bool,
    focused: bool,
    outside_drag: bool,
    dirty: bool,
    redraws: u64,
    uploads: u64,
    recoveries: u64,
    last_stat: Option<Instant>,
    raster_total: Duration,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let w = Arc::new(
            el.create_window(
                WindowAttributes::default()
                    .with_title("Motolii Stage integration probe")
                    .with_inner_size(LogicalSize::new(1280., 720.)),
            )
            .unwrap(),
        );
        self.scale = w.scale_factor();
        self.gizmo = [420., 260.];
        self.focused = w.has_focus();
        self.gpu = Some(Gpu::new(&w));
        self.window = Some(Arc::clone(&w));
        self.dirty = true;
        self.last_stat = Some(Instant::now());
        w.request_redraw();
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, e: WindowEvent) {
        if self.window.as_ref().is_none_or(|w| w.id() != id) {
            return;
        }
        match e {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                self.gpu.as_mut().unwrap().resize(s);
                self.dirty = true;
                self.window.as_ref().unwrap().request_redraw()
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale = scale_factor;
                self.dirty = true;
                self.window.as_ref().unwrap().request_redraw()
            }
            WindowEvent::Focused(v) => {
                self.focused = v;
                self.dirty = true;
                self.window.as_ref().unwrap().request_redraw()
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                self.cursor = [x as f32 / self.scale as f32, y as f32 / self.scale as f32];
                if self.dragging {
                    self.gizmo = [
                        self.cursor[0] - self.drag_offset[0],
                        self.cursor[1] - self.drag_offset[1],
                    ];
                    self.dirty = true;
                    self.window.as_ref().unwrap().request_redraw()
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if self.dragging {
                    self.outside_drag = true;
                    self.dirty = true;
                    self.window.as_ref().unwrap().request_redraw()
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    let inside = self.cursor[0] >= self.gizmo[0]
                        && self.cursor[0] <= self.gizmo[0] + 220.
                        && self.cursor[1] >= self.gizmo[1]
                        && self.cursor[1] <= self.gizmo[1] + 130.;
                    if inside {
                        self.dragging = true;
                        self.outside_drag = false;
                        self.drag_offset = [
                            self.cursor[0] - self.gizmo[0],
                            self.cursor[1] - self.gizmo[1],
                        ]
                    }
                } else if self.dragging {
                    self.dragging = false;
                    self.dirty = true;
                    self.window.as_ref().unwrap().request_redraw()
                }
            }
            WindowEvent::RedrawRequested => {
                let g = self.gpu.as_mut().unwrap();
                if self.dirty {
                    self.raster_total +=
                        g.draw_overlay(self.scale, self.gizmo, self.focused, self.outside_drag);
                    self.uploads += 1;
                    self.dirty = false
                }
                if g.present().is_err() {
                    self.recoveries += 1
                } else {
                    self.redraws += 1
                }
                if self.dragging {
                    self.window.as_ref().unwrap().request_redraw()
                }
                let avg = if self.uploads == 0 {
                    0.0
                } else {
                    self.raster_total.as_secs_f64() * 1000. / self.uploads as f64
                };
                self.window.as_ref().unwrap().set_title(&format!("Stage present | {}x{} @ {:.2}x | present={} upload={} avg raster+draw={:.2}ms | focus={} outside-drag={}",g.config.width,g.config.height,self.scale,self.redraws,self.uploads,avg,self.focused,self.outside_drag));
                self.last_stat = Some(Instant::now())
            }
            _ => {}
        }
    }
}
fn main() {
    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Wait);
    el.run_app(&mut App::default()).unwrap();
}
