//! 輪郭を作り替える演算(Lottie の外: AE の Wiggle Paths、Cavalry の Path Relax・Add Divisions・
//! Reverse・Extend・Chop・Resample・Bend)。どれも `(path, params) → path` の純関数で、時間は phase で受ける。
use crate::doc::vector::geom::{
    bezier_point, bezier_tangent, contour_polyline_samples, Contour, Path, Point, Vertex,
};
use crate::doc::vector::PointType;

use super::trim::{extract_window, flatten_segments};
use super::{build_point_type_vertices, split_bezier};

fn edges(c: &Contour) -> usize {
    let n = c.vertices.len();
    if n <= 1 { 0 } else if c.closed { n } else { n - 1 }
}

/// 位相で滑らかに移る `[-1, 1]` の値雑音。同じ (seed, index, phase) は同じ値。
fn value_noise(seed: u64, index: u32, phase: f64) -> f64 {
    let cell = phase.floor();
    let f = phase - cell;
    let f = f * f * (3.0 - 2.0 * f);
    let a = crate::doc::store::placement::noise(seed, index, cell as i64 as u64);
    let b = crate::doc::store::placement::noise(seed, index, (cell as i64 + 1) as u64);
    a + (b - a) * f
}

/// AE の Wiggle Paths。辺ごとに `detail` 分割の点を置き、法線方向へ `size` までの雑音で揺らす。
/// Illustrator の Roughen は Points=Corner のこれ。
pub(crate) fn wiggle(path: &Path, size: f64, detail: f64, point_type: PointType, phase: f64, seed: u64) -> Path {
    let steps = detail.round().max(1.0) as usize;
    let mut index = 0u32;
    path.iter().map(|c| {
        let edge_count = edges(c);
        if edge_count == 0 || size == 0.0 { return c.clone(); }
        let n = c.vertices.len();
        let mut points = Vec::new();
        let mut push = |base: Point, tangent: Point| {
            let unit = tangent.normalized();
            let normal = Point { x: -unit.y, y: unit.x };
            points.push(base.add(normal.scale(size * value_noise(seed, index, phase))));
            index += 1;
        };
        for e in 0..edge_count {
            let (v0, v1) = (&c.vertices[e], &c.vertices[(e + 1) % n]);
            for k in 0..steps {
                let t = k as f64 / steps as f64;
                push(bezier_point(v0, v1, t), bezier_tangent(v0, v1, t));
            }
        }
        if !c.closed {
            let (v0, v1) = (&c.vertices[n - 2], &c.vertices[n - 1]);
            push(v1.point, bezier_tangent(v0, v1, 1.0));
        }
        Contour { vertices: build_point_type_vertices(&points, point_type, c.closed), closed: c.closed }
    }).collect()
}

/// Cavalry の Path Relax。頂点を両隣の中点へ `strength` だけ寄せるのを `iterations` 回。開いた端は動かない。
pub(crate) fn smooth(path: &Path, strength: f64, iterations: f64) -> Path {
    let strength = strength.clamp(0.0, 1.0);
    let rounds = iterations.round().max(0.0) as usize;
    path.iter().map(|c| {
        let n = c.vertices.len();
        if n < 3 || strength == 0.0 { return c.clone(); }
        let mut out = c.clone();
        for _ in 0..rounds {
            let prev: Vec<Point> = out.vertices.iter().map(|v| v.point).collect();
            for i in 0..n {
                if !c.closed && (i == 0 || i == n - 1) { continue; }
                let mid = prev[(i + n - 1) % n].add(prev[(i + 1) % n]).scale(0.5);
                out.vertices[i].point = prev[i].add(mid.sub(prev[i]).scale(strength));
            }
        }
        out
    }).collect()
}

