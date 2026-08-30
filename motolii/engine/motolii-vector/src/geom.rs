
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub(crate) fn add(self, o: Point) -> Point {
        Point {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }

    pub(crate) fn sub(self, o: Point) -> Point {
        Point {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }

    pub(crate) fn scale(self, s: f64) -> Point {
        Point {
            x: self.x * s,
            y: self.y * s,
        }
    }

    pub(crate) fn dot(self, o: Point) -> f64 {
        self.x * o.x + self.y * o.y
    }

    pub(crate) fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub(crate) fn normalized(self) -> Point {
        let l = self.length();
        if l < f64::EPSILON {
            Point::ZERO
        } else {
            self.scale(1.0 / l)
        }
    }

    pub(crate) fn rotate(self, angle: f64) -> Point {
        let (s, c) = angle.sin_cos();
        Point {
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub point: Point,
    pub in_tangent: Point,
    pub out_tangent: Point,
}

impl Vertex {
    pub fn corner(point: Point) -> Self {
        Self {
            point,
            in_tangent: Point::ZERO,
            out_tangent: Point::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Contour {
    pub vertices: Vec<Vertex>,
    pub closed: bool,
}

impl Contour {
    pub fn closed(points: impl IntoIterator<Item = Point>) -> Self {
        Self {
            vertices: points.into_iter().map(Vertex::corner).collect(),
            closed: true,
        }
    }

    pub fn open(points: impl IntoIterator<Item = Point>) -> Self {
        Self {
            vertices: points.into_iter().map(Vertex::corner).collect(),
            closed: false,
        }
    }
}

pub type Path = Vec<Contour>;

pub(crate) fn rect(size: Point) -> Path {
    let hx = size.x * 0.5;
    let hy = size.y * 0.5;
    vec![Contour::closed([
        Point { x: -hx, y: -hy },
        Point { x: hx, y: -hy },
        Point { x: hx, y: hy },
        Point { x: -hx, y: hy },
    ])]
}

pub(crate) fn ellipse(size: Point) -> Path {
    const KAPPA: f64 = 0.552_284_749_830_793_6;
    let rx = size.x * 0.5;
    let ry = size.y * 0.5;
    let kx = rx * KAPPA;
    let ky = ry * KAPPA;
    vec![Contour {
        closed: true,
        vertices: vec![
            Vertex {
                point: Point { x: rx, y: 0.0 },
                in_tangent: Point { x: 0.0, y: -ky },
                out_tangent: Point { x: 0.0, y: ky },
            },
            Vertex {
                point: Point { x: 0.0, y: ry },
                in_tangent: Point { x: kx, y: 0.0 },
                out_tangent: Point { x: -kx, y: 0.0 },
            },
            Vertex {
                point: Point { x: -rx, y: 0.0 },
                in_tangent: Point { x: 0.0, y: ky },
                out_tangent: Point { x: 0.0, y: -ky },
            },
            Vertex {
                point: Point { x: 0.0, y: -ry },
                in_tangent: Point { x: -kx, y: 0.0 },
                out_tangent: Point { x: kx, y: 0.0 },
            },
        ],
    }]
}

pub(crate) fn polystar(
    points: f64,
    outer_radius: f64,
    inner_radius: f64,
    star_type: crate::StarType,
) -> Path {
    let n = points.max(0.0).round() as usize;
    if n < 3 {
        return Path::new();
    }
    let start = -std::f64::consts::FRAC_PI_2;
    let at = |angle: f64, radius: f64| {
        Vertex::corner(Point {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        })
    };
    let vertices = match star_type {
        crate::StarType::Polygon => {
            let step = std::f64::consts::TAU / n as f64;
            (0..n)
                .map(|i| at(start + step * i as f64, outer_radius))
                .collect()
        }
        crate::StarType::Star => {
            let step = std::f64::consts::PI / n as f64;
            (0..n * 2)
                .map(|i| {
                    let radius = if i % 2 == 0 {
                        outer_radius
                    } else {
                        inner_radius
                    };
                    at(start + step * i as f64, radius)
                })
                .collect()
        }
    };
    vec![Contour {
        vertices,
        closed: true,
    }]
}

pub(crate) fn is_straight(v0: &Vertex, v1: &Vertex) -> bool {
    v0.out_tangent == Point::ZERO && v1.in_tangent == Point::ZERO
}

pub(crate) fn lerp_point(a: Point, b: Point, t: f64) -> Point {
    a.add(b.sub(a).scale(t))
}

pub(crate) fn bezier_point(v0: &Vertex, v1: &Vertex, t: f64) -> Point {
    if is_straight(v0, v1) {
        return lerp_point(v0.point, v1.point, t);
    }
    let p0 = v0.point;
    let p1 = v0.point.add(v0.out_tangent);
    let p2 = v1.point.add(v1.in_tangent);
    let p3 = v1.point;
    let mt = 1.0 - t;
    p0.scale(mt * mt * mt)
        .add(p1.scale(3.0 * mt * mt * t))
        .add(p2.scale(3.0 * mt * t * t))
        .add(p3.scale(t * t * t))
}

pub(crate) fn bezier_tangent(v0: &Vertex, v1: &Vertex, t: f64) -> Point {
    if is_straight(v0, v1) {
        return v1.point.sub(v0.point);
    }
    let p0 = v0.point;
    let p1 = v0.point.add(v0.out_tangent);
    let p2 = v1.point.add(v1.in_tangent);
    let p3 = v1.point;
    let mt = 1.0 - t;
    p1.sub(p0)
        .scale(3.0 * mt * mt)
        .add(p2.sub(p1).scale(6.0 * mt * t))
        .add(p3.sub(p2).scale(3.0 * t * t))
}

pub(crate) fn centroid_of(vertices: &[Vertex]) -> Point {
    let sum = vertices.iter().fold(Point::ZERO, |acc, v| acc.add(v.point));
    sum.scale(1.0 / vertices.len() as f64)
}

pub(crate) fn contour_polyline_samples(c: &Contour) -> Vec<Point> {
    let n = c.vertices.len();
    if n <= 1 {
        return c.vertices.iter().map(|v| v.point).collect();
    }
    let edge_count = if c.closed { n } else { n - 1 };
    let mut pts = Vec::new();
    for e in 0..edge_count {
        let v0 = &c.vertices[e];
        let v1 = &c.vertices[(e + 1) % n];
        if pts.is_empty() {
            pts.push(v0.point);
        }
        let is_closing = c.closed && e == edge_count - 1;
        if is_straight(v0, v1) {
            if !is_closing {
                pts.push(v1.point);
            }
        } else {
            let last = if is_closing {
                ARC_SAMPLES - 1
            } else {
                ARC_SAMPLES
            };
            for i in 1..=last {
                let t = i as f64 / ARC_SAMPLES as f64;
                pts.push(bezier_point(v0, v1, t));
            }
        }
    }
    pts
}

pub(crate) const ARC_SAMPLES: usize = 24;

pub(crate) fn segment_sample_lengths(v0: &Vertex, v1: &Vertex) -> ([f64; ARC_SAMPLES + 1], f64) {
    let mut cum = [0.0; ARC_SAMPLES + 1];
    let mut prev = v0.point;
    for i in 1..=ARC_SAMPLES {
        let t = i as f64 / ARC_SAMPLES as f64;
        let cur = bezier_point(v0, v1, t);
        cum[i] = cum[i - 1] + cur.sub(prev).length();
        prev = cur;
    }
    let total = cum[ARC_SAMPLES];
    (cum, total)
}

pub(crate) fn t_at_length(cum: &[f64; ARC_SAMPLES + 1], total_len: f64, target: f64) -> f64 {
    if total_len <= f64::EPSILON {
        return 0.0;
    }
    let target = target.clamp(0.0, total_len);
    for i in 0..ARC_SAMPLES {
        if target <= cum[i + 1] {
            let seg_len = cum[i + 1] - cum[i];
            let local = if seg_len > f64::EPSILON {
                (target - cum[i]) / seg_len
            } else {
                0.0
            };
            let t0 = i as f64 / ARC_SAMPLES as f64;
            let t1 = (i + 1) as f64 / ARC_SAMPLES as f64;
            return t0 + (t1 - t0) * local;
        }
    }
    1.0
}

pub(crate) fn normalize_angle(a: f64) -> f64 {
    let two_pi = std::f64::consts::TAU;
    let mut x = a % two_pi;
    if x <= -std::f64::consts::PI {
        x += two_pi;
    } else if x > std::f64::consts::PI {
        x -= two_pi;
    }
    x
}

pub(crate) fn arc_vertices(center: Point, radius: f64, a0: f64, a1: f64) -> Vec<Vertex> {
    if radius <= 0.0 || (a1 - a0).abs() < 1e-12 {
        let p = center.add(Point {
            x: radius * a0.cos(),
            y: radius * a0.sin(),
        });
        return vec![Vertex::corner(p)];
    }
    let sweep = a1 - a0;
    let max_seg = std::f64::consts::FRAC_PI_2;
    let segments = ((sweep.abs() / max_seg) - 1e-9).ceil().max(1.0) as usize;
    let seg_sweep = sweep / segments as f64;
    let k = 4.0 / 3.0 * (seg_sweep / 4.0).tan() * radius;
    (0..=segments)
        .map(|i| {
            let ang = a0 + seg_sweep * i as f64;
            let p = center.add(Point {
                x: radius * ang.cos(),
                y: radius * ang.sin(),
            });
            let tangent = Point {
                x: -ang.sin(),
                y: ang.cos(),
            };
            let out_t = if i < segments {
                tangent.scale(k)
            } else {
                Point::ZERO
            };
            let in_t = if i > 0 {
                tangent.scale(-k)
            } else {
                Point::ZERO
            };
            Vertex {
                point: p,
                in_tangent: in_t,
                out_tangent: out_t,
            }
        })
        .collect()
}
