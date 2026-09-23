//! The textures a composition's pictures are drawn into.

use super::*;

impl Compositor {
    /// A picture's texture (the blend target format) from re_renderer's pool: passes draw into it,
    /// later passes and views read it, and it returns to the pool when nothing holds it.
    pub(crate) fn picture_texture(&self, width: u32, height: u32) -> re_renderer::GpuTexture {
        effects::pass_texture(&self.ctx, width, height, crate::render::compositor::BLEND_TARGET_FORMAT)
    }
}
