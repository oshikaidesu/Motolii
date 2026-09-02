
use crate::doc::vector::geom::{
    arc_vertices, bezier_point, bezier_tangent, centroid_of, contour_polyline_samples, is_straight,
    lerp_point, normalize_angle, segment_sample_lengths, t_at_length, Contour, Path, Point, Vertex,
};
use crate::doc::vector::{Composite, LineJoin, PointType, RepeaterTransform, TrimMultiple, VectorError};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Instance {
    pub(crate) path: Path,
    pub(crate) opacity: f64,
}

pub(crate) fn round_corners_contour(c: &Contour, radius: f64) -> Contour {
    let n = c.vertices.len();
    if radius <= 0.0 || n <= 2 {
        return c.clone();
    }
    let mut out: Vec<Vertex> = Vec::new();
    for i in 0..n {
        if !c.closed && (i == 0 || i == n - 1) {
            out.push(c.vertices[i]);
            continue;
        }
        let prev = c.vertices[(i + n - 1) % n].point;
        let cur = c.vertices[i].point;
        let next = c.vertices[(i + 1) % n].point;
        let to_prev = prev.sub(cur);
        let to_next = next.sub(cur);
        let len_prev = to_prev.length();
        let len_next = to_next.length();
        if len_prev < f64::EPSILON || len_next < f64::EPSILON {
            out.push(c.vertices[i]);
            continue;
        }
        let u1 = to_prev.scale(1.0 / len_prev);
        let u2 = to_next.scale(1.0 / len_next);
        let cos_theta = u1.dot(u2).clamp(-1.0, 1.0);
        let theta = cos_theta.acos();
        if theta < 1e-6 || (std::f64::consts::PI - theta).abs() < 1e-6 {
            out.push(c.vertices[i]);
            continue;
        }
        let tan_half = (theta / 2.0).tan();
        if tan_half.abs() < 1e-9 {
            out.push(c.vertices[i]);
            continue;
        }
        let mut d = radius / tan_half;
        d = d.min(len_prev).min(len_next);
        if d <= 1e-9 {
            out.push(c.vertices[i]);
            continue;
        }
        let p1 = cur.add(u1.scale(d));
        let p2 = cur.add(u2.scale(d));
        let actual_radius = d * tan_half;
        let half = theta / 2.0;
        let bisector = u1.add(u2).normalized();
        let center_dist = if half.sin().abs() < 1e-9 {
            0.0
        } else {
            actual_radius / half.sin()
        };
        let center = cur.add(bisector.scale(center_dist));
        let a1 = (p1.y - center.y).atan2(p1.x - center.x);
        let a2 = (p2.y - center.y).atan2(p2.x - center.x);
        let diff = normalize_angle(a2 - a1);
        out.extend(arc_vertices(center, actual_radius, a1, a1 + diff));
    }
    Contour {
        vertices: out,
        closed: c.closed,
    }
}

pub(crate) fn round_corners(path: &Path, radius: f64) -> Path {
    path.iter()
        .map(|c| round_corners_contour(c, radius))
        .collect()
}

fn pucker_bloat_contour(c: &Contour, amount: f64) -> Contour {
    if c.vertices.len() <= 1 {
        return c.clone();
    }
    let centroid = centroid_of(&c.vertices);
    let vertices = c
        .vertices
        .iter()
        .map(|v| {
            let d = v.point.sub(centroid);
            let new_point = centroid.add(d.scale(1.0 - amount));
            let handle_shift = d.scale(2.0 * amount);
            let tangent_scale = 1.0 + amount;
            Vertex {
                point: new_point,
                in_tangent: v.in_tangent.scale(tangent_scale).add(handle_shift),
                out_tangent: v.out_tangent.scale(tangent_scale).add(handle_shift),
            }
        })
        .collect();
    Contour {
        vertices,
        closed: c.closed,
    }
}

