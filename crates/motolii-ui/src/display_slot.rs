//! 静止preview用の独立display textureとregister-once境界。

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::OnceLock;

use egui::TextureId;
use egui_wgpu::Renderer;
use motolii_core::{FrameDesc, PixelFormat};
use motolii_gpu::GpuCtx;
use motolii_render::RenderedFrame;

static NEXT_SLOT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DisplaySlotEvidence {
    pub(crate) slot_id: u64,
    pub(crate) copy_count: u32,
    pub(crate) registration_count: u32,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DisplaySlotError {
    #[error("display slot requires Rgba8Unorm, got {0:?}")]
    UnsupportedFormat(PixelFormat),
    #[error("display slot descriptor mismatch: expected {expected:?}, got {actual:?}")]
    DescriptorMismatch {
        expected: FrameDesc,
        actual: FrameDesc,
    },
}

pub(crate) struct DisplaySlot {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    desc: FrameDesc,
    slot_id: u64,
    copy_count: AtomicU32,
    registration_count: AtomicU32,
    texture_id: OnceLock<TextureId>,
}

impl DisplaySlot {
    pub(crate) fn copy_from_rendered(
        gpu: &GpuCtx,
        rendered: &RenderedFrame,
    ) -> Result<Self, DisplaySlotError> {
        if rendered.desc.format != PixelFormat::Rgba8Unorm {
            return Err(DisplaySlotError::UnsupportedFormat(rendered.desc.format));
        }
        let desc = rendered.desc;
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-ui-static-display-slot"),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let slot = Self {
            texture,
            view,
            desc,
            slot_id: NEXT_SLOT_ID.fetch_add(1, Ordering::Relaxed),
            copy_count: AtomicU32::new(0),
            registration_count: AtomicU32::new(0),
            texture_id: OnceLock::new(),
        };
        slot.copy(gpu, rendered)?;
        Ok(slot)
    }

    fn copy(&self, gpu: &GpuCtx, rendered: &RenderedFrame) -> Result<(), DisplaySlotError> {
        if self.desc != rendered.desc {
            return Err(DisplaySlotError::DescriptorMismatch {
                expected: self.desc,
                actual: rendered.desc,
            });
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-ui-static-display-copy"),
            });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &rendered.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.desc.width,
                height: self.desc.height,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit(std::iter::once(encoder.finish()));
        self.copy_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub(crate) fn register_once(
        &self,
        device: &wgpu::Device,
        renderer: &mut Renderer,
    ) -> TextureId {
        *self.texture_id.get_or_init(|| {
            self.registration_count.fetch_add(1, Ordering::Relaxed);
            renderer.register_native_texture(device, &self.view, wgpu::FilterMode::Linear)
        })
    }

    pub(crate) fn desc(&self) -> FrameDesc {
        self.desc
    }

    pub(crate) fn evidence(&self) -> DisplaySlotEvidence {
        DisplaySlotEvidence {
            slot_id: self.slot_id,
            copy_count: self.copy_count.load(Ordering::Relaxed),
            registration_count: self.registration_count.load(Ordering::Relaxed),
        }
    }

    #[cfg(test)]
    pub(crate) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
