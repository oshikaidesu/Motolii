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
    Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, Keyframe,
    KeyframeTrack, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
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

/// 静止値の param track を1本立てる(1keyframe・Hold — 時間で動かない静止設定を
/// animatable track に乗せる時の最小形、裁定72)。
fn set_static_param(doc: &mut Document, layer: LayerId, effect: EffectId, name: &str, value: f64) {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::F64(value),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_param(effect, name).unwrap(),
        track,
    })
    .unwrap();
}

/// **落ちるテスト先行 → oracle (a)**: `"motolii.glow"` を積んだ layer の出力は
/// pass 無しと**異なる**。S3 コミット時点では `translate_effect_passes` が
/// 常に `None` を返していたので赤だった(同じ絵になっていた) — S4 で
/// `"motolii.glow"` → `EffectPass::Glow` の対応表エントリが増えたので緑になる。
/// 中間灰色(閾値未満だと絵が変わらない)を使い、`threshold` を明示的に
/// 低く設定して bright-pass を確実に起動させる(既定 `threshold=1.0` だと
/// 8bit SDR 由来の layer は luminance が1.0を超えないため常に無反応 — これは
/// 意図した挙動であって不具合ではない、`GLOW_DEFAULT_THRESHOLD` の doc 参照)。
#[test]
fn layer_with_known_glow_effect_renders_differently_from_layer_without_effects() {
    let mid_gray = || LayerMeta {
        source: LayerSource::Solid {
            rgba: [200, 200, 200, 255],
            width: W,
            height: H,
        },
        order: 0,
        timing: LayerTiming::place(0, None, 100_000),
    };

    let mut doc_without = Document::new();
    doc_without
        .apply(Intent::SetComposition(Composition {
            width: W,
            height: H,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
    let layer_a = LayerId(1);
    doc_without.apply(Intent::AddLayer(layer_a)).unwrap();
    doc_without
        .apply(Intent::SetMeta {
            layer: layer_a,
            meta: mid_gray(),
        })
        .unwrap();
    let mut engine = Engine::new().expect("engine");
    let without_effects = engine.render_frame(&doc_without.view(), t(0)).unwrap();

    let mut doc_with = Document::new();
    doc_with
        .apply(Intent::SetComposition(Composition {
            width: W,
            height: H,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 60,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
    let layer_b = LayerId(1);
    doc_with.apply(Intent::AddLayer(layer_b)).unwrap();
    doc_with
        .apply(Intent::SetMeta {
            layer: layer_b,
            meta: mid_gray(),
        })
        .unwrap();
    let effect = EffectId(0);
    doc_with
        .apply(Intent::SetEffects {
            layer: layer_b,
            effects: vec![EffectInstance {
                id: effect,
                plugin_id: "motolii.glow".to_owned(),
            }],
        })
        .unwrap();
    set_static_param(&mut doc_with, layer_b, effect, "threshold", 0.3);
    set_static_param(&mut doc_with, layer_b, effect, "intensity", 1.0);
    let with_glow = engine.render_frame(&doc_with.view(), t(0)).unwrap();

    assert_ne!(
        without_effects, with_glow,
        "既知 plugin_id \"motolii.glow\" は pass を積むはずなので画が変わるはず"
    );
    let (cx, cy) = (W / 2, H / 2);
    let base = pixel(&without_effects, cx, cy);
    let glowed = pixel(&with_glow, cx, cy);
    assert!(
        glowed[0] >= base[0] && glowed[1] >= base[1] && glowed[2] >= base[2],
        "glow は明るくする方向のはず(暗くはならない): base={base:?} glowed={glowed:?}"
    );
}

/// **store 経由の統合1本**(EXACT TARGET の ORACLE 節): Document に effect+param
/// track を積み、`Engine::render_frame` を時刻違いで2回呼ぶと param アニメが
/// 画素差になる。`intensity` を 0フレームで 0.0、30フレームで 1.0 へアニメさせる
/// (`threshold` は静止 0.3 で bright-pass を常に起動させておく) — S1(resolve が
/// param track を評価する)→ S3(engine が名前つき param を読む)→ S4(shader が
/// intensity を実際に使う)の**縦の1本**が絵まで届くことを縛る。
#[test]
fn glow_intensity_animation_changes_the_frame_over_time() {
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
                rgba: [200, 200, 200, 255],
                width: W,
                height: H,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();

    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.glow".to_owned(),
        }],
    })
    .unwrap();
    set_static_param(&mut doc, layer, effect, "threshold", 0.3);

    let mut intensity = KeyframeTrack::new();
    intensity.insert(Keyframe {
        t: t(0),
        value: Value::F64(0.0),
        interp: Interp::Linear,
        spatial: None,
    });
    intensity.insert(Keyframe {
        t: t(30),
        value: Value::F64(1.0),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_param(effect, "intensity").unwrap(),
        track: intensity,
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let at_start = engine.render_frame(&doc.view(), t(0)).unwrap();
    let at_end = engine.render_frame(&doc.view(), t(30)).unwrap();

    assert_ne!(
        at_start, at_end,
        "intensity のアニメが画素まで届いているはず(0フレームと30フレームが同じ絵ではだめ)"
    );
    let (cx, cy) = (W / 2, H / 2);
    let start_px = pixel(&at_start, cx, cy);
    let end_px = pixel(&at_end, cx, cy);
    assert!(
        end_px[0] >= start_px[0] && end_px[1] >= start_px[1] && end_px[2] >= start_px[2],
        "intensity=1 の30フレーム目は intensity=0 の0フレーム目より明るい(か同じ)はず: \
         start={start_px:?} end={end_px:?}"
    );
}

/// disabled な effect は `resolve()` の時点で `ResolvedLayer.effects` に現れない
/// (`ResolvedEffect` の型 doc、`crate::effect` 参照)——engine 側の変換に届く前に
/// store が弾いていることの対比確認。上の試験と同じ絵になるはず。
///
/// **裁定213**: `enabled` はもう `EffectInstance` の静止フィールドではなく
/// `effect.{id}.enabled` の track(`Value::Bool`、Hold 補間)——disable するには
/// `Intent::SetTrack` でその track へ `false` を書く(`PropertyId::effect_enabled`
/// doc 参照)。
#[test]
fn disabled_effect_does_not_reach_the_frame_either() {
    let (mut doc, layer) = doc_with_solid_layer();
    let effect = EffectId(0);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: effect,
            plugin_id: "motolii.not-yet-implemented".to_owned(),
        }],
    })
    .unwrap();
    let mut disabled_track = KeyframeTrack::new();
    disabled_track.insert(Keyframe {
        t: t(0),
        value: Value::Bool(false),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::effect_enabled(effect),
        track: disabled_track,
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
