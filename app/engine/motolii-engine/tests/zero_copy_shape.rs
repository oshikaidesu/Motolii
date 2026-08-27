//! **発注(2026-08-22)「シェイプが画に出るようにする」の RETURN 要件3**: ゼロコピー
//! 経路(`Engine::render_frame_to_texture` → `layers_from_resolved`)にも
//! shape を通す——`tests/zero_copy_matte_text.rs` が text/matte で固定したのと
//! 同じ形の oracle を shape に対して繰り返す(helper 関数は意図的に複製——
//! `zero_copy_matte_text.rs`/`motolii-compositor/tests/zero_copy.rs` もそれぞれ
//! 自前で持っており、この crate に共有 test-support module は無い)。
//!
//! ここで縛るのは2点:
//! 1. **Preview(zero-copy GPU 経路)= Export(CPU readback 経路)**——同じ
//!    Document/時刻から `render_frame`(CPU)と `render_frame_to_texture`(GPU)を
//!    両方呼び、ピクセルへ落として比較する(許容 ±2/channel、
//!    `zero_copy_matte_text.rs` と同じ理由の許容——shape も GPU の
//!    テクスチャサンプリング/合成パスを経由する)。
//! 2. zero-copy 経路で実際に shape の画素が出ること(`render_resolved_to_texture`
//!    (shell が直接呼ぶ後方互換ラッパー、空の `shape_documents` 固定)ではなく
//!    `render_frame_to_texture`/`render_resolved_to_texture_with_shapes` が
//!    `shape_documents` を実際に運ぶことの確認)。

use motolii_compositor::HeadlessGpu;
use motolii_engine::Engine;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, RationalTime,
    Shape, ShapeNode,
};
use motolii_vector::{Brush, Fill, FillRule, PathSource, Point as VPoint, Rgb};

const W: u32 = 64;
const H: u32 = 64;

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

fn place_shape_layer(doc: &mut Document, layer: LayerId) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Shape,
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

fn red_rect(size: f64) -> Shape {
    Shape {
        source: PathSource::Rectangle {
            size: VPoint { x: size, y: size },
        },
        ops: Vec::new(),
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            }),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }),
        stroke: None,
    }
}

/// `zero_copy_matte_text.rs::gpu_engine` と同型。
fn gpu_engine() -> (Engine, wgpu::Device, wgpu::Queue) {
    let HeadlessGpu {
        adapter,
        device,
        queue,
    } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let engine = Engine::with_device(device.clone(), queue.clone()).expect("Engine::with_device");
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

/// `zero_copy_matte_text.rs::blit_and_readback` と同型(main_target が
/// `COPY_SRC` を持たないので、bind して sample → 自前の `COPY_SRC` texture へ
/// blit してから読み戻す)。
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
        label: Some("zero_copy_shape test blit bind group layout"),
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
        label: Some("zero_copy_shape test blit bind group"),
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
        label: Some("zero_copy_shape test blit pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zero_copy_shape test blit shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BLIT_WGSL)),
    });
    let readable = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zero_copy_shape test readable target"),
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
        view_formats: &[],
    });
    let readable_view = readable.create_view(&wgpu::TextureViewDescriptor::default());
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zero_copy_shape test blit pipeline"),
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
        label: Some("zero_copy_shape test blit encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zero_copy_shape test blit pass"),
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
        label: Some("zero_copy_shape test readback buffer"),
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
    rx.recv()
        .expect("map_async callback never fired")
        .expect("map failed");

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

/// `zero_copy_matte_text.rs::max_abs_diff` と同型。
fn max_abs_diff(a: &[u8], b: &[u8]) -> u8 {
    assert_eq!(a.len(), b.len(), "比較対象のバッファ長が違う");
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

/// **RETURN 要件3(両経路で shape が出る + 一致)**: `tests/shape_layer.rs::
/// rectangle_shape_renders_visible_pixels_through_render_frame` と同じ document を、
/// zero-copy 経路で描いても画素として出ること、かつ CPU 経路と同じ絵になること。
#[test]
fn rectangle_shape_zero_copy_matches_cpu_export_and_renders_visible_pixels() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(40.0))],
    })
    .unwrap();

    let mut cpu_engine = Engine::new().expect("headless engine");
    let cpu_frame = cpu_engine
        .render_frame(&doc.view(), t(0))
        .expect("CPU export 経路(render_frame)が shape layer を描けるはず");

    let (mut gpu_engine, device, queue) = gpu_engine();
    let (_texture, view) = gpu_engine
        .render_frame_to_texture(&doc.view(), t(0))
        .expect("zero-copy 経路(render_frame_to_texture)が shape layer を描けるはず");
    let gpu_frame = blit_and_readback(&device, &queue, &view, W, H);

    let colored = |frame: &[u8]| {
        frame
            .chunks_exact(4)
            .filter(|p| p[0] > 40 && p[1] < 20 && p[2] < 20)
            .count()
    };
    assert!(
        colored(&cpu_frame) > 100,
        "CPU 経路で矩形シェイプの画素が出ていない(試験の前提が崩れている)"
    );
    assert!(
        colored(&gpu_frame) > 100,
        "zero-copy 経路で矩形シェイプの画素が出ていない\
         (shape の結線がまだゼロ-copy 経路に届いていない疑い)"
    );

    let diff = max_abs_diff(&cpu_frame, &gpu_frame);
    assert!(
        diff <= 2,
        "Preview(zero-copy)と Export(CPU)で矩形シェイプの絵が食い違う\
         (最大チャンネル差={diff}、許容は ±2)"
    );
}

/// zero-copy 経路の後方互換ラッパー(`render_resolved_to_texture`、shell が直接
/// 呼ぶシグネチャ固定)は shape_documents を持たない——空のままでも `Err` には
/// ならず、単に shape を描かないだけであることを確かめる(shell を触らずに
/// 後方互換性を保っていることの直接固定、`lib.rs` の
/// `Engine::render_resolved_to_texture` doc 参照)。
#[test]
fn backward_compatible_wrapper_renders_nothing_for_shapes_but_does_not_error() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place_shape_layer(&mut doc, layer);
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(red_rect(40.0))],
    })
    .unwrap();

    let view = doc.view();
    let composition = view.composition().unwrap().unwrap();
    let comp = composition.spec();
    let camera = view.resolve_camera(t(0)).unwrap();
    let resolved = view.resolved_layers(t(0)).unwrap();

    let (mut engine, device, queue) = gpu_engine();
    let (_texture, gpu_view) = engine
        .render_resolved_to_texture(
            comp,
            composition.background,
            camera,
            t(0),
            &resolved,
            &std::collections::HashMap::new(),
        )
        .expect("後方互換ラッパーは shape があっても Err にならないはず");
    let gpu_frame = blit_and_readback(&device, &queue, &gpu_view, W, H);
    let colored = gpu_frame
        .chunks_exact(4)
        .filter(|p| p[0] > 40 && p[1] < 20 && p[2] < 20)
        .count();
    assert_eq!(
        colored, 0,
        "shape_documents を渡さない後方互換ラッパーは shape を描かないはず\
         (shell 未改修のままの既存呼び出しと同じ挙動)"
    );
}
