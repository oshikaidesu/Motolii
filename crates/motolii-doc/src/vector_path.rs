//! VectorRecipe → pathgeom::Path。既存 StandardShape だけを同じ Path に載せる。

use motolii_core::RationalTime;
use motolii_eval::DataTracks;

use crate::param_eval::{eval_f64, ParamEvalError, ResolvedLayerParams};
use crate::pathgeom::{self, Path};
use crate::schema::{StandardShape, VectorContent, VectorRecipe};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum VectorPathError {
    #[error("unsupported vector content")]
    Unsupported,
    #[error(transparent)]
    Param(#[from] ParamEvalError),
}

/// Rect/Ellipse を Path へ lower する。modifiers・SvgAsset・TextPath・Group は未接続のまま。
pub fn eval_vector_recipe_path(
    recipe: &VectorRecipe,
    timeline_time: RationalTime,
    tracks: &DataTracks,
    resolved: &ResolvedLayerParams,
) -> Result<Path, VectorPathError> {
    if !recipe.modifiers.is_empty() {
        return Err(VectorPathError::Unsupported);
    }
    let VectorContent::StandardShape { shape } = &recipe.content else {
        return Err(VectorPathError::Unsupported);
    };
    Ok(match shape {
        StandardShape::Rect { width, height } => pathgeom::rect(
            eval_f64(width, timeline_time, tracks, resolved)?,
            eval_f64(height, timeline_time, tracks, resolved)?,
        ),
        StandardShape::Ellipse { width, height } => pathgeom::ellipse(
            eval_f64(width, timeline_time, tracks, resolved)?,
            eval_f64(height, timeline_time, tracks, resolved)?,
        ),
    })
}
