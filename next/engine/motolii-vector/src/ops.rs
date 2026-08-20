//! 演算子スタックの中身。**旧 `pathgeom.rs` からの移植**(裁定10)。
//!
//! 移植したのは `shape-1` が名指しした3つだけ: `trim` / `repeater` / `round_corners`。
//! 同じファイルに居た `pucker_bloat` / `zigzag` / `offset` / `twist` / `wiggle` は
//! **持ってきていない** — 使わない物を抱えると `check.sh` の `owns:` が自前実装の量を
//! 偽る(軸4)。`shape-2` を作る日に、その束のぶんだけ旧 workspace から取る。
//!
//! 移植元との差は2箇所だけで、どちらも意図がある:
//!
//! 1. `Path` が構造体から `Vec<Contour>` の別名になった(輪郭の列以外を持たないため)
//! 2. **repeater が `Path` ではなく [`Instance`] の列を返す**。移植元は
//!    「opacity は幾何に影響しないので合成側の責務」と書いて `so`/`eo` を捨てていたが、
//!    **この crate が合成側**なので、捨て先がここになった。幾何(`affine_pow_real` 他)は
//!    1行も変えていない。

use crate::geom::{
    arc_vertices, is_straight, lerp_point, normalize_angle, segment_sample_lengths, t_at_length,
    Contour, Path, Point, Vertex,
};
use crate::{Composite, RepeaterTransform, TrimMultiple};

/// パス1つと、それに掛かる不透明度の重み。
///
/// repeater が `so`/`eo` を持つために要る — コピーごとに alpha が違うので、
/// 「1枚のパス」では表せない。重みは合成時に fill/stroke の alpha へ掛かる。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Instance {
    pub(crate) path: Path,
    pub(crate) opacity: f64,
}

// ---------------------------------------------------------------------------
// rounded-corners(移植: round_corners_contour)
// 各頂点(開路は両端を除く)を半径 radius の fillet へ置換する。
// タンジェントハンドルは弧を cubic bezier 近似(90°ごと分割)して保持する。
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// repeater(移植: Affine / build_affine / affine_pow_* / apply_matrix_to_contour)
// M = T(position)·R(rotation)·S(scale)·T(-anchor) を k=index+offset 回合成適用する。
// 整数 k は行列の反復合成、小数部は k,k+1 の行列を線形補間する
// (2Dアフィンの真の実数冪 = Lie群指数写像は要求されていないため、簡略近似と明示する)。
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

impl Affine {
    const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    fn apply(&self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.tx,
            y: self.b * p.x + self.d * p.y + self.ty,
        }
    }

    fn apply_vector(&self, v: Point) -> Point {
        Point {
            x: self.a * v.x + self.c * v.y,
            y: self.b * v.x + self.d * v.y,
        }
    }

    /// self ∘ rhs (rhs を先に適用)。
    fn mul(&self, rhs: &Affine) -> Affine {
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

    /// 逆アフィン。特異(スケール≈0)なら None — 負整数冪の退避先。
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

/// 移植元との唯一の差: `rotation` を**度**で受ける(裁定58「rotation は度のまま」)。
/// 行列を組む前にここで1度だけラジアンへ落とす。
fn build_affine(t: &RepeaterTransform) -> Affine {
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
    // 負整数冪は逆行列で |n| 回合成。Lottie Repeater の負 offset がここに来る。
    // スケール≈0で特異なら恒等 — 逆行列が発散するため幾何を打ち切る。
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

fn apply_matrix_to_contour(c: &Contour, m: &Affine) -> Contour {
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

/// コピーを作る。**移植元 `repeater_path` の対応形**で、違いは戻り値が
/// `Path` ではなく [`Instance`] の列であること — `so`/`eo` を捨てずに運ぶため。
///
/// `composite` は**描く順**を決める。`Above` はコピー0が先(= 後の物が上に載る)で、
/// これは移植元が輪郭を積んだ順と同じ。
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
        // 端点を含む線形補間。コピーが1つだけなら start_opacity(0除算しない)。
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

// ---------------------------------------------------------------------------
// trim(移植)
// 弧長パラメータ化(サンプリング近似)による幾何トリム。
// Individually は輪郭を連結した1つの長さ空間として扱う。
// ---------------------------------------------------------------------------

/// De Casteljau 分割: `t` で [start,end] を(先頭・分割点・末尾)の3頂点に割る。
fn split_bezier(v0: &Vertex, v1: &Vertex, t: f64) -> (Vertex, Vertex, Vertex) {
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

/// `[t0,t1]`(0≤t0≤t1≤1)区間の部分曲線を取り出す。両端が 0/1 ならタンジェントを保つ。
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

/// (start,end,offset) から物理窓(0..1 に正規化した弧長分率、from≤to)を最大2つ導出する。
/// **`offset` が「切り出し窓の回転」になるのはここ** — 窓が端を跨いだら2つに割れる。
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

/// 弧長 `[from,to]`(絶対長さ)に重なるセグメント群から新しい開いた輪郭群を切り出す。
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
