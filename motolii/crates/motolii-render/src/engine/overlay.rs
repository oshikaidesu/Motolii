//! Track Overlay を描く: このコマの塊から箱・印の形を組み、comp と同じ大きさの画布に描く(描くのは形の道 = fork の re_renderer)。
//! 欄は docs/reviews/2026-09-14-tracery2-port-spec.md の Box / Marker。

use crate::doc::core::CompSpec;
use crate::doc::store::analysis::BlobMark;
use crate::doc::store::overlay::{color_of, number_of, switch_of};
use crate::doc::store::{LayerId, Value};
use crate::doc::vector::{Brush, Contour, Dash, Fill, LineCap, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeNode, StarType, Stroke};
use crate::render::compositor::LayerContent;
use crate::render::engine::{Engine, EngineError};

type Params = [(String, Value)];

fn rgb(c: [f64; 4]) -> Rgb {
    Rgb { r: c[0], g: c[1], b: c[2] }
}

fn at(center: [f32; 2], rotation: f64, child: Shape) -> ShapeNode {
    ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform { position: Point { x: center[0] as f64, y: center[1] as f64 }, rotation, ..RepeaterTransform::IDENTITY },
        children: vec![ShapeNode::Leaf(child)],
    })
}

/// 箱 1 つ。Square / Circle は長辺に揃え、Custom Size は半径で決める。Gap は 4 辺(4 弧)の真ん中を残し角を抜く。
fn box_shape(params: &Params, mark: &BlobMark) -> Option<ShapeNode> {
    let kind = number_of(params, "box_shape").round() as i64;
    let mut size = [mark.size[0] as f64, mark.size[1] as f64];
    if matches!(kind, 1 | 3) { let side = size[0].max(size[1]); size = [side, side]; }
    if switch_of(params, "custom_size") { let d = number_of(params, "custom_radius").max(0.0) * 2.0; size = [d, d]; }
    let source = if matches!(kind, 2 | 3) { PathSource::Ellipse { size: Point { x: size[0], y: size[1] } } } else { PathSource::Rectangle { size: Point { x: size[0], y: size[1] } } };
    let stroke = switch_of(params, "box_stroke").then(|| {
        let c = color_of(params, "box_stroke_color");
        let perimeter = if matches!(kind, 2 | 3) { std::f64::consts::PI * (size[0] + size[1]) * 0.5 } else { 2.0 * (size[0] + size[1]) };
        let gap = switch_of(params, "box_gap").then(|| number_of(params, "box_gap_size").clamp(0.0, 0.95));
        Stroke {
            brush: Brush::Solid(rgb(c)),
            width: number_of(params, "box_stroke_width").max(0.0),
            opacity: number_of(params, "box_stroke_opacity").clamp(0.0, 1.0) * c[3],
            dash: gap.map(|g| { let quarter = perimeter / 4.0; Dash { pattern: vec![quarter * (1.0 - g), quarter * g], offset: quarter * g * 0.5 } }),
            ..Default::default()
        }
    });
    let fill = switch_of(params, "box_fill").then(|| {
        let c = color_of(params, "box_fill_color");
        Fill { brush: Brush::Solid(rgb(c)), opacity: number_of(params, "box_fill_opacity").clamp(0.0, 1.0) * c[3], ..Default::default() }
    });
    (stroke.is_some() || fill.is_some()).then(|| at(mark.center, 0.0, Shape { source, ops: Vec::new(), fill, stroke }))
}

/// 印 1 つ。Plus / Cross は 2 本の線、Polygon は正多角形(Polygon Fill なら塗り)。
fn marker_shape(params: &Params, mark: &BlobMark) -> ShapeNode {
    let c = color_of(params, "marker_color");
    let opacity = number_of(params, "marker_opacity").clamp(0.0, 1.0) * c[3];
    let size = number_of(params, "marker_size").max(0.0);
    let thickness = number_of(params, "marker_thickness").max(0.0);
    let rotation = number_of(params, "marker_rotation");
    let stroke = Stroke { brush: Brush::Solid(rgb(c)), width: thickness, opacity, cap: LineCap::Butt, ..Default::default() };
    let fill = Fill { brush: Brush::Solid(rgb(c)), opacity, ..Default::default() };
    let h = size * 0.5;
    let bars = |extra: f64| {
        let path = vec![Contour::open([Point { x: -h, y: 0.0 }, Point { x: h, y: 0.0 }]), Contour::open([Point { x: 0.0, y: -h }, Point { x: 0.0, y: h }])];
        at(mark.center, rotation + extra, Shape { source: PathSource::Bezier(path), ops: Vec::new(), fill: None, stroke: Some(stroke.clone()) })
    };
    match number_of(params, "marker_type").round() as i64 {
        0 => at(mark.center, 0.0, Shape { source: PathSource::Ellipse { size: Point { x: size, y: size } }, ops: Vec::new(), fill: Some(fill), stroke: None }),
        1 => bars(0.0),
        2 => bars(45.0),
        _ => {
            let filled = switch_of(params, "polygon_fill");
            let polygon = PathSource::PolyStar { points: number_of(params, "polygon_sides").round().clamp(3.0, 12.0), outer_radius: h, inner_radius: h, star_type: StarType::Polygon };
            at(mark.center, rotation, Shape { source: polygon, ops: Vec::new(), fill: filled.then_some(fill), stroke: (!filled).then_some(stroke) })
        }
    }
}

