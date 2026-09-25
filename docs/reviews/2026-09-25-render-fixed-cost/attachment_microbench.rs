use std::time::Instant;
const W: u32 = 1920;
const H: u32 = 1080;

const SHADER: &str = r#"
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = array<vec2f,3>(vec2f(-1.0,-1.0), vec2f(3.0,-1.0), vec2f(-1.0,3.0));
    return vec4f(p[i], 0.5, 1.0);
}
@fragment fn fs() -> @location(0) vec4f { return vec4f(0.2, 0.1, 0.05, 0.5); }
@fragment fn fs_sample(@builtin(sample_index) s: u32) -> @location(0) vec4f { return vec4f(0.2, 0.1, 0.05, 0.5) * (1.0 + f32(s) * 0.0); }
"#;

struct Case { name: &'static str, samples: u32, color: wgpu::TextureFormat, depth: Option<wgpu::TextureFormat>, store_msaa: bool, blend: bool, sample_rate: bool, resolve_fmt: Option<wgpu::TextureFormat>, draw: bool, transient: bool }

fn main() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
    println!("{:?}", adapter.get_info().name);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
    use wgpu::TextureFormat as F;
    let cases = [
        Case { name: "A motolii: msaa4 rgba16f + d32, resolve 16f, blend", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: false, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: false },
        Case { name: "B no draw (clear+resolve only)", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: false, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: false, transient: false },
        Case { name: "C no depth", samples: 4, color: F::Rgba16Float, depth: None, store_msaa: false, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: false },
        Case { name: "D no blend", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: false, blend: false, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: false },
        Case { name: "E msaa4 rgba8", samples: 4, color: F::Rgba8Unorm, depth: Some(F::Depth32Float), store_msaa: false, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba8Unorm), draw: true, transient: false },
        Case { name: "F msaa4 rgba16f resolve to rgba8", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: false, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: true },
        Case { name: "G no msaa rgba16f + d32", samples: 1, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: false, blend: true, sample_rate: false, resolve_fmt: None, draw: true, transient: false },
        Case { name: "H no msaa rgba16f no depth", samples: 1, color: F::Rgba16Float, depth: None, store_msaa: false, blend: true, sample_rate: false, resolve_fmt: None, draw: true, transient: false },
        Case { name: "I no msaa rgba8 no depth", samples: 1, color: F::Rgba8Unorm, depth: None, store_msaa: false, blend: true, sample_rate: false, resolve_fmt: None, draw: true, transient: false },
        Case { name: "J msaa4 fp16 store msaa too", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: true, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: false },
        Case { name: "K msaa4 fp16 sample-rate shading", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth32Float), store_msaa: false, blend: true, sample_rate: true, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: false },
        Case { name: "L msaa4 fp16 depth16", samples: 4, color: F::Rgba16Float, depth: Some(F::Depth16Unorm), store_msaa: false, blend: true, sample_rate: false, resolve_fmt: Some(F::Rgba16Float), draw: true, transient: false },
    ];
    for c in cases {
        let size = wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 };
        let tex = |fmt, samples, extra: wgpu::TextureUsages| device.create_texture(&wgpu::TextureDescriptor { label: None, size, mip_level_count: 1, sample_count: samples, dimension: wgpu::TextureDimension::D2, format: fmt, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | extra, view_formats: &[] });
        let _ = c.transient;
        let color = tex(c.color, c.samples, wgpu::TextureUsages::empty());
        let resolve = c.resolve_fmt.map(|f| tex(f, 1, wgpu::TextureUsages::TEXTURE_BINDING));
        let depth = c.depth.map(|f| tex(f, c.samples, wgpu::TextureUsages::empty()));
        let cv = color.create_view(&Default::default());
        let rv = resolve.as_ref().map(|t| t.create_view(&Default::default()));
        let dv = depth.as_ref().map(|t| t.create_view(&Default::default()));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[], immediate_size: 0 });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None, layout: Some(&layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: Default::default(),
            depth_stencil: c.depth.map(|f| wgpu::DepthStencilState { format: f, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Greater), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState { count: c.samples, mask: !0, alpha_to_coverage_enabled: false },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some(if c.sample_rate { "fs_sample" } else { "fs" }), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState { format: c.color, blend: c.blend.then_some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })] }),
            multiview_mask: None, cache: None,
        });
        let run = |n: u32| {
            let mut enc = device.create_command_encoder(&Default::default());
            for _ in 0..n {
                let (view, resolve_target) = if c.samples > 1 { (&cv, rv.as_ref()) } else { (&cv, None) };
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, depth_slice: None, resolve_target, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: if c.samples > 1 && !c.store_msaa { wgpu::StoreOp::Discard } else { wgpu::StoreOp::Store } } })],
                    depth_stencil_attachment: dv.as_ref().map(|v| wgpu::RenderPassDepthStencilAttachment { view: v, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Discard }), stencil_ops: None }),
                    timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
                });
                if c.draw { pass.set_pipeline(&pipeline); pass.draw(0..3, 0..1); }
            }
            queue.submit([enc.finish()]);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        };
        run(10);
        let mut best = f64::MAX;
        for _ in 0..5 { let t = Instant::now(); run(100); best = best.min(t.elapsed().as_secs_f64() * 1e3 / 100.0); }
        println!("{:8.3} ms  {}", best, c.name);
    }
}
