// as: motolii/crates/motolii-render/src/compositor/headless.rs
impl super::Compositor {
    pub(crate) fn wait_offline(&self) -> Result<(), super::CompositorError> {
        self.ctx.device.poll(wgpu::PollType::wait_indefinitely()).map(|_| ()).map_err(|e| super::CompositorError::Draw(e.to_string()))
    }
}
