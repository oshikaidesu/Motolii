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

        let glow_pipelines = effects::GlowPipelines::new(&ctx.device);
        let isf_bloom = effects::IsfProgram::compile(
            &ctx,
            effects::BLOOM_SOURCE,
            effects::ISF_TARGET_FORMAT,
        )
        .map_err(|e| CompositorError::Isf(e.to_string()))?;
        let wgsl_gradient = effects::WgslFragmentProgram::compile(
            &ctx,
            "gradient",
            effects::GRADIENT_SOURCE,
            effects::GRADIENT_TARGET_FORMAT,
        );
        let wgsl_tri_led = effects::WgslFragmentProgram::compile(
            &ctx,
            "tri_led",
            effects::TRI_LED_SOURCE,
            effects::TRI_LED_TARGET_FORMAT,
        );
        let blend_pipelines = blend::SeparableBlendPipelines::new(&ctx);
        let matte_pipelines = matte::MattePipelines::new(&ctx);

        Ok(Self {
            ctx,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            glow_pipelines,
            isf_bloom,
            wgsl_gradient,
            wgsl_tri_led,
            blend_pipelines,
            matte_pipelines,
            sequential_submits: 0,
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
            crate::doc::core::ColorSpace::Rec709Full => (YuvMatrixCoefficients::Bt709, YuvRange::Full),
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