pub(crate) fn pucker_bloat(path: &Path, amount: f64) -> Path {
    path.iter()
        .map(|c| pucker_bloat_contour(c, amount))
        .collect()
}

fn zigzag_contour(c: &Contour, amplitude: f64, frequency: f64, point_type: PointType) -> Contour {
    if c.vertices.len() <= 1 {
        return c.clone();
    }
    let ridge_count = frequency.max(0.0).round() as usize;
    if ridge_count == 0 || amplitude == 0.0 {
        return c.clone();
    }
    let n = c.vertices.len();
    let edge_count = if c.closed { n } else { n - 1 };
    let mut points: Vec<Point> = Vec::new();
    let steps = ridge_count * 2;
    for e in 0..edge_count {
        let v0 = &c.vertices[e];
        let v1 = &c.vertices[(e + 1) % n];
        let (cum, seg_len) = segment_sample_lengths(v0, v1);
        if seg_len < f64::EPSILON {
            continue;
        }
        points.push(bezier_point(v0, v1, 0.0));
        for k in 1..steps {
            let target = (k as f64 / steps as f64) * seg_len;
            let t = t_at_length(&cum, seg_len, target);
            let base = bezier_point(v0, v1, t);
            let tangent = bezier_tangent(v0, v1, t);
            let tlen = tangent.length();
            let unit = if tlen < f64::EPSILON {
                let chord = v1.point.sub(v0.point);
                if chord.length() < f64::EPSILON {
                    continue;
                }
                chord.normalized()
            } else {
                tangent.scale(1.0 / tlen)
            };
            let normal = Point {
                x: -unit.y,
                y: unit.x,
            };
            let sign = if k % 2 == 1 { 1.0 } else { -1.0 };
            points.push(base.add(normal.scale(sign * amplitude)));
        }
    }
    if !c.closed {
        points.push(c.vertices[n - 1].point);
    }
    Contour {
        vertices: build_point_type_vertices(&points, point_type, c.closed),
        closed: c.closed,
    }
}

fn build_point_type_vertices(points: &[Point], point_type: PointType, closed: bool) -> Vec<Vertex> {
    let n = points.len();
    (0..n)
        .map(|i| {
            let p = points[i];
            match point_type {
                PointType::Corner => Vertex::corner(p),
                PointType::Smooth => {
                    let prev = if i == 0 {
                        if closed {
                            points[n - 1]
                        } else {
                            p
                        }
                    } else {
                        points[i - 1]
                    };
                    let next = if i == n - 1 {
                        if closed {
                            points[0]
                        } else {
                            p
                        }
                    } else {
                        points[i + 1]
                    };
                    let handle = next.sub(prev).scale(1.0 / 6.0);
                    Vertex {
                        point: p,
                        in_tangent: handle.scale(-1.0),
                        out_tangent: handle,
                    }
                }
            }
        })
        .collect()
}

pub(crate) fn zigzag(path: &Path, amplitude: f64, frequency: f64, point_type: PointType) -> Path {
    path.iter()
        .map(|c| zigzag_contour(c, amplitude, frequency, point_type))
        .collect()
}

fn offset_contour(
    c: &Contour,
    amount: f64,
    line_join: LineJoin,
    miter_limit: f64,
) -> Result<Contour, VectorError> {
    if c.vertices.len() <= 1 {
        return Ok(c.clone());
    }
    if !c.closed {
        return Err(VectorError::OpenPathOffset);
    }
    let pts = contour_polyline_samples(c);
    let n = pts.len();
    let orientation_sign = if polygon_signed_area(&pts) >= 0.0 {
        1.0
    } else {
        -1.0
    };

    let mut offset_edges: Vec<(Point, Point)> = Vec::with_capacity(n);
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let dir = b.sub(a);
        let len = dir.length();
        if len < f64::EPSILON {
            offset_edges.push((a, b));
            continue;
        }
        let unit = dir.scale(1.0 / len);
        let outward = Point {
            x: unit.y,
            y: -unit.x,
        }
        .scale(orientation_sign);
        let shift = outward.scale(amount);
        offset_edges.push((a.add(shift), b.add(shift)));
    }

    let mut out_points: Vec<Point> = Vec::new();
    for i in 0..n {
        let (prev_a, prev_b) = offset_edges[(i + n - 1) % n];
        let (cur_a, cur_b) = offset_edges[i];
        join_corner(
            &mut out_points,
            prev_a,
            prev_b,
            cur_a,
            cur_b,
            pts[i],
            amount,
            line_join,
            miter_limit,
        );
    }
    Ok(Contour {
        vertices: out_points.into_iter().map(Vertex::corner).collect(),
        closed: true,
    })
}

