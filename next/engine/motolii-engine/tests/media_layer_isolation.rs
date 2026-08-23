//! A05(next/reference/axis/A05-missing.tsv): `Engine::texture_for` の `Media` 枝の
//! `probe`/`read_frame_at` 失敗が `?` でそのまま外へ伝播すると、comp 合成フレーム
//! 全体が `Err` になり、**壊れていない他の layer まで**その瞬間出せなくなる
//! (関数直上のコメントが約束する「この layer だけ落とす」に反する)。
//!
//! ここでは「壊れた/存在しない Media の layer が1枚混じっていても、他の正常な
//! layer は変わらず描かれ、comp 全体としては `Ok` が返る」ことを画素で確かめる。
//! M16(render 失敗でも画面を空にしない)・Q3(拒否は理由がその場で分かる・
//! 沈黙禁止)の両方に対応する。

use motolii_engine::Engine;
use motolii_store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    RationalTime,
};

const W: u32 = 64;
const H: u32 = 64;

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

fn place_solid(doc: &mut Document, layer: LayerId, rgba: [u8; 4], order: i16) {
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

/// 実在しないパス。probe(ffprobe起動)は必ず失敗する — decode まで進まない。
fn place_broken_media(doc: &mut Document, layer: LayerId, order: i16) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Media {
                path: "/nonexistent/does-not-exist-a05.mp4".to_owned(),
                fingerprint: None,
            },
            order,
            timing: LayerTiming::place(0, None, 100_000),
        },
    })
    .unwrap();
}

/// 本命: 壊れた Media layer が1枚混じっていても、comp 合成全体は `Err` にならず、
/// 他の正常な(緑の) Solid layer はいつも通り描かれる。
///
/// 修正前は `texture_for` の `probe(path)?` がそのまま `EngineError` を上へ伝播し、
/// `render_frame` 自体が `Err` になっていた(= 緑も含めフレーム全体が出ない)。
#[test]
fn a_broken_media_layer_does_not_blank_the_whole_composite() {
    let mut doc = doc_with_comp();
    let healthy = LayerId(1);
    let broken = LayerId(2);
    place_solid(&mut doc, healthy, [0, 255, 0, 255], 0);
    place_broken_media(&mut doc, broken, 1);

    let mut engine = Engine::new().expect("engine");
    let frame = engine
        .render_frame(&doc.view(), RationalTime::try_new(0, 30).unwrap())
        .expect("1layerがprobe失敗でも、comp合成全体はErrにならないはず(このlayerだけ隔離)");

    let px = pixel(&frame, 32, 32);
    assert_eq!(
        px,
        [0, 255, 0, 255],
        "壊れたMedia layerを隔離した後ろの緑のSolid layerが見えるはず: {px:?}"
    );
}

/// 壊れた layer 単体(他に正常な layer が無い)でも `Err` にならず、
/// comp の背景(黒)がそのまま出ること — 「1枚も描けない」は「フレーム全体が
/// 出せない」とは違う(M16)。
#[test]
fn a_broken_media_layer_alone_still_renders_the_background() {
    let mut doc = doc_with_comp();
    let broken = LayerId(1);
    place_broken_media(&mut doc, broken, 0);

    let mut engine = Engine::new().expect("engine");
    let frame = engine
        .render_frame(&doc.view(), RationalTime::try_new(0, 30).unwrap())
        .expect("唯一のlayerがprobe失敗でも、render_frame全体はErrにならないはず");

    let px = pixel(&frame, 32, 32);
    assert_eq!(px, [0, 0, 0, 255], "背景の黒が出るはず: {px:?}");
}

/// 黙って握りつぶさない(Q3): 隔離した失敗の事実と理由が engine から読み出せる。
#[test]
fn a_broken_media_layer_failure_reason_is_readable_afterward() {
    let mut doc = doc_with_comp();
    let broken = LayerId(1);
    place_broken_media(&mut doc, broken, 0);

    let mut engine = Engine::new().expect("engine");
    engine
        .render_frame(&doc.view(), RationalTime::try_new(0, 30).unwrap())
        .expect("このlayerだけ隔離されるのでErrにはならない");

    let failures = engine.layer_failures();
    assert_eq!(failures.len(), 1, "壊れたlayerが1枚→理由も1件のはず: {failures:?}");
    assert!(
        failures[0].contains("does-not-exist-a05.mp4"),
        "理由にどの素材が読めなかったか出るはず: {failures:?}"
    );
}
