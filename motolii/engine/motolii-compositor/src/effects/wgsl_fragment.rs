//! 外から来た WGSL フラグメント関数1本を、GLSL 変換を経ずにそのまま
//! `re_renderer` の GPU 資源プールへ載せる経路。`effects::isf` は
//! `naga::front::glsl` で GLSL → WGSL テキストへ変換してから
//! `ctx.gpu_resources.shader_modules` へ渡す——ここは元から WGSL テキストなので
//! その変換段を持たない。プールへ渡す先(`get_or_create` の4口)は `isf` と同じ形。
//!
//! 書き手が書くのは `fs_main` 1本だけ(ISF の `.fs` と同じ契約——vertex 段と
//! フルスクリーン三角形はホストが用意する)。今回の書き手は uniform も
//! texture も1つも読まないので、bind group layout は空のまま
//! (`PipelineLayoutDesc.entries: vec![]`)。

use std::path::PathBuf;

use re_renderer::{GpuRenderPipelineHandle, PipelineLayoutDesc, RenderContext, RenderPipelineDesc, ShaderModuleDesc};
#[cfg(load_shaders_from_disk)]
use re_renderer::{FileServer, new_recommended_file_resolver};
#[cfg(not(load_shaders_from_disk))]
use re_renderer::{FileSystem as _, get_filesystem};

/// vgpu(vercel-labs)の `gradient` 例の `fs_main`(そのまま)を、ホスト側の
/// フルスクリーン三角形 vertex 段と合体させた1ファイル。`vs_main` が出す
/// `@location(0) uv` は三角形の頂点位置を `[0,1]` へ写しただけ。
///
/// `load_shaders_from_disk` が立たないビルド(release/wasm)向けの埋め込み——
/// 実ファイルは `shaders/gradient.wgsl`。
pub(crate) const GRADIENT_SOURCE: &str = include_str!("shaders/gradient.wgsl");

pub(crate) const GRADIENT_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// vgpu(vercel-labs)の Triangle LED Hero を1パスへ畳んだもの(`shaders/tri_led.wgsl`
/// doc 参照)。`GRADIENT_SOURCE` と同じ形——`fs_main` は uniform も texture も読まない。
pub(crate) const TRI_LED_SOURCE: &str = include_str!("shaders/tri_led.wgsl");

pub(crate) const TRI_LED_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// GPU に建った1本の WGSL フラグメント program。`effects::isf::IsfProgram` と
/// 同じ「初回生成して以後使い回す」規律——texture/uniform を持たないので
/// `record` は bind group を1つも作らない。
pub(crate) struct WgslFragmentProgram {
    pipeline: GpuRenderPipelineHandle,
}

impl WgslFragmentProgram {
    pub(crate) fn compile(
        ctx: &RenderContext,
        // shader ごとの仮想パス/label を分けるための一意名(`"gradient"`/`"tri_led"`)。
        // 同じ名前を2つの program に使うと `MemFileSystem`/pipeline キャッシュが
        // 内容ではなくパスでキャッシュするため、片方が他方を上書きしてしまう。
        name: &str,
        #[cfg_attr(load_shaders_from_disk, allow(unused_variables))] wgsl_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        // `ShaderModuleDesc::source` はファイル参照。`load_shaders_from_disk` が
        // 立つビルド(native debug)では実ファイルを `FileServer` に watch させ、
        // 保存のたびに `RenderContext::begin_frame` が拾い直す(re_renderer 側の
        // 仕組み、ここでは触らない)。立たないビルドでは `MemFileSystem` へ
        // `wgsl_source`(= `include_str!` で埋め込み済み)を書き込むだけ。
        #[cfg(load_shaders_from_disk)]
        let path = {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let abs_path = manifest_dir.join(format!("src/effects/shaders/{name}.wgsl"));
            let resolver = new_recommended_file_resolver();
            FileServer::get_mut(|fs| fs.watch(&resolver, &abs_path, false))
                .expect("{name}.wgsl exists next to wgsl_fragment.rs")
        };
        #[cfg(not(load_shaders_from_disk))]
        let path = {
            let path = PathBuf::from(format!("motolii-compositor/wgsl-fragment/{name}.wgsl"));
            get_filesystem()
                .create_file(&path, wgsl_source.to_owned().into())
                .expect("wgsl fragment source is valid utf8");
            path
        };

        let shader_handle = ctx.gpu_resources.shader_modules.get_or_create(
            ctx,
            &ShaderModuleDesc {
                label: format!("motolii-compositor-wgsl-fragment-{name}").into(),
                source: path,
                extra_workaround_replacements: Vec::new(),
            },
        );

        let pipeline_layout = ctx.gpu_resources.pipeline_layouts.get_or_create(
            ctx,
            &PipelineLayoutDesc {
                label: format!("motolii-compositor-wgsl-fragment-{name}-pipeline-layout").into(),
                // texture も uniform も読まない shader なので bind group layout は無い。
                entries: vec![],
            },
        );

        let pipeline = ctx.gpu_resources.render_pipelines.get_or_create(
            ctx,
            &RenderPipelineDesc {
                label: format!("motolii-compositor-wgsl-fragment-{name}-pipeline").into(),
                pipeline_layout,
                vertex_entrypoint: "vs_main".to_owned(),
                vertex_handle: shader_handle,
                fragment_entrypoint: "fs_main".to_owned(),
                fragment_handle: shader_handle,
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

        Self { pipeline }
    }

    /// 1 pass を `encoder` へ積む。texture/uniform を読まないので bind group は
    /// 1つも set しない。
    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        dst_view: &wgpu::TextureView,
    ) {
        let render_pipelines = ctx.gpu_resources.render_pipelines.resources();
        let pipeline = render_pipelines
            .get(self.pipeline)
            .expect("wgsl fragment pipeline");

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("motolii-compositor-wgsl-fragment-pass"),
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
        pass.draw(0..3, 0..1);
    }
}
