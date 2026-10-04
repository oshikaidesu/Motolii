//! One camera. Solids and the video card share it. Type is drawn into that picture.
//! Glass is the usual screen-space refraction: the scene is already in `current`,
//! one sample, Schlick, Beer–Lambert. Not a ray march.

use std::collections::BTreeMap;

use glam::Mat4;
use wgpu_3dgs_viewer::core::IterGaussian;

use super::shelf::{self, Kind, Pipe, Shelf};
use super::thor::Thor;

struct Slot {
    texture: wgpu::Texture,
    key: Option<u32>,
}

pub(super) struct At<'a> {
    pub time: u32,
    pub y: &'a wgpu::Texture,
    pub uv: &'a wgpu::Texture,
}

pub(super) struct World {
    current: wgpu::Texture,
    msaa: wgpu::Texture,
    depth: wgpu::Texture,
    dry: Vec<Slot>,
    shown: wgpu::Texture,
    last_t: Option<u32>,
    shelf: Shelf,
    pipes: BTreeMap<Kind, Pipe>,
    mesh_buf: wgpu::Buffer,
    card_buf: wgpu::Buffer,
    mark_buf: wgpu::Buffer,
    echo_buf: wgpu::Buffer,
    uniform: wgpu::Buffer,
    mark_uniform: wgpu::Buffer,
    echo_uniforms: [wgpu::Buffer; 2],
    glass_uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    splats: wgpu_3dgs_viewer::Viewer,
    width: f32,
    height: f32,
}

impl World {
    pub(super) fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Result<Self, String> {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let current = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("current"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let msaa = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa"),
            size,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size,
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let echo_size = wgpu::Extent3d {
            width: (width / 2).max(1),
            height: (height / 2).max(1),
            depth_or_array_layers: 1,
        };
        let dry = (0..13)
            .map(|index| Slot {
                texture: color_target(device, echo_size, &format!("dry-{index}")),
                key: None,
            })
            .collect();
        let shown = color_target(device, size, "shown");
        let shelf = Shelf::open();
        let mut pipes = BTreeMap::new();
        for kind in Kind::ALL {
            pipes.insert(kind, first_pipe(device, &shelf, kind)?);
        }
        let cloud = cloud(width, height);
        let (translation, scale) = place(&cloud);
        let mut splats = wgpu_3dgs_viewer::Viewer::new(device, wgpu::TextureFormat::Bgra8Unorm, &cloud)
            .map_err(|error| error.to_string())?;
        splats.update_model_transform(
            queue,
            translation,
            glam32::Quat::IDENTITY,
            glam32::Vec3::splat(scale),
        );
        let degree = if cloud.len() > 100 { 3 } else { 0 };
        splats.update_gaussian_transform(
            queue,
            1.0,
            wgpu_3dgs_viewer::core::GaussianDisplayMode::Splat,
            wgpu_3dgs_viewer::core::GaussianShDegree::new(degree).ok_or("sh degree")?,
            false,
            wgpu_3dgs_viewer::core::GaussianMaxStdDev::new(2.0).ok_or("std dev")?,
        );
        Ok(Self {
            current,
            msaa,
            depth,
            dry,
            shown,
            last_t: None,
            shelf,
            pipes,
            mesh_buf: upload(device, &cubes(), "cubes"),
            card_buf: upload(device, &video_card(), "card"),
            mark_buf: upload(device, &text_card(), "text"),
            echo_buf: upload(device, &echo_card(), "echo"),
            uniform: uniform_buf(device, 128, "vp"),
            mark_uniform: uniform_buf(device, 128, "text-vp"),
            echo_uniforms: [
                uniform_buf(device, 256, "echo-0"),
                uniform_buf(device, 256, "echo-1"),
            ],
            glass_uniform: uniform_buf(device, 16, "glass"),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("space"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                ..Default::default()
            }),
            splats,
            width: width as f32,
            height: height as f32,
        })
    }

