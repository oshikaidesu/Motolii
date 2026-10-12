// rayq shapes: a torus and a box with round edges, in glass, by the GPU's own ray query (glass.wgsl).
// Settings by environment: MODEL 0|1|2 (see glass.wgsl), STEPS wavelengths (model 2; 16), IOR (1.5, at 589 nm),
// ABBE (how little the index changes with the wavelength; real glass 25..65, smaller = stronger colours; 30),
// LINES 1|4 a pixel (4), MOST surfaces met by one wavelength of one line (12), SMALL share not followed further (0.02),
// TURN of the studio (-2.2), WHITE=1 the studio white everywhere + no film curve (nothing may come out but white:
// a test that no light is made or lost), FRAMES n: time of n passes in a row at one line a pixel.
use crate::{open, pixels_of, save_png, structure, studio_cube, wait_some};
use std::time::Instant;

const W: u32 = 1920;
const H: u32 = 1080;

type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn mul(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn norm(a: V3) -> V3 { mul(a, 1.0 / dot(a, a).sqrt()) }
/// Turned about x by `ax`, then about y by `ay` (radians).
fn turned(v: V3, ax: f32, ay: f32) -> V3 {
    let (s, c) = ax.sin_cos();
    let v = [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c];
    let (s, c) = ay.sin_cos();
    [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
}

#[derive(Default)]
struct Mesh { pos: Vec<f32>, idx: Vec<u32>, nrm: Vec<f32> }
impl Mesh {
    /// A grid of points (and their smooth normals) as triangles; `wrap`: the grid closes on itself both ways.
    fn grid(&mut self, nu: usize, nv: usize, wrap: bool, at: impl Fn(usize, usize) -> (V3, V3), place: V3, ax: f32, ay: f32) {
        let base = (self.pos.len() / 3) as u32;
        let (cu, cv) = if wrap { (nu, nv) } else { (nu + 1, nv + 1) };
        for u in 0..cu {
            for v in 0..cv {
                let (p, n) = at(u, v);
                self.pos.extend(add(turned(p, ax, ay), place));
                self.nrm.extend(turned(n, ax, ay));
            }
        }
        for u in 0..nu {
            for v in 0..nv {
                let i = |a: usize, b: usize| base + ((a % cu) * cv + (b % cv)) as u32;
                self.idx.extend([i(u, v), i(u + 1, v), i(u + 1, v + 1), i(u, v), i(u + 1, v + 1), i(u, v + 1)]);
            }
        }
    }
    fn torus(&mut self, big: f32, small: f32, place: V3, ax: f32, ay: f32) {
        let (nu, nv) = (320, 160);
        self.grid(nu, nv, true, |u, v| {
            let (a, b) = (u as f32 / nu as f32 * std::f32::consts::TAU, v as f32 / nv as f32 * std::f32::consts::TAU);
            let n = [a.cos() * b.cos(), b.sin(), a.sin() * b.cos()];
            (add([a.cos() * big, 0.0, a.sin() * big], mul(n, small)), n)
        }, place, ax, ay);
    }
    /// A box of half sizes `half` whose edges and corners are round with radius `round`.
    fn round_box(&mut self, half: V3, round: f32, place: V3, ax: f32, ay: f32) {
        let n = 96;
        for face in 0..6 {
            let (axis, sign) = (face / 2, if face % 2 == 0 { 1.0 } else { -1.0 });
            let (a1, a2) = ((axis + 1) % 3, (axis + 2) % 3);
            self.grid(n, n, false, |u, v| {
                let mut c = [0.0f32; 3];
                c[axis] = sign * half[axis];
                c[a1] = half[a1] * (2.0 * u as f32 / n as f32 - 1.0);
                c[a2] = half[a2] * (2.0 * v as f32 / n as f32 - 1.0);
                let inner = [c[0].clamp(-(half[0] - round), half[0] - round), c[1].clamp(-(half[1] - round), half[1] - round), c[2].clamp(-(half[2] - round), half[2] - round)];
                let out = norm(sub(c, inner));
                (add(inner, mul(out, round)), out)
            }, place, ax, ay);
        }
    }
}

/// CIE 1931 colour matching functions by the fit of Wyman, Sloan and Shirley (2013), then linear sRGB.
fn rgb_of(nm: f32) -> V3 {
    let g = |mu: f32, s1: f32, s2: f32| { let t = (nm - mu) / if nm < mu { s1 } else { s2 }; (-0.5 * t * t).exp() };
    let x = 1.056 * g(599.8, 37.9, 31.0) + 0.362 * g(442.0, 16.0, 26.7) - 0.065 * g(501.1, 20.4, 26.2);
    let y = 0.821 * g(568.8, 46.9, 40.5) + 0.286 * g(530.9, 16.3, 31.1);
    let z = 1.217 * g(437.0, 11.8, 36.0) + 0.681 * g(459.0, 26.0, 13.8);
    [3.2406 * x - 1.5372 * y - 0.4986 * z, -0.9689 * x + 1.8758 * y + 0.0415 * z, 0.0557 * x - 0.2040 * y + 1.0570 * z]
}
/// The index of refraction at a wavelength (Cauchy's two-term law through the index at 589.3 nm and the Abbe number).
fn index_at(nm: f32, at_589: f32, abbe: f32) -> f32 {
    let inv = |l: f32| 1.0 / (l * l);
    at_589 + (at_589 - 1.0) / abbe * (inv(nm) - inv(589.3)) / (inv(486.1) - inv(656.3))
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params { eye: [f32; 4], fwd: [f32; 4], right: [f32; 4], up: [f32; 4], size: [u32; 2], steps: u32, model: u32, look: [f32; 4], opts: [f32; 4], wave: [[f32; 4]; 32] }

pub fn run(studio_path: &str, out_path: &str) {
    let env = |name: &str, d: f32| std::env::var(name).ok().and_then(|s| s.parse().ok()).unwrap_or(d);
    let mut m = Mesh::default();
    let only = std::env::var("ONLY").unwrap_or_default();   // (tests: "torus" or "box" alone)
    if only != "box" { m.torus(1.0, 0.42, [-1.75, 0.0, 0.0], 0.95, 0.5); }
    if only != "torus" { m.round_box([0.85, 0.85, 0.85], 0.22, [1.75, 0.0, 0.0], 0.45, 0.65); }
    let bad = m.pos.iter().chain(m.nrm.iter()).filter(|v| !v.is_finite()).count();
    let far = m.idx.iter().filter(|&&i| i as usize >= m.pos.len() / 3).count();
    let flat = m.idx.chunks_exact(3).filter(|t| { let p = |i: u32| [m.pos[i as usize * 3], m.pos[i as usize * 3 + 1], m.pos[i as usize * 3 + 2]]; let c = cross(sub(p(t[1]), p(t[0])), sub(p(t[2]), p(t[0]))); dot(c, c) == 0.0 }).count();
    println!("mesh: {} numbers that are not numbers, {} corners out of range, {} triangles with no area", bad, far, flat);
    if let Ok(path) = std::env::var("DUMP") {   // (tests: the mesh as a file `rayq band` reads)
        let mut b: Vec<u8> = Vec::new();
        b.extend(((m.pos.len() / 3) as u32).to_le_bytes());
        b.extend((m.idx.len() as u32).to_le_bytes());
        b.extend(bytemuck::cast_slice::<f32, u8>(&m.pos));
        b.extend(bytemuck::cast_slice::<u32, u8>(&m.idx));
        b.extend(bytemuck::cast_slice::<f32, u8>(&m.nrm));
        std::fs::write(&path, b).unwrap();
        println!("wrote {path}");
        return;
    }
    let g = open();
    let (tlas, ms) = structure(&g, &m.pos, &m.idx);
    println!("{} triangles (a torus, a box with round edges); acceleration structure built in {:.1} ms", m.idx.len() / 3, ms);

    let model = env("MODEL", 2.0) as u32;
    let steps = if model == 2 { (env("STEPS", 16.0) as usize).clamp(1, 32) } else { 1 };
    let (ior, abbe) = (env("IOR", 1.5), env("ABBE", 30.0));
    let mut wave = [[0.0f32; 4]; 32];
    let mut sum = [0.0f32; 3];
    for i in 0..steps {
        let nm = 400.0 + 300.0 * (i as f32 + 0.5) / steps as f32;
        let c = if steps == 1 { [1.0, 1.0, 1.0] } else { rgb_of(nm) };
        wave[i] = [c[0], c[1], c[2], if steps == 1 { ior } else { index_at(nm, ior, abbe) }];
        for k in 0..3 { sum[k] += c[k]; }
    }
    for w in wave.iter_mut().take(steps) { for k in 0..3 { w[k] /= sum[k]; } }   // (all wavelengths together: white)
    // RGB3=1: Blender's standing recipe instead (three Glass BSDFs added: red, green and blue each traced whole with
    // its own index of refraction). The wavelengths stand for the three primaries of the screen.
    let rgb3 = std::env::var("RGB3").is_ok() && model == 2;
    let steps = if rgb3 { 3 } else { steps };
    if rgb3 {
        for (i, nm) in [611.0f32, 549.0, 465.0].into_iter().enumerate() {
            wave[i] = [0.0, 0.0, 0.0, index_at(nm, ior, abbe)];
            wave[i][i] = 1.0;
        }
        println!("three channels (red, green, blue), index of refraction {:.4} / {:.4} / {:.4}, Abbe number {}", wave[0][3], wave[1][3], wave[2][3], abbe);
    }
    if steps > 1 && !rgb3 { println!("{} wavelengths, 400..700 nm; index of refraction {:.4} (violet end) .. {:.4} (red end), Abbe number {}", steps, wave[0][3], wave[steps - 1][3], abbe); }

    let eye: V3 = [0.0, 4.6, 6.6];
    let f = norm(sub([0.0, -0.1, 0.0], eye));
    let r = norm(cross(f, [0.0, 1.0, 0.0]));
    let u = cross(r, f);
    let th = (0.5 * 27.0f32).to_radians().tan();
    let v4 = |a: V3| [a[0], a[1], a[2], 0.0];
    let white = std::env::var("WHITE").is_ok();
    let frames = env("FRAMES", 0.0) as usize;
    let params = Params {
        eye: v4(eye), fwd: v4(f), right: v4(mul(r, th * W as f32 / H as f32)), up: v4(mul(u, th)), size: [W, H], steps: steps as u32, model,
        look: [60000.0 / (1.2 * 256.0 * 125.0), env("TURN", -2.2), if white { 1.0 } else { 0.0 }, if frames > 0 { 1.0 } else { env("LINES", 4.0) }],
        opts: [if white { 1.0 } else { 0.0 }, 0.0, env("SMALL", 0.02), env("MOST", 12.0)],
        wave,
    };

    use wgpu::util::DeviceExt;
    let module = g.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(std::env::var("SHADER").map(|p| std::fs::read_to_string(p).expect("shader file")).unwrap_or(include_str!("glass.wgsl").to_string()).into()) });   // (SHADER: tests)
    let pipeline = g.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some("paint"), compilation_options: Default::default(), cache: None });
    let store = |data: &[u8], usage| g.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: data, usage });
    let (idx, nrm) = (store(bytemuck::cast_slice(&m.idx), wgpu::BufferUsages::STORAGE), store(bytemuck::cast_slice(&m.nrm), wgpu::BufferUsages::STORAGE));
    let pbuf = store(bytemuck::bytes_of(&params), wgpu::BufferUsages::UNIFORM);
    let cube = studio_cube(&g, studio_path);
    let cube_view = cube.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::Cube), ..Default::default() });
    let how = g.device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
    let painted = g.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let bind = g.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: tlas.as_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: pbuf.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: idx.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: nrm.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&cube_view) },
            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&how) },
            wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(&painted.create_view(&Default::default())) },
        ],
    });
    let pass = || {
        let mut enc = g.device.create_command_encoder(&Default::default());
        {
            let mut p = enc.begin_compute_pass(&Default::default());
            p.set_pipeline(&pipeline);
            p.set_bind_group(0, &bind, &[]);
            p.dispatch_workgroups(W.div_ceil(8), H.div_ceil(8), 1);
        }
        g.queue.submit(Some(enc.finish()));
    };
    let wait = || wait_some(&g);
    if frames > 0 {   // (passes in a row, the last one waited for: see main.rs, time)
        let mut ms = Vec::new();
        for _ in 0..5 {
            for _ in 0..10 { pass(); }
            wait();
            let t0 = Instant::now();
            for _ in 0..frames { pass(); }
            wait();
            ms.push(t0.elapsed().as_secs_f64() * 1000.0 / frames as f64);
        }
        ms.sort_by(|a, b| a.total_cmp(b));
        println!("TIME: model {model}, {steps} wavelength(s), {W}x{H}, one line a pixel, {frames} passes in a row x 5: {:.2} / {:.2} / {:.2} ms a pass (least / middle / most)", ms[0], ms[2], ms[4]);
        return;
    }
    pass();
    let px = pixels_of(&g, &painted, W, H);
    if white {
        // nothing but white may come out: how far from white are the glass pixels?
        let glass: Vec<&[u8]> = px.chunks_exact(4).filter(|p| p[3] > 250).collect();
        let off = |p: &&[u8]| (0..3).map(|k| (p[k] as i32 - 255).abs()).max().unwrap();
        let worst = glass.iter().map(off).max().unwrap_or(0);
        let wrong = glass.iter().filter(|p| off(p) > 2).count();
        println!("WHITE test: {} glass pixels; off white by more than 2/255: {} ({:.3}%); worst {} / 255", glass.len(), wrong, 100.0 * wrong as f64 / glass.len().max(1) as f64, worst);
    }
    save_png(out_path, W, H, &px);
}
