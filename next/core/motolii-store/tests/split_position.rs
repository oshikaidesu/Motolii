//! position の split variant — x/y を別 track に割る形(裁定61)。
//!
//! `position`(Vec2 単一 track)が既定で、split は**後から選べる代替**。
//! 両方を同時に持つ状態は「単一 track が優先」で解決する(place を打ち直す前に
//! split へ乗り換える途中経過を壊さない)。

use motolii_store::{
    Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn doc_with_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    let layer = LayerId(1);
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
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

fn prop(name: &str) -> PropertyId {
    PropertyId::new(name).expect("property name")
}

/// x/y を別 track に割っても、単一 `position` と同じ場所へ解決される。
#[test]
fn split_x_and_y_tracks_combine_into_the_same_translation() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: prop("position.x"),
            track: still(Value::F64(10.0)),
        },
        Intent::SetTrack {
            layer,
            property: prop("position.y"),
            track: still(Value::F64(20.0)),
        },
    ])
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let translation = resolved.placement.transform.translation;
    assert!(
        (translation.x - 10.0).abs() < 1e-5 && (translation.y - 20.0).abs() < 1e-5,
        "split x/y が合成されていない: {translation:?}"
    );
}

/// 片方だけキーを打っている場合、打っていない側は 0.0(裁定20 と同じ「静止値」扱い)。
#[test]
fn a_missing_split_component_defaults_to_zero() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetTrack {
        layer,
        property: prop("position.x"),
        track: still(Value::F64(42.0)),
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let translation = resolved.placement.transform.translation;
    assert!((translation.x - 42.0).abs() < 1e-5);
    assert!((translation.y).abs() < 1e-5);
}

/// 単一 `position`(Vec2)が優先される — split との併存は移行途中の状態として許すが、
/// 解決結果は単一 track の方を勝たせる。
#[test]
fn a_single_position_track_wins_over_split_tracks() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: prop("position"),
            track: still(Value::Vec2([1.0, 2.0])),
        },
        Intent::SetTrack {
            layer,
            property: prop("position.x"),
            track: still(Value::F64(999.0)),
        },
    ])
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let translation = resolved.placement.transform.translation;
    assert!(
        (translation.x - 1.0).abs() < 1e-5 && (translation.y - 2.0).abs() < 1e-5,
        "単一 position が優先されていない: {translation:?}"
    );
}

/// position を一切打たなければ、split でも既定 [0,0]。
#[test]
fn no_position_track_at_all_defaults_to_zero() {
    let (doc, layer) = doc_with_layer();
    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let translation = resolved.placement.transform.translation;
    assert!(translation.x.abs() < 1e-5 && translation.y.abs() < 1e-5);
}
