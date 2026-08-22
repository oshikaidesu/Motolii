//! `LayerAttrs::blend_mode`/`matte` の通し試験(next/reference/KNOWN.md「bm/matte/ao
//! 未消費」の消費・第1切片)。
//!
//! `motolii-store` は `ResolvedLayer` まで運ぶだけ(裁定108(e))で、これまで
//! `Engine::render_frame` はどちらも一切読んでいなかった — どんな値を書いても黙って
//! 無視されていた。ここでは:
//! - `BlendMode::Normal`(既定)は今まで通り描ける
//! - **BL3/BL4(2026-08-22)時点で `blend_mode` は17値全部が描ける**(分離可能11種+
//!   非分離4種、`motolii-compositor` のモジュール doc 参照)——対応外の値は
//!   もう無い
//! - `matte` は依然 `EngineError::UnsupportedMatte` を返す。**理由が変わった**:
//!   合成器側の matte 適用パス(`motolii_compositor::Compositor::matte_layer`)は
//!   BL4 で実装済みだが、`render_frame` のループが「matte 元 layer を通常描画から
//!   除外する」判定に要る `LayerId` 相関を store からまだ引けない
//!   (`EngineError::UnsupportedMatte` の doc 参照)——黙って型抜き前の絵を出さず、
//!   引き続き明示的に止める

use motolii_engine::{Engine, EngineError};
use motolii_store::{
    BlendMode, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
    LayerSource, LayerTiming, Matte, MatteMode, RationalTime,
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

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 60,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn place(doc: &mut Document, layer: LayerId, rgba: [u8; 4], order: i16) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba,
                width: W,
                height: H,
            },
            order,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

/// `Normal`(既定)はこれまで通り描ける — 消費を足しても既存の絵は変えない。
#[test]
fn normal_blend_mode_still_renders() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, [255, 0, 0, 255], 0);

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert!(
        pixel(&frame, 32, 32)[0] > 200,
        "赤が出ているはず: {:?}",
        pixel(&frame, 32, 32)
    );
}

/// `blend_mode` を明示的に `Normal` へ書いても(=既定値の書き直し)絵は変わらない。
#[test]
fn explicit_normal_blend_mode_matches_the_default() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, [255, 0, 0, 255], 0);

    let mut engine = Engine::new().expect("engine");
    let before = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Normal),
            ..Default::default()
        },
    })
    .unwrap();
    let after = engine.render_frame(&doc.view(), t(0)).unwrap();

    assert_eq!(before, after);
}

/// **BL4(2026-08-22)**: 非分離4種(Hue/Saturation/Color/Luminosity)も
/// `motolii-compositor` 側に実装が揃い、`translate_blend_mode` が全17値を `Ok` で
/// 写すようになった——KNOWN.md の「bm/matte/ao 未消費」の bm 側はこれで閉じた
/// (代表して `Hue` を使う。数値検証は `tests/blend_nonseparable.rs`)。
/// この時点で `motolii_store::BlendMode` に「合成器が拒む」値は無い——
/// `EngineError::UnsupportedBlendMode` は型として残るが、この関数からはもう
/// 構築されない。
#[test]
fn nonseparable_blend_mode_is_accepted_and_renders() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, [200, 60, 60, 255], 0);
    place(&mut doc, top, [50, 90, 200, 255], 1);

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Hue),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let result = engine.render_frame(&doc.view(), t(0));
    assert!(
        result.is_ok(),
        "Hue(非分離、BL4で対応済み)は拒まれてはいけない: {result:?}"
    );
}

/// **BL3**: 分離可能 blend(Multiply〜Exclusion)は `Add` と同じ経路で受け付けられる
/// (代表して Multiply — 数値の正しさは `tests/blend_separable.rs` が縛る)。
#[test]
fn separable_blend_mode_is_accepted_and_renders() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, [200, 200, 200, 255], 0);
    place(&mut doc, top, [200, 0, 0, 255], 1);

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Multiply),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let result = engine.render_frame(&doc.view(), t(0));
    assert!(
        result.is_ok(),
        "Multiply は対応外として拒まれてはいけない: {result:?}"
    );
}

/// **BL2**: `Add` は `Multiply` 等と違い明示的に受け付けられる(`multiplicative_tint.a
/// = 0` で無改造に出せる、`motolii-compositor` のモジュール doc 参照)。視覚 golden は
/// `motolii-compositor` の `tests/compose.rs::add_blend_*` 系が既に縛っているので、
/// ここでは「エラーにならず、Normal より明るく出る」ことだけ確かめる(重ねた2枚が
/// 加算されるので白い base の上に赤を足すと赤チャンネルが飽和する)。
#[test]
fn add_blend_mode_is_accepted_and_renders_brighter() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, [200, 0, 0, 255], 0);
    place(&mut doc, top, [200, 0, 0, 255], 1);

    let mut engine = Engine::new().expect("engine");
    let before = engine.render_frame(&doc.view(), t(0)).unwrap();

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Add),
            ..Default::default()
        },
    })
    .unwrap();
    let after = engine
        .render_frame(&doc.view(), t(0))
        .expect("Add は対応外として拒まれてはいけない");

    let before_red = pixel(&before, 32, 32)[0];
    let after_red = pixel(&after, 32, 32)[0];
    assert!(
        after_red >= before_red,
        "Add で2枚重ねたのに Normal より明るくなっていない: before={before_red} after={after_red}"
    );
}

/// matte もまだ合成器に繋いでいない(shader 拡張が要る、fork seam 候補) —
/// 黙って型抜き前の絵を出さず、明示的に `Err` を返す。
#[test]
fn matte_is_rejected_not_silently_ignored() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, [255, 255, 255, 255], 0);
    place(&mut doc, top, [255, 0, 0, 255], 1);

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: base,
                mode: MatteMode::Luma,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let result = engine.render_frame(&doc.view(), t(0));
    assert!(
        matches!(
            result,
            Err(EngineError::UnsupportedMatte(Matte {
                layer: reported,
                mode: MatteMode::Luma,
            })) if reported == base
        ),
        "matte 付き layer はまだ描けないので明示的に拒まれるはず: {result:?}"
    );
}
