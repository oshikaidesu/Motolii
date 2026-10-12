// rayq: a probe. Can the GPU's own ray-against-mesh query, as wgpu 29 offers it (EXPERIMENTAL_RAY_QUERY), follow the
// lines of sight through the glass band, and how fast? Not a renderer: one compute pass, numbers out.
//
//   rayq one                         one triangle, two lines (does the query work at all?)
//   rayq shapes <studio.ktx> <out.png>   simple glass solids (a torus, a box with round edges), three models of the
//                                    optics by MODEL=0|1|2 (src/shapes.rs, src/glass.wgsl)
//   rayq band <mesh.bin> [check|time|pace]   the band's mesh (written by filament/band.mm MODEL=D), band.mm's "ref" camera,
//                                    the same rule as model D (band_bvh.mat). check: against a CPU trace that tries
//                                    every triangle. time: GPU time of the pass alone.
//
// mesh.bin: u32 vertices, u32 indices, f32 xyz per vertex, u32 per index, then f32 xyz per vertex (smooth normals).
mod shapes;

use std::time::Instant;
use wgpu::util::DeviceExt;

const W: u32 = 1080;
const H: u32 = 1920;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub fn open() -> Gpu {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).expect("adapter");
    let info = adapter.get_info();
    println!("adapter: {} ({:?}); ray query offered: {}", info.name, info.backend, adapter.features().contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY));
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::EXPERIMENTAL_RAY_QUERY,
        required_limits: wgpu::Limits::default().using_minimum_supported_acceleration_structure_values(),
        // (wgpu asks for this to be said out loud: the ray-tracing API may still change or have faults)
        experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
        ..Default::default()
    }))
    .expect("device");
    Gpu { device, queue }
}

/// The mesh as the GPU's own acceleration structure: one bottom level, one instance of it (unmoved) in the top level.
pub fn structure(g: &Gpu, pos: &[f32], idx: &[u32]) -> (wgpu::Tlas, f64) {
    let vb = g.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(pos), usage: wgpu::BufferUsages::BLAS_INPUT });
    let ib = g.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(idx), usage: wgpu::BufferUsages::BLAS_INPUT });
    let size = wgpu::BlasTriangleGeometrySizeDescriptor {
        vertex_format: wgpu::VertexFormat::Float32x3,
        vertex_count: (pos.len() / 3) as u32,
        index_format: Some(wgpu::IndexFormat::Uint32),
        index_count: Some(idx.len() as u32),
        flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
    };
    let blas = g.device.create_blas(
        &wgpu::CreateBlasDescriptor { label: None, flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE, update_mode: wgpu::AccelerationStructureUpdateMode::Build },
        wgpu::BlasGeometrySizeDescriptors::Triangles { descriptors: vec![size.clone()] },
    );
    let mut tlas = g.device.create_tlas(&wgpu::CreateTlasDescriptor {
        label: None,
        max_instances: 1,
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    });
    *tlas.get_mut_single(0).unwrap() = Some(wgpu::TlasInstance::new(&blas, [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0], 0, 0xff));
    let t0 = Instant::now();
    let mut enc = g.device.create_command_encoder(&Default::default());
    enc.build_acceleration_structures(
        std::iter::once(&wgpu::BlasBuildEntry {
            blas: &blas,
            geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
                size: &size,
                vertex_buffer: &vb,
                first_vertex: 0,
                vertex_stride: 12,
                index_buffer: Some(&ib),
                first_index: Some(0),
                transform_buffer: None,
                transform_buffer_offset: None,
            }]),
        }),
        std::iter::once(&tlas),
    );
    g.queue.submit(Some(enc.finish()));
    g.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    (tlas, t0.elapsed().as_secs_f64() * 1000.0)
}

fn read(g: &Gpu, buf: &wgpu::Buffer, bytes: u64) -> Vec<u8> {
    let back = g.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: bytes, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = g.device.create_command_encoder(&Default::default());
    enc.copy_buffer_to_buffer(buf, 0, &back, 0, bytes);
    g.queue.submit(Some(enc.finish()));
    back.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    g.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let v = back.slice(..).get_mapped_range().to_vec();
    v
}

