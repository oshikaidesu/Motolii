use std::collections::HashMap;

pub(crate) mod isf;
pub(crate) mod vism;
mod wgsl_fragment;

pub(crate) use isf::IsfProgram;
pub use isf::{IsfInput, IsfInputType, IsfManifest};
pub(crate) use vism::FLOAT_TARGET_FORMAT;
pub(crate) use wgsl_fragment::{
    WgslFragmentProgram, BLEND_SOURCE, MATTE_SOURCE, VELLO_BLEND_PRELUDE,
};

#[derive(Clone, Copy)]
pub(crate) struct VismSource {
    pub(crate) name: &'static str,
    pub(crate) extension: &'static str,
    pub(crate) source: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/vism_inventory.rs"));

pub(crate) struct VismDefinition {
    pub(crate) source: VismSource,
    pub(crate) manifest: IsfManifest,
}

impl VismDefinition {
    pub(crate) fn plugin_id(&self) -> &str {
        self.manifest.id.as_deref().unwrap_or(self.source.name)
    }

    pub(crate) fn output_format(&self) -> wgpu::TextureFormat {
        if self.manifest.output_float {
            FLOAT_TARGET_FORMAT
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        }
    }
}

pub(crate) fn vism_definitions() -> &'static [VismDefinition] {
    static DEFINITIONS: std::sync::OnceLock<Vec<VismDefinition>> = std::sync::OnceLock::new();
    DEFINITIONS.get_or_init(|| {
        VISM_SOURCES
            .iter()
            .copied()
            .map(|source| VismDefinition {
                source,
                manifest: isf::parse_isf_source(source.source)
                    .unwrap_or_else(|error| panic!("{}: {error}", source.name))
                    .0,
            })
            .collect()
    })
}

pub(crate) enum EffectProgram {
    Wgsl(WgslFragmentProgram),
    Isf(IsfProgram),
}

impl EffectProgram {
    pub(crate) fn compile(
        ctx: &re_renderer::RenderContext,
        source: VismSource,
        output_format: wgpu::TextureFormat,
    ) -> Result<Self, isf::IsfError> {
        match source.extension {
            "wgsl" => Ok(Self::Wgsl(WgslFragmentProgram::compile(
                ctx,
                source.name,
                source.source,
                output_format,
            ))),
            "fs" => Ok(Self::Isf(IsfProgram::compile(
                ctx,
                source.source,
                output_format,
            )?)),
            _ => unreachable!("build.rs filters Vism extensions"),
        }
    }

    pub(crate) fn image_input_count(&self) -> usize {
        match self {
            Self::Wgsl(program) => program.image_input_count(),
            Self::Isf(program) => program.image_input_count(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        ctx: &re_renderer::RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        scratch: &mut EffectScratch,
        sources: &[&wgpu::TextureView],
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        match self {
            Self::Wgsl(program) => program.record_over(
                ctx,
                encoder,
                scratch,
                sources,
                dst_view,
                params,
                render_size,
            ),
            Self::Isf(program) => program.record(
                ctx,
                encoder,
                scratch,
                sources,
                dst_view,
                params,
                render_size,
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectPass {
    pub(crate) plugin_id: String,
    pub(crate) params: Vec<(String, f32)>,
    pub(crate) padding: u32,
    pub(crate) output_format: wgpu::TextureFormat,
}

impl EffectPass {
    pub fn padding(&self) -> u32 {
        self.padding
    }

    pub(crate) fn intermediate_format(&self) -> Option<wgpu::TextureFormat> {
        Some(self.output_format)
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
