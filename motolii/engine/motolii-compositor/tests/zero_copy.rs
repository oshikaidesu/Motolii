
use motolii_compositor::{
    BlendMode, CompSpec, Compositor, EffectPass, HeadlessGpu, Layer, LayerPlacement,
    LayerWithPasses, ResolvedCamera,
};

const W: u32 = 64;
const H: u32 = 64;

fn solid(rgba: [u8; 4], w: u32, h: u32) -> Vec<u8> {
    rgba.iter().copied().cycle().take((w * h * 4) as usize).collect()
}

fn comp() -> CompSpec {
    CompSpec { width: W, height: H }
}

fn one_layer(texture: motolii_compositor::GpuTexture2D) -> Layer {
    Layer {
        texture,
        size: [W as f32, H as f32],
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform([0.0, 0.0], [0.0, 0.0], [1.0, 1.0], 0.0, 0.0, 0.0),
            order: 0,
            opacity: 1.0,
            z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
        },
        pinned: false,
        blend_mode: BlendMode::Normal,
    }
}

fn with_device_compositor() -> (Compositor, wgpu::Device, wgpu::Queue) {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let compositor = Compositor::with_device_using_headless_defaults(device.clone(), queue.clone())
        .expect("with_device");
    (compositor, device, queue)
}

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let uv = corners[vertex_index];
    var out: VertexOutput;
    out.position = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(src_texture, src_sampler, in.uv);
}
"#;

fn blit_and_readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    src_view: &wgpu::TextureView,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zero_copy test blit bind group layout"),
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
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zero_copy test blit bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(src_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zero_copy test blit pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zero_copy test blit shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BLIT_WGSL)),
    });
    let readable = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zero_copy test readable target"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    });
    let readable_view = readable.create_view(&wgpu::TextureViewDescriptor::default());
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zero_copy test blit pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zero_copy test blit encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zero_copy test blit pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &readable_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..6, 0..1);
    }

    let bytes_per_row_unaligned = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bytes_per_row = bytes_per_row_unaligned.div_ceil(align) * align;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zero_copy test readback buffer"),
        size: (bytes_per_row as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &readable, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: Some(h) },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    rx.recv().expect("map_async callback never fired").expect("map failed");

    let data = slice.get_mapped_range();
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for row in 0..h as usize {
        let src_start = row * bytes_per_row as usize;
        let dst_start = row * (w as usize) * 4;
        out[dst_start..dst_start + (w as usize) * 4].copy_from_slice(&data[src_start..src_start + (w as usize) * 4]);
    }
    drop(data);
    buffer.unmap();
    out
}

#[test]
fn render_to_texture_matches_render_with_timing_for_an_opaque_solid_layer() {
    let (mut compositor, device, queue) = with_device_compositor();

    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();
    let (readback_frame, _timing) = compositor
        .render_with_timing(comp(), ResolvedCamera::default(), &[one_layer(red.clone())])
        .unwrap();

    let (texture, view) = compositor
        .render_to_texture(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: one_layer(red),
                passes: vec![],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .unwrap();

    assert_eq!(texture.width(), W, "render_to_texture が返す texture の幅が comp と一致しない");
    assert_eq!(texture.height(), H, "render_to_texture が返す texture の高さが comp と一致しない");

    let readback_gpu = blit_and_readback(&device, &queue, &view, W, H);

    assert_eq!(
        readback_gpu, readback_frame,
        "render_to_texture(readback 無し)の main_target が\
         render_with_timing(readback 有り)と違う絵になっている\
         (不透明単色なので sRGB 端点により tolerance ゼロで一致するはず)"
    );
}

#[test]
fn render_to_texture_is_deterministic_across_repeated_calls() {
    let (mut compositor, device, queue) = with_device_compositor();

    let red = compositor
        .upload_rgba("red", &solid([200, 40, 90, 255], W, H), W, H)
        .unwrap();

    let (_texture_a, view_a) = compositor
        .render_to_texture(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: one_layer(red.clone()),
                passes: vec![],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .unwrap();
    let bytes_a = blit_and_readback(&device, &queue, &view_a, W, H);

    let (_texture_b, view_b) = compositor
        .render_to_texture(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: one_layer(red),
                passes: vec![],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .unwrap();
    let bytes_b = blit_and_readback(&device, &queue, &view_b, W, H);

    assert_eq!(bytes_a, bytes_b, "同じ入力を2回 render_to_texture したのに絵が違う");
    assert!(
        bytes_a.chunks_exact(4).any(|px| px != [0, 0, 0, 0]),
        "読み戻した絵が全ゼロ(blit_and_readback 自体が機能していない疑い)"
    );
}

#[test]
fn render_to_texture_reuses_scratch_texture_across_frames() {
    let (mut compositor, _device, _queue) = with_device_compositor();

    let red = compositor
        .upload_rgba("red", &solid([200, 40, 90, 255], W, H), W, H)
        .unwrap();

    assert_eq!(compositor.effect_passes_created_textures(), 0);

    let _first = compositor
        .render_to_texture(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: one_layer(red.clone()),
                passes: vec![EffectPass::Identity],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .unwrap();
    assert_eq!(
        compositor.effect_passes_created_textures(),
        1,
        "pass を持つ layer は少なくとも1枚のオフスクリーンを新規生成するはず"
    );

    for _ in 0..4 {
        let _next = compositor
            .render_to_texture(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: one_layer(red.clone()),
                    passes: vec![EffectPass::Identity],
                }],
                motolii_compositor::NO_BACKGROUND,
            )
            .unwrap();
    }

    assert_eq!(
        compositor.effect_passes_created_textures(),
        1,
        "render_to_texture 経由でも scratch はプールへ戻って使い回されるはず\
         (RB 調査 発見3番: 毎フレーム新規確保の修理 — 定数のままでなければ red)"
    );
}

#[test]
fn render_to_texture_reused_scratch_shows_the_new_frames_content_not_the_old_one() {
    let (mut compositor, device, queue) = with_device_compositor();

    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();
    let blue = compositor
        .upload_rgba("blue", &solid([0, 0, 255, 255], W, H), W, H)
        .unwrap();

    let (_texture_a, view_a) = compositor
        .render_to_texture(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: one_layer(red),
                passes: vec![EffectPass::Identity],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .unwrap();
    let bytes_a = blit_and_readback(&device, &queue, &view_a, W, H);
    assert_eq!(&bytes_a[0..4], &[255, 0, 0, 255], "1フレーム目は赤のはず");

    let (_texture_b, view_b) = compositor
        .render_to_texture(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: one_layer(blue),
                passes: vec![EffectPass::Identity],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .unwrap();
    let bytes_b = blit_and_readback(&device, &queue, &view_b, W, H);
    assert_eq!(
        &bytes_b[0..4],
        &[0, 0, 255, 255],
        "scratch を使い回した2フレーム目は青のはず\
         (前フレームの赤が透けて見えたら poll なし再利用の同期が壊れている)"
    );

    assert_eq!(
        compositor.effect_passes_created_textures(),
        1,
        "同じ形の scratch は使い回されるはず(このテストの前提)"
    );
}
