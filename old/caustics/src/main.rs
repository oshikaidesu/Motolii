//! Spectral caustics on a floor from glass shapes, in wgpu.
//!
//! The optics are borrowed unchanged from visgl/luma.gl's SpectralCausticsRenderer (MIT):
//! `shaders/trace.wgsl` (six wavelengths, per-wavelength refraction, exit-surface search) and
//! `shaders/splat.wgsl` (additive Gaussian photon splats into an XYZ map). What is written here is
//! only the plumbing: light-space capture, buffers, the mesh of the glass, and the readout.
mod geometry;

use bytemuck::{Pod, Zeroable};
use geometry::Mesh;
use glam::{Mat4, Vec3};
use std::borrow::Cow;
use std::time::Instant;
use wgpu::util::DeviceExt;

const BANDS: u32 = 6;

fn env_f32(name: &str, default: f32) -> f32 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn capture() -> u32 { env_f32("CAUSTIC_CAPTURE", 256.0) as u32 }
fn map_size() -> u32 { env_f32("CAUSTIC_MAP", 1024.0) as u32 }
fn splat_radius() -> f32 { env_f32("CAUSTIC_RADIUS", 3.0) }
fn floor_size() -> f32 { env_f32("CAUSTIC_FLOOR", 3.1) }

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TraceUniforms {
    inverse_light_view_projection: [f32; 16],
    light_view_projection: [f32; 16],
    receiver_origin_width: [f32; 4],
    receiver_tangent_height: [f32; 4],
    receiver_bitangent_intensity: [f32; 4],
    receiver_normal_ior: [f32; 4],
    absorption_dispersion: [f32; 4],
    target_sizes: [f32; 4],
}

struct Pass {
    capture_front: wgpu::RenderPipeline,
    capture_back: wgpu::RenderPipeline,
    capture_layout: wgpu::BindGroupLayout,
    trace: wgpu::ComputePipeline,
    trace_layout: wgpu::BindGroupLayout,
    splat: wgpu::RenderPipeline,
    footprint: wgpu::RenderPipeline,
    splat_layout: wgpu::BindGroupLayout,
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn shader(device: &wgpu::Device, label: &str, src: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(src.to_string())),
    })
}