    pub(super) fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        present: &wgpu::TextureView,
        thor: &Thor,
        t: u32,
        frames: &[At<'_>],
    ) -> Result<(), String> {
        self.reload(device);
        if let Some(prev) = self.last_t {
            if t != prev.wrapping_add(1) {
                eprintln!("A1: echo scrub {prev} -> {t}");
            }
        }
        thor.lettering()?;
        let needed = frames_needed(t);
        let mut misses = 0u32;
        for time in &needed {
            let source = frames.iter().find(|frame| frame.time == *time).ok_or("missing frame")?;
            if self.ensure_dry(device, queue, thor, *time, source.y, source.uv, t)? {
                misses += 1;
            }
        }
        let eye = eye_at(t);
        let aspect = self.width / self.height;
        let plates = self.plates(eye, aspect, t);
        let source = frames.iter().find(|frame| frame.time == t).ok_or("missing frame")?;
        self.render_scene(device, queue, thor, t, source.y, source.uv, &plates, present)?;
        if t == 30 || misses > 1 {
            eprintln!("A1: frames {needed:?} misses {misses}");
        }
        self.last_t = Some(t);
        Ok(())
    }

    /// Files saved since the last frame become pipelines now, before any pass of this frame
    /// reads them. The GPU is idle here: the loop drained it before publishing the last frame.
    fn reload(&mut self, device: &wgpu::Device) {
        for kind in self.shelf.changed() {
            let (source, origin) = self.shelf.read(kind);
            match shelf::build(device, kind, &source) {
                Ok(pipe) => {
                    self.pipes.insert(kind, pipe);
                    eprintln!("A1: shelf reloaded {} from {origin}", kind.name());
                }
                Err(error) => eprintln!(
                    "\x1b[31mERROR shelf {} ({origin}): {error}\nlast valid pipeline retained\x1b[0m",
                    kind.name()
                ),
            }
        }
    }

    fn pipe(&self, kind: Kind) -> &Pipe {
        &self.pipes[&kind]
    }

