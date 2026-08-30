use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;

use anyrender::{PaintRef, PaintScene, ResourceId};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use peniko::kurbo::{Affine, Rect};
use peniko::{Fill, ImageBrush, ImageSampler};
use wgpu_context::DeviceHandle;

pub enum TriMessage {
    SetSpin(bool),
}

pub struct TriWidget {
    state: State,
    tx: Sender<TriMessage>,
    rx: Receiver<TriMessage>,
    spin: bool,
    angle: f32,
    last_tick: Instant,
}

enum State {
    Suspended,
    Active(Box<ActiveRenderer>),
}

impl TriWidget {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            state: State::Suspended,
            tx,
            rx,
            spin: true,
            angle: 0.0,
            last_tick: Instant::now(),
        }
    }

    pub fn sender(&self) -> Sender<TriMessage> {
        self.tx.clone()
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                TriMessage::SetSpin(spin) => self.spin = spin,
            }
        }
    }
}

impl Widget for TriWidget {
    fn connected(&mut self) {}
    fn disconnected(&mut self) {}

    fn can_create_surfaces(&mut self, render_ctx: &mut dyn anyrender::RenderContext) {
        if let Some(ctx) = render_ctx.renderer_specific_context() {
            if let Ok(device_handle) = ctx.downcast::<DeviceHandle>() {
                self.state = State::Active(Box::new(ActiveRenderer::new(&device_handle)));
            } else {
                println!("PROBE room=widget verdict=non-wgpu-backend");
            }
        } else {
            println!("PROBE room=widget verdict=no-renderer-context");
        }
    }

    fn destroy_surfaces(&mut self) {
        self.state = State::Suspended;
    }

    fn requires_redraw(&self) -> bool {
        self.spin
    }

    fn handle_event(&mut self, _event: &blitz_traits::events::UiEvent) {}

    fn paint(
        &mut self,
        render_ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> anyrender::Scene {
        self.process_messages();

        let now = Instant::now();
        if self.spin {
            self.angle += now.duration_since(self.last_tick).as_secs_f32();
        }
        self.last_tick = now;

        let mut scene = anyrender::Scene::new();
        let State::Active(renderer) = &mut self.state else {
            println!("PROBE room=widget verdict=paint-while-suspended");
            return scene;
        };
        if width == 0 || height == 0 {
            return scene;
        }

        let resource_id = renderer.render(render_ctx, self.angle, width, height);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Resource(ImageBrush {
                image: resource_id,
                sampler: ImageSampler::default(),
            }),
            None,
            &Rect::from_origin_size((0.0, 0.0), (width as f64, height as f64)),
        );
        scene
    }
}

struct TextureAndHandle {
    texture: wgpu::Texture,
    handle: ResourceId,
}

struct ActiveRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    displayed_texture: Option<TextureAndHandle>,
    next_texture: Option<TextureAndHandle>,
}

impl ActiveRenderer {
    fn new(device_handle: &DeviceHandle) -> Self {
        let device = &device_handle.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 16,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
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
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            device: device.clone(),
            queue: device_handle.queue.clone(),
            pipeline,
            displayed_texture: None,
            next_texture: None,
        }
    }

    fn render(
        &mut self,
        ctx: &mut dyn anyrender::RenderContext,
        angle: f32,
        width: u32,
        height: u32,
    ) -> ResourceId {
        if self
            .next_texture
            .as_ref()
            .is_some_and(|t| t.texture.width() != width || t.texture.height() != height)
        {
            let handle = self.next_texture.take().unwrap().handle;
            ctx.unregister_resource(handle);
        }

        let texture_and_handle = match &self.next_texture {
            Some(next) => next,
            None => {
                let texture = create_texture(&self.device, width, height);
                let handle = ctx
                    .try_register_custom_resource(Box::new(texture.clone()))
                    .expect("wgpu backend accepts wgpu textures");
                self.next_texture = Some(TextureAndHandle { texture, handle });
                self.next_texture.as_ref().unwrap()
            }
        };
        let texture = &texture_and_handle.texture;
        let handle = texture_and_handle.handle;

        let immediates = Immediates {
            angle_aspect: [angle, width as f32 / height as f32, 0.0, 0.0],
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.25,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                multiview_mask: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_immediates(0, bytemuck::bytes_of(&immediates));
            rpass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));

        std::mem::swap(&mut self.next_texture, &mut self.displayed_texture);
        handle
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Immediates {
    angle_aspect: [f32; 4],
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}
