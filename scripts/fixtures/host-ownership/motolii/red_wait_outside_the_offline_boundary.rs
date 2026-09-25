// as: motolii/crates/motolii-render/src/compositor/headless.rs
// The offline boundary is one function; another function in the same file does not inherit it.
impl super::Compositor {
    pub(crate) fn wait_offline(&self) -> Result<(), super::CompositorError> {
        self.ctx.device.poll(wgpu::PollType::wait_indefinitely()).map(|_| ()).map_err(|e| super::CompositorError::Draw(e.to_string()))
    }
    pub(crate) fn wait_for_the_stage(&self) {
        let _ = self.ctx.device.poll(wgpu::PollType::wait_indefinitely());
    }
}
