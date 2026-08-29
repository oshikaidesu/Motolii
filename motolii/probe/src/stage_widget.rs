use std::sync::Arc;
use std::time::Instant;

use crate::playback::Clock;
use anyrender::{PaintRef, PaintScene, ResourceId};
use motolii_store::{KeyframeTrack, RationalTime, Value};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use peniko::kurbo::{Affine, Rect};
use peniko::{Fill, ImageBrush, ImageSampler};
use re_renderer::renderer::TestTriangleDrawData;
use re_renderer::view_builder::{Projection, TargetConfiguration, ViewBuilder};
use re_renderer::{MsaaMode, RenderConfig, RenderContext, Rgba, ViewBuilderId};
use wgpu_context::DeviceHandle;

pub struct StageWidget {
    state: State,
    start: Instant,
    frames: u64,
    clock: Arc<Clock>,
    /// fixtureのサビ歌詞 position(Bezier入り)。カメラの向きを駆動。
    sabi_position: Option<KeyframeTrack>,
    /// fixtureのタイトルロゴ opacity。カメラ距離を駆動。
    logo_opacity: Option<KeyframeTrack>,
}

enum State {
    Suspended,
    Active(Box<Active>),
}

struct Active {
    ctx: RenderContext,
    displayed: Option<TexAndHandle>,
    next: Option<TexAndHandle>,
    view_id: u64,
}

struct TexAndHandle {
    texture: wgpu::Texture,
    handle: ResourceId,
}

impl StageWidget {
    pub fn new(
        clock: Arc<Clock>,
        sabi_position: Option<KeyframeTrack>,
        logo_opacity: Option<KeyframeTrack>,
    ) -> Self {
        Self {
            state: State::Suspended,
            start: Instant::now(),
            frames: 0,
            clock,
            sabi_position,
            logo_opacity,
        }
    }
}

fn create_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("probe-stage-target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ViewBuilder::MAIN_TARGET_COLOR_FORMAT,
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
        match RenderContext::new_from_device(
            device_handle.device.clone(),
            device_handle.queue.clone(),
            ViewBuilder::MAIN_TARGET_COLOR_FORMAT,
            |_caps| RenderConfig { msaa_mode: MsaaMode::Off },
        ) {
            Ok(ctx) => {
                println!("PROBE room=stage verdict=re_renderer-context-up");
                self.state = State::Active(Box::new(Active {
                    ctx,
                    displayed: None,
                    next: None,
                    view_id: 0,
                }));
            }
            Err(e) => println!("PROBE room=stage verdict=context-error {e}"),
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
        if first {
            println!("PROBE room=stage verdict=first-paint {}x{}", width, height);
        }

        if active
            .next
            .as_ref()
            .is_some_and(|t| t.texture.width() != width || t.texture.height() != height)
        {
            let handle = active.next.take().unwrap().handle;
            render_ctx.unregister_resource(handle);
        }
        let tex_and_handle = match &active.next {
            Some(next) => next,
            None => {
                let texture = create_target(&active.ctx.device, width, height);
                let handle = render_ctx
                    .try_register_custom_resource(Box::new(texture.clone()))
                    .expect("wgpu backend accepts wgpu textures");
                active.next = Some(TexAndHandle { texture, handle });
                active.next.as_ref().unwrap()
            }
        };
        let target = tex_and_handle.texture.clone();
        let handle = tex_and_handle.handle;

        // fixtureのイージング済み値でカメラを駆動する——ガタつけば即見える。
        // サブフレーム評価(den=3000)で30fpsキーの間も連続に読む。
        let t_sec = self.clock.now_sec();
        let rt = RationalTime::try_new((t_sec * 3000.0) as i64, 3000)
            .unwrap_or(RationalTime::ZERO);
        let eased_x = match self.sabi_position.as_ref().map(|tr| tr.eval(rt)) {
            Some(Value::Vec2([x, _])) => x,
            _ => 960.0,
        };
        let eased_opacity = match self.logo_opacity.as_ref().map(|tr| tr.eval(rt)) {
            Some(Value::F64(v)) => v,
            _ => 1.0,
        };
        let azimuth = ((eased_x / 1920.0) - 0.5) as f32 * 2.4
            + self.start.elapsed().as_secs_f32() * 0.05;
        let dist = 4.5 + (1.0 - eased_opacity as f32) * 4.0;
        let eye = glam::Vec3::new(azimuth.sin() * dist, 2.5, azimuth.cos() * dist);
        let view_from_world = macaw::IsoTransform::look_at_rh(eye, glam::Vec3::ZERO, glam::Vec3::Y)
            .unwrap_or(macaw::IsoTransform::IDENTITY);

        let config = TargetConfiguration {
            name: "probe-stage".into(),
            resolution_in_pixel: [width, height],
            view_from_world,
            projection_from_view: Projection::Perspective {
                vertical_fov: 70.0 * std::f32::consts::TAU / 360.0,
                near_plane_distance: 0.01,
                aspect_ratio: width as f32 / height as f32,
            },
            ..Default::default()
        };

        active.ctx.begin_frame();
        let triangle = TestTriangleDrawData::new(&active.ctx);
        let mut view_builder = match ViewBuilder::new_with_external_resolved(
            &active.ctx,
            config,
            ViewBuilderId::new(active.view_id),
            &target,
        ) {
            Ok(vb) => vb,
            Err(e) => {
                println!("PROBE room=stage verdict=view-error {e}");
                return scene;
            }
        };
        active.view_id += 1;

        view_builder.queue_draw(&active.ctx, triangle);
        let command_buffer = match view_builder.draw(&active.ctx, Rgba::TRANSPARENT) {
            Ok(cb) => cb,
            Err(e) => {
                println!("PROBE room=stage verdict=draw-error {e}");
                return scene;
            }
        };
        active.ctx.before_submit();
        active.ctx.queue.submit([command_buffer]);
        if first {
            println!("PROBE room=stage verdict=first-submit-ok");
        }

        std::mem::swap(&mut active.next, &mut active.displayed);

        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Resource(ImageBrush { image: handle, sampler: ImageSampler::default() }),
            None,
            &Rect::from_origin_size((0.0, 0.0), (width as f64, height as f64)),
        );
        scene
    }
}
