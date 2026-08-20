use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;

use crate::bezier::{cubic_bezier_ease, sample, solve_curve_x};
use crate::value::Value;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum TrackError {
    #[error("Bezier control point x1/x2 must be in [0,1], got x1={x1} x2={x2}")]
    InvalidBezier { x1: f64, x2: f64 },
    #[error(
        "Bezier split is unrepresentable: progress={progress}, x1={x1}, y1={y1}, x2={x2}, y2={y2}"
    )]
    UnrepresentableBezierSplit {
        progress: f64,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        curve_parameter: Option<f64>,
        split_value: Option<f64>,
        left_x_denominator: f64,
        right_x_denominator: f64,
        left_y_denominator: Option<f64>,
        right_y_denominator: Option<f64>,
    },
    #[error("keyframes must be sorted by strictly increasing time without duplicates")]
    UnsortedOrDuplicateKeys,
}

/// キーフレーム区間(このキーから次のキーまで)の補間方法。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interp {
    /// 次のキーまで値を保持
    Hold,
    Linear,
    /// cubic-bezier(x1,y1,x2,y2)イージング(x1,x2∈[0,1])
    Bezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
}

impl Interp {
    pub fn split_at(&self, progress: f64) -> Result<(Interp, Interp), TrackError> {
        if !progress.is_finite() || !(0.0..=1.0).contains(&progress) {
            if let Interp::Bezier { x1, y1, x2, y2 } = *self {
                return Err(TrackError::UnrepresentableBezierSplit {
                    progress,
                    x1,
                    y1,
                    x2,
                    y2,
                    curve_parameter: None,
                    split_value: None,
                    left_x_denominator: progress,
                    right_x_denominator: 1.0 - progress,
                    left_y_denominator: None,
                    right_y_denominator: None,
                });
            }
            return Err(TrackError::UnrepresentableBezierSplit {
                progress,
                x1: 0.0,
                y1: 0.0,
                x2: 0.0,
                y2: 0.0,
                curve_parameter: None,
                split_value: None,
                left_x_denominator: progress,
                right_x_denominator: 1.0 - progress,
                left_y_denominator: None,
                right_y_denominator: None,
            });
        }

        match *self {
            Interp::Hold => Ok((Interp::Hold, Interp::Hold)),
            Interp::Linear => Ok((Interp::Linear, Interp::Linear)),
            Interp::Bezier { x1, y1, x2, y2 } => {
                let s = solve_curve_x(x1, x2, progress);
                if !s.is_finite() || !(0.0..=1.0).contains(&s) {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: None,
                        left_x_denominator: progress,
                        right_x_denominator: 1.0 - progress,
                        left_y_denominator: None,
                        right_y_denominator: None,
                    });
                }

                let v = sample(y1, y2, s);
                if !v.is_finite() {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: progress,
                        right_x_denominator: 1.0 - progress,
                        left_y_denominator: None,
                        right_y_denominator: None,
                    });
                }

                let inv = 1.0 - s;
                let left_x_denom = progress;
                let right_x_denom = 1.0 - progress;
                let left_y_denom = v;
                let right_y_denom = 1.0 - v;

                if !left_x_denom.is_finite()
                    || !right_x_denom.is_finite()
                    || !left_y_denom.is_finite()
                    || !right_y_denom.is_finite()
                    || left_x_denom == 0.0
                    || right_x_denom == 0.0
                    || left_y_denom == 0.0
                    || right_y_denom == 0.0
                {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: left_x_denom,
                        right_x_denominator: right_x_denom,
                        left_y_denominator: Some(left_y_denom),
                        right_y_denominator: Some(right_y_denom),
                    });
                }

                let l1 = (x1 * s, y1 * s);
                let l2 = (x1 * inv + x2 * s, y1 * inv + y2 * s);
                let l3 = (x2 * inv + s, y2 * inv + s);
                let l12 = (l1.0 * inv + l2.0 * s, l1.1 * inv + l2.1 * s);
                let l23 = (l2.0 * inv + l3.0 * s, l2.1 * inv + l3.1 * s);

                let left = Interp::Bezier {
                    x1: l1.0 / left_x_denom,
                    y1: l1.1 / left_y_denom,
                    x2: l12.0 / left_x_denom,
                    y2: l12.1 / left_y_denom,
                };
                let right = Interp::Bezier {
                    x1: (l23.0 - progress) / right_x_denom,
                    y1: (l23.1 - v) / right_y_denom,
                    x2: (l3.0 - progress) / right_x_denom,
                    y2: (l3.1 - v) / right_y_denom,
                };

                if !is_valid_bezier_control(left) || !is_valid_bezier_control(right) {
                    return Err(TrackError::UnrepresentableBezierSplit {
                        progress,
                        x1,
                        y1,
                        x2,
                        y2,
                        curve_parameter: Some(s),
                        split_value: Some(v),
                        left_x_denominator: left_x_denom,
                        right_x_denominator: right_x_denom,
                        left_y_denominator: Some(left_y_denom),
                        right_y_denominator: Some(right_y_denom),
                    });
                }

                Ok((left, right))
            }
        }
    }
}

