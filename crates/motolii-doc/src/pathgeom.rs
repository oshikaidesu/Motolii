//! PathOp幾何実装(D1i-2)。
//!
//! 意味・単位・範囲の正本は `docs/specs/M2-document-model.md`「PathOp意味論表」。
//! 契約: `(path, params, t) → path` の純関数(`Wiggle`のみ`t`とseedに依存する決定論ノイズ)。
//! ここは**解決済みスカラー**(DocParamをキーフレーム評価で落とした後の値)を受け取る —
//! DocParamの評価(データトラック解決・LookAt/Follow解決)は D3(doc→render グラフ変換)の責務。
//! 開路Offsetの拒否はここ(幾何側)で行う: `Document::validate`はSvgAsset/TextPath由来の
//! 開閉を静的に知りえない(レシピはAssetIdしか持たない)。

use crate::schema::{CompositeOrder, LineJoin, PointType, TrimMode};

/// 正準空間の2Dベクトル/点。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    fn add(self, o: Point) -> Point {
        Point {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }

    fn sub(self, o: Point) -> Point {
        Point {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }

    fn scale(self, s: f64) -> Point {
        Point {
            x: self.x * s,
            y: self.y * s,
        }
    }

    fn dot(self, o: Point) -> f64 {
        self.x * o.x + self.y * o.y
    }

    fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    fn normalized(self) -> Point {
        let l = self.length();
        if l < f64::EPSILON {
            Point::ZERO
        } else {
            self.scale(1.0 / l)
        }
    }

    /// CCW回転(正準空間はY-up。角度はラジアン)。
    fn rotate(self, angle: f64) -> Point {
        let (s, c) = angle.sin_cos();
        Point {
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c,
        }
    }
}

/// パス頂点。`in_tangent`/`out_tangent`は頂点相対のcubic bezierハンドル(Lottie `v`/`i`/`o`と同型)。
/// 両方ゼロなら直線(コーナー)。
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// 1輪郭。`closed=false`はOffsetがunsupportedになる唯一の入力条件(意味論表)。
#[derive(Debug, Clone, PartialEq, Default)]
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

/// 複数輪郭からなるパス。各輪郭は独立に処理する(意味論表「複数輪郭」)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    pub contours: Vec<Contour>,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum PathOpError {
    /// v1のOffsetは閉路限定(Clipper2 offset。意味論表)。
    #[error("PathOp::Offset does not support open contours in v1 (closed paths only)")]
    OpenPathOffsetUnsupported,
}

/// Repeater.transformの解決済み表現(Transform2Dの4スロットをスカラー化)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedTransform {
    pub position: Point,
    pub anchor: Point,
    pub scale: Point,
    pub rotation: f64,
}

impl ResolvedTransform {
    pub const IDENTITY: ResolvedTransform = ResolvedTransform {
        position: Point::ZERO,
        anchor: Point::ZERO,
        scale: Point { x: 1.0, y: 1.0 },
        rotation: 0.0,
    };
}

/// `schema::PathOp`の解決済み(DocParam評価後)対応形。D3がDocParam→ここへ落とす。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedPathOp {
    PuckerBloat {
        amount: f64,
    },
    ZigZag {
        amount: f64,
        ridges: f64,
        point_type: PointType,
    },
    Offset {
        distance: f64,
        line_join: LineJoin,
        miter_limit: f64,
    },
    RoundCorners {
        radius: f64,
    },
    Trim {
        start: f64,
        end: f64,
        offset: f64,
        mode: TrimMode,
    },
    Twist {
        angle: f64,
        center: Point,
    },
    Wiggle {
        amp: f64,
        freq: f64,
        seed: u64,
    },
    Repeater {
        copies: f64,
        offset: f64,
        transform: ResolvedTransform,
        composite: CompositeOrder,
        start_opacity: f64,
        end_opacity: f64,
    },
}