fn one() {
    let g = open();
    let (tlas, _) = structure(&g, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0], &[0, 1, 2]);
    let module = g.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(include_str!("one.wgsl").into()) });
    let pipeline = g.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None });
    let answer = g.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 32, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let bind = g.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: tlas.as_binding() }, wgpu::BindGroupEntry { binding: 1, resource: answer.as_entire_binding() }],
    });
    let mut enc = g.device.create_command_encoder(&Default::default());
    {
        let mut p = enc.begin_compute_pass(&Default::default());
        p.set_pipeline(&pipeline);
        p.set_bind_group(0, &bind, &[]);
        p.dispatch_workgroups(1, 1, 1);
    }
    g.queue.submit(Some(enc.finish()));
    let bytes = read(&g, &answer, 32);
    let a: &[f32] = bytemuck::cast_slice(&bytes);
    println!("line through (0.25, 0.25): kind {} (1 = a triangle), way {}, where in the triangle ({}, {})   expected: 1, 1, (0.25, 0.25)", a[0], a[1], a[2], a[3]);
    println!("line past the triangle:    kind {} (0 = nothing met)", a[4]);
    let ok = a[0] == 1.0 && (a[1] - 1.0).abs() < 1e-5 && (a[2] - 0.25).abs() < 1e-5 && (a[3] - 0.25).abs() < 1e-5 && a[4] == 0.0;
    println!("{}", if ok { "PASS" } else { "FAIL" });
}

// ---- the band

