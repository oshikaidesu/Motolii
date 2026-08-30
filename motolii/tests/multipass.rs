//! Vism の多段(ISF `PASSES`)が順番どおりに全部走っていること。
//! 「出力が変わったか」では段落ちを捕まえられないので、滲みの向きと量を刺す。

use motolii::render::compositor::{
    BlendMode, CompSpec, Compositor, EffectPass, HeadlessGpu, Layer, LayerPlacement,
    LayerWithPasses, ResolvedCamera,
};

const W: u32 = 64;
const H: u32 = 64;
const BOX: u32 = 16;

fn compositor() -> Compositor {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    Compositor::with_device_using_headless_defaults(device, queue).expect("with_device")
}

/// 真ん中に白い正方形を1つ置いて描く。
fn render(c: &mut Compositor, passes: Vec<EffectPass>) -> Vec<u8> {
    let tex = c
        .upload_rgba("box", &vec![255u8; (BOX * BOX * 4) as usize], BOX, BOX)
        .expect("upload");
    let layer = Layer {
        texture: tex,
        size: [BOX as f32, BOX as f32],
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
            rotation_x: 0.0,
            rotation_y: 0.0,
        },
        pinned: false,
        blend_mode: BlendMode::Normal,
    };
    c.render_with_effects(
        CompSpec { width: W, height: H },
        ResolvedCamera::default(),
        &[LayerWithPasses { layer, passes }],
        motolii::render::compositor::NO_BACKGROUND,
    )
    .expect("render_with_effects")
}

fn at(px: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

/// 形の外(層は左上なので右と下)の明るさの合計。滲みが無ければ 0。
fn halo(px: &[u8]) -> u32 {
    let mut sum = 0u32;
    for d in 0..3u32 {
        for t in 0..BOX {
            sum += at(px, t, BOX + d)[1] as u32; // 下
            sum += at(px, BOX + d, t)[1] as u32; // 右
        }
    }
    sum
}

#[test]
fn glow_spreads_light_outside_the_shape() {
    let mut c = compositor();
    let plain = render(&mut c, Vec::new());
    let glowing = render(
        &mut c,
        vec![EffectPass::Glow {
            threshold: 0.0,
            intensity: 2.0,
            radius: 3.0,
        }],
    );

    assert_eq!(halo(&plain), 0, "エフェクト無しで形の外が光っている(前提が違う)");
    assert!(
        halo(&glowing) > 0,
        "glow を積んでも形の外へ光が滲んでいない — 4段のどれかが走っていない"
    );
}

#[test]
fn glow_spreads_in_both_directions() {
    let mut c = compositor();
    let glowing = render(
        &mut c,
        vec![EffectPass::Glow {
            threshold: 0.0,
            intensity: 2.0,
            radius: 3.0,
        }],
    );
    let below = at(&glowing, BOX / 2, BOX + 1)[1];
    let right = at(&glowing, BOX + 1, BOX / 2)[1];
    assert!(
        below > 0 && right > 0,
        "縦横どちらかに滲んでいない — 横ぼかしと縦ぼかしの段が繋がっていない\
         (下={below} 右={right})"
    );
}
