//! DocParam/DocValue の型・有限性・補間検査。構造走査から切り離すため。

use crate::doc_keyframe::validate_interp;
use crate::doc_value::DocValue;
use crate::param::DocParam;
use crate::param_expect::{path_op_scalar, vec2_axis, ExpectedValueType, ParamConstraints};
use crate::schema::{StandardShape, VectorContent};
use crate::Document;

use super::DocumentError;

pub(crate) fn validate_param(
    doc: &Document,
    param: &DocParam,
    constraints: ParamConstraints,
    path: &str,
) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(v) => validate_value(v, constraints, path),
        DocParam::Keyframes(track) => {
            if track.keys().is_empty() {
                return Err(DocumentError::EmptyKeyframeTrack {
                    path: path.to_string(),
                });
            }
            let mut expected_kind: Option<&'static str> = None;
            for key in track.keys() {
                let kind = key.value.kind_name();
                match expected_kind {
                    None => expected_kind = Some(kind),
                    Some(prev) if prev != kind => {
                        return Err(DocumentError::KeyframeVariantMismatch {
                            path: path.to_string(),
                            expected: prev.to_string(),
                            got: kind.to_string(),
                        });
                    }
                    Some(_) => {}
                }
                validate_interp_at(path, &key.interp)?;
                validate_value(&key.value, constraints, path)?;
            }
            Ok(())
        }
        DocParam::Data { fallback, .. } => validate_value(fallback, constraints, path),
        DocParam::Vec2Axes { x, y } => {
            if constraints.expected != ExpectedValueType::Vec2 {
                return Err(DocumentError::ParamTypeMismatch {
                    path: path.to_string(),
                    expected: constraints.expected.name().to_string(),
                    got: "Vec2Axes".to_string(),
                });
            }
            validate_param(doc, x, vec2_axis(), &format!("{path}.x"))?;
            validate_param(doc, y, vec2_axis(), &format!("{path}.y"))
        }
        DocParam::LookAt { target, .. } => {
            if !constraints.allow_look_at {
                return Err(DocumentError::SpatialLinkNotAllowed {
                    path: path.to_string(),
                });
            }
            doc.require_layer(*target)
        }
        DocParam::Follow { target, offset } => {
            if !constraints.allow_follow {
                return Err(DocumentError::SpatialLinkNotAllowed {
                    path: path.to_string(),
                });
            }
            if !offset[0].is_finite() || !offset[1].is_finite() {
                return Err(DocumentError::NonFiniteValue {
                    path: format!("{path}.offset"),
                });
            }
            doc.require_layer(*target)
        }
    }
}

/// 未知plugin向け: 期待型なし。有限性・AssetRef存在・Bezierのみ。
pub(crate) fn validate_param_structure(
    doc: &Document,
    param: &DocParam,
    path: &str,
) -> Result<(), DocumentError> {
    match param {
        DocParam::Const(v) => validate_value_structure(v, path),
        DocParam::Keyframes(track) => {
            if track.keys().is_empty() {
                return Err(DocumentError::EmptyKeyframeTrack {
                    path: path.to_string(),
                });
            }
            let mut expected_kind: Option<&'static str> = None;
            for key in track.keys() {
                let kind = key.value.kind_name();
                match expected_kind {
                    None => expected_kind = Some(kind),
                    Some(prev) if prev != kind => {
                        return Err(DocumentError::KeyframeVariantMismatch {
                            path: path.to_string(),
                            expected: prev.to_string(),
                            got: kind.to_string(),
                        });
                    }
                    Some(_) => {}
                }
                validate_interp_at(path, &key.interp)?;
                validate_value_structure(&key.value, path)?;
            }
            Ok(())
        }
        DocParam::Data { fallback, .. } => validate_value_structure(fallback, path),
        DocParam::Vec2Axes { x, y } => {
            validate_param_structure(doc, x, &format!("{path}.x"))?;
            validate_param_structure(doc, y, &format!("{path}.y"))
        }
        DocParam::LookAt { target, .. } | DocParam::Follow { target, .. } => {
            doc.require_layer(*target)
        }
    }
}