    fn ensure_dry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        thor: &Thor,
        time: u32,
        y: &wgpu::Texture,
        uv: &wgpu::Texture,
        display: u32,
    ) -> Result<bool, String> {
        if self.dry.iter().any(|slot| slot.key == Some(time)) {
            return Ok(false);
        }
        let keep = frames_needed(display);
        let keys: Vec<Option<u32>> = self.dry.iter().map(|slot| slot.key).collect();
        let index = choose_slot(&keys, time, &keep);
        let dest = self.dry[index].texture.clone();
        let view = dest.create_view(&Default::default());
        self.render_scene(device, queue, thor, time, y, uv, &[], &view)?;
        self.dry[index].key = Some(time);
        if time != display {
            eprintln!("A1: echo refill {time} for display {display}");
        }
        Ok(true)
    }

    fn plates(&self, eye: glam::Vec3, aspect: f32, t: u32) -> Vec<(wgpu::Texture, Mat4, f32)> {
        let mut plates = Vec::new();
        if t >= 12 {
            if let Some(texture) = self.stored(t - 12) {
                plates.push((texture, echo_plate(eye, 2.15, -1.55, 7.4, 0.34, aspect), 1.0));
            }
        }
        if t >= 6 {
            if let Some(texture) = self.stored(t - 6) {
                plates.push((texture, echo_plate(eye, -2.35, -1.15, 6.8, 0.4, aspect), 1.0));
            }
        }
        plates
    }

    fn stored(&self, time: u32) -> Option<wgpu::Texture> {
        self.dry.iter().find(|slot| slot.key == Some(time)).map(|slot| slot.texture.clone())
    }

    fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        thor: &Thor,
        frame: u32,
        y: &wgpu::Texture,
        uv: &wgpu::Texture,
        plates: &[(wgpu::Texture, Mat4, f32)],
        dest: &wgpu::TextureView,
    ) -> Result<(), String> {
        let placed = super::comp::frame(frame);
        let eye = eye_at(frame);
        let look = Mat4::look_at_rh(eye, glam::Vec3::new(0.0, 0.05, 0.0), glam::Vec3::Y);
        let perspective = Mat4::perspective_rh(0.72, self.width / self.height, 0.08, 40.0);
        let vp = wgpu_clip() * perspective * look;
        write_mats(queue, &self.uniform, vp, Mat4::IDENTITY);
        write_mats(queue, &self.mark_uniform, vp, Mat4::from_translation(glam::Vec3::from_array(placed.text)));
        let mut plate_binds = Vec::new();
        for (index, (texture, model, fade)) in plates.iter().enumerate() {
            write_echo(queue, &self.echo_uniforms[index], vp, *model, *fade);
            plate_binds.push(self.echo_bind(device, texture, &self.echo_uniforms[index]));
        }
        let color = self.current.create_view(&Default::default());
        let msaa = self.msaa.create_view(&Default::default());
        let depth = self.depth.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("space") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("world"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa,
                    resolve_target: Some(&color),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.07, g: 0.065, b: 0.06, a: 1.0 }),
                        store: wgpu::StoreOp::Discard,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            for bind in &plate_binds {
                pass.set_pipeline(&self.pipe(Kind::Echo).pipeline);
                pass.set_bind_group(0, bind, &[]);
                pass.set_vertex_buffer(0, self.echo_buf.slice(..));
                pass.draw(0..6, 0..1);
            }
            pass.set_pipeline(&self.pipe(Kind::Mesh).pipeline);
            pass.set_bind_group(0, &bind_uniform(device, &self.pipe(Kind::Mesh).layout, &self.uniform), &[]);
            pass.set_vertex_buffer(0, self.mesh_buf.slice(..));
            pass.draw(0..108, 0..1);
            pass.set_pipeline(&self.pipe(Kind::Card).pipeline);
            pass.set_bind_group(0, &self.card_bind(device, y, uv), &[]);
            pass.set_vertex_buffer(0, self.card_buf.slice(..));
            pass.draw(0..6, 0..1);
            pass.set_pipeline(&self.pipe(Kind::Mark).pipeline);
            pass.set_bind_group(0, &self.mark_bind(device, thor.texture()), &[]);
            pass.set_vertex_buffer(0, self.mark_buf.slice(..));
            pass.draw(0..6, 0..1);
        }
        queue.submit([encoder.finish()]);
        draw_splats(self, device, queue, &color, look, perspective);
        let glass_uv = project(vp, placed.glass);
        let glass = [glass_uv[0], glass_uv[1], 0.22, self.width / self.height];
        queue.write_buffer(&self.glass_uniform, 0, &bytes_f32(&glass));
        let shown = self.shown.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("glass") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("glass"),
                color_attachments: &[Some(attach(&shown, wgpu::LoadOp::Clear(wgpu::Color::BLACK)))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipe(Kind::Glass).pipeline);
            pass.set_bind_group(0, &self.glass_bind(device), &[]);
            pass.draw(0..3, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(attach(dest, wgpu::LoadOp::Clear(wgpu::Color::BLACK)))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipe(Kind::Blit).pipeline);
            pass.set_bind_group(0, &self.blit_bind(device), &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([encoder.finish()]);
        Ok(())
    }

    fn echo_bind(&self, device: &wgpu::Device, layer: &wgpu::Texture, uniform: &wgpu::Buffer) -> wgpu::BindGroup {
        let view = layer.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("echo"),
            layout: &self.pipe(Kind::Echo).layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    fn blit_bind(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let view = self.shown.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blit"),
            layout: &self.pipe(Kind::Blit).layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    fn mark_bind(&self, device: &wgpu::Device, layer: &wgpu::Texture) -> wgpu::BindGroup {
        let view = layer.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text"),
            layout: &self.pipe(Kind::Mark).layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.mark_uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    fn card_bind(&self, device: &wgpu::Device, y: &wgpu::Texture, uv: &wgpu::Texture) -> wgpu::BindGroup {
        let y_view = y.create_view(&Default::default());
        let uv_view = uv.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("card"),
            layout: &self.pipe(Kind::Card).layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&y_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&uv_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    fn glass_bind(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let view = self.current.create_view(&Default::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glass"),
            layout: &self.pipe(Kind::Glass).layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: self.glass_uniform.as_entire_binding() },
            ],
        })
    }
}

/// At start a broken file is reported in red and its slot comes from the embedded copy,
/// so the picture always opens.
fn first_pipe(device: &wgpu::Device, shelf: &Shelf, kind: Kind) -> Result<Pipe, String> {
    let (source, origin) = shelf.read(kind);
    match shelf::build(device, kind, &source) {
        Ok(pipe) => Ok(pipe),
        Err(error) => {
            eprintln!("\x1b[31mERROR shelf {} ({origin}): {error}\nembedded copy used\x1b[0m", kind.name());
            shelf::build(device, kind, kind.embedded()).map_err(|error| format!("embedded {}: {error}", kind.name()))
        }
    }
}

