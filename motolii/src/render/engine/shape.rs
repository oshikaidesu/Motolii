
use motolii_store::ShapeNode;
use motolii_vector::{Canvas, Raster, VectorError};

pub fn rasterize_shapes(
    shapes: &[ShapeNode],
    canvas: &Canvas,
) -> Result<Option<Raster>, VectorError> {
    if shapes.is_empty() {
        return Ok(None);
    }
    Ok(Some(motolii_vector::render_tree(shapes, canvas)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_vector::{Brush, Fill, FillRule, PathSource, Point, Rgb, Shape, Stroke};

    fn canvas(w: u32, h: u32) -> Canvas {
        Canvas::centered(w, h)
    }

    fn visible_pixels(raster: &Raster) -> usize {
        raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .count()
    }

    fn filled_rect(size: f64, rgb: Rgb) -> Shape {
        Shape {
            source: PathSource::Rectangle {
                size: Point { x: size, y: size },
            },
            ops: Vec::new(),
            fill: Some(Fill {
                brush: Brush::Solid(rgb),
                rule: FillRule::NonZero,
                opacity: 1.0,
                hidden: false,
            }),
            stroke: None,
        }
    }

    #[test]
    fn empty_shape_list_yields_no_texture() {
        let result = rasterize_shapes(&[], &canvas(64, 64)).expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn rectangle_with_fill_produces_visible_pixels() {
        let shape = filled_rect(
            40.0,
            Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        );
        let raster = rasterize_shapes(&[ShapeNode::Leaf(shape)], &canvas(64, 64))
            .expect("render")
            .expect("非空の shape 列は Some のはず");
        assert!(visible_pixels(&raster) > 1_000);
        assert!(raster
            .premultiplied_rgba8
            .chunks_exact(4)
            .any(|p| p[0] > 200 && p[1] == 0 && p[2] == 0 && p[3] > 200));
    }

    #[test]
    fn ellipse_with_stroke_only_produces_visible_pixels() {
        let shape = Shape {
            source: PathSource::Ellipse {
                size: Point { x: 40.0, y: 40.0 },
            },
            ops: Vec::new(),
            fill: None,
            stroke: Some(Stroke {
                brush: Brush::Solid(Rgb {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                }),
                width: 6.0,
                opacity: 1.0,
                ..Stroke::default()
            }),
        };
        let raster = rasterize_shapes(&[ShapeNode::Leaf(shape)], &canvas(64, 64))
            .expect("render")
            .expect("非空の shape 列は Some のはず");
        assert!(visible_pixels(&raster) > 50);
    }

    #[test]
    fn shape_is_centered_on_the_canvas_not_anchored_to_the_top_left() {
        let shape = filled_rect(
            20.0,
            Rgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
        );
        let raster = rasterize_shapes(&[ShapeNode::Leaf(shape)], &canvas(64, 64))
            .expect("render")
            .expect("非空の shape 列は Some のはず");
        let pixel = |x: u32, y: u32| -> [u8; 4] {
            let i = ((y * raster.width + x) * 4) as usize;
            [
                raster.premultiplied_rgba8[i],
                raster.premultiplied_rgba8[i + 1],
                raster.premultiplied_rgba8[i + 2],
                raster.premultiplied_rgba8[i + 3],
            ]
        };
        assert!(pixel(32, 32)[3] > 200, "canvas 中央には塗りが乗るはず");
        assert_eq!(pixel(1, 1)[3], 0, "canvas 左上隅は 20x20 の矩形の外のはず");
    }
}
