//! layer 単位オフスクリーンパスの枠(裁定153 S2、2026-08-21)。
//!
//! `EffectPass::Identity` は絵を変えない — ここで縛るのは「枠そのものの正しさ」:
//! 1. Identity を積んだ layer の出力は、pass 無しの同じ layer と画素一致する。
//! 2. pass が空の layer は、オフスクリーン texture を一切作らずに従来経路と同じ絵を出す。
//! 3. 同じ形の layer を2回描いても、新規生成される texture は1枚のまま
//!    (フレームをまたいで使い回す — 毎フレーム作り直さない)。

use motolii_compositor::{
    BlendMode, CompSpec, Compositor, EffectPass, Layer, LayerPlacement, LayerWithPasses,
    ResolvedCamera,
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

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn comp() -> CompSpec {
    CompSpec {
        width: W,
        height: H,
    }
}

fn layer(texture: motolii_compositor::GpuTexture2D) -> Layer {
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

/// **落ちるテスト先行で確かめた枠の正しさ(1)**: Identity pass を積んだ layer の出力は
/// pass 無しの同じ layer と画素一致する。枠が無い状態はコンパイルエラーで赤だった
/// (`EffectPass`/`LayerWithPasses`/`render_with_effects` が存在しなかった) —
/// 実装後はここが green になる。
#[test]
fn identity_pass_output_matches_no_pass_output() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let checker = compositor
        .upload_rgba("checker", &solid([200, 40, 90, 180], W, H), W, H)
        .unwrap();

    let without_pass = compositor
        .render_with_effects(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: layer(checker.clone()),
                passes: vec![],
            }],
        )
        .unwrap();

    let with_identity = compositor
        .render_with_effects(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: layer(checker),
                passes: vec![EffectPass::Identity],
            }],
        )
        .unwrap();

    assert_eq!(
        without_pass, with_identity,
        "Identity pass は絵を変えないはず(枠の正しさ) — pass無し/pass有りで画素が食い違った"
    );
}

/// **落ちるテスト先行で確かめた枠の正しさ(2)**: pass 無し layer は `render_with_effects`
/// 経由でも `Compositor::render`(従来経路)と完全に同じ絵を出す ── かつオフスクリーンを
/// 一切作らない(`effect_passes_created_textures() == 0`)。
#[test]
fn passless_layer_matches_the_traditional_render_path_and_allocates_nothing() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let checker = compositor
        .upload_rgba("checker", &solid([10, 220, 130, 255], W, H), W, H)
        .unwrap();

    let traditional = compositor
        .render(comp(), ResolvedCamera::default(), &[layer(checker.clone())])
        .unwrap();

    assert_eq!(
        compositor.effect_passes_created_textures(),
        0,
        "render() 単体ではオフスクリーンを作らないはず(そもそも触らない経路)"
    );

    let via_new_entry = compositor
        .render_with_effects(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: layer(checker),
                passes: vec![],
            }],
        )
        .unwrap();

    assert_eq!(
        traditional, via_new_entry,
        "pass が空の layer は render_with_effects 経由でも従来経路と同じ絵になるはず"
    );
    assert_eq!(
        compositor.effect_passes_created_textures(),
        0,
        "pass が空の layer はオフスクリーン texture を一切作らないはず(コスト増ゼロ)"
    );
}

/// **落ちるテスト先行で確かめた枠の正しさ(3)**: pass を持つ layer は実際に
/// オフスクリーン texture を使う(=分岐が本当に効いている、pixel一致だけでは
/// 「たまたま同じ」を否定できないので生成回数でも縛る)。同じ形の layer をもう一度
/// 描いても新規生成は増えない(= プールで使い回している)。
#[test]
fn identity_pass_reuses_the_scratch_texture_across_frames() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let checker = compositor
        .upload_rgba("checker", &solid([90, 90, 200, 255], W, H), W, H)
        .unwrap();

    assert_eq!(compositor.effect_passes_created_textures(), 0);

    let _first = compositor
        .render_with_effects(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: layer(checker.clone()),
                passes: vec![EffectPass::Identity],
            }],
        )
        .unwrap();
    assert_eq!(
        compositor.effect_passes_created_textures(),
        1,
        "pass を持つ layer は少なくとも1枚のオフスクリーンを新規生成するはず"
    );

    let _second = compositor
        .render_with_effects(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: layer(checker),
                passes: vec![EffectPass::Identity],
            }],
        )
        .unwrap();
    assert_eq!(
        compositor.effect_passes_created_textures(),
        1,
        "同じサイズ/形式の2回目は前回のプールを再利用するはず(毎フレーム作り直さない)"
    );
}

/// 既存の alpha 経路(`Premultiplied` blend)を壊していないことの再確認 —
/// `render_with_effects` の pass 無し layer でも半透明 alpha が生き残ること。
#[test]
fn passless_layer_preserves_alpha_through_render_with_effects() {
    let mut compositor = Compositor::headless().expect("headless GPU");
    let half_alpha = compositor
        .upload_rgba("half-alpha", &solid([128, 0, 0, 128], W, H), W, H)
        .unwrap();

    let out = compositor
        .render_with_effects(
            comp(),
            ResolvedCamera::default(),
            &[LayerWithPasses {
                layer: layer(half_alpha),
                passes: vec![],
            }],
        )
        .unwrap();

    let px = pixel(&out, 32, 32);
    assert_eq!(
        px[3], 128,
        "pass無し layer は render_with_effects 経由でも alpha=128 のまま出るはず: {px:?}"
    );
}