fn is_valid_bezier_control(interp: Interp) -> bool {
    if let Interp::Bezier { x1, y1, x2, y2 } = interp {
        [x1, y1, x2, y2].iter().all(|v| v.is_finite())
            && (0.0..=1.0).contains(&x1)
            && (0.0..=1.0).contains(&x2)
    } else {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub t: RationalTime,
    pub value: Value,
    pub interp: Interp,
}

/// 時刻順にソートされたキーフレーム列。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "KeyframeTrackDe")]
pub struct KeyframeTrack {
    keys: Vec<Keyframe>,
}

#[derive(Deserialize)]
struct KeyframeTrackDe {
    keys: Vec<Keyframe>,
}

impl TryFrom<KeyframeTrackDe> for KeyframeTrack {
    type Error = TrackError;

    fn try_from(value: KeyframeTrackDe) -> Result<Self, Self::Error> {
        let track = Self { keys: value.keys };
        track.validate()?;
        Ok(track)
    }
}

impl KeyframeTrack {
    pub fn new() -> Self {
        Self::default()
    }

    /// キーを挿入する。同時刻のキーが既にあれば置き換える。
    pub fn insert(&mut self, key: Keyframe) {
        match self.keys.binary_search_by(|k| k.t.cmp(&key.t)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn keys(&self) -> &[Keyframe] {
        &self.keys
    }

    pub fn validate(&self) -> Result<(), TrackError> {
        for window in self.keys.windows(2) {
            if window[0].t >= window[1].t {
                return Err(TrackError::UnsortedOrDuplicateKeys);
            }
        }
        for key in &self.keys {
            if let Interp::Bezier { x1, y1, x2, y2 } = key.interp {
                // y1/y2 も有限必須(x は範囲検査で NaN を弾けるが y は素通しだった — D1h)
                if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
                    return Err(TrackError::InvalidBezier { x1, x2 });
                }
                if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
                    return Err(TrackError::InvalidBezier { x1, x2 });
                }
            }
        }
        Ok(())
    }

    /// 時刻tでの値。範囲外は端の値でクランプ。キーが無い場合はF64(0.0)。
    pub fn eval(&self, t: RationalTime) -> Value {
        let keys = &self.keys;
        if keys.is_empty() {
            return Value::F64(0.0);
        }
        if t <= keys[0].t {
            return keys[0].value.clone();
        }
        let last = keys.len() - 1;
        if t >= keys[last].t {
            return keys[last].value.clone();
        }
        // keys[i].t <= t < keys[i+1].t となるiを探す
        let i = match keys.binary_search_by(|k| k.t.cmp(&t)) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let (a, b) = (&keys[i], &keys[i + 1]);
        match a.interp {
            Interp::Hold => a.value.clone(),
            Interp::Linear => Value::lerp(&a.value, &b.value, segment_u(a.t, b.t, t)),
            Interp::Bezier { x1, y1, x2, y2 } => {
                let u = cubic_bezier_ease(x1, y1, x2, y2, segment_u(a.t, b.t, t));
                Value::lerp(&a.value, &b.value, u)
            }
        }
    }
}

/// 区間内正規化位置u ∈ [0,1)。区間端は有理数で厳密に扱い、u自体はf64でよい
/// (uは1フレーム内の補間位置であり、蓄積しないためドリフトしない)。
fn segment_u(a: RationalTime, b: RationalTime, t: RationalTime) -> f64 {
    let den = seconds_since(b, a);
    if den == 0.0 {
        return 0.0;
    }
    seconds_since(t, a) / den
}

/// `t - origin` の秒。差分がi64 RationalTimeに収まれば厳密経路、溢れ時はf64秒差へフォールバック
/// (評価値を0に握り潰さない — M2E-16 P1)。
fn seconds_since(t: RationalTime, origin: RationalTime) -> f64 {
    match t.try_sub(origin) {
        Ok(rel) => rel.as_seconds_f64(),
        Err(_) => t.as_seconds_f64() - origin.as_seconds_f64(),
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::Fps;

    fn key(t: RationalTime, v: f64, interp: Interp) -> Keyframe {
        Keyframe {
            t,
            value: Value::F64(v),
            interp,
        }
    }

    #[test]
    fn empty_track_returns_zero() {
        assert_eq!(
            KeyframeTrack::new().eval(RationalTime::ZERO),
            Value::F64(0.0)
        );
    }

    #[test]
    fn clamps_outside_range() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::from_seconds(1), 10.0, Interp::Linear));
        tr.insert(key(RationalTime::from_seconds(2), 20.0, Interp::Linear));
        assert_eq!(tr.eval(RationalTime::ZERO), Value::F64(10.0));
        assert_eq!(tr.eval(RationalTime::from_seconds(5)), Value::F64(20.0));
    }

    #[test]
    fn linear_interpolation_at_rational_times() {
        let mut tr = KeyframeTrack::new();
        let fps = Fps::try_new(30, 1).unwrap();
        tr.insert(key(RationalTime::ZERO, 0.0, Interp::Linear));
        tr.insert(key(
            RationalTime::try_from_frame(30, fps).unwrap(),
            30.0,
            Interp::Linear,
        ));
        // フレーム12(=0.4秒)で値12.0
        let v = tr.eval(RationalTime::try_from_frame(12, fps).unwrap());
        assert!((v.as_f64().unwrap() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn hold_keeps_value_until_next_key() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::ZERO, 1.0, Interp::Hold));
        tr.insert(key(RationalTime::from_seconds(1), 2.0, Interp::Linear));
        assert_eq!(
            tr.eval(RationalTime::try_new(999, 1000).unwrap()),
            Value::F64(1.0)
        );
        assert_eq!(tr.eval(RationalTime::from_seconds(1)), Value::F64(2.0));
    }

    #[test]
    fn bezier_ease_in_out_midpoint() {
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
        tr.insert(key(RationalTime::from_seconds(2), 100.0, Interp::Linear));
        let mid = tr.eval(RationalTime::from_seconds(1)).as_f64().unwrap();
        assert!((mid - 50.0).abs() < 1e-3);
        // ease-in: 序盤は線形より遅い
        let early = tr
            .eval(RationalTime::try_new(1, 2).unwrap())
            .as_f64()
            .unwrap();
        assert!(early < 25.0);
    }

    #[test]
    fn rejects_unsorted_keys_on_validate() {
        let track = KeyframeTrack {
            keys: vec![
                key(RationalTime::from_seconds(2), 2.0, Interp::Linear),
                key(RationalTime::from_seconds(1), 1.0, Interp::Linear),
            ],
        };
        assert_eq!(track.validate(), Err(TrackError::UnsortedOrDuplicateKeys));
    }

    #[test]
    fn rejects_invalid_bezier_on_validate() {
        let track = KeyframeTrack {
            keys: vec![
                Keyframe {
                    t: RationalTime::ZERO,
                    value: Value::F64(0.0),
                    interp: Interp::Bezier {
                        x1: 1.5,
                        y1: 0.0,
                        x2: 0.5,
                        y2: 1.0,
                    },
                },
                key(RationalTime::from_seconds(1), 1.0, Interp::Linear),
            ],
        };
        assert!(matches!(
            track.validate(),
            Err(TrackError::InvalidBezier { x1, x2 })
            if (x1 - 1.5).abs() < f64::EPSILON && (x2 - 0.5).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn insert_replaces_same_time_key() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::ZERO, 1.0, Interp::Linear));
        tr.insert(key(RationalTime::ZERO, 5.0, Interp::Linear));
        assert_eq!(tr.keys().len(), 1);
        assert_eq!(tr.eval(RationalTime::ZERO), Value::F64(5.0));
    }







    #[test]
    fn keyframe_linear_across_i64_span_does_not_collapse_to_zero() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(
            RationalTime::from_seconds(i64::MIN),
            10.0,
            Interp::Linear,
        ));
        tr.insert(key(
            RationalTime::from_seconds(i64::MAX),
            20.0,
            Interp::Linear,
        ));
        // ゼロ近傍は区間のほぼ中央 → 15付近。差分Overflowを0.0に握り潰さないこと。
        let mid = tr.eval(RationalTime::ZERO).as_f64().unwrap();
        assert!(
            (mid - 15.0).abs() < 1.0,
            "expected ~15 near span midpoint, got {mid}"
        );
        assert_eq!(
            tr.eval(RationalTime::from_seconds(i64::MAX)),
            Value::F64(20.0)
        );
    }


}