/// Grid(Tracery 2 の Grid 節): Edge は塊の箱の辺から立てた線(`overlay::edge_lines`、近い辺は 1 本に)、Cartesian は等間隔。
/// 線は画面の端から端まで。濃さは線ごと。
fn grid_shapes(params: &Params, marks: &[BlobMark], comp: [f64; 2]) -> Vec<ShapeNode> {
    let c = color_of(params, "grid_color");
    let base = number_of(params, "grid_opacity").clamp(0.0, 1.0) * c[3];
    let width = number_of(params, "grid_thickness").max(0.0);
    let line = |from: Point, to: Point, opacity: f64| ShapeNode::Leaf(Shape {
        source: PathSource::Bezier(vec![Contour::open([from, to])]),
        ops: Vec::new(),
        fill: None,
        stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width, opacity: base * opacity, cap: LineCap::Butt, ..Default::default() }),
    });
    let mut out = Vec::new();
    if number_of(params, "grid_mode").round() as i64 == 1 {
        let (columns, rows) = (number_of(params, "grid_columns").round().max(1.0), number_of(params, "grid_rows").round().max(1.0));
        for i in 0..=columns as i64 {
            let x = comp[0] * i as f64 / columns;
            out.push(line(Point { x, y: 0.0 }, Point { x, y: comp[1] }, 1.0));
        }
        for i in 0..=rows as i64 {
            let y = comp[1] * i as f64 / rows;
            out.push(line(Point { x: 0.0, y }, Point { x: comp[0], y }, 1.0));
        }
        return out;
    }
    let merge = number_of(params, "grid_merge").max(0.0) as f32;
    let xs: Vec<f32> = marks.iter().flat_map(|m| [m.center[0] - m.size[0] * 0.5, m.center[0] + m.size[0] * 0.5]).collect();
    let ys: Vec<f32> = marks.iter().flat_map(|m| [m.center[1] - m.size[1] * 0.5, m.center[1] + m.size[1] * 0.5]).collect();
    for (x, opacity) in crate::doc::store::overlay::edge_lines(&xs, merge) {
        out.push(line(Point { x: f64::from(x), y: 0.0 }, Point { x: f64::from(x), y: comp[1] }, f64::from(opacity)));
    }
    for (y, opacity) in crate::doc::store::overlay::edge_lines(&ys, merge) {
        out.push(line(Point { x: 0.0, y: f64::from(y) }, Point { x: comp[0], y: f64::from(y) }, f64::from(opacity)));
    }
    out
}

/// このコマの塊から、Grid → Box → Marker の順に形を組む(comp の座標)。
pub(crate) fn overlay_shapes(params: &Params, marks: &[BlobMark], comp: [f64; 2]) -> Vec<ShapeNode> {
    let mut out = Vec::new();
    if switch_of(params, "grid") {
        out.extend(grid_shapes(params, marks, comp));
    }
    if switch_of(params, "box") {
        out.extend(marks.iter().filter_map(|m| box_shape(params, m)));
    }
    if switch_of(params, "marker") {
        out.extend(marks.iter().map(|m| marker_shape(params, m)));
    }
    out
}

