use re_renderer::resource_managers::ImageDataDesc;
use re_renderer::RenderContext;

use crate::render::compositor::*;

pub(crate) fn wait_for_gpu(device: &wgpu::Device, stage: &str) -> Result<(), CompositorError> {
    let test_host = cfg!(test) || std::env::var("MOTOLII_GPU_TEST").as_deref() == Ok("1");
    let timeout = test_host.then_some(std::time::Duration::from_secs(30));
    if test_host { eprintln!("MOTOLII_GPU_WAIT stage={stage} submission=latest timeout=30s"); }
    let result = device.poll(wgpu::PollType::Wait { submission_index: None, timeout })
        .map(|_| ())
        .map_err(|error| CompositorError::Draw(format!("GPU wait stage={stage} submission=latest: {error}")));
    if test_host { if let Err(error) = &result { panic!("{error}"); } }
    result
}

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
        let ctx = RenderContext::new_from_device(device, queue, output_format, config_provider)
            .map_err(|e| CompositorError::Context(e.to_string()))?;

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
        eprintln!("MOTOLII_COMPOSITOR_INIT phase=selection_bounds");
        let selection_bounds = selection_bounds::SelectionBounds::new(&ctx.device, re_renderer::OutlineMaskProcessor::mask_sample_count(ctx.device_caps().tier) > 1);
        #[cfg(test)]
        eprintln!("MOTOLII_COMPOSITOR_INIT phase=ready cached_effects=0");
        Ok(Self {
            ctx,
            window: crate::render::compositor::Window { width: 0, height: 0, roi: [0.0; 4], projection_camera: None },
            measurement_enabled: false,
            motion: None,
            measurement: Default::default(),
            surface_work: Default::default(),
            reflection_resources: None,
            light_cookie: None,
            reflection_cache_enabled: true,
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
            reflection_entry: None,
            backdrop_resource: None,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            baked_effects: Default::default(),
            effect_programs,
            surface_programs: Default::default(),
            clock: None,
            feedback: Default::default(),
            feedback_revision: 0,
            blend_vism,
            selection_bounds,
            matte_vism,
            coverage_programs: Default::default(),
            catalog,
            sequential_submits: 0,
            last_submission: None,
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
        self.surface_programs.clear();
        // 焼いた絵は plugin_id と欄の値で引く。本文だけ変わった効果は同じ鍵で当たるので、世代が動いたら全部捨てる。
        self.baked_effects.clear(&mut self.effect_scratch);
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

    /// GPU の texture を CPU へ読み戻す(行の詰め物を外した生の byte)。Freeze の cache が書類の隣へ置く時に使う。
    pub(crate) fn read_texture_bytes(&mut self, texture: &wgpu::Texture) -> Result<Vec<u8>, CompositorError> {
        let bytes_per_pixel = texture.format().block_copy_size(None).ok_or_else(|| CompositorError::Effect(format!("{:?} は読み戻せない", texture.format())))?;
        let (width, height) = (texture.width(), texture.height());
        let row = width * bytes_per_pixel;
        let padded = row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let staging = self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-freeze-readback"), size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        });
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-freeze-readback") });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo { buffer: &staging, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(height) } },
            texture.size(),
        );
        self.pending.push(encoder.finish());
        self.flush_pending();
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
        wait_for_gpu(&self.ctx.device, "texture-readback")?;
        rx.recv_timeout(std::time::Duration::from_secs(30)).map_err(|e| CompositorError::Draw(format!("texture-readback map callback: {e}")))?.map_err(|e| CompositorError::Draw(e.to_string()))?;
        let data = slice.get_mapped_range();
        let mut out = Vec::with_capacity((row * height) as usize);
        for y in 0..height as usize {
            let start = y * padded as usize;
            out.extend_from_slice(&data[start..start + row as usize]);
        }
        drop(data);
        staging.unmap();
        Ok(out)
    }

    /// CPU の byte(乗算済み線形、Rgba16Float)を板として置ける texture に(Freeze の cache を disk から戻す時)。
    pub(crate) fn upload_rgba16f(&mut self, label: &str, bytes: Vec<u8>, width: u32, height: u32) -> Result<GpuTexture2D, CompositorError> {
        self.next_effect_key += 1;
        let key = self.next_effect_key;
        self.ctx.texture_manager_2d.get_or_try_create_with(key, &self.ctx, || Ok::<_, std::convert::Infallible>(ImageDataDesc {
            label: label.into(),
            data: bytes.into(),
            format: wgpu::TextureFormat::Rgba16Float.into(),
            width_height: [width, height],
            alpha_channel_usage: re_renderer::AlphaChannelUsage::AlphaChannelInUse,
        })).map_err(|e| CompositorError::Rectangles(e.to_string()))
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
