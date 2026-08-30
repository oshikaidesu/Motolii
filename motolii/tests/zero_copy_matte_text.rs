
use motolii::render::compositor::HeadlessGpu;
use motolii::render::engine::Engine;
use motolii::doc::store::{
    Composition, ContentKeyframe, ContentTrack, Document, Fps, FontRef, Intent, LayerAttrsPatch,
    LayerId, LayerMeta, LayerSource, LayerTiming, Matte, MatteMode, RationalTime,
    TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId,
};

const W: u32 = 64;
const H: u32 = 64;
const HIRAGINO: &str = "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc";

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn place_solid(doc: &mut Document, layer: LayerId, rgba: [u8; 4], order: i16) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba,
                width: W,
                height: H,
            },
            order,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn place_text_layer(doc: &mut Document, layer: LayerId) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Text,
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn style(font_path: &str, family: &str, size: f32, fill: [f64; 4]) -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: FontRef {
            path: font_path.to_owned(),
            fingerprint: None,
            family: family.to_owned(),
            style: "Regular".to_owned(),
        },
        size,
        fill,
        line_height: None,
        tracking: 0.0,
        stroke_color: None,
        stroke_width: 0.0,
        stroke_over_fill: false,
        axes: Vec::new(),
        features: Vec::new(),
    }
}

fn document_with(content: &str, style_row: TextDocumentStyle) -> TextDocument {
    let mut content_track = ContentTrack::new();
    content_track.insert(ContentKeyframe {
        t: t(0),
        content: content.to_owned(),
    });
    TextDocument {
        content: content_track,
        justify: TextJustify::Left,
        wrap_size: None,
        styles: vec![style_row],
        slot_id: None,
        ranges: Vec::new(),
        alignment: TextAlignmentOptions::default(),
        runs: Vec::new(),
    }
}

fn gpu_engine() -> (Engine, wgpu::Device, wgpu::Queue) {
    let HeadlessGpu {
        adapter,
        device,
        queue,
    } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let engine =
        Engine::with_device(device.clone(), queue.clone()).expect("Engine::with_device");
    (engine, device, queue)
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
        label: Some("zero_copy_matte_text test blit bind group layout"),
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
        label: Some("zero_copy_matte_text test blit bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zero_copy_matte_text test blit pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zero_copy_matte_text test blit shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BLIT_WGSL)),
    });
    let readable = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zero_copy_matte_text test readable target"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
    });
    let readable_view = readable.create_view(&wgpu::TextureViewDescriptor::default());
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zero_copy_matte_text test blit pipeline"),
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
        label: Some("zero_copy_matte_text test blit encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zero_copy_matte_text test blit pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &readable_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
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
        label: Some("zero_copy_matte_text test readback buffer"),
        size: (bytes_per_row as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &readable,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    rx.recv().expect("map_async callback never fired").expect("map failed");

    let data = slice.get_mapped_range();
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for row in 0..h as usize {
        let src_start = row * bytes_per_row as usize;
        let dst_start = row * (w as usize) * 4;
        out[dst_start..dst_start + (w as usize) * 4]
            .copy_from_slice(&data[src_start..src_start + (w as usize) * 4]);
    }
    drop(data);
    buffer.unmap();
    out
}

fn max_abs_diff(a: &[u8], b: &[u8]) -> u8 {
    assert_eq!(a.len(), b.len(), "比較対象のバッファ長が違う");
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

#[test]
fn matte_zero_copy_matches_cpu_export_within_tolerance() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place_solid(&mut doc, base, [0, 0, 255, 128], 0);
    place_solid(&mut doc, top, [255, 0, 0, 255], 1);
    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: base,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let mut cpu_engine = Engine::new().expect("headless engine");
    let cpu_frame = cpu_engine
        .render_frame(&doc.view(), t(0))
        .expect("CPU export 経路(render_frame)が matte を描けるはず");

    let (mut gpu_engine, device, queue) = gpu_engine();
    let (_texture, view) = gpu_engine
        .render_frame_to_texture(&doc.view(), t(0))
        .expect("zero-copy 経路(render_frame_to_texture)が matte を描けるはず");
    let gpu_frame = blit_and_readback(&device, &queue, &view, W, H);

    let diff = max_abs_diff(&cpu_frame, &gpu_frame);
    assert!(
        diff <= 2,
        "Preview(zero-copy)と Export(CPU)で matte 適用後の絵が食い違う\
         (最大チャンネル差={diff}、許容は ±2)"
    );

    let center = ((H / 2 * W + W / 2) * 4) as usize;
    assert!(
        cpu_frame[center] < 250,
        "matte が効いていない疑い(Alpha matte で赤の alpha が縮み\
         黒背景が透けるはず): {:?}",
        &cpu_frame[center..center + 4]
    );
}

#[test]
fn all_four_matte_modes_render_through_zero_copy_path() {
    for mode in [
        MatteMode::Alpha,
        MatteMode::InvertedAlpha,
        MatteMode::Luma,
        MatteMode::InvertedLuma,
    ] {
        let mut doc = doc_with_comp();
        let (base, top) = (LayerId(1), LayerId(2));
        place_solid(&mut doc, base, [200, 200, 200, 200], 0);
        place_solid(&mut doc, top, [255, 0, 0, 255], 1);
        doc.apply(Intent::SetAttrs {
            layer: top,
            patch: LayerAttrsPatch {
                matte: Some(Some(Matte { layer: base, mode })),
                ..Default::default()
            },
        })
        .unwrap();

        let (mut engine, _device, _queue) = gpu_engine();
        let result = engine.render_frame_to_texture(&doc.view(), t(0));
        assert!(
            result.is_ok(),
            "{mode:?} は zero-copy 経路でも拒まれてはいけない: {result:?}"
        );
    }
}

#[test]
fn japanese_text_zero_copy_matches_cpu_export_and_renders_visible_pixels() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_text_layer(&mut doc, layer);
    doc.apply(Intent::SetTextDocument {
        layer,
        document: document_with(
            "文字が画素になる",
            style(HIRAGINO, "Hiragino Sans", 24.0, [0.0, 0.0, 1.0, 1.0]),
        ),
    })
    .unwrap();

    let mut cpu_engine = Engine::new().expect("headless engine");
    let cpu_frame = cpu_engine
        .render_frame(&doc.view(), t(0))
        .expect("CPU export 経路(render_frame)が text layer を描けるはず");

    let (mut gpu_engine, device, queue) = gpu_engine();
    let (_texture, view) = gpu_engine
        .render_frame_to_texture(&doc.view(), t(0))
        .expect("zero-copy 経路(render_frame_to_texture)が text layer を描けるはず");
    let gpu_frame = blit_and_readback(&device, &queue, &view, W, H);

    let colored = |frame: &[u8]| {
        frame
            .chunks_exact(4)
            .filter(|p| p[0] > 40 || p[2] > 40)
            .count()
    };
    assert!(
        colored(&cpu_frame) > 50,
        "CPU 経路で日本語テキストの画素が出ていない(試験の前提が崩れている)"
    );
    assert!(
        colored(&gpu_frame) > 50,
        "zero-copy 経路で日本語テキストの画素が出ていない\
         (matte とテキストの結線がまだゼロ-copy 経路に届いていない疑い)"
    );

    let diff = max_abs_diff(&cpu_frame, &gpu_frame);
    assert!(
        diff <= 2,
        "Preview(zero-copy)と Export(CPU)で日本語テキストの絵が食い違う\
         (最大チャンネル差={diff}、許容は ±2)"
    );
}
