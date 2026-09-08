use re_renderer::resource_managers::ImageDataDesc;
use re_renderer::RenderContext;

use crate::render::compositor::*;

impl Compositor {
    pub fn headless() -> Result<Self, CompositorError> {
        let gpu = headless::HeadlessGpu::new()?;
        Self::with_device(
            gpu.device,
            gpu.queue,
            crate::render::compositor::PRESENTABLE_FORMAT,
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

        let catalog = super::catalog_snapshot();
        let effect_programs = catalog.definitions.iter().filter(|d| d.manifest.expose && d.manifest.stage == effects::IsfStage::Pass)
            .map(|d| (d.plugin_id().to_owned(), effects::EffectProgram::compile(&ctx, d))).collect();
        let builtin = |name: &str| -> Result<effects::EffectProgram, CompositorError> {
            let definition = catalog.definitions.iter().find(|d| d.source.name == name)
                .ok_or_else(|| CompositorError::Effect(format!("missing validated {name} program")))?;
            Ok(effects::EffectProgram::compile(&ctx, definition))
        };
        let blend_vism = builtin("blend")?;
        let matte_vism = builtin("matte")?;

        Ok(Self {
            ctx,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            effect_programs,
            mesh_programs: Default::default(),
            blend_vism,
            matte_vism,
            coverage_programs: Default::default(),
            catalog,
            sequential_submits: 0,
            pending: Vec::new(),
        })
    }

    pub(crate) fn refresh_catalog_programs(&mut self) {
        let next = super::catalog_snapshot();
        if next.generation == self.catalog.generation { return; }
        let changed: Vec<_> = next.definitions.iter().filter(|d| self.catalog.definitions.iter()
            .find(|old| old.source.name == d.source.name)
            .is_none_or(|old| old.source.source != d.source.source || old.vertex_text != d.vertex_text || old.fragment_text != d.fragment_text)).collect();
        #[cfg(load_shaders_from_disk)]
        {
            let frame = self.ctx.active_frame_idx();
            let paths = changed.iter().flat_map(|d| d.paths()).collect();
            let resolver = re_renderer::new_recommended_file_resolver();
            let pools = &mut self.ctx.gpu_resources;
            pools.shader_modules.begin_frame(&self.ctx.device, &resolver, frame, &paths);
            pools.render_pipelines.begin_frame(&self.ctx.device, frame, &pools.shader_modules, &pools.pipeline_layouts);
        }
        self.mesh_programs.clear();
        for definition in changed {
            if definition.manifest.stage != effects::IsfStage::Pass { continue; }
            let program = effects::EffectProgram::compile(&self.ctx, definition);
            match definition.source.name.as_str() {
                "blend" => self.blend_vism = program,
                "matte" => { self.matte_vism = program; self.coverage_programs.clear(); }
                _ if definition.manifest.expose => { self.effect_programs.insert(definition.plugin_id().to_owned(), program); }
                _ => {}
            }
        }
        self.catalog = next;
    }

    pub fn with_device_using_headless_defaults(
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<Self, CompositorError> {
        Self::with_device(
            device,
            queue,
            crate::render::compositor::PRESENTABLE_FORMAT,
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
