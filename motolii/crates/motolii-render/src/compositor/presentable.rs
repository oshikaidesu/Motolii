

use crate::render::compositor::{
    CompSpec, Compositor, CompositorError,
    LayerWithPasses, ResolvedCamera, Window,
};

impl Compositor {
    pub fn device(&self) -> &wgpu::Device {
        &self.ctx.device
    }

    pub fn render_context(&self) -> &re_renderer::RenderContext {
        &self.ctx
    }

    pub fn render_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
    ) -> Result<(), CompositorError> {
        self.render_into_window(target, comp, camera, layers, background_color, Window::output(comp))
    }

    /// 出力寸法以外の窓へ描く(Stage: タブの寸法へ、関心域で pan & scan)。
    pub fn render_into_window(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
        window: Window,
    ) -> Result<(), CompositorError> {
        check_presentable_target(target, window)?;
        self.window = window;

        let (effective_textures, effective_paddings, effective_spills, checked_out) =
            self.effective_layer_textures(layers)?;

        let inputs = crate::render::compositor::render_effects::sequential_inputs(
            layers,
            &effective_textures,
            &effective_paddings,
            &effective_spills,
        );
        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        let outline = self.outline_view(comp, camera, &inputs)?;
        self.finalize_into(target, comp, camera, background, background_color, outline)?;

        for (width, height, format, scratch_texture) in checked_out {
            self.effect_scratch
                .release(width, height, format, scratch_texture);
        }
        Ok(())
    }

    /// 直前の `render_into` で選ばれていた層の画面上の広がり(番号 → `[x0, y0, x1, y1]` 画素)。
    /// GPU から届く前なら `None`。
    pub fn selection_screen_bounds(&mut self) -> Option<Vec<(u8, [f32; 4])>> {
        self.selection_bounds.as_mut()?.take(&self.ctx.device)
    }
}

/// 窓へ渡す形式 = re_renderer の出力形式。composite shader が自前で `srgb_from_linear` を
/// 掛ける(composite.wgsl)ので、**sRGB 形式にしてはいけない** — hardware がもう一度 encode して
/// 窓だけ白く浮く(export は `Rgba8Unorm` 読み戻しで正しかった。2026-09-07)。
pub const PRESENTABLE_FORMAT: wgpu::TextureFormat = if cfg!(feature = "shared-bgra-output") { wgpu::TextureFormat::Bgra8Unorm } else { re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT };

pub fn check_presentable_target(
    target: &wgpu::Texture,
    window: Window,
) -> Result<(), CompositorError> {
    if target.format() != PRESENTABLE_FORMAT {
        return Err(CompositorError::PresentableFormat {
            got: format!("{:?}", target.format()),
        });
    }
    if target.width() != window.width || target.height() != window.height {
        return Err(CompositorError::PresentableSize {
            got: [target.width(), target.height()],
            expected: window.size(),
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
