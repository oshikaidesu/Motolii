
use motolii_compositor::{
    BlendMode, CompSpec, Compositor, EffectPass, HeadlessGpu, Layer, LayerPlacement,
    LayerWithPasses, ResolvedCamera, PRESENTABLE_FORMAT,
};

const W: u32 = 64;
const H: u32 = 64;

fn comp() -> CompSpec {
    CompSpec { width: W, height: H }
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
        label: Some("isf-effect-regression"),
        size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
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
        label: Some("isf-effect-regression-readback"),
        size: (bytes_per_row as u64) * (H as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("isf-effect-regression-readback-encoder"),
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
        wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
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
            transform: LayerPlacement::from_transform([0.0, 0.0], [28.0, 28.0], [1.0, 1.0], 0.0, 0.0, 0.0),
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
fn render_into_applies_isf_effect_passes() {
    let (mut compositor, device, queue) = with_device_and_queue();

    let gray = vec![230u8, 230u8, 230u8, 255u8]
        .iter()
        .cloned()
        .cycle()
        .take((8 * 8 * 4) as usize)
        .collect::<Vec<u8>>();
    let source = compositor.upload_rgba("isf-gray", &gray, 8, 8).expect("upload_rgba");

    let target_without = readable_presentable(&device);
    compositor
        .render_into(
            &target_without,
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses { layer: small_layer(source.clone()), passes: vec![] }],
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
                layer: small_layer(source),
                passes: vec![EffectPass::Isf {
                    params: vec![
                        ("threshold".to_owned(), 0.1),
                        ("intensity".to_owned(), 3.0),
                        ("radius".to_owned(), 2.0),
                    ],
                }],
            }],
            motolii_compositor::NO_BACKGROUND,
        )
        .expect("render_into with an Isf pass");
    let bytes_with = readback_rgba(&device, &queue, &target_with);

    assert_ne!(
        bytes_with, bytes_without,
        "EffectPass::Isf を積んでも render_into の出力が変わらない\
         — 生 ISF ファイル(JSON manifest + GLSL)から naga で組んだ pipeline が\
         実際には効いていない"
    );
}