/// Cavalry の Add Divisions。辺ごとに `divisions` 個の頂点を等分の t で差し込む。形は変わらない。
pub(crate) fn subdivide(path: &Path, divisions: f64) -> Path {
    let divisions = divisions.round().max(0.0) as usize;
    path.iter().map(|c| {
        let edge_count = edges(c);
        if edge_count == 0 || divisions == 0 { return c.clone(); }
        let n = c.vertices.len();
        let mut out: Vec<Vertex> = Vec::new();
        let mut pending_in = c.vertices[0].in_tangent;
        for e in 0..edge_count {
            let mut a = c.vertices[e];
            a.in_tangent = pending_in;
            let mut b = c.vertices[(e + 1) % n];
            for k in 0..divisions {
                let (a2, mid, b2) = split_bezier(&a, &b, 1.0 / (divisions - k + 1) as f64);
                out.push(Vertex { point: a2.point, in_tangent: a.in_tangent, out_tangent: a2.out_tangent });
                a = mid;
                b = b2;
            }
            out.push(a);
            pending_in = b.in_tangent;
        }
        if c.closed { out[0].in_tangent = pending_in; } else { out.push(Vertex { point: c.vertices[n - 1].point, in_tangent: pending_in, out_tangent: Point::ZERO }); }
        Contour { vertices: out, closed: c.closed }
    }).collect()
}

/// Cavalry の Reverse Path。向きを逆に。Trim や Chop の始点が入れ替わる。
pub(crate) fn reverse(path: &Path) -> Path {
    path.iter().map(|c| Contour {
        vertices: c.vertices.iter().rev().map(|v| Vertex { point: v.point, in_tangent: v.out_tangent, out_tangent: v.in_tangent }).collect(),
        closed: c.closed,
    }).collect()
}

/// Cavalry の Extend Open Paths。開いた輪郭の両端を、端の接線の向きへ真っ直ぐ伸ばす。
pub(crate) fn extend(path: &Path, start: f64, end: f64) -> Path {
    path.iter().map(|c| {
        let n = c.vertices.len();
        if c.closed || n < 2 { return c.clone(); }
        let mut out = c.clone();
        if end != 0.0 {
            let dir = bezier_tangent(&c.vertices[n - 2], &c.vertices[n - 1], 1.0).normalized();
            out.vertices.push(Vertex::corner(c.vertices[n - 1].point.add(dir.scale(end))));
        }
        if start != 0.0 {
            let dir = bezier_tangent(&c.vertices[0], &c.vertices[1], 0.0).normalized();
            out.vertices.insert(0, Vertex::corner(c.vertices[0].point.sub(dir.scale(start))));
        }
        out
    }).collect()
}

/// Cavalry の Chop Path。輪郭を `length` の切れ端に、`gap` を空けて刻む。
pub(crate) fn chop(path: &Path, length: f64, gap: f64) -> Path {
    if length <= 0.0 { return path.clone(); }
    let mut out = Path::new();
    for c in path {
        if c.vertices.len() <= 1 { out.push(c.clone()); continue; }
        let segs = flatten_segments(std::slice::from_ref(c));
        let total: f64 = segs.iter().map(|s| s.len).sum();
        if total <= f64::EPSILON { out.push(c.clone()); continue; }
        let mut from = 0.0;
        while from < total {
            let to = (from + length).min(total);
            out.extend(extract_window(&segs, from, to));
            from += length + gap.max(0.0);
            if gap <= 0.0 && length <= 0.0 { break; }
        }
    }
    out
}