type V3 = [f32; 3];
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn mul(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn norm(a: V3) -> V3 { mul(a, 1.0 / dot(a, a).sqrt()) }

struct Mesh { pos: Vec<f32>, idx: Vec<u32>, nrm: Vec<f32> }
impl Mesh {
    fn p(&self, i: u32) -> V3 { let i = i as usize * 3; [self.pos[i], self.pos[i + 1], self.pos[i + 2]] }
    fn n(&self, i: u32) -> V3 { let i = i as usize * 3; [self.nrm[i], self.nrm[i + 1], self.nrm[i + 2]] }
}
fn load(path: &str) -> Mesh {
    let raw = std::fs::read(path).expect("mesh file");
    let head: [u32; 2] = [u32::from_le_bytes(raw[0..4].try_into().unwrap()), u32::from_le_bytes(raw[4..8].try_into().unwrap())];
    let (nv, ni) = (head[0] as usize, head[1] as usize);
    let f = |from: usize, n: usize| -> Vec<f32> { raw[from..from + n * 4].chunks(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect() };
    let pos = f(8, nv * 3);
    let idx: Vec<u32> = raw[8 + nv * 12..8 + nv * 12 + ni * 4].chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
    assert!(raw.len() >= 8 + nv * 24 + ni * 4, "mesh file without normals: run filament/out/band with MODEL=D again");
    let nrm = f(8 + nv * 12 + ni * 4, nv * 3);
    Mesh { pos, idx, nrm }
}

// band.mm's "ref" scene
const EYE: V3 = [0.9, 0.55, 2.0];
const TARGET: V3 = [1.25, 0.75, 0.0];
const FOV: f32 = 30.0;
const IOR: f32 = 1.5;
const CAP: i32 = 32;

fn camera() -> (V3, V3, V3) {   // forward, right * tan * aspect, up * tan
    let f = norm(sub(TARGET, EYE));
    let r = norm(cross(f, [0.0, 1.0, 0.0]));
    let u = cross(r, f);
    let th = (0.5 * FOV * std::f32::consts::PI / 180.0).tan();
    (f, mul(r, th * W as f32 / H as f32), mul(u, th))
}

#[derive(Clone, Copy)]
struct Hit { t: f32, tri: u32, u: f32, v: f32 }
/// Every triangle tried (the test twin; same tolerances as band.mm's CHECK for model D).
fn first_hit(m: &Mesh, o: V3, d: V3, t_least: f32) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    for (k, tri) in m.idx.chunks_exact(3).enumerate() {
        let a = m.p(tri[0]);
        let e1 = sub(m.p(tri[1]), a);
        let e2 = sub(m.p(tri[2]), a);
        let pv = cross(d, e2);
        let det = dot(e1, pv);
        if det.abs() < 1e-20 { continue; }
        let tv = sub(o, a);
        let u = dot(tv, pv) / det;
        if !(-1e-5..=1.00001).contains(&u) { continue; }
        let qv = cross(tv, e1);
        let v = dot(d, qv) / det;
        if v < -1e-5 || u + v > 1.00001 { continue; }
        let t = dot(e2, qv) / det;
        if t >= t_least && best.map_or(true, |b| t < b.t) { best = Some(Hit { t, tri: k as u32, u, v }); }
    }
    best
}
fn flat_normal(m: &Mesh, tri: u32, d: V3) -> V3 {
    let i = tri as usize * 3;
    let a = m.p(m.idx[i]);
    let n = norm(cross(sub(m.p(m.idx[i + 1]), a), sub(m.p(m.idx[i + 2]), a)));
    if dot(n, d) > 0.0 { mul(n, -1.0) } else { n }
}
/// One line of sight by the rule of model D. None: the line meets no glass.
/// (mirrorings before leaving, or 99: still inside after CAP, or -1: no surface ahead; whole way inside; first way; first triangle; triangle entered)
fn cpu_line(m: &Mesh, x: u32, y: u32) -> Option<(f32, f32, f32, u32, u32)> {
    let (f, r, u) = camera();
    let ndx = (x as f32 + 0.5) / W as f32 * 2.0 - 1.0;
    let ndy = (y as f32 + 0.5) / H as f32 * 2.0 - 1.0;
    let d = norm(add(f, add(mul(r, ndx), mul(u, ndy))));
    let h1 = first_hit(m, EYE, d, 1e-4)?;
    let i = h1.tri as usize * 3;
    let mut n = norm(add(add(mul(m.n(m.idx[i]), 1.0 - h1.u - h1.v), mul(m.n(m.idx[i + 1]), h1.u)), mul(m.n(m.idx[i + 2]), h1.v)));   // (the way in: the smooth normal)
    if dot(n, d) > 0.0 { n = mul(n, -1.0); }
    let ci = -dot(n, d);
    let k = 1.0 - (1.0 - ci * ci) / (IOR * IOR);
    let mut t = norm(add(mul(d, 1.0 / IOR), mul(n, ci / IOR - k.max(0.0).sqrt())));
    let mut o = add(add(EYE, mul(d, h1.t)), mul(t, 0.001));
    let (mut count, mut way, mut first_t, mut first_tri) = (99.0f32, 0.0f32, -1.0f32, 0u32);
    for b in 0..=CAP {
        let Some(h) = first_hit(m, o, t, -1e-5) else { count = -1.0; break; };
        way += h.t + 0.001;
        if b == 0 { first_t = h.t + 0.001; first_tri = h.tri; }
        let fnm = flat_normal(m, h.tri, t);
        let co = -dot(fnm, t);
        if 1.0 - IOR * IOR * (1.0 - co * co) >= 0.0 { count = b as f32; break; }
        let at = add(o, mul(t, h.t));
        t = norm(add(t, mul(fnm, 2.0 * co)));
        o = add(at, mul(t, 0.001));
    }
    Some((count, way, first_t, first_tri, h1.tri))
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params { eye: [f32; 4], fwd: [f32; 4], right: [f32; 4], up: [f32; 4], size: [u32; 2], ior: f32, most: f32, look: [f32; 4] }

struct Band { g: Gpu, module: wgpu::ShaderModule, tlas: wgpu::Tlas, pos: wgpu::Buffer, idx: wgpu::Buffer, nrm: wgpu::Buffer, pipe_check: wgpu::ComputePipeline, pipe_draw: wgpu::ComputePipeline, bind_check: wgpu::BindGroup, bind_draw: wgpu::BindGroup, params: wgpu::Buffer, numbers: wgpu::Buffer }

fn band_open(m: &Mesh) -> Band {
    let g = open();
    let (tlas, ms) = structure(&g, &m.pos, &m.idx);
    println!("acceleration structure of {} triangles built in {:.1} ms (submitted and waited for)", m.idx.len() / 3, ms);
    let module = g.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(std::env::var("SHADER").map(|p| std::fs::read_to_string(p).expect("shader file")).unwrap_or(include_str!("band.wgsl").to_string()).into()) });   // (SHADER: tests)
    let pipe = |entry: &str| g.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some(entry), compilation_options: Default::default(), cache: None });
    let (pipe_check, pipe_draw) = (pipe("check"), pipe("draw"));
    let store = |data: &[u8]| g.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: data, usage: wgpu::BufferUsages::STORAGE });
    let (pos, idx, nrm) = (store(bytemuck::cast_slice(&m.pos)), store(bytemuck::cast_slice(&m.idx)), store(bytemuck::cast_slice(&m.nrm)));
    let params = g.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: std::mem::size_of::<Params>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let numbers = g.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (W * H) as u64 * 32, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
    let picture = g.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let view = picture.create_view(&Default::default());
    let common = |pipeline: &wgpu::ComputePipeline, at: u32, last: wgpu::BindingResource| {
        g.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: tlas.as_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: params.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: pos.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: idx.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: nrm.as_entire_binding() },
                wgpu::BindGroupEntry { binding: at, resource: last },
            ],
        })
    };
    let bind_check = common(&pipe_check, 5, wgpu::BindingResource::Buffer(numbers.as_entire_buffer_binding()));
    let bind_draw = common(&pipe_draw, 6, wgpu::BindingResource::TextureView(&view));
    // (the bind groups hold the binding numbers 5 and 6 apart: see band.wgsl)
    Band { g, module, tlas, pos, idx, nrm, pipe_check, pipe_draw, bind_check, bind_draw, params, numbers }
}
fn set(b: &Band, most: f32) { set_look(b, most, [0.0; 4]); }
fn set_look(b: &Band, most: f32, look: [f32; 4]) {
    let (f, r, u) = camera();
    let v4 = |a: V3| [a[0], a[1], a[2], 0.0];
    b.g.queue.write_buffer(&b.params, 0, bytemuck::bytes_of(&Params { eye: v4(EYE), fwd: v4(f), right: v4(r), up: v4(u), size: [W, H], ior: IOR, most, look }));
}