fn polygon_signed_area(pts: &[Point]) -> f64 {
    let n = pts.len();
    let mut sum = 0.0;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        sum += a.x * b.y - b.x * a.y;
    }
    sum * 0.5
}

fn points_close(a: Point, b: Point) -> bool {
    a.sub(b).length() < 1e-9
}

fn line_intersection(p1: Point, p2: Point, p3: Point, p4: Point) -> Option<Point> {
    let d1 = p2.sub(p1);
    let d2 = p4.sub(p3);
    let denom = d1.x * d2.y - d1.y * d2.x;
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = ((p3.x - p1.x) * d2.y - (p3.y - p1.y) * d2.x) / denom;
    Some(p1.add(d1.scale(t)))
}

#[allow(clippy::too_many_arguments)]
fn join_corner(
    out: &mut Vec<Point>,
    prev_a: Point,
    prev_b: Point,
    cur_a: Point,
    cur_b: Point,
    vertex: Point,
    amount: f64,
    line_join: LineJoin,
    miter_limit: f64,
) {
    if points_close(prev_b, cur_a) {
        out.push(prev_b);
        return;
    }
    if line_join == LineJoin::Miter {
        if let Some(p) = line_intersection(prev_a, prev_b, cur_a, cur_b) {
            let miter_len = p.sub(vertex).length();
            let limit_len = miter_limit * amount.abs().max(f64::EPSILON);
            if miter_len <= limit_len {
                out.push(p);
                return;
            }
        }
    }
    out.push(prev_b);
    if line_join == LineJoin::Round {
        let r = amount.abs();
        if r > f64::EPSILON {
            let a0 = (prev_b.y - vertex.y).atan2(prev_b.x - vertex.x);
            let a1 = (cur_a.y - vertex.y).atan2(cur_a.x - vertex.x);
            let diff = normalize_angle(a1 - a0);
            const MAX_STEP: f64 = std::f64::consts::FRAC_PI_8;
            let steps = (diff.abs() / MAX_STEP).ceil().max(2.0) as usize;
            for i in 1..steps {
                let ang = a0 + diff * (i as f64 / steps as f64);
                out.push(vertex.add(Point {
                    x: r * ang.cos(),
                    y: r * ang.sin(),
                }));
            }
        }
    }
    out.push(cur_a);
}

pub(crate) fn offset_path(
    path: &Path,
    amount: f64,
    line_join: LineJoin,
    miter_limit: f64,
) -> Result<Path, VectorError> {
    path.iter()
        .map(|c| offset_contour(c, amount, line_join, miter_limit))
        .collect()
}

fn twist_contour(c: &Contour, degrees: f64, center: Point) -> Contour {
    if c.vertices.len() <= 1 {
        return c.clone();
    }
    let max_r = c
        .vertices
        .iter()
        .map(|v| v.point.sub(center).length())
        .fold(0.0_f64, f64::max);
    if max_r <= f64::EPSILON {
        return c.clone();
    }
    let angle = degrees.to_radians();
    let vertices = c
        .vertices
        .iter()
        .map(|v| {
            let d = v.point.sub(center);
            let r = d.length();
            let local_angle = angle * (1.0 - r / max_r);
            Vertex {
                point: center.add(d.rotate(local_angle)),
                in_tangent: v.in_tangent.rotate(local_angle),
                out_tangent: v.out_tangent.rotate(local_angle),
            }
        })
        .collect();
    Contour {
        vertices,
        closed: c.closed,
    }
}

