//! 重ねるソフトとしての器の形を見張る。**時間ではなく回数**を測る。
//!
//! 層ごとに submit して `poll(wait_indefinitely)` していた頃は、blend mode を
//! 使った層1枚につき GPU を2回止めていた(55層なら110回)。全面テクスチャも
//! 層ごとに新品を確保していた(1080p で 8.29MB × 110 = 912MB/フレーム)。
//! どちらも層数に比例する — 重ねるほど遅くなる形だった。

use motolii::render::compositor::{
    BlendMode, CompSpec, Compositor, HeadlessGpu, Layer, LayerPlacement, MatteMode, ResolvedCamera,
};

const W: u32 = 320;
const H: u32 = 180;

fn compositor() -> Compositor {
    let HeadlessGpu { adapter, device, queue } = HeadlessGpu::new().expect("headless GPU");
    drop(adapter);
    Compositor::with_device_using_headless_defaults(device, queue).expect("with_device")
}

fn layers(c: &mut Compositor, count: usize, blend: BlendMode) -> Vec<Layer> {
    let tex = c
        .upload_rgba("tile", &vec![128u8; (64 * 64 * 4) as usize], 64, 64)
        .expect("upload");
    (0..count)
        .map(|i| Layer {
            texture: tex.clone(),
            size: [64.0, 64.0],
            placement: LayerPlacement {
                transform: LayerPlacement::from_transform(
                    [i as f32, i as f32],
                    [0.0, 0.0],
                    [1.0, 1.0],
                    0.0,
                    0.0,
                    0.0,
                ),
                order: i as i16,
                opacity: 1.0,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
            },
            pinned: false,
            blend_mode: blend,
        })
        .collect()
}

fn submits_for(count: usize, blend: BlendMode) -> u64 {
    let mut c = compositor();
    let comp = CompSpec { width: W, height: H };
    let ls = layers(&mut c, count, blend);
    let before = c.sequential_submits();
    c.render_sequential(comp, ResolvedCamera::default(), &ls, [0.0, 0.0, 0.0, 1.0])
        .expect("render_sequential");
    c.sequential_submits() - before
}

#[test]
fn blend_layers_do_not_cost_a_submit_each() {
    let few = submits_for(2, BlendMode::Multiply);
    let many = submits_for(16, BlendMode::Multiply);
    assert_eq!(
        few, many,
        "blend mode の層数で submit 回数が変わっている(層ごとに GPU を止める形へ戻っている): \
         2層={few} 16層={many}"
    );
}

#[test]
fn a_normal_stack_costs_the_same_as_a_blended_stack() {
    let normal = submits_for(16, BlendMode::Normal);
    let multiply = submits_for(16, BlendMode::Multiply);
    assert_eq!(
        normal, multiply,
        "blend mode を使うだけで submit が増えている: Normal={normal} Multiply={multiply}"
    );
}

/// 上の2本が「どちらも0」で通ってしまわないための足場(眠る番人を作らない)。
#[test]
fn the_counter_actually_counts() {
    let submits = submits_for(16, BlendMode::Multiply);
    assert_eq!(
        submits, 1,
        "16層の合成が1回の submit で終わっていない(0なら計器が死んでいる): {submits}"
    );
}

/// matte も同じ形だった — 1枚につき3回(層の canvas・matte の canvas・matte パス)
/// GPU を止めていた。
#[test]
fn matte_layers_do_not_cost_a_submit_each() {
    let count_for = |n: usize| -> u64 {
        let mut c = compositor();
        let comp = CompSpec { width: W, height: H };
        let ls = layers(&mut c, n * 2, BlendMode::Normal);
        let before = c.sequential_submits();
        let matted: Vec<Layer> = (0..n)
            .map(|i| {
                c.matte_layer(
                    comp,
                    ResolvedCamera::default(),
                    &ls[i * 2],
                    &ls[i * 2 + 1],
                    MatteMode::Alpha,
                )
                .expect("matte_layer")
            })
            .collect();
        c.render_sequential(comp, ResolvedCamera::default(), &matted, [0.0, 0.0, 0.0, 1.0])
            .expect("render_sequential");
        c.sequential_submits() - before
    };
    let few = count_for(1);
    let many = count_for(8);
    assert_eq!(
        few, many,
        "matte の枚数で submit 回数が変わっている: 1枚={few} 8枚={many}"
    );
}
