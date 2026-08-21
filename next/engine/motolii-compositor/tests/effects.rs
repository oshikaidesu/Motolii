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

/// 内蔵 vism 第1号(裁定153 S4、2026-08-21)。移植元:
/// `spikes/m5-known-implementation/M5-R0/src/glow.rs`(bright-pass→水平blur→垂直blur→
/// 加算合成)。ここで縛るのは M5 proof が確かめていた3性質を、fixture 依存を剥がした
/// 製品経路(`Compositor::render_with_effects`)の上で再確認すること — oracle (b)/(c)/(d)。
mod glow {
    use super::*;

    /// **write-set 外の発見**(縫い目調査5節には無かった、実装中に見つけた fixture
    /// 依存): proof の composite は 1.0 超えを clamp しない(readback が直接
    /// half-float を読むので、そのまま観測して確かめている)。製品側は composite
    /// 結果が通常合成(`re_renderer::renderer::rectangle_fs.wgsl` の `decode_color`)
    /// へ戻り、そこは `decode_srgb` 有効時に `[0,1]` を外れた値を「壊れたデータ」と
    /// 見なして `ERROR_RGBA`(マゼンタ `[255,0,255,255]`)へ差し替える固定の検査を
    /// 持っている——`Compositor::render_with_effects` の他の layer(裸の
    /// `Rgba8Unorm`、値は常に `[0,1]` 内)はこの検査に触れたことが無かったので
    /// S2 まででは表面化しなかった。composite_fs の出力を明示的に clamp して
    /// 「1.0超えは白に飽和する」という、このリポジトリの既存の加算合成
    /// (`tests/compose.rs::add_blend_saturates_to_white_when_the_linear_sum_exceeds_one`)
    /// と同じ規約に揃えたので、飽和先はマゼンタではなく白であることを縛る。
    #[test]
    fn composite_overflow_saturates_to_white_not_the_error_color() {
        let mut compositor = Compositor::headless().expect("headless GPU");
        let bright_gray = compositor
            .upload_rgba("bright-gray", &solid([200, 200, 200, 255], W, H), W, H)
            .unwrap();
        let mut gray_layer = layer(bright_gray);
        gray_layer.placement.order = 0;

        // このパラメータの組では合成前の線形値が 1.0 を大きく超える
        // (0.784 + 0.784*(0.784-0.3)/0.784 = 0.784+0.484 ≈ 1.27)。
        let out = compositor
            .render_with_effects(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: gray_layer,
                    passes: vec![EffectPass::Glow {
                        threshold: 0.3,
                        intensity: 1.0,
                        radius: 1.0,
                    }],
                }],
            )
            .unwrap();

        let px = pixel(&out, W / 2, H / 2);
        assert_eq!(
            px,
            [255, 255, 255, 255],
            "1.0超えは ERROR_RGBA([255,0,255,255])ではなく白へ飽和するはず: {px:?}"
        );
    }

    fn bright_point(w: u32, h: u32) -> Vec<u8> {
        let mut pixels = solid([0, 0, 0, 255], w, h);
        let (cx, cy) = (w / 2, h / 2);
        let center = ((cy * w + cx) * 4) as usize;
        pixels[center..center + 4].copy_from_slice(&[255, 255, 255, 255]);
        pixels
    }

    /// **落ちるテスト先行 → oracle (b)**: 明るい中心画素の周囲に加算 halo が出る。
    /// pixel一致だけでは「たまたま」を否定できないので、baseline(黒のまま)と
    /// glow有り(明るくなっているはず)を数値で対比する — proof の
    /// `glow_expands_alpha_only_within_the_declared_two_pixel_radius` と同じ着眼、
    /// ただし製品経路(`render_with_effects`)の上で確認する。
    #[test]
    fn brightens_pixels_neighboring_a_bright_point() {
        let mut compositor = Compositor::headless().expect("headless GPU");
        let texture = compositor
            .upload_rgba("bright-point", &bright_point(W, H), W, H)
            .unwrap();

        let without_glow = compositor
            .render_with_effects(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: layer(texture.clone()),
                    passes: vec![],
                }],
            )
            .unwrap();

        let with_glow = compositor
            .render_with_effects(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: layer(texture),
                    passes: vec![EffectPass::Glow {
                        threshold: 0.5,
                        intensity: 1.0,
                        radius: 1.0,
                    }],
                }],
            )
            .unwrap();

        // 中心から1画素隣。5-tap blur は既定 radius=1.0 で ±2texel まで届くので
        // 中心に隣接する画素は必ず bloom を受け取る(proof の `blur_at` 参照)。
        let (cx, cy) = (W / 2, H / 2);
        let base = pixel(&without_glow, cx + 1, cy);
        let glowed = pixel(&with_glow, cx + 1, cy);
        assert_eq!(base, [0, 0, 0, 255], "baseline はまだ黒いはず: {base:?}");
        assert!(
            glowed[0] > base[0] && glowed[1] > base[1] && glowed[2] > base[2],
            "glow 有りだと中心近傍が明るくなるはず(halo): base={base:?} glowed={glowed:?}"
        );
    }

    /// **落ちるテスト先行 → oracle (c)**: glow を積んでも既存の alpha 経路
    /// (`tests/compose.rs::alpha_survives_the_composite_step` 系)を壊さない ——
    /// 閾値に届かない(暗い)半透明 layer は alpha がそのまま出る
    /// (`passless_layer_preserves_alpha_through_render_with_effects` と同型の確認)。
    #[test]
    fn preserves_alpha_when_nothing_crosses_the_threshold() {
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
                    passes: vec![EffectPass::Glow {
                        threshold: 1.0,
                        intensity: 0.75,
                        radius: 1.0,
                    }],
                }],
            )
            .unwrap();

        let px = pixel(&out, 32, 32);
        assert_eq!(
            px[3], 128,
            "暗い(閾値未満の)layer は glow を積んでも alpha=128 のまま出るはず: {px:?}"
        );
    }

    /// **落ちるテスト先行 → oracle (d)**: `intensity: 0` は実質 pass-through。
    /// bright-pass/blur 自体は回る(明るい画素は依然として閾値を超える)が、
    /// composite の `glow * intensity` が 0 になるので画素は pass 無しと一致するはず。
    #[test]
    fn zero_intensity_matches_the_source_exactly() {
        let mut compositor = Compositor::headless().expect("headless GPU");
        let texture = compositor
            .upload_rgba("bright-point-zero-intensity", &bright_point(W, H), W, H)
            .unwrap();

        let without_glow = compositor
            .render_with_effects(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: layer(texture.clone()),
                    passes: vec![],
                }],
            )
            .unwrap();

        let with_zero_intensity = compositor
            .render_with_effects(
                comp(),
                ResolvedCamera::default(),
                &[LayerWithPasses {
                    layer: layer(texture),
                    passes: vec![EffectPass::Glow {
                        threshold: 0.5,
                        intensity: 0.0,
                        radius: 1.0,
                    }],
                }],
            )
            .unwrap();

        assert_eq!(
            without_glow, with_zero_intensity,
            "intensity=0 は実質 pass-through のはず — pass無しと画素一致するはず"
        );
    }
}
