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

    /// 最後に出した束の番号(まだ 1 度も出していなければ `None`)。
    pub fn last_submission(&self) -> Option<wgpu::SubmissionIndex> {
        self.last_submission.clone()
    }

    pub(crate) fn flush_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let submit_start = std::time::Instant::now();
        self.ctx.before_submit();
        let batch: Vec<wgpu::CommandBuffer> = self.pending.drain(..).collect();
        self.last_submission = Some(self.ctx.queue.submit(batch));
        self.sequential_submits += 1;
        // staging buffer の回収は submit の**後**に一度だけ。
        self.ctx.begin_frame();
        // 1 コマで何度でも出るので足す。段の時計はこれを含んだままで、内訳として別に出す。
        self.measurement.submit_us += submit_start.elapsed().as_micros() as u64;
    }
}

mod accumulate;
mod finalize;
mod layer;
