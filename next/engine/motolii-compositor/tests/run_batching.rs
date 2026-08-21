//! BL3 merge(`118cdbf4`)の構造退行の根治 — run-batching。
//!
//! `Compositor::accumulate_sequential` は現状、blend mode に関わらず layer 毎に
//! 新規 `ViewBuilder`→submit→blocking poll を払う。Normal/Add の連続区間(run)は
//! 1つの `ViewBuilder` に深度順 rect を積んで1回の submit で描けるはず——分離可能
//! blend(`separable_mode_index` が `Some`)の layer の出現点**だけ**で run を切る。
//!
//! oracle は同期点カウンタ `Compositor::sequential_submits`(試験専用
//! introspection、`effect_passes_created_textures` と同じ累計カウンタ規律)——
//! `accumulate_sequential` 内の `queue.submit` 呼び出し毎に増分する。

use motolii_compositor::{
    BlendMode, CompSpec, Compositor, LayerPlacement, LayerWithPasses, ResolvedCamera,
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

fn layer_with(
    compositor: &mut Compositor,
    label: &str,
    rgba: [u8; 4],
    order: i16,
    blend_mode: BlendMode,
) -> LayerWithPasses {
    let texture = compositor
        .upload_rgba(label, &solid(rgba, W, H), W, H)
        .unwrap();
    LayerWithPasses {
        layer: motolii_compositor::Layer {
            texture,
            size: [W as f32, H as f32],
            placement: placement(order),
            pinned: false,
            blend_mode,
        },
        passes: Vec::new(),
    }
}

/// ORACLE(a): all-Normal な3 layer は blend の切れ目が無い——1回の submit で
/// 描けるはず(旧単一パス・裁定171 のゼロコピー経路と構造的に同等)。
///
/// FIX 前は layer 毎に submit していたので **3** になり、この assert は赤で
/// 落ちる(red 出力を RETURN に記録)。FIX 後は run が1本(3 layer 丸ごと)なので
/// **1** になるはず。
#[test]
fn three_normal_layers_batch_into_a_single_submit() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let layers = vec![
        layer_with(&mut compositor, "l0", [255, 0, 0, 255], 0, BlendMode::Normal),
        layer_with(&mut compositor, "l1", [0, 255, 0, 200], 1, BlendMode::Normal),
        layer_with(&mut compositor, "l2", [0, 0, 255, 128], 2, BlendMode::Normal),
    ];

    let before = compositor.sequential_submits();
    let _ = compositor
        .render_to_texture(comp(), ResolvedCamera::default(), &layers)
        .unwrap();
    let after = compositor.sequential_submits();

    assert_eq!(
        after - before,
        1,
        "all-Normal 3 layer の accumulate_sequential は run が1本のみのはずなので \
         submit 増分は1のはず(layer 毎に submit するなら3になる — BL3 merge の退行)"
    );
}

/// ORACLE(b): [Normal, Multiply, Normal, Normal] — run は分離可能 blend
/// (`Multiply`)の出現点だけで切れる。
///
/// 式(実装から論証、`accumulate_sequential` の分岐に対応):
/// - run1 = [L0(Normal)] — L1(Multiply)の手前で切れる、単独の run でも
///   1 submit(background rect 無し・layer rect 1枚)。
/// - L1(Multiply) は分離可能パス: (1) layer 単体を自分の main_target へ描く
///   solo draw で1 submit、(2) 直前 accumulator(run1の結果)があるので
///   2枚読み blend pass でさらに1 submit。solo+blend = 2 submit。
/// - run2 = [L2(Normal), L3(Normal)] — 末尾まで続く連続区間、1つの
///   ViewBuilder に background rect(run1相当の直前 accumulator を敷く必要は
///   ここでは Multiply の blend 結果) + 2枚の layer rect をまとめて1 submit。
///
/// 合計 = run1(1) + separable solo(1) + blend pass(1) + run2(1) = **4**。
///
/// FIX 前(layer 毎に submit)は L0(1) + L1 solo+blend(2) + L2(1) + L3(1) = **5**
/// になり、この assert は赤で落ちる。
#[test]
fn a_separable_layer_splits_the_run_but_does_not_explode_submits() {
    let mut compositor = Compositor::headless().expect("headless GPU");

    let layers = vec![
        layer_with(&mut compositor, "l0", [255, 0, 0, 255], 0, BlendMode::Normal),
        layer_with(&mut compositor, "l1", [0, 255, 0, 255], 1, BlendMode::Multiply),
        layer_with(&mut compositor, "l2", [0, 0, 255, 200], 2, BlendMode::Normal),
        layer_with(&mut compositor, "l3", [10, 10, 10, 128], 3, BlendMode::Normal),
    ];

    let before = compositor.sequential_submits();
    let _ = compositor
        .render_to_texture(comp(), ResolvedCamera::default(), &layers)
        .unwrap();
    let after = compositor.sequential_submits();

    assert_eq!(
        after - before,
        4,
        "run2本(1+2)+separable solo 1+blend pass 1 の合計は4のはず \
         (layer 毎に submit するなら5になる — BL3 merge の退行、テスト doc の式参照)"
    );
}
