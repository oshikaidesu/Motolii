
pub mod edit;
pub mod stack_edit;
mod geom;
mod group;
mod ops;
mod raster;

pub mod coverage;

pub mod text;
pub mod morph;
pub mod strokes;

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
    pub fn to_linear(self) -> Rgb { Rgb { r: srgb_to_linear(self.r), g: srgb_to_linear(self.g), b: srgb_to_linear(self.b) } }
    pub fn to_srgb(self) -> Rgb { Rgb { r: linear_to_srgb(self.r), g: linear_to_srgb(self.g), b: linear_to_srgb(self.b) } }
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
    /// stop の間を色がどう渡るか。書類に書く定義で、鍵は打たない。
    #[serde(default)]
    pub blend: GradientBlend,
}

/// 2 つの stop の間の道。空間の一覧は CSS Color 4 の閉集合(sRGB・linear・Oklab・Oklch の短/長)と段階。
/// 顔料の混色(Kubelka–Munk)と HDR の空間は別の核なので、核が入った時に足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradientBlend {
    /// 表示の sRGB を直線で。Photoshop の Classic、CSS の legacy。
    Rgb,
    /// 光の量で直線に(linear sRGB)。
    LinearRgb,
    /// 知覚で直線に。Photoshop 2023 の Perceptual、CSS の既定。
    #[default]
    Oklab,
    /// 色相環を短い方へ回る。
    OklchShort,
    /// 色相環を長い方へ回る。
    OklchLong,
    /// 混ぜない。stop の中点で切り替わる。
    Steps,
}

impl GradientBlend {
    pub const ALL: [GradientBlend; 6] = [Self::Rgb, Self::LinearRgb, Self::Oklab, Self::OklchShort, Self::OklchLong, Self::Steps];
    pub fn name(self) -> &'static str {
        match self { Self::Rgb => "rgb", Self::LinearRgb => "linear_rgb", Self::Oklab => "oklab", Self::OklchShort => "oklch_short", Self::OklchLong => "oklch_long", Self::Steps => "steps" }
    }
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.name() == name)
    }
    /// 2 色の間の u(0..1)の色。
    pub fn mix(self, a: Rgb, b: Rgb, u: f64) -> Rgb {
        let u = u.clamp(0.0, 1.0);
        let lerp = |x: f64, y: f64| x + (y - x) * u;
        match self {
            Self::Rgb => Rgb { r: lerp(a.r, b.r), g: lerp(a.g, b.g), b: lerp(a.b, b.b) },
            Self::Steps => if u < 0.5 { a } else { b },
            Self::LinearRgb => {
                let (la, lb) = (a.to_linear(), b.to_linear());
                Rgb { r: lerp(la.r, lb.r), g: lerp(la.g, lb.g), b: lerp(la.b, lb.b) }.to_srgb()
            }
            Self::Oklab => {
                let (la, lb) = (oklab(a), oklab(b));
                from_oklab([lerp(la[0], lb[0]), lerp(la[1], lb[1]), lerp(la[2], lb[2])])
            }
            Self::OklchShort | Self::OklchLong => {
                let (la, lb) = (oklab(a), oklab(b));
                let (ca, cb) = (la[1].hypot(la[2]), lb[1].hypot(lb[2]));
                // 無彩色の端は相手の色相を借りる(弓なりに膨らまない)。
                let ha = if ca < 1e-4 { lb[2].atan2(lb[1]) } else { la[2].atan2(la[1]) };
                let hb = if cb < 1e-4 { ha } else { lb[2].atan2(lb[1]) };
                let mut d = hb - ha;
                let tau = std::f64::consts::TAU;
                d -= (d / tau).round() * tau;
                if matches!(self, Self::OklchLong) && d.abs() < std::f64::consts::PI && d != 0.0 { d -= d.signum() * tau; }
                let h = ha + d * u;
                let c = lerp(ca, cb);
                from_oklab([lerp(la[0], lb[0]), c * h.cos(), c * h.sin()])
            }
        }
    }
}

fn srgb_to_linear(v: f64) -> f64 { if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) } }
fn linear_to_srgb(v: f64) -> f64 { let v = v.clamp(0.0, 1.0); if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 } }
/// Ottosson 2020 の行列。
fn oklab(c: Rgb) -> [f64; 3] {
    let l = c.to_linear();
    let lm = (0.4122214708 * l.r + 0.5363325363 * l.g + 0.0514459929 * l.b).cbrt();
    let m = (0.2119034982 * l.r + 0.6806995451 * l.g + 0.1073969566 * l.b).cbrt();
    let s = (0.0883024619 * l.r + 0.2817188376 * l.g + 0.6299787005 * l.b).cbrt();
    [0.2104542553 * lm + 0.7936177850 * m - 0.0040720468 * s, 1.9779984951 * lm - 2.4285922050 * m + 0.4505937099 * s, 0.0259040371 * lm + 0.7827717662 * m - 0.8086757660 * s]
}
fn from_oklab([l, a, b]: [f64; 3]) -> Rgb {
    let lm = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (l - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    Rgb { r: 4.0767416621 * lm - 3.3077115913 * m + 0.2309699292 * s, g: -1.2684380046 * lm + 2.6097574011 * m - 0.3413193965 * s, b: -0.0041960863 * lm - 0.7034186147 * m + 1.7076147010 * s }.to_srgb()
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

    /// 描き手が sRGB の直線しか混ぜられない時(tiny-skia・Lottie)の stop の列。
    /// 直線以外の道は間を細かく刻んで渡す。嘘で達成。
    pub fn baked_stops(&self) -> Vec<GradientStop> {
        let mut stops = self.stops.clone();
        stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        if matches!(self.blend, GradientBlend::Rgb) || stops.len() < 2 { return stops; }
        let n = if matches!(self.blend, GradientBlend::Steps) { 1 } else { 12 };
        let mut out = Vec::with_capacity(stops.len() * n);
        for w in stops.windows(2) {
            let (a, b) = (w[0], w[1]);
            out.push(a);
            if matches!(self.blend, GradientBlend::Steps) {
                let mid = (a.offset + b.offset) * 0.5;
                out.push(GradientStop { offset: mid, color: a.color });
                out.push(GradientStop { offset: mid, color: b.color });
                continue;
            }
            for k in 1..n {
                let u = k as f64 / n as f64;
                out.push(GradientStop { offset: a.offset + (b.offset - a.offset) * u, color: self.blend.mix(a.color, b.color, u) });
            }
        }
        out.push(*stops.last().unwrap());
        out
    }

    /// 0..1 の位置の色。stop の間は blend の道で混ぜ、外は端の色。
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
                self.blend.mix(a.color, b.color, u)
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
