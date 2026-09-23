// as: motolii/crates/motolii-render/src/compositor/glass_fast.rs
// A "faster glass" that binds its textures itself instead of through the host's bind group pool.
impl Compositor {
    pub(crate) fn fast_glass(&mut self, view: &wgpu::TextureView, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("fast glass"), layout, entries: &[] })
    }
}
