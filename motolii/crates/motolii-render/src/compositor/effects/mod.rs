use std::collections::HashMap;

pub(crate) mod isf;
pub(crate) mod vism;
pub(crate) mod catalog;
pub(crate) mod surface_program;
mod wgsl_fragment;
pub mod subtype;

pub use isf::{IsfInput, IsfInputType, IsfManifest, IsfStage};
pub(crate) use vism::FLOAT_TARGET_FORMAT;
#[cfg(not(load_shaders_from_disk))]
pub(crate) use wgsl_fragment::VELLO_BLEND_PRELUDE;

#[cfg(not(load_shaders_from_disk))]
pub(crate) struct EmbeddedVismSource {
    name: &'static str,
    extension: &'static str,
    source: &'static str,
}

#[cfg(not(load_shaders_from_disk))]
pub(crate) struct EmbeddedVismPicture {
    file: &'static str,
    bytes: &'static [u8],
}

#[cfg(not(load_shaders_from_disk))]
include!(concat!(env!("OUT_DIR"), "/vism_inventory.rs"));

#[derive(Clone)]
pub(crate) struct VismSource {
    pub(crate) name: String,
    pub(crate) extension: String,
    pub(crate) source: std::sync::Arc<str>,
}

#[derive(Clone)]
pub(crate) struct VismDefinition {
    pub(crate) source: VismSource,
    pub(crate) manifest: IsfManifest,
    pub(crate) interface: String,
    pub(crate) vertex_text: String,
    pub(crate) fragment_text: String,
    pub(crate) vertex_entry: String,
    pub(crate) fragment_entry: String,
    /// 欄ごとの性格(manifest.param_inputs() と同じ並び)。
    pub(crate) subtypes: Vec<subtype::ParamSubtype>,
}

impl VismDefinition {
    pub(crate) fn plugin_id(&self) -> &str { self.manifest.id.as_deref().unwrap_or(&self.source.name) }
    /// 棚に出す名前: manifest の `LABEL`、無ければ file 名を Title Case に。
    pub(crate) fn label(&self) -> String {
        self.manifest.label.clone().unwrap_or_else(|| self.source.name.split(['_', '-']).map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        }).collect::<Vec<_>>().join(" "))
    }
    /// hook の snippet(stage が pass でない時)。
    pub(crate) fn hook_source(&self) -> &str { &self.vertex_text }
    pub(crate) fn output_format(&self) -> wgpu::TextureFormat {
        if matches!(self.source.name.as_str(), "blend" | "matte") { crate::render::compositor::BLEND_TARGET_FORMAT }
        else if self.manifest.output_float { FLOAT_TARGET_FORMAT } else { wgpu::TextureFormat::Rgba8Unorm }
    }
    pub(crate) fn paths(&self) -> [std::path::PathBuf; 2] {
        [vism::catalog_stage_path(&self.source.name, "vertex"), vism::catalog_stage_path(&self.source.name, "fragment")]
    }
    pub(crate) fn stage(&self) -> Result<(), String> {
        if !matches!(self.manifest.stage, IsfStage::Pass | IsfStage::Warp) {
            return Ok(()); // hook の snippet は MeshProgram が合成する時に書く。
        }
        let [vertex, fragment] = self.paths();
        for (path, source) in [(&vertex, &self.vertex_text), (&fragment, &self.fragment_text)] {
            vism::write_catalog_stage(path, source)?;
            if self.manifest.specialize_passes {
                for index in 0..self.manifest.passes.len() {
                    vism::write_catalog_stage(&vism::pass_path(path, index), &vism::specialize_pass(source, self.manifest.param_inputs().count(), index))?;
                }
            }
        }
        Ok(())
    }
}

pub(crate) struct EffectProgram(vism::VismProgram);

impl EffectProgram {
    pub(crate) fn compile(ctx: &re_renderer::RenderContext, definition: &VismDefinition) -> Self {
        Self::compile_for(ctx, definition, definition.output_format())
    }
    /// 宣言と違う出力 format で組む(同じ shader を別 format の texture へ描く時)。
    pub(crate) fn compile_for(ctx: &re_renderer::RenderContext, definition: &VismDefinition, output_format: wgpu::TextureFormat) -> Self {
        let [vertex, fragment] = definition.paths();
        Self(vism::VismProgram::new(ctx, &format!("motolii-vism-{}-{output_format:?}", definition.source.name), definition.manifest.clone(),
            vism::ShaderStageSource { path: vertex, entry_point: definition.vertex_entry.clone() },
            vism::ShaderStageSource { path: fragment, entry_point: definition.fragment_entry.clone() },
            output_format))
    }
    pub(crate) fn image_input_count(&self) -> usize { self.0.image_input_count() }
    pub(crate) fn params_at_density(&self, params: &[(String, f32)], density: f32) -> Vec<(String, f32)> { self.0.params_at_density(params, density) }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(&self, ctx: &re_renderer::RenderContext, encoder: &mut wgpu::CommandEncoder,
        scratch: &mut EffectScratch, sources: &[&wgpu::TextureView], dst_view: &wgpu::TextureView,
        params: &[(String, f32)], render_size: [f32; 2]) {
        self.0.record(ctx, encoder, scratch, sources, dst_view, params, render_size)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_in_frame(&self, ctx: &re_renderer::RenderContext, encoder: &mut wgpu::CommandEncoder,
        scratch: &mut EffectScratch, sources: &[&wgpu::TextureView], dst_view: &wgpu::TextureView,
        params: &[(String, f32)], frame: vism::ImageFrame) {
        self.0.record_in_frame(ctx, encoder, scratch, sources, dst_view, params, frame)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_over(&self, ctx: &re_renderer::RenderContext, encoder: &mut wgpu::CommandEncoder,
        scratch: &mut EffectScratch, sources: &[&wgpu::TextureView], dst_view: &wgpu::TextureView,
        params: &[(String, f32)], render_size: [f32; 2]) {
        self.record(ctx, encoder, scratch, sources, dst_view, params, render_size)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectPass {
    pub(crate) plugin_id: String,
    pub(crate) params: Vec<(String, f32)>,
    pub(crate) padding: u32,
    pub(crate) output_format: wgpu::TextureFormat,
    /// coverage 外の出力の混ぜ方(溢れの法)。
    pub(crate) spill: Option<crate::render::compositor::BlendMode>,
    /// 2 枚目以降の image が要る時刻のずれ(秒。負が過去)。ホストがその時刻の絵を渡す。
    pub(crate) image_time_offsets: Vec<f32>,
    /// 時計(`TIME` 系)を読む。合成側が記録の直前に時計の値を欄の列へ足す。
    pub(crate) uses_clock: bool,
}

/// 多成分の欄(点・色)は、成分ごとに 1 つの f32 として運ぶ。0 番は欄の名前そのまま、
/// 1 番以降は `name.1` `name.2` `name.3`。書く側(translate)と読む側(vism)はこの 1 つを使う。
pub(crate) fn component_key(name: &str, component: usize) -> String {
    if component == 0 { name.to_owned() } else { format!("{name}.{component}") }
}

impl EffectPass {
    pub fn padding(&self) -> u32 {
        self.padding
    }

    /// 2 枚目以降の image が要る時刻のずれ(秒。負が過去)。
    pub fn image_time_offsets(&self) -> &[f32] {
        &self.image_time_offsets
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
