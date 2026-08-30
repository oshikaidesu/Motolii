
use std::collections::HashMap;

mod glow;
pub(crate) mod isf;
mod vism;
mod wgsl_fragment;

pub(crate) use glow::{GlowPipelines, GLOW_INTERMEDIATE_FORMAT};
pub use isf::{IsfInput, IsfInputType, IsfManifest};
pub(crate) use isf::{IsfProgram, BLOOM_SOURCE, ISF_TARGET_FORMAT};
pub(crate) use wgsl_fragment::{
    WgslFragmentProgram, GRADIENT_SOURCE, GRADIENT_TARGET_FORMAT, TRI_LED_SOURCE,
    TRI_LED_TARGET_FORMAT,
};

#[derive(Clone, Debug, PartialEq)]
pub enum EffectPass {
    Identity,
    Glow {
        threshold: f32,
        intensity: f32,
        radius: f32,
    },
    Isf {
        params: Vec<(String, f32)>,
    },
    Gradient,
    TriLed,
}

impl EffectPass {
    pub fn padding(&self) -> u32 {
        match self {
            EffectPass::Identity => 0,
            EffectPass::Glow { radius, .. } => {
                let step = radius.round().max(1.0) as u32;
                step * 2
            }
            EffectPass::Isf { .. } => 0,
            EffectPass::Gradient => 0,
            EffectPass::TriLed => 0,
        }
    }

    pub(crate) fn intermediate_format(&self) -> Option<wgpu::TextureFormat> {
        match self {
            EffectPass::Identity => None,
            EffectPass::Glow { .. } => Some(GLOW_INTERMEDIATE_FORMAT),
            EffectPass::Isf { .. } => Some(ISF_TARGET_FORMAT),
            EffectPass::Gradient => Some(GRADIENT_TARGET_FORMAT),
            EffectPass::TriLed => Some(TRI_LED_TARGET_FORMAT),
        }
    }
}

#[derive(Default)]
pub(crate) struct EffectScratch {
    free: HashMap<(u32, u32, wgpu::TextureFormat), Vec<wgpu::Texture>>,
    created: u64,
}

impl EffectScratch {
    pub(crate) fn acquire(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> wgpu::Texture {
        let key = (width, height, format);
        if let Some(pool) = self.free.get_mut(&key) {
            if let Some(texture) = pool.pop() {
                return texture;
            }
        }
        self.created += 1;
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-compositor-effect-scratch"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    pub(crate) fn release(
        &mut self,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        texture: wgpu::Texture,
    ) {
        self.free
            .entry((width, height, format))
            .or_default()
            .push(texture);
    }

    pub(crate) fn created_count(&self) -> u64 {
        self.created
    }
}