fn tex_entry(binding: u32, depth: bool, vis: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: vis,
        ty: wgpu::BindingType::Texture {
            sample_type: if depth {
                wgpu::TextureSampleType::Depth
            } else {
                wgpu::TextureSampleType::Float { filterable: false }
            },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn buf_entry(binding: u32, ty: wgpu::BufferBindingType, vis: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: vis,
        ty: wgpu::BindingType::Buffer { ty, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn build_passes(gpu: &Gpu) -> Pass {
    let d = &gpu.device;
    let capture_sh = shader(d, "capture", include_str!("shaders/capture.wgsl"));
    let trace_sh = shader(d, "trace", include_str!("shaders/trace_hits.wgsl"));
    let cells_sh = shader(d, "cells", include_str!("shaders/cells.wgsl"));

    let capture_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("capture"),
        entries: &[buf_entry(0, wgpu::BufferBindingType::Uniform, wgpu::ShaderStages::VERTEX)],
    });
    let capture_pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("capture"),
        bind_group_layouts: &[Some(&capture_layout)],
        immediate_size: 0,
    });
    let vertex_attrs = [
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
        wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
    ];
    let vbl = wgpu::VertexBufferLayout {
        array_stride: 24,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attrs,
    };
    let make_capture = |cull: wgpu::Face, compare: wgpu::CompareFunction| {
        d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("capture"),
            layout: Some(&capture_pl),
            vertex: wgpu::VertexState {
                module: &capture_sh,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[vbl.clone()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(cull),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(compare),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &capture_sh,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    let capture_front = make_capture(wgpu::Face::Back, wgpu::CompareFunction::Less);
    let capture_back = make_capture(wgpu::Face::Front, wgpu::CompareFunction::Greater);

    let cs = wgpu::ShaderStages::COMPUTE;
    let trace_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("trace"),
        entries: &[
            tex_entry(0, false, cs),
            tex_entry(1, true, cs),
            tex_entry(2, false, cs),
            tex_entry(3, true, cs),
            buf_entry(4, wgpu::BufferBindingType::Storage { read_only: false }, cs),
            buf_entry(5, wgpu::BufferBindingType::Uniform, cs),
        ],
    });
    let trace_pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("trace"),
        bind_group_layouts: &[Some(&trace_layout)],
        immediate_size: 0,
    });
    let trace = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("trace"),
        layout: Some(&trace_pl),
        module: &trace_sh,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let vs = wgpu::ShaderStages::VERTEX;
    let splat_layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("cells"),
        entries: &[
            buf_entry(0, wgpu::BufferBindingType::Storage { read_only: true }, vs),
            buf_entry(1, wgpu::BufferBindingType::Uniform, vs),
        ],
    });
    let splat_pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("cells"),
        bind_group_layouts: &[Some(&splat_layout)],
        immediate_size: 0,
    });
    let additive = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    let make_cells = |entry: &str| {
        d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cells"),
            layout: Some(&splat_pl),
            vertex: wgpu::VertexState {
                module: &cells_sh,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &cells_sh,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState { color: additive, alpha: additive }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    let splat = make_cells("vsCaustic");
    let footprint = make_cells("vsFootprint");

    Pass { capture_front, capture_back, capture_layout, trace, trace_layout, splat, footprint, splat_layout }
}

struct Object {
    mesh: Mesh,
    vb: wgpu::Buffer,
    ib: wgpu::Buffer,
    light_vp: Mat4,
    capture_ub: wgpu::Buffer,
    trace_ub: wgpu::Buffer,
    // capture targets
    front_n: wgpu::TextureView,
    front_d: wgpu::TextureView,
    back_n: wgpu::TextureView,
    back_d: wgpu::TextureView,
    splats: wgpu::Buffer,
}

struct Material {
    n550: f32,
    dispersion: f32,
    absorption: [f32; 3],
    intensity: f32,
}

fn make_target(gpu: &Gpu, format: wgpu::TextureFormat, extra: wgpu::TextureUsages) -> wgpu::TextureView {
    gpu.device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: capture(), height: capture(), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | extra,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

fn light_matrices(mesh: &Mesh, sun_dir: Vec3) -> Mat4 {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for v in &mesh.vertices {
        lo = lo.min(Vec3::from(v.position));
        hi = hi.max(Vec3::from(v.position));
    }
    let center = (lo + hi) * 0.5;
    let up = if sun_dir.normalize().y.abs() > 0.99 { Vec3::X } else { Vec3::Y };
    let view = Mat4::look_at_rh(center - sun_dir.normalize() * 10.0, center, up);
    let (mut vlo, mut vhi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for v in &mesh.vertices {
        let p = view.transform_point3(Vec3::from(v.position));
        vlo = vlo.min(p);
        vhi = vhi.max(p);
    }
    let pad = 0.02;
    let proj = Mat4::orthographic_rh(vlo.x - pad, vhi.x + pad, vlo.y - pad, vhi.y + pad, -vhi.z - 0.1, -vlo.z + 0.1);
    proj * view
}

fn make_object(gpu: &Gpu, mesh: Mesh, sun_dir: Vec3, mat: &Material) -> Object {
    let light_vp = light_matrices(&mesh, sun_dir);
    let vb = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vb"),
        contents: bytemuck::cast_slice(&mesh.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let ib = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ib"),
        contents: bytemuck::cast_slice(&mesh.indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let capture_ub = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("capture-ub"),
        contents: bytemuck::cast_slice(&light_vp.to_cols_array()),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let u = TraceUniforms {
        inverse_light_view_projection: light_vp.inverse().to_cols_array(),
        light_view_projection: light_vp.to_cols_array(),
        receiver_origin_width: [0.0, 0.0, 0.0, floor_size()],
        receiver_tangent_height: [1.0, 0.0, 0.0, floor_size()],
        receiver_bitangent_intensity: [0.0, 0.0, 1.0, mat.intensity],
        receiver_normal_ior: [0.0, 1.0, 0.0, mat.n550],
        absorption_dispersion: [mat.absorption[0], mat.absorption[1], mat.absorption[2], mat.dispersion],
        target_sizes: [capture() as f32, map_size() as f32, splat_radius(), 0.0],
    };
    let trace_ub = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("trace-ub"),
        contents: bytemuck::bytes_of(&u),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let splats = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("splats"),
        size: (capture() * capture() * BANDS) as u64 * 32,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    Object {
        mesh,
        vb,
        ib,
        light_vp,
        capture_ub,
        trace_ub,
        front_n: make_target(gpu, wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::empty()),
        front_d: make_target(gpu, wgpu::TextureFormat::Depth32Float, wgpu::TextureUsages::empty()),
        back_n: make_target(gpu, wgpu::TextureFormat::Rgba16Float, wgpu::TextureUsages::empty()),
        back_d: make_target(gpu, wgpu::TextureFormat::Depth32Float, wgpu::TextureUsages::empty()),
        splats,
    }
}

/// Records capture + trace + splat for one refractor into `map`.
fn record(gpu: &Gpu, p: &Pass, o: &Object, map: &wgpu::TextureView, first: bool, enc: &mut wgpu::CommandEncoder, ts: Option<(&wgpu::QuerySet, u32)>) {
    let d = &gpu.device;
    let capture_bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.capture_layout,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: o.capture_ub.as_entire_binding() }],
    });
    for (pass_i, (pipeline, normal, depth, clear_depth)) in [
        (&p.capture_front, &o.front_n, &o.front_d, 1.0f32),
        (&p.capture_back, &o.back_n, &o.back_d, 0.0f32),
    ]
    .into_iter()
    .enumerate()
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("capture"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: normal,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.5, g: 0.5, b: 0.5, a: 0.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_depth),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: ts.map(|(qs, base)| wgpu::RenderPassTimestampWrites {
                query_set: qs,
                beginning_of_pass_write_index: Some(base + pass_i as u32 * 2),
                end_of_pass_write_index: Some(base + pass_i as u32 * 2 + 1),
            }),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &capture_bg, &[]);
        pass.set_vertex_buffer(0, o.vb.slice(..));
        pass.set_index_buffer(o.ib.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..o.mesh.indices.len() as u32, 0, 0..1);
    }

    let trace_bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.trace_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&o.front_n) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&o.front_d) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&o.back_n) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&o.back_d) },
            wgpu::BindGroupEntry { binding: 4, resource: o.splats.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: o.trace_ub.as_entire_binding() },
        ],
    });
    {
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("trace"),
            timestamp_writes: ts.map(|(qs, base)| wgpu::ComputePassTimestampWrites {
                query_set: qs,
                beginning_of_pass_write_index: Some(base + 4),
                end_of_pass_write_index: Some(base + 5),
            }),
        });
        pass.set_pipeline(&p.trace);
        pass.set_bind_group(0, &trace_bg, &[]);
        pass.dispatch_workgroups((capture() * capture() + 63) / 64, 1, 1);
    }

    let splat_bg = d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &p.splat_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: o.splats.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: o.trace_ub.as_entire_binding() },
        ],
    });
    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("cells"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: map,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if first { wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT) } else { wgpu::LoadOp::Load },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: ts.map(|(qs, base)| wgpu::RenderPassTimestampWrites {
            query_set: qs,
            beginning_of_pass_write_index: Some(base + 6),
            end_of_pass_write_index: Some(base + 7),
        }),
        occlusion_query_set: None,
        multiview_mask: None,
    });
    let instances = (capture() - 1) * (capture() - 1) * BANDS;
    pass.set_bind_group(0, &splat_bg, &[]);
    pass.set_pipeline(&p.splat);
    pass.draw(0..6, 0..instances);
    pass.set_pipeline(&p.footprint);
    pass.draw(0..6, 0..instances / BANDS);
}

