
use crate::doc::vector::ops::{self, Affine};
use crate::doc::vector::{Canvas, Fill, Path, RepeaterTransform, Shape, Stroke, VectorError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShapeNode {
    Leaf(Shape),
    Group(ShapeGroup),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeGroup {
    pub transform: RepeaterTransform,
    pub children: Vec<ShapeNode>,
}

pub fn flatten(nodes: &[ShapeNode]) -> Result<Vec<Shape>, VectorError> {
    let mut out = Vec::new();
    flatten_into(nodes, &Affine::IDENTITY, &mut out)?;
    Ok(out)
}

fn flatten_into(
    nodes: &[ShapeNode],
    parent_world: &Affine,
    out: &mut Vec<Shape>,
) -> Result<(), VectorError> {
    for node in nodes {
        match node {
            ShapeNode::Leaf(shape) => {
                for instance in crate::doc::vector::resolve(shape)? {
                    out.push(Shape {
                        source: crate::doc::vector::PathSource::Bezier(transform_path(
                            &instance.path,
                            parent_world,
                        )),
                        ops: Vec::new(),
                        fill: shape.fill.as_ref().map(|f| Fill {
                            opacity: f.opacity * instance.opacity,
                            ..f.clone()
                        }),
                        stroke: shape.stroke.as_ref().map(|s| Stroke {
                            opacity: s.opacity * instance.opacity,
                            ..s.clone()
                        }),
                    });
                }
            }
            ShapeNode::Group(group) => {
                let own = ops::build_affine(&group.transform);
                let world = parent_world.mul(&own);
                flatten_into(&group.children, &world, out)?;
            }
        }
    }
    Ok(())
}

fn transform_path(path: &Path, m: &Affine) -> Path {
    path.iter()
        .map(|c| ops::apply_matrix_to_contour(c, m))
        .collect()
}

/// 形が占める範囲(vector 座標。ストロークの太さを含む)。空なら `None`。
pub fn content_bounds(nodes: &[ShapeNode]) -> Result<Option<[f64; 4]>, VectorError> {
    let mut acc: Option<[f64; 4]> = None;
    for shape in flatten(nodes)? {
        let grow = shape.stroke.as_ref().map_or(0.0, |s| s.width * 0.5);
        for instance in crate::doc::vector::resolve(&shape)? {
            let Some(b) = crate::doc::vector::raster::path_bounds(&instance.path) else {
                continue;
            };
            let b = [b[0] - grow, b[1] - grow, b[2] + grow, b[3] + grow];
            acc = Some(match acc {
                None => b,
                Some(a) => [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])],
            });
        }
    }
    Ok(acc)
}

/// 形が占める範囲だけの canvas。層の箱が中身に吸い付く(形の層の素材座標は、この canvas の左上が原点)。
/// 反アリアスのはみ出しを1画素見込む。
pub fn content_canvas(nodes: &[ShapeNode]) -> Result<Option<Canvas>, VectorError> {
    const AA: f64 = 1.0;
    let Some(b) = content_bounds(nodes)? else {
        return Ok(None);
    };
    let min_x = (b[0] - AA).floor();
    let min_y = (b[1] - AA).floor();
    let max_x = (b[2] + AA).ceil();
    let max_y = (b[3] + AA).ceil();
    Ok(Some(Canvas {
        width: ((max_x - min_x) as i64).max(1) as u32,
        height: ((max_y - min_y) as i64).max(1) as u32,
        origin_x: -min_x as i32,
        origin_y: -min_y as i32,
    }))
}

/// 輪郭だけを素材座標の原点のまわりで伸ばした形(線の太さは伸ばさない)。Blob Track の箱合わせと、並べる法の Fill。
pub fn stretch_outline(shapes: &[ShapeNode], stretch: [f32; 2]) -> Vec<ShapeNode> {
    use crate::doc::vector::{Composite, OpKind, Point, RepeaterTransform, ShapeOp};
    let op = ShapeOp::new(OpKind::Repeater {
        copies: 1.0,
        offset: 1.0,
        transform: RepeaterTransform { scale: Point { x: stretch[0] as f64, y: stretch[1] as f64 }, ..RepeaterTransform::IDENTITY },
        composite: Composite::Above,
        start_opacity: 1.0,
        end_opacity: 1.0,
    });
    fn push(node: &ShapeNode, op: &ShapeOp) -> ShapeNode {
        match node {
            ShapeNode::Leaf(shape) => { let mut shape = shape.clone(); shape.ops.push(op.clone()); ShapeNode::Leaf(shape) }
            ShapeNode::Group(group) => { let mut group = group.clone(); group.children = group.children.iter().map(|c| push(c, op)).collect(); ShapeNode::Group(group) }
        }
    }
    shapes.iter().map(|n| push(n, &op)).collect()
}

pub fn render_tree(nodes: &[ShapeNode], canvas: &Canvas) -> Result<crate::doc::vector::Raster, VectorError> {
    let mut pixmap = crate::doc::vector::raster::new_pixmap(canvas)?;
    let origin = crate::doc::vector::Point {
        x: canvas.origin_x as f64,
        y: canvas.origin_y as f64,
    };
    for shape in flatten(nodes)? {
        for instance in crate::doc::vector::resolve(&shape)? {
            crate::doc::vector::raster::draw(
                &mut pixmap,
                &instance.path,
                origin,
                shape.fill.as_ref(),
                shape.stroke.as_ref(),
                instance.opacity,
            );
        }
    }
    Ok(crate::doc::vector::raster::finish(pixmap, canvas))
}
