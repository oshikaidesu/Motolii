//! 裁定256 — 共有面検査は通す。書き込みは blit で先に通さない。

use motolii_compositor::{
    check_presentable_target, BlendMode, CompSpec, Compositor, CompositorError, EffectPass,
    HeadlessGpu, Layer, LayerPlacement, LayerWithPasses, ResolvedCamera, PRESENTABLE_FORMAT,
};

const W: u32 = 64;
const H: u32 = 64;

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

fn with_device() -> (Compositor, wgpu::Device) {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let compositor = Compositor::with_device_using_headless_defaults(device.clone(), queue)
        .expect("with_device");
    (compositor, device)
}

/// `with_device` と同じだが、readback に要る `Queue` も持ち帰る
/// (`render_into_applies_effect_passes` 専用 — 他の試験は書き込みの成否しか見ない
/// ので `Queue` を要らない)。
fn with_device_and_queue() -> (Compositor, wgpu::Device, wgpu::Queue) {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let compositor =
        Compositor::with_device_using_headless_defaults(device.clone(), queue.clone())
            .expect("with_device");
    (compositor, device, queue)
}

/// `render_into` の target は呼び手が作る(=`RENDER_ATTACHMENT` に加えて
/// `COPY_SRC` を足せる)ので、`render_to_texture` の main_target のような blit
/// 迂回は要らない——`copy_texture_to_buffer` で直接読み戻す。
fn readable_presentable(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render_into-effect-regression"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: PRESENTABLE_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn readback_rgba(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let bytes_per_row_unaligned = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bytes_per_row = bytes_per_row_unaligned.div_ceil(align) * align;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("render_into-effect-regression-readback"),
        size: (bytes_per_row as u64) * (H as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render_into-effect-regression-readback-encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
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
    let mut out = vec![0u8; (W as usize) * (H as usize) * 4];
    for row in 0..H as usize {
        let src_start = row * bytes_per_row as usize;
        let dst_start = row * (W as usize) * 4;
        out[dst_start..dst_start + (W as usize) * 4]
            .copy_from_slice(&data[src_start..src_start + (W as usize) * 4]);
    }
    drop(data);
    buffer.unmap();
    out
}

/// 8x8 の layer を comp 中央 (28,28) へ、`passes` を積んで置く。
fn small_layer(texture: motolii_compositor::GpuTexture2D) -> Layer {
    Layer {
        texture,
        size: [8.0, 8.0],
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform(
                [0.0, 0.0],
                [28.0, 28.0],
                [1.0, 1.0],
                0.0,
                0.0,
                0.0,
            ),
            order: 0,
            opacity: 1.0,
            z: 0.0,
        },
        pinned: false,
        blend_mode: BlendMode::Normal,
    }
}

/// **2026-08-28 の穴の直接オラクル**: `render_into` は元々 `lwp.passes` を一切
/// 読まず `lwp.layer.texture` をそのまま描いていた——Inspector の FX STACK で
/// GLOW を積んでも、`Intent::SetEffects` の書き込みも Inspector の投影も
/// 正しく動くのに、Makepad の共有面(zero-copy Stage)には絵が一切反映されて
/// いなかった(export/CPU 読み戻し経路は元から正しかった)。この試験は
/// 「同じ layer に `EffectPass::Glow` を積むと `render_into` の出力が変わる」
/// ことだけを縛る——変わらなければ、この穴が再発している。
#[test]
fn render_into_applies_effect_passes() {
    let (mut compositor, device, queue) = with_device_and_queue();

    let white = compositor
        .upload_rgba("white", &vec![255u8; (8 * 8 * 4) as usize], 8, 8)
        .expect("upload_rgba");

    let target_without = readable_presentable(&device);
    compositor
        .render_into(
            &target_without,
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: small_layer(white.clone()),
                passes: vec![],
            }],
        )
        .expect("render_into without passes");
    let bytes_without = readback_rgba(&device, &queue, &target_without);

    let target_with = readable_presentable(&device);
    compositor
        .render_into(
            &target_with,
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: small_layer(white),
                passes: vec![EffectPass::Glow {
                    threshold: 0.0,
                    intensity: 5.0,
                    radius: 1.0,
                }],
            }],
        )
        .expect("render_into with a Glow pass");
    let bytes_with = readback_rgba(&device, &queue, &target_with);

    assert_ne!(
        bytes_with, bytes_without,
        "EffectPass::Glow を積んでも render_into の出力が変わらない\
         — render_into が passes を無視している(2026-08-28 の穴の再発)"
    );
}

fn texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("presentable-test"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn presentable(device: &wgpu::Device) -> wgpu::Texture {
    texture(
        device,
        W,
        H,
        PRESENTABLE_FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    )
}

#[test]
fn presentable_target_accepts_host_spec() {
    let (_compositor, device) = with_device();
    check_presentable_target(&presentable(&device), comp()).expect("host spec");
}

#[test]
fn presentable_target_rejects_wrong_format() {
    let (_compositor, device) = with_device();
    let got = check_presentable_target(
        &texture(
            &device,
            W,
            H,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        ),
        comp(),
    );
    assert!(matches!(got, Err(CompositorError::PresentableFormat { .. })));
}

#[test]
fn presentable_target_rejects_wrong_size() {
    let (_compositor, device) = with_device();
    let got = check_presentable_target(
        &texture(
            &device,
            8,
            8,
            PRESENTABLE_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        ),
        comp(),
    );
    assert!(matches!(
        got,
        Err(CompositorError::PresentableSize {
            got: [8, 8],
            expected: [W, H]
        })
    ));
}

#[test]
fn presentable_target_rejects_missing_render_attachment() {
    let (_compositor, device) = with_device();
    let got = check_presentable_target(
        &texture(
            &device,
            W,
            H,
            PRESENTABLE_FORMAT,
            wgpu::TextureUsages::TEXTURE_BINDING,
        ),
        comp(),
    );
    assert!(matches!(got, Err(CompositorError::PresentableUsage)));
}

#[test]
fn render_into_writes_the_external_target() {
    let (mut compositor, device) = with_device();
    let target = presentable(&device);
    compositor
        .render_into(&target, comp(), ResolvedCamera::default(), &[] as &[LayerWithPasses])
        .expect("external resolved へ直接書く");
}

// `render_into_does_not_blit_before_external_resolved` はここに居た。すぐ上の
// `render_into_writes_the_external_target` と**同じ呼び出しに正反対を要求**して
// いたので、片方は必ず落ちる。待ちの番人の方が古く、裁定256 で fork に
// `ViewBuilder::new_with_external_resolved` が着いた時点で前提が消えていた
// (2026-08-27 撤去)。
