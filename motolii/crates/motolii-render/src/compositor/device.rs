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
            re_renderer::RenderConfig::best_for_device_caps,
        )
    }

    pub fn with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        output_format: wgpu::TextureFormat,
        config_provider: impl FnOnce(&re_renderer::device_caps::DeviceCaps) -> re_renderer::RenderConfig,
    ) -> Result<Self, CompositorError> {
        // 棚の検証は、この device が本当に持っている物で行う(`Capabilities::all()` は嘘)。棚は device の後に組まれる。
        effects::block_program::note_device(&device);
        #[cfg(test)]
        eprintln!("MOTOLII_COMPOSITOR_INIT phase=render_context");
        let mut ctx = RenderContext::new_from_device(device, queue, output_format, config_provider)
            .map_err(|e| CompositorError::Context(e.to_string()))?;
        // Motolii is the embedder: it opens the first frame, as the viewer app does (`begin_frame`).
        ctx.begin_frame();

        #[cfg(test)]
        eprintln!("MOTOLII_COMPOSITOR_INIT phase=catalog");
        let catalog = super::catalog_snapshot();
        let effect_programs = Default::default();
        let builtin = |name: &str| -> Result<effects::LazyEffectProgram, CompositorError> {
            let definition = catalog.definitions.iter().find(|d| d.source.name == name)
                .ok_or_else(|| CompositorError::Effect(format!("missing validated {name} program (catalog: {})", catalog.errors.join("; "))))?;
            Ok(effects::LazyEffectProgram::new(definition.clone()))
        };
        let blend_vism = builtin("blend")?;
        let matte_vism = builtin("matte")?;

        #[cfg(test)]
        eprintln!("MOTOLII_COMPOSITOR_INIT phase=ready cached_effects=0");
        Ok(Self {
            ctx,
            measurement_enabled: false,
            world_environment: None,
            motion: None,
            measurement: Default::default(),
            surface_work: Default::default(),
            reflection_resources: None,
            light_cookie: None,
            gpu_instance_sharing_enabled: true,
            reflection_scene_probe: false,
            #[cfg(test)]
            reflection_probe_experiment: 3,
            #[cfg(test)]
            reflection_diagnostic_enabled: false,
            #[cfg(test)]
            reflection_diagnostic: None,
            #[cfg(test)]
            reflection_diagnostic_skip: None,
            #[cfg(test)]
            reflection_diagnostic_near: None,
            next_readback: 1,
            baked_effects: Default::default(),
            effect_programs,
            surface_programs: Default::default(),
            clock: None,
            feedback: Default::default(),
            feedback_revision: 0,
            feedback_seen: Vec::new(),
            blend_vism,
            matte_vism,
            coverage_programs: Default::default(),
            catalog,
            last_submission: None,
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
            let frame = self.ctx.active_frame.frame_index;
            let paths = changed.iter().flat_map(|d| d.paths()).collect();
            let resolver = re_renderer::new_recommended_file_resolver();
            let pools = &mut self.ctx.gpu_resources;
            pools.shader_modules.begin_frame(&self.ctx.device, &resolver, frame, &paths);
            pools.render_pipelines.begin_frame(&self.ctx.device, frame, &pools.shader_modules, &pools.pipeline_layouts);
        }
        self.surface_programs.clear();
        // 焼いた絵は plugin_id と欄の値で引く。本文だけ変わった効果は同じ鍵で当たるので、世代が動いたら全部捨てる。
        self.baked_effects.clear();
        for definition in changed {
            if !matches!(definition.manifest.stage, effects::IsfStage::Pass | effects::IsfStage::Warp) { continue; }
            match definition.source.name.as_str() {
                "blend" => self.blend_vism = effects::LazyEffectProgram::new((*definition).clone()),
                "matte" => { self.matte_vism = effects::LazyEffectProgram::new((*definition).clone()); self.coverage_programs.clear(); }
                // 棚に出さない warp も段の契約として組む(turbulent_warp は退役後も material.rs の試験が使う)。
                _ => { self.effect_programs.remove(definition.plugin_id()); }
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
            re_renderer::RenderConfig::best_for_device_caps,
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


    /// CPU の byte(乗算済み線形、Rgba16Float)を板として置ける texture に(Freeze の cache を disk から戻す時)。
    pub(crate) fn upload_rgba16f(&mut self, label: &str, bytes: Vec<u8>, width: u32, height: u32) -> Result<GpuTexture2D, CompositorError> {
        self.ctx.texture_manager_2d.create(&self.ctx, ImageDataDesc {
            label: label.into(),
            data: bytes.into(),
            format: wgpu::TextureFormat::Rgba16Float.into(),
            width_height: [width, height],
            alpha_channel_usage: re_renderer::AlphaChannelUsage::AlphaChannelInUse,
        }).map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    /// A drawn picture (premultiplied), as a texture a layer is drawn with.
    pub fn import_premultiplied(
        &mut self,
        texture: &re_renderer::GpuTexture,
    ) -> Result<GpuTexture2D, CompositorError> {
        GpuTexture2D::new(texture.clone(), re_renderer::AlphaChannelUsage::AlphaChannelInUse)
            .ok_or_else(|| CompositorError::Effect("a picture is a 2D texture".into()))
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