pub(crate) fn twist(path: &Path, degrees: f64, center: Point) -> Path {
    path.iter()
        .map(|c| twist_contour(c, degrees, center))
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

impl Affine {
    pub(crate) const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub(crate) fn apply(&self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.tx,
            y: self.b * p.x + self.d * p.y + self.ty,
        }
    }

    pub(crate) fn apply_vector(&self, v: Point) -> Point {
        Point {
            x: self.a * v.x + self.c * v.y,
            y: self.b * v.x + self.d * v.y,
        }
    }

    pub(crate) fn mul(&self, rhs: &Affine) -> Affine {
        Affine {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: self.a * rhs.tx + self.c * rhs.ty + self.tx,
            ty: self.b * rhs.tx + self.d * rhs.ty + self.ty,
        }
    }

    fn lerp(&self, other: &Affine, t: f64) -> Affine {
        Affine {
            a: self.a + (other.a - self.a) * t,
            b: self.b + (other.b - self.b) * t,
            c: self.c + (other.c - self.c) * t,
            d: self.d + (other.d - self.d) * t,
            tx: self.tx + (other.tx - self.tx) * t,
            ty: self.ty + (other.ty - self.ty) * t,
        }
    }

    fn det2(&self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    fn invert(&self) -> Option<Affine> {
        let det = self.det2();
        if det.abs() < f64::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Affine {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            tx: (self.c * self.ty - self.d * self.tx) * inv_det,
            ty: (self.b * self.tx - self.a * self.ty) * inv_det,
        })
    }
}

pub(crate) fn build_affine(t: &RepeaterTransform) -> Affine {
    let (s, c) = t.rotation.to_radians().sin_cos();
    let rs_a = c * t.scale.x;
    let rs_b = s * t.scale.x;
    let rs_c = -s * t.scale.y;
    let rs_d = c * t.scale.y;
    let tx = rs_a * (-t.anchor.x) + rs_c * (-t.anchor.y) + t.position.x;
    let ty = rs_b * (-t.anchor.x) + rs_d * (-t.anchor.y) + t.position.y;
    Affine {
        a: rs_a,
        b: rs_b,
        c: rs_c,
        d: rs_d,
        tx,
        ty,
    }
}

fn affine_pow_int(m: &Affine, n: i64) -> Affine {
    if n == 0 {
        return Affine::IDENTITY;
    }
    if n > 0 {
        let mut result = Affine::IDENTITY;
        for _ in 0..n {
            result = m.mul(&result);
        }
        return result;
    }
    let Some(inv) = m.invert() else {
        return Affine::IDENTITY;
    };
    let mut result = Affine::IDENTITY;
    for _ in 0..(-n) {
        result = inv.mul(&result);
    }
    result
}

fn affine_pow_real(m: &Affine, k: f64) -> Affine {
    if k.abs() <= f64::EPSILON {
        return Affine::IDENTITY;
    }
    let lo = k.floor();
    let frac = k - lo;
    let m_lo = affine_pow_int(m, lo as i64);
    if frac.abs() <= f64::EPSILON {
        return m_lo;
    }
    let m_hi = m.mul(&m_lo);
    m_lo.lerp(&m_hi, frac)
}

pub(crate) fn apply_matrix_to_contour(c: &Contour, m: &Affine) -> Contour {
    let vertices = c
        .vertices
        .iter()
        .map(|v| Vertex {
            point: m.apply(v.point),
            in_tangent: m.apply_vector(v.in_tangent),
            out_tangent: m.apply_vector(v.out_tangent),
        })
        .collect();
    Contour {
        vertices,
        closed: c.closed,
    }
}

