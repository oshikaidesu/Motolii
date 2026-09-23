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

    /// One rendering of the composition (a view, an output frame) is one renderer frame: resource
    /// pools collect and staging memory is sized once per rendering, not once per submission.
    pub(crate) fn begin_render_frame(&mut self) {
        self.flush_pending();
        self.ctx.begin_frame();
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
        // staging buffer の回収は submit の**後**に。frame は終えない: 1 回の描画で何度も出すので、
        // 描画 frame の境目は `begin_render_frame` が一度だけ置く(re_renderer の資源回収と在庫見積りは
        // frame 単位で、submit ごとに回すと毎回作り直しになる)。
        self.ctx.after_submit_within_frame();
        // 1 コマで何度でも出るので足す。段の時計はこれを含んだままで、内訳として別に出す。
        self.measurement.submit_us += submit_start.elapsed().as_micros() as u64;
    }
}

mod accumulate;
mod finalize;
mod layer;
