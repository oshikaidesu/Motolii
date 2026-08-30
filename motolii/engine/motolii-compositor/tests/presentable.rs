
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

fn with_device_and_queue() -> (Compositor, wgpu::Device, wgpu::Queue) {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let compositor =
        Compositor::with_device_using_headless_defaults(device.clone(), queue.clone())
            .expect("with_device");
    (compositor, device, queue)
}

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
        view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
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
            rotation_x: 0.0,
            rotation_y: 0.0,
        },
        pinned: false,
        blend_mode: BlendMode::Normal,
    }
}

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
            motolii_compositor::NO_BACKGROUND,
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
            motolii_compositor::NO_BACKGROUND,
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
        .render_into(&target, comp(), ResolvedCamera::default(), &[] as &[LayerWithPasses], motolii_compositor::NO_BACKGROUND)
        .expect("external resolved へ直接書く");
}

fn tilted_layer(texture: motolii_compositor::GpuTexture2D, deg: f32) -> Layer {
    let mut layer = small_layer(texture);
    layer.placement.rotation_x = deg;
    layer
}

#[test]
fn render_into_draws_tilted_plates() {
    let (mut compositor, device, queue) = with_device_and_queue();
    let white = compositor
        .upload_rgba("white", &vec![255u8; 8 * 8 * 4], 8, 8)
        .expect("upload_rgba");

    let mut count = |compositor: &mut Compositor, deg: f32| {
        let target = readable_presentable(&device);
        compositor
            .render_into(
                &target,
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: tilted_layer(white.clone(), deg),
                    passes: vec![],
                }],
                motolii_compositor::NO_BACKGROUND,
            )
            .expect("render_into");
        readback_rgba(&device, &queue, &target)
            .chunks(4)
            .filter(|p| p[3] > 0)
            .count()
    };

    let flat = count(&mut compositor, 0.0);
    let tilted = count(&mut compositor, 60.0);
    assert!(flat > 0 && tilted > 0, "flat={flat} tilted={tilted}");
    assert!(tilted < flat, "60° 傾けても縮まない: flat={flat} tilted={tilted}");
}

#[test]
fn tilt_survives_a_pinned_background() {
    let (mut compositor, device, queue) = with_device_and_queue();
    let white = compositor
        .upload_rgba("white", &vec![255u8; 8 * 8 * 4], 8, 8)
        .expect("upload_rgba");
    let mut blue = vec![0u8; (W * H * 4) as usize];
    for p in blue.chunks_mut(4) {
        p[2] = 255;
        p[3] = 255;
    }
    let bg = compositor
        .upload_rgba("bg", &blue, W, H)
        .expect("upload_rgba bg");

    let mut white_pixels = |compositor: &mut Compositor, deg: f32| {
        let target = readable_presentable(&device);
        let background = Layer {
            texture: bg.clone(),
            size: [W as f32, H as f32],
            placement: LayerPlacement {
                order: -1,
                ..LayerPlacement::default()
            },
            pinned: true,
            blend_mode: BlendMode::Normal,
        };
        compositor
            .render_into(
                &target,
                comp(),
                ResolvedCamera::default(),
                &[
                    LayerWithPasses { layer: background, passes: vec![] },
                    LayerWithPasses { layer: tilted_layer(white.clone(), deg), passes: vec![] },
                ],
                motolii_compositor::NO_BACKGROUND,
            )
            .expect("render_into");
        readback_rgba(&device, &queue, &target)
            .chunks(4)
            .filter(|p| p[0] > 128 && p[1] > 128)
            .count()
    };

    let flat = white_pixels(&mut compositor, 0.0);
    let tilted = white_pixels(&mut compositor, 20.0);
    assert!(flat > 0, "傾き 0 で板が出ていない");
    assert!(tilted * 2 > flat, "傾けた板が背景に負けて消えた: flat={flat} tilted={tilted}");
}
