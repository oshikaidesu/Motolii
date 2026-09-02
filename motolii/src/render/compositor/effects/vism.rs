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
use super::EffectScratch;

/// 入力の並びから束縛番号を決める。image は texture と sampler の2つを使う。
pub(crate) fn image_texture_binding(image_index_in_order: usize) -> u32 {
    (image_index_in_order * 2) as u32
}

/// param uniform の後ろに描画サイズが1つ付く。
pub(crate) fn render_size_binding(param_count: usize) -> u32 {
    param_count as u32
}

/// その後ろが `PASSINDEX`(ISF 仕様)。
pub(crate) fn pass_index_binding(param_count: usize) -> u32 {
    param_count as u32 + 1
}

/// `FLOAT` を宣言した中間ターゲットの形式(蓄積・HDR)。
pub(crate) const FLOAT_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

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

/// ホットリロード付きビルドでは、生成した WGSL を**実ファイル**として置く必要がある
/// (上流の resolver がディスクから読む)。中身で名前を決め、書き込みは一時ファイル
/// →rename で原子的に行う。**固定の1枚へ書くと、複数の Compositor を同時に作った時に
/// 半端な中身を読んでしまう**(テストが並列で走ると実際に起きた)。
#[cfg(load_shaders_from_disk)]
pub(crate) fn stage_source_on_disk(name: &str, text: &str) -> PathBuf {
    use std::hash::{Hash as _, Hasher as _};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let stamp = hasher.finish();

    let dir = std::env::temp_dir().join("motolii-vism-wgsl");
    std::fs::create_dir_all(&dir).expect("temp dir を作れる");
    let path = dir.join(format!("{name}-{stamp:016x}.wgsl"));
    if !path.exists() {
        let tmp = dir.join(format!("{name}-{stamp:016x}.{}.tmp", std::process::id()));
        std::fs::write(&tmp, text).expect("temp へ書ける");
        // rename は同一ディレクトリなら原子的。読み手が半端な中身を見ない
        let _ = std::fs::rename(&tmp, &path);
    }
    path
}

/// 言語ごとの入口が用意する物 — 上流のファイルシステムに載った WGSL の場所と入口名。
pub(crate) struct ShaderStageSource {
    pub(crate) path: PathBuf,
    pub(crate) entry_point: String,
}

pub(crate) struct VismProgram {
    manifest: IsfManifest,
    /// パスごとに1本(出力の形式が違う)。`PASSES` が無ければ1本だけ。
    pipelines: Vec<GpuRenderPipelineHandle>,
    texture_layout: GpuBindGroupLayoutHandle,
    params_layout: GpuBindGroupLayoutHandle,
    sampler: wgpu::Sampler,
    image_order: Vec<usize>,
    param_order: Vec<usize>,
}

