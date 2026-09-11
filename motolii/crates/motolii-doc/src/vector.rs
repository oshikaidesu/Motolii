
pub mod edit;
pub mod stack_edit;
mod geom;
mod group;
mod ops;
mod raster;

pub mod coverage;

pub mod text;

use serde::{Deserialize, Serialize};

pub use geom::{Contour, Path, Point, Vertex};
pub use group::{content_bounds, flatten, render_tree, ShapeGroup, ShapeNode};

use geom::{ellipse, polystar, rect};
pub use ops::Instance;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub source: PathSource,
    pub ops: Vec<ShapeOp>,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

impl Shape {
    pub fn new(source: PathSource) -> Self {
        Self {
            source,
            ops: Vec::new(),
            fill: None,
            stroke: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathSource {
    Bezier(Path),
    Rectangle { size: Point },
    Ellipse { size: Point },
    PolyStar {
        points: f64,
        outer_radius: f64,
        inner_radius: f64,
        star_type: StarType,
    },
}

impl PathSource {
    fn to_path(&self) -> Path {
        match self {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeOp {
    pub hidden: bool,
    pub kind: OpKind,
}

impl ShapeOp {
    pub fn new(kind: OpKind) -> Self {
        Self {
            hidden: false,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpKind {
    TrimPath {
        start: f64,
        end: f64,
        offset: f64,
        multiple: TrimMultiple,
    },
    Repeater {
        copies: f64,
        offset: f64,
        transform: RepeaterTransform,
        composite: Composite,
        start_opacity: f64,
        end_opacity: f64,
    },
    RoundedCorners {
        radius: f64,
    },
    PuckerBloat {
        amount: f64,
    },
    ZigZag {
        amplitude: f64,
        frequency: f64,
        point_type: PointType,
    },
    OffsetPath {
        amount: f64,
        join: LineJoin,
        miter_limit: f64,
    },
    Twist {
        angle: f64,
        center: Point,
    },
    /// AE の Wiggle Paths(Illustrator の Roughen は Points=Corner)。Lottie には出ない。
    Wiggle {
        size: f64,
        detail: f64,
        point_type: PointType,
        phase: f64,
        seed: u64,
    },
    /// Cavalry の Path Relax。
    Smooth {
        strength: f64,
        iterations: f64,
    },
    /// Cavalry の Add Divisions。
    Subdivide {
        divisions: f64,
    },
    /// Cavalry の Reverse Path。
    Reverse,
    /// Cavalry の Extend Open Paths。
    Extend {
        start: f64,
        end: f64,
    },
    /// Cavalry の Chop Path。
    Chop {
        length: f64,
        gap: f64,
    },
    /// Cavalry の Resample Path。
    Resample {
        spacing: f64,
        point_type: PointType,
    },
    /// Cavalry の Bend Deformer。
    Bend {
        angle: f64,
        center: Point,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RepeaterTransform {
    pub anchor: Point,
    pub position: Point,
    pub scale: Point,
    pub rotation: f64,
}

impl RepeaterTransform {
    pub const IDENTITY: RepeaterTransform = RepeaterTransform {
        anchor: Point::ZERO,
        position: Point::ZERO,
        scale: Point { x: 1.0, y: 1.0 },
        rotation: 0.0,
    };
}

impl Default for RepeaterTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Brush {
    Solid(Rgb),
    Gradient(Gradient),
}

impl Default for Brush {
    fn default() -> Self {
        Brush::Solid(Rgb::BLACK)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    pub kind: GradientType,
    pub start: Point,
    pub end: Point,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f64,
    pub color: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GradientType {
    #[default]
    Linear,
    Radial,
    /// 中心のまわりを一周(CSS の conic、Figma の angular)。start→end の向きが 0。
    Angular,
    /// 中心から菱形に広がる(Figma の diamond)。start→end が菱形の半対角。
    Diamond,
}

impl Gradient {
    /// 点が gradient のどこか(0..1)。描く側(GPU・tiny-skia)は全部これを読む。
    pub fn parameter(&self, p: Point) -> f64 {
        let d = self.end.sub(self.start);
        let len2 = d.dot(d);
        if len2 <= 0.0 { return 0.0; }
        let v = p.sub(self.start);
        let t = match self.kind {
            GradientType::Linear => v.dot(d) / len2,
            GradientType::Radial => (v.dot(v) / len2).sqrt(),
            GradientType::Angular => {
                let turn = (v.y.atan2(v.x) - d.y.atan2(d.x)) / std::f64::consts::TAU;
                turn - turn.floor()
            }
            GradientType::Diamond => {
                let len = len2.sqrt();
                let along = v.dot(d) / len;
                let across = (v.y * d.x - v.x * d.y) / len;
                (along.abs() + across.abs()) / len
            }
        };
        if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) }
    }

    /// 0..1 の位置の色。stop の間は直線で混ぜ、外は端の色。
    pub fn color_at(&self, t: f64) -> Rgb {
        let mut stops = self.stops.clone();
        stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        match (stops.first(), stops.last()) {
            (None, _) | (_, None) => Rgb::BLACK,
            (Some(first), _) if t <= first.offset => first.color,
            (_, Some(last)) if t >= last.offset => last.color,
            _ => {
                let i = stops.iter().position(|s| s.offset > t).unwrap_or(stops.len() - 1);
                let (a, b) = (&stops[i - 1], &stops[i]);
                let u = ((t - a.offset) / (b.offset - a.offset).max(f64::EPSILON)).clamp(0.0, 1.0);
                Rgb { r: a.color.r + (b.color.r - a.color.r) * u, g: a.color.g + (b.color.g - a.color.g) * u, b: a.color.b + (b.color.b - a.color.b) * u }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    pub brush: Brush,
    pub rule: FillRule,
    pub opacity: f64,
    pub hidden: bool,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            brush: Brush::default(),
            rule: FillRule::NonZero,
            opacity: 1.0,
            hidden: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub brush: Brush,
    pub width: f64,
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter_limit: f64,
    pub dash: Option<Dash>,
    pub opacity: f64,
    pub hidden: bool,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            brush: Brush::default(),
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 4.0,
            dash: None,
            opacity: 1.0,
            hidden: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dash {
    pub pattern: Vec<f64>,
    pub offset: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StarType {
    #[default]
    Star,
    Polygon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PointType {
    #[default]
    Corner,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Composite {
    #[default]
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrimMultiple {
    #[default]
    Simultaneously,
    Individually,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub premultiplied_rgba8: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VectorError {
    #[error("canvas size {width}x{height} cannot be rasterized")]
    CanvasSize { width: u32, height: u32 },
    #[error("offset-path is closed-contour only; an open contour reached it")]
    OpenPathOffset,
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
        path: shape.source.to_path(),
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
