//! Scratch Lighting Pack (2026-09-25, BB theater): not a contract, not a default. It takes a 3D run's
//! pictures as Renderables — a world quad, its picture's coverage and colour, and what the layer
//! already says about light (Cast Shadow: blocks the key light; Glow: gives light) — and realizes
//! surface, lighting and image formation on its own targets, then hands the run's canvas back.
//! Switched on by `MOTOLII_LIGHT_PACK=on|off` (off: the pictures as authored, no light) with
//! `MOTOLII_IMAGE_FORMATION=film|clamp`. The shading is WGSL read at start
//! (`MOTOLII_LIGHT_PACK_WGSL`, else the bundled file), so the look is tuned without a Rust build.

use super::*;

const ATLAS: u32 = 2048;
const SLOT: u32 = 128;
const SLOTS_PER_ROW: u32 = ATLAS / SLOT;
const ATLAS_LEVELS: u32 = 8; // 128 -> 1: a slot's last level is its picture's mean
const MAX_QUADS: usize = (SLOTS_PER_ROW * SLOTS_PER_ROW) as usize;
const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// The light field: probes on a grid over the world's pictures, each the light arriving there
/// (L1 spherical harmonics, one texture per colour channel), gathered anew every frame.
const PROBES: [u32; 3] = [22, 11, 22];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct PackMode {
    pub lit: bool,
    pub film: bool,
}

impl PackMode {
    pub(crate) fn from_env() -> Option<Self> {
        let lit = match std::env::var("MOTOLII_LIGHT_PACK").ok()?.as_str() {
            "on" => true,
            "off" => false,
            _ => return None,
        };
        let film = std::env::var("MOTOLII_IMAGE_FORMATION").map_or(true, |f| f != "clamp");
        Some(Self { lit, film })
    }
}

#[derive(Clone, Copy, Default)]
struct Globals {
    clip_from_world: [[f32; 4]; 4],
    eye: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    params: [f32; 4],
    window: [f32; 4],
    background: [f32; 4],
    grid_min: [f32; 4],
    grid_cell: [f32; 4],
    world_from_clip: [[f32; 4]; 4],
}

#[derive(Clone, Copy, Default)]
struct GpuQuad {
    /// xyz corner, w slot
    corner: [f32; 4],
    u: [f32; 4],
    v: [f32; 4],
    /// xyz unit normal (u × v), w flags: 1 blocks the key light, 2 gives light
    n: [f32; 4],
    /// x decode (0 premultiplied linear-by-hardware, 1 straight sRGB, 2 linear premultiplied), y opacity, z emission strength
    info: [f32; 4],
}

/// Little-endian f32s as the GPU reads them.
fn bytes(floats: impl IntoIterator<Item = f32>) -> Vec<u8> {
    floats.into_iter().flat_map(f32::to_le_bytes).collect()
}

impl Globals {
    fn floats(&self) -> Vec<f32> {
        let mut out: Vec<f32> = self.clip_from_world.iter().flatten().copied().collect();
        for v in [self.eye, self.sun_dir, self.sun_color, self.params, self.window, self.background, self.grid_min, self.grid_cell] {
            out.extend(v);
        }
        out.extend(self.world_from_clip.iter().flatten());
        out
    }
}

impl GpuQuad {
    fn floats(&self) -> [[f32; 4]; 5] {
        [self.corner, self.u, self.v, self.n, self.info]
    }
}

pub(crate) struct LightPack {
    atlas: wgpu::Texture,
    atlas_levels: Vec<wgpu::TextureView>,
    globals: wgpu::Buffer,
    quads: wgpu::Buffer,
    sampler: wgpu::Sampler,
    scene_group: wgpu::BindGroup,
    blit_group: wgpu::BindGroup,
    picture_layout: wgpu::BindGroupLayout,
    level_layout: wgpu::BindGroupLayout,
    blit: wgpu::RenderPipeline,
    downsample: wgpu::RenderPipeline,
    depth: wgpu::RenderPipeline,
    color: wgpu::RenderPipeline,
    formation_layout: wgpu::BindGroupLayout,
    formation: wgpu::RenderPipeline,
    /// Two light fields: the first bounce, then the second read through the first.
    probe_views: [Vec<wgpu::TextureView>; 2],
    probe_groups: [wgpu::BindGroup; 2],
    probe_first: wgpu::RenderPipeline,
    probe: wgpu::RenderPipeline,
    /// The front surface's normal and picture, then the low-frequency light read from it at half size.
    screen_layout: wgpu::BindGroupLayout,
    screen: wgpu::RenderPipeline,
    near_light_layout: wgpu::BindGroupLayout,
    /// A planar mesh's picture, kept while the same mesh is drawn at the same size (a shape that
    /// does not change is not drawn again). Entries not used by a frame are let go.
    planar_pictures: std::collections::HashMap<(usize, u32, u32), re_renderer::GpuTexture>,
    planar_used: std::collections::HashSet<(usize, u32, u32)>,
}

