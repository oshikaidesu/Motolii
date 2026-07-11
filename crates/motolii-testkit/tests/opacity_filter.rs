//! INF-7g: OpacityFilter の出力が amount に比例することのゴールデン。

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime};
use motolii_eval::Value;
use motolii_gpu::{download_rgba, upload_rgba, PipelineCache};
use motolii_plugin::reference::OPACITY_FILTER;
use motolii_plugin::{FilterPlugin, ResolvedParams, TextureRef};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

#[test]
fn opacity_half_scales_premul_rgba() {
    let Some(gpu) = gpu_or_skip() else { return };
    let frame = FrameDesc::packed(4, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let mut input_rgba = vec![0u8; frame.data_size()];
    for px in input_rgba.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 100, 50, 255]);
    }
    let input = upload_rgba(&gpu, &frame, &input_rgba);
    let output = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("opacity-out"),
        size: wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let mut params = ResolvedParams::new();
    params.insert("amount", Value::F64(0.5));
    let mut pipelines = PipelineCache::new();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("opacity-golden"),
        });
    OPACITY_FILTER
        .render(
            &gpu,
            &mut pipelines,
            &mut encoder,
            RationalTime::ZERO,
            &params,
            TextureRef {
                texture: &input,
                desc: frame,
            },
            TextureRef {
                texture: &output,
                desc: frame,
            },
        )
        .unwrap();
    gpu.queue.submit(std::iter::once(encoder.finish()));

    let actual = download_rgba(&gpu, &output).unwrap();
    let mut expected = vec![0u8; frame.data_size()];
    for px in expected.chunks_exact_mut(4) {
        px.copy_from_slice(&[100, 50, 25, 127]);
    }
    assert_rgba_close(
        "opacity-half",
        RgbaImageDesc {
            width: frame.width,
            height: frame.height,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}