pub(crate) fn repeater(
    input: &[Instance],
    copies: f64,
    offset: f64,
    transform: &RepeaterTransform,
    composite: Composite,
    start_opacity: f64,
    end_opacity: f64,
) -> Vec<Instance> {
    let n = copies.floor() as i64;
    if n <= 0 {
        return Vec::new();
    }
    let m = build_affine(transform);
    let order: Vec<i64> = match composite {
        Composite::Above => (0..n).collect(),
        Composite::Below => (0..n).rev().collect(),
    };
    let mut out = Vec::with_capacity(input.len() * n as usize);
    for i in order {
        let weight = if n == 1 {
            start_opacity
        } else {
            let f = i as f64 / (n - 1) as f64;
            start_opacity + (end_opacity - start_opacity) * f
        };
        let mk = affine_pow_real(&m, i as f64 + offset);
        for inst in input {
            out.push(Instance {
                path: inst
                    .path
                    .iter()
                    .map(|c| apply_matrix_to_contour(c, &mk))
                    .collect(),
                opacity: inst.opacity * weight,
            });
        }
    }
    out
}

pub(crate) fn split_bezier(v0: &Vertex, v1: &Vertex, t: f64) -> (Vertex, Vertex, Vertex) {
    if is_straight(v0, v1) {
        let m = lerp_point(v0.point, v1.point, t);
        return (
            Vertex {
                point: v0.point,
                in_tangent: v0.in_tangent,
                out_tangent: Point::ZERO,
            },
            Vertex {
                point: m,
                in_tangent: Point::ZERO,
                out_tangent: Point::ZERO,
            },
            Vertex {
                point: v1.point,
                in_tangent: Point::ZERO,
                out_tangent: v1.out_tangent,
            },
        );
    }
    let p0 = v0.point;
    let p1 = v0.point.add(v0.out_tangent);
    let p2 = v1.point.add(v1.in_tangent);
    let p3 = v1.point;
    let a = lerp_point(p0, p1, t);
    let b = lerp_point(p1, p2, t);
    let cc = lerp_point(p2, p3, t);
    let d = lerp_point(a, b, t);
    let e = lerp_point(b, cc, t);
    let m = lerp_point(d, e, t);
    (
        Vertex {
            point: p0,
            in_tangent: v0.in_tangent,
            out_tangent: a.sub(p0),
        },
        Vertex {
            point: m,
            in_tangent: d.sub(m),
            out_tangent: e.sub(m),
        },
        Vertex {
            point: p3,
            in_tangent: cc.sub(p3),
            out_tangent: v1.out_tangent,
        },
    )
}

fn sub_bezier(v0: &Vertex, v1: &Vertex, t0: f64, t1: f64) -> (Vertex, Vertex) {
    if t0 <= 0.0 && t1 >= 1.0 {
        return (*v0, *v1);
    }
    let (_, tail_start, tail_end) = split_bezier(v0, v1, t0.max(0.0));
    if t1 >= 1.0 {
        return (tail_start, tail_end);
    }
    let denom = (1.0 - t0).max(f64::EPSILON);
    let local_t1 = ((t1 - t0) / denom).clamp(0.0, 1.0);
    let (head_start, head_end, _) = split_bezier(&tail_start, &tail_end, local_t1);
    (head_start, head_end)
}

struct FlatSegment {
    contour_idx: usize,
    v0: Vertex,
    v1: Vertex,
    len: f64,
}

fn contour_segments(c: &Contour) -> Vec<(Vertex, Vertex)> {
    let n = c.vertices.len();
    let m = if c.closed { n } else { n.saturating_sub(1) };
    (0..m)
        .map(|i| (c.vertices[i], c.vertices[(i + 1) % n]))
        .collect()
}

fn flatten_segments(contours: &[Contour]) -> Vec<FlatSegment> {
    let mut out = Vec::new();
    for (ci, c) in contours.iter().enumerate() {
        if c.vertices.len() <= 1 {
            continue;
        }
        for (v0, v1) in contour_segments(c) {
            let (_, len) = segment_sample_lengths(&v0, &v1);
            out.push(FlatSegment {
                contour_idx: ci,
                v0,
                v1,
                len,
            });
        }
    }
    out
}

