// as: motolii/crates/motolii-render/src/compositor/scratch.rs
// A pool of its own: textures kept and handed out again instead of the host's texture pool.
pub(crate) struct ScratchTextures {
    free: Vec<re_renderer::GpuTexture>,
}

impl ScratchTextures {
    pub(crate) fn take(&mut self, ctx: &re_renderer::RenderContext, desc: &re_renderer::TextureDesc) -> re_renderer::GpuTexture {
        self.free.pop().unwrap_or_else(|| ctx.gpu_resources.textures.alloc(&ctx.device, desc))
    }
}
