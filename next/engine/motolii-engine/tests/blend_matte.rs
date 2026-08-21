//! `LayerAttrs::blend_mode`/`matte` の通し試験(next/reference/KNOWN.md「bm/matte/ao
//! 未消費」の消費・第1切片)。
//!
//! `motolii-store` は `ResolvedLayer` まで運ぶだけ(裁定108(e))で、これまで
//! `Engine::render_frame` はどちらも一切読んでいなかった — どんな値を書いても黙って
//! 無視されていた。ここでは:
//! - `BlendMode::Normal`(既定)は今まで通り描ける
//! - `Normal` 以外は `motolii-compositor` の固定 blend equation では表現できない
//!   (`motolii-compositor` のモジュール doc 参照)ので、**黙って Normal へ近似せず**
//!   `EngineError::UnsupportedBlendMode` を返す
//! - `matte` は shader 拡張(2枚目の texture を読む)が要る(fork seam 候補)ので、
//!   同じく**黙って型抜き前の絵を出さず** `EngineError::UnsupportedMatte` を返す

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

/// **落ちるテスト先行**: `Normal` 以外の blend mode は合成器がまだ表現できないので、
/// 黙って近似せず明示的に `Err` を返す(KNOWN.md に「未消費」と書かれていた穴を
/// 「読むが対応外は拒む」まで塞いだ、というのがこの束の主張)。
#[test]
fn unsupported_blend_modes_are_rejected_not_silently_approximated() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer, [255, 0, 0, 255], 0);

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Multiply),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let result = engine.render_frame(&doc.view(), t(0));
    assert!(
        matches!(
            result,
            Err(EngineError::UnsupportedBlendMode(BlendMode::Multiply))
        ),
        "Multiply は未対応のまま明示的に拒まれるはず: {result:?}"
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
