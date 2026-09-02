#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatteMode {
    Alpha,
    InvertedAlpha,
    Luma,
    InvertedLuma,
}

pub(crate) fn matte_mode_index(mode: MatteMode) -> u32 {
    match mode {
        MatteMode::Alpha => 0,
        MatteMode::InvertedAlpha => 1,
        MatteMode::Luma => 2,
        MatteMode::InvertedLuma => 3,
    }
}

impl super::Compositor {
    /// 層ローカルのalpha coverageを、Track Matteと同じ借り物Vismで掛ける。
    /// MaskはEffectより前なので、ここでは配置・blend・Effectを一切知らない。
    pub(crate) fn apply_local_alpha_mask(
        &mut self,
        layer: &super::GpuTexture2D,
        mask: &super::GpuTexture2D,
    ) -> Result<super::GpuTexture2D, super::CompositorError> {
        let [width, height] = layer.width_height();
        if mask.width_height() != [width, height] {
            return Err(super::CompositorError::Effect(format!(
                "mask coverage size {:?} does not match layer texture {width}x{height}",
                mask.width_height()
            )));
        }

        let layer_resource = self
            .ctx
            .gpu_resources
            .textures
            .get_from_handle(layer.handle())
            .map_err(|error| super::CompositorError::Effect(error.to_string()))?;
        let mask_resource = self
            .ctx
            .gpu_resources
            .textures
            .get_from_handle(mask.handle())
            .map_err(|error| super::CompositorError::Effect(error.to_string()))?;
        let layer_view = layer_resource.texture.create_view(&Default::default());
        let mask_view = mask_resource.texture.create_view(&Default::default());
        let out_texture = self.create_blend_scratch_texture(width, height);
        let out_view = out_texture.create_view(&Default::default());
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-compositor-local-mask-pass-encoder"),
            });

        let Self {
            ctx,
            matte_vism,
            effect_scratch,
            ..
        } = self;
        matte_vism.record_over(
            ctx,
            &mut encoder,
            effect_scratch,
            &[&layer_view, &mask_view],
            &out_view,
            &[("mode".to_owned(), matte_mode_index(MatteMode::Alpha) as f32)],
            [width as f32, height as f32],
        );
        self.pending.push(encoder.finish());
        self.import_premultiplied(&out_texture)
    }
}
