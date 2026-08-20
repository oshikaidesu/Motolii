//! 裁定153 S3: store(`ResolvedLayer.effects`, S1)→ compositor(`EffectPass`, S2)の
//! 語彙変換配線。`translate_blend_mode` と同型の変換を1本足し、`Engine::render` の
//! 内部経路を `Compositor::render_with_effects` へ切り替える(S2 の
//! `LayerWithPasses`/`EffectPass::Identity` を消費する側)。
//!
//! **落ちるテスト先行**: この試験は S3 着手前は次のいずれかで赤だった:
//! - `Intent::SetEffects` を積んだ layer があっても engine 側が effect を一切読んでおらず
//!   (`ResolvedLayer.effects` を無視していた)、素通りしていた(=試験自体は書けても
//!   「経路が本当に render_with_effects を通る」ことを縛れなかった)
//!
//! 2026-08-21 時点で実 effect(shader pass)は1つも実装されていない
//! (`motolii_compositor::EffectPass` は `Identity` だけ、S4 が Glow 等を足す)。
//! つまり**全 plugin_id が未知**——ここで縛るのは「effect を持つ layer でも
//! フレームが壊れない・pass 無し layer と画素一致する(=未知 id は無音 skip)」こと。

use motolii_engine::Engine;
use motolii_store::{
    Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta,
    LayerSource, LayerTiming, RationalTime,
};

const W: u32 = 64;
const H: u32 = 64;

fn t(frame: i64) -> RationalTime {
    RationalTime::try_new(frame, 30).unwrap()
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}

fn doc_with_solid_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [10, 220, 130, 255],
                width: W,
                height: H,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
    (doc, layer)
}

/// **本命**: 未知 plugin_id の effect を積んだ layer は、effect 無し layer と
/// 画素一致するフレームを返す(=未知 id は無音 skip、パニックしない)。
/// 現時点で「対応している」plugin_id は1つも無い(`EffectPass::Identity` はまだ
/// どの id にも紐付いていない)ので、これは「今日 effect を積んでも壊れない」ことの
/// 直接固定になる。
#[test]
fn layer_with_unknown_effect_renders_identically_to_layer_without_effects() {
    let (doc_without, _) = doc_with_solid_layer();
    let mut engine = Engine::new().expect("engine");
    let without_effects = engine.render_frame(&doc_without.view(), t(0)).unwrap();

    let (mut doc_with, layer) = doc_with_solid_layer();
    doc_with
        .apply(Intent::SetEffects {
            layer,
            effects: vec![EffectInstance {
                id: EffectId(0),
                plugin_id: "motolii.not-yet-implemented".to_owned(),
                enabled: true,
            }],
        })
        .unwrap();
    let with_unknown_effect = engine.render_frame(&doc_with.view(), t(0)).unwrap();

    assert_eq!(
        without_effects, with_unknown_effect,
        "未知 plugin_id の effect は無音 skip のはず(pass を積まない)——画素が食い違った"
    );
    assert_eq!(
        pixel(&with_unknown_effect, 32, 32),
        [10, 220, 130, 255],
        "effect を積んだ layer の中身自体は普通に描けているはず: {:?}",
        pixel(&with_unknown_effect, 32, 32)
    );
}

/// disabled な effect は `resolve()` の時点で `ResolvedLayer.effects` に現れない
/// (`ResolvedEffect` の型 doc、`crate::effect` 参照)——engine 側の変換に届く前に
/// store が弾いていることの対比確認。上の試験と同じ絵になるはず。
#[test]
fn disabled_effect_does_not_reach_the_frame_either() {
    let (mut doc, layer) = doc_with_solid_layer();
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.not-yet-implemented".to_owned(),
            enabled: false,
        }],
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert_eq!(
        pixel(&frame, 32, 32),
        [10, 220, 130, 255],
        "disabled effect があっても普通に描けているはず: {:?}",
        pixel(&frame, 32, 32)
    );
}