fn wgsl_source() -> String {
    std::env::var("MOTOLII_LIGHT_PACK_WGSL").ok().and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_else(|| include_str!("light_pack.wgsl").to_owned())
}

impl LightPack {
    fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("light-pack"), source: wgpu::ShaderSource::Wgsl(wgsl_source().into()) });
        let atlas = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("light-pack-atlas"),
            size: wgpu::Extent3d { width: ATLAS, height: ATLAS, depth_or_array_layers: 1 },
            mip_level_count: ATLAS_LEVELS, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
        });
        let atlas_levels = (0..ATLAS_LEVELS).map(|level| atlas.create_view(&wgpu::TextureViewDescriptor { base_mip_level: level, mip_level_count: Some(1), ..Default::default() })).collect();
        let globals = device.create_buffer(&wgpu::BufferDescriptor { label: Some("light-pack-globals"), size: 64 * 4, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let quads = device.create_buffer(&wgpu::BufferDescriptor { label: Some("light-pack-quads"), size: (20 * 4 * MAX_QUADS) as u64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("light-pack-linear"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, mipmap_filter: wgpu::MipmapFilterMode::Linear, ..Default::default()
        });
        let uniform = |binding| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let storage = |binding| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None };
        let texture = |binding| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None };
        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-scene"), entries: &[uniform(0), storage(1), texture(2), sampler_entry(3)] });
        let blit_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-blit-scene"), entries: &[uniform(0), storage(1), sampler_entry(3)] });
        let picture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-picture"), entries: &[texture(0)] });
        let level_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-level"), entries: &[texture(0), sampler_entry(1)] });
        let probe_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-probes"), entries: &[texture(0), texture(1), texture(2)] });
        let field = || -> Vec<wgpu::TextureView> { (0..3).map(|_| device.create_texture(&wgpu::TextureDescriptor {
            label: Some("light-pack-probes"),
            size: wgpu::Extent3d { width: PROBES[0] * PROBES[1], height: PROBES[2], depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: HDR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
        }).create_view(&wgpu::TextureViewDescriptor::default())).collect() };
        let probe_views = [field(), field()];
        let probe_groups = [0, 1].map(|i| device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-probes"), layout: &probe_layout, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&probe_views[i][0]) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&probe_views[i][1]) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&probe_views[i][2]) },
        ] }));
        let formation_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-formation"), entries: &[uniform(0), texture(1), sampler_entry(2)] });
        let depth_texture = wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None };
        let screen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-screen"), entries: &[texture(0), depth_texture] });
        let near_light_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("light-pack-near"), entries: &[texture(0), sampler_entry(1)] });
        let atlas_view = atlas.create_view(&wgpu::TextureViewDescriptor::default());
        let scene_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-scene"), layout: &scene_layout, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: quads.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&atlas_view) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) },
        ] });
        let blit_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-blit-scene"), layout: &blit_layout, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: globals.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: quads.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&sampler) },
        ] });
        let layout = |groups: &[&wgpu::BindGroupLayout]| {
            let groups: Vec<Option<&wgpu::BindGroupLayout>> = groups.iter().map(|g| Some(*g)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("light-pack"), bind_group_layouts: &groups, immediate_size: 0 })
        };
        let pipeline = |label: &str, layout: &wgpu::PipelineLayout, vs: &str, fs: &str, format: wgpu::TextureFormat, blend: Option<wgpu::BlendState>, depth: Option<wgpu::DepthStencilState>, color: bool| {
            let targets = [Some(wgpu::ColorTargetState { format, blend, write_mask: wgpu::ColorWrites::ALL })];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label), layout: Some(layout),
                vertex: wgpu::VertexState { module: &module, entry_point: Some(vs), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: depth,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(fs), compilation_options: Default::default(), targets: if color { &targets } else { &[] } }),
                multiview_mask: None, cache: None,
            })
        };
        let premultiplied = Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING);
        let blit = pipeline("light-pack-blit", &layout(&[&blit_layout, &picture_layout]), "vs_slot", "fs_blit", HDR_FORMAT, None, None, true);
        let downsample = pipeline("light-pack-downsample", &layout(&[&level_layout]), "vs_full", "fs_downsample", HDR_FORMAT, None, None, true);
        let depth_state = |write: bool, compare| Some(wgpu::DepthStencilState { format: DEPTH_FORMAT, depth_write_enabled: Some(write), depth_compare: Some(compare), stencil: Default::default(), bias: Default::default() });
        let scene = layout(&[&scene_layout, &picture_layout]);
        let depth = pipeline("light-pack-depth", &scene, "vs_quad", "fs_depth", HDR_FORMAT, None, depth_state(true, wgpu::CompareFunction::Greater), true);
        let color = pipeline("light-pack-color", &layout(&[&scene_layout, &picture_layout, &probe_layout, &near_light_layout]), "vs_quad", "fs_color", HDR_FORMAT, premultiplied, depth_state(false, wgpu::CompareFunction::GreaterEqual), true);
        let screen = pipeline("light-pack-screen", &layout(&[&scene_layout, &screen_layout]), "vs_full", "fs_screen", HDR_FORMAT, None, None, true);
        let probe_pipeline = |entry: &str, groups: &[&wgpu::BindGroupLayout]| {
            let targets = [0, 1, 2].map(|_| Some(wgpu::ColorTargetState { format: HDR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL }));
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry), layout: Some(&layout(groups)),
                vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_full"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(entry), compilation_options: Default::default(), targets: &targets }),
                multiview_mask: None, cache: None,
            })
        };
        let probe_first = probe_pipeline("fs_probe_first", &[&scene_layout]);
        let probe = probe_pipeline("fs_probe", &[&scene_layout, &picture_layout, &probe_layout]);
        let formation = pipeline("light-pack-formation", &layout(&[&formation_layout]), "vs_full", "fs_formation", BLEND_TARGET_FORMAT, None, None, true);
        Self { atlas, atlas_levels, globals, quads, sampler, scene_group, blit_group, picture_layout, level_layout, blit, downsample, depth, color, formation_layout, formation, probe_views, probe_groups, probe_first, probe, screen_layout, screen, near_light_layout, planar_pictures: Default::default(), planar_used: Default::default() }
    }
}

