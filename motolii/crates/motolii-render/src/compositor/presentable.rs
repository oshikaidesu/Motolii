

use crate::render::compositor::{
    CompSpec, Compositor, CompositorError,
    LayerWithPasses, ResolvedCamera,
};

impl Compositor {
    pub fn device(&self) -> &wgpu::Device {
        &self.ctx.device
    }

    pub fn render_context(&self) -> &re_renderer::RenderContext {
        &self.ctx
    }

    pub fn render_into(
        &mut self,
        target: &wgpu::Texture,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
        background_color: [f32; 4],
    ) -> Result<(), CompositorError> {
        check_presentable_target(target, comp)?;

        let (effective_textures, effective_paddings, checked_out) =
            self.effective_layer_textures(layers)?;

        let inputs = crate::render::compositor::render_effects::sequential_inputs(
            layers,
            &effective_textures,
            &effective_paddings,
        );
        let background = self.accumulate_sequential(comp, camera, &inputs, background_color)?;
        self.finalize_into(target, comp, camera, background, background_color)?;

        for (width, height, format, scratch_texture) in checked_out {
            self.effect_scratch
                .release(width, height, format, scratch_texture);
        }
        Ok(())
    }
}

pub const PRESENTABLE_FORMAT: wgpu::TextureFormat = if cfg!(feature = "shared-bgra-output") { wgpu::TextureFormat::Bgra8UnormSrgb } else { wgpu::TextureFormat::Rgba8UnormSrgb };

pub fn check_presentable_target(
    target: &wgpu::Texture,
    comp: CompSpec,
) -> Result<(), CompositorError> {
    if target.format() != PRESENTABLE_FORMAT {
        return Err(CompositorError::PresentableFormat {
            got: format!("{:?}", target.format()),
        });
    }
    if target.width() != comp.width || target.height() != comp.height {
        return Err(CompositorError::PresentableSize {
            got: [target.width(), target.height()],
            expected: [comp.width, comp.height],
        });
    }
    if !target
        .usage()
        .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
    {
        return Err(CompositorError::PresentableUsage);
    }
    Ok(())
}
