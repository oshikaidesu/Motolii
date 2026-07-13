//! DocParam評価(D3)。Follow は解決済み位置、LookAt は self→target の回転角。
//!
//! DataTrack 実出力型と期待型(fallback のバリアント)の照合もここで行う(D1h後段)。

use crate::doc_value::DocValue;
use crate::param::{DocParam, LookAtAxis};
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
    /// LookAt は self 位置が要る。汎用 `eval_doc_param` では評価不能。
    #[error("DocParam::LookAt requires self position; use eval_look_at_rotation")]
    LookAtRequiresSelfPosition,
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
    /// validate の `ParentCycle` と同型。評価時に未検証文書へも適用する。
    #[error("transform.parent cycle involving layer {layer}")]
    ParentCycle { layer: u64 },
    /// LookAt/Follow/parent/Group 継承の依存が循環している。
    #[error("spatial link cycle involving layer {layer}")]
    SpatialLinkCycle { layer: u64 },
    #[error("transform.parent {parent} does not resolve to a layer")]
    DanglingParent { parent: u64 },
    #[error("singular placement space on layer {layer} (cannot map Follow into parent/group)")]
    SingularPlacementSpace { layer: u64 },
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
        // concept: rotation(t)=look_at(self, target) — self 無しでは角度が決まらない。
        DocParam::LookAt { .. } => Err(ParamEvalError::LookAtRequiresSelfPosition),
        DocParam::Follow { target, offset } => resolved
            .position(*target)
            .map(|p| Value::Vec2([p[0] + offset[0], p[1] + offset[1]]))
            .ok_or(ParamEvalError::UnresolvedFollow(target.get())),
    }
}

/// 2点間の LookAt 角度(Y-up 正準・atan2)。座標空間は呼び出し側が揃える。
///
/// `PlusX`: 0回転が +X を向く。`PlusY`: 0回転が +Y を向く(`angle - π/2`)。
pub fn look_at_angle(from: [f64; 2], to: [f64; 2], axis: LookAtAxis) -> f64 {
    let angle = (to[1] - from[1]).atan2(to[0] - from[0]);
    match axis {
        LookAtAxis::PlusX => angle,
        LookAtAxis::PlusY => angle - std::f64::consts::FRAC_PI_2,
    }
}

/// `rotation = look_at(self.center, target.center)` — `self_pos`/`target` は同一座標空間。
pub fn eval_look_at_rotation(
    self_pos: [f64; 2],
    target: LayerId,
    axis: LookAtAxis,
    resolved: &ResolvedLayerParams,
) -> Result<f64, ParamEvalError> {
    let target_pos = resolved
        .position(target)
        .ok_or(ParamEvalError::UnresolvedLookAt(target.get()))?;
    Ok(look_at_angle(self_pos, target_pos, axis))
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

/// rotation スロット用。LookAt なら `self_pos` を渡して角度へ落とす。
pub fn eval_rotation(
    param: &DocParam,
    self_pos: [f64; 2],
    t: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<f64, ParamEvalError> {
    match param {
        DocParam::LookAt { target, axis } => {
            eval_look_at_rotation(self_pos, *target, *axis, resolved)
        }
        other => eval_f64(other, t, tracks, resolved),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    fn approx(got: f64, want: f64) {
        let d = (got - want).abs();
        assert!(d < 1e-12, "got {got} want {want} (Δ={d})");
    }

    #[test]
    fn look_at_axis_affects_angle() {
        let target = LayerId::from_raw(1);
        let mut resolved = ResolvedLayerParams::default();
        let at = |resolved: &ResolvedLayerParams, axis| {
            eval_look_at_rotation([0.0, 0.0], target, axis, resolved).unwrap()
        };

        // self(0,0) → target(1,1): atan2(1,1)=π/4
        resolved.insert_position(target, [1.0, 1.0]);
        approx(at(&resolved, LookAtAxis::PlusX), FRAC_PI_4);
        approx(at(&resolved, LookAtAxis::PlusY), FRAC_PI_4 - FRAC_PI_2);

        // +X 方向(1,0): PlusX→0、PlusY→-π/2(0回転が+Y)
        resolved.insert_position(target, [1.0, 0.0]);
        approx(at(&resolved, LookAtAxis::PlusX), 0.0);
        approx(at(&resolved, LookAtAxis::PlusY), -FRAC_PI_2);

        // +Y 方向(0,1)
        resolved.insert_position(target, [0.0, 1.0]);
        approx(at(&resolved, LookAtAxis::PlusX), FRAC_PI_2);
        approx(at(&resolved, LookAtAxis::PlusY), 0.0);

        // 左(-1,0)
        resolved.insert_position(target, [-1.0, 0.0]);
        approx(at(&resolved, LookAtAxis::PlusX), PI);
    }

    #[test]
    fn look_at_unresolved_target_is_typed_error() {
        let target = LayerId::from_raw(99);
        let err = eval_look_at_rotation(
            [0.0, 0.0],
            target,
            LookAtAxis::PlusX,
            &ResolvedLayerParams::default(),
        )
        .unwrap_err();
        assert_eq!(err, ParamEvalError::UnresolvedLookAt(99));
    }

    #[test]
    fn eval_doc_param_look_at_requires_self() {
        let param = DocParam::LookAt {
            target: LayerId::from_raw(1),
            axis: LookAtAxis::PlusY,
        };
        let err = eval_doc_param(
            &param,
            RationalTime::ZERO,
            &DataTracks::new(),
            &ResolvedLayerParams::default(),
        )
        .unwrap_err();
        assert_eq!(err, ParamEvalError::LookAtRequiresSelfPosition);
    }
}
