//! Vism のプログラム — **シェーダ + 宣言された型付き入力**。
//!
//! 束縛(texture / sampler / param uniform)は**マニフェストが決める**。入口だけが
//! 言語ごとに分かれる(ISF の GLSL は naga を通り、`vism/*.wgsl` はそのまま)。
//! パイプラインもシェーダモジュールも上流(`re_renderer`)のプールから貰う。

use std::path::PathBuf;

use re_renderer::{
    BindGroupLayoutDesc, GpuBindGroupLayoutHandle, GpuRenderPipelineHandle, PipelineLayoutDesc,
    RenderContext, RenderPipelineDesc, ShaderModuleDesc,
};

use super::isf::{IsfInputType, IsfManifest};

/// 入力の並びから束縛番号を決める。image は texture と sampler の2つを使う。
pub(crate) fn image_texture_binding(image_index_in_order: usize) -> u32 {
    (image_index_in_order * 2) as u32
}

/// param uniform の後ろに描画サイズが1つ付く。
pub(crate) fn render_size_binding(param_count: usize) -> u32 {
    param_count as u32
}

/// 入力を image と param の2列へ分ける(束縛の並びの正本)。
pub(crate) fn orders(manifest: &IsfManifest) -> (Vec<usize>, Vec<usize>) {
    let mut images = Vec::new();
    let mut params = Vec::new();
    for (index, input) in manifest.inputs.iter().enumerate() {
        if input.ty == IsfInputType::Image {
            images.push(index);
        } else {
            params.push(index);
        }
    }
    (images, params)
}

/// 言語ごとの入口が用意する物 — 上流のファイルシステムに載った WGSL の場所と入口名。
pub(crate) struct ShaderStageSource {
    pub(crate) path: PathBuf,
    pub(crate) entry_point: String,
}

pub(crate) struct VismProgram {
    manifest: IsfManifest,
    pipeline: GpuRenderPipelineHandle,
    texture_layout: GpuBindGroupLayoutHandle,
    params_layout: GpuBindGroupLayoutHandle,
    sampler: wgpu::Sampler,
    image_order: Vec<usize>,
    param_order: Vec<usize>,
}

impl VismProgram {
    pub(crate) fn new(
        ctx: &RenderContext,
        label: &str,
        manifest: IsfManifest,
        vertex: ShaderStageSource,
        fragment: ShaderStageSource,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let device = &ctx.device;
        let (image_order, param_order) = orders(&manifest);

        let vertex_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: format!("{label}-vertex").into(),
                source: vertex.path,
                extra_workaround_replacements: Vec::new(),
            },
        );
        let fragment_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: format!("{label}-fragment").into(),
                source: fragment.path,
                extra_workaround_replacements: Vec::new(),
            },
        );

        let mut texture_entries: Vec<wgpu::BindGroupLayoutEntry> =
            Vec::with_capacity(image_order.len() * 2);
        for order_index in 0..image_order.len() {
            let tex_binding = image_texture_binding(order_index);
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        let texture_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            device,
            &BindGroupLayoutDesc {
                label: format!("{label}-texture-layout").into(),
                entries: texture_entries,
            },
        );

        let mut param_entries: Vec<wgpu::BindGroupLayoutEntry> = (0..param_order.len() as u32)
            .map(uniform_entry)
            .collect();
        param_entries.push(uniform_entry(render_size_binding(param_order.len())));
        let params_layout = ctx.gpu_resources.bind_group_layouts.get_or_create(
            device,
            &BindGroupLayoutDesc {
                label: format!("{label}-params-layout").into(),
                entries: param_entries,
            },
        );

        let pipeline_layout = ctx.gpu_resources.pipeline_layouts.get_or_create(
            ctx,
            &PipelineLayoutDesc {
                label: format!("{label}-pipeline-layout").into(),
                entries: vec![texture_layout, params_layout],
            },
        );

        let pipeline = ctx.gpu_resources.render_pipelines.get_or_create(
            ctx,
            &RenderPipelineDesc {
                label: format!("{label}-pipeline").into(),
                pipeline_layout,
                vertex_entrypoint: vertex.entry_point,
                vertex_handle,
                fragment_entrypoint: fragment.entry_point,
                fragment_handle,
                vertex_buffers: Default::default(),
                render_targets: re_renderer::external::smallvec::smallvec![Some(
                    wgpu::ColorTargetState {
                        format: output_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }
                )],
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            },
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{label}-sampler")),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            manifest,
            pipeline,
            texture_layout,
            params_layout,
            sampler,
            image_order,
            param_order,
        }
    }

    /// `sources` は宣言された image 入力と同じ並び。足りない分は最後の1枚を使う
    /// (image 入力が1つの効果に1枚だけ渡す、が最も多い)。
    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        sources: &[&wgpu::TextureView],
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        let device = &ctx.device;
        let queue = &ctx.queue;
        let bind_group_layouts = ctx.gpu_resources.bind_group_layouts.resources();
        let texture_layout = bind_group_layouts
            .get(self.texture_layout)
            .expect("vism texture bind group layout");
        let params_layout = bind_group_layouts
            .get(self.params_layout)
            .expect("vism params bind group layout");

        let mut texture_entries: Vec<wgpu::BindGroupEntry> =
            Vec::with_capacity(self.image_order.len() * 2);
        for order_index in 0..self.image_order.len() {
            let view = sources
                .get(order_index)
                .or_else(|| sources.last())
                .expect("image 入力を宣言した Vism には最低1枚要る");
            let tex_binding = image_texture_binding(order_index);
            texture_entries.push(wgpu::BindGroupEntry {
                binding: tex_binding,
                resource: wgpu::BindingResource::TextureView(view),
            });
            texture_entries.push(wgpu::BindGroupEntry {
                binding: tex_binding + 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            });
        }
        let texture_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vism-texture-bind"),
            layout: texture_layout,
            entries: &texture_entries,
        });

        let mut buffers: Vec<wgpu::Buffer> = Vec::with_capacity(self.param_order.len() + 1);
        for &index in &self.param_order {
            let input = &self.manifest.inputs[index];
            let count = input.ty.component_count().max(1);
            let mut components = input.default;
            if let Some((_, value)) = params.iter().find(|(name, _)| name == &input.name) {
                components[0] = *value;
            }
            let mut bytes = vec![0u8; count * 4];
            for i in 0..count {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&components[i].to_le_bytes());
            }
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vism-param"),
                size: bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, &bytes);
            buffers.push(buffer);
        }
        let render_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vism-render-info"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut render_info_bytes = [0u8; 8];
        render_info_bytes[0..4].copy_from_slice(&render_size[0].to_le_bytes());
        render_info_bytes[4..8].copy_from_slice(&render_size[1].to_le_bytes());
        queue.write_buffer(&render_info_buffer, 0, &render_info_bytes);
        buffers.push(render_info_buffer);

        let param_entries: Vec<wgpu::BindGroupEntry> = buffers
            .iter()
            .enumerate()
            .map(|(binding, buffer)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: buffer.as_entire_binding(),
            })
            .collect();
        let params_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vism-params-bind"),
            layout: params_layout,
            entries: &param_entries,
        });

        let render_pipelines = ctx.gpu_resources.render_pipelines.resources();
        let pipeline = render_pipelines.get(self.pipeline).expect("vism pipeline");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("vism-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &texture_bind, &[]);
        pass.set_bind_group(1, &params_bind, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
