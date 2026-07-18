//! ホスト所有のレンダパイプラインキャッシュ(F-10)。
//!
//! プラグインはパイプラインを所有せず、キーでここへ要求する。
//! 初回だけコンパイル、以降はヒット — 純関数契約と「毎フレーム生成禁止」を両立する。

use std::collections::HashMap;

use crate::GpuCtx;

/// キャッシュキー = 安定ID + WGSLソース(同一IDでソースが変わったら別エントリ)。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineCacheKey {
    pub id: &'static str,
    pub wgsl: &'static str,
}

/// フルスクリーン三角形 + uniform([f32; 16]) の定型パイプライン(0-input LayerSource)。
pub struct CachedFullscreenUniform16 {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// フレーム間で再利用。毎フレームは `queue.write_buffer` のみ。
    pub uniform_buffer: wgpu::Buffer,
}

/// フルスクリーン三角形 + texture + sampler + uniform(vec4) の定型パイプライン。
pub struct CachedTexSampleUniform4 {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    /// フレーム間で再利用。毎フレームは `queue.write_buffer` のみ。
    pub uniform_buffer: wgpu::Buffer,
}

#[derive(Default)]
pub struct PipelineCache {
    tex_sample_uniform4: HashMap<PipelineCacheKey, CachedTexSampleUniform4>,
    fullscreen_uniform16: HashMap<PipelineCacheKey, CachedFullscreenUniform16>,
    hits: u64,
    misses: u64,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// uniform [f32; 16](binding 0 のみ) レイアウトのフルスクリーンパイプラインを取得/生成。
    pub fn get_or_create_fullscreen_uniform16(
        &mut self,
        gpu: &GpuCtx,
        key: PipelineCacheKey,
    ) -> &CachedFullscreenUniform16 {
        if self.fullscreen_uniform16.contains_key(&key) {
            self.hits += 1;
            return &self.fullscreen_uniform16[&key];
        }
        self.misses += 1;
        let entry = create_fullscreen_uniform16(gpu, &key);
        self.fullscreen_uniform16.insert(key.clone(), entry);
        &self.fullscreen_uniform16[&key]
    }

    /// texture(0) + sampler(1) + uniform vec4(2) レイアウトのパイプラインを取得/生成。
    pub fn get_or_create_tex_sample_uniform4(
        &mut self,
        gpu: &GpuCtx,
        key: PipelineCacheKey,
    ) -> &CachedTexSampleUniform4 {
        if self.tex_sample_uniform4.contains_key(&key) {
            self.hits += 1;
            return &self.tex_sample_uniform4[&key];
        }
        self.misses += 1;
        let entry = create_tex_sample_uniform4(gpu, &key);
        self.tex_sample_uniform4.insert(key.clone(), entry);
        &self.tex_sample_uniform4[&key]
    }
}

fn create_fullscreen_uniform16(gpu: &GpuCtx, key: &PipelineCacheKey) -> CachedFullscreenUniform16 {
    let shader = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(key.id),
            source: wgpu::ShaderSource::Wgsl(key.wgsl.into()),
        });
    let bind_group_layout = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(key.id),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(key.id),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
    let pipeline = gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(key.id),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
    let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(key.id),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    CachedFullscreenUniform16 {
        pipeline,
        bind_group_layout,
        uniform_buffer,
    }
}

fn create_tex_sample_uniform4(gpu: &GpuCtx, key: &PipelineCacheKey) -> CachedTexSampleUniform4 {
    let shader = gpu
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(key.id),
            source: wgpu::ShaderSource::Wgsl(key.wgsl.into()),
        });
    let bind_group_layout = gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(key.id),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    let pipeline_layout = gpu
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(key.id),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
    let pipeline = gpu
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(key.id),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
    let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(key.id),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(key.id),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    CachedTexSampleUniform4 {
        pipeline,
        bind_group_layout,
        sampler,
        uniform_buffer,
    }
}

impl std::fmt::Debug for PipelineCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineCache")
            .field(
                "entries",
                &(self.tex_sample_uniform4.len() + self.fullscreen_uniform16.len()),
            )
            .field("hits", &self.hits)
            .field("misses", &self.misses)
            .finish()
    }
}