fn wrap01(x: f64) -> f64 {
    let mut r = x % 1.0;
    if r < 0.0 {
        r += 1.0;
    }
    r
}

fn resolve_windows(start: f64, end: f64, offset: f64) -> Vec<(f64, f64)> {
    let s = start + offset;
    let mut e = end + offset;
    if e < s {
        e += 1.0;
    }
    let coverage = (e - s).clamp(0.0, 1.0);
    if coverage <= f64::EPSILON {
        return Vec::new();
    }
    let s_wrapped = wrap01(s);
    let e_pos = s_wrapped + coverage;
    if e_pos <= 1.0 + 1e-9 {
        vec![(s_wrapped, e_pos.min(1.0))]
    } else {
        vec![(s_wrapped, 1.0), (0.0, e_pos - 1.0)]
    }
}

fn extract_window(segments: &[FlatSegment], from: f64, to: f64) -> Vec<Contour> {
    if to - from <= f64::EPSILON {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current: Vec<Vertex> = Vec::new();
    let mut current_contour: Option<usize> = None;
    let mut acc = 0.0;
    for seg in segments {
        let seg_start = acc;
        let seg_end = acc + seg.len;
        acc = seg_end;

        let overlaps = seg_end > from + 1e-12 && seg_start < to - 1e-12;
        let boundary_break = current_contour.is_some_and(|idx| idx != seg.contour_idx);
        if boundary_break && !current.is_empty() {
            result.push(Contour {
                vertices: std::mem::take(&mut current),
                closed: false,
            });
        }
        if !overlaps {
            if !current.is_empty() {
                result.push(Contour {
                    vertices: std::mem::take(&mut current),
                    closed: false,
                });
            }
            current_contour = None;
            continue;
        }
        current_contour = Some(seg.contour_idx);

        let (cum, seg_total) = segment_sample_lengths(&seg.v0, &seg.v1);
        let local_from = (from - seg_start).max(0.0);
        let local_to = (to - seg_start).min(seg.len);
        let t0 = if local_from <= f64::EPSILON {
            0.0
        } else {
            t_at_length(&cum, seg_total, local_from)
        };
        let t1 = if local_to >= seg.len - f64::EPSILON {
            1.0
        } else {
            t_at_length(&cum, seg_total, local_to)
        };
        let (sv, ev) = sub_bezier(&seg.v0, &seg.v1, t0, t1);
        if current.is_empty() {
            current.push(sv);
        }
        current.push(ev);
    }
    if !current.is_empty() {
        result.push(Contour {
            vertices: current,
            closed: false,
        });
    }
    result
}

pub(crate) fn trim(path: &Path, start: f64, end: f64, offset: f64, multiple: TrimMultiple) -> Path {
    let windows = resolve_windows(start, end, offset);
    if windows.is_empty() {
        return Path::new();
    }
    match multiple {
        TrimMultiple::Simultaneously => {
            let mut out = Path::new();
            for c in path {
                if c.vertices.len() <= 1 {
                    out.push(c.clone());
                    continue;
                }
                let segs = flatten_segments(std::slice::from_ref(c));
                let total: f64 = segs.iter().map(|s| s.len).sum();
                if total <= f64::EPSILON {
                    out.push(c.clone());
                    continue;
                }
                for (fs, ft) in &windows {
                    out.extend(extract_window(&segs, fs * total, ft * total));
                }
            }
            out
        }
        TrimMultiple::Individually => {
            let segs = flatten_segments(path);
            let total: f64 = segs.iter().map(|s| s.len).sum();
            if total <= f64::EPSILON {
                return path.clone();
            }
            let mut out = Path::new();
            for (fs, ft) in &windows {
                out.extend(extract_window(&segs, fs * total, ft * total));
            }
            out
        }
    }
}
