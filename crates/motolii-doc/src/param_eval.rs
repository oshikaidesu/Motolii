//! DocParam評価(D3)。
use motolii_core::RationalTime;
use motolii_eval::{DataTracks, Value};
use crate::param::DocParam;
use crate::LayerId;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParamEvalError {
    #[error("DocParam::LookAt unresolved (layer {0})")] UnresolvedLookAt(u64),
    #[error("DocParam::Follow unresolved (layer {0})")] UnresolvedFollow(u64),
    #[error("expected {expected}, got {got:?}")] TypeMismatch { expected: &'static str, got: Value },
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedLayerParams { positions: std::collections::HashMap<u64, [f64; 2]> }
impl ResolvedLayerParams { pub fn insert_position(&mut self, layer: LayerId, pos: [f64; 2]) { self.positions.insert(layer.get(), pos); } }

pub fn eval_doc_param(param: &DocParam, t: RationalTime, tracks: &DataTracks, resolved: &ResolvedLayerParams) -> Result<Value, ParamEvalError> {
    match param {
        DocParam::Const(v) => Ok(v.clone()),
        DocParam::Keyframes(k) => Ok(k.eval(t)),
        DocParam::Data { track, fallback } => Ok(tracks.get(track).map(|d| d.eval(t)).unwrap_or_else(|| fallback.clone())),
        DocParam::Vec2Axes { x, y } => {
            let xv = eval_doc_param(x, t, tracks, resolved)?;
            let yv = eval_doc_param(y, t, tracks, resolved)?;
            match (&xv, &yv) {
                (Value::F64(x), Value::F64(y)) => Ok(Value::Vec2([*x, *y])),
                (Value::Vec2([x, _]), Value::F64(y)) => Ok(Value::Vec2([*x, *y])),
                (Value::F64(x), Value::Vec2([y, _])) => Ok(Value::Vec2([*x, *y])),
                _ => Err(ParamEvalError::TypeMismatch { expected: "Vec2", got: xv }),
            }
        }
        DocParam::LookAt { target, .. } => resolved.positions.get(&target.get()).copied().map(|p| Value::Vec2(p)).ok_or(ParamEvalError::UnresolvedLookAt(target.get())),
        DocParam::Follow { target, offset } => resolved.positions.get(&target.get()).copied().map(|p| Value::Vec2([p[0]+offset[0], p[1]+offset[1]])).ok_or(ParamEvalError::UnresolvedFollow(target.get())),
    }
}
pub fn eval_f64(param: &DocParam, t: RationalTime, tracks: &DataTracks, resolved: &ResolvedLayerParams) -> Result<f64, ParamEvalError> {
    match eval_doc_param(param, t, tracks, resolved)? { Value::F64(v) => Ok(v), o => Err(ParamEvalError::TypeMismatch { expected: "F64", got: o }) }
}
pub fn eval_vec2(param: &DocParam, t: RationalTime, tracks: &DataTracks, resolved: &ResolvedLayerParams) -> Result<[f64; 2], ParamEvalError> {
    match eval_doc_param(param, t, tracks, resolved)? { Value::Vec2(v) => Ok(v), o => Err(ParamEvalError::TypeMismatch { expected: "Vec2", got: o }) }
}
pub fn eval_color(param: &DocParam, t: RationalTime, tracks: &DataTracks, resolved: &ResolvedLayerParams) -> Result<[f64; 4], ParamEvalError> {
    match eval_doc_param(param, t, tracks, resolved)? { Value::Color(v) => Ok(v), o => Err(ParamEvalError::TypeMismatch { expected: "Color", got: o }) }
}
