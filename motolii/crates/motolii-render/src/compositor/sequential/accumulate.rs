//! Clip groups: the upper layer's picture over its base's, where the base is (source-atop).

use super::*;

impl Compositor {
    pub(crate) fn source_atop(
        &mut self,
        base: &GpuTexture2D,
        upper: &GpuTexture2D,
        blend: BlendMode,
    ) -> Result<GpuTexture2D, CompositorError> {
        let compose = 9u32;
        let mode = match blend {
            BlendMode::Normal => compose,
            BlendMode::Add => return Err(CompositorError::UnsupportedBlendMode(blend)),
            other => (vello_blend_mode(other).expect("mix mode") & !0xff) | compose,
        };
        let [width, height] = base.width_height();
        if upper.width_height() != [width, height] {
            return Err(CompositorError::Effect("Clipping inputs must share composition dimensions".into()));
        }
        let base_resource = self.ctx.gpu_resources.textures.get_from_handle(base.handle())
            .map_err(|error| CompositorError::Effect(error.to_string()))?;
        let upper_resource = self.ctx.gpu_resources.textures.get_from_handle(upper.handle())
            .map_err(|error| CompositorError::Effect(error.to_string()))?;
        let base_view = base_resource.texture.create_view(&Default::default());
        let upper_view = upper_resource.texture.create_view(&Default::default());
        let output = self.create_blend_scratch_texture(width, height);
        let output_view = output.create_view(&Default::default());
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-clipping-source-atop"),
        });
        let Self { ctx, blend_vism, effect_scratch, .. } = self;
        blend_vism.get(ctx).record_over(
            ctx, &mut encoder, effect_scratch, &[&base_view, &upper_view], &output_view,
            &[("mode".to_owned(), mode as f32)], [width as f32, height as f32],
        );
        self.pending.push(encoder.finish());
        self.import_premultiplied(&output)
    }




}