struct PackQuad {
    gpu: GpuQuad,
    texture: re_renderer::GpuTexture,
    in_run: bool,
    distance: f32,
}

impl Compositor {
    /// A 3D run drawn by the pack: every 3D picture of `inputs` is the world it lights, those in
    /// `run` are drawn. Returns the run's canvas (premultiplied, display-encoded as the stack is).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn light_pack_run(
        &mut self,
        mode: PackMode,
        comp: CompSpec,
        window: Window,
        projection: crate::doc::core::CameraProjection,
        inputs: &[SequentialInput<'_>],
        run: std::ops::Range<usize>,
        environment: Option<&GpuEnvironmentData>,
        background: Option<[f32; 4]>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<re_renderer::GpuTexture, CompositorError> {
        if self.light_pack.is_none() {
            self.light_pack = Some(LightPack::new(&self.ctx.device));
        }
        let mut quads: Vec<PackQuad> = Vec::new();
        for (index, input) in inputs.iter().enumerate() {
            // A spill (a Glow's halo, laid by its own blend) is how the effect shows, not a surface in the room.
            if input.projection == crate::doc::store::LayerProjection::TwoD || input.blend_mode != BlendMode::Normal || quads.len() == MAX_QUADS {
                continue;
            }
            let (texture, decode, local_min) = match input.content {
                SequentialContent::Rect(t) | SequentialContent::LinearRect(t) => {
                    let decode = if matches!(input.content, SequentialContent::LinearRect(_)) { 2.0 } else if t.format().is_srgb() { 0.0 } else { 1.0 };
                    let Ok(texture) = self.ctx.gpu_resources.textures.get_from_handle(t.handle()) else { continue };
                    (texture, decode, input.local_min)
                }
                // A shape or a text as outlines (a planar mesh): its picture, drawn as a 2D layer draws it.
                SequentialContent::Model(model) if model.planar_size.is_some() => {
                    let (_, u, v) = projected_placement_corners(comp, input.projection_camera, input.projection, input.placement, glam::Vec2::ZERO, input.local_size);
                    (self.planar_picture(model, &input.shading, u.length(), v.length(), encoder)?, 0.0, glam::Vec2::ZERO)
                }
                _ => continue,
            };
            let (corner, u, v) = projected_placement_corners(comp, input.projection_camera, input.projection, input.placement, local_min, input.local_size);
            let n = u.cross(v).normalize_or_zero();
            if !corner.is_finite() || n == glam::Vec3::ZERO {
                continue;
            }
            let flags = if input.shadow > 0.0 { 1.0 } else { 0.0 } + if input.emission > 0.0 { 2.0 } else { 0.0 };
            let slot = quads.len() as f32;
            let centre = corner + (u + v) * 0.5;
            quads.push(PackQuad {
                gpu: GpuQuad {
                    corner: [corner.x, corner.y, corner.z, slot],
                    u: [u.x, u.y, u.z, u.length()],
                    v: [v.x, v.y, v.z, v.length()],
                    n: [n.x, n.y, n.z, flags],
                    info: [decode, input.opacity, input.emission, 0.0],
                },
                texture,
                in_run: run.contains(&index),
                distance: centre.distance(projection.eye),
            });
        }
        {
            let pack = self.light_pack.as_mut().expect("made above");
            let used = std::mem::take(&mut pack.planar_used);
            pack.planar_pictures.retain(|key, _| used.contains(key));
        }
        if std::env::var("MOTOLII_LIGHT_PACK_DEBUG").is_ok() {
            for (i, input) in inputs.iter().enumerate() {
                eprintln!("PACK input {i} proj {:?} shadow {} emission {} opacity {}", input.projection, input.shadow, input.emission, input.opacity);
            }
            for q in &quads { eprintln!("PACK quad corner {:?} n {:?} info {:?} in_run {}", q.gpu.corner, q.gpu.n, q.gpu.info, q.in_run); }
        }
        let sun = environment.map_or(super::environment::SunSpec::fixed_lights(), |e| e.sun);
        // The light field's grid: over every picture of the world.
        let (mut lo, mut hi) = (glam::Vec3::splat(f32::INFINITY), glam::Vec3::splat(f32::NEG_INFINITY));
        for q in &quads {
            let c = glam::Vec3::from_slice(&q.gpu.corner[..3]);
            let (u, v) = (glam::Vec3::from_slice(&q.gpu.u[..3]), glam::Vec3::from_slice(&q.gpu.v[..3]));
            for p in [c, c + u, c + v, c + u + v] { lo = lo.min(p); hi = hi.max(p); }
        }
        if !lo.is_finite() { lo = glam::Vec3::ZERO; hi = glam::Vec3::ONE; }
        let cell = (hi - lo).max(glam::Vec3::splat(1.0)) / (glam::UVec3::from_array(PROBES).as_vec3() - 1.0);
        let clip_from_world = window_from_comp(comp, window) * projection.projection_matrix() * projection.view_matrix();
        // the infinite reverse-z projection inverts except at depth 0 (the horizon), which no surface has
        let world_from_clip = clip_from_world.inverse();
        let globals = Globals {
            clip_from_world: clip_from_world.to_cols_array_2d(),
            eye: projection.eye.extend(1.0).to_array(),
            sun_dir: sun.direction.normalize_or_zero().extend(sun.weight).to_array(),
            sun_color: sun.color.extend(1.0).to_array(),
            params: [quads.len() as f32, if mode.lit { 1.0 } else { 0.0 }, if mode.film { 1.0 } else { 0.0 }, 0.0],
            window: [window.width as f32, window.height as f32, if background.is_some() { 1.0 } else { 0.0 }, 0.0],
            background: background.unwrap_or_default(),
            grid_min: lo.extend(0.0).to_array(),
            grid_cell: cell.extend(0.0).to_array(),
            world_from_clip: world_from_clip.to_cols_array_2d(),
        };
        let pack = self.light_pack.as_ref().expect("made above");
        let device = &self.ctx.device;
        self.ctx.queue.write_buffer(&pack.globals, 0, &bytes(globals.floats()));
        if !quads.is_empty() {
            self.ctx.queue.write_buffer(&pack.quads, 0, &bytes(quads.iter().flat_map(|q| q.gpu.floats()).flatten()));
        }
        let pictures: Vec<wgpu::BindGroup> = quads.iter().map(|q| device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light-pack-picture"), layout: &pack.picture_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&q.texture.default_view) }],
        })).collect();

