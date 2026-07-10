use motolii_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime};
use motolii_eval::Value;
use motolii_gpu::{download_rgba, upload_rgba, GpuCtx, PipelineCache};
use motolii_nodes::{
    create_rgba_render_target, CanonicalPoint, CanonicalSize, CircleOverlay, CompositeMode,
    CompositeNode, FilterNode, LineOverlay, OverlayNode, RectOverlay,
};
use motolii_plugin::reference::CLEAR_FILTER;
use motolii_plugin::TextureRef;
use motolii_testkit::{assert_rgba_close, gpu_or_skip, RgbaImageDesc};

#[test]
fn clear_filter_runs_through_node_and_matches_golden() {
    let Some(gpu) = gpu_or_skip() else { return };
    let desc = FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let input = create_rgba_render_target(&gpu, desc, "clear-filter-input");
    let output = create_rgba_render_target(&gpu, desc, "clear-filter-output");
    let mut pipelines = PipelineCache::new();

    let mut node = FilterNode::new(&CLEAR_FILTER);
    node.set_param("color", Value::Color([1.0, 0.0, 0.0, 1.0]));
    node.render(
        &gpu,
        &mut pipelines,
        RationalTime::ZERO,
        TextureRef {
            texture: &input,
            desc,
        },
        TextureRef {
            texture: &output,
            desc,
        },
    )
    .unwrap();

    let actual = download_rgba(&gpu, &output).unwrap();
    let mut expected = vec![0u8; desc.data_size()];
    for px in expected.chunks_exact_mut(4) {
        px.copy_from_slice(&[255, 0, 0, 255]);
    }
    assert_rgba_close(
        "clear-filter-red",
        RgbaImageDesc {
            width: desc.width,
            height: desc.height,
        },
        &actual,
        &expected,
        0,
    );
}

#[test]
fn overlay_rect_uses_canonical_space_across_resolutions() {
    let Some(gpu) = gpu_or_skip() else { return };
    let cases = [
        (
            16,
            8,
            CanonicalPoint::CENTER,
            CanonicalSize {
                width: 0.5,
                height: 0.5,
            },
        ),
        (
            32,
            16,
            CanonicalPoint::CENTER,
            CanonicalSize {
                width: 0.5,
                height: 0.5,
            },
        ),
        (
            15,
            9,
            CanonicalPoint { x: 0.1, y: 0.25 },
            CanonicalSize {
                width: 1.0 / 3.0,
                height: 4.0 / 9.0,
            },
        ),
        (
            30,
            18,
            CanonicalPoint { x: 0.1, y: 0.25 },
            CanonicalSize {
                width: 1.0 / 3.0,
                height: 4.0 / 9.0,
            },
        ),
    ];

    for (width, height, center, size) in cases {
        let desc = FrameDesc::packed(
            width,
            height,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            false,
        );
        let input_data = gradient_pattern(desc);
        let input = upload_rgba(&gpu, &desc, &input_data);
        let output = create_rgba_render_target(&gpu, desc, "overlay-output");
        prefill_with_magenta(&gpu, desc, &output);

        let overlay = OverlayNode::with_rect(
            &gpu,
            RectOverlay {
                center,
                size,
                color: [0.0, 1.0, 0.0, 1.0],
            },
        );
        overlay
            .render(
                &gpu,
                TextureRef {
                    texture: &input,
                    desc,
                },
                TextureRef {
                    texture: &output,
                    desc,
                },
            )
            .unwrap();

        let actual = download_rgba(&gpu, &output).unwrap();
        let expected =
            expected_rect_over_pattern(desc, &input_data, [0, 255, 0, 255], center, size);
        assert_rgba_close(
            &format!("overlay-canonical-{width}x{height}"),
            RgbaImageDesc { width, height },
            &actual,
            &expected,
            0,
        );
    }
}