/// PathOp意味論表の共通契約を1箇所で執行する純関数ディスパッチャ。
///
/// 退化規約(意味論表「退化」): 空パス→空パス、各輪郭は頂点1以下なら恒等。
pub fn apply(path: &Path, op: &ResolvedPathOp, t: f64) -> Result<Path, PathOpError> {
    if path.contours.is_empty() {
        return Ok(path.clone());
    }
    Ok(match op {
        ResolvedPathOp::PuckerBloat { amount } => {
            map_contours(path, |_, c| pucker_bloat_contour(c, *amount))
        }
        ResolvedPathOp::ZigZag {
            amount,
            ridges,
            point_type,
        } => map_contours(path, |_, c| {
            zigzag_contour(c, *amount, *ridges, *point_type)
        }),
        ResolvedPathOp::Offset {
            distance,
            line_join,
            miter_limit,
        } => {
            let mut out = Path::default();
            for c in &path.contours {
                out.contours
                    .push(offset_contour(c, *distance, *line_join, *miter_limit)?);
            }
            out
        }
        ResolvedPathOp::RoundCorners { radius } => {
            map_contours(path, |_, c| round_corners_contour(c, *radius))
        }
        ResolvedPathOp::Trim {
            start,
            end,
            offset,
            mode,
        } => trim(path, *start, *end, *offset, *mode),
        ResolvedPathOp::Twist { angle, center } => {
            map_contours(path, |_, c| twist_contour(c, *angle, *center))
        }
        ResolvedPathOp::Wiggle { amp, freq, seed } => {
            map_contours(path, |i, c| wiggle_contour(c, *amp, *freq, *seed, i, t))
        }
        ResolvedPathOp::Repeater {
            copies,
            offset,
            transform,
            composite,
            start_opacity: _,
            end_opacity: _,
        } => {
            // opacityは幾何(頂点座標)に影響しない — 合成時の重み付けはD3/render側の責務(F-7注)。
            repeater_path(path, *copies, *offset, transform, *composite)
        }
    })
}

fn map_contours(path: &Path, f: impl Fn(usize, &Contour) -> Contour) -> Path {
    Path {
        contours: path
            .contours
            .iter()
            .enumerate()
            .map(|(i, c)| f(i, c))
            .collect(),
    }
}

fn centroid_of(vertices: &[Vertex]) -> Point {
    let sum = vertices.iter().fold(Point::ZERO, |acc, v| acc.add(v.point));
    sum.scale(1.0 / vertices.len() as f64)
}

// ---------------------------------------------------------------------------
// pucker_bloat: amount∈[-1,1]。0=恒等、+1=頂点が重心へ、-1=重心から距離2倍。
// 接線は頂点と逆向きに補間(意味論表) — 頂点の絶対変位が-amount*dなら、ハンドルの
// 絶対位置は+amount*dだけ動く(d=頂点-重心)。これにより一次実装として自己整合的に固定する。
// ---------------------------------------------------------------------------
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
            Vertex {
                point: new_point,
                in_tangent: v.in_tangent.add(handle_shift),
                out_tangent: v.out_tangent.add(handle_shift),
            }
        })
        .collect();
    Contour {
        vertices,
        closed: c.closed,
    }
}

