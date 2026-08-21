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
/// **裁定161 BL1b で緑化**: BL1(裁定160)では `ViewBuilder::composite` しか
/// 使えず赤だった(`RenderContext::output_format_color()` に固定された
/// non-srgb-tagged format にしか描けず、layer 毎にガンマ round-trip を踏んで
/// いた)。fork へ `ViewBuilder::main_target()` read accessor を足し(BL1b)、
/// `render_sequential` を「per-layer は srgb-tagged accumulator への
/// blit-blend(GPU 自動 decode/encode 任せ、composite() 不使用)・最終変換は
/// 全 layer を重ね終えた後に1回だけ」という形へ書き換えた結果、この fixture は
/// バイト一致する(`src/lib.rs` の `Compositor::render_sequential` module doc
/// 参照)。plumbing の健全性(単一 layer・空 comp)は下の2試験が別途縛る。
#[test]
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