/// Cavalry の Resample Path。弧長に沿って `spacing` ごとに点を置き直す。
pub(crate) fn resample(path: &Path, spacing: f64, point_type: PointType) -> Path {
    if spacing <= 0.0 { return path.clone(); }
    path.iter().map(|c| {
        let samples = contour_polyline_samples(c);
        if samples.len() < 2 { return c.clone(); }
        let mut points = vec![samples[0]];
        let mut carried = 0.0;
        let last = if c.closed { samples.len() } else { samples.len() - 1 };
        for i in 0..last {
            let (a, b) = (samples[i], samples[(i + 1) % samples.len()]);
            let len = b.sub(a).length();
            let mut d = spacing - carried;
            while d <= len {
                points.push(a.add(b.sub(a).scale(d / len)));
                d += spacing;
            }
            carried = len - (d - spacing);
        }
        if c.closed && points.len() > 1 && points.last().unwrap().sub(points[0]).length() < spacing * 0.5 { points.pop(); }
        if !c.closed && points.last() != samples.last() { points.push(*samples.last().unwrap()); }
        Contour { vertices: build_point_type_vertices(&points, point_type, c.closed), closed: c.closed }
    }).collect()
}

/// Cavalry の Bend Deformer。`center` を通る縦軸のまわりで、横幅ぶんを `angle` の弧へ曲げる。
/// 幅は path 自身から取る(Twist と同じ自己正規化)。
pub(crate) fn bend(path: &Path, angle_degrees: f64, center: Point) -> Path {
    let angle = angle_degrees.to_radians();
    let (min_x, max_x) = path.iter().flat_map(|c| c.vertices.iter().map(|v| v.point.x)).fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| (lo.min(x), hi.max(x)));
    let width = max_x - min_x;
    if angle.abs() < 1e-9 || !(width > f64::EPSILON) { return path.clone(); }
    let radius = width / angle;
    let map = |p: Point| {
        let theta = (p.x - center.x) / width * angle;
        let r = radius - (p.y - center.y);
        Point { x: center.x + r * theta.sin(), y: center.y + radius - r * theta.cos() }
    };
    path.iter().map(|c| Contour {
        vertices: c.vertices.iter().map(|v| {
            let p = map(v.point);
            Vertex { point: p, in_tangent: map(v.point.add(v.in_tangent)).sub(p), out_tangent: map(v.point.add(v.out_tangent)).sub(p) }
        }).collect(),
        closed: c.closed,
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::vector::geom::is_straight;

    fn p(x: f64, y: f64) -> Point { Point { x, y } }
    fn square() -> Path { vec![Contour::closed([p(0.0, 0.0), p(100.0, 0.0), p(100.0, 100.0), p(0.0, 100.0)])] }
    fn line() -> Path { vec![Contour::open([p(0.0, 0.0), p(100.0, 0.0)])] }
    fn arc() -> Path {
        vec![Contour { closed: false, vertices: vec![
            Vertex { point: p(0.0, 0.0), in_tangent: Point::ZERO, out_tangent: p(0.0, 60.0) },
            Vertex { point: p(100.0, 0.0), in_tangent: p(0.0, 60.0), out_tangent: Point::ZERO },
        ] }]
    }

    /// 逆向きは 2 回で元に戻り、接線は入と出が入れ替わる。
    #[test]
    fn reverse_twice_is_identity_and_swaps_tangents() {
        let a = arc();
        let r = reverse(&a);
        assert_eq!(r[0].vertices[0].point, p(100.0, 0.0));
        assert_eq!(r[0].vertices[0].out_tangent, p(0.0, 60.0));
        assert_eq!(reverse(&r), a);
    }

    /// 分割は点を増やすだけで曲線の上を離れない。
    #[test]
    fn subdivide_keeps_every_point_on_the_curve() {
        let a = arc();
        let s = subdivide(&a, 3.0);
        assert_eq!(s[0].vertices.len(), 5);
        let (v0, v1) = (&a[0].vertices[0], &a[0].vertices[1]);
        for (k, v) in s[0].vertices.iter().enumerate() {
            let expect = bezier_point(v0, v1, k as f64 / 4.0);
            assert!(v.point.sub(expect).length() < 1e-9, "{k}: {:?} vs {:?}", v.point, expect);
        }
        assert_eq!(subdivide(&square(), 1.0)[0].vertices.len(), 8);
    }

    /// 伸ばすと端が接線の向きへ真っ直ぐ出る。閉じた輪郭は触らない。
    #[test]
    fn extend_pushes_open_ends_along_their_tangents() {
        let e = extend(&line(), 10.0, 20.0);
        assert_eq!(e[0].vertices.first().unwrap().point, p(-10.0, 0.0));
        assert_eq!(e[0].vertices.last().unwrap().point, p(120.0, 0.0));
        assert!(is_straight(&e[0].vertices[0], &e[0].vertices[1]));
        assert_eq!(extend(&square(), 10.0, 10.0), square());
    }

    /// 刻むと長さ+隙間ごとの切れ端になる。
    #[test]
    fn chop_cuts_pieces_of_the_given_length() {
        let c = chop(&line(), 30.0, 10.0);
        assert_eq!(c.len(), 3);
        let len = |k: &Contour| k.vertices.windows(2).map(|w| w[1].point.sub(w[0].point).length()).sum::<f64>();
        assert!((len(&c[0]) - 30.0).abs() < 1e-6 && (len(&c[2]) - 20.0).abs() < 1e-6, "{:?}", c.iter().map(len).collect::<Vec<_>>());
        assert_eq!(c[1].vertices[0].point.x, 40.0);
    }

    /// 置き直した点は等間隔で、端は保つ。
    #[test]
    fn resample_spaces_points_evenly() {
        let r = resample(&line(), 25.0, PointType::Corner);
        let xs: Vec<f64> = r[0].vertices.iter().map(|v| v.point.x).collect();
        assert_eq!(xs, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
        assert_eq!(resample(&square(), 50.0, PointType::Corner)[0].vertices.len(), 8);
    }

    /// 均す(smooth)と角は隣の中点へ寄り、強さ 1 で中点に着く。開いた端は動かない。
    #[test]
    fn smooth_moves_corners_toward_their_neighbours() {
        let s = smooth(&square(), 1.0, 1.0);
        assert_eq!(s[0].vertices[1].point, p(50.0, 50.0));
        let zig = vec![Contour::open([p(0.0, 0.0), p(50.0, 100.0), p(100.0, 0.0)])];
        let s = smooth(&zig, 0.5, 1.0);
        assert_eq!(s[0].vertices[0].point, p(0.0, 0.0));
        assert_eq!(s[0].vertices[1].point, p(50.0, 50.0));
    }

    /// 揺らぎは種で決まり、大きさを超えず、大きさ 0 なら何もしない。
    #[test]
    fn wiggle_is_deterministic_and_bounded() {
        let a = wiggle(&line(), 8.0, 4.0, PointType::Corner, 0.3, 7);
        assert_eq!(a, wiggle(&line(), 8.0, 4.0, PointType::Corner, 0.3, 7));
        assert_ne!(a, wiggle(&line(), 8.0, 4.0, PointType::Corner, 0.3, 8));
        assert_eq!(a[0].vertices.len(), 5);
        assert!(a[0].vertices.iter().all(|v| v.point.y.abs() <= 8.0 + 1e-9));
        assert!(a[0].vertices.iter().any(|v| v.point.y.abs() > 0.0));
        assert_eq!(wiggle(&line(), 0.0, 4.0, PointType::Corner, 0.0, 7), line());
    }

    /// 曲げは角度 0 で何もせず、180° で横幅が半円になり両端が向かい合う。
    #[test]
    fn bend_wraps_the_width_into_an_arc() {
        assert_eq!(bend(&line(), 0.0, Point::ZERO), line());
        let b = bend(&line(), 180.0, p(50.0, 0.0));
        let (a, z) = (b[0].vertices[0].point, b[0].vertices[1].point);
        let radius = 100.0 / std::f64::consts::PI;
        assert!((a.x - (50.0 - radius)).abs() < 1e-6 && (z.x - (50.0 + radius)).abs() < 1e-6, "{a:?} {z:?}");
        assert!((a.y - radius).abs() < 1e-6 && (z.y - radius).abs() < 1e-6);
    }
}
