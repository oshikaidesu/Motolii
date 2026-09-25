// as: motolii/crates/motolii-render/src/engine/render.rs
// The drawing route reads a picture back in the same frame and waits for it.
impl Engine {
    pub fn picture_now(&mut self, id: re_renderer::GpuReadbackIdentifier) {
        self.compositor.next_frame();
        self.compositor.ctx.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let _ = crate::render::compositor::readback::take_texture(&self.compositor.ctx, id);
    }
}
