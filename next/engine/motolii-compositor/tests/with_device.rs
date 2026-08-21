//! `Compositor::with_device`(裁定170 M2、2026-08-21)のオラクル。
//!
//! **まだ誰も呼ばない**関数なので、ここでの唯一の主張は「[`Compositor::headless`]
//! と同じ絵が出る」——`headless()` は `with_device` の薄いラッパーへ書き換わった
//! だけで、adapter/device/queue の建て方(`HeadlessGpu`)も渡す format/config も
//! 従来のまま。別 device(≒別 `HeadlessGpu::new()` 呼び出し)を `with_device` へ
//! 渡しても、`tests/compose.rs` の `render_is_deterministic_across_devices` 相当の
//! 論法(別 device でも絵は変わらない)がそのまま効く。

use motolii_compositor::{
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, ResolvedCamera,
};

const W: u32 = 64;
const H: u32 = 64;

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

fn one_layer(texture: motolii_compositor::GpuTexture2D) -> Layer {
    Layer {
        texture,
        size: [W as f32, H as f32],
        placement: LayerPlacement {
            transform: LayerPlacement::from_transform(
                [0.0, 0.0],
                [0.0, 0.0],
                [1.0, 1.0],
                0.0,
                0.0,
                0.0,
            ),
            order: 0,
            opacity: 1.0,
            z: 0.0,
        },
        pinned: false,
        blend_mode: BlendMode::Normal,
    }
}

/// `Compositor::headless()` が呼んでいるのと**同じ** format/config を、呼び手側で
/// 自分で組み立てて `Compositor::with_device` へ渡す——`headless()` の内部を
/// 二重化しているのではなく、「headless() を外から見た時に他の呼び手が渡すはずの
/// 値そのもの」を渡している(将来 iced 側もこの形で呼ぶ、裁定170 の趣旨)。
fn with_device_like_headless(gpu: HeadlessGpu) -> Compositor {
    Compositor::with_device(
        &gpu.adapter,
        gpu.device,
        gpu.queue,
        re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
        |_caps| re_renderer::RenderConfig {
            msaa_mode: re_renderer::MsaaMode::Off,
        },
    )
    .expect("with_device")
}

#[test]
fn with_device_matches_headless() {
    let mut via_headless = Compositor::headless().expect("headless GPU");
    let red_a = via_headless
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();
    let frame_headless = via_headless
        .render(comp(), ResolvedCamera::default(), &[one_layer(red_a)])
        .unwrap();

    let gpu = HeadlessGpu::new().expect("headless GPU (手組み)");
    let mut via_with_device = with_device_like_headless(gpu);
    let red_b = via_with_device
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();
    let frame_with_device = via_with_device
        .render(comp(), ResolvedCamera::default(), &[one_layer(red_b)])
        .unwrap();

    assert_eq!(
        frame_headless, frame_with_device,
        "with_device で組んだ Compositor は headless() と同じ絵を出すはず\
         (headless() は with_device の薄いラッパーになった)"
    );
}

/// 3種の effect/timing 入口([`Compositor::render_with_timing`] は experiment 済みだが
/// `render` 自体を通しても)を `with_device` 経由でも普通に使えること——第二
/// コンストラクタを足しただけで `Compositor` の他のメソッドが壊れていないことの
/// 最小確認(裁定170 M2 の「配線ゼロ」は呼び出し側の話であって、この crate 内で
/// `with_device` が使い物にならないという意味ではない)。
#[test]
fn with_device_supports_render_with_timing() {
    let gpu = HeadlessGpu::new().expect("headless GPU");
    let mut compositor = with_device_like_headless(gpu);

    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();

    let (frame, timing) = compositor
        .render_with_timing(comp(), ResolvedCamera::default(), &[one_layer(red)])
        .unwrap();

    assert_eq!(frame.len(), (W * H * 4) as usize);
    assert!(
        timing.total_us() > 0,
        "内訳が全部ゼロなのは計測できていない"
    );
}
