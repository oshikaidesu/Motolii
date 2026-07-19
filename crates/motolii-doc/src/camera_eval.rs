//! `CompCameraDoc` → runtime `CompCamera` 評価（D3f）。

use motolii_core::{CanonicalPoint, CompCamera, CompCameraError};
use motolii_eval::DataTracks;

use crate::eval_time::EvaluationTime;
use crate::param_eval::{eval_f64, eval_vec2, ParamEvalError, ResolvedLayerParams};
use crate::schema::CompCameraDoc;
use crate::Document;

#[derive(Debug, thiserror::Error)]
pub enum CameraEvalError {
    #[error(transparent)]
    Param(#[from] ParamEvalError),
    #[error(transparent)]
    Camera(#[from] CompCameraError),
}

/// `EvaluationTime.timeline_time` で composition camera を評価する。
pub fn eval_comp_camera_doc(
    doc: &Document,
    eval: EvaluationTime,
    tracks: &DataTracks,
) -> Result<CompCamera, CameraEvalError> {
    let resolved = ResolvedLayerParams::default();
    let t = eval.timeline_time;
    let CompCameraDoc::PlanarOrthographic {
        center,
        roll_radians,
        height,
    } = &doc.composition.camera;

    let center_v = eval_vec2(center, t, tracks, &resolved)?;
    let roll = eval_f64(roll_radians, t, tracks, &resolved)?;
    let h = eval_f64(height, t, tracks, &resolved)?;

    Ok(CompCamera::try_new(
        CanonicalPoint {
            x: center_v[0],
            y: center_v[1],
        },
        roll,
        h,
        doc.composition.aspect_num(),
        doc.composition.aspect_den(),
    )?)
}
