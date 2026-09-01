//! キーが生まれる規則は1つ。
//!
//! 裁定(2026-09-01): **属性は素の値を持ち、キーは利用者が ◇ を押した時だけ在る**。
//! 打たれるのは**いま居る時刻**。窓のどの入口から触っても同じでなければ、
//! 「触っただけで0秒にキーが生えた」ように見える。

mod testkit;

use motolii::doc::store::{
    property, Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    PropertyId, RationalTime, Value,
};

fn doc_with_a_layer() -> (Document, LayerId) {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 480,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Shape,
            order: 0,
            timing: LayerTiming::place(0, None, 300),
        },
    })
    .unwrap();
    (doc, layer)
}

fn position() -> PropertyId {
    PropertyId::new(property::POSITION).unwrap()
}

#[test]
fn touching_a_property_does_not_grow_a_key() {
    let (mut doc, layer) = doc_with_a_layer();
    let at = RationalTime::try_new(60, 30).unwrap();

    let intent = doc.place(layer, &position(), Value::Vec2([10.0, 20.0]), at);
    doc.apply(intent).unwrap();

    assert!(
        doc.view().track(layer, &position()).unwrap().is_none(),
        "◇ を押していないのにキーが生えた"
    );
    assert_eq!(
        doc.view().value_at(layer, &position(), at).unwrap(),
        Some(Value::Vec2([10.0, 20.0])),
        "素の値が置けていない"
    );
}

#[test]
fn once_time_is_open_the_key_lands_on_the_time_you_are_at() {
    let (mut doc, layer) = doc_with_a_layer();
    let zero = RationalTime::try_new(0, 30).unwrap();
    let later = RationalTime::try_new(18, 30).unwrap();

    // ◇ を押した = 今の時刻に1つ立てる
    let mut track = motolii::doc::store::KeyframeTrack::new();
    track.insert(motolii::doc::store::Keyframe {
        t: zero,
        value: Value::Vec2([0.0, 0.0]),
        interp: motolii::doc::store::Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetTrack { layer, property: position(), track }).unwrap();

    let intent = doc.place(layer, &position(), Value::Vec2([100.0, 0.0]), later);
    doc.apply(intent).unwrap();

    let track = doc.view().track(layer, &position()).unwrap().expect("開いたはず");
    let times: Vec<_> = track.keys().iter().map(|k| k.t).collect();
    assert_eq!(times, vec![zero, later], "いま居る時刻に打たれていない: {times:?}");
}

#[test]
fn the_camera_follows_the_same_rule() {
    let (mut doc, _) = doc_with_a_layer();
    let property = PropertyId::camera(property::CAMERA_CENTER).unwrap();
    let at = RationalTime::try_new(60, 30).unwrap();

    let intent = doc.place_camera(&property, Value::Vec2([5.0, 6.0]), at);
    doc.apply(intent).unwrap();

    assert!(
        doc.view().camera_track(&property).unwrap().is_none(),
        "カメラを触っただけでキーが生えた"
    );
    assert_eq!(
        doc.view().camera_value_at(&property, at).unwrap(),
        Some(Value::Vec2([5.0, 6.0]))
    );
}