fn check(m: &Mesh, every: u32) {
    let b = band_open(m);
    set(&b, CAP as f32);
    let mut enc = b.g.device.create_command_encoder(&Default::default());
    {
        let mut p = enc.begin_compute_pass(&Default::default());
        p.set_pipeline(&b.pipe_check);
        p.set_bind_group(0, &b.bind_check, &[]);
        p.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
    }
    b.g.queue.submit(Some(enc.finish()));
    let bytes = read(&b.g, &b.numbers, (W * H) as u64 * 32);
    let gpu: &[f32] = bytemuck::cast_slice(&bytes);

    // the CPU's lines, on a grid of pixels, on all cores
    let mut pixels = Vec::new();
    let mut y = every / 2;
    while y < H { let mut x = every / 2; while x < W { pixels.push((x, y)); x += every; } y += every; }
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let t0 = Instant::now();
    let mut cpu: Vec<Option<(f32, f32, f32, u32, u32)>> = vec![None; pixels.len()];
    std::thread::scope(|s| {
        for (part, out) in pixels.chunks(pixels.len().div_ceil(threads)).zip(cpu.chunks_mut(pixels.len().div_ceil(threads))) {
            s.spawn(move || for (p, o) in part.iter().zip(out.iter_mut()) { *o = cpu_line(m, p.0, p.1); });
        }
    });
    println!("CPU: {} lines of sight, every triangle tried, {:.1} s", pixels.len(), t0.elapsed().as_secs_f64());

    let slot = |c: f32| if c < -0.5 { CAP as usize + 2 } else if c > 98.0 { CAP as usize + 1 } else { c.round() as usize };
    let (mut lines, mut gpu_none, mut cpu_none, mut same_in, mut same_first, mut same_count, mut firsts) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut cpu_after, mut gpu_after) = (vec![0u64; CAP as usize + 3], vec![0u64; CAP as usize + 3]);
    let (mut len_err, mut way_err) = (Vec::new(), Vec::new());
    for (p, c) in pixels.iter().zip(cpu.iter()) {
        let at = ((p.1 * W + p.0) * 8) as usize;
        let glass = gpu[at + 4] > 0.5;
        match c {
            None => { if glass { cpu_none += 1; } }
            Some(c) => {
                lines += 1;
                if !glass { gpu_none += 1; continue; }
                cpu_after[slot(c.0)] += 1;
                gpu_after[slot(gpu[at])] += 1;
                if c.4 == gpu[at + 5] as u32 { same_in += 1; }
                if c.0 == gpu[at] { same_count += 1; }
                if c.2 > 0.0 && gpu[at + 2] > 0.0 {
                    firsts += 1;
                    if c.3 == gpu[at + 3] as u32 { same_first += 1; }
                    len_err.push((gpu[at + 2] - c.2).abs() / c.2);
                    if c.0 == gpu[at] && c.0 >= 0.0 && c.0 < 98.0 { way_err.push((gpu[at + 1] - c.1).abs() / c.1); }
                }
            }
        }
    }
    len_err.sort_by(|a, b| a.total_cmp(b));
    way_err.sort_by(|a, b| a.total_cmp(b));
    let q = |v: &Vec<f32>, at: f64| if v.is_empty() { 0.0 } else { v[(at * (v.len() - 1) as f64) as usize] * 100.0 };
    let both = lines - gpu_none;
    let pc = |a: u64, of: u64| 100.0 * a as f64 / of.max(1) as f64;
    println!("CHECK: {} lines into the glass on the CPU; the GPU's query says no glass for {} of them, and glass for {} the CPU says miss", lines, gpu_none, cpu_none);
    println!("    triangle entered: the same for {:.2}% of {}", pc(same_in, both), both);
    println!("    first surface inside: the same triangle for {:.2}% ({} of {}); way to it off by {:.4}% (median), {:.4}% (9 in 10), {:.3}% (worst)", pc(same_first, firsts), same_first, firsts, q(&len_err, 0.5), q(&len_err, 0.9), q(&len_err, 1.0));
    println!("    mirrorings before leaving: the same number for {:.2}% of {}; whole way inside off by {:.4}% (median), {:.4}% (9 in 10) where the number is the same", pc(same_count, both), both, q(&way_err, 0.5), q(&way_err, 0.9));
    println!("    mirrorings              CPU      GPU (ray query)");
    for i in 0..CAP as usize + 3 {
        if cpu_after[i] == 0 && gpu_after[i] == 0 { continue; }
        let name = if i == CAP as usize + 1 { format!("still inside after {CAP}") } else if i == CAP as usize + 2 { "no surface ahead".to_string() } else { i.to_string() };
        println!("    {:<22} {:6.2}%  {:6.2}%", name, pc(cpu_after[i], both), pc(gpu_after[i], both));
    }
    // every glass pixel (GPU only): out within N mirrorings; a line still inside at the limit is NOT out
    let mut after = vec![0u64; CAP as usize + 3];
    let mut glass = 0u64;
    for px in gpu.chunks_exact(8) {
        if px[4] > 0.5 { glass += 1; after[slot(px[0])] += 1; }
    }
    print!("every glass pixel ({glass}, GPU): out within N mirrorings (the rest is NOT out):");
    let mut sum = 0;
    for i in 0..=CAP as usize {
        sum += after[i];
        if i <= 5 || i == 8 || i == 16 || i == CAP as usize { print!("  {}: {:.2}%", i, pc(sum, glass)); }
    }
    println!("  | still inside after {}: {:.2}%, no surface ahead: {:.2}%", CAP, pc(after[CAP as usize + 1], glass), pc(after[CAP as usize + 2], glass));
}

