
use crate::render::vector::ops::{self, Affine};
use crate::render::vector::{Canvas, Fill, Path, RepeaterTransform, Shape, Stroke, VectorError};
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
                for instance in crate::render::vector::resolve(shape)? {
                    out.push(Shape {
                        source: crate::render::vector::PathSource::Bezier(transform_path(
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

pub fn render_tree(nodes: &[ShapeNode], canvas: &Canvas) -> Result<crate::render::vector::Raster, VectorError> {
    let mut pixmap = crate::render::vector::raster::new_pixmap(canvas)?;
    let origin = crate::render::vector::Point {
        x: canvas.origin_x as f64,
        y: canvas.origin_y as f64,
    };
    for shape in flatten(nodes)? {
        for instance in crate::render::vector::resolve(&shape)? {
            crate::render::vector::raster::draw(
                &mut pixmap,
                &instance.path,
                origin,
                shape.fill.as_ref(),
                shape.stroke.as_ref(),
                instance.opacity,
            );
        }
    }
    Ok(crate::render::vector::raster::finish(pixmap, canvas))
}