fn draw_splats(
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    color: &wgpu::TextureView,
    look: Mat4,
    perspective: Mat4,
) {
    let camera = SharedCam {
        view: glam32::Mat4::from_cols_array(&look.to_cols_array()),
        proj: glam32::Mat4::from_cols_array(&perspective.to_cols_array()),
    };
    world.splats.update_camera(
        queue,
        &camera,
        glam32::UVec2::new(world.width as u32, world.height as u32),
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("splats") });
    let count = world.splats.gaussians_buffer.len() as u32;
    world.splats.preprocessor.preprocess(&mut encoder, count);
    world.splats.radix_sorter.sort(&mut encoder, &world.splats.radix_sort_indirect_args_buffer);
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("splats"),
            color_attachments: &[Some(attach(color, wgpu::LoadOp::Load))],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        world.splats.renderer.render_with_pass(&mut pass, &world.splats.indirect_args_buffer);
    }
    queue.submit([encoder.finish()]);
}

struct SharedCam {
    view: glam32::Mat4,
    proj: glam32::Mat4,
}

impl wgpu_3dgs_viewer::CameraTrait for SharedCam {
    fn view(&self) -> glam32::Mat4 {
        self.view
    }

    fn projection(&self, _aspect_ratio: f32) -> glam32::Mat4 {
        self.proj
    }
}

fn gaussian_budget(width: u32, height: u32) -> usize {
    const ONE_SORT_AT_1080P: usize = 100_000;
    const SORTS_PER_FRAME: usize = 2;
    let pixels = (width as usize).saturating_mul(height as usize).max(1);
    let one_sort = ONE_SORT_AT_1080P.saturating_mul(1920 * 1080) / pixels;
    (one_sort / SORTS_PER_FRAME).max(1)
}

fn spz_count(path: &str) -> Result<usize, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut head = [0u8; 12];
    std::io::Read::read_exact(&mut decoder, &mut head).map_err(|error| error.to_string())?;
    let magic = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
    if magic != 0x5053_474e {
        return Err("not an SPZ header".into());
    }
    let count = u32::from_le_bytes([head[8], head[9], head[10], head[11]]);
    Ok(count as usize)
}

fn cloud(width: u32, height: u32) -> wgpu_3dgs_viewer::core::Gaussians {
    let path = std::env::var("MOTOLII_GAUSSIAN").unwrap_or_else(|_| "/tmp/hornedlizard.spz".into());
    let budget = gaussian_budget(width, height);
    match spz_count(&path) {
        Ok(count) if count > budget => {
            eprintln!(
                "\x1b[31mERROR gaussian refused: {count} splats > budget {budget} at {width}x{height}, two sorts, 60fps. Not uploaded.\x1b[0m"
            );
            four_splats().into()
        }
        Ok(_) => match wgpu_3dgs_viewer::core::Gaussians::read_from_file(
            &path,
            wgpu_3dgs_viewer::core::GaussiansSource::Spz,
        ) {
            Ok(gaussians) => {
                eprintln!("A1: {} gaussians from {path}", gaussians.len());
                gaussians
            }
            Err(error) => {
                eprintln!("\x1b[31mERROR gaussian read failed: {error}\x1b[0m");
                four_splats().into()
            }
        },
        Err(_) => four_splats().into(),
    }
}

fn place(cloud: &wgpu_3dgs_viewer::core::Gaussians) -> (glam32::Vec3, f32) {
    if cloud.len() < 100 {
        return (glam32::Vec3::ZERO, 1.0);
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut zs = Vec::new();
    let step = (cloud.len() / 8000).max(1);
    for (index, gaussian) in cloud.iter_gaussian().enumerate() {
        if index % step != 0 {
            continue;
        }
        xs.push(gaussian.pos.x);
        ys.push(gaussian.pos.y);
        zs.push(gaussian.pos.z);
    }
    let (min_x, max_x) = span(&mut xs);
    let (min_y, max_y) = span(&mut ys);
    let (min_z, max_z) = span(&mut zs);
    let min = glam32::Vec3::new(min_x, min_y, min_z);
    let max = glam32::Vec3::new(max_x, max_y, max_z);
    let center = (min + max) * 0.5;
    let extent = (max - min).max_element().max(0.001);
    let scale = 1.8 / extent;
    eprintln!("A1: gaussian extent {extent:.3} scale {scale:.4}");
    (-center * scale, scale)
}

fn span(values: &mut [f32]) -> (f32, f32) {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let last = values.len().saturating_sub(1);
    let lo = values[(last as f32 * 0.08) as usize];
    let hi = values[(last as f32 * 0.92) as usize];
    (lo, hi)
}

fn four_splats() -> Vec<wgpu_3dgs_viewer::core::Gaussian> {
    let spots = [
        ([-1.45, 0.85, 0.55], [232, 96, 64, 255]),
        ([-0.85, 1.35, 0.35], [244, 196, 120, 255]),
        ([1.55, 1.05, 0.45], [96, 168, 176, 255]),
        ([0.15, 1.55, 0.2], [236, 228, 208, 255]),
    ];
    spots
        .into_iter()
        .map(|(at, rgba)| wgpu_3dgs_viewer::core::Gaussian {
            rot: glam32::Quat::IDENTITY,
            pos: glam32::Vec3::from_array(at),
            color: glam32::U8Vec4::from_array(rgba),
            sh: [glam32::Vec3::ZERO; 15],
            scale: glam32::Vec3::splat(0.28),
        })
        .collect()
}

fn wgpu_clip() -> Mat4 {
    Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
    ])
}