fn pass(b: &Band, numbers: bool) {
    let mut enc = b.g.device.create_command_encoder(&Default::default());
    {
        let mut p = enc.begin_compute_pass(&Default::default());
        p.set_pipeline(if numbers { &b.pipe_check } else { &b.pipe_draw });
        p.set_bind_group(0, if numbers { &b.bind_check } else { &b.bind_draw }, &[]);
        p.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
    }
    b.g.queue.submit(Some(enc.finish()));
}
fn wait(b: &Band) { b.g.device.poll(wgpu::PollType::wait_indefinitely()).unwrap(); }
const SETTINGS: [(&str, f32); 6] = [("the way in only (no glass walked)", -1.0), ("in + first surface (0 mirrorings)", 0.0), ("up to 3 mirrorings", 3.0), ("up to 5", 5.0), ("up to 8", 8.0), ("up to 32", 32.0)];
fn three(v: &mut Vec<f64>) -> String { v.sort_by(|a, b| a.total_cmp(b)); format!("{:5.2} / {:5.2} / {:5.2}", v[0], v[v.len() / 2], v[v.len() - 1]) }

/// Time of the ray pass alone: `frames` passes handed over one after another, the last one waited for; whole time /
/// frames. (A GPU that is waited for after every pass dozes off in between, and its own timestamps came back as
/// nonsense on Metal, so neither is used.) Rounds in turn, so a busy GPU (other windows) hits every setting alike.
/// The second column writes the check's numbers (two vec4 a pixel into a buffer) instead of the picture: the pass
/// whose answers `check` compared with the CPU, so the time is of work that was really done.
fn time(m: &Mesh, frames: usize) {
    let b = band_open(m);
    let rounds = 5;
    let mut ms = vec![[Vec::new(), Vec::new()]; SETTINGS.len()];
    for _ in 0..rounds {
        for (k, (_, most)) in SETTINGS.iter().enumerate() {
            set(&b, *most);
            for numbers in [false, true] {
                for _ in 0..20 { pass(&b, numbers); }   // (wake the GPU up)
                wait(&b);
                let t0 = Instant::now();
                for _ in 0..frames { pass(&b, numbers); }
                wait(&b);
                ms[k][numbers as usize].push(t0.elapsed().as_secs_f64() * 1000.0 / frames as f64);
            }
        }
    }
    println!("TIME, the ray pass alone, {W}x{H}, every pixel sends its line of sight; {frames} passes in a row, {rounds} rounds in turn; ms a pass: least / middle / most");
    println!("    {:<40} {:>24} {:>30}", "", "picture (16-bit float RGBA)", "the check's numbers (buffer)");
    for (k, (name, _)) in SETTINGS.iter().enumerate() {
        let [a, c] = &mut ms[k];
        println!("    {:<40} {:>24} {:>30}", name, three(a), three(c));
    }
}

