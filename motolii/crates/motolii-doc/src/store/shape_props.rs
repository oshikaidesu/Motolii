//! 形の元の値を層の property に。星の頂点数・半径、矩形と楕円の大きさは Inspector の欄になり、
//! 他の property と同じにキーが打てる。書類の形(`shapes`)は既定として残り、property が上書きする。

use crate::doc::eval::Value;
use crate::doc::store::{property, ShapeNode};
use crate::doc::vector::{Brush, Gradient, GradientType, PathSource, Point, Rgb, Stroke};

/// 線の無い形に線の色や太さを付けた時の太さ(Create の線と同じ)。
pub const DEFAULT_STROKE_WIDTH: f64 = 6.0;

/// 形の元(source)が占める範囲 `[x0, y0, x1, y1]`。矩形・楕円・星は原点中心、Bezier は端点と制御点の凸包。
/// gradient の軸はこの範囲に対する比で決まる。
pub fn source_bounds(source: &PathSource) -> [f64; 4] {
    match source {
        PathSource::Rectangle { size } | PathSource::Ellipse { size } => [-size.x * 0.5, -size.y * 0.5, size.x * 0.5, size.y * 0.5],
        PathSource::PolyStar { outer_radius, .. } => { let r = outer_radius.abs(); [-r, -r, r, r] }
        PathSource::Bezier(path) => {
            let mut b: Option<[f64; 4]> = None;
            for v in path.iter().flat_map(|c| &c.vertices) {
                for p in [v.point, v.point.add(v.in_tangent), v.point.add(v.out_tangent)] {
                    b = Some(match b { None => [p.x, p.y, p.x, p.y], Some(a) => [a[0].min(p.x), a[1].min(p.y), a[2].max(p.x), a[3].max(p.y)] });
                }
            }
            b.unwrap_or([0.0; 4])
        }
    }
}

/// gradient の軸を比で読む: 向き(°)、中心のずれ(bounds の半幅・半高に対する %)、広がり(%)。
/// 線形の広がり 100% は、その向きで bounds を端から端まで。放射の 100% は bounds の大きい方の半分を半径に。
pub struct GradientAxis { pub angle: f64, pub center: [f64; 2], pub spread: f64 }

fn half_extent(bounds: [f64; 4], angle: f64) -> f64 {
    let (w, h) = (bounds[2] - bounds[0], bounds[3] - bounds[1]);
    let (s, c) = angle.to_radians().sin_cos();
    (c.abs() * w + s.abs() * h) * 0.5
}

pub fn axis_of(source: &PathSource, g: &Gradient) -> GradientAxis {
    let b = source_bounds(source);
    let (hw, hh) = (((b[2] - b[0]) * 0.5).max(1e-6), ((b[3] - b[1]) * 0.5).max(1e-6));
    let c0 = Point { x: (b[0] + b[2]) * 0.5, y: (b[1] + b[3]) * 0.5 };
    let d = g.end.sub(g.start);
    let angle = d.y.atan2(d.x).to_degrees();
    let (mid, spread) = match g.kind {
        GradientType::Linear => (g.start.add(g.end).scale(0.5), d.length() * 0.5 / half_extent(b, angle).max(1e-6) * 100.0),
        GradientType::Radial | GradientType::Angular | GradientType::Diamond => (g.start, d.length() / hw.max(hh) * 100.0),
    };
    GradientAxis { angle, center: [(mid.x - c0.x) / hw * 100.0, (mid.y - c0.y) / hh * 100.0], spread }
}

/// 比から軸の 2 点へ。`axis_of` の逆。
pub fn axis_points(source: &PathSource, kind: GradientType, axis: &GradientAxis) -> (Point, Point) {
    let b = source_bounds(source);
    let (hw, hh) = ((b[2] - b[0]) * 0.5, (b[3] - b[1]) * 0.5);
    let c = Point { x: (b[0] + b[2]) * 0.5 + axis.center[0] / 100.0 * hw, y: (b[1] + b[3]) * 0.5 + axis.center[1] / 100.0 * hh };
    match kind {
        GradientType::Linear => {
            let (s, co) = axis.angle.to_radians().sin_cos();
            let half = axis.spread / 100.0 * half_extent(b, axis.angle);
            let d = Point { x: co * half, y: s * half };
            (c.sub(d), c.add(d))
        }
        GradientType::Radial | GradientType::Angular | GradientType::Diamond => {
            let (s, co) = axis.angle.to_radians().sin_cos();
            let r = axis.spread / 100.0 * hw.max(hh);
            (c, c.add(Point { x: co * r, y: s * r }))
        }
    }
}