fn write_mats(queue: &wgpu::Queue, buffer: &wgpu::Buffer, vp: Mat4, model: Mat4) {
    let mut bytes = [0u8; 128];
    bytes[..64].copy_from_slice(&bytes_f32(&vp.to_cols_array()));
    bytes[64..].copy_from_slice(&bytes_f32(&model.to_cols_array()));
    queue.write_buffer(buffer, 0, &bytes);
}

fn bytes_f32(values: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for value in values {
        out.extend_from_slice(&value.to_ne_bytes());
    }
    out
}

fn uniform_buf(device: &wgpu::Device, size: u64, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn upload(device: &wgpu::Device, data: &[f32], label: &str) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (data.len() * 4) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: true,
    });
    buffer.slice(..).get_mapped_range_mut().copy_from_slice(&bytes_f32(data));
    buffer.unmap();
    buffer
}

fn bind_uniform(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform"),
        layout,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
    })
}

fn attach(view: &wgpu::TextureView, load: wgpu::LoadOp<wgpu::Color>) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
        depth_slice: None,
    }
}

fn cubes() -> Vec<f32> {
    let mut out = Vec::new();
    push_cube(&mut out, [-1.55, 0.15, -1.35], 0.55, [0.86, 0.45, 0.28, 1.0]);
    push_cube(&mut out, [1.45, -0.2, -1.7], 0.42, [0.25, 0.55, 0.62, 1.0]);
    push_cube(&mut out, [0.15, 0.95, -2.1], 0.36, [0.93, 0.88, 0.74, 1.0]);
    out
}

fn push_cube(out: &mut Vec<f32>, at: [f32; 3], size: f32, color: [f32; 4]) {
    let faces = [[0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0]];
    for normal in faces {
        let (u, v) = tangent(normal);
        let center = [at[0] + normal[0] * size, at[1] + normal[1] * size, at[2] + normal[2] * size];
        for corner in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
            out.extend_from_slice(&[
                center[0] + (u[0] * corner[0] + v[0] * corner[1]) * size,
                center[1] + (u[1] * corner[0] + v[1] * corner[1]) * size,
                center[2] + (u[2] * corner[0] + v[2] * corner[1]) * size,
            ]);
            out.extend_from_slice(&color);
        }
    }
}