#[test]
fn composite_normal_over_uses_premultiplied_alpha() {
    let Some(gpu) = gpu_or_skip() else { return };
    let desc = FrameDesc::packed(4, 3, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);

    let bg_px = [0u8, 128, 0, 128]; // premul green, alpha ~= 0.5
    let fg_px = [128u8, 0, 0, 128]; // premul red, alpha ~= 0.5
    let background_data = tiled(desc, bg_px);
    let foreground_data = tiled(desc, fg_px);
    let background = upload_rgba(&gpu, &desc, &background_data);
    let foreground = upload_rgba(&gpu, &desc, &foreground_data);
    let output = create_rgba_render_target(&gpu, desc, "composite-output");

    CompositeNode::new(&gpu)
        .render(
            &gpu,
            TextureRef {
                texture: &background,
                desc,
            },
            TextureRef {
                texture: &foreground,
                desc,
            },
            TextureRef {
                texture: &output,
                desc,
            },
        )
        .unwrap();

    let actual = download_rgba(&gpu, &output).unwrap();
    let expected_px = premul_over_u8(bg_px, fg_px);
    let expected = tiled(desc, expected_px);
    assert_rgba_close(
        "composite-premul-over",
        RgbaImageDesc {
            width: desc.width,
            height: desc.height,
        },
        &actual,
        &expected,
        1,
    );
}

#[test]
fn overlay_circle_uses_canonical_space_across_resolutions() {
    let Some(gpu) = gpu_or_skip() else { return };
    let cases = [
        (16, 8, CanonicalPoint::CENTER, 0.25),
        (32, 16, CanonicalPoint::CENTER, 0.25),
        (15, 9, CanonicalPoint { x: 0.1, y: 0.25 }, 1.0 / 6.0),
        (30, 18, CanonicalPoint { x: 0.1, y: 0.25 }, 1.0 / 6.0),
    ];

    for (width, height, center, radius) in cases {
        let desc = FrameDesc::packed(
            width,
            height,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            false,
        );
        let input_data = gradient_pattern(desc);
        let input = upload_rgba(&gpu, &desc, &input_data);
        let output = create_rgba_render_target(&gpu, desc, "overlay-circle-output");
        prefill_with_magenta(&gpu, desc, &output);

        OverlayNode::with_circle(
            &gpu,
            CircleOverlay {
                center,
                radius,
                color: [0.0, 0.0, 1.0, 1.0],
            },
        )
        .render(
            &gpu,
            TextureRef {
                texture: &input,
                desc,
            },
            TextureRef {
                texture: &output,
                desc,
            },
        )
        .unwrap();

        let actual = download_rgba(&gpu, &output).unwrap();
        let expected =
            expected_circle_over_pattern(desc, &input_data, [0, 0, 255, 255], center, radius);
        assert_rgba_close(
            &format!("overlay-circle-{width}x{height}"),
            RgbaImageDesc { width, height },
            &actual,
            &expected,
            0,
        );
    }
}

#[test]
fn overlay_line_uses_canonical_space_across_resolutions() {
    let Some(gpu) = gpu_or_skip() else { return };
    let cases = [
        (
            16,
            8,
            CanonicalPoint { x: -0.4, y: 0.3 },
            CanonicalPoint { x: 0.4, y: -0.3 },
            0.125,
        ),
        (
            32,
            16,
            CanonicalPoint { x: -0.4, y: 0.3 },
            CanonicalPoint { x: 0.4, y: -0.3 },
            0.125,
        ),
        (
            15,
            9,
            CanonicalPoint { x: -0.2, y: 0.4 },
            CanonicalPoint { x: 0.5, y: -0.1 },
            1.0 / 9.0,
        ),
        (
            30,
            18,
            CanonicalPoint { x: -0.2, y: 0.4 },
            CanonicalPoint { x: 0.5, y: -0.1 },
            1.0 / 9.0,
        ),
    ];

    for (width, height, start, end, line_width) in cases {
        let desc = FrameDesc::packed(
            width,
            height,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            false,
        );
        let input_data = gradient_pattern(desc);
        let input = upload_rgba(&gpu, &desc, &input_data);
        let output = create_rgba_render_target(&gpu, desc, "overlay-line-output");
        prefill_with_magenta(&gpu, desc, &output);

        OverlayNode::with_line(
            &gpu,
            LineOverlay {
                start,
                end,
                width: line_width,
                color: [1.0, 0.5, 0.0, 1.0],
            },
        )
        .render(
            &gpu,
            TextureRef {
                texture: &input,
                desc,
            },
            TextureRef {
                texture: &output,
                desc,
            },
        )
        .unwrap();

        let actual = download_rgba(&gpu, &output).unwrap();
        let expected = expected_line_over_pattern(
            desc,
            &input_data,
            [255, 128, 0, 255],
            start,
            end,
            line_width,
        );
        assert_rgba_close(
            &format!("overlay-line-{width}x{height}"),
            RgbaImageDesc { width, height },
            &actual,
            &expected,
            0,
        );
    }
}

