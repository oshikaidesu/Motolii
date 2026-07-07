use serde::{Deserialize, Serialize};

use oc_core::{Fps, RationalTime};

use crate::bezier::cubic_bezier_ease;
use crate::value::Value;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub t: RationalTime,
    pub value: Value,
    pub interp: Interp,
}

/// 時刻順にソートされたキーフレーム列。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyframeTrack {
    keys: Vec<Keyframe>,
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
    let num = (t - a).as_seconds_f64();
    let den = (b - a).as_seconds_f64();
    num / den
}

/// 解析結果などの等間隔サンプル列。start位置からsample_rateで並ぶ。
/// キーフレームと同じく「時刻→値」として評価できる(ParamSource::Dataから参照)。
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let rel = t - self.start;
        // サンプル位置(浮動小数)= rel * rate
        let pos = rel.as_seconds_f64() * self.sample_rate.as_f64();
        if pos <= 0.0 {
            return self.values[0].clone();
        }
        let last = self.values.len() - 1;
        if pos >= last as f64 {
            return self.values[last].clone();
        }
        let i = pos.floor() as usize;
        let u = pos - i as f64;
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
        let fps = Fps::new(30, 1);
        tr.insert(key(RationalTime::ZERO, 0.0, Interp::Linear));
        tr.insert(key(RationalTime::from_frame(30, fps), 30.0, Interp::Linear));
        // フレーム12(=0.4秒)で値12.0
        let v = tr.eval(RationalTime::from_frame(12, fps));
        assert!((v.as_f64().unwrap() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn hold_keeps_value_until_next_key() {
        let mut tr = KeyframeTrack::new();
        tr.insert(key(RationalTime::ZERO, 1.0, Interp::Hold));
        tr.insert(key(RationalTime::from_seconds(1), 2.0, Interp::Linear));
        assert_eq!(tr.eval(RationalTime::new(999, 1000)), Value::F64(1.0));
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
        let early = tr.eval(RationalTime::new(1, 2)).as_f64().unwrap();
        assert!(early < 25.0);
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
            sample_rate: Fps::new(10, 1),
            values: (0..=10).map(|i| Value::F64(i as f64)).collect(),
        };
        // start前はクランプ
        assert_eq!(dt.eval(RationalTime::ZERO), Value::F64(0.0));
        // start + 0.55秒 = サンプル位置5.5 → 5.5
        let v = dt.eval(RationalTime::new(155, 100)).as_f64().unwrap();
        assert!((v - 5.5).abs() < 1e-9);
        // 末尾以降はクランプ
        assert_eq!(dt.eval(RationalTime::from_seconds(10)), Value::F64(10.0));
    }

    #[test]
    fn param_source_data_with_fallback() {
        let mut ctx = DataTracks::new();
        ctx.insert(
            "centroid.x",
            DataTrack {
                start: RationalTime::ZERO,
                sample_rate: Fps::new(30, 1),
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
}
