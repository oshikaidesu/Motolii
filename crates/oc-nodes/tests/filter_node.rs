use oc_core::{ColorSpace, FrameDesc, PixelFormat, RationalTime};
use oc_eval::Value;
use oc_gpu::{download_rgba, upload_rgba, GpuCtx};
use oc_nodes::{
    create_rgba_render_target, CanonicalPoint, CanonicalSize, CompositeNode, FilterNode,
    OverlayNode, RectOverlay,
};
use oc_plugin::reference::CLEAR_FILTER;
use oc_plugin::TextureRef;
use oc_testkit::{assert_rgba_close, RgbaImageDesc};

fn gpu_or_skip() -> Option<GpuCtx> {
    match GpuCtx::new_headless() {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("SKIP: no GPU adapter: {e}");
            None
        }
    }
}

#[test]
fn clear_filter_runs_through_node_and_matches_golden() {
    let Some(gpu) = gpu_or_skip() else { return };
    let desc = FrameDesc::packed(16, 8, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let input = create_rgba_render_target(&gpu, desc, "clear-filter-input");
    let output = create_rgba_render_target(&gpu, desc, "clear-filter-output");

    let mut node = FilterNode::new(&CLEAR_FILTER);
    node.set_param("color", Value::Color([1.0, 0.0, 0.0, 1.0]));
    node.render(
        &gpu,
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

        let overlay = OverlayNode::new(
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

fn prefill_with_magenta(gpu: &GpuCtx, desc: FrameDesc, output: &wgpu::Texture) {
    let input = create_rgba_render_target(gpu, desc, "overlay-prefill-input");
    let mut clear = FilterNode::new(&CLEAR_FILTER);
    clear.set_param("color", Value::Color([1.0, 0.0, 1.0, 1.0]));
    clear
        .render(
            gpu,
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