impl Engine {
    /// Track Overlay を持つ層の中身(comp 大、左上が層の位置)。Show Mask なら解析の二値。
    pub(super) fn overlay_content(&mut self, layer: LayerId, comp: CompSpec) -> Result<Option<(LayerContent, [f32; 2])>, EngineError> {
        let Some(frame) = self.overlay_frames.get(&layer) else { return Ok(None) };
        let natural = [comp.width as f32, comp.height as f32];
        if switch_of(&frame.params, "show_mask") {
            let Some((bits, w, h)) = frame.mask.clone() else { return Ok(None) };
            let bytes: Vec<u8> = bits.iter().flat_map(|b| {
                let v = half::f16::from_f32(if *b > 0 { 1.0 } else { 0.0 }).to_le_bytes();
                let one = half::f16::from_f32(1.0).to_le_bytes();
                [v[0], v[1], v[0], v[1], v[0], v[1], one[0], one[1]]
            }).collect();
            let texture = self.compositor.upload_rgba16f("motolii-overlay-mask", bytes, w, h)?;
            return Ok(Some((LayerContent::LinearTexture(texture), natural)));
        }
        let shapes = overlay_shapes(&frame.params, &frame.marks, [f64::from(comp.width), f64::from(comp.height)]);
        if shapes.is_empty() {
            return Ok(None);
        }
        let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        let texture = self.compositor.render_paths("track-overlay", &shapes, &canvas, 1.0, super::texture::raster_pixel_budget(comp))?;
        Ok(texture.map(|t| (LayerContent::Texture(t), natural)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, Value)]) -> Vec<(String, Value)> {
        pairs.iter().map(|(n, v)| ((*n).to_owned(), v.clone())).collect()
    }

    #[test]
    fn boxes_follow_the_shape_choice_and_markers_are_optional() {
        let marks = [BlobMark { id: 0, center: [100.0, 50.0], size: [40.0, 20.0], age: 0 }];
        let shapes = overlay_shapes(&params(&[]), &marks, [400.0, 300.0]);
        assert_eq!(shapes.len(), 1, "既定は箱だけ");
        let ShapeNode::Group(g) = &shapes[0] else { panic!() };
        assert_eq!((g.transform.position.x, g.transform.position.y), (100.0, 50.0));
        let ShapeNode::Leaf(leaf) = &g.children[0] else { panic!() };
        assert_eq!(leaf.source, PathSource::Rectangle { size: Point { x: 40.0, y: 20.0 } });
        let circle = overlay_shapes(&params(&[("box_shape", Value::F64(3.0)), ("marker", Value::F64(1.0))]), &marks, [400.0, 300.0]);
        assert_eq!(circle.len(), 2, "箱 + 印");
        let ShapeNode::Group(g) = &circle[0] else { panic!() };
        let ShapeNode::Leaf(leaf) = &g.children[0] else { panic!() };
        assert_eq!(leaf.source, PathSource::Ellipse { size: Point { x: 40.0, y: 40.0 } }, "Circle は長辺の円");
        let gapped = overlay_shapes(&params(&[("box_gap", Value::F64(1.0))]), &marks, [400.0, 300.0]);
        let ShapeNode::Group(g) = &gapped[0] else { panic!() };
        let ShapeNode::Leaf(leaf) = &g.children[0] else { panic!() };
        assert!(leaf.stroke.as_ref().and_then(|s| s.dash.as_ref()).is_some_and(|d| d.pattern.len() == 2), "Gap は破線");
        assert!(overlay_shapes(&params(&[("box", Value::F64(0.0))]), &marks, [400.0, 300.0]).is_empty());
    }

    #[test]
    fn an_edge_grid_raises_one_line_per_edge_across_the_screen() {
        let marks = [
            BlobMark { id: 0, center: [100.0, 50.0], size: [40.0, 20.0], age: 0 },
            BlobMark { id: 1, center: [102.0, 150.0], size: [40.0, 20.0], age: 0 },
        ];
        let grid = overlay_shapes(&params(&[("box", Value::F64(0.0)), ("grid", Value::F64(1.0)), ("grid_merge", Value::F64(10.0))]), &marks, [400.0, 300.0]);
        assert_eq!(grid.len(), 8, "left, right, top, bottom of each box");
        let xs: Vec<f64> = grid.iter().filter_map(|n| match n {
            ShapeNode::Leaf(Shape { source: PathSource::Bezier(path), .. }) if path[0].vertices[0].point.x == path[0].vertices[1].point.x => Some(path[0].vertices[0].point.x),
            _ => None,
        }).collect();
        assert!((xs[0] - xs[2]).abs() < 0.5, "the two left edges 2 px apart share one vertical line: {xs:?}");
        let ShapeNode::Leaf(first) = &grid[0] else { panic!() };
        let PathSource::Bezier(path) = &first.source else { panic!() };
        assert_eq!((path[0].vertices[0].point.y, path[0].vertices[1].point.y), (0.0, 300.0), "a line runs edge to edge of the screen");
    }
}