/// One ray pass every 1/60 s, each waited for: how long from handing it over to its end, as a frame would see it.
/// Run it alone, or while something else (Filament's frames) keeps the GPU busy.
fn pace(m: &Mesh, frames: usize) {
    let b = band_open(m);
    let per: usize = std::env::var("PER").ok().and_then(|s| s.parse().ok()).unwrap_or(1);   // (passes handed over each 1/60 s)
    let spin = std::env::var("SPIN").is_ok();
    let hz: f64 = std::env::var("HZ").ok().and_then(|s| s.parse().ok()).unwrap_or(60.0);   // (how often; 60 unless said)
    println!("    ({per} pass(es) each time, {hz} times a second; waiting by {})", if spin { "asking again and again" } else { "wgpu's own wait" });
    println!("PACE, one ray pass every 1/60 s, {frames} of them, each waited for; ms from handing over to done: least / middle / most; late = longer than 16.67 ms");
    for (name, most) in [SETTINGS[2], SETTINGS[3], SETTINGS[4]] {
        set(&b, most);
        let mut ms = Vec::new();
        let start = Instant::now();
        for f in 0..frames + 30 {
            let due = std::time::Duration::from_secs_f64(f as f64 / hz);
            if let Some(left) = due.checked_sub(start.elapsed()) { std::thread::sleep(left); }
            let t0 = Instant::now();
            for _ in 0..per { pass(&b, false); }
            if spin {
                // (asking again and again instead of wgpu's own wait, which sleeps 1 ms at a time on Metal)
                while !b.g.device.poll(wgpu::PollType::Poll).unwrap().is_queue_empty() { std::hint::spin_loop(); }
            } else {
                wait(&b);
            }
            if f >= 30 { ms.push(t0.elapsed().as_secs_f64() * 1000.0); }
        }
        let late = ms.iter().filter(|&&t| t > 1000.0 / 60.0).count();
        let mean = ms.iter().sum::<f64>() / ms.len() as f64;
        println!("    {:<24} {}   mean {:5.2}   late {} of {}", name, three(&mut ms), mean, late, frames);
    }
}

