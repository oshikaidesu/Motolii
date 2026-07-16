//! D7: MaskNode 各モードの GPU ゴールデン(シェーダ直結)。

use motolii_core::{ColorSpace, FrameDesc, PixelFormat};
use motolii_gpu::{download_rgba, upload_rgba};
use motolii_nodes::{create_rgba_render_target, ClippingMaskMode, MaskNode};
use motolii_plugin::TextureRef;
use motolii_testkit::clipping_mask::{clipping_mask_mul_u8, ClippingMaskRef};
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

fn tiled(desc: FrameDesc, px: [u8; 4]) -> Vec<u8> {
    let mut out = vec![0u8; desc.data_size()];
    for chunk in out.chunks_exact_mut(4) {
        chunk.copy_from_slice(&px);
    }
    out
}

fn mode_ref(mode: ClippingMaskMode) -> ClippingMaskRef {
    match mode {
        ClippingMaskMode::Alpha => ClippingMaskRef::Alpha,
        ClippingMaskMode::Luminance => ClippingMaskRef::Luminance,
        ClippingMaskMode::InvertAlpha => ClippingMaskRef::InvertAlpha,
        ClippingMaskMode::InvertLuminance => ClippingMaskRef::InvertLuminance,
    }
}

fn run_mask(mode: ClippingMaskMode, content_px: [u8; 4], mask_px: [u8; 4], label: &str) {
    let Some(gpu) = gpu_or_skip() else { return };
    let desc = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let content = upload_rgba(&gpu, &desc, &tiled(desc, content_px));
    let mask = upload_rgba(&gpu, &desc, &tiled(desc, mask_px));
    let output = create_rgba_render_target(&gpu, desc, "mask-out");

    MaskNode::with_mode(&gpu, mode)
        .render(
            &gpu,
            TextureRef {
                texture: &content,
                desc,
            },
            TextureRef {
                texture: &mask,
                desc,
            },
            TextureRef {
                texture: &output,
                desc,
            },
        )
        .unwrap();

    let actual = download_rgba(&gpu, &output).unwrap();
    let expected_px = clipping_mask_mul_u8(content_px, mask_px, mode_ref(mode));
    let expected = tiled(desc, expected_px);
    assert_rgba_close(
        label,
        RgbaImageDesc {
            width: desc.width,
            height: desc.height,
        },
        &actual,
        &expected,
        tol::GPU_RASTER,
    );
}

#[test]
fn mask_node_alpha_multiplies_by_mask_alpha() {
    run_mask(
        ClippingMaskMode::Alpha,
        [0, 255, 0, 255],
        [255, 255, 255, 128],
        "mask-node-alpha",
    );
}

#[test]
fn mask_node_luminance_uses_bt709_on_premul_rgb() {
    run_mask(
        ClippingMaskMode::Luminance,
        [255, 255, 255, 255],
        [255, 0, 0, 255],
        "mask-node-luminance",
    );
}

#[test]
fn mask_node_invert_alpha() {
    run_mask(
        ClippingMaskMode::InvertAlpha,
        [0, 255, 0, 255],
        [255, 255, 255, 255],
        "mask-node-invert-alpha",
    );
}

#[test]
fn mask_node_invert_luminance() {
    run_mask(
        ClippingMaskMode::InvertLuminance,
        [255, 255, 255, 255],
        [255, 0, 0, 255],
        "mask-node-invert-luminance",
    );
}
