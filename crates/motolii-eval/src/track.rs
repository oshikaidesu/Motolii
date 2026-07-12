use serde::{Deserialize, Serialize};

use motolii_core::{Fps, RationalTime};

use crate::bezier::cubic_bezier_ease;
use crate::value::Value;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum TrackError {
    #[error("Bezier control point x1/x2 must be in [0,1], got x1={x1} x2={x2}")]
    InvalidBezier { x1: f64, x2: f64 },
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

/// 解析結果などの等間隔サンプル列。start位置からsample_rateで並ぶ。
/// キーフレームと同じく「時刻→値」として評価できる(ParamSource::Dataから参照)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTrack {
    pub start: RationalTime,
    pub sample_rate: Fps,
    pub values: Vec<Value>,
}

impl DataTrack {
    /// 時刻tでの値(サンプル間は線形補間、範囲外は端でクランプ)。空ならF64(0.0)。
    pub fn eval(&self, t: RationalTime) -> Value {
        if self.values.is_empty() {
            return Value::F64(0.0);
        }
        // Ord比較は交差乗算がi128に収まる。差分のRationalTime化より先に端クランプする。
        if t <= self.start {
            return self.values[0].clone();
        }
        let last = self.values.len() - 1;
        // 添字は有理数床(S7)。i128溢れでも256bit乗除で床を求める。
        // Err(Overflow)は商がi64に収まらない巨大添字のみ → 末尾クランプでよい。
        let (i, u) = match t.try_to_sample_index_since(self.start, self.sample_rate) {
            Ok((idx, _)) if idx < 0 => return self.values[0].clone(),
            Ok((idx, frac)) => (idx as usize, frac),
            Err(_) => return self.values[last].clone(),
        };
        if i >= last {
            return self.values[last].clone();
        }
        Value::lerp(&self.values[i], &self.values[i + 1], u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataTrackId, DataTracks, ParamSource};

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
    fn data_track_sampling() {
        let dt = DataTrack {
            start: RationalTime::from_seconds(1),
            sample_rate: Fps::try_new(10, 1).unwrap(),
            values: (0..=10).map(|i| Value::F64(i as f64)).collect(),
        };
        // start前はクランプ
        assert_eq!(dt.eval(RationalTime::ZERO), Value::F64(0.0));
        // start + 0.55秒 = サンプル位置5.5 → 5.5
        let v = dt
            .eval(RationalTime::try_new(155, 100).unwrap())
            .as_f64()
            .unwrap();
        assert!((v - 5.5).abs() < 1e-9);
        // 末尾以降はクランプ
        assert_eq!(dt.eval(RationalTime::from_seconds(10)), Value::F64(10.0));
    }

    /// S7: NTSC rate でフレーム格子上の時刻が values[i] に一致する(f64床で1つ前に落ちない)。
    #[test]
    fn data_track_ntsc_exact_frame_hits_sample() {
        let rate = Fps::try_new(30000, 1001).unwrap();
        let values: Vec<Value> = (0..=30).map(|i| Value::F64(i as f64)).collect();
        let dt = DataTrack {
            start: RationalTime::ZERO,
            sample_rate: rate,
            values,
        };
        for frame in [0i64, 1, 14, 15, 29, 30] {
            let t = RationalTime::try_from_frame(frame, rate).unwrap();
            assert_eq!(
                dt.eval(t),
                Value::F64(frame as f64),
                "frame {frame} must hit sample {frame}"
            );
        }
    }

    /// S7: 非補間型は境界で1サンプル前に落ちず、区間内は Hold(先頭側)。
    #[test]
    fn data_track_asset_ref_hold_at_ntsc_boundary() {
        let rate = Fps::try_new(30000, 1001).unwrap();
        let dt = DataTrack {
            start: RationalTime::ZERO,
            sample_rate: rate,
            values: vec![Value::AssetRef(1), Value::AssetRef(2), Value::AssetRef(3)],
        };
        assert_eq!(
            dt.eval(RationalTime::try_from_frame(1, rate).unwrap()),
            Value::AssetRef(2)
        );
        // サンプル0と1の中間 → lerp が非補間のため先頭側を保持
        let half = rate
            .frame_duration()
            .try_mul(RationalTime::try_new(1, 2).unwrap())
            .unwrap();
        assert_eq!(dt.eval(half), Value::AssetRef(1));
        assert_eq!(
            dt.eval(RationalTime::try_from_frame(2, rate).unwrap()),
            Value::AssetRef(3)
        );
    }

    /// S7: i128中間積が溢れても先頭近傍なら index 0(末尾へ誤クランプしない)。
    #[test]
    fn data_track_near_start_despite_i128_mul_overflow() {
        let d = i64::MAX;
        let start = RationalTime::try_new(1, d - 2).unwrap();
        let t = RationalTime::try_new(1000, d).unwrap();
        let rate = Fps::try_new(d - 1, d).unwrap();
        let dt = DataTrack {
            start,
            sample_rate: rate,
            values: vec![Value::F64(10.0), Value::F64(20.0), Value::F64(30.0)],
        };
        assert!(t > start);
        let v = dt.eval(t).as_f64().unwrap();
        // index 0・微小 u なので values[0] 近傍(末尾30へ誤クランプしない)
        assert!(
            (v - 10.0).abs() < 1e-6,
            "expected near 10.0 at start of track, got {v}"
        );
    }

    #[test]
    fn data_track_end_clamps_when_relative_overflows_i64() {
        // start=MIN, t=MAX の差分は RationalTime に再格納できないが、末尾クランプの20を返す。
        let dt = DataTrack {
            start: RationalTime::from_seconds(i64::MIN),
            sample_rate: Fps::try_new(1, 1).unwrap(),
            values: vec![Value::F64(10.0), Value::F64(20.0)],
        };
        assert_eq!(
            dt.eval(RationalTime::from_seconds(i64::MAX)),
            Value::F64(20.0)
        );
    }

    #[test]
    fn data_track_start_clamps_across_i64_bounds() {
        let dt = DataTrack {
            start: RationalTime::from_seconds(i64::MAX),
            sample_rate: Fps::try_new(1, 1).unwrap(),
            values: vec![Value::F64(10.0), Value::F64(20.0)],
        };
        assert_eq!(
            dt.eval(RationalTime::from_seconds(i64::MIN)),
            Value::F64(10.0)
        );
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

    #[test]
    fn param_source_data_with_fallback() {
        let mut ctx = DataTracks::new();
        ctx.insert(
            "centroid.x",
            DataTrack {
                start: RationalTime::ZERO,
                sample_rate: Fps::try_new(30, 1).unwrap(),
                values: vec![Value::F64(3.0), Value::F64(5.0)],
            },
        );
        let p = ParamSource::Data {
            track: "centroid.x".into(),
            fallback: Value::F64(-1.0),
        };
        assert_eq!(p.eval(RationalTime::ZERO, &ctx), Value::F64(3.0));

        let missing = ParamSource::Data {
            track: DataTrackId("nope".into()),
            fallback: Value::F64(-1.0),
        };
        assert_eq!(missing.eval(RationalTime::ZERO, &ctx), Value::F64(-1.0));
    }

    #[test]
    fn vec2_axes_uses_data_fallback_when_track_is_not_scalar() {
        let mut tracks = DataTracks::new();
        tracks.insert(
            "vec",
            DataTrack {
                start: RationalTime::ZERO,
                sample_rate: Fps::try_new(1, 1).unwrap(),
                values: vec![Value::Vec2([9.0, 9.0])],
            },
        );
        let source = ParamSource::Vec2Axes {
            x: Box::new(ParamSource::Data {
                track: DataTrackId("vec".into()),
                fallback: Value::F64(0.42),
            }),
            y: Box::new(ParamSource::Const(Value::F64(0.0))),
        };
        assert_eq!(
            source.eval(RationalTime::ZERO, &tracks),
            Value::Vec2([0.42, 0.0])
        );
    }
}
