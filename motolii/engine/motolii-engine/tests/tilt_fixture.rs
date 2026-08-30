//! 傾き・`position.z` を持つ層が背景に埋もれないこと。
//!
//! 背景を comp 全域の板として世界に置くと、上流が矩形をカメラからの距離で並べ替える
//! ため(`re_renderer` の `DrawDataDrawable::from_world_position`)、光軸から離れた層は
//! z を持った瞬間に背景より遠いと判定されて上塗りされた。背景は clear 色になった。

use motolii_engine::Engine;
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

/// 背景の黒でない画素の数を、背景あり(`render_frame`)/なしで測る。
fn lit(property_name: &str, value: f64) -> (usize, usize) {
    let mut fx = motolii_fixture::build();
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

/// 傾きが**効く**ことと、**絵が失われない**ことを同時に縛る。片方だけだと
/// 「素通しで無視」と「丸ごと消失」を取り違える。
#[test]
fn tilt_changes_the_picture_without_erasing_it() {
    let (flat, _) = lit(property::ROTATION_X, 0.0);
    let (t2, t2_nobg) = lit(property::ROTATION_X, 2.0);
    let (t20, t20_nobg) = lit(property::ROTATION_X, 20.0);

    assert_ne!(flat, t2, "2° 傾けても絵が変わらない: 傾きが描画へ届いていない");
    assert_eq!(t2, t2_nobg, "2°: 背景の有無で層の見え方が変わる");
    assert_eq!(t20, t20_nobg, "20°: 背景の有無で層の見え方が変わる");
}

/// `position.z` も同じ機序で消えていた。手前・奥のどちらへ動かしても残ること。
#[test]
fn depth_does_not_erase_the_picture() {
    let (near, near_nobg) = lit(property::POSITION_Z, -200.0);
    let (far, far_nobg) = lit(property::POSITION_Z, 200.0);

    assert!(near > 0 && far > 0, "z を動かすと絵が消える");
    assert_eq!(near, near_nobg, "手前へ: 背景の有無で層の見え方が変わる");
    assert_eq!(far, far_nobg, "奥へ: 背景の有無で層の見え方が変わる");
}
