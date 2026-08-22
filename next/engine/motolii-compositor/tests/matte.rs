//! BL4 track matte(`Compositor::matte_layer`)の数値検証。
//!
//! `motolii-engine` の `blend_matte.rs` は matte をまだ store→engine の
//! `LayerId` 相関の欠落で `Err` にする(`EngineError::UnsupportedMatte` doc 参照)ので、
//! matte の WGSL パス自体の数値検証は**この compositor レベル**で行う——
//! `Compositor::upload_rgba`/`Compositor::matte_layer`/`Compositor::render_sequential`
//! を直接呼び、store/Document を経由しない(`tests/compose.rs`/`tests/run_batching.rs`
//! と同じ「Layer を手で組み立てる」流儀)。
//!
//! **coverage(0〜1のスカラー)は出力の alpha チャンネルへそのまま乗る**
//! (`matte` モジュール doc「4モードの式」: `out = layer * coverage`、alpha は
//! premultiplication の影響を受けずどのフォーマットでも gamma 補正されない)ので、
//! このテストは主に**alpha チャンネル**を読んで coverage の正しさを検証する——
//! RGB は sRGB⇄linear の round-trip を挟む(`blend` モジュール doc「色空間の注意」と
//! 同じ前提)ので、色相の傾向(「matte 元の色が出力に漏れていないか」)だけを
//! 不等号で確認する。

use motolii_compositor::{
    BlendMode, CompSpec, Compositor, Layer, LayerPlacement, MatteMode, ResolvedCamera,
};

const W: u32 = 64;
const H: u32 = 64;
/// 8bit 量子化 + GPU sRGB ハードウェア変換近似の誤差を吸収する許容差(LSB)。
/// alpha は gamma 変換を経ないので、本来はもっと厳しい許容差で足りるはずだが、
/// `blend_separable.rs` の `TOLERANCE` と揃えて安全側に倒す。
const TOLERANCE: i32 = 4;

fn solid(rgba: [u8; 4], w: u32, h: u32) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((w * h * 4) as usize)
        .collect()
}

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

fn placement(order: i16) -> LayerPlacement {
    LayerPlacement {
        transform: LayerPlacement::from_transform(
            [0.0, 0.0],
            [0.0, 0.0],
            [1.0, 1.0],
            0.0,
            0.0,
            0.0,
        ),
        order,
        opacity: 1.0,
        z: 0.0,
    }
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn layer(compositor: &mut Compositor, label: &str, rgba: [u8; 4], order: i16) -> Layer {
    let texture = compositor
        .upload_rgba(label, &solid(rgba, W, H), W, H)
        .unwrap();
    Layer {
        texture,
        size: [W as f32, H as f32],
        placement: placement(order),
        pinned: false,
        blend_mode: BlendMode::Normal,
    }
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// **ORACLE(Alpha)**: coverage = matte 元の alpha そのまま。matte 元の色(緑)は
/// target(赤)の見た目には一切影響しない——alpha だけが効くことの直接固定。
#[test]
fn alpha_mode_scales_output_alpha_by_matte_alpha() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let target = layer(&mut compositor, "target", [255, 0, 0, 255], 0);
    let matte_source = layer(&mut compositor, "matte", [0, 255, 0, 128], 1);

    let matted = compositor
        .matte_layer(comp(), ResolvedCamera::default(), &target, &matte_source, MatteMode::Alpha)
        .expect("matte_layer(Alpha) が失敗した");

    let frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[matted])
        .unwrap();
    let px = pixel(&frame, W / 2, H / 2);

    assert!(
        (px[3] as i32 - 128).abs() <= TOLERANCE,
        "Alpha matte の coverage(=出力 alpha)は matte 元の alpha(128)そのままのはず: {px:?}"
    );
    assert!(
        px[0] > px[2],
        "target(赤)の色相が出ているはず、matte 元(緑)の色が漏れていないか: {px:?}"
    );
}

/// **ORACLE(InvertedAlpha)**: coverage = 1 - matte 元の alpha。
#[test]
fn inverted_alpha_mode_inverts_coverage() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let target = layer(&mut compositor, "target", [255, 0, 0, 255], 0);
    let matte_source = layer(&mut compositor, "matte", [0, 255, 0, 64], 1);

    let matted = compositor
        .matte_layer(
            comp(),
            ResolvedCamera::default(),
            &target,
            &matte_source,
            MatteMode::InvertedAlpha,
        )
        .expect("matte_layer(InvertedAlpha) が失敗した");

    let frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[matted])
        .unwrap();
    let px = pixel(&frame, W / 2, H / 2);

    let expected = 255 - 64;
    assert!(
        (px[3] as i32 - expected).abs() <= TOLERANCE,
        "InvertedAlpha matte の coverage は 1-alpha(=255-64={expected})のはず: {px:?}"
    );
}

