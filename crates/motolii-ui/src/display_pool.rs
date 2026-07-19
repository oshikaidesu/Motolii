//! U1a-1: 表示専用の安定 texture view を一度だけ egui へ登録する private pool。

use std::sync::OnceLock;

use egui::TextureId;
use egui_wgpu::Renderer;
use motolii_core::FrameDesc;
use motolii_gpu::GpuCtx;
use motolii_render::RenderedFrame;

#[cfg(test)]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(test)]
static REGISTER_COUNT: AtomicU32 = AtomicU32::new(0);

pub(crate) struct DisplayPool {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    desc: FrameDesc,
    registered: OnceLock<TextureId>,
}

impl DisplayPool {
    pub(crate) fn new(gpu: &GpuCtx, desc: FrameDesc) -> Self {
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-ui-display-pool"),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            _texture: texture,
            view,
            desc,
            registered: OnceLock::new(),
        }
    }

    pub(crate) fn copy_from_rendered(&self, gpu: &GpuCtx, frame: &RenderedFrame) {
        assert_eq!(
            (self.desc.width, self.desc.height),
            (frame.desc.width, frame.desc.height),
            "display pool size must match rendered frame"
        );
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-ui-display-copy"),
            });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &frame.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self._texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: frame.desc.width,
                height: frame.desc.height,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit(std::iter::once(encoder.finish()));
    }

    pub(crate) fn desc(&self) -> FrameDesc {
        self.desc
    }

    pub(crate) fn stable_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// App 構築時（CreationContext）に一度だけ呼ぶ。
    pub(crate) fn register_once(
        &self,
        device: &wgpu::Device,
        renderer: &mut Renderer,
    ) -> TextureId {
        *self.registered.get_or_init(|| {
            #[cfg(test)]
            REGISTER_COUNT.fetch_add(1, Ordering::SeqCst);
            renderer.register_native_texture(device, self.stable_view(), wgpu::FilterMode::Linear)
        })
    }

    #[cfg(test)]
    pub(crate) fn register_count_for_test() -> u32 {
        REGISTER_COUNT.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn reset_register_count_for_test() {
        REGISTER_COUNT.store(0, Ordering::SeqCst);
    }
}