#[cfg(test)]
mod split_tests {
    use super::*;

    fn bezier_controls(interp: &Interp) -> (f64, f64, f64, f64) {
        match interp {
            Interp::Bezier { x1, y1, x2, y2 } => (*x1, *y1, *x2, *y2),
            _ => unreachable!(),
        }
    }

    fn eval_split_curve(
        left: (f64, f64, f64, f64),
        right: (f64, f64, f64, f64),
        split: f64,
        x: f64,
    ) -> f64 {
        if x <= 0.0 {
            0.0
        } else if x < split {
            let u = x / split;
            cubic_bezier_ease(left.0, left.1, left.2, left.3, u)
        } else if x < 1.0 {
            let u = if split == 1.0 {
                1.0
            } else {
                (x - split) / (1.0 - split)
            };
            cubic_bezier_ease(right.0, right.1, right.2, right.3, u)
        } else {
            1.0
        }
    }

    #[test]
    fn split_hold_and_linear_keeps_variant() {
        assert_eq!(
            Interp::Hold.split_at(0.42).expect("hold split"),
            (Interp::Hold, Interp::Hold)
        );
        assert_eq!(
            Interp::Linear.split_at(0.42).expect("linear split"),
            (Interp::Linear, Interp::Linear)
        );
    }

    #[test]
    fn split_bezier_returns_bezier_pair() {
        let interp = Interp::Bezier {
            x1: 0.42,
            y1: 0.1,
            x2: 0.58,
            y2: 0.9,
        };
        let (left, right) = interp.split_at(0.35).expect("bezier split");
        let (lx1, _, lx2, _) = bezier_controls(&left);
        let (rx1, _, rx2, _) = bezier_controls(&right);

        assert!((0.0..=1.0).contains(&lx1));
        assert!((0.0..=1.0).contains(&lx2));
        assert!((0.0..=1.0).contains(&rx1));
        assert!((0.0..=1.0).contains(&rx2));
    }

