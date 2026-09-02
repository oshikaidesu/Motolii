
use tiny_skia::{
    FillRule as TsFillRule, GradientStop as TsStop, LineCap as TsCap, LineJoin as TsJoin,
    LinearGradient, Paint, PathBuilder, Pixmap, RadialGradient, Shader, SpreadMode,
    Stroke as TsStroke, StrokeDash, Transform,
};

use crate::doc::vector::geom::{is_straight, Contour, Path, Point};
use crate::doc::vector::{
    Brush, Canvas, Fill, FillRule, Gradient, GradientType, LineCap, LineJoin, Raster, Stroke,
    VectorError,
};

fn to_tiny_skia(path: &Path, origin: Point) -> Option<tiny_skia::Path> {
    let mut b = PathBuilder::new();
    let mut any = false;
    for c in path {
        if c.vertices.len() < 2 {
            continue;
        }
        any = true;
        emit_contour(&mut b, c, origin);
    }
    if !any {
        return None;
    }
    b.finish()
}

fn emit_contour(b: &mut PathBuilder, c: &Contour, origin: Point) {
    let at = |p: Point| ((p.x + origin.x) as f32, (p.y + origin.y) as f32);
    let n = c.vertices.len();
    let (x0, y0) = at(c.vertices[0].point);
    b.move_to(x0, y0);
    let edges = if c.closed { n } else { n - 1 };
    for i in 0..edges {
        let v0 = &c.vertices[i];
        let v1 = &c.vertices[(i + 1) % n];
        let (x, y) = at(v1.point);
        if is_straight(v0, v1) {
            b.line_to(x, y);
        } else {
            let (cx1, cy1) = at(v0.point.add(v0.out_tangent));
            let (cx2, cy2) = at(v1.point.add(v1.in_tangent));
            b.cubic_to(cx1, cy1, cx2, cy2, x, y);
        }
    }
    if c.closed {
        b.close();
    }
}

/// tiny-skia に測らせる(自前の bbox は持たない)。
pub(crate) fn path_bounds(path: &Path) -> Option<[f64; 4]> {
    let built = to_tiny_skia(path, Point { x: 0.0, y: 0.0 })?;
    let b = built.compute_tight_bounds()?;
    Some([b.left() as f64, b.top() as f64, b.right() as f64, b.bottom() as f64])
}

fn color_of(c: crate::doc::vector::Rgb, alpha: f64) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(
        clamp01(c.r) as f32,
        clamp01(c.g) as f32,
        clamp01(c.b) as f32,
        clamp01(alpha) as f32,
    )
    .unwrap_or(tiny_skia::Color::TRANSPARENT)
}

fn paint_for(brush: &Brush, origin: Point, alpha: f64) -> Paint<'static> {
    let shader = match brush {
        Brush::Solid(c) => Shader::SolidColor(color_of(*c, alpha)),
        Brush::Gradient(g) => gradient_shader(g, origin, alpha)
            .unwrap_or(Shader::SolidColor(tiny_skia::Color::TRANSPARENT)),
    };
    Paint {
        shader,
        anti_alias: true,
        ..Paint::default()
    }
}

fn gradient_shader(g: &Gradient, origin: Point, alpha: f64) -> Option<Shader<'static>> {
    let at = |p: Point| tiny_skia::Point::from_xy((p.x + origin.x) as f32, (p.y + origin.y) as f32);
    let mut sorted_stops = g.stops.clone();
    sorted_stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    let stops: Vec<TsStop> = sorted_stops
        .iter()
        .map(|s| TsStop::new(clamp01(s.offset) as f32, color_of(s.color, alpha)))
        .collect();
    match g.kind {
        GradientType::Linear => LinearGradient::new(
            at(g.start),
            at(g.end),
            stops,
            SpreadMode::Pad,
            Transform::identity(),
        ),
        GradientType::Radial => {
            let radius = g.end.sub(g.start).length() as f32;
            RadialGradient::new(
                at(g.start),
                at(g.start),
                radius,
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            )
        }
    }
}

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

impl From<FillRule> for TsFillRule {
    fn from(r: FillRule) -> Self {
        match r {
            FillRule::NonZero => TsFillRule::Winding,
            FillRule::EvenOdd => TsFillRule::EvenOdd,
        }
    }
}

impl From<LineCap> for TsCap {
    fn from(c: LineCap) -> Self {
        match c {
            LineCap::Butt => TsCap::Butt,
            LineCap::Round => TsCap::Round,
            LineCap::Square => TsCap::Square,
        }
    }
}

impl From<LineJoin> for TsJoin {
    fn from(j: LineJoin) -> Self {
        match j {
            LineJoin::Miter => TsJoin::Miter,
            LineJoin::Round => TsJoin::Round,
            LineJoin::Bevel => TsJoin::Bevel,
        }
    }
}

pub(crate) fn new_pixmap(canvas: &Canvas) -> Result<Pixmap, VectorError> {
    Pixmap::new(canvas.width, canvas.height).ok_or(VectorError::CanvasSize {
        width: canvas.width,
        height: canvas.height,
    })
}

pub(crate) fn draw(
    pixmap: &mut Pixmap,
    path: &Path,
    origin: Point,
    fill: Option<&Fill>,
    stroke: Option<&Stroke>,
    weight: f64,
) {
    let Some(ts_path) = to_tiny_skia(path, origin) else {
        return;
    };
    if let Some(f) = fill.filter(|f| !f.hidden) {
        let paint = paint_for(&f.brush, origin, f.opacity * weight);
        pixmap.fill_path(&ts_path, &paint, f.rule.into(), Transform::identity(), None);
    }
    if let Some(s) = stroke.filter(|s| !s.hidden && s.width > 0.0) {
        let paint = paint_for(&s.brush, origin, s.opacity * weight);
        let ts_stroke = TsStroke {
            width: s.width as f32,
            miter_limit: s.miter_limit as f32,
            line_cap: s.cap.into(),
            line_join: s.join.into(),
            dash: s.dash.as_ref().and_then(|d| {
                StrokeDash::new(
                    d.pattern.iter().map(|v| *v as f32).collect(),
                    d.offset as f32,
                )
            }),
        };
        pixmap.stroke_path(&ts_path, &paint, &ts_stroke, Transform::identity(), None);
    }
}

pub(crate) fn finish(pixmap: Pixmap, canvas: &Canvas) -> Raster {
    Raster {
        width: canvas.width,
        height: canvas.height,
        premultiplied_rgba8: pixmap.take(),
    }
}