/// Inspector の 1 行。`value` は property が無い時の既定(書類の形の値)。
pub struct ShapeRow {
    pub name: &'static str,
    pub label: &'static str,
    pub value: Value,
    pub range: Option<(f64, f64)>,
}

fn first_leaf(shapes: &[ShapeNode]) -> Option<&crate::doc::vector::Shape> {
    shapes.iter().find_map(|n| match n {
        ShapeNode::Leaf(s) => Some(s),
        ShapeNode::Group(g) => first_leaf(&g.children),
    })
}

/// 開いた 1 本の Bezier(Line・Bezier の recipe)の横幅。Length の既定。
fn open_width(source: &PathSource) -> Option<f64> {
    let PathSource::Bezier(path) = source else { return None };
    let [contour] = path.as_slice() else { return None };
    if contour.closed || contour.vertices.len() < 2 { return None; }
    let xs = contour.vertices.iter().map(|v| v.point.x);
    let (lo, hi) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| (lo.min(x), hi.max(x)));
    (hi - lo > f64::EPSILON).then_some(hi - lo)
}

/// 層の形が持つ欄は gradient の軸だけ。形の寸法は path が正本で欄を持たない(2026-08-29)。
/// 線は効果の責務で、ここには欄も色も無い。
pub fn rows(shapes: &[ShapeNode]) -> Vec<ShapeRow> {
    let mut rows = Vec::new();
    if let Some(leaf) = first_leaf(shapes) {
        if let Some(Brush::Gradient(g)) = leaf.fill.as_ref().map(|f| &f.brush) {
            let axis = axis_of(&leaf.source, g);
            rows.push(ShapeRow { name: property::FILL_ANGLE, label: "Angle", value: Value::F64(axis.angle), range: None });
            rows.push(ShapeRow { name: property::FILL_CENTER, label: "Center", value: Value::Vec2(axis.center), range: None });
            rows.push(ShapeRow { name: property::FILL_SPREAD, label: "Spread", value: Value::F64(axis.spread), range: Some((0.0, 1000.0)) });
        }
    }
    rows
}