        // The atlas: every picture at a slot, then its levels down to the mean.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light-pack-atlas"), color_attachments: &[color_attachment(&pack.atlas_levels[0], wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT))],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_pipeline(&pack.blit);
            pass.set_bind_group(0, &pack.blit_group, &[]);
            for (i, picture) in pictures.iter().enumerate() {
                let (x, y) = ((i as u32 % SLOTS_PER_ROW) * SLOT, (i as u32 / SLOTS_PER_ROW) * SLOT);
                pass.set_viewport(x as f32, y as f32, SLOT as f32, SLOT as f32, 0.0, 1.0);
                pass.set_bind_group(1, picture, &[]);
                pass.draw(0..3, i as u32..i as u32 + 1);
            }
        }
        for level in 1..ATLAS_LEVELS as usize {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-level"), layout: &pack.level_layout, entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&pack.atlas_levels[level - 1]) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&pack.sampler) },
            ] });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light-pack-atlas-level"), color_attachments: &[color_attachment(&pack.atlas_levels[level], wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT))],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_pipeline(&pack.downsample);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        }

        // The light field, gathered from the frame's pictures alone.
        for bounce in 0..if mode.lit { 2 } else { 0 } {
            let targets: Vec<Option<wgpu::RenderPassColorAttachment<'_>>> = pack.probe_views[bounce].iter().map(|v| color_attachment(v, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT))).collect();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light-pack-probes"), color_attachments: &targets,
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_bind_group(0, &pack.scene_group, &[]);
            if bounce == 0 {
                pass.set_pipeline(&pack.probe_first);
            } else {
                // group 1 is unused by the probes; any picture fills the slot
                pass.set_pipeline(&pack.probe);
                if let Some(any) = pictures.first() { pass.set_bind_group(1, any, &[]); }
                pass.set_bind_group(2, &pack.probe_groups[0], &[]);
            }
            pass.draw(0..3, 0..1);
        }

        // Surface + light: the run's pictures far to near, into a scene-linear target.
        let target = |format, label: &str| self.ctx.gpu_resources.textures.alloc(device, &re_renderer::TextureDesc {
            label: label.to_owned().into(),
            size: wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let hdr = target(HDR_FORMAT, "light-pack-hdr");
        let depth = target(DEPTH_FORMAT, "light-pack-depth");
        let gbuf = target(HDR_FORMAT, "light-pack-front");
        let half = Window { width: window.width.div_ceil(2), height: window.height.div_ceil(2), roi: window.roi };
        let near = self.ctx.gpu_resources.textures.alloc(device, &re_renderer::TextureDesc {
            label: "light-pack-near".into(),
            size: wgpu::Extent3d { width: half.width, height: half.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let screen_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-screen"), layout: &pack.screen_layout, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&gbuf.default_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&depth.default_view) },
        ] });
        let near_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-near"), layout: &pack.near_light_layout, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&near.default_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&pack.sampler) },
        ] });
        let mut order: Vec<usize> = (0..quads.len()).filter(|&i| quads[i].in_run).collect();
        order.sort_by(|&a, &b| quads[b].distance.total_cmp(&quads[a].distance));
        for (pipeline, color) in [(&pack.depth, false), (&pack.color, true)] {
            if color && mode.lit {
                // the low-frequency light of the front surfaces, at half size
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("light-pack-near"), color_attachments: &[color_attachment(&near.default_view, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT))],
                    depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
                });
                pass.set_pipeline(&pack.screen);
                pass.set_bind_group(0, &pack.scene_group, &[]);
                pass.set_bind_group(1, &screen_group, &[]);
                pass.draw(0..3, 0..1);
            }
            let attachment = [color_attachment(if color { &hdr.default_view } else { &gbuf.default_view }, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT))];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(if color { "light-pack-color" } else { "light-pack-depth" }),
                color_attachments: &attachment,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.default_view,
                    depth_ops: Some(wgpu::Operations { load: if color { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(0.0) }, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &pack.scene_group, &[]);
            if color {
                pass.set_bind_group(2, &pack.probe_groups[1], &[]);
                pass.set_bind_group(3, &near_group, &[]);
            }
            for &i in &order {
                pass.set_bind_group(1, &pictures[i], &[]);
                pass.draw(0..6, i as u32..i as u32 + 1);
            }
        }

        // Image formation onto the run's canvas.
        let canvas = self.view_canvas_for(window, BLEND_TARGET_FORMAT);
        let pack = self.light_pack.as_ref().expect("made above");
        let group = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("light-pack-formation"), layout: &pack.formation_layout, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: pack.globals.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&hdr.default_view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&pack.sampler) },
        ] });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light-pack-formation"), color_attachments: &[color_attachment(&canvas.default_view, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT))],
                depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            pass.set_pipeline(&pack.formation);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        }
        drop((hdr, depth, gbuf, near));
        Ok(canvas)
    }
}