#[test]
fn composite_add_and_multiply_use_premultiplied_alpha() {
    let Some(gpu) = gpu_or_skip() else { return };
    let cases = [(4, 3), (16, 8)];

    let bg_px = [0u8, 128, 0, 128];
    let fg_px = [128u8, 0, 0, 128];

    for (width, height) in cases {
        let desc = FrameDesc::packed(
            width,
            height,
            PixelFormat::Rgba8Unorm,
            ColorSpace::Srgb,
            true,
        );
        let background_data = tiled(desc, bg_px);
        let foreground_data = tiled(desc, fg_px);
        let background = upload_rgba(&gpu, &desc, &background_data);
        let foreground = upload_rgba(&gpu, &desc, &foreground_data);

        for (label, mode, expected_px) in [
            (
                "add",
                CompositeMode::Add,
                premul_add_u8(bg_px, fg_px),
            ),
            (
                "multiply",
                CompositeMode::Multiply,
                premul_multiply_u8(bg_px, fg_px),
            ),
        ] {
            let output = create_rgba_render_target(&gpu, desc, "composite-output");
            CompositeNode::with_mode(&gpu, mode)
                .render(
                    &gpu,
                    TextureRef {
                        texture: &background,
                        desc,
                    },
                    TextureRef {
                        texture: &foreground,
                        desc,
                    },
                    TextureRef {
                        texture: &output,
                        desc,
                    },
                )
                .unwrap();

            let actual = download_rgba(&gpu, &output).unwrap();
            let expected = tiled(desc, expected_px);
            assert_rgba_close(
                &format!("composite-{label}-{width}x{height}"),
                RgbaImageDesc { width, height },
                &actual,
                &expected,
                1,
            );
        }
    }
}

fn prefill_with_magenta(gpu: &GpuCtx, desc: FrameDesc, output: &wgpu::Texture) {
    let input = create_rgba_render_target(gpu, desc, "overlay-prefill-input");
    let mut pipelines = PipelineCache::new();
    let mut clear = FilterNode::new(&CLEAR_FILTER);
    clear.set_param("color", Value::Color([1.0, 0.0, 1.0, 1.0]));
    clear
        .render(
            gpu,
            &mut pipelines,
            RationalTime::ZERO,
            TextureRef {
                texture: &input,
                desc,
            },
            TextureRef {
                texture: output,
                desc,
            },
        )
        .unwrap();
}

fn gradient_pattern(desc: FrameDesc) -> Vec<u8> {
    let mut out = vec![0u8; desc.data_size()];
    for y in 0..desc.height {
        for x in 0..desc.width {
            let i = ((y * desc.width + x) * 4) as usize;
            out[i] = if desc.width <= 1 {
                0
            } else {
                (x * 255 / (desc.width - 1)) as u8
            };
            out[i + 1] = if desc.height <= 1 {
                0
            } else {
                (y * 255 / (desc.height - 1)) as u8
            };
            out[i + 2] = ((x + y) * 255 / (desc.width + desc.height - 2)) as u8;
            out[i + 3] = 255;
        }
    }
    out
}

fn tiled(desc: FrameDesc, px: [u8; 4]) -> Vec<u8> {
    let mut out = vec![0u8; desc.data_size()];
    for p in out.chunks_exact_mut(4) {
        p.copy_from_slice(&px);
    }
    out
}

fn premul_over_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let bg = bg.map(|v| v as f64 / 255.0);
    let fg = fg.map(|v| v as f64 / 255.0);
    let inv_a = 1.0 - fg[3];
    [
        to_u8(fg[0] + bg[0] * inv_a),
        to_u8(fg[1] + bg[1] * inv_a),
        to_u8(fg[2] + bg[2] * inv_a),
        to_u8(fg[3] + bg[3] * inv_a),
    ]
}