/// property で上書きした形。無い欄は書類の値のまま。群の中まで届く。
pub fn apply(shapes: &[ShapeNode], get: &dyn Fn(&str) -> Option<Value>) -> Vec<ShapeNode> {
    let number = |name: &str, fallback: f64| match get(name) { Some(Value::F64(v)) if v.is_finite() => v, _ => fallback };
    let pair = |name: &str, fallback: Point| match get(name) { Some(Value::Vec2([x, y])) if x.is_finite() && y.is_finite() => Point { x, y }, _ => fallback };
    shapes.iter().map(|n| match n {
        ShapeNode::Leaf(shape) => {
            let mut shape = shape.clone();
            shape.source = match shape.source {
                PathSource::PolyStar { points, outer_radius, inner_radius, star_type } => PathSource::PolyStar {
                    points: number(property::SHAPE_POINTS, points).max(3.0),
                    outer_radius: number(property::SHAPE_OUTER_RADIUS, outer_radius),
                    inner_radius: number(property::SHAPE_INNER_RADIUS, inner_radius),
                    star_type,
                },
                PathSource::Rectangle { size } => PathSource::Rectangle { size: pair(property::SHAPE_SIZE, size) },
                PathSource::Ellipse { size } => PathSource::Ellipse { size: pair(property::SHAPE_SIZE, size) },
                PathSource::Bezier(path) => PathSource::Bezier(match (open_width(&PathSource::Bezier(path.clone())), get(property::SHAPE_LENGTH)) {
                    // 長さは左端を軸に x を伸縮する。接線も一緒に。
                    (Some(width), Some(Value::F64(length))) if length.is_finite() && length >= 0.0 => {
                        let left = path[0].vertices.iter().map(|v| v.point.x).fold(f64::INFINITY, f64::min);
                        let k = length / width;
                        path.iter().map(|c| crate::doc::vector::Contour { closed: c.closed, vertices: c.vertices.iter().map(|v| crate::doc::vector::Vertex {
                            point: Point { x: left + (v.point.x - left) * k, y: v.point.y },
                            in_tangent: Point { x: v.in_tangent.x * k, y: v.in_tangent.y },
                            out_tangent: Point { x: v.out_tangent.x * k, y: v.out_tangent.y },
                        }).collect() }).collect()
                    }
                    _ => path,
                }),
            };
            // gradient: 軸は比の property が書類の 2 点を上書きし、stop は番号ごとに色と位置を上書きする。
            if let Some(fill) = shape.fill.as_mut() {
                if let Brush::Gradient(g) = &mut fill.brush {
                    let axis = axis_of(&shape.source, g);
                    let angle = number(property::FILL_ANGLE, axis.angle);
                    let center = pair(property::FILL_CENTER, Point { x: axis.center[0], y: axis.center[1] });
                    let spread = number(property::FILL_SPREAD, axis.spread);
                    if angle != axis.angle || [center.x, center.y] != axis.center || spread != axis.spread {
                        let (start, end) = axis_points(&shape.source, g.kind, &GradientAxis { angle, center: [center.x, center.y], spread });
                        g.start = start;
                        g.end = end;
                    }
                    for (i, stop) in g.stops.iter_mut().enumerate() {
                        if let Some(Value::F64(o)) = get(&format!("{}{i}.offset", property::FILL_STOP_PREFIX)) { if o.is_finite() { stop.offset = o.clamp(0.0, 1.0); } }
                        if let Some(Value::Color(c)) = get(&format!("{}{i}.color", property::FILL_STOP_PREFIX)) { stop.color = Rgb { r: c[0], g: c[1], b: c[2] }; }
                    }
                }
            }
            if let Some(Value::Color(c)) = get(property::SHAPE_FILL_COLOR) {
                let fill = shape.fill.get_or_insert_with(Default::default);
                if matches!(fill.brush, Brush::Solid(_)) { fill.brush = Brush::Solid(Rgb { r: c[0], g: c[1], b: c[2] }); }
            }
            if let Some(Value::F64(width)) = get(property::SHAPE_STROKE_WIDTH) {
                if width.is_finite() {
                    if width <= 0.0 {
                        shape.stroke = None;
                    } else {
                        let mut stroke = shape.stroke.take().unwrap_or_else(|| Stroke { brush: Brush::Solid(Rgb::BLACK), ..Stroke::default() });
                        stroke.width = width;
                        shape.stroke = Some(stroke);
                    }
                }
            }
            ShapeNode::Leaf(shape)
        }
        ShapeNode::Group(group) => {
            let mut group = group.clone();
            group.children = apply(&group.children, get);
            ShapeNode::Group(group)
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::vector::{Shape, StarType};

    fn star() -> Vec<ShapeNode> {
        vec![ShapeNode::Leaf(Shape::new(PathSource::PolyStar { points: 5.0, outer_radius: 100.0, inner_radius: 50.0, star_type: StarType::Star }))]
    }

    /// 欄は形の種類で決まり、既定は書類の値。property があればそれが勝ち、無い欄は書類のまま。
    #[test]
    fn properties_override_the_documents_shape_values() {
        assert!(super::rows(&star()).is_empty(), "形の寸法は欄にならない");
        let shown = apply(&star(), &|name| (name == property::SHAPE_POINTS).then_some(Value::F64(7.0)));
        let ShapeNode::Leaf(leaf) = &shown[0] else { panic!("葉") };
        assert_eq!(leaf.source, PathSource::PolyStar { points: 7.0, outer_radius: 100.0, inner_radius: 50.0, star_type: StarType::Star });
        assert_eq!(apply(&star(), &|_| None), star());
        let rect = vec![ShapeNode::Leaf(Shape::new(PathSource::Rectangle { size: Point { x: 10.0, y: 20.0 } }))];
        let shown = apply(&rect, &|_| Some(Value::Vec2([30.0, 40.0])));
        assert!(matches!(&shown[0], ShapeNode::Leaf(s) if s.source == PathSource::Rectangle { size: Point { x: 30.0, y: 40.0 } }));
    }

    /// gradient の軸は比: 比 → 点 → 比 が閉じ、property で向き・中心・広がりが動き、形の大きさに付いてくる。
    #[test]
    fn gradient_axis_is_a_ratio_of_the_shapes_bounds() {
        use crate::doc::vector::{Fill, GradientStop};
        let source = PathSource::Rectangle { size: Point { x: 100.0, y: 50.0 } };
        let g = Gradient { kind: GradientType::Linear, start: Point { x: -50.0, y: 0.0 }, end: Point { x: 50.0, y: 0.0 }, stops: vec![GradientStop { offset: 0.0, color: Rgb::BLACK }, GradientStop { offset: 1.0, color: Rgb { r: 1.0, g: 1.0, b: 1.0 } }] };
        let axis = axis_of(&source, &g);
        assert!((axis.angle).abs() < 1e-9 && axis.center == [0.0, 0.0] && (axis.spread - 100.0).abs() < 1e-9, "{} {:?} {}", axis.angle, axis.center, axis.spread);
        assert_eq!(axis_points(&source, GradientType::Linear, &axis), (g.start, g.end));
        let mut shape = crate::doc::vector::Shape::new(source.clone());
        shape.fill = Some(Fill { brush: Brush::Gradient(g.clone()), ..Fill::default() });
        let shown = apply(&[ShapeNode::Leaf(shape)], &|n| match n {
            property::FILL_ANGLE => Some(Value::F64(90.0)),
            property::FILL_SPREAD => Some(Value::F64(50.0)),
            property::SHAPE_SIZE => Some(Value::Vec2([200.0, 100.0])),
            "fill.stop.1.color" => Some(Value::Color([1.0, 0.0, 0.0, 1.0])),
            _ => None,
        });
        let ShapeNode::Leaf(leaf) = &shown[0] else { panic!("葉") };
        let Some(Fill { brush: Brush::Gradient(g2), .. }) = &leaf.fill else { panic!("gradient") };
        // 90°: 縦向き。大きさ 200×100 の半高 50 の半分 = 25 が半長。形の大きさに付いてくる。
        assert!((g2.start.x).abs() < 1e-9 && (g2.start.y + 25.0).abs() < 1e-9 && (g2.end.y - 25.0).abs() < 1e-9, "{:?} {:?}", g2.start, g2.end);
        assert_eq!(g2.stops[1].color, Rgb { r: 1.0, g: 0.0, b: 0.0 });
        let radial = Gradient { kind: GradientType::Radial, ..g.clone() };
        let axis = axis_of(&source, &radial);
        assert_eq!(axis_points(&source, GradientType::Radial, &axis), (radial.start, radial.end));
    }

    /// 角度と菱形: 一周で 0→1、菱形の角で 1。描く側は全部この 1 つの関数を読む。
    #[test]
    fn angular_and_diamond_parameters_close_the_circle_and_the_diamond() {
        use crate::doc::vector::GradientStop;
        let stops = vec![GradientStop { offset: 0.0, color: Rgb::BLACK }, GradientStop { offset: 1.0, color: Rgb { r: 1.0, g: 1.0, b: 1.0 } }];
        let angular = Gradient { kind: GradientType::Angular, start: Point::ZERO, end: Point { x: 10.0, y: 0.0 }, stops: stops.clone() };
        assert!((angular.parameter(Point { x: 0.0, y: 10.0 }) - 0.25).abs() < 1e-9);
        assert!((angular.parameter(Point { x: -10.0, y: 0.0 }) - 0.5).abs() < 1e-9);
        assert!(angular.parameter(Point { x: 10.0, y: -0.001 }) > 0.99);
        let diamond = Gradient { kind: GradientType::Diamond, start: Point::ZERO, end: Point { x: 10.0, y: 0.0 }, stops };
        assert!((diamond.parameter(Point { x: 5.0, y: 5.0 }) - 1.0).abs() < 1e-9);
        assert!((diamond.parameter(Point { x: 0.0, y: 10.0 }) - 1.0).abs() < 1e-9);
        assert!((diamond.parameter(Point { x: 2.5, y: 0.0 }) - 0.25).abs() < 1e-9);
        assert_eq!(diamond.color_at(0.5), Rgb { r: 0.5, g: 0.5, b: 0.5 });
    }

    /// 線の太さと長さは property で上書きできるが欄は持たない。0 で線が消え、線の無い形に付ければ黒い線が生える。
    #[test]
    fn stroke_width_and_length_override_without_rows() {
        let grown = apply(&star(), &|n| (n == property::SHAPE_STROKE_WIDTH).then_some(Value::F64(3.0)));
        let ShapeNode::Leaf(leaf) = &grown[0] else { panic!("葉") };
        assert_eq!(leaf.stroke.as_ref().map(|s| (s.width, s.brush.clone())), Some((3.0, Brush::Solid(Rgb::BLACK))));
        let line = vec![ShapeNode::Leaf(crate::doc::vector::Shape { stroke: Some(Stroke { width: 6.0, ..Stroke::default() }), ..crate::doc::vector::Shape::new(PathSource::Bezier(vec![crate::doc::vector::Contour::open([Point { x: 10.0, y: 0.0 }, Point { x: 110.0, y: 0.0 }])])) })];
        assert!(super::rows(&line).is_empty());
        let longer = apply(&line, &|n| match n { property::SHAPE_LENGTH => Some(Value::F64(250.0)), property::SHAPE_STROKE_WIDTH => Some(Value::F64(0.0)), _ => None });
        let ShapeNode::Leaf(leaf) = &longer[0] else { panic!("葉") };
        let PathSource::Bezier(path) = &leaf.source else { panic!("線") };
        assert_eq!(path[0].vertices[1].point, Point { x: 260.0, y: 0.0 });
        assert!(leaf.stroke.is_none(), "太さ 0 は線無し");
    }
}