fn read_map(gpu: &Gpu, map: &wgpu::Texture) -> Vec<f32> {
    let bpr = map_size() * 8;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (bpr * map_size()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: map,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(map_size()) },
        },
        wgpu::Extent3d { width: map_size(), height: map_size(), depth_or_array_layers: 1 },
    );
    gpu.queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = slice.get_mapped_range();
    let halves: &[u16] = bytemuck::cast_slice(&data);
    halves.iter().map(|h| half::f16::from_bits(*h).to_f32()).collect()
}

fn write_png(path: &str, w: u32, h: u32, rgb: &[u8]) {
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(rgb).unwrap();
}

fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    let info = adapter.get_info();
    println!("adapter {} ({:?})", info.name, info.backend);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { required_features: wgpu::Features::TIMESTAMP_QUERY, ..Default::default() })).expect("device");
    let gpu = Gpu { device, queue };
    let passes = build_passes(&gpu);

    let sun = Vec3::new(0.92, -0.42, 0.22);
    let glass = Material { n550: 1.52, dispersion: env_f32("CAUSTIC_DISPERSION", 0.030), absorption: [0.02, 0.01, 0.02], intensity: env_f32("CAUSTIC_INTENSITY", 6.0) };
    let objects = vec![
        make_object(&gpu, geometry::glass_loop(), sun, &glass),
        make_object(&gpu, geometry::pebble(Vec3::new(0.62, 0.075, -0.22), Vec3::new(0.16, 0.075, 0.12), 0.5), sun, &glass),
    ];

    let map_tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("caustic-map"),
        size: wgpu::Extent3d { width: map_size(), height: map_size(), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let map_view = map_tex.create_view(&Default::default());

    let frame = |gpu: &Gpu| {
        let mut enc = gpu.device.create_command_encoder(&Default::default());
        for (i, o) in objects.iter().enumerate() {
            record(gpu, &passes, o, &map_view, i == 0, &mut enc, None);
        }
        gpu.queue.submit([enc.finish()]);
    };

    frame(&gpu);
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let t0 = Instant::now();
    let runs = 120;
    for _ in 0..runs {
        frame(&gpu);
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
    let ms = t0.elapsed().as_secs_f64() * 1000.0 / runs as f64;
    println!(
        "frame (capture+trace+splat, {} objects, {} photons each): {:.2} ms   budget 60fps = 16.67 ms",
        objects.len(),
        capture() * capture() * BANDS,
        ms
    );

    // Per-pass GPU time from timestamp queries (not CPU wall time): median of 41 frames.
    {
        let count = objects.len() as u32 * 8;
        let qs = gpu.device.create_query_set(&wgpu::QuerySetDescriptor { label: Some("ts"), ty: wgpu::QueryType::Timestamp, count });
        let resolve = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: count as u64 * 8,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: count as u64 * 8,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let period = gpu.queue.get_timestamp_period() as f64 / 1.0e6; // ms per tick
        let mut samples: Vec<[f64; 4]> = Vec::new();
        for _ in 0..41 {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            for (i, o) in objects.iter().enumerate() {
                record(&gpu, &passes, o, &map_view, i == 0, &mut enc, Some((&qs, i as u32 * 8)));
            }
            enc.resolve_query_set(&qs, 0..count, &resolve, 0);
            enc.copy_buffer_to_buffer(&resolve, 0, &read, 0, count as u64 * 8);
            gpu.queue.submit([enc.finish()]);
            let slice = read.slice(..);
            slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
            gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            {
                let data = slice.get_mapped_range();
                let t: &[u64] = bytemuck::cast_slice(&data);
                let mut sums = [0f64; 4];
                for i in 0..objects.len() {
                    for k in 0..4 {
                        sums[k] += (t[i * 8 + k * 2 + 1].saturating_sub(t[i * 8 + k * 2])) as f64 * period;
                    }
                }
                samples.push(sums);
            }
            read.unmap();
        }
        let med = |k: usize| {
            let mut v: Vec<f64> = samples.iter().map(|s| s[k]).collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v[v.len() / 2]
        };
        let mut totals: Vec<f64> = samples.iter().map(|s| s.iter().sum()).collect();
        totals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "GPU median ({} objects): capture front {:.3}  back {:.3}  trace {:.3}  cells {:.3}  | total median {:.3}  min {:.3}  max {:.3} ms",
            objects.len(), med(0), med(1), med(2), med(3), totals[totals.len() / 2], totals[0], totals[totals.len() - 1]
        );
    }

    // Read the XYZ map back and make a top-down picture of the floor with the caustic added.
    let px = read_map(&gpu, &map_tex);
    let mut rgb = vec![0u8; (map_size() * map_size() * 3) as usize];
    let (mut max_y, mut sum_y) = (0f32, 0f64);
    let exposure = env_f32("CAUSTIC_EXPOSURE", 0.9);
    for i in 0..(map_size() * map_size()) as usize {
        let (x, y, z, cover) = (px[i * 4], px[i * 4 + 1], px[i * 4 + 2], px[i * 4 + 3].clamp(0.0, 1.0));
        max_y = max_y.max(y);
        sum_y += y as f64;
        let lin = [
            3.2409699 * x - 1.5373832 * y - 0.4986108 * z,
            -0.9692436 * x + 1.8759675 * y + 0.0415551 * z,
            0.0556301 * x - 0.2039770 * y + 1.0569715 * z,
        ];
        // pale floor: ambient + the sun, minus the sun the glass takes, plus the light the glass gives back
        let sun = env_f32("CAUSTIC_SUN", 1.0);
        for c in 0..3 {
            let v = 0.10 + 0.55 * (sun * (1.0 - cover) + lin[c].max(0.0) * sun) * exposure;
            let t = v / (1.0 + v * 0.35);
            let s = if t <= 0.0031308 { 12.92 * t } else { 1.055 * t.powf(1.0 / 2.4) - 0.055 };
            rgb[i * 3 + c] = (s.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    let alpha: Vec<f32> = (0..(map_size() * map_size()) as usize).map(|i| px[i * 4 + 3]).collect();
    println!("alpha(cover) max {:.2}  pixels>0.5: {}", alpha.iter().cloned().fold(0.0, f32::max), alpha.iter().filter(|a| **a > 0.5).count());
    println!("map Y max {:.3} mean {:.5}", max_y, sum_y / (map_size() * map_size()) as f64);
    write_png("floor-caustics.png", map_size(), map_size(), &rgb);
    println!("wrote floor-caustics.png");
    let _ = objects.iter().map(|o| o.light_vp).count();
}
