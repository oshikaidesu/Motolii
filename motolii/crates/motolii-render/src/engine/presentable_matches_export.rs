use motolii_edit::{Document, Intent};
use super::*;
use crate::doc::store::{Composition, Fps, RationalTime};

#[test]
fn the_window_target_holds_the_same_bytes_as_the_export_readback() {
    let (w, h) = (64u32, 32u32);
    let fps = Fps::try_new(30, 1).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: w,
        height: h,
        fps,
        duration_frames: 1,
        // 中間調でないと encode の回数が見えない
        background: [0.5, 0.25, 0.1, 1.0],
    }))
    .unwrap();
    let mut engine = super::Engine::new().unwrap();
    let t = RationalTime::try_from_frame(0, fps).unwrap();
    let export = engine.render_frame(&doc.view(), t).unwrap();
    assert!(export[0] > 8 && export[0] < 247, "中間調のはず: {:?}", &export[..4]);

    let format = crate::render::compositor::PRESENTABLE_FORMAT;
    let device = engine.gpu_device().clone();
    let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    engine.render_frame_into(&doc.view(), t, &target).unwrap();

    let bytes_per_row = (w * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(bytes_per_row * h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(h),
            },
        },
        size,
    );
    engine.gpu_queue().submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = slice.get_mapped_range();

    let bgra = format!("{format:?}").starts_with("Bgra");
    for y in 0..h as usize {
        for x in 0..w as usize {
            let window = &data[y * bytes_per_row as usize + x * 4..][..4];
            let exported = &export[(y * w as usize + x) * 4..][..4];
            for c in 0..3 {
                let wc = if bgra { 2 - c } else { c };
                assert_eq!(window[wc], exported[c], "({x},{y}) channel {c}: 窓 {:?} export {:?}", window, exported);
            }
        }
    }
}
