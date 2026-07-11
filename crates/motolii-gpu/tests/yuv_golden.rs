//! M1-T3完了条件: YUV→RGB変換のGPU結果がCPU参照実装(=理論値)と一致すること(B-3)。
//! レビュー指摘(#3)により、709/601 × limited/full の組合せを検証する。
//!
//! GPUアダプタが無い環境ではskip(CIはlavapipeで必ず実行)。

use motolii_core::{ColorSpace, PixelFormat};
use motolii_gpu::{download_rgba, solid_yuv420p, YuvToRgba};
use motolii_testkit::cpu_reference::yuv_to_rgba_reference;
use motolii_testkit::{assert_rgba_close, gpu_or_skip, RgbaImageDesc};

#[test]
fn yuv_matches_reference_across_color_spaces() {
    let Some(gpu) = gpu_or_skip() else { return };
    let mut conv = YuvToRgba::new(&gpu);

    // 代表色 (name, Y, U, V)。各色空間で同じYUV値を入れ、係数の違いが
    // 出力RGBの違いとして正しく現れることをCPU参照実装と突き合わせる。
    let bars = [
        ("white", 235u8, 128u8, 128u8),
        ("black", 16, 128, 128),
        ("red", 63, 102, 240),
        ("green", 173, 42, 26),
        ("blue", 32, 240, 118),
        ("gray", 126, 128, 128),
    ];
    let spaces = [
        ("709limited", ColorSpace::Rec709Limited),
        ("709full", ColorSpace::Rec709Full),
        ("601limited", ColorSpace::Rec601Limited),
    ];

    for (cs_name, cs) in spaces {
        for (name, y, u, v) in bars {
            let frame = solid_yuv420p(16, 16, y, u, v, cs);
            let reference = yuv_to_rgba_reference(&frame);
            let out_tex = conv.convert(&gpu, &frame).unwrap();
            let gpu_out = download_rgba(&gpu, &out_tex).unwrap();
            // GPUのラスタライズ/量子化で±1は出うるので許容1
            assert_rgba_close(
                &format!("{cs_name}/{name}"),
                RgbaImageDesc {
                    width: frame.desc.width,
                    height: frame.desc.height,
                },
                &gpu_out,
                &reference,
                1,
            );
        }
    }
}

#[test]
fn color_space_changes_output() {
    // 同じYUV値でも601と709で異なるRGBになる(決め打ちの再発防止)
    let Some(gpu) = gpu_or_skip() else { return };
    let mut conv = YuvToRgba::new(&gpu);
    let red_709 = solid_yuv420p(16, 16, 63, 102, 240, ColorSpace::Rec709Limited);
    let red_601 = solid_yuv420p(16, 16, 63, 102, 240, ColorSpace::Rec601Limited);
    let out_709 = download_rgba(&gpu, &conv.convert(&gpu, &red_709).unwrap()).unwrap();
    let out_601 = download_rgba(&gpu, &conv.convert(&gpu, &red_601).unwrap()).unwrap();
    assert_ne!(
        out_709, out_601,
        "601/709で出力が同一 = 係数が無視されている"
    );
}

#[test]
fn upload_smoke() {
    let Some(gpu) = gpu_or_skip() else { return };
    use motolii_core::FrameDesc;
    let desc = FrameDesc::packed(17, 5, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut data = vec![0u8; desc.data_size()];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    let _tex = motolii_gpu::upload_rgba(&gpu, &desc, &data);
}
