//! A03副監督A発注(2026-08-23)の検収条件1点(裁定218)。
//!
//! 「hidden の track に t=0で false・t=10で true を打つと、`resolve(layer, t=0)` は
//! `Some`、`resolve(layer, t=10)` は `None` を返す」— これが無いと目的(hidden が
//! 時間軸に乗ったこと)を達していない、唯一の検収条件そのもの(裁定218 (a))。
//! track の無い layer が今まで通り静的値で動くことも同じ test 内で確認する
//! (後方互換、裁定20)。

use motolii_store::{
    Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [255, 0, 0, 255],
                    width: 64,
                    height: 64,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    (doc, layer)
}

/// **裁定218 の検収条件そのもの**。`hidden` に t=0(false)/t=10(true) の Hold track
/// を打つと、comp t=0 では layer が resolve され、t=10 では消える。
#[test]
fn hidden_track_can_be_keyframed_across_time() {
    let (mut doc, layer) = doc_with_layer();

    let mut hidden_track = KeyframeTrack::new();
    hidden_track.insert(Keyframe {
        t: t(0),
        value: Value::Bool(false),
        interp: Interp::Hold,
        spatial: None,
    });
    hidden_track.insert(Keyframe {
        t: t(10),
        value: Value::Bool(true),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::hidden(),
        track: hidden_track,
    })
    .unwrap();

    assert!(
        doc.view().resolve(layer, t(0)).unwrap().is_some(),
        "t=0(hidden=false のキー)では layer が resolve されるはず"
    );
    assert!(
        doc.view().resolve(layer, t(10)).unwrap().is_none(),
        "t=10(hidden=true のキー)では layer が消えるはず"
    );
}

/// **後方互換**(裁定20・裁定218 の同一 test 内での確認可の指定通り)。hidden の
/// track を一度も書いていない layer は、今まで通り静的 `LayerAttrs::hidden`
/// (既定 false)で動く。
#[test]
fn a_layer_with_no_hidden_track_resolves_by_the_static_default() {
    let (doc, layer) = doc_with_layer();

    assert!(
        doc.view().resolve(layer, t(0)).unwrap().is_some(),
        "track が無い layer は静的な既定値(非 hidden)で resolve されるはず"
    );
}