/// Filament's skybox file of the studio (KTX 1, a cube of R11G11B10 floats, one level), handed to the GPU as it is.
pub fn studio_cube(g: &Gpu, studio_path: &str) -> wgpu::Texture {
    let raw = std::fs::read(studio_path).expect("studio file");
    let word = |at: usize| u32::from_le_bytes(raw[at..at + 4].try_into().unwrap());
    assert!(&raw[1..7] == b"KTX 11" && word(28) == 0x8C3A && word(52) == 6 && word(56) == 1, "not a one-level cube of R11G11B10 floats");
    let (side, from) = (word(36), 64 + word(60) as usize);
    let face = word(from) as usize;
    g.device.create_texture_with_data(
        &g.queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: side, height: side, depth_or_array_layers: 6 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg11b10Ufloat,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &raw[from + 4..from + 4 + face * 6],
    )
}
/// Waits for what was handed to the GPU, but not for ever: wgpu's Metal wait only looks for "completed", so work the
/// GPU gave up on (an error, a time limit) would be waited for without end.
pub fn wait_some(g: &Gpu) {
    let limit = std::time::Duration::from_secs(20);
    match g.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(limit) }) {
        Ok(status) if !status.is_queue_empty() => { println!("GAVE UP: the GPU did not finish within 20 s"); std::process::exit(2); }
        Err(e) => { println!("GAVE UP: {e:?}"); std::process::exit(2); }
        _ => {}
    }
}
/// An RGBA8 texture read back (tests and still pictures only).
pub fn pixels_of(g: &Gpu, t: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let row = (w * 4).div_ceil(256) * 256;   // (a row in a copy must be a multiple of 256 bytes)
    let back = g.device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = g.device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        t.as_image_copy(),
        wgpu::TexelCopyBufferInfo { buffer: &back, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    g.queue.submit(Some(enc.finish()));
    back.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
    wait_some(g);
    let padded = back.slice(..).get_mapped_range();
    padded.chunks(row as usize).flat_map(|r| r[..(w * 4) as usize].iter().copied()).collect()
}
pub fn save_png(path: &str, w: u32, h: u32, pixels: &[u8]) {
    let mut e = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).expect("picture file")), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(pixels).unwrap();
    println!("wrote {path}");
}

/// The lines as a picture (see band.wgsl, paint). studio: Filament's skybox file of the studio (KTX 1, a cube of
/// R11G11B10 floats, one level), handed to the GPU as it is.
fn picture(m: &Mesh, studio_path: &str, out_path: &str) {
    let b = band_open(m);
    let cube = studio_cube(&b.g, studio_path);
    let cube_view = cube.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::Cube), ..Default::default() });
    let how = b.g.device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
    let painted = b.g.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let pipeline = b.g.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &b.module, entry_point: Some("paint"), compilation_options: Default::default(), cache: None });
    let bind = b.g.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: b.tlas.as_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: b.params.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: b.pos.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: b.idx.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: b.nrm.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&cube_view) },
            wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::Sampler(&how) },
            wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::TextureView(&painted.create_view(&Default::default())) },
        ],
    });
    let env = |name: &str, d: f32| std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(d);
    // (the brightness band.mm gives the studio: 60000 at f/16, 1/125 s, ISO 100; the turn: the scene's light is turned 2.2 about the up axis)
    set_look(&b, env("MOST", 8.0), [60000.0 / (1.2 * 256.0 * 125.0), env("TURN", -2.2), env("SMOOTH", 0.0), 0.0]);
    let mut enc = b.g.device.create_command_encoder(&Default::default());
    {
        let mut p = enc.begin_compute_pass(&Default::default());
        p.set_pipeline(&pipeline);
        p.set_bind_group(0, &bind, &[]);
        p.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
    }
    b.g.queue.submit(Some(enc.finish()));
    save_png(out_path, W, H, &pixels_of(&b.g, &painted, W, H));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("one") => one(),
        Some("shapes") => shapes::run(args.get(2).expect("studio .ktx"), args.get(3).expect("picture .png")),
        Some("band") => {
            let m = load(args.get(2).expect("mesh file"));
            match args.get(3).map(String::as_str) {
                Some("time") => time(&m, args.get(4).and_then(|s| s.parse().ok()).unwrap_or(200)),
                Some("picture") => picture(&m, args.get(4).expect("studio .ktx"), args.get(5).expect("picture .png")),
                Some("pace") => pace(&m, args.get(4).and_then(|s| s.parse().ok()).unwrap_or(600)),
                _ => check(&m, args.get(4).and_then(|s| s.parse().ok()).unwrap_or(8)),
            }
        }
        _ => println!("rayq one | rayq band <mesh.bin> [check [every] | time [frames] | pace [frames] | picture <studio.ktx> <out.png>]"),
    }
}