/// **ORACLE(Luma)**: 灰色 matte(R=G=B)なら、Rec.709 の重み(合計1.0)は自明に効かず
/// coverage = matte の輝度(sRGB→linear)そのものになる——このテストが独立に計算した
/// `srgb_to_linear` の値と突き合わせる。
#[test]
fn luma_mode_scales_by_matte_luminance() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    const GRAY: u8 = 200;
    let target = layer(&mut compositor, "target", [255, 0, 0, 255], 0);
    let matte_source = layer(&mut compositor, "matte", [GRAY, GRAY, GRAY, 255], 1);

    let matted = compositor
        .matte_layer(comp(), ResolvedCamera::default(), &target, &matte_source, MatteMode::Luma)
        .expect("matte_layer(Luma) が失敗した");

    let frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[matted])
        .unwrap();
    let px = pixel(&frame, W / 2, H / 2);

    let expected_linear = srgb_to_linear(GRAY as f64 / 255.0);
    let expected = (expected_linear * 255.0).round() as i32;
    assert!(
        (px[3] as i32 - expected).abs() <= TOLERANCE,
        "Luma matte(灰色 matte 元)の coverage は輝度(sRGB→linear)のはず: got={} expected≈{expected} px={px:?}",
        px[3]
    );
}

/// **ORACLE(InvertedLuma の既知の癖)**: matte 元が完全に透明(alpha=0)な領域は
/// premultiplied rgb が (0,0,0) になるので Luma coverage は 0 —— InvertedLuma では
/// 逆に coverage が 1(完全露出)になる。これは AE でドキュメント化された挙動
/// (`matte` モジュール doc「4モードの式」参照)であり、バグではなく仕様として
/// そのまま固定する。
#[test]
fn inverted_luma_reveals_fully_transparent_matte_regions() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let target = layer(&mut compositor, "target", [255, 0, 0, 255], 0);
    // 色は白だが alpha=0(完全透明)。
    let matte_source = layer(&mut compositor, "matte", [255, 255, 255, 0], 1);

    let luma = compositor
        .matte_layer(comp(), ResolvedCamera::default(), &target, &matte_source, MatteMode::Luma)
        .expect("matte_layer(Luma) が失敗した");
    let luma_frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[luma])
        .unwrap();
    let luma_px = pixel(&luma_frame, W / 2, H / 2);

    let inverted = compositor
        .matte_layer(
            comp(),
            ResolvedCamera::default(),
            &target,
            &matte_source,
            MatteMode::InvertedLuma,
        )
        .expect("matte_layer(InvertedLuma) が失敗した");
    let inverted_frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[inverted])
        .unwrap();
    let inverted_px = pixel(&inverted_frame, W / 2, H / 2);

    assert!(
        luma_px[3] <= TOLERANCE as u8,
        "透明な matte 元は premultiplied rgb=0 なので Luma coverage はほぼ0のはず: {luma_px:?}"
    );
    assert!(
        inverted_px[3] as i32 >= 255 - TOLERANCE,
        "InvertedLuma は透明領域を「黒」として反転するので coverage はほぼ1(完全露出)の\
         はず(AE の既知の癖、matte モジュール doc 参照): {inverted_px:?}"
    );
}

/// **決定性**。同じ入力から同じ絵が出ること(`tests/compose.rs::render_is_deterministic`
/// と同型)。
#[test]
fn matte_layer_is_deterministic() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let target = layer(&mut compositor, "target", [200, 50, 50, 255], 0);
    let matte_source = layer(&mut compositor, "matte", [10, 10, 10, 180], 1);

    let first = compositor
        .matte_layer(comp(), ResolvedCamera::default(), &target, &matte_source, MatteMode::Alpha)
        .unwrap();
    let first_frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[first])
        .unwrap();

    let second = compositor
        .matte_layer(comp(), ResolvedCamera::default(), &target, &matte_source, MatteMode::Alpha)
        .unwrap();
    let second_frame = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[second])
        .unwrap();

    assert_eq!(
        first_frame, second_frame,
        "同じ target/matte_source/mode から違う絵が出る(決定的でない)"
    );
}
