//! 裁定171 v2(M4、supervisor 裁定で ALLOWLIST 拡張)——
//! [`Compositor::render_to_texture`]の直接オラクル。
//!
//! `motolii-shell` 側の統合試験(`playback_ticks_do_not_trigger_cpu_readback_
//! after_warmup`)は「(CPU readback が)呼ばれない」ことしか縛れない——ここでは
//! 「呼んだ結果が `render_with_timing`(readback 経路、既存4メソッドの1つ・
//! 無改造)と同じ絵になる」ことを、この crate だけで(shell を経由せず)
//! 直接確かめる。
//!
//! ## main_target を読み戻す手段(`copy_texture_to_buffer` は使えない)
//!
//! fork の `main_target_resolved` は `RENDER_ATTACHMENT | TEXTURE_BINDING`
//! usage で確保される(`crates/viewer/re_renderer/src/view_builder.rs` 実測)
//! ——`COPY_SRC` が無いので `copy_texture_to_buffer` は検証なしにゼロ埋め
//! バッファを返す(実測、最初のドラフトはこれで red 偽陽性を踏んだ)。
//! `motolii-shell` の presenter Pipeline が実際にやること(bind_group へ束ねて
//! sample する、`TEXTURE_BINDING` は main_target が持っている)と**同じ手段**で
//! 読み戻す——自前の Rgba8UnormSrgb+`COPY_SRC` texture へ fullscreen quad で
//! blit してから、その texture を `copy_texture_to_buffer` する。
//!
//! ## 許容誤差ゼロで比較できる理由
//!
//! sRGB 転送関数は 0.0/1.0 の両端点で恒等(`srgb(0)=0`・`srgb(1)=1`)——
//! 純粋な赤(`[255, 0, 0, 255]`、不透明)を敷けば、`main_target` の自動
//! sRGB エンコード(GPU 書き込み時)も `composite.wgsl` の手動
//! `srgb_from_linear`(readback 経路)も同じ端点を通るので、量子化誤差の
//! 余地がない——`render_to_texture`(readback しない)と
//! `render_with_timing`(readback する)の出力がバイト単位で一致するはず、
//! という主張を tolerance なしで書ける。

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
        },
        pinned: false,
        blend_mode: BlendMode::Normal,
    }
}

/// `tests/with_device.rs` の常設 oracle と同じ形(adapter の実物なしで
/// `with_device` が pipeline を建てられることを構造で縛る)。device/queue の
/// clone もついでに返す——`render_to_texture` が返す texture は compositor
/// を建てた device の上にあるので、読み戻しにも**同じ** device/queue が
/// 要る(`wgpu::Device`/`Queue` は clone 可能な薄いハンドル)。
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

/// `render_to_texture` の main_target(`TEXTURE_BINDING` のみ、`COPY_SRC`
/// 無し)を、`motolii-shell` の presenter Pipeline と同じ手段(fullscreen quad
/// で bind して sample する)で、`COPY_SRC` 付きの自前 texture へ blit して
/// から CPU へ読み戻す。sRGB タグ付き texture 同士(main_target も出力先も
/// `Rgba8UnormSrgb`)なので、GPU の自動 decode(sample 時)/encode(書き込み時)
/// が打ち消し合い、中身の意味は変えない(モジュール doc 参照)。
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
    // sRGB タグ付き・`COPY_SRC` 付き — main_target と同じ意味の texture だが、
    // ここへ書き込む GPU は「自動 encode」する(main_target 自体が作られた時と
    // 同じ機序)ので、`textureSample`(自動 decode)→ 素通し → 書き込み(自動
    // encode)で意味は変わらない。
    let readable = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zero_copy test readable target"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
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

/// **裁定171 v2 M4 の中心主張**: `render_to_texture`(readback しない)が
/// `render_with_timing`(readback する、既存4メソッドの1つ・無改造)と
/// バイト単位で同じ絵を返す——不透明な単色なので tolerance 無しで比較できる
/// (モジュール doc 参照)。読み戻しは `motolii-shell` の presenter Pipeline と
/// 同じ手段(sample、`blit_and_readback` doc 参照)を使う。
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

/// **裁定171 v2 M4 EXACT TARGET 2**: 内容が変わらなければ何度呼んでも同じ絵
/// (世代ゲートは呼び出し側 — `motolii-shell` — の責務だが、`render_to_texture`
/// 自身が呼び出しごとに違う絵を返す不安定な関数でないことは、この crate 単体
/// でも縛れる)。
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
        )
        .unwrap();
    let bytes_b = blit_and_readback(&device, &queue, &view_b, W, H);

    assert_eq!(bytes_a, bytes_b, "同じ入力を2回 render_to_texture したのに絵が違う");
    assert!(
        bytes_a.chunks_exact(4).any(|px| px != [0, 0, 0, 0]),
        "読み戻した絵が全ゼロ(blit_and_readback 自体が機能していない疑い)"
    );
}

/// **RB 調査(`docs/reviews/2026-08-22-residual-bottleneck-survey.md` 発見3番)の
/// 直接オラクル(red 先行)**: `render_to_texture` は `effect_scratch.acquire` を
/// 呼ぶが一度も `.release` しない(修理前のモジュール doc が自認)ため、effect を
/// 持つ layer が動くフレームは毎回新規 scratch texture を確保する。S2 の
/// `identity_pass_reuses_the_scratch_texture_across_frames`(`tests/effects.rs`、
/// `render_with_effects` に対する既存保証)と同水準の「確保回数は定数」を、
/// GPU 直経路(zero-copy)にも要求する。
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
        )
        .unwrap();
    assert_eq!(
        compositor.effect_passes_created_textures(),
        1,
        "pass を持つ layer は少なくとも1枚のオフスクリーンを新規生成するはず"
    );

    // 連続 N フレーム(N=4)——確保回数が定数のままであることを縛る
    // (修理前は poll なし再利用ができず、release されないので毎回+1 されて red)。
    for _ in 0..4 {
        let _next = compositor
            .render_to_texture(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: one_layer(red.clone()),
                    passes: vec![EffectPass::Identity],
                }],
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

/// **正しさの直接確認(修理の副作用チェック)**: scratch をフレームをまたいで
/// 使い回すようになっても、前フレームの内容を引きずらない——Identity pass は
/// 全画素を copy で上書きするので、色を変えた2フレーム目は前フレームの色を
/// 見せないはず。poll なしの再利用が「同一 queue の submission 順で書いてから
/// 読む」(裁定171 v2 §0-5 と同じ論法)を実際に守れているかの直接証拠——
/// 同期が壊れていれば前フレームの色が透けて見えるか絵が乱れる。
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
