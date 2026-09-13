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
fn target_format(manifest: &IsfManifest, name: &str) -> wgpu::TextureFormat {
    let pass = manifest.passes.iter().find(|p| p.target.as_deref() == Some(name));
    match pass.map(|p| (p.float, p.channels)) {
        Some((true, 1)) => wgpu::TextureFormat::R16Float,
        Some((true, 2)) => wgpu::TextureFormat::Rg16Float,
        Some((true, _)) => FLOAT_TARGET_FORMAT,
        _ => crate::render::compositor::BLEND_TARGET_FORMAT,
    }
}

pub(crate) fn pass_path(path: &std::path::Path, index: usize) -> std::path::PathBuf {
    path.with_extension(format!("pass-{index}.wgsl"))
}

pub(crate) fn specialize_pass(source: &str, params: usize, index: usize) -> String {
    source.replace(
        &format!("@group(1) @binding({}) var<uniform> pass_index: f32;", pass_index_binding(params)),
        &format!("const pass_index: f32 = {index}.0;"),
    )
}

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

/// 時計(`TIME` `TIMEDELTA` `FRAMEINDEX`、ISF 仕様)は pass_index と同じ 1 つの buffer に乗る
/// (x = 段, y = TIME, z = TIMEDELTA, w = FRAMEINDEX)。uniform buffer の数は shader ごとに上限があり
/// (turbulent_warp は既に 12 本)、束縛を 1 つ増やすと組めなくなる。
/// 時計の値は欄と同じ列で運ぶ(署名を増やさない)。この名前は ISF が予約しているので欄とは衝突しない。
pub(crate) const CLOCK_KEYS: [&str; 3] = ["TIME", "TIMEDELTA", "FRAMEINDEX"];

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

pub(crate) fn catalog_stage_path(name: &str, stage: &str) -> PathBuf {
    #[cfg(load_shaders_from_disk)]
    { std::env::temp_dir().join(format!("motolii-vism-runtime-{}", std::process::id())).join(format!("{name}-{stage}.wgsl")) }
    #[cfg(not(load_shaders_from_disk))]
    { PathBuf::from(format!("motolii-vism/{name}-{stage}.wgsl")) }
}

pub(crate) fn write_catalog_stage(path: &std::path::Path, text: &str) -> Result<(), String> {
    #[cfg(load_shaders_from_disk)]
    {
        std::fs::create_dir_all(path.parent().ok_or("shader stage has no parent")?).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("pending");
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        std::fs::rename(tmp, path).map_err(|e| e.to_string())
    }
    #[cfg(not(load_shaders_from_disk))]
    {
        use re_renderer::FileSystem;
        re_renderer::get_filesystem().create_file(path, text.to_owned().into()).map_err(|e| e.to_string())
    }
}

/// 言語ごとの入口が用意する物 — 上流のファイルシステムに載った WGSL の場所と入口名。
pub(crate) struct ShaderStageSource {
    pub(crate) path: PathBuf,
    pub(crate) entry_point: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ImageFrame {
    pub size: [f32; 2],
    pub origin: [f32; 2],
    pub pixels: [u32; 2],
}

impl ImageFrame {
    pub fn density(self) -> [f32; 2] { std::array::from_fn(|i| self.pixels[i] as f32 / self.size[i].max(1.0)) }
    pub fn padded(self, pixels: u32) -> Self {
        let density = self.density();
        Self {
            size: std::array::from_fn(|i| self.size[i] + 2.0 * pixels as f32 / density[i]),
            origin: std::array::from_fn(|i| self.origin[i] - pixels as f32 / density[i]),
            pixels: self.pixels.map(|n| n + pixels * 2),
        }
    }
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

