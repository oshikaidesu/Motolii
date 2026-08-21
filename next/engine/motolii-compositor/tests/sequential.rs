//! BL1(裁定160・調査R9)— 逐次合成(accumulator)経路の枠。
//!
//! `Compositor::render_sequential` は layer を **1枚ずつ** オフスクリーン accumulator へ
//! 焼き込む。blend 式はまだ Normal 固定(式の分岐は BL3/BL4)— ここで縛るのは
//! 「1枚ずつ描いても、一括描画([`Compositor::render`])と同じ絵が出るか」だけ。
//!
//! 代表 fixture: 不透明な base + 半透明で重なる layer + pinned layer。

use motolii_compositor::{BlendMode, CompSpec, Compositor, Layer, LayerPlacement, ResolvedCamera};

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

fn placement(offset: [f32; 2], order: i16, opacity: f32) -> LayerPlacement {
    LayerPlacement {
        transform: LayerPlacement::from_transform(
            [0.0, 0.0],
            offset,
            [1.0, 1.0],
            0.0,
            0.0,
            0.0,
        ),
        order,
        opacity,
        z: 0.0,
    }
}

/// ORACLE(b): 代表 fixture(2 layer 重なり・alpha 半透明・pinned 混在)で
/// `render` と `render_sequential` がバイト一致すること。
///
/// **実測で赤(EVIDENCE_GAP)**: `render_sequential` のモジュール doc(`src/lib.rs`)が
/// 理由を書いている——`ViewBuilder::composite` の render target format が
/// `RenderContext::output_format_color()` に固定されていて srgb-tagged にできず、
/// GPU 自動 srgb decode/encode(= linear 空間の alpha-over)を accumulator へ
/// 持ち込めない。実測: 4096 画素中 1264 画素が不一致、最大チャンネル差49
/// (`cargo test -p motolii-compositor --test sequential -- --nocapture` で再現)。
/// `ignore`: このオラクルは「fork 手術なしには緑にできない」という**成果**を
/// 固定するために残す(裁定160 BL1「止まる許可」)。plumbing 自体
/// (単一 layer・空 comp)が壊れていないことは下の2試験が緑で縛っている。
#[test]
#[ignore = "EVIDENCE_GAP: composite() は non-srgb-tagged 固定 format にしか描けず、\
            accumulator へ linear 空間の over を持ち込めない(src/lib.rs の render_sequential doc 参照)。\
            fork 側の ViewBuilder に main_target アクセサを足すか、rectangles.rs 相当を複製しない限り緑にならない。"]
fn sequential_matches_render_for_overlapping_alpha_and_pinned_fixture() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let base = compositor
        .upload_rgba("base", &solid([200, 40, 90, 255], W, H), W, H)
        .unwrap();
    let overlay = compositor
        .upload_rgba("overlay", &solid([20, 180, 60, 128], 32, 32), 32, 32)
        .unwrap();
    let pin = compositor
        .upload_rgba("pin", &solid([10, 10, 220, 200], 16, 16), 16, 16)
        .unwrap();

    let layers = vec![
        Layer {
            texture: base,
            size: [W as f32, H as f32],
            placement: placement([0.0, 0.0], 0, 1.0),
            pinned: false,
            blend_mode: BlendMode::Normal,
        },
        Layer {
            texture: overlay,
            size: [32.0, 32.0],
            placement: placement([16.0, 16.0], 1, 0.75),
            pinned: false,
            blend_mode: BlendMode::Normal,
        },
        Layer {
            texture: pin,
            size: [16.0, 16.0],
            placement: placement([4.0, 4.0], 2, 1.0),
            pinned: true,
            blend_mode: BlendMode::Normal,
        },
    ];

    let expected = compositor
        .render(comp(), ResolvedCamera::default(), &layers)
        .unwrap();
    let actual = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &layers)
        .unwrap();

    assert_eq!(
        expected.len(),
        actual.len(),
        "render_sequential の出力サイズが render と違う"
    );

    if expected != actual {
        let mut diffs = 0usize;
        let mut max_delta = 0i32;
        let mut sample: Option<(u32, u32, [u8; 4], [u8; 4])> = None;
        for y in 0..H {
            for x in 0..W {
                let i = ((y * W + x) * 4) as usize;
                let e = [expected[i], expected[i + 1], expected[i + 2], expected[i + 3]];
                let a = [actual[i], actual[i + 1], actual[i + 2], actual[i + 3]];
                if e != a {
                    diffs += 1;
                    for c in 0..4 {
                        let d = (e[c] as i32 - a[c] as i32).abs();
                        if d > max_delta {
                            max_delta = d;
                        }
                    }
                    if sample.is_none() {
                        sample = Some((x, y, e, a));
                    }
                }
            }
        }
        eprintln!(
            "EVIDENCE: {diffs}/{total} pixels differ, max per-channel delta={max_delta}, \
             first diff at {sample:?}",
            total = W * H
        );
    }

    assert_eq!(
        expected, actual,
        "render_sequential(逐次 accumulator)は render(一括)とバイト一致するはず"
    );
}

/// 単純化した回帰: layer 1枚だけなら重なりが無いので、精度の議論を持ち込まず
/// 「入口として正しく動くか」だけを縛る。
#[test]
fn sequential_matches_render_for_a_single_opaque_layer() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let red = compositor
        .upload_rgba("red", &solid([255, 0, 0, 255], W, H), W, H)
        .unwrap();

    let layers = vec![Layer {
        texture: red,
        size: [W as f32, H as f32],
        placement: placement([0.0, 0.0], 0, 1.0),
        pinned: false,
        blend_mode: BlendMode::Normal,
    }];

    let expected = compositor
        .render(comp(), ResolvedCamera::default(), &layers)
        .unwrap();
    let actual = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &layers)
        .unwrap();

    assert_eq!(expected, actual, "layer 1枚だけの場合はバイト一致するはず");
}

/// layer が無い comp は `render` と同じ clear 色になるはず。
#[test]
fn sequential_matches_render_for_empty_comp() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let expected = compositor
        .render(comp(), ResolvedCamera::default(), &[])
        .unwrap();
    let actual = compositor
        .render_sequential(comp(), ResolvedCamera::default(), &[])
        .unwrap();
    assert_eq!(expected, actual);
}
