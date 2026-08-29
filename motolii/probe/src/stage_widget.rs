use std::sync::{Arc, Mutex};

use crate::playback::Clock;
use anyrender::{PaintRef, PaintScene, ResourceId};
use motolii_store::{Document, RationalTime};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use motolii_engine::Engine;
use peniko::kurbo::{Affine, Rect};
use peniko::{Fill, ImageBrush, ImageSampler};
use wgpu_context::DeviceHandle;

pub struct StageWidget {
    state: State,
    frames: u64,
    clock: Arc<Clock>,
    doc: Arc<Mutex<Document>>,
}

enum State {
    Suspended,
    Active(Box<Active>),
}

struct Active {
    engine: Engine,
    displayed: Option<TexAndHandle>,
    next: Option<TexAndHandle>,
}

struct TexAndHandle {
    texture: wgpu::Texture,
    handle: ResourceId,
}

impl StageWidget {
    pub fn new(clock: Arc<Clock>, doc: Arc<Mutex<Document>>) -> Self {
        Self { state: State::Suspended, frames: 0, clock, doc }
    }
}

fn create_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("probe-stage-target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // motolii-compositor::presentable::PRESENTABLE_FORMAT(render_frame_intoの
        // check_presentable_targetが要求する format)と同じ Rgba8UnormSrgb。
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // COPY_SRC: anyrender_velloは登録textureをcopyで取り込む。無いとsubmit全体が検証エラーで落ちる。
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

impl Widget for StageWidget {
    fn connected(&mut self) {}
    fn disconnected(&mut self) {}

    fn can_create_surfaces(&mut self, render_ctx: &mut dyn anyrender::RenderContext) {
        let Some(ctx) = render_ctx.renderer_specific_context() else {
            println!("PROBE room=stage verdict=no-renderer-context");
            return;
        };
        let Ok(device_handle) = ctx.downcast::<DeviceHandle>() else {
            println!("PROBE room=stage verdict=non-wgpu-backend");
            return;
        };
        match Engine::with_device(device_handle.device.clone(), device_handle.queue.clone()) {
            Ok(engine) => {
                println!("PROBE room=stage verdict=engine-up");
                self.state = State::Active(Box::new(Active { engine, displayed: None, next: None }));
            }
            Err(e) => println!("PROBE room=stage verdict=engine-error {e}"),
        }
    }

    fn destroy_surfaces(&mut self) {
        println!("PROBE room=stage verdict=destroy-surfaces");
        self.state = State::Suspended;
    }

    fn requires_redraw(&self) -> bool {
        true
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
        let mut scene = anyrender::Scene::new();
        self.frames += 1;
        let first = self.frames == 1;
        let State::Active(active) = &mut self.state else {
            if first {
                println!("PROBE room=stage verdict=paint-while-suspended");
            }
            return scene;
        };
        if width == 0 || height == 0 {
            if first {
                println!("PROBE room=stage verdict=zero-size");
            }
            return scene;
        }

        let doc = self.doc.lock().unwrap();
        let view = doc.view();
        let Some(composition) = view.composition().ok().flatten() else {
            if first {
                println!("PROBE room=stage verdict=no-composition");
            }
            return scene;
        };
        let (cw, ch) = (composition.width, composition.height);

        if first {
            println!("PROBE room=stage verdict=first-paint {}x{} comp={}x{}", width, height, cw, ch);
        }

        if active.next.as_ref().is_some_and(|t| t.texture.width() != cw || t.texture.height() != ch) {
            let handle = active.next.take().unwrap().handle;
            render_ctx.unregister_resource(handle);
        }
        let tex_and_handle = match &active.next {
            Some(next) => next,
            None => {
                let texture = create_target(active.engine.gpu_device(), cw, ch);
                let handle = render_ctx
                    .try_register_custom_resource(Box::new(texture.clone()))
                    .expect("wgpu backend accepts wgpu textures");
                active.next = Some(TexAndHandle { texture, handle });
                active.next.as_ref().unwrap()
            }
        };
        let target = tex_and_handle.texture.clone();
        let handle = tex_and_handle.handle;

        let t_sec = self.clock.now_sec();
        let rt = RationalTime::try_new((t_sec * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO);

        if let Err(e) = active.engine.render_frame_into(&view, rt, &target) {
            println!("PROBE room=stage verdict=render-error {e}");
            return scene;
        }
        drop(view);
        drop(doc);
        if first {
            println!("PROBE room=stage verdict=first-submit-ok");
        }

        std::mem::swap(&mut active.next, &mut active.displayed);

        let (w, h) = (width as f64, height as f64);
        let (cw, ch) = (target.width() as f64, target.height() as f64);
        let s = (w / cw).min(h / ch);
        let (fw, fh) = (cw * s, ch * s);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Resource(ImageBrush { image: handle, sampler: ImageSampler::default() }),
            None,
            &Rect::from_origin_size(((w - fw) * 0.5, (h - fh) * 0.5), (fw, fh)),
        );
        scene
    }
}