    /// 論理 px で書かれた欄(`SUBTYPE` が DISTANCE / TRANSLATION)を、絵の密度で画素へ写す。
    /// shader は ISF の作法どおり画素で書き、密度は host が知っている(広がりの法)。
    pub(crate) fn params_at_density(&self, params: &[(String, f32)], density: f32) -> Vec<(String, f32)> {
        if (density - 1.0).abs() < 1e-6 { return params.to_vec(); }
        params.iter().map(|(name, value)| {
            // 成分の鍵 `name.1` も、その欄の性格で決める。
            let base = name.split_once('.').map_or(name.as_str(), |(base, tail)| if tail.chars().all(|c| c.is_ascii_digit()) { base } else { name.as_str() });
            let scaled = self.manifest.inputs.iter().any(|input| input.name == base
                && matches!(input.subtype.as_deref(), Some("DISTANCE" | "TRANSLATION")));
            (name.clone(), if scaled { value * density } else { *value })
        }).collect()
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
        let image_count = image_order.len() + manifest.target_slots().len();

        let mut texture_entries: Vec<wgpu::BindGroupLayoutEntry> =
            Vec::with_capacity(image_count * 2);
        for order_index in 0..image_count {
            let tex_binding = image_texture_binding(order_index);
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            texture_entries.push(wgpu::BindGroupLayoutEntry {
                binding: tex_binding + 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        if manifest.stage == super::IsfStage::Warp { param_entries.push(uniform_entry(param_order.len() as u32 + 2)); }
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
                .map(|pass| match pass.target.as_deref() {
                    None => output_format,
                    Some(name) => target_format(&manifest, name),
                })
                .collect()
        };
        let pipelines = pass_formats
            .iter()
            .enumerate()
            .map(|(index, format)| {
                let module = |stage: &str, path: &std::path::PathBuf| ctx.gpu_resources.shader_modules.get_or_create(ctx, &ShaderModuleDesc {
                    label: format!("{label}-{stage}-{index}").into(),
                    source: if manifest.specialize_passes { pass_path(path, index) } else { path.clone() },
                    extra_workaround_replacements: Vec::new(),
                });
                let vertex_handle = module("vertex", &vertex.path);
                let fragment_handle = module("fragment", &fragment.path);
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
            mag_filter: if manifest.linear_sampling { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest },
            min_filter: if manifest.linear_sampling { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest },
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
        self.record_in_frame(ctx, encoder, scratch, sources, dst_view, params, ImageFrame { size: render_size, origin: [0.0;2], pixels: render_size.map(|n| n.max(1.0) as u32) });
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_in_frame(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, scratch: &mut EffectScratch, sources: &[&wgpu::TextureView], dst_view: &wgpu::TextureView, params: &[(String,f32)], frame: ImageFrame) {
        self.record_feedback_in_frame(ctx, encoder, scratch, sources, dst_view, params, frame, None)
    }

    /// PERSISTENT な target は host の状態(`feedback`)の 2 枚を使う: 書く pass とその前の pass は
    /// 前のフレーム(`prev`)を読み、書く先は今のフレーム(`next`)。後の pass は `next` を読む。
    /// 状態が無い(鍵が刻まれていない)persistent は、毎フレーム透明を初期条件にする。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_feedback_in_frame(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, scratch: &mut EffectScratch, sources: &[&wgpu::TextureView], dst_view: &wgpu::TextureView, params: &[(String,f32)], frame: ImageFrame, feedback: Option<(&mut super::FeedbackState, super::FeedbackStep)>) {
        let device = &ctx.device;
        let queue = &ctx.queue;
        let extent = frame.pixels;
        let render_size = frame.size;

        // 中間ターゲットを借りる(宣言順 = 後続パスが読む順)。
        let slots = self.manifest.target_slots();
        let persistent = self.manifest.persistent_targets();
        let (mut state, mut step) = match feedback { Some((state, step)) => (Some(state), step), None => (None, super::FeedbackStep::Restart) };
        let clear = |encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView| {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("vism-feedback-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            drop(pass);
        };
        // (texture, view, format, 借り物か, 前のフレームの view)
        let mut targets: Vec<(wgpu::Texture, wgpu::TextureView, wgpu::TextureFormat, bool, Option<wgpu::TextureView>)> = Vec::new();
        for name in &slots {
            let format = target_format(&self.manifest, name);
            let declaration = self.manifest.passes.iter().find(|p| p.target.as_deref() == Some(name));
            let width = declaration.and_then(|p| p.width).map_or(extent[0], |v| v.resolve(extent[0]));
            let height = declaration.and_then(|p| p.height).map_or(extent[1], |v| v.resolve(extent[1]));
            if !persistent.contains(name) {
                let texture = scratch.acquire(device, width, height, format);
                let view = texture.create_view(&Default::default());
                targets.push((texture, view, format, true, None));
                continue;
            }
            match state.as_deref_mut() {
                Some(state) => {
                    let fits = state.targets.get(*name).is_some_and(|t| t.next.width() == width && t.next.height() == height && t.next.format() == format);
                    if !fits {
                        let make = || device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("motolii-feedback-state"), size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                            view_formats: &[],
                        });
                        state.targets.insert((*name).to_owned(), super::FeedbackTarget { prev: make(), next: make() });
                        // 寸法や形式が変われば履歴は続けられない: 初期条件から。
                        step = super::FeedbackStep::Restart;
                    }
                    let target = state.targets.get_mut(*name).expect("just ensured");
                    match step {
                        super::FeedbackStep::Advance => std::mem::swap(&mut target.prev, &mut target.next),
                        super::FeedbackStep::Restart => clear(encoder, &target.prev.create_view(&Default::default())),
                        super::FeedbackStep::Reuse => {}
                    }
                    let view = target.next.create_view(&Default::default());
                    let prev_view = target.prev.create_view(&Default::default());
                    targets.push((target.next.clone(), view, format, false, Some(prev_view)));
                }
                None => {
                    // 持ち主が無い: 前のフレームは透明(借り物を空にして読ませる)。
                    let prev = scratch.acquire(device, width, height, format);
                    let prev_view = prev.create_view(&Default::default());
                    clear(encoder, &prev_view);
                    let texture = scratch.acquire(device, width, height, format);
                    let view = texture.create_view(&Default::default());
                    targets.push((texture, view, format, true, Some(prev_view)));
                    scratch.release(width, height, format, prev);
                }
            }
        }
        // 各 slot を書く pass の番(persistent の読み分けに使う)。
        let writer_of: Vec<Option<usize>> = slots.iter().map(|name| self.manifest.passes.iter().position(|p| p.target.as_deref() == Some(name))).collect();

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
        for (_, view, _, _, _) in &targets {
            views.push(view);
        }

        // **同じパスの中で、書き込み先を読める形で束ねてはいけない**(WebGPU の使用衝突。
        // 束ねるとそのパスが丸ごと無効になり、絵が出ない)。書き込み先のスロットには
        // 代わりに入力の1枚目を挿しておく — シェーダは自分の出力先を読まない。
        // persistent な slot は別: 書く pass とその前は前のフレーム、後の pass は今のフレームを読む。
        let inputs = self.image_order.len();
        let bind_for_pass = |pass_index: usize, writing: Option<usize>| {
            let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(views.len() * 2);
            for (order_index, view) in views.iter().enumerate() {
                let slot = order_index.checked_sub(inputs);
                let prev = slot.and_then(|s| targets[s].4.as_ref());
                let bound = match (prev, slot.and_then(|s| writer_of[s])) {
                    (Some(prev), Some(writer)) if pass_index <= writer => prev,
                    _ => match writing {
                        Some(w) if w == order_index => views[0],
                        _ => view,
                    },
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
                self.manifest.passes.get(pass_index).map_or(render_size, |p| [p.width.map_or(render_size[0], |w| w.resolve(extent[0]) as f32), p.height.map_or(render_size[1], |h| h.resolve(extent[1]) as f32)]),
                pass_index as u32,
                frame.origin,
            );
            let writing = self
                .manifest
                .passes
                .get(pass_index)
                .and_then(|pass| pass.target.as_deref())
                .and_then(|name| slots.iter().position(|slot| *slot == name))
                .map(|slot| self.image_order.len() + slot);
            let target_view = match writing {
                Some(slot) => views[slot],
                None => dst_view,
            };
            let texture_bind = bind_for_pass(pass_index, writing);
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
            drop(pass);
        }

        // 記録し終えたので返す。次に借りた者のパスは、この後ろで実行される。状態の 2 枚は host の物。
        for (texture, _, format, borrowed, _) in targets {
            if borrowed { scratch.release(texture.width(), texture.height(), format, texture); }
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
        origin: [f32; 2],
    ) -> wgpu::BindGroup {
        let mut buffers: Vec<wgpu::Buffer> = Vec::with_capacity(self.param_order.len() + 2);
        for &index in &self.param_order {
            let input = &self.manifest.inputs[index];
            let count = input.ty.component_count().max(1);
            let mut components = input.default;
            for i in 0..count {
                let key = super::component_key(&input.name, i);
                if let Some((_, value)) = params.iter().find(|(name, _)| name == &key) {
                    components[i] = *value;
                }
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
        // (段, TIME, TIMEDELTA, FRAMEINDEX)。同梱の WGSL は先頭の f32 だけを pass_index として読む。
        let mut pass_info = [0u8; 16];
        pass_info[..4].copy_from_slice(&(pass_index as f32).to_le_bytes());
        for (i, key) in CLOCK_KEYS.iter().enumerate() {
            let v = params.iter().find(|(name, _)| name == key).map_or(0.0, |(_, v)| *v);
            pass_info[(i + 1) * 4..(i + 2) * 4].copy_from_slice(&v.to_le_bytes());
        }
        buffers.push(uniform_buffer(device, queue, "vism-pass-index", &pass_info));

        if self.manifest.stage == super::IsfStage::Warp {
            let mut bytes = [0u8; 8];
            bytes[..4].copy_from_slice(&origin[0].to_le_bytes());
            bytes[4..].copy_from_slice(&origin[1].to_le_bytes());
            buffers.push(uniform_buffer(device, queue, "vism-material-origin", &bytes));
        }


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
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
