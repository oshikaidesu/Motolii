//! M1-T12用の最小参照プラグイン群。
//!
//! Filter/CompositeはGPU render passだけを発行する。CPUピクセル処理の迂回路は持たない。

use std::sync::OnceLock;

use super::*;

pub static CLEAR_FILTER: ClearFilter = ClearFilter;
pub static TINT_FILTER: TintFilter = TintFilter;
pub static CLEAR_LAYER_SOURCE: ClearLayerSource = ClearLayerSource;
pub static CLEAR_COMPOSITE: ClearComposite = ClearComposite;

pub fn register_reference_plugins(registry: &mut PluginRegistry) -> Result<(), PluginError> {
    registry.register_layer_source(&CLEAR_LAYER_SOURCE)?;
    registry.register_filter(&CLEAR_FILTER)?;
    registry.register_filter(&TINT_FILTER)?;
    registry.register_composite(&CLEAR_COMPOSITE)?;
    Ok(())
}

pub fn register_reference_contracts(
    catalog: &mut PluginCatalogBuilder,
) -> Result<(), PluginContractError> {
    for (kind, node, migrations) in [
        (PluginKind::LayerSource, clear_layer_source_desc(), vec![]),
        (PluginKind::Filter, clear_filter_desc(), vec![]),
        (PluginKind::Filter, tint_filter_desc(), vec![]),
        (PluginKind::Composite, clear_composite_desc(), vec![]),
    ] {
        catalog.register(PluginContract {
            kind,
            node: node.clone(),
            migrations,
        })?;
    }
    Ok(())
}

pub fn reference_catalog() -> Result<PluginCatalog, PluginContractError> {
    let mut builder = PluginCatalogBuilder::new();
    register_reference_contracts(&mut builder)?;
    builder.build()
}

pub struct ClearFilter;

impl FilterPlugin for ClearFilter {
    fn desc(&self) -> &NodeDesc {
        clear_filter_desc()
    }

    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        _ctx: &RenderCtx,
        params: &ResolvedParams,
        _input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        clear_texture(
            encoder,
            output,
            color_from_params("core.filter.clear", params)?,
        );
        Ok(())
    }
}

/// PipelineCache実証用の実Filter(所見2/F-10)。入力をcolorで乗算する。
pub struct TintFilter;

impl FilterPlugin for TintFilter {
    fn desc(&self) -> &NodeDesc {
        tint_filter_desc()
    }

    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        _ctx: &RenderCtx,
        params: &ResolvedParams,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        use motolii_gpu::PipelineCacheKey;

        let cached = pipelines.get_or_create_tex_sample_uniform4(
            gpu,
            PipelineCacheKey {
                id: "core.filter.tint",
                wgsl: TINT_WGSL,
            },
        );
        // UI/APIのcolorはstraight。シェーダ側でunpremul→乗算→premulする。
        let [r, g, b, a] = params.require_color("core.filter.tint", "color")?;
        let color = [r as f32, g as f32, b as f32, a as f32];
        gpu.queue
            .write_buffer(&cached.uniform_buffer, 0, bytemuck::bytes_of(&color));
        let input_view = input
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // bind group / view は入力テクスチャがフレームごとに差し替わるため都度生成
        // (OverlayNodeと同じ。バッファ/パイプラインはキャッシュ済み)。
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("core.filter.tint.bg"),
            layout: &cached.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cached.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cached.uniform_buffer.as_entire_binding(),
                },
            ],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("core.filter.tint.pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&cached.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        Ok(())
    }
}

pub struct ClearLayerSource;

impl LayerSourcePlugin for ClearLayerSource {
    fn desc(&self) -> &NodeDesc {
        clear_layer_source_desc()
    }

    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        _t: RationalTime,
        params: &ResolvedParams,
        _ctx: LayerSourceContext,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        clear_texture(
            encoder,
            output,
            color_from_params("core.layer_source.clear", params)?,
        );
        Ok(())
    }
}

pub struct ClearComposite;

impl CompositePlugin for ClearComposite {
    fn desc(&self) -> &NodeDesc {
        clear_composite_desc()
    }

    fn render(
        &self,
        _gpu: &GpuCtx,
        _pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        _ctx: &RenderCtx,
        params: &ResolvedParams,
        _inputs: &[TextureRef<'_>],
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        clear_texture(
            encoder,
            output,
            color_from_params("core.composite.clear", params)?,
        );
        Ok(())
    }
}

fn clear_filter_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.filter.clear"),
        version: 1,
        display_name: "Clear",
        category: "Utility",
        tags: &["clear", "fill", "reference"],
        params: color_params(),
        min_inputs: 1,
        max_inputs: 1,
    })
}

fn tint_filter_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.filter.tint"),
        version: 1,
        display_name: "Tint",
        category: "Color",
        tags: &["tint", "color", "reference"],
        params: vec![ParamDef {
            id: "color",
            value_type: ValueType::Color,
            default: Value::Color([1.0, 1.0, 1.0, 1.0]),
            f64_domain: None,
        }],
        min_inputs: 1,
        max_inputs: 1,
    })
}

const TINT_WGSL: &str = r#"
struct TintUniform {
    color: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> tint: TintUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tin = textureSample(input_tex, tex_sampler, in.uv);
    let t = tint.color;
    let out_a = tin.a * t.a;
    let rgb = select(tin.rgb / max(tin.a, 1e-5), vec3<f32>(0.0), tin.a == 0.0) * t.rgb;
    return vec4<f32>(rgb * out_a, out_a);
}
"#;

fn clear_layer_source_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.layer_source.clear"),
        version: 1,
        display_name: "Clear Layer Source",
        category: "Generate",
        tags: &["clear", "fill", "reference"],
        params: color_params(),
        min_inputs: 0,
        max_inputs: 0,
    })
}

fn clear_composite_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.composite.clear"),
        version: 1,
        display_name: "Clear Composite",
        category: "Composite",
        tags: &["clear", "fill", "reference"],
        params: color_params(),
        min_inputs: 2,
        max_inputs: usize::MAX,
    })
}

fn color_params() -> Vec<ParamDef> {
    vec![ParamDef {
        id: "color",
        value_type: ValueType::Color,
        default: Value::Color([0.0, 0.0, 0.0, 0.0]),
        f64_domain: None,
    }]
}

fn color_from_params(plugin: &str, params: &ResolvedParams) -> Result<wgpu::Color, PluginError> {
    let [r, g, b, a] = params.require_color(plugin, "color")?;
    Ok(wgpu::Color { r, g, b, a })
}

fn clear_texture(encoder: &mut wgpu::CommandEncoder, output: TextureRef<'_>, color: wgpu::Color) {
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("motolii-plugin-clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        multiview_mask: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
}
