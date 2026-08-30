
use motolii_store::{MaskMode, Path as EvalPath, ResolvedMask};
use motolii_vector::coverage::{self, Coverage};
use motolii_vector::{
    Brush, Canvas, Fill, FillRule, LineJoin, OpKind, PathSource, Raster, Rgb, Shape, ShapeOp,
    VectorError,
};

fn eval_path_to_vector_path(path: &EvalPath) -> motolii_vector::Path {
    let vertices = path
        .vertices
        .iter()
        .map(|v| motolii_vector::Vertex {
            point: motolii_vector::Point {
                x: v.point[0],
                y: v.point[1],
            },
            in_tangent: motolii_vector::Point {
                x: v.in_tangent[0],
                y: v.in_tangent[1],
            },
            out_tangent: motolii_vector::Point {
                x: v.out_tangent[0],
                y: v.out_tangent[1],
            },
        })
        .collect();
    vec![motolii_vector::Contour {
        vertices,
        closed: path.closed,
    }]
}

pub fn rasterize_mask_coverage(
    mask: &ResolvedMask,
    canvas: &Canvas,
) -> Result<Raster, VectorError> {
    let ops = (mask.expansion != 0.0).then(|| {
        vec![ShapeOp::new(OpKind::OffsetPath {
            amount: mask.expansion,
            join: LineJoin::Miter,
            miter_limit: 4.0,
        })]
    }).unwrap_or_default();
    let shape = Shape {
        source: PathSource::Bezier(eval_path_to_vector_path(&mask.shape)),
        ops,
        fill: Some(Fill {
            brush: Brush::Solid(Rgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            }),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }),
        stroke: None,
    };
    motolii_vector::render(&shape, canvas)
}

#[derive(Debug, thiserror::Error)]
pub enum MaskFoldError {
    #[error(transparent)]
    Rasterize(#[from] VectorError),
    #[error(transparent)]
    CoverageSize(#[from] coverage::CoverageSizeMismatch),
}

fn adjust_for_composition(raw: &Coverage, inverted: bool, opacity: f32) -> Coverage {
    let bytes = raw
        .bytes
        .iter()
        .map(|&b| {
            let v = if inverted { 255 - b } else { b };
            (v as f32 * opacity).round().clamp(0.0, 255.0) as u8
        })
        .collect();
    Coverage {
        width: raw.width,
        height: raw.height,
        bytes,
    }
}

fn identity_for(mode: MaskMode, width: u32, height: u32) -> Coverage {
    match mode {
        MaskMode::Add | MaskMode::Lighten => Coverage::empty(width, height),
        MaskMode::Subtract | MaskMode::Intersect | MaskMode::Darken | MaskMode::Difference => {
            Coverage::full(width, height)
        }
    }
}

fn apply_mode(
    mode: MaskMode,
    acc: &Coverage,
    mask: &Coverage,
) -> Result<Coverage, coverage::CoverageSizeMismatch> {
    match mode {
        MaskMode::Add => coverage::add(acc, mask),
        MaskMode::Subtract => coverage::subtract(acc, mask),
        MaskMode::Intersect => coverage::intersect(acc, mask),
        MaskMode::Lighten => coverage::lighten(acc, mask),
        MaskMode::Darken => coverage::darken(acc, mask),
        MaskMode::Difference => coverage::difference(acc, mask),
    }
}

pub fn fold_masks(masks: &[ResolvedMask], canvas: &Canvas) -> Result<Coverage, MaskFoldError> {
    let mut acc: Option<Coverage> = None;
    for mask in masks {
        let raster = rasterize_mask_coverage(mask, canvas)?;
        let raw = Coverage::from_raster_alpha(&raster);
        let adjusted = adjust_for_composition(&raw, mask.inverted, mask.opacity);
        let next = match acc.as_ref() {
            Some(prev) => apply_mode(mask.mode, prev, &adjusted)?,
            None => {
                let identity = identity_for(mask.mode, canvas.width, canvas.height);
                apply_mode(mask.mode, &identity, &adjusted)?
            }
        };
        acc = Some(next);
    }
    Ok(acc.unwrap_or_else(|| Coverage::full(canvas.width, canvas.height)))
}
