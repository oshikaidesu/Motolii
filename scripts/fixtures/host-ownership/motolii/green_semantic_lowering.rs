// as: motolii/crates/motolii-render/src/compositor/view.rs
// Resource requirements and recordings the host submits: textures from its pool, bind groups
// from its pool, a cassette's pipeline from its pools, an encoder handed to queue_commands.
impl Compositor {
    fn glass_run(&mut self, window: Window, below: &re_renderer::GpuTexture, sources: &[Vec<GpuTexture2D>]) -> Result<(), CompositorError> {
        let canvas = self.ctx.gpu_resources.textures.alloc(&self.ctx.device, &canvas_desc(window));
        let module = self.ctx.gpu_resources.shader_modules.get_or_create(&self.ctx, &cassette_module());
        let bind = self.ctx.gpu_resources.bind_groups.alloc(&self.ctx.device, &self.ctx.gpu_resources, &bind_desc(&canvas));
        let mut config = sequential_target_config("run", window);
        config.backdrop = Some(self.import_premultiplied(below)?);
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("run") });
        let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, config, ViewBuilderId::new(1), &canvas.texture)?;
        builder.draw_into(&self.ctx, Rgba::TRANSPARENT, &mut encoder)?;
        self.ctx.queue_commands([encoder.finish()]);
        let _ = (module, bind, sources);
        Ok(())
    }
}