    #[test]
    fn split_bezier_preserves_curve_at_multiple_samples() {
        let interp = Interp::Bezier {
            x1: 0.28,
            y1: -0.2,
            x2: 0.84,
            y2: 0.8,
        };
        let split = 0.43;
        let (left, right) = interp.split_at(split).expect("bezier split");
        let left = bezier_controls(&left);
        let right = bezier_controls(&right);
        let split_value = cubic_bezier_ease(0.28, -0.2, 0.84, 0.8, split);

        for i in 0..=100 {
            let x = i as f64 / 100.0;
            if (x - split).abs() < f64::EPSILON {
                continue;
            }
            let original = cubic_bezier_ease(0.28, -0.2, 0.84, 0.8, x);
            let expected_normalized = if x < split {
                original / split_value
            } else {
                (original - split_value) / (1.0 - split_value)
            };
            let split_value = eval_split_curve(left, right, split, x);

            assert!(
                (expected_normalized - split_value).abs() <= 1e-6,
                "x={x} expected={expected_normalized} split={split_value}"
            );
        }
    }

    #[test]
    fn split_bezier_rejects_invalid_progress() {
        let interp = Interp::Bezier {
            x1: 0.3,
            y1: 0.1,
            x2: 0.7,
            y2: 0.9,
        };
        assert!(matches!(
            interp.split_at(f64::NAN),
            Err(TrackError::UnrepresentableBezierSplit { .. })
        ));
        assert!(matches!(
            interp.split_at(-0.01),
            Err(TrackError::UnrepresentableBezierSplit { .. })
        ));
        assert!(matches!(
            interp.split_at(1.01),
            Err(TrackError::UnrepresentableBezierSplit { .. })
        ));
    }

    #[test]
    fn split_bezier_rejects_unrepresentable() {
        let interp = Interp::Bezier {
            x1: 0.5,
            y1: 0.0,
            x2: 0.5,
            y2: -1.0,
        };
        let err = interp
            .split_at(0.0)
            .expect_err("must reject unrepresentable split");
        match err {
            TrackError::UnrepresentableBezierSplit {
                left_x_denominator,
                right_x_denominator,
                left_y_denominator,
                right_y_denominator,
                ..
            } => {
                assert_eq!(left_x_denominator, 0.0);
                assert_eq!(right_x_denominator, 1.0);
                assert!(left_y_denominator.is_some());
                assert!(right_y_denominator.is_some());
                assert!(left_y_denominator
                    .as_ref()
                    .expect("left y denom should be present")
                    .is_finite());
                assert!(right_y_denominator
                    .as_ref()
                    .expect("right y denom should be present")
                    .is_finite());
            }
            _ => panic!("wrong error kind"),
        }
    }

    #[test]
    fn split_bezier_rejects_infinite_control_value_with_finite_progress() {
        let interp = Interp::Bezier {
            x1: 0.5,
            y1: f64::INFINITY,
            x2: 0.5,
            y2: 0.8,
        };
        let err = interp
            .split_at(0.35)
            .expect_err("must reject inf y control");
        let TrackError::UnrepresentableBezierSplit { split_value, .. } = err else {
            panic!("wrong error kind");
        };

        assert!(split_value.expect("split value is computed").is_infinite());
    }
}
