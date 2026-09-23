use re_renderer::renderer::{RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{GpuTexture, Rgba, ViewBuilderId};

use crate::render::compositor::*;

impl Compositor {
    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    /// 最後に出した束の番号(まだ 1 度も出していなければ `None`)。
    pub fn last_submission(&self) -> Option<wgpu::SubmissionIndex> {
        self.last_submission.clone()
    }


    /// Ends the renderer frame that is open — everything recorded into it goes out in one
    /// submission — and begins the next. The tick's one frame boundary; a feedback replay's past
    /// frames are frames of their own.
    pub(crate) fn next_frame(&mut self) -> Option<wgpu::SubmissionIndex> {
        let submit_start = std::time::Instant::now();
        // The embedder's frame boundary, as re_viewer's app draws one: `before_submit` hands the frame
        // to the queue, `begin_frame` opens the next.
        let submitted = self.ctx.before_submit();
        if submitted.is_some() {
            self.last_submission = submitted.clone();
        }
        self.ctx.begin_frame();
        self.measurement.submit_us += submit_start.elapsed().as_micros() as u64;
        submitted
    }
}

mod accumulate;
mod finalize;
mod layer;
