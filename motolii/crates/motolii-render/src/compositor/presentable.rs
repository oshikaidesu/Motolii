

use crate::render::compositor::{Compositor, CompositorError, Window};

impl Compositor {
    pub fn device(&self) -> &wgpu::Device {
        &self.ctx.device
    }

    pub fn render_context(&self) -> &re_renderer::RenderContext {
        &self.ctx
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
