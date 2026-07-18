//! VSM-A3-3: Radial Repeater GPU 画素意味の独立 CPU oracle 固定。

use std::f64::consts::PI;

use motolii_core::{ColorSpace, CompCamera, FrameDesc, PixelFormat, RationalTime};
use motolii_eval::Value;
use motolii_gpu::PipelineCache;
use motolii_plugin::{
    LayerSourceContext, LayerSourcePlugin, PluginError, ResolvedParams, TextureRef,
};
use motolii_plugin_radial_repeater::RADIAL_REPEATER_LAYER_SOURCE;
use motolii_testkit::purity::render_layer_source_rgba;
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

#[test]
fn radial_repeater_matches_independent_cpu_oracle() {
    let Some(gpu) = gpu_or_skip() else { return };

    let frame = FrameDesc::packed(48, 36, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let params = radial_params(7.0, 0.27, 0.055, 0.35, 0.85, [0.82, 0.41, 0.19, 0.72]);
    let t = RationalTime::try_new(5, 4).unwrap();
    let ctx = layer_ctx();
    let mut pipelines = PipelineCache::new();

    let gpu_rgba = render_layer_source_rgba(
        "radial-oracle-gpu",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &params,
        ctx,
        frame,
    )
    .unwrap();
    let expected = cpu_oracle_rgba(
        frame,
        OracleParams {
            count: 7,
            radius: 0.27,
            dot_radius: 0.055,
            phase: 0.35,
            angular_speed: 0.85,
            color: [0.82, 0.41, 0.19, 0.72],
        },
        t,
    );

    assert_rgba_close(
        "radial_repeater_matches_independent_cpu_oracle",
        RgbaImageDesc {
            width: frame.width,
            height: frame.height,
        },
        &gpu_rgba,
        &expected,
        tol::GPU_RASTER,
    );
}

/// `phase=0` のとき i=0 の中心は +X 軸上。
#[test]
fn phase_zero_places_first_instance_on_positive_x() {
    let Some(gpu) = gpu_or_skip() else { return };

    let frame = FrameDesc::packed(64, 64, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let params = radial_params(1.0, 0.30, 0.06, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]);
    let t = RationalTime::ZERO;
    let ctx = layer_ctx();
    let mut pipelines = PipelineCache::new();

    let rgba = render_layer_source_rgba(
        "phase-zero-pos-x",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &params,
        ctx,
        frame,
    )
    .unwrap();

    let (peak_x, peak_y, peak_a) = brightest_pixel(&rgba, frame.width, frame.height);
    assert!(
        peak_a > 200,
        "expected opaque dot near +X, peak alpha={peak_a}"
    );
    let (px, py) = pixel_center_canonical(peak_x, peak_y, frame.width, frame.height);
    assert!(
        px > 0.15,
        "phase=0 center should be on +X axis, canonical px={px}"
    );
    assert!(
        py.abs() < 0.08,
        "phase=0 center should stay near horizontal axis, canonical py={py}"
    );
}

/// 正の `angular_speed` は CCW（正準 Y-up で +Y 側へ進む）。
#[test]
fn positive_angular_speed_rotates_counterclockwise_with_time() {
    let Some(gpu) = gpu_or_skip() else { return };

    let frame = FrameDesc::packed(64, 64, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let params = radial_params(1.0, 0.30, 0.06, 0.0, PI / 2.0, [1.0, 1.0, 1.0, 1.0]);
    let t = RationalTime::from_seconds(1);
    let ctx = layer_ctx();
    let mut pipelines = PipelineCache::new();

    let rgba = render_layer_source_rgba(
        "ccw-angular-speed",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &params,
        ctx,
        frame,
    )
    .unwrap();

    let (peak_x, peak_y, peak_a) = brightest_pixel(&rgba, frame.width, frame.height);
    assert!(
        peak_a > 200,
        "expected opaque dot after rotation, peak alpha={peak_a}"
    );
    let (px, py) = pixel_center_canonical(peak_x, peak_y, frame.width, frame.height);
    assert!(
        px.abs() < 0.08,
        "after π/2 rad CCW rotation center should be near vertical axis, px={px}"
    );
    assert!(
        py > 0.15,
        "CCW from +X should move center toward +Y, canonical py={py}"
    );
}

#[test]
fn n8_overlap_alpha_addition_rejected() {
    let Some(gpu) = gpu_or_skip() else { return };

    let frame = FrameDesc::packed(32, 32, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let color = [0.6, 0.3, 0.9, 0.8];
    let dot_radius = 0.08;
    let t = RationalTime::from_seconds(2);
    let ctx = layer_ctx();
    let mut pipelines = PipelineCache::new();

    let one = render_layer_source_rgba(
        "n8-count-1",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &radial_params(1.0, 0.0, dot_radius, 0.2, 0.4, color),
        ctx,
        frame,
    )
    .unwrap();
    let many = render_layer_source_rgba(
        "n8-count-64",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &radial_params(64.0, 0.0, dot_radius, 0.2, 0.4, color),
        ctx,
        frame,
    )
    .unwrap();

    assert_eq!(
        one, many,
        "overlapping instances at radius=0 must not add alpha/RGB (union SDF)"
    );
}

#[test]
fn p6_draft_final_same_t_params_desc() {
    let Some(gpu) = gpu_or_skip() else { return };

    let frame = FrameDesc::packed(24, 24, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let params = radial_params(5.0, 0.22, 0.04, 0.1, 0.25, [0.5, 0.7, 0.9, 0.6]);
    let t = RationalTime::from_seconds(3);
    let ctx = layer_ctx();
    let mut pipelines = PipelineCache::new();

    let first = render_layer_source_rgba(
        "p6-first",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &params,
        ctx,
        frame,
    )
    .unwrap();
    let second = render_layer_source_rgba(
        "p6-second",
        &gpu,
        &mut pipelines,
        &RADIAL_REPEATER_LAYER_SOURCE,
        t,
        &params,
        ctx,
        frame,
    )
    .unwrap();

    assert_eq!(
        first, second,
        "identical (t, params, FrameDesc) must yield byte-exact same output (P6)"
    );
}

#[test]
fn n10_zero_dimension_typed_rejection() {
    let Some(gpu) = gpu_or_skip() else { return };

    let valid = FrameDesc::packed(1, 1, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("n10-valid-1x1"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let params = radial_params(1.0, 0.0, 0.04, 0.0, 0.0, [1.0, 1.0, 1.0, 1.0]);
    let ctx = layer_ctx();
    let mut pipelines = PipelineCache::new();

    for (label, bad_desc) in [
        ("zero-width", FrameDesc { width: 0, ..valid }),
        ("zero-height", FrameDesc { height: 0, ..valid }),
    ] {
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        let err = RADIAL_REPEATER_LAYER_SOURCE
            .render(
                &gpu,
                &mut pipelines,
                &mut encoder,
                RationalTime::ZERO,
                &params,
                ctx,
                TextureRef {
                    texture: &texture,
                    desc: bad_desc,
                },
            )
            .unwrap_err();
        assert!(
            matches!(err, PluginError::Render(_)),
            "{label}: expected PluginError::Render, got {err:?}"
        );
    }
}

fn layer_ctx() -> LayerSourceContext {
    LayerSourceContext {
        camera: CompCamera::DEFAULT,
    }
}

fn radial_params(
    count: f64,
    radius: f64,
    dot_radius: f64,
    phase: f64,
    angular_speed: f64,
    color: [f64; 4],
) -> ResolvedParams {
    let mut params = ResolvedParams::new();
    params.insert("count", Value::F64(count));
    params.insert("radius", Value::F64(radius));
    params.insert("dot_radius", Value::F64(dot_radius));
    params.insert("phase", Value::F64(phase));
    params.insert("angular_speed", Value::F64(angular_speed));
    params.insert("color", Value::Color(color));
    params
}

struct OracleParams {
    count: u32,
    radius: f64,
    dot_radius: f64,
    phase: f64,
    angular_speed: f64,
    color: [f64; 4],
}

/// A3S §7: union min SDF → 1 回 coverage → straight→premul → u8 round。
fn cpu_oracle_rgba(frame: FrameDesc, params: OracleParams, t: RationalTime) -> Vec<u8> {
    let OracleParams {
        count,
        radius,
        dot_radius,
        phase,
        angular_speed,
        color,
    } = params;
    let t_sec = t.as_seconds_f64();

    let width = frame.width as f64;
    let height = frame.height as f64;
    let w = 1.0 / height;
    let mut rgba = vec![0u8; frame.data_size()];

    for y in 0..frame.height {
        for x in 0..frame.width {
            let px = (x as f64 + 0.5 - width / 2.0) / height;
            let py = (height / 2.0 - (y as f64 + 0.5)) / height;

            let mut d = f64::INFINITY;
            let n = count as f64;
            for i in 0..count {
                let theta = phase + angular_speed * t_sec + 2.0 * PI * i as f64 / n;
                let cx = radius * theta.cos();
                let cy = radius * theta.sin();
                let dist = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - dot_radius;
                d = d.min(dist);
            }

            let c = (0.5 - d / w).clamp(0.0, 1.0);
            let premul = [
                color[0] * color[3] * c,
                color[1] * color[3] * c,
                color[2] * color[3] * c,
                color[3] * c,
            ];
            let idx = ((y * frame.width + x) * 4) as usize;
            for (channel, value) in premul.iter().enumerate() {
                rgba[idx + channel] = quantize_u8(*value);
            }
        }
    }
    rgba
}

fn quantize_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn brightest_pixel(rgba: &[u8], width: u32, height: u32) -> (u32, u32, u8) {
    let mut best = (0u32, 0u32, 0u8);
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let a = rgba[idx + 3];
            if a > best.2 {
                best = (x, y, a);
            }
        }
    }
    best
}

fn pixel_center_canonical(x: u32, y: u32, width: u32, height: u32) -> (f64, f64) {
    let w = width as f64;
    let h = height as f64;
    let px = (x as f64 + 0.5 - w / 2.0) / h;
    let py = (h / 2.0 - (y as f64 + 0.5)) / h;
    (px, py)
}
