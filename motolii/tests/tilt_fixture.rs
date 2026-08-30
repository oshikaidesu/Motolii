
use motolii::render::engine::Engine;
use motolii_store::{
    property, Document, Intent, Interp, Keyframe, KeyframeTrack, PropertyId, RationalTime, Value,
};

fn seed(doc: &mut Document, name: &str, deg: f64) {
    let layers = doc.view().layers();
    let intents: Vec<_> = layers
        .iter()
        .map(|l| {
            let mut track = KeyframeTrack::new();
            track.insert(Keyframe {
                t: RationalTime::ZERO,
                value: Value::F64(deg),
                interp: Interp::Linear,
                spatial: None,
            });
            Intent::SetTrack {
                layer: *l,
                property: PropertyId::new(name).expect("標準 property"),
                track,
            }
        })
        .collect();
    doc.apply_all(intents).expect("track を打てる");
}

fn lit(property_name: &str, value: f64) -> (usize, usize) {
    let mut fx = motolii::doc::fixture::build();
    if value != 0.0 {
        seed(&mut fx.doc, property_name, value);
    }
    let mut engine = Engine::new().expect("engine");
    let view = fx.doc.view();
    let count = |bytes: Vec<u8>| {
        bytes
            .chunks(4)
            .filter(|p| p[0] > 8 || p[1] > 8 || p[2] > 8)
            .count()
    };
    let with = count(engine.render_frame(&view, RationalTime::ZERO).expect("render_frame"));
    let without = count(
        engine
            .render_frame_without_background(&view, RationalTime::ZERO)
            .expect("render_frame_without_background"),
    );
    (with, without)
}

#[test]
fn tilt_changes_the_picture_without_erasing_it() {
    let (flat, _) = lit(property::ROTATION_X, 0.0);
    let (t2, t2_nobg) = lit(property::ROTATION_X, 2.0);
    let (t20, t20_nobg) = lit(property::ROTATION_X, 20.0);

    assert_ne!(flat, t2, "2° 傾けても絵が変わらない: 傾きが描画へ届いていない");
    assert_eq!(t2, t2_nobg, "2°: 背景の有無で層の見え方が変わる");
    assert_eq!(t20, t20_nobg, "20°: 背景の有無で層の見え方が変わる");
}

#[test]
fn depth_does_not_erase_the_picture() {
    let (near, near_nobg) = lit(property::POSITION_Z, -200.0);
    let (far, far_nobg) = lit(property::POSITION_Z, 200.0);

    assert!(near > 0 && far > 0, "z を動かすと絵が消える");
    assert_eq!(near, near_nobg, "手前へ: 背景の有無で層の見え方が変わる");
    assert_eq!(far, far_nobg, "奥へ: 背景の有無で層の見え方が変わる");
}
