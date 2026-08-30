
use motolii::render::compositor::{
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, LayerWithPasses,
    ResolvedCamera, PRESENTABLE_FORMAT,
};

const W: u32 = 32;
const H: u32 = 32;

fn comp() -> CompSpec {
    CompSpec { width: W, height: H }
}

fn layers(c: &mut Compositor, mode: BlendMode) -> Vec<LayerWithPasses> {
    let under = c
        .upload_rgba("under", &vec![200u8; (W * H * 4) as usize], W, H)
        .expect("under");
    let over = c
        .upload_rgba("over", &vec![120u8; (W * H * 4) as usize], W, H)
        .expect("over");
    let place = |texture, order, blend_mode| LayerWithPasses {
        layer: Layer {
            texture,
            size: [W as f32, H as f32],
            placement: LayerPlacement { order, ..LayerPlacement::default() },
            pinned: false,
            blend_mode,
        },
        passes: vec![],
    };
    vec![place(under, 0, BlendMode::Normal), place(over, 1, mode)]
}

fn readback(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bpr = (W * 4).div_ceil(align) * align;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bpr as u64) * (H as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let data = slice.get_mapped_range();
    let out = data[..4].to_vec();
    drop(data);
    buf.unmap();
    out
}

#[test]
fn render_into_matches_render_with_effects_for_separable_blends() {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    let mut c = Compositor::with_device_using_headless_defaults(device.clone(), queue.clone())
        .expect("with_device");

    let mut mismatched = Vec::new();
    for mode in [BlendMode::Normal, BlendMode::Add, BlendMode::Multiply, BlendMode::Screen, BlendMode::Difference] {
        let ls = layers(&mut c, mode);
        let cpu = c
            .render_with_effects(comp(), ResolvedCamera::default(), &ls, motolii::render::compositor::NO_BACKGROUND)
            .expect("render_with_effects")[..4]
            .to_vec();

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        c.render_into(&target, comp(), ResolvedCamera::default(), &ls, motolii::render::compositor::NO_BACKGROUND)
            .expect("render_into");
        let gpu = readback(&device, &queue, &target);

        if cpu.iter().zip(&gpu).any(|(a, b)| (*a as i32 - *b as i32).abs() > 2) {
            mismatched.push(format!("{mode:?}: export={cpu:?} stage={gpu:?}"));
        }
    }

    assert!(
        mismatched.is_empty(),
        "Stage と export の絵が違う blend mode がある:\n{}",
        mismatched.join("\n")
    );
}
