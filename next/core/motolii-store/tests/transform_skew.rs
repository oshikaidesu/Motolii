//! transform の skew/skew_axis — `LayerPlacement::from_transform` に空けてあった穴
//! (裁定69)を Document の property track から埋める。

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

// `anchor`/`position` は既定 [0,0] のままなので、`transform.matrix2` の各軸ベクトルが
// そのまま「基底ベクトル (1,0) / (0,1) がどこへ写るか」を表す(glam を直接依存に
// 足さずに済ませるため — `matrix2`/`x_axis`/`y_axis`/`translation` はどれも
// `LayerPlacement::transform` の既存の公開フィールドを辿るだけ)。

/// skew を打っていない layer は今まで通り(恒等)。既存 layer が壊れないことの回帰試験。
#[test]
fn no_skew_track_means_no_skew() {
    let (doc, layer) = doc_with_layer();
    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let off_axis = resolved.placement.transform.matrix2.y_axis;
    assert!((off_axis.x).abs() < 1e-5, "skew が無いのに x がずれた: {off_axis:?}");
    assert!((off_axis.y - 1.0).abs() < 1e-5);
}

/// skew(sk)だけ打つと skew_axis の既定 0(x軸)で shear する。
/// x軸上の点は不動、y方向の点だけ x へずれる。
#[test]
fn skew_without_axis_defaults_to_the_x_axis() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply(Intent::SetTrack {
        layer,
        property: prop("skew"),
        track: still(Value::F64(45.0)),
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let on_axis = resolved.placement.transform.matrix2.x_axis;
    let off_axis = resolved.placement.transform.matrix2.y_axis;

    assert!((on_axis.x - 1.0).abs() < 1e-4 && on_axis.y.abs() < 1e-4, "{on_axis:?}");
    assert!(
        (off_axis.x - 1.0).abs() < 1e-3,
        "45度 skew で x が tan(45)=1 だけずれるはず: {off_axis:?}"
    );
}

/// skew_axis(sa)を変えると不動点が動く — sk と sa は独立変数。
#[test]
fn skew_axis_moves_the_fixed_line() {
    let (mut doc, layer) = doc_with_layer();
    doc.apply_all([
        Intent::SetTrack {
            layer,
            property: prop("skew"),
            track: still(Value::F64(45.0)),
        },
        Intent::SetTrack {
            layer,
            property: prop("skew_axis"),
            track: still(Value::F64(90.0)),
        },
    ])
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    let on_axis = resolved.placement.transform.matrix2.y_axis;
    assert!(
        (on_axis.x).abs() < 1e-4 && (on_axis.y - 1.0).abs() < 1e-4,
        "軸を90度にしたのに y軸上の点が不動点でない: {on_axis:?}"
    );
}
