//! 形の操作 — 書いた形を道にし(`to_path`)、積んだ op を掛け(`resolve`)、絵にする(`render`)。
//! 型(`PathSource`・`ShapeOp`・`Shape`)は書類の値なのでコアに残り、**変換と適用はここ**
//! (2026-09-20 利用者裁定「変換機構はホストが担うべきでない。ファストパーティの立ち位置」)。

pub mod coverage;
pub mod ops;
pub mod raster;
pub mod strokes;

pub(crate) use ops::Affine;
pub use ops::Instance;
pub use raster::{Canvas, Raster};

use crate::doc::vector::*;
use crate::doc::vector::geom::{ellipse, polystar, rect};

/// 書いた形を道にする — 星・楕円・矩形、そして将来の SVG もここへ並ぶ。
pub fn to_path(source: &PathSource) -> Path {
    {
        match source {
            PathSource::Bezier(p) => p.clone(),
            PathSource::Rectangle { size } => rect(*size),
            PathSource::Ellipse { size } => ellipse(*size),
            PathSource::PolyStar {
                points,
                outer_radius,
                inner_radius,
                star_type,
            } => polystar(*points, *outer_radius, *inner_radius, *star_type),
        }
    }
}


impl Canvas {
    pub fn centered(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            origin_x: (width / 2) as i32,
            origin_y: (height / 2) as i32,
        }
    }
}


pub fn render(shape: &Shape, canvas: &Canvas) -> Result<Raster, VectorError> {
    render_tree(&[ShapeNode::Leaf(shape.clone())], canvas)
}

/// 輪郭ごとの純関数を、複製の全部へ。
fn each(instances: Vec<Instance>, f: impl Fn(&Path) -> Path) -> Vec<Instance> {
    instances.into_iter().map(|i| Instance { path: f(&i.path), opacity: i.opacity }).collect()
}

pub fn resolve(shape: &Shape) -> Result<Vec<Instance>, VectorError> {
    let mut instances = vec![Instance {
        path: crate::picture::shapes_ops::to_path(&shape.source),
        opacity: 1.0,
    }];
    for op in &shape.ops {
        if op.hidden {
            continue;
        }
        instances = match &op.kind {
            OpKind::TrimPath {
                start,
                end,
                offset,
                multiple,
            } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::trim(&i.path, *start, *end, *offset, *multiple),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::RoundedCorners { radius } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::round_corners(&i.path, *radius),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::PuckerBloat { amount } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::pucker_bloat(&i.path, *amount),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::ZigZag {
                amplitude,
                frequency,
                point_type,
            } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::zigzag(&i.path, *amplitude, *frequency, *point_type),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::OffsetPath {
                amount,
                join,
                miter_limit,
            } => instances
                .into_iter()
                .map(|i| {
                    Ok(Instance {
                        path: ops::offset_path(&i.path, *amount, *join, *miter_limit)?,
                        opacity: i.opacity,
                    })
                })
                .collect::<Result<Vec<_>, VectorError>>()?,
            OpKind::Twist { angle, center } => instances
                .into_iter()
                .map(|i| Instance {
                    path: ops::twist(&i.path, *angle, *center),
                    opacity: i.opacity,
                })
                .collect(),
            OpKind::Wiggle { size, detail, point_type, phase, seed } => each(instances, |p| ops::wiggle(p, *size, *detail, *point_type, *phase, *seed)),
            OpKind::Smooth { strength, iterations } => each(instances, |p| ops::smooth(p, *strength, *iterations)),
            OpKind::Subdivide { divisions } => each(instances, |p| ops::subdivide(p, *divisions)),
            OpKind::Reverse => each(instances, ops::reverse),
            OpKind::Extend { start, end } => each(instances, |p| ops::extend(p, *start, *end)),
            OpKind::Chop { length, gap } => each(instances, |p| ops::chop(p, *length, *gap)),
            OpKind::Resample { spacing, point_type } => each(instances, |p| ops::resample(p, *spacing, *point_type)),
            OpKind::Bend { angle, center } => each(instances, |p| ops::bend(p, *angle, *center)),
            OpKind::Oscillator { amplitude, frequency, offset, detail } => each(instances, |p| ops::oscillate(p, *amplitude, *frequency, *offset, *detail)),
            OpKind::Repeater {
                copies,
                offset,
                transform,
                composite,
                start_opacity,
                end_opacity,
            } => ops::repeater(
                &instances,
                *copies,
                *offset,
                transform,
                *composite,
                *start_opacity,
                *end_opacity,
            ),
        };
    }
    Ok(instances)
}

/// 形が占める範囲(vector 座標。ストロークの太さを含む)。空なら `None`。
pub fn content_bounds(nodes: &[ShapeNode]) -> Result<Option<[f64; 4]>, VectorError> {
    let mut acc: Option<[f64; 4]> = None;
    for shape in flatten(nodes)? {
        let grow = shape.stroke.as_ref().map_or(0.0, |s| s.width * 0.5);
        for instance in crate::picture::shapes_ops::resolve(&shape)? {
            let Some(b) = crate::picture::shapes_ops::raster::path_bounds(&instance.path) else {
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

pub fn render_tree(nodes: &[ShapeNode], canvas: &Canvas) -> Result<crate::picture::shapes_ops::Raster, VectorError> {
    let mut pixmap = crate::picture::shapes_ops::raster::new_pixmap(canvas)?;
    let origin = crate::doc::vector::Point {
        x: canvas.origin_x as f64,
        y: canvas.origin_y as f64,
    };
    for shape in flatten(nodes)? {
        for instance in crate::picture::shapes_ops::resolve(&shape)? {
            crate::picture::shapes_ops::raster::draw(
                &mut pixmap,
                &instance.path,
                origin,
                shape.fill.as_ref(),
                shape.stroke.as_ref(),
                instance.opacity,
            );
        }
    }
    Ok(crate::picture::shapes_ops::raster::finish(pixmap, canvas))
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
                for instance in crate::picture::shapes_ops::resolve(shape)? {
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
