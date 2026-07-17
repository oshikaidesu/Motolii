//! `core.filter.opacity` version 1 — 外部参照plugin crate実証(VSM-A1-3)。

use std::sync::OnceLock;

use motolii_plugin::bytemuck;
use motolii_plugin::wgpu;
use motolii_plugin::F64Domain;
use motolii_plugin::FilterPlugin;
use motolii_plugin::GpuCtx;
use motolii_plugin::NodeDesc;
use motolii_plugin::ParamDef;
use motolii_plugin::PipelineCache;
use motolii_plugin::PipelineCacheKey;
use motolii_plugin::PluginContract;
use motolii_plugin::PluginError;
use motolii_plugin::PluginId;
use motolii_plugin::PluginKind;
use motolii_plugin::RenderCtx;
use motolii_plugin::ResolvedParams;
use motolii_plugin::TextureRef;
use motolii_plugin::Value;
use motolii_plugin::ValueType;

/// INF-7g 実演: LLMが new-plugin 型紙から肉付けした参照Filter。
pub static OPACITY_FILTER: OpacityFilter = OpacityFilter;

pub fn opacity_contract() -> PluginContract {
    PluginContract {
        kind: PluginKind::Filter,
        node: opacity_filter_desc().clone(),
        migrations: vec![],
    }
}

/// 不透明度乗算(INF-7g)。premul RGBA 全体に `amount` を掛ける。
pub struct OpacityFilter;

impl FilterPlugin for OpacityFilter {
    fn desc(&self) -> &NodeDesc {
        opacity_filter_desc()
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
        let cached = pipelines.get_or_create_tex_sample_uniform4(
            gpu,
            PipelineCacheKey {
                id: "core.filter.opacity",
                wgsl: OPACITY_WGSL,
            },
        );
        let amount = params
            .require_f64("core.filter.opacity", "amount")?
            .clamp(0.0, 1.0) as f32;
        let uniform = [amount, 0.0, 0.0, 0.0];
        gpu.queue
            .write_buffer(&cached.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
        let input_view = input
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("core.filter.opacity.bg"),
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
                label: Some("core.filter.opacity.pass"),
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

fn opacity_filter_desc() -> &'static NodeDesc {
    static DESC: OnceLock<NodeDesc> = OnceLock::new();
    DESC.get_or_init(|| NodeDesc {
        id: PluginId("core.filter.opacity"),
        version: 1,
        display_name: "Opacity",
        category: "Color",
        tags: &["opacity", "alpha", "reference"],
        params: vec![ParamDef {
            id: "amount",
            value_type: ValueType::F64,
            default: Value::F64(1.0),
            f64_domain: Some(F64Domain::unit()),
        }],
        min_inputs: 1,
        max_inputs: 1,
    })
}

const OPACITY_WGSL: &str = r#"
struct OpacityUniform {
    // x = amount (0..1). yzw unused (tex_sample_uniform4 スロットに合わせる)。
    amount: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> opacity: OpacityUniform;

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
    return tin * opacity.amount.x;
}
"#;