// ---------------------------------------------------------------------------
// zig_zag: 既存の頂点間コード(直線近似。入力タンジェントは捨てる — v1簡略化)を
// ridges*2分割し、外向き法線方向に交互にamountだけ変位させる。
// point_type=corner→ゼロタンジェント、smooth→前後点方向の自動タンジェント。
// ---------------------------------------------------------------------------
fn zigzag_contour(c: &Contour, amount: f64, ridges: f64, point_type: PointType) -> Contour {
    if c.vertices.len() <= 1 {
        return c.clone();
    }
    let ridge_count = ridges.max(0.0).round() as usize;
    if ridge_count == 0 || amount == 0.0 {
        return c.clone();
    }
    let n = c.vertices.len();
    let edge_count = if c.closed { n } else { n - 1 };
    let mut points: Vec<Point> = Vec::new();
    let steps = ridge_count * 2;
    for e in 0..edge_count {
        let a = c.vertices[e].point;
        let b = c.vertices[(e + 1) % n].point;
        points.push(a);
        let dir = b.sub(a);
        let len = dir.length();
        if len < f64::EPSILON {
            continue;
        }
        let unit = dir.scale(1.0 / len);
        let normal = Point {
            x: -unit.y,
            y: unit.x,
        };
        for k in 1..steps {
            let f = k as f64 / steps as f64;
            let base = a.add(dir.scale(f));
            let sign = if k % 2 == 1 { 1.0 } else { -1.0 };
            points.push(base.add(normal.scale(sign * amount)));
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

// ---------------------------------------------------------------------------
// round_corners: 各頂点(開路は両端を除く)を半径radiusのfilletへ置換。
// タンジェントハンドルは弧をcubic bezier近似(90°ごと分割)して保持する。
// ---------------------------------------------------------------------------
fn round_corners_contour(c: &Contour, radius: f64) -> Contour {
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

fn normalize_angle(a: f64) -> f64 {
    let two_pi = std::f64::consts::TAU;
    let mut x = a % two_pi;
    if x <= -std::f64::consts::PI {
        x += two_pi;
    } else if x > std::f64::consts::PI {
        x -= two_pi;
    }
    x
}

/// 弧(center, radius, a0→a1)をcubic bezier近似で頂点列化する(90°以下ごとに分割)。
fn arc_vertices(center: Point, radius: f64, a0: f64, a1: f64) -> Vec<Vertex> {
    if radius <= 0.0 || (a1 - a0).abs() < 1e-12 {
        let p = center.add(Point {
            x: radius * a0.cos(),
            y: radius * a0.sin(),
        });
        return vec![Vertex::corner(p)];
    }
    let sweep = a1 - a0;
    let max_seg = std::f64::consts::FRAC_PI_2;
    // 浮動小数の丸め誤差でちょうど90°がわずかに超過してsegmentsが1つ増えないよう許容誤差を入れる。
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

// ---------------------------------------------------------------------------
// offset: 閉路限定(意味論表)。エッジを外向き法線方向にdistanceだけ平行移動し、
// line_joinで角を結合する(Clipper2 offset型)。自己交差の修復はしない。
// ---------------------------------------------------------------------------
fn offset_contour(
    c: &Contour,
    distance: f64,
    line_join: LineJoin,
    miter_limit: f64,
) -> Result<Contour, PathOpError> {
    if c.vertices.len() <= 1 {
        return Ok(c.clone());
    }
    if !c.closed {
        return Err(PathOpError::OpenPathOffsetUnsupported);
    }
    let pts: Vec<Point> = c.vertices.iter().map(|v| v.point).collect();
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
        let shift = outward.scale(distance);
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
            distance,
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

/// prev_b(前エッジの終点)とcur_a(現エッジの始点)の間隙を`line_join`で塞ぐ。
/// Miter成立時は交点1つが両者を置き換える(prev_b/cur_aどちらも残らない)。
/// Bevel/Round、またMiterの`miter_limit`超過フォールバックはprev_b/cur_aを両方残す。
#[allow(clippy::too_many_arguments)]
fn join_corner(
    out: &mut Vec<Point>,
    prev_a: Point,
    prev_b: Point,
    cur_a: Point,
    cur_b: Point,
    vertex: Point,
    distance: f64,
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
            let limit_len = miter_limit * distance.abs().max(f64::EPSILON);
            if miter_len <= limit_len {
                out.push(p);
                return;
            }
        }
        // 交点なし(平行)またはmiter_limit超過: Clipper2既定と同じくbevelへ縮退。
    }
    out.push(prev_b);
    if line_join == LineJoin::Round {
        let r = distance.abs();
        if r > f64::EPSILON {
            let a0 = (prev_b.y - vertex.y).atan2(prev_b.x - vertex.x);
            let a1 = (cur_a.y - vertex.y).atan2(cur_a.x - vertex.x);
            let diff = normalize_angle(a1 - a0);
            let arc = arc_vertices(vertex, r, a0, a0 + diff);
            let interior = arc.len().saturating_sub(2);
            for v in arc.iter().skip(1).take(interior) {
                out.push(v.point);
            }
        }
    }
    out.push(cur_a);
}

// ---------------------------------------------------------------------------
// twist: 各輪郭内で中心からの最大距離を基準に自己正規化する減衰回転(AE Twist)。
// 中心で最大角度、輪郭自身の外縁でゼロになる — 外部半径パラメータを持たない
// (意味論表がradiusを列挙していないため、輪郭固有の自己正規化を採る)。
// ---------------------------------------------------------------------------
fn twist_contour(c: &Contour, angle: f64, center: Point) -> Contour {
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

// ---------------------------------------------------------------------------
// wiggle: PCG32ベースのvalue noise(意味論表`pcg32_value_noise`)。頂点ごとに
// 独立した決定論的乱数を(seed, 輪郭index, 頂点index, 軸)から導出する。
// ---------------------------------------------------------------------------
fn pcg32_hash(mut state: u64) -> u32 {
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let xorshifted = (((state >> 18) ^ state) >> 27) as u32;
    let rot = (state >> 59) as u32;
    xorshifted.rotate_right(rot)
}

fn combine_u64(a: u64, b: u64) -> u64 {
    a.wrapping_mul(0x9E3779B97F4A7C15) ^ b.wrapping_mul(0xC2B2AE3D27D4EB4F)
}

fn hash_to_unit(seed: u64, salt: u64) -> f64 {
    let combined = combine_u64(seed, salt);
    let h = pcg32_hash(combined);
    (h as f64 / u32::MAX as f64) * 2.0 - 1.0
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise_1d(seed: u64, salt: u64, t: f64) -> f64 {
    let k = t.floor();
    let frac = t - k;
    let ki = k as i64 as u64;
    let a = hash_to_unit(seed, combine_u64(salt, ki));
    let b = hash_to_unit(seed, combine_u64(salt, ki.wrapping_add(1)));
    let s = smoothstep(frac);
    a + (b - a) * s
}

fn wiggle_contour(
    c: &Contour,
    amp: f64,
    freq: f64,
    seed: u64,
    contour_idx: usize,
    t: f64,
) -> Contour {
    if c.vertices.len() <= 1 {
        return c.clone();
    }
    let vertices = c
        .vertices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let point_salt = combine_u64(contour_idx as u64, i as u64);
            let nx = value_noise_1d(seed, combine_u64(point_salt, 0), t * freq);
            let ny = value_noise_1d(seed, combine_u64(point_salt, 1), t * freq);
            let disp = Point {
                x: nx * amp,
                y: ny * amp,
            };
            Vertex {
                point: v.point.add(disp),
                in_tangent: v.in_tangent,
                out_tangent: v.out_tangent,
            }
        })
        .collect();
    Contour {
        vertices,
        closed: c.closed,
    }
}

// ---------------------------------------------------------------------------
// repeater: M = T(position)·R(rotation)·S(scale)·T(-anchor) をk=index+offset回
// 合成適用する。整数kは行列の反復合成、小数部はk,k+1の行列を線形補間する
// (2Dアフィンの真の実数冪=Lie群指数写像は本表が要求しないため、v1簡略近似として明示する)。
// opacityは幾何(頂点)に影響しない(F-7注: Duplicator全体はPathOpに畳まない)。
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

    /// self ∘ rhs (rhsを先に適用)。
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
}

fn build_affine(t: &ResolvedTransform) -> Affine {
    let (s, c) = t.rotation.sin_cos();
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
    if n <= 0 {
        return Affine::IDENTITY;
    }
    let mut result = Affine::IDENTITY;
    for _ in 0..n {
        result = m.mul(&result);
    }
    result
}

fn affine_pow_real(m: &Affine, k: f64) -> Affine {
    if k <= 0.0 {
        return Affine::IDENTITY;
    }
    let lo = k.floor();
    let frac = k - lo;
    let m_lo = affine_pow_int(m, lo as i64);
    if frac <= f64::EPSILON {
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

fn repeater_path(
    path: &Path,
    copies: f64,
    offset: f64,
    transform: &ResolvedTransform,
    composite: CompositeOrder,
) -> Path {
    let n = copies.max(0.0).floor() as i64;
    if n <= 0 {
        return Path::default();
    }
    let m = build_affine(transform);
    let order: Vec<i64> = match composite {
        CompositeOrder::Above => (0..n).collect(),
        CompositeOrder::Below => (0..n).rev().collect(),
    };
    let mut out = Path::default();
    for i in order {
        let k = (i as f64 + offset).max(0.0);
        let mk = affine_pow_real(&m, k);
        for c in &path.contours {
            out.contours.push(apply_matrix_to_contour(c, &mk));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// trim: 弧長パラメータ化(サンプリング近似)による幾何トリム。sequentialは
// 輪郭を連結した1つの長さ空間として扱う(意味論表)。
// ---------------------------------------------------------------------------
const ARC_SAMPLES: usize = 24;

/// タンジェントが両方ゼロ(`Vertex::corner`)の直線区間か。
/// 直線は等速(弧長∝t)でパラメータ化する — ゼロ長ハンドルの退化cubicは
/// 数式上イーズ曲線になり弧長サンプリングが不正確になるため特殊扱いする。
fn is_straight(v0: &Vertex, v1: &Vertex) -> bool {
    v0.out_tangent == Point::ZERO && v1.in_tangent == Point::ZERO
}

fn bezier_point(v0: &Vertex, v1: &Vertex, t: f64) -> Point {
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

fn segment_sample_lengths(v0: &Vertex, v1: &Vertex) -> ([f64; ARC_SAMPLES + 1], f64) {
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

fn t_at_length(cum: &[f64; ARC_SAMPLES + 1], total_len: f64, target: f64) -> f64 {
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

fn lerp_point(a: Point, b: Point, t: f64) -> Point {
    a.add(b.sub(a).scale(t))
}

/// De Casteljau分割: `t`で[start,end]を(先頭・分割点・末尾)の3頂点に割る。
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

/// `[t0,t1]`(0≤t0≤t1≤1)区間の部分曲線を取り出す。両端が0/1ならタンジェントを保つ。
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

/// (start,end,offset)から物理窓(0..1に正規化した弧長分率、from≤to)を最大2つ導出する。
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

/// 弧長`[from,to]`(絶対長さ)に重なるセグメント群から新しい開いた輪郭群を切り出す。
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

fn trim(path: &Path, start: f64, end: f64, offset: f64, mode: TrimMode) -> Path {
    let windows = resolve_windows(start, end, offset);
    if windows.is_empty() {
        return Path::default();
    }
    match mode {
        TrimMode::Parallel => {
            let mut out = Path::default();
            for c in &path.contours {
                if c.vertices.len() <= 1 {
                    out.contours.push(c.clone());
                    continue;
                }
                let segs = flatten_segments(std::slice::from_ref(c));
                let total: f64 = segs.iter().map(|s| s.len).sum();
                if total <= f64::EPSILON {
                    out.contours.push(c.clone());
                    continue;
                }
                for (fs, ft) in &windows {
                    out.contours
                        .extend(extract_window(&segs, fs * total, ft * total));
                }
            }
            out
        }
        TrimMode::Sequential => {
            let segs = flatten_segments(&path.contours);
            let total: f64 = segs.iter().map(|s| s.len).sum();
            if total <= f64::EPSILON {
                return path.clone();
            }
            let mut out = Path::default();
            for (fs, ft) in &windows {
                out.contours
                    .extend(extract_window(&segs, fs * total, ft * total));
            }
            out
        }
    }
}
