
use crate::doc::store::ShapeNode;
use crate::render::vector::{Canvas, Raster, VectorError};

pub fn rasterize_shapes(
    shapes: &[ShapeNode],
    canvas: &Canvas,
) -> Result<Option<Raster>, VectorError> {
    if shapes.is_empty() {
        return Ok(None);
    }
    Ok(Some(crate::render::vector::render_tree(shapes, canvas)?))
}
