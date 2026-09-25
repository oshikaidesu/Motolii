//! The Look pass. Once per view, after the stack is composed (scene-linear HDR): one
//! bright-pass pyramid gives bloom, halation, a star and an anamorphic streak, and the composite splits
//! colour where it means something (lateral aberration, print plates, dispersed glare). Nothing per
//! layer or per instance: the passes are fixed, whatever the work holds.
//! Which strengths apply is the composition's `Look` (Studio unless chosen); nothing else selects it.

use wgpu::util::DeviceExt as _;

use super::*;

const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// Bright-pass levels: 1/2 .. 1/64 of the view.
const LEVELS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LookParams {
    pub threshold: f32,
    pub knee: f32,
    pub bloom: f32,
    pub halation: f32,
    pub star: f32,
    pub streak: f32,
    /// Lateral chromatic aberration at the frame's corner, px at 1080 lines.
    pub lateral: f32,
    /// Print plates out of register, px at 1080 lines.
    pub plate: f32,
    pub star_angle: f32,
    pub spectral: f32,
    /// Star blade length, eighth-size texels (taps).
    pub star_length: f32,
    /// How far the glare's red and blue part, px.
    pub dispersion: f32,
}

impl LookParams {
    /// Default Studio: subtle — only real highlights glow, a thin star on the brightest.
    pub(crate) const STUDIO: Self = Self { threshold: 1.5, knee: 0.5, bloom: 0.05, halation: 0.02, star: 0.10, streak: 0.0, lateral: 0.7, plate: 0.0, star_angle: 0.26, spectral: 0.5, star_length: 14.0, dispersion: 1.0 };
    /// Poster / Optical: medium — type splits at its edges, plates slip, highlights halate and streak.
    pub(crate) const POSTER: Self = Self { threshold: 1.5, knee: 0.5, bloom: 0.07, halation: 0.06, star: 0.16, streak: 0.10, lateral: 1.4, plate: 0.8, star_angle: 0.26, spectral: 0.8, star_length: 18.0, dispersion: 2.0 };
    /// Jewel / Crystal: strong stars with rainbow blades, restrained bloom.
    pub(crate) const JEWEL: Self = Self { threshold: 1.5, knee: 0.4, bloom: 0.05, halation: 0.02, star: 0.20, streak: 0.04, lateral: 0.9, plate: 0.0, star_angle: 0.26, spectral: 1.0, star_length: 18.0, dispersion: 2.5 };

    pub(crate) const fn of(look: crate::doc::store::Look) -> Self {
        match look {
            crate::doc::store::Look::Studio => Self::STUDIO,
            crate::doc::store::Look::Poster => Self::POSTER,
            crate::doc::store::Look::Jewel => Self::JEWEL,
        }
    }

    fn uniform(&self, source_texel: [f32; 2], target: [f32; 2], scale: f32) -> [f32; 16] {
        [
            self.threshold, self.knee, self.bloom, self.halation,
            self.star, self.streak, self.lateral * scale, self.plate * scale,
            source_texel[0], source_texel[1], target[0], target[1],
            self.star_angle, self.spectral, self.star_length, self.dispersion * scale,
        ]
    }
}

pub(crate) struct LookPipelines {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    prefilter: wgpu::RenderPipeline,
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    star: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
}

impl LookPipelines {
    fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("motolii-look"), source: wgpu::ShaderSource::Wgsl(include_str!("look.wgsl").into()) });
        let texture = |binding| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("motolii-look"), entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            texture(1), texture(2), texture(3),
            wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
        ] });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("motolii-look"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = |fs: &str, format: wgpu::TextureFormat| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(fs), layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_full"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(fs), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None, cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-look"), address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default()
        });
        Self {
            prefilter: pipeline("fs_prefilter", HDR), down: pipeline("fs_down", HDR), up: pipeline("fs_up", HDR),
            star: pipeline("fs_star", HDR), composite: pipeline("fs_composite", BLEND_TARGET_FORMAT),
            layout, sampler,
        }
    }
}

impl Compositor {
    /// The view's picture with its Look; the picture itself when the Look is off.
    pub(super) fn look_pass(&mut self, look: crate::doc::store::Look, window: Window, scene: re_renderer::GpuTexture, encoder: &mut wgpu::CommandEncoder) -> re_renderer::GpuTexture {
        let params = LookParams::of(look);
        if self.look.is_none() {
            let scope = self.ctx.device.push_error_scope(wgpu::ErrorFilter::Validation);
            let pipelines = LookPipelines::new(&self.ctx.device);
            if let Some(error) = pollster::block_on(scope.pop()) {
                eprintln!("the Look pass does not compile: {error}");
                return scene;
            }
            self.look = Some(pipelines);
        }
        let (width, height) = (window.width.max(1), window.height.max(1));
        // Lengths are the look's at 1080 lines; a smaller view scales them.
        let scale = height as f32 / 1080.0;
        let level_size = |k: usize| [(width >> (k + 1)).max(1), (height >> (k + 1)).max(1)];
        let texture = |[w, h]: [u32; 2], format| effects::vism::pass_texture(&self.ctx, w, h, format);
        let down: Vec<re_renderer::GpuTexture> = (0..LEVELS).map(|k| texture(level_size(k), HDR)).collect();
        let up: Vec<re_renderer::GpuTexture> = (0..LEVELS - 1).map(|k| texture(level_size(k), HDR)).collect();
        let star = texture(level_size(1), HDR);
        let out = texture([width, height], BLEND_TARGET_FORMAT);
        let look = self.look.as_ref().expect("look pipelines");
        let device = &self.ctx.device;
        let texel = |t: &re_renderer::GpuTexture| [1.0 / t.texture.width() as f32, 1.0 / t.texture.height() as f32];
        let size = |t: &re_renderer::GpuTexture| [t.texture.width() as f32, t.texture.height() as f32];
        let mut pass = |pipeline: &wgpu::RenderPipeline, uniform: [f32; 16], target: &re_renderer::GpuTexture, sources: [&re_renderer::GpuTexture; 3]| {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("motolii-look"), contents: &uniform.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>(), usage: wgpu::BufferUsages::UNIFORM });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("motolii-look"), layout: &look.layout, entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&sources[0].default_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&sources[1].default_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&sources[2].default_view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&look.sampler) },
            ] });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-look"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &target.default_view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store } })],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            rp.set_pipeline(pipeline);
            rp.set_bind_group(0, &group, &[]);
            rp.draw(0..3, 0..1);
        };
        // Bright pass and pyramid.
        pass(&look.prefilter, params.uniform(texel(&scene), size(&down[0]), scale), &down[0], [&scene, &scene, &scene]);
        for k in 1..LEVELS {
            pass(&look.down, params.uniform(texel(&down[k - 1]), size(&down[k]), scale), &down[k], [&down[k - 1], &down[k - 1], &down[k - 1]]);
        }
        // Star and streak read the pyramid before it is summed back up.
        pass(&look.star, params.uniform(texel(&down[2]), size(&star), scale), &star, [&down[2], &down[3], &down[4]]);
        for k in (0..LEVELS - 1).rev() {
            let lower = if k == LEVELS - 2 { &down[LEVELS - 1] } else { &up[k + 1] };
            pass(&look.up, params.uniform(texel(lower), size(&up[k]), scale), &up[k], [&down[k], lower, lower]);
        }
        pass(&look.composite, params.uniform(texel(&scene), [width as f32, height as f32], scale), &out, [&scene, &up[0], &star]);
        out
    }
}