impl VismProgram {
    pub(crate) fn image_input_count(&self) -> usize {
        self.image_order.len()
    }

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
        // 読める image = 宣言された入力 + **中間ターゲット**(後続のパスが名前で読む)。
        let target_count = manifest
            .passes
            .iter()
            .filter(|p| p.target.is_some())
            .count();
        let image_count = image_order.len() + target_count;

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
            Vec::with_capacity(image_count * 2);
        for order_index in 0..image_count {
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

        let mut param_entries: Vec<wgpu::BindGroupLayoutEntry> =
            (0..param_order.len() as u32).map(uniform_entry).collect();
        param_entries.push(uniform_entry(render_size_binding(param_order.len())));
        // PASSINDEX(ISF 仕様。何段目かをシェーダへ渡す)
        param_entries.push(uniform_entry(pass_index_binding(param_order.len())));
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

        let pass_formats: Vec<wgpu::TextureFormat> = if manifest.passes.is_empty() {
            vec![output_format]
        } else {
            manifest
                .passes
                .iter()
                .map(|pass| match (&pass.target, pass.float) {
                    (None, _) => output_format,
                    (Some(_), true) => FLOAT_TARGET_FORMAT,
                    (Some(_), false) => crate::render::compositor::BLEND_TARGET_FORMAT,
                })
                .collect()
        };
        let pipelines = pass_formats
            .iter()
            .enumerate()
            .map(|(index, format)| {
                ctx.gpu_resources.render_pipelines.get_or_create(
                    ctx,
                    &RenderPipelineDesc {
                        label: format!("{label}-pipeline-{index}").into(),
                        pipeline_layout,
                        vertex_entrypoint: vertex.entry_point.clone(),
                        vertex_handle,
                        fragment_entrypoint: fragment.entry_point.clone(),
                        fragment_handle,
                        vertex_buffers: Default::default(),
                        render_targets: re_renderer::external::smallvec::smallvec![Some(
                            wgpu::ColorTargetState {
                                format: *format,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            }
                        )],
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                    },
                )
            })
            .collect();

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
            pipelines,
            texture_layout,
            params_layout,
            sampler,
            image_order,
            param_order,
        }
    }

    /// 宣言された image 入力に `sources` を順に渡し、`PASSES` があればその段数だけ
    /// 描く。中間ターゲットは Host(`EffectScratch`)から借りて、記録し終えたら返す。
    /// **フレームを跨いで持ち越さない**(裁定: StatefulFilter 拒否)。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        scratch: &mut EffectScratch,
        sources: &[&wgpu::TextureView],
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        let device = &ctx.device;
        let queue = &ctx.queue;
        let extent = [
            render_size[0].max(1.0) as u32,
            render_size[1].max(1.0) as u32,
        ];

        // 中間ターゲットを借りる(宣言順 = 後続パスが読む順)。
        let mut targets: Vec<(wgpu::Texture, wgpu::TextureView, wgpu::TextureFormat)> = Vec::new();
        for pass in self.manifest.passes.iter().filter(|p| p.target.is_some()) {
            let format = if pass.float {
                FLOAT_TARGET_FORMAT
            } else {
                crate::render::compositor::BLEND_TARGET_FORMAT
            };
            let texture = scratch.acquire(device, extent[0], extent[1], format);
            let view = texture.create_view(&Default::default());
            targets.push((texture, view, format));
        }

        let bind_group_layouts = ctx.gpu_resources.bind_group_layouts.resources();
        let texture_layout = bind_group_layouts
            .get(self.texture_layout)
            .expect("vism texture bind group layout");
        let params_layout = bind_group_layouts
            .get(self.params_layout)
            .expect("vism params bind group layout");
        let render_pipelines = ctx.gpu_resources.render_pipelines.resources();

        // 読める image = 入力 + 中間ターゲット。並びは宣言順で固定。
        let mut views: Vec<&wgpu::TextureView> = Vec::new();
        for order_index in 0..self.image_order.len() {
            let view = sources
                .get(order_index)
                .or_else(|| sources.last())
                .expect("image 入力を宣言した Vism には最低1枚要る");
            views.push(view);
        }
        for (_, view, _) in &targets {
            views.push(view);
        }

        // **同じパスの中で、書き込み先を読める形で束ねてはいけない**(WebGPU の使用衝突。
        // 束ねるとそのパスが丸ごと無効になり、絵が出ない)。書き込み先のスロットには
        // 代わりに入力の1枚目を挿しておく — シェーダは自分の出力先を読まない。
        let bind_for_pass = |writing: Option<usize>| {
            let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(views.len() * 2);
            for (order_index, view) in views.iter().enumerate() {
                let bound = match writing {
                    Some(w) if w == order_index => views[0],
                    _ => view,
                };
                let tex_binding = image_texture_binding(order_index);
                entries.push(wgpu::BindGroupEntry {
                    binding: tex_binding,
                    resource: wgpu::BindingResource::TextureView(bound),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: tex_binding + 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                });
            }
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("vism-texture-bind"),
                layout: texture_layout,
                entries: &entries,
            })
        };

        let pass_count = self.pipelines.len();
        for pass_index in 0..pass_count {
            let params_bind = self.params_bind_group(
                device,
                queue,
                params_layout,
                params,
                render_size,
                pass_index as u32,
            );
            let writing = match self.manifest.passes.get(pass_index) {
                Some(pass) if pass.target.is_some() => Some(
                    self.image_order.len()
                        + self.manifest.passes[..pass_index]
                            .iter()
                            .filter(|p| p.target.is_some())
                            .count(),
                ),
                _ => None,
            };
            let target_view = match writing {
                Some(slot) => views[slot],
                None => dst_view,
            };
            let texture_bind = bind_for_pass(writing);
            let pipeline = render_pipelines
                .get(self.pipelines[pass_index])
                .expect("vism pipeline");

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("vism-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
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

        // 記録し終えたので返す。次に借りた者のパスは、この後ろで実行される。
        for (texture, _, format) in targets {
            scratch.release(extent[0], extent[1], format, texture);
        }
    }

    fn params_bind_group(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        params: &[(String, f32)],
        render_size: [f32; 2],
        pass_index: u32,
    ) -> wgpu::BindGroup {
        let mut buffers: Vec<wgpu::Buffer> = Vec::with_capacity(self.param_order.len() + 2);
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
            buffers.push(uniform_buffer(device, queue, "vism-param", &bytes));
        }
        let mut render_info = [0u8; 8];
        render_info[0..4].copy_from_slice(&render_size[0].to_le_bytes());
        render_info[4..8].copy_from_slice(&render_size[1].to_le_bytes());
        buffers.push(uniform_buffer(
            device,
            queue,
            "vism-render-info",
            &render_info,
        ));
        buffers.push(uniform_buffer(
            device,
            queue,
            "vism-pass-index",
            &(pass_index as f32).to_le_bytes(),
        ));

        let entries: Vec<wgpu::BindGroupEntry> = buffers
            .iter()
            .enumerate()
            .map(|(binding, buffer)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: buffer.as_entire_binding(),
            })
            .collect();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vism-params-bind"),
            layout,
            entries: &entries,
        })
    }
}

fn uniform_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    bytes: &[u8],
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, bytes);
    buffer
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
