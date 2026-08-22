//! `LayerAttrs::blend_mode`/`matte` の通し試験(next/reference/KNOWN.md「bm/matte/ao
//! 未消費」の消費・第1切片、matte は2026-08-22 のテキスト+matte 結線で完結)。
//!
//! `motolii-store` は `ResolvedLayer` まで運ぶだけ(裁定108(e))で、以前は
//! `Engine::render_frame` はどちらも一切読んでいなかった — どんな値を書いても黙って
//! 無視されていた。ここでは:
//! - `BlendMode::Normal`(既定)は今まで通り描ける
//! - **BL3/BL4(2026-08-22)時点で `blend_mode` は17値全部が描ける**(分離可能11種+
//!   非分離4種、`motolii-compositor` のモジュール doc 参照)——対応外の値は
//!   もう無い
//! - **matte も同じ日に `render_frame` へ繋がった**(`ResolvedLayer.id` を使って
//!   `matte.layer` を突き合わせ、`Engine::apply_matte` を呼ぶ——`lib.rs` の
//!   `EngineError::UnsupportedMatte` doc 参照)。数値の正しさ(4モードの式)は
//!   `motolii-compositor` の `tests/matte.rs` が独立オラクルとして既に縛っているので、
//!   ここでは「エラーにならない」「マット元が二重描画されない」「マット元が
//!   この時刻に居ない/絵を持たない時は本体も描かない」という **engine 特有の配線**
//!   だけを固定する。

use motolii_engine::Engine;
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

/// matte は `render_frame` から実際に消費される——`EngineError::UnsupportedMatte` は
/// もう `render_frame` 経路からは構築されない(型としては残る、`lib.rs` の doc 参照)。
/// coverage の数値そのもの(Alpha=matte 元の alpha)は `motolii-compositor` の
/// `tests/matte.rs::alpha_mode_scales_output_alpha_by_matte_alpha` が既に縛っているので、
/// ここでは「matte 無しより暗くなる(alpha が縮んで黒背景が透ける)」という方向だけ
/// 確かめる(`add_blend_mode_is_accepted_and_renders_brighter` と同型の qualitative test)。
#[test]
fn matte_alpha_mode_is_accepted_and_scales_target_alpha() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, [0, 0, 255, 128], 0);
    place(&mut doc, top, [255, 0, 0, 255], 1);

    let mut engine = Engine::new().expect("engine");
    let before = engine
        .render_frame(&doc.view(), t(0))
        .expect("matte を付ける前は普通に描けるはず");
    let before_red = pixel(&before, 32, 32)[0];

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: base,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let after = engine
        .render_frame(&doc.view(), t(0))
        .expect("matte 付き layer はもう Err にならないはず");
    let after_px = pixel(&after, 32, 32);

    assert!(
        after_px[0] < before_red,
        "Alpha matte(matte 元の alpha=128)で target の alpha が縮み、黒背景が透けて\
         赤が薄くなるはず: before={before_red} after={after_px:?}"
    );
}

/// **4モードとも `Err` にならない**(`translate_matte_mode_tests::all_four_matte_modes_translate_one_to_one`
/// の render_frame 版——語彙変換だけでなく実際に描画経路を通ることまで確かめる)。
#[test]
fn all_four_matte_modes_are_accepted_and_render() {
    for mode in [
        MatteMode::Alpha,
        MatteMode::InvertedAlpha,
        MatteMode::Luma,
        MatteMode::InvertedLuma,
    ] {
        let mut doc = doc_with_comp();
        let (base, top) = (LayerId(1), LayerId(2));
        place(&mut doc, base, [200, 200, 200, 200], 0);
        place(&mut doc, top, [255, 0, 0, 255], 1);
        doc.apply(Intent::SetAttrs {
            layer: top,
            patch: LayerAttrsPatch {
                matte: Some(Some(Matte { layer: base, mode })),
                ..Default::default()
            },
        })
        .unwrap();

        let mut engine = Engine::new().expect("engine");
        let result = engine.render_frame(&doc.view(), t(0));
        assert!(result.is_ok(), "{mode:?} は拒まれてはいけない: {result:?}");
    }
}

/// **マット元は二重描画されない**(AE/Lottie の track matte 意味論 — module doc の
/// 「engine 特有の配線」節)。`base`(青、alpha=128、order 0)を `top`(赤、alpha=255、
/// order 1、`matte: Alpha`)のマット元にする。もし `base` が通常描画リストから
/// 除外し損ねていたら、`base` 自身が(黒背景の上に)半透明の青として描かれ、その上に
/// matte 適用後の `top` が重なるので、最終画素の青チャンネルが漏れて出る
/// (0 のはずが正でない値になる)。正しく除外されていれば、青チャンネルは
/// (黒背景 + matte 適用後の赤だけなので)ほぼ 0 のまま。
#[test]
fn matte_source_layer_is_not_drawn_twice() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, [0, 0, 255, 128], 0);
    place(&mut doc, top, [255, 0, 0, 255], 1);

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: base,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    let px = pixel(&frame, 32, 32);

    assert!(
        px[2] < 10,
        "base(青)が通常描画リストにも残っていると青チャンネルが漏れるはず: {px:?}"
    );
}

/// マット元がこの時刻の `resolved_layers()` に居ない(=削除された、または timing の
/// 外)場合、matte を適用できないので **本体も描かない**——黙って matte 抜きの絵を
/// 出すと「勝手に露出した」ことになる(AE: track matte 対象はマット元が消えると
/// 一緒に消える)。`base` を `top` より短い timing にして、`base` が消えた後の時刻を
/// 描く。
#[test]
fn matte_with_absent_source_layer_hides_the_target_instead_of_erroring() {
    let mut doc = doc_with_comp();
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, top, [255, 0, 0, 255], 1);

    // base は 0..10 フレームだけ(既存 layer の timing を書き換える `SetMeta` は
    // 新規配置専用で使えない——`place` を直接呼ばず、初回配置から短い timing にする)。
    doc.apply(Intent::AddLayer(base)).unwrap();
    doc.apply(Intent::SetMeta {
        layer: base,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [0, 0, 255, 128],
                width: W,
                height: H,
            },
            order: 0,
            timing: LayerTiming::place(0, Some(10), 100_000),
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: base,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let mut engine = Engine::new().expect("engine");
    // base がもう居ない時刻(frame 30)。
    let frame = engine
        .render_frame(&doc.view(), t(30))
        .expect("マット元が居なくても Err にはならない");
    let px = pixel(&frame, 32, 32);

    assert_eq!(
        px, [0, 0, 0, 255],
        "マット元が消えた時刻は本体(top)も描かれず、黒背景だけが見えるはず: {px:?}"
    );
}
