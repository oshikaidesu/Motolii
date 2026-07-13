//! DocParam評価(D3)。LookAt/Follow は解決済みレイヤー位置を参照する。
//!
//! DataTrack 実出力型と期待型(fallback のバリアント)の照合もここで行う(D1h後段)。

use crate::doc_value::DocValue;
use crate::param::DocParam;
use crate::param_expect::ExpectedValueType;
use crate::LayerId;
use motolii_core::RationalTime;
use motolii_eval::{DataTrackId, DataTracks, Value};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParamEvalError {
    #[error("DocParam::LookAt unresolved (layer {0})")]
    UnresolvedLookAt(u64),
    #[error("DocParam::Follow unresolved (layer {0})")]
    UnresolvedFollow(u64),
    #[error("expected {expected}, got {got:?}")]
    TypeMismatch { expected: &'static str, got: Value },
    #[error(
        "DataTrack `{track}` output type mismatch: expected {expected}, got {got:?} (fallback was {fallback})"
    )]
    DataTrackTypeMismatch {
        track: String,
        expected: &'static str,
        got: Value,
        fallback: &'static str,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedLayerParams {
    positions: std::collections::HashMap<u64, [f64; 2]>,
}

impl ResolvedLayerParams {
    pub fn insert_position(&mut self, layer: LayerId, pos: [f64; 2]) {
        self.positions.insert(layer.get(), pos);
    }

    pub fn position(&self, layer: LayerId) -> Option<[f64; 2]> {
        self.positions.get(&layer.get()).copied()
    }
}

fn doc_value_type(v: &DocValue) -> ExpectedValueType {
    match v {
        DocValue::F64(_) => ExpectedValueType::F64,
        DocValue::Vec2(_) => ExpectedValueType::Vec2,
        DocValue::Vec3(_) => ExpectedValueType::Vec3,
        DocValue::Color(_) => ExpectedValueType::Color,
        DocValue::AssetRef(_) => ExpectedValueType::AssetRef,
    }
}

fn value_matches_expected(expected: ExpectedValueType, v: &Value) -> bool {
    matches!(
        (expected, v),
        (ExpectedValueType::F64, Value::F64(_))
            | (ExpectedValueType::Vec2, Value::Vec2(_))
            | (ExpectedValueType::Vec3, Value::Vec3(_))
            | (ExpectedValueType::Color, Value::Color(_))
            | (ExpectedValueType::AssetRef, Value::AssetRef(_))
    )
}

pub fn eval_doc_param(
    param: &DocParam,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<Value, ParamEvalError> {
    match param {
        DocParam::Const(v) => Ok(v.to_eval()),
        DocParam::Keyframes(k) => Ok(k.eval(t)),
        DocParam::Data { track, fallback } => eval_data_track(track, fallback, t, tracks),
        DocParam::Vec2Axes { x, y } => {
            let xv = eval_doc_param(x, t, tracks, resolved)?;
            let yv = eval_doc_param(y, t, tracks, resolved)?;
            match (&xv, &yv) {
                (Value::F64(x), Value::F64(y)) => Ok(Value::Vec2([*x, *y])),
                (Value::Vec2([x, _]), Value::F64(y)) => Ok(Value::Vec2([*x, *y])),
                (Value::F64(x), Value::Vec2([y, _])) => Ok(Value::Vec2([*x, *y])),
                _ => Err(ParamEvalError::TypeMismatch {
                    expected: "Vec2",
                    got: xv,
                }),
            }
        }
        DocParam::LookAt { target, .. } => resolved
            .position(*target)
            .map(Value::Vec2)
            .ok_or(ParamEvalError::UnresolvedLookAt(target.get())),
        DocParam::Follow { target, offset } => resolved
            .position(*target)
            .map(|p| Value::Vec2([p[0] + offset[0], p[1] + offset[1]]))
            .ok_or(ParamEvalError::UnresolvedFollow(target.get())),
    }
}

fn eval_data_track(
    track: &DataTrackId,
    fallback: &DocValue,
    t: RationalTime,
    tracks: &DataTracks,
) -> Result<Value, ParamEvalError> {
    let expected = doc_value_type(fallback);
    match tracks.get(track) {
        None => Ok(fallback.to_eval()),
        Some(dt) => {
            let got = dt.eval(t);
            if !value_matches_expected(expected, &got) {
                return Err(ParamEvalError::DataTrackTypeMismatch {
                    track: track.0.clone(),
                    expected: expected.name(),
                    got,
                    fallback: fallback.kind_name(),
                });
            }
            Ok(got)
        }
    }
}

pub fn eval_f64(
    param: &DocParam,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<f64, ParamEvalError> {
    match eval_doc_param(param, t, tracks, resolved)? {
        Value::F64(v) => Ok(v),
        o => Err(ParamEvalError::TypeMismatch {
            expected: "F64",
            got: o,
        }),
    }
}

pub fn eval_vec2(
    param: &DocParam,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<[f64; 2], ParamEvalError> {
    match eval_doc_param(param, t, tracks, resolved)? {
        Value::Vec2(v) => Ok(v),
        o => Err(ParamEvalError::TypeMismatch {
            expected: "Vec2",
            got: o,
        }),
    }
}

pub fn eval_color(
    param: &DocParam,
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<[f64; 4], ParamEvalError> {
    match eval_doc_param(param, t, tracks, resolved)? {
        Value::Color(v) => Ok(v),
        o => Err(ParamEvalError::TypeMismatch {
            expected: "Color",
            got: o,
        }),
    }
}
