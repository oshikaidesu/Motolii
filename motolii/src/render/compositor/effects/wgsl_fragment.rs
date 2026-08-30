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
        // マニフェストは任意。持たない .wgsl は入力ゼロの Vism として扱う。
        let manifest = parse_isf_source(wgsl_source)
            .map(|(manifest, _body)| manifest)
            .unwrap_or_default();

        #[cfg(load_shaders_from_disk)]
        let path = {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let abs_path = manifest_dir.join(format!("vism/{name}.wgsl"));
            let resolver = new_recommended_file_resolver();
            FileServer::get_mut(|fs| fs.watch(&resolver, &abs_path, false))
                .expect("{name}.wgsl は vism/ に在る")
        };
        #[cfg(not(load_shaders_from_disk))]
        let path = {
            let path = PathBuf::from(format!("motolii-vism/{name}.wgsl"));
            get_filesystem()
                .create_file(&path, wgsl_source.to_owned().into())
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
        dst_view: &wgpu::TextureView,
    ) {
        self.inner.record(ctx, encoder, &[], dst_view, &[], [0.0, 0.0]);
    }
}
