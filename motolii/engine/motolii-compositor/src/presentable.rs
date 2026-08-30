//! 共有面へ書く口(裁定256)。
//!
//! Makepad / OS handle を知らない。渡された `wgpu::Texture` が Host 仕様
//! (`Rgba8UnormSrgb`・comp サイズ・`RENDER_ATTACHMENT`)かを見て、
//! `ViewBuilder::new_with_external_resolved` でその面へ直接書く。
//! ここから blit しない。

use re_renderer::renderer::{ColormappedTexture, RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{Rgba, ViewBuilderId};

use crate::{
    sequential_target_config, to_point3, to_vector3, CompSpec, Compositor, CompositorError,
    LayerWithPasses, ResolvedCamera,
};

impl Compositor {
    pub fn device(&self) -> &wgpu::Device {
        &self.ctx.device
    }

    pub fn render_context(&self) -> &re_renderer::RenderContext {
        &self.ctx
    }

    /// 共有面へ直接書く口(裁定256)。検査失敗では内部状態を変えない。
    ///
    /// **effect pass を適用してから描く**(2026-08-28 修理)——[`Self::render_with_effects`]/
    /// [`Self::render_to_texture`]と同じく `layers[i].passes` を
    /// `Self::effective_layer_textures` へ通し、その実効 texture(と、pass が出力を
    /// 拡張した分の padding)を rect へ使う。**この関数だけが `lwp.layer.texture` を
    /// そのまま描いていた**ため、共有面(Makepad の zero-copy Stage)は effect を
    /// 一切反映していなかった——export/CPU 読み戻し経路(上記2つ)は元から正しかった。
    pub fn render_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
    ) -> Result<(), CompositorError> {
        check_presentable_target(target, comp)?;

        // layer ごとに「合成へ渡す実効 texture」を決める(`render_effects.rs` の
        // `Self::effective_layer_textures` が3経路で共有する核)。`checked_out` は
        // この関数の合成が終わってから(下の poll の後)プールへ返す——
        // `render_with_effects`/`render_to_texture` と同じ返却タイミング。
        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        // 合成は `render_with_effects`(export)と**同じ** `accumulate_sequential` を通す。
        // ここで独自に rect を組むと分離可能 blend が落ちる(Stage だけ Normal に
        // 見えて書き出すと別の絵になる)。
        let inputs = crate::render_effects::sequential_inputs(
            layers,
            &effective_textures,
            &effective_paddings,
        );
        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        self.finalize_into(target, comp, camera, background, background_color)?;

        // scratch をプールへ返す——この合成(上の poll)が終わった後なので、
        // `render_with_effects`/`render_to_texture` と同じ返却タイミング
        // (`effective_layer_textures` の doc 参照)。
        for (width, height, format, scratch_texture) in checked_out {
            self.effect_scratch
                .release(width, height, format, scratch_texture);
        }
        Ok(())
    }
}

/// 製品 Stage の共有面が満たす画素形式。
pub const PRESENTABLE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// 共有面として受けられるか。失敗しても Document / compositor 内部状態は変えない。
pub fn check_presentable_target(
    target: &wgpu::Texture,
    comp: CompSpec,
) -> Result<(), CompositorError> {
    if target.format() != PRESENTABLE_FORMAT {
        return Err(CompositorError::PresentableFormat {
            got: format!("{:?}", target.format()),
        });
    }
    if target.width() != comp.width || target.height() != comp.height {
        return Err(CompositorError::PresentableSize {
            got: [target.width(), target.height()],
            expected: [comp.width, comp.height],
        });
    }
    if !target
        .usage()
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        return Err(CompositorError::PresentableUsage);
    }
    Ok(())
}
