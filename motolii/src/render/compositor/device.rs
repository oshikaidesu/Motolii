use re_renderer::resource_managers::ImageDataDesc;
use re_renderer::RenderContext;

use crate::render::compositor::*;

impl Compositor {
    pub fn headless() -> Result<Self, CompositorError> {
        let gpu = headless::HeadlessGpu::new()?;
        Self::with_device(
            gpu.device,
            gpu.queue,
            re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
            |_caps| re_renderer::RenderConfig {
                msaa_mode: re_renderer::MsaaMode::Off,
            },
        )
    }

    pub fn with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        output_format: wgpu::TextureFormat,
        config_provider: impl FnOnce(&re_renderer::device_caps::DeviceCaps) -> re_renderer::RenderConfig,
    ) -> Result<Self, CompositorError> {
        let ctx = RenderContext::new_from_device(device, queue, output_format, config_provider)
            .map_err(|e| CompositorError::Context(e.to_string()))?;

        let mut effect_programs = std::collections::HashMap::new();
        for definition in effects::vism_definitions()
            .iter()
            .filter(|definition| definition.manifest.expose)
        {
            let program = effects::EffectProgram::compile(
                &ctx,
                definition.source,
                definition.output_format(),
            )
            .map_err(|error| CompositorError::Isf(error.to_string()))?;
            effect_programs.insert(definition.plugin_id().to_owned(), program);
        }
        let blend_vism = effects::WgslFragmentProgram::compile_with_prelude(
            &ctx,
            "blend",
            effects::VELLO_BLEND_PRELUDE,
            effects::BLEND_SOURCE,
            crate::render::compositor::BLEND_TARGET_FORMAT,
        );
        let matte_vism = effects::WgslFragmentProgram::compile_with_prelude(
            &ctx,
            "matte",
            effects::VELLO_BLEND_PRELUDE,
            effects::MATTE_SOURCE,
            crate::render::compositor::BLEND_TARGET_FORMAT,
        );

        Ok(Self {
            ctx,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            effect_programs,
            blend_vism,
            matte_vism,
            sequential_submits: 0,
            pending: Vec::new(),
        })
    }

    pub fn with_device_using_headless_defaults(
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<Self, CompositorError> {
        Self::with_device(
            device,
            queue,
            re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
            |_caps| re_renderer::RenderConfig {
                msaa_mode: re_renderer::MsaaMode::Off,
            },
        )
    }

    pub fn upload_rgba(
        &self,
        label: &str,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.ctx
            .texture_manager_2d
            .create(
                &self.ctx,
                ImageDataDesc {
                    label: label.into(),
                    data: rgba.to_vec().into(),
                    format: wgpu::TextureFormat::Rgba8Unorm.into(),
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::AlphaChannelInUse,
                },
            )
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    #[cfg(test)]
    pub fn begin_frame_for_test(&mut self) {
        self.ctx.begin_frame();
    }

    /// 鍵で覚えてもらう上げ口。**当たれば `make` は走らない**ので、
    /// 焼き直しも読み直しも起きない。覚えるのは texture_manager の仕事で、
    /// こちらは表を持たない。
    pub fn cached_rgba<E: std::fmt::Display>(
        &self,
        key: u64,
        label: &str,
        make: impl FnOnce() -> Result<(Vec<u8>, u32, u32), E>,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.ctx
            .texture_manager_2d
            .get_or_try_create_with(key, &self.ctx, || {
                let (rgba, width, height) = make()?;
                Ok::<_, E>(ImageDataDesc {
                    label: label.into(),
                    data: rgba.into(),
                    format: wgpu::TextureFormat::Rgba8Unorm.into(),
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::AlphaChannelInUse,
                })
            })
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    /// 焼いた絵を板として使えるようにする(上流の取り込み口)。
    pub fn import_premultiplied(
        &mut self,
        texture: &wgpu::Texture,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.next_effect_key += 1;
        let key = self.next_effect_key;
        self.ctx
            .texture_manager_2d
            .import_gpu_premultiplied(key, &self.ctx, texture)
            .map_err(|e| CompositorError::Effect(e.to_string()))
    }

    pub fn upload_yuv420p(
        &self,
        label: &str,
        data: &[u8],
        width: u32,
        height: u32,
        color: crate::doc::core::ColorSpace,
    ) -> Result<GpuTexture2D, CompositorError> {
        use re_renderer::resource_managers::{
            SourceImageDataFormat, YuvMatrixCoefficients, YuvPixelLayout, YuvRange,
        };

        let (coefficients, range) = match color {
            crate::doc::core::ColorSpace::Rec709Limited => {
                (YuvMatrixCoefficients::Bt709, YuvRange::Limited)
            }
            crate::doc::core::ColorSpace::Rec709Full => {
                (YuvMatrixCoefficients::Bt709, YuvRange::Full)
            }
            crate::doc::core::ColorSpace::Rec601Limited => {
                (YuvMatrixCoefficients::Bt601, YuvRange::Limited)
            }
            other => {
                return Err(CompositorError::Rectangles(format!(
                    "YUV420p に RGB 系の色空間が渡された: {other:?}"
                )))
            }
        };

        self.ctx
            .texture_manager_2d
            .create(
                &self.ctx,
                ImageDataDesc {
                    label: label.into(),
                    data: data.to_vec().into(),
                    format: SourceImageDataFormat::Yuv {
                        layout: YuvPixelLayout::Y_U_V420,
                        coefficients,
                        range,
                    },
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::Opaque,
                },
            )
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }
}
