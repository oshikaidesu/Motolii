//! `LayerAttrs::solo` の通し試験(画素段)。`next/core/motolii-store/tests/solo_locked.rs`
//! が resolve 段で固定した規則を、`Engine::render_frame` が実際に出す画素でも確かめる。
//!
//! - comp のどこかに solo な layer が居ると、solo でない layer は絵に出ない
//! - solo な layer 自身はいつも通り出る
//! - 誰も solo でなければ、solo を足す前と同じ絵になる(既定 false が振る舞いを
//!   変えないことの固定)

use motolii_engine::Engine;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
    LayerTiming, RationalTime,
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

/// order が高いほど手前。全面を覆う solid を1枚だけ置く(order は呼び手が渡す)。
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

fn set_solo(doc: &mut Document, layer: LayerId, solo: bool) {
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            solo: Some(solo),
            ..Default::default()
        },
    })
    .unwrap();
}

/// solo な layer が1枚でも居ると、solo でない layer は画面に出ない
/// (背景の緑が消え、solo にした前面の赤だけが残る)。
#[test]
fn a_solo_layer_hides_non_solo_siblings_in_the_rendered_pixels() {
    let mut doc = doc_with_comp();
    let background = LayerId(1);
    let foreground = LayerId(2);
    place(&mut doc, background, [0, 255, 0, 255], 0);
    place(&mut doc, foreground, [255, 0, 0, 255], 1);

    let mut engine = Engine::new().expect("engine");
    let before = engine.render_frame(&doc.view(), t(0)).unwrap();
    // solo を足す前は前面(赤)が背景(緑)を覆っている。
    assert!(pixel(&before, 32, 32)[0] > 200, "solo 前は赤が出ているはず");

    set_solo(&mut doc, foreground, true);
    let after = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert!(
        pixel(&after, 32, 32)[0] > 200,
        "solo な layer 自身はいつも通り出るはず: {:?}",
        pixel(&after, 32, 32)
    );

    // 背景だけを消して確かめる: 前面を hidden にすると、solo でない背景(緑)は
    // 出ない(solo モードでは背景が候補から落ちているので、前面が退いても
    // 緑は現れないはず — 何も描かれない = comp の背景色 or 透明のまま)。
    doc.apply(Intent::SetAttrs {
        layer: foreground,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .unwrap();
    let with_solo_hidden = engine.render_frame(&doc.view(), t(0)).unwrap();
    let px = pixel(&with_solo_hidden, 32, 32);
    assert!(
        px[1] < 50,
        "solo でない背景(緑)は solo モード中は復活しないはず: {px:?}"
    );
}

/// 誰も solo でなければ、solo を1度も触らない場合と同じ絵になる
/// (既定 false が既存の絵を変えないことの固定)。
#[test]
fn no_solo_renders_the_same_as_before_the_field_existed() {
    let mut doc = doc_with_comp();
    let background = LayerId(1);
    let foreground = LayerId(2);
    place(&mut doc, background, [0, 255, 0, 255], 0);
    place(&mut doc, foreground, [255, 0, 0, 255], 1);

    let mut engine = Engine::new().expect("engine");
    let frame = engine.render_frame(&doc.view(), t(0)).unwrap();
    assert!(
        pixel(&frame, 32, 32)[0] > 200,
        "solo を誰も立てていなければ前面がそのまま出るはず"
    );
}
