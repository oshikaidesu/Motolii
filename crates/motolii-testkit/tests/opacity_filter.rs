//! INF-7g: OpacityFilter の出力が amount に比例することのゴールデン。

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_eval::Value;
use motolii_gpu::{download_rgba, upload_rgba, GpuCtx, PipelineCache};
use motolii_plugin::reference::OPACITY_FILTER;
use motolii_plugin::{FilterPlugin, RenderCtx, ResolvedParams, TextureRef};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const WIDTH: u32 = 4;
const HEIGHT: u32 = 2;
const NONUNIFORM_PREMUL_RGBA: [u8; 32] = [
    0, 0, 0, 0, 32, 16, 8, 64, 64, 32, 16, 128, 200, 100, 50, 200, 254, 128, 64, 254, 12, 24, 36,
    48, 90, 60, 30, 120, 8, 4, 2, 16,
];

#[test]
fn opacity_zero_clears_every_component() {
    let Some(gpu) = gpu_or_skip() else { return };
    let actual = render_opacity(&gpu, &NONUNIFORM_PREMUL_RGBA, 0.0);
    assert_pixels("opacity-zero", &actual, &[0; 32]);
}

#[test]
fn opacity_one_preserves_nonuniform_premul_rgba() {
    let Some(gpu) = gpu_or_skip() else { return };
    let actual = render_opacity(&gpu, &NONUNIFORM_PREMUL_RGBA, 1.0);
    assert_pixels("opacity-one", &actual, &NONUNIFORM_PREMUL_RGBA);
}

#[test]
fn opacity_half_scales_nonuniform_premul_rgba() {
    let Some(gpu) = gpu_or_skip() else { return };
    let actual = render_opacity(&gpu, &NONUNIFORM_PREMUL_RGBA, 0.5);
    let expected = NONUNIFORM_PREMUL_RGBA.map(|component| component / 2);
    assert_pixels("opacity-half-nonuniform", &actual, &expected);
}

fn render_opacity(gpu: &GpuCtx, input_rgba: &[u8], amount: f64) -> Vec<u8> {
    let frame = FrameDesc::packed(
        WIDTH,
        HEIGHT,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    );
    assert_eq!(input_rgba.len(), frame.data_size());
    let input = upload_rgba(gpu, &frame, input_rgba);
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
    params.insert("amount", Value::F64(amount));
    let mut pipelines = PipelineCache::new();
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("opacity-golden"),
        });
    OPACITY_FILTER
        .render(
            gpu,
            &mut pipelines,
            &mut encoder,
            &RenderCtx::new(RationalTime::ZERO, Quality::FINAL),
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
    download_rgba(gpu, &output).unwrap()
}

fn assert_pixels(name: &str, actual: &[u8], expected: &[u8]) {
    assert_rgba_close(
        name,
        RgbaImageDesc {
            width: WIDTH,
            height: HEIGHT,
        },
        actual,
        expected,
        tol::GPU_RASTER,
    );
}
