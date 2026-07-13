//! D1i-3: Bezier solver / キーフレーム補間の意味論ゴールデン(S16)。
//! CSS `cubic-bezier` 定義・色は保存空間線形・回転は最短ラップしないことを固定する。
//! 本ファイルのアサーション更新は禁止(新variant+新ファイルのみ)。

use motolii_core::RationalTime;
use motolii_doc::{DocKeyframe, DocKeyframeTrack, DocValue, KeyframeId};
use motolii_eval::{cubic_bezier_ease, Interp, Keyframe, KeyframeTrack, Value};

fn approx(a: f64, b: f64, eps: f64) {
    assert!((a - b).abs() < eps, "expected {b}, got {a}");
}

#[test]
fn cubic_bezier_endpoints_are_fixed() {
    assert_eq!(cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, 0.0), 0.0);
    assert_eq!(cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, 1.0), 1.0);
}

#[test]
fn cubic_bezier_linear_controls_are_identity() {
    for i in 0..=20 {
        let x = i as f64 / 20.0;
        let y = cubic_bezier_ease(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, x);
        approx(y, x, 1e-6);
    }
}

#[test]
fn cubic_bezier_ease_in_out_sample_points() {
    // CSS ease-in-out 相当(0.42,0,0.58,1)。数値は現行 solver の審判。
    let f = |x: f64| cubic_bezier_ease(0.42, 0.0, 0.58, 1.0, x);
    approx(f(0.25), 0.129_161_900_568_787_7, 1e-12);
    approx(f(0.5), 0.5, 1e-12);
    approx(f(0.75), 0.870_838_099_431_212_2, 1e-12);
    assert!(f(0.25) < 0.25);
    assert!(f(0.75) > 0.75);
}

#[test]
fn cubic_bezier_y_overshoot_is_allowed() {
    let y = cubic_bezier_ease(0.3, 1.5, 0.7, 1.5, 0.5);
    approx(y, 1.25, 1e-12);
}

#[test]
fn keyframe_bezier_uses_easing_then_value_lerp() {
    let mut tr = KeyframeTrack::new();
    tr.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::F64(0.0),
        interp: Interp::Bezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        },
    });
    tr.insert(Keyframe {
        t: RationalTime::from_seconds(2),
        value: Value::F64(100.0),
        interp: Interp::Linear,
    });
    // t=1 → segment_u=0.5 → ease≈0.5 → 50
    approx(
        tr.eval(RationalTime::from_seconds(1)).as_f64().unwrap(),
        50.0,
        1e-3,
    );
    // 序盤は線形(25)より遅い
    let early = tr
        .eval(RationalTime::try_new(1, 2).unwrap())
        .as_f64()
        .unwrap();
    assert!(early < 25.0, "ease-in early={early}");
}

#[test]
fn color_lerp_is_component_linear_in_stored_srgb() {
    // 仕様: 非線形sRGB・straight-alpha の成分ごと線形(知覚空間へ行かない)。
    let a = Value::Color([0.0, 0.0, 0.0, 0.0]);
    let b = Value::Color([1.0, 0.5, 0.0, 1.0]);
    assert_eq!(
        Value::lerp(&a, &b, 0.25),
        Value::Color([0.25, 0.125, 0.0, 0.25])
    );
}

#[test]
fn rotation_scalar_lerp_does_not_shortest_path_wrap() {
    // 350°→10° 相当(ラジアン)。最短ラップなら +20°、現行は -340° 方向の線形。
    let a = Value::F64(350.0_f64.to_radians());
    let b = Value::F64(10.0_f64.to_radians());
    let mid = Value::lerp(&a, &b, 0.5);
    let expected = (350.0_f64.to_radians() + 10.0_f64.to_radians()) / 2.0;
    approx(mid.as_f64().unwrap(), expected, 1e-12);
    // 最短経路の中点(0°)ではないことを明示
    assert!((mid.as_f64().unwrap() - 0.0).abs() > 1.0);
}

#[test]
fn doc_keyframe_track_eval_delegates_to_eval_bezier() {
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(1),
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Bezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        },
    });
    track.insert(DocKeyframe {
        id: KeyframeId::from_raw(2),
        t: RationalTime::from_seconds(2),
        value: DocValue::F64(100.0),
        interp: Interp::Linear,
    });
    approx(
        track.eval(RationalTime::from_seconds(1)).as_f64().unwrap(),
        50.0,
        1e-3,
    );
}