fn color_attachment(view: &wgpu::TextureView, load: wgpu::LoadOp<wgpu::Color>) -> Option<wgpu::RenderPassColorAttachment<'_>> {
    Some(wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } })
}

impl Compositor {
    /// A planar mesh's picture: drawn as a 2D layer draws it, at up to 1024 px on its long side.
    fn planar_picture(&mut self, model: &GpuModelData, shading: &effects::surface_program::SurfaceShading, width: f32, height: f32, encoder: &mut wgpu::CommandEncoder) -> Result<re_renderer::GpuTexture, CompositorError> {
        let natural = model.planar_size.expect("planar");
        let k = (1024.0 / width.max(height).max(1.0)).min(1.0);
        let (w, h) = (((width * k).round() as u32).clamp(8, 2048), ((height * k).round() as u32).clamp(8, 2048));
        let key = (model as *const GpuModelData as usize, w, h);
        let pack = self.light_pack.as_mut().expect("made before pictures");
        pack.planar_used.insert(key);
        if let Some(kept) = pack.planar_pictures.get(&key) {
            return Ok(kept.clone());
        }
        let texture = self.ctx.gpu_resources.textures.alloc(&self.ctx.device, &re_renderer::TextureDesc {
            label: "light-pack-shape".into(),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format: BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        let window = Window { width: w, height: h, roi: [0.0, 0.0, w as f32, h as f32] };
        let mut builder = re_renderer::view_builder::ViewBuilder::new_with_external_resolved(&self.ctx, screen_target_config("light-pack-shape", window), re_renderer::ViewBuilderId::new(self.next_readback), &texture.texture)
            .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        let scale = glam::Affine3A::from_scale(glam::vec3(w as f32 / natural[0].max(1.0), h as f32 / natural[1].max(1.0), 1.0));
        let program = shading.program.clone().or_else(|| self.standard_surface_program());
        let instances: Vec<re_renderer::renderer::GpuMeshInstance> = model.instances.iter().cloned().map(|mut instance| {
            instance.world_from_mesh = scale * instance.world_from_mesh;
            instance.additive_tint = re_renderer::Color32::from_rgba_unmultiplied(0, 0, 0, 255);
            instance.program = program.clone();
            instance.params = shading.params;
            instance
        }).collect();
        let layers = vec![re_renderer::renderer::DrawOrder::default(); instances.len()];
        let mesh = re_renderer::renderer::MeshDrawData::new_ordered(&self.ctx, &instances, re_renderer::ClipPlane::NONE, &layers).map_err(|e| CompositorError::Draw(e.to_string()))?;
        builder.queue_draw(&self.ctx, mesh).map_err(|e| CompositorError::Draw(e.to_string()))?;
        builder.draw_into(&self.ctx, re_renderer::Rgba::TRANSPARENT, encoder).map_err(|e| CompositorError::Draw(e.to_string()))?;
        self.light_pack.as_mut().expect("made").planar_pictures.insert(key, texture.clone());
        Ok(texture)
    }
}

/// The composition's clip space to the window's (a Stage view shows a region of the output).
fn window_from_comp(comp: CompSpec, window: Window) -> glam::Mat4 {
    let [rx, ry, rw, rh] = window.roi;
    let (w, h) = (comp.width as f32, comp.height as f32);
    let (sx, sy) = (w / rw.max(1e-3), h / rh.max(1e-3));
    let ox = (w - 2.0 * rx) / rw.max(1e-3) - 1.0;
    let oy = 1.0 - (h - 2.0 * ry) / rh.max(1e-3);
    glam::Mat4::from_cols(glam::vec4(sx, 0.0, 0.0, 0.0), glam::vec4(0.0, sy, 0.0, 0.0), glam::vec4(0.0, 0.0, 1.0, 0.0), glam::vec4(ox, oy, 0.0, 1.0))
}
