// as: motolii/crates/motolii-render/src/compositor/matte.rs
// Records and submits on its own, beside the host's frame.
impl Compositor {
    fn matte_now(&mut self) {
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.record_matte(&mut encoder);
        self.ctx.queue.submit([encoder.finish()]);
    }
}
