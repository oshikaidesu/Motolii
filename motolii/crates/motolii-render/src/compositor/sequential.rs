use re_renderer::renderer::{RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{GpuTexture, Rgba, ScreenshotProcessor, ViewBuilderId};

use crate::render::compositor::*;

pub(crate) struct BackdropResource {
    dimensions: [u32; 2],
    texture: wgpu::Texture,
    imported: GpuTexture2D,
}

impl Compositor {
    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    pub fn sequential_submits(&self) -> u64 {
        self.sequential_submits
    }

    pub(crate) fn flush_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        self.ctx.before_submit();
        let batch: Vec<wgpu::CommandBuffer> = self.pending.drain(..).collect();
        self.ctx.queue.submit(batch);
        self.sequential_submits += 1;
        // staging buffer の回収は submit の**後**に一度だけ。
        self.ctx.begin_frame();
    }
}

mod accumulate;
mod finalize;
mod layer;
