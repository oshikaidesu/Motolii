//! usage: wgsl-fx <effect.wgsl> <in.png> <out.png> [name=value[,value...]]...
//! The effect file is the contract: a fragment function `fs_main`, the input at @group(0) @binding(0/1) (texture, sampler),
//! and an optional `var<uniform>` struct at @group(1) @binding(0) whose member names are the parameters.

use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: wgsl-fx <effect.wgsl> <in.png> <out.png> [name=value]...");
        std::process::exit(2);
    }
    let source = std::fs::read_to_string(&args[0]).expect("read effect");
    let given: HashMap<String, Vec<f32>> = args[3..]
        .iter()
        .filter_map(|a| a.split_once('='))
        .map(|(k, v)| (k.to_string(), v.split(',').filter_map(|x| x.parse().ok()).collect()))
        .collect();

    // Parameters: the members of the group(1) uniform struct, by name. Nothing else is a parameter.
    let module = naga::front::wgsl::parse_str(&source).unwrap_or_else(|e| {
        eprintln!("{}", e.emit_to_string(&source));
        std::process::exit(1);
    });
    let mut uniform_bytes: Vec<u8> = Vec::new();
    let mut has_uniform = false;
    for (_, var) in module.global_variables.iter() {
        let is_params = matches!(var.space, naga::AddressSpace::Uniform)
            && var.binding.as_ref().map_or(false, |b| b.group == 1 && b.binding == 0);
        if !is_params {
            continue;
        }
        has_uniform = true;
        if let naga::TypeInner::Struct { members, span } = &module.types[var.ty].inner {
            uniform_bytes = vec![0u8; ((*span as usize) + 15) & !15];
            for m in members {
                let name = m.name.clone().unwrap_or_default();
                let width = match &module.types[m.ty].inner {
                    naga::TypeInner::Scalar(_) => 1,
                    naga::TypeInner::Vector { size, .. } => *size as usize,
                    _ => 0,
                };
                if let Some(values) = given.get(&name) {
                    for (i, v) in values.iter().take(width).enumerate() {
                        let at = m.offset as usize + i * 4;
                        uniform_bytes[at..at + 4].copy_from_slice(&v.to_le_bytes());
                    }
                } else {
                    eprintln!("note: parameter `{name}` not given, left 0");
                }
            }
        }
    }

    let input = image::open(&args[1]).expect("read input").to_rgba8();
    let (w, h) = input.dimensions();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).expect("adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("device");

    let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
    let src = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("in"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        src.as_image_copy(),
        &input,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
        size,
    );
    let dst = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("out"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let vs = "@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {\n  var p = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));\n  return vec4<f32>(p[i], 0.0, 1.0);\n}\n";
    let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("vs"), source: wgpu::ShaderSource::Wgsl(vs.into()) });
    let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("effect"), source: wgpu::ShaderSource::Wgsl(source.clone().into()) });

    let tex_entry = |binding, ty| wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::FRAGMENT, ty, count: None };
    let bgl0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            tex_entry(0, wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }),
            tex_entry(1, wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)),
        ],
    });
    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[tex_entry(0, wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None })],
    });
    let layouts: Vec<Option<&wgpu::BindGroupLayout>> = if has_uniform { vec![Some(&bgl0), Some(&bgl1)] } else { vec![Some(&bgl0)] };
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &layouts, immediate_size: 0 });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fx"),
        layout: Some(&layout),
        vertex: wgpu::VertexState { module: &vs_module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba8UnormSrgb, blend: None, write_mask: wgpu::ColorWrites::ALL })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });
    let src_view = src.create_view(&Default::default());
    let group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl0,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&src_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }],
    });
    let ubuf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: uniform_bytes.len().max(16) as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    if has_uniform {
        queue.write_buffer(&ubuf, 0, &uniform_bytes);
    }
    let group1 = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &bgl1, entries: &[wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() }] });

    let dst_view = dst.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &dst_view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group0, &[]);
        if has_uniform {
            pass.set_bind_group(1, &group1, &[]);
        }
        pass.draw(0..3, 0..1);
    }
    let padded = ((4 * w + 255) / 256) * 256;
    let readback = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (padded * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    encoder.copy_texture_to_buffer(
        dst.as_image_copy(),
        wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) } },
        size,
    );
    queue.submit([encoder.finish()]);
    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let data = slice.get_mapped_range();
    let mut out = image::RgbaImage::new(w, h);
    for y in 0..h {
        let row = &data[(y * padded) as usize..(y * padded + 4 * w) as usize];
        out.as_mut()[(y * 4 * w) as usize..((y + 1) * 4 * w) as usize].copy_from_slice(row);
    }
    out.save(&args[2]).expect("save");
}