fn tangent(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if n[1].abs() > 0.9 { [0.0, 0.0, 1.0] } else { [0.0, 1.0, 0.0] };
    let u = normalize(cross(up, n));
    (u, cross(n, u))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(0.0001);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn project(vp: Mat4, at: [f32; 3]) -> [f32; 2] {
    let clip = vp * glam::Vec4::new(at[0], at[1], at[2], 1.0);
    if clip.w <= 0.05 {
        return [2.0, 2.0];
    }
    let x = clip.x / clip.w;
    let y = clip.y / clip.w;
    [x * 0.5 + 0.5, 0.5 - y * 0.5]
}

pub(super) fn frames_needed(t: u32) -> Vec<u32> {
    let mut times = Vec::new();
    if t >= 12 {
        times.push(t - 12);
    }
    if t >= 6 {
        times.push(t - 6);
    }
    times.push(t);
    times
}

fn choose_slot(keys: &[Option<u32>], time: u32, keep: &[u32]) -> usize {
    if let Some(index) = keys.iter().position(|key| *key == Some(time)) {
        return index;
    }
    if let Some(index) = keys.iter().position(|key| key.is_none()) {
        return index;
    }
    keys.iter()
        .enumerate()
        .filter(|(_, key)| key.map(|existing| !keep.contains(&existing)).unwrap_or(true))
        .min_by_key(|(_, key)| key.unwrap_or(u32::MAX))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn eye_at(t: u32) -> glam::Vec3 {
    let yaw = super::comp::frame(t).yaw;
    glam::Vec3::new(yaw.sin() * 4.6, 1.05, yaw.cos() * 4.6)
}

fn echo_plate(eye: glam::Vec3, shift: f32, lift: f32, distance: f32, fill: f32, aspect: f32) -> Mat4 {
    let forward = (glam::Vec3::new(0.0, 0.05, 0.0) - eye).normalize();
    let right = forward.cross(glam::Vec3::Y).try_normalize().unwrap_or(glam::Vec3::X);
    let up = right.cross(forward);
    let pos = eye + forward * distance + right * shift + up * lift;
    let half_h = distance * 0.36_f32.tan() * fill;
    let half_w = half_h * aspect;
    Mat4::from_cols(
        (right * half_w).extend(0.0),
        (up * half_h).extend(0.0),
        forward.extend(0.0),
        pos.extend(1.0),
    )
}

fn write_echo(queue: &wgpu::Queue, buffer: &wgpu::Buffer, vp: Mat4, model: Mat4, fade: f32) {
    let mut bytes = [0u8; 132];
    bytes[..64].copy_from_slice(&bytes_f32(&vp.to_cols_array()));
    bytes[64..128].copy_from_slice(&bytes_f32(&model.to_cols_array()));
    bytes[128..132].copy_from_slice(&fade.to_ne_bytes());
    queue.write_buffer(buffer, 0, &bytes);
}

fn color_target(device: &wgpu::Device, size: wgpu::Extent3d, label: &str) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn echo_card() -> Vec<f32> {
    let mut out = Vec::new();
    for (x, y, u, v) in [
        (-1.0, -1.0, 0.0, 1.0),
        (1.0, -1.0, 1.0, 1.0),
        (1.0, 1.0, 1.0, 0.0),
        (-1.0, -1.0, 0.0, 1.0),
        (1.0, 1.0, 1.0, 0.0),
        (-1.0, 1.0, 0.0, 0.0),
    ] {
        out.extend_from_slice(&[x, y, 0.0, u, v]);
    }
    out
}

fn text_card() -> Vec<f32> {
    let (u0, u1) = (180.0 / 640.0, 520.0 / 640.0);
    let (v0, v1) = (16.0 / 360.0, 120.0 / 360.0);
    let mut out = Vec::new();
    for (x, y, u, v) in [
        (-0.72, -0.16, u0, v1),
        (0.72, -0.16, u1, v1),
        (0.72, 0.16, u1, v0),
        (-0.72, -0.16, u0, v1),
        (0.72, 0.16, u1, v0),
        (-0.72, 0.16, u0, v0),
    ] {
        out.extend_from_slice(&[x, y, 0.0, u, v]);
    }
    out
}

fn video_card() -> Vec<f32> {
    let mut out = Vec::new();
    for (x, y, u, v) in [
        (-1.15, -0.65, 0.0, 1.0),
        (1.15, -0.65, 1.0, 1.0),
        (1.15, 0.65, 1.0, 0.0),
        (-1.15, -0.65, 0.0, 1.0),
        (1.15, 0.65, 1.0, 0.0),
        (-1.15, 0.65, 0.0, 0.0),
    ] {
        out.extend_from_slice(&[x, y, 0.0, u, v]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{choose_slot, frames_needed, gaussian_budget};

    #[test]
    fn scrub_does_not_keep_the_playback_head() {
        assert_eq!(frames_needed(40), vec![28, 34, 40]);
        assert_eq!(frames_needed(8), vec![2, 8]);
        let mut keys: Vec<Option<u32>> = (28..=40).map(Some).collect();
        let scrubbed = frames_needed(8);
        for time in &scrubbed {
            let index = choose_slot(&keys, *time, &scrubbed);
            keys[index] = Some(*time);
        }
        assert!(786_233 > gaussian_budget(2000, 1184));
        assert!(gaussian_budget(2000, 1184) < 80_000);
        assert!(keys.contains(&Some(2)));
        assert!(keys.contains(&Some(8)));
        assert!(!keys.contains(&Some(28)));
    }
}