fn premul_add_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let bg = bg.map(|v| v as f64 / 255.0);
    let fg = fg.map(|v| v as f64 / 255.0);
    [
        to_u8((fg[0] + bg[0]).min(1.0)),
        to_u8((fg[1] + bg[1]).min(1.0)),
        to_u8((fg[2] + bg[2]).min(1.0)),
        to_u8((fg[3] + bg[3]).min(1.0)),
    ]
}

fn premul_multiply_u8(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let bg = bg.map(|v| v as f64 / 255.0);
    let fg = fg.map(|v| v as f64 / 255.0);
    [
        to_u8(fg[0] * bg[0]),
        to_u8(fg[1] * bg[1]),
        to_u8(fg[2] * bg[2]),
        to_u8(fg[3] * bg[3]),
    ]
}

fn to_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn expected_rect_over_pattern(
    desc: FrameDesc,
    base: &[u8],
    color: [u8; 4],
    center: CanonicalPoint,
    size: CanonicalSize,
) -> Vec<u8> {
    let mut out = base.to_vec();
    let h = desc.height as f64;
    let center_x = desc.width as f64 * 0.5 + center.x * h;
    let center_y = desc.height as f64 * 0.5 - center.y * h;
    let rect_w = size.width * h;
    let rect_h = size.height * h;
    let min_x = center_x - rect_w * 0.5;
    let max_x = center_x + rect_w * 0.5;
    let min_y = center_y - rect_h * 0.5;
    let max_y = center_y + rect_h * 0.5;

    for y in 0..desc.height {
        for x in 0..desc.width {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            if px >= min_x && px < max_x && py >= min_y && py < max_y {
                let i = ((y * desc.width + x) * 4) as usize;
                out[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

fn expected_circle_over_pattern(
    desc: FrameDesc,
    base: &[u8],
    color: [u8; 4],
    center: CanonicalPoint,
    radius: f64,
) -> Vec<u8> {
    let mut out = base.to_vec();
    let h = desc.height as f64;
    let center_x = desc.width as f64 * 0.5 + center.x * h;
    let center_y = desc.height as f64 * 0.5 - center.y * h;
    let radius_px = radius * h;
    let radius_sq = radius_px * radius_px;

    for y in 0..desc.height {
        for x in 0..desc.width {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;
            let dx = px - center_x;
            let dy = py - center_y;
            if dx * dx + dy * dy < radius_sq {
                let i = ((y * desc.width + x) * 4) as usize;
                out[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

fn expected_line_over_pattern(
    desc: FrameDesc,
    base: &[u8],
    color: [u8; 4],
    start: CanonicalPoint,
    end: CanonicalPoint,
    width: f64,
) -> Vec<u8> {
    let mut out = base.to_vec();
    let h = desc.height as f32;
    let center_x = desc.width as f32 * 0.5;
    let center_y = desc.height as f32 * 0.5;
    let start_x = center_x + start.x as f32 * h;
    let start_y = center_y - start.y as f32 * h;
    let end_x = center_x + end.x as f32 * h;
    let end_y = center_y - end.y as f32 * h;
    let half_width = width as f32 * h * 0.5;

    for y in 0..desc.height {
        for x in 0..desc.width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            if dist_to_segment_f32(px, py, start_x, start_y, end_x, end_y) < half_width {
                let i = ((y * desc.width + x) * 4) as usize;
                out[i..i + 4].copy_from_slice(&color);
            }
        }
    }
    out
}

fn dist_to_segment_f32(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let abx = bx - ax;
    let aby = by - ay;
    let denom = abx * abx + aby * aby;
    let (cx, cy) = if denom <= 1e-8 {
        (ax, ay)
    } else {
        let t = ((px - ax) * abx + (py - ay) * aby) / denom;
        let t = t.clamp(0.0, 1.0);
        (ax + abx * t, ay + aby * t)
    };
    let dx = px - cx;
    let dy = py - cy;
    (dx * dx + dy * dy).sqrt()
}
