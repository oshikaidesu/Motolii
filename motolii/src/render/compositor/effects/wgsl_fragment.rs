//! WGSL の入口 — `vism/*.wgsl` をそのまま上流のプールへ載せる。
//!
//! ISF(GLSL)との違いは**言語だけ**。マニフェスト(先頭の `/*{ ... }*/`)の読み手も
//! 束縛の作り手も同じで、プログラム本体は [`VismProgram`]。

use std::path::PathBuf;

#[cfg(load_shaders_from_disk)]
use re_renderer::{new_recommended_file_resolver, FileServer};
#[cfg(not(load_shaders_from_disk))]
use re_renderer::{get_filesystem, FileSystem as _};
use re_renderer::RenderContext;

use super::isf::parse_isf_source;
use super::vism::{ShaderStageSource, VismProgram};

pub(crate) const GRADIENT_SOURCE: &str = include_str!("../../../../vism/gradient.wgsl");

pub(crate) const GRADIENT_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) const TRI_LED_SOURCE: &str = include_str!("../../../../vism/tri_led.wgsl");

pub(crate) const TRI_LED_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) const BLEND_SOURCE: &str = include_str!("../../../../vism/blend.wgsl");

pub(crate) const MATTE_SOURCE: &str = include_str!("../../../../vism/matte.wgsl");

pub(crate) const GLOW_SOURCE: &str = include_str!("../../../../vism/glow.wgsl");

/// 借りた式(`reference/vello-blend.wgsl`、vello_shaders 0.10.0 原文)。
/// W3C Compositing の 16 mix + 13 compose がここに在る。Motolii は式を持たない。
pub(crate) const VELLO_BLEND_PRELUDE: &str = include_str!("../../../../reference/vello-blend.wgsl");

pub(crate) struct WgslFragmentProgram {
    inner: VismProgram,
}

impl WgslFragmentProgram {
    pub(crate) fn compile(
        ctx: &RenderContext,
        name: &str,
        #[cfg_attr(load_shaders_from_disk, allow(unused_variables))] wgsl_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        Self::compile_inner(ctx, name, None, wgsl_source, output_format)
    }

    /// 借りた式を前置きしてから組む。前置きが在る間はホットリロードを切る
    /// (ディスクの1枚と中身が違うので、監視しても嘘になる)。
    pub(crate) fn compile_with_prelude(
        ctx: &RenderContext,
        name: &str,
        prelude: &str,
        wgsl_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        Self::compile_inner(ctx, name, Some(prelude), wgsl_source, output_format)
    }

    fn compile_inner(
        ctx: &RenderContext,
        name: &str,
        prelude: Option<&str>,
        #[cfg_attr(load_shaders_from_disk, allow(unused_variables))] wgsl_source: &str,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        // マニフェストは任意。持たない .wgsl は入力ゼロの Vism として扱う。
        let manifest = parse_isf_source(wgsl_source)
            .map(|(manifest, _body)| manifest)
            .unwrap_or_default();

        #[cfg(load_shaders_from_disk)]
        let path = if let Some(prelude) = prelude {
            // 前置きを繋いだ物は vism/ の1枚と中身が違う。実体を temp へ置いて読ませる。
            super::vism::stage_source_on_disk(name, &format!("{prelude}\n{wgsl_source}"))
        } else {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let abs_path = manifest_dir.join(format!("vism/{name}.wgsl"));
            let resolver = new_recommended_file_resolver();
            FileServer::get_mut(|fs| fs.watch(&resolver, &abs_path, false))
                .expect("{name}.wgsl は vism/ に在る")
        };
        #[cfg(not(load_shaders_from_disk))]
        let path = {
            let text = match prelude {
                Some(prelude) => format!("{prelude}\n{wgsl_source}"),
                None => wgsl_source.to_owned(),
            };
            let path = PathBuf::from(format!("motolii-vism/{name}.wgsl"));
            get_filesystem()
                .create_file(&path, text.into())
                .expect("vism の .wgsl が utf8 である");
            path
        };

        Self {
            inner: VismProgram::new(
                ctx,
                &format!("motolii-vism-{name}"),
                manifest,
                ShaderStageSource {
                    path: path.clone(),
                    entry_point: "vs_main".to_owned(),
                },
                ShaderStageSource {
                    path,
                    entry_point: "fs_main".to_owned(),
                },
                output_format,
            ),
        }
    }

    pub(crate) fn record(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        scratch: &mut super::EffectScratch,
        dst_view: &wgpu::TextureView,
        render_size: [f32; 2],
    ) {
        self.inner
            .record(ctx, encoder, scratch, &[], dst_view, &[], render_size);
    }

    /// 宣言した image 入力へ順に texture を渡して描く。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_over(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        scratch: &mut super::EffectScratch,
        sources: &[&wgpu::TextureView],
        dst_view: &wgpu::TextureView,
        params: &[(String, f32)],
        render_size: [f32; 2],
    ) {
        self.inner.record(
            ctx,
            encoder,
            scratch,
            sources,
            dst_view,
            params,
            render_size,
        );
    }
}
