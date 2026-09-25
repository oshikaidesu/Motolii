// as: motolii/crates/motolii-render/src/compositor/sequential.rs
impl Compositor {
    pub(crate) fn next_frame(&mut self) -> Option<wgpu::SubmissionIndex> {
        let submitted = self.ctx.before_submit();
        self.ctx.begin_frame();
        submitted
    }
}