pub(crate) fn validate_interp_at(
    path: &str,
    interp: &motolii_eval::Interp,
) -> Result<(), DocumentError> {
    validate_interp(interp).map_err(|e| match e {
        crate::doc_keyframe::DocKeyframeError::NonFiniteBezier => DocumentError::NonFiniteBezier {
            path: path.to_string(),
        },
        crate::doc_keyframe::DocKeyframeError::InvalidBezier { x1, x2 } => {
            DocumentError::InvalidBezier {
                path: path.to_string(),
                x1,
                x2,
            }
        }
        other => DocumentError::NonFiniteBezier {
            path: format!("{path} ({other})"),
        },
    })
}

/// ID採番前のDraft keyframe列: 空拒否・variant一致・値の構造検査。
pub(crate) fn validate_keyframe_draft_values(
    _doc: &Document,
    values: &[crate::doc_value::DocValue],
    path: &str,
) -> Result<(), DocumentError> {
    if values.is_empty() {
        return Err(DocumentError::EmptyKeyframeTrack {
            path: path.to_string(),
        });
    }
    let mut expected_kind: Option<&'static str> = None;
    for value in values {
        let kind = value.kind_name();
        match expected_kind {
            None => expected_kind = Some(kind),
            Some(prev) if prev != kind => {
                return Err(DocumentError::KeyframeVariantMismatch {
                    path: path.to_string(),
                    expected: prev.to_string(),
                    got: kind.to_string(),
                });
            }
            Some(_) => {}
        }
        validate_value_structure(value, path)?;
    }
    Ok(())
}

fn validate_value(
    value: &DocValue,
    constraints: ParamConstraints,
    path: &str,
) -> Result<(), DocumentError> {
    if !constraints.expected.matches(value) {
        return Err(DocumentError::ParamTypeMismatch {
            path: path.to_string(),
            expected: constraints.expected.name().to_string(),
            got: value.kind_name().to_string(),
        });
    }
    validate_value_structure(value, path)?;
    if constraints.unit_interval {
        match value {
            DocValue::F64(v) if !(0.0..=1.0).contains(v) => {
                return Err(DocumentError::ValueOutOfRange {
                    path: path.to_string(),
                });
            }
            DocValue::Color(c) if c.iter().any(|x| !(0.0..=1.0).contains(x)) => {
                return Err(DocumentError::ValueOutOfRange {
                    path: path.to_string(),
                });
            }
            _ => {}
        }
    }
    if let DocValue::F64(v) = value {
        if constraints.exclusive_min.is_some_and(|min| *v <= min) {
            return Err(DocumentError::ValueOutOfRange {
                path: path.to_string(),
            });
        }
        if constraints.min.is_some_and(|min| *v < min)
            || constraints.max.is_some_and(|max| *v > max)
        {
            return Err(DocumentError::ValueOutOfRange {
                path: path.to_string(),
            });
        }
        if constraints.integer && v.fract().abs() > f64::EPSILON {
            return Err(DocumentError::ValueOutOfRange {
                path: path.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_value_structure(value: &DocValue, path: &str) -> Result<(), DocumentError> {
    match value {
        DocValue::F64(v) => {
            if !v.is_finite() {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::Vec2(v) => {
            if v.iter().any(|x| !x.is_finite()) {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::Vec3(v) => {
            if v.iter().any(|x| !x.is_finite()) {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::Color(c) => {
            if c.iter().any(|x| !x.is_finite()) {
                return Err(DocumentError::NonFiniteValue {
                    path: path.to_string(),
                });
            }
        }
        DocValue::AssetRef(_) => {}
    }
    Ok(())
}

pub(super) fn validate_vector_content(
    doc: &Document,
    content: &VectorContent,
    path: &str,
) -> Result<(), DocumentError> {
    match content {
        VectorContent::StandardShape { shape } => match shape {
            StandardShape::Rect { width, height } | StandardShape::Ellipse { width, height } => {
                validate_param(doc, width, path_op_scalar(), &format!("{path}.width"))?;
                validate_param(doc, height, path_op_scalar(), &format!("{path}.height"))
            }
        },
        VectorContent::SvgAsset { .. } | VectorContent::TextPath { .. } => Ok(()),
        VectorContent::Group { children } => {
            for (i, child) in children.iter().enumerate() {
                validate_vector_content(doc, child, &format!("{path}.children[{i}]"))?;
            }
            Ok(())
        }
    }
}
