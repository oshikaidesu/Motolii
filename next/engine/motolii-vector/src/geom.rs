//! パスの器と、パラメータからパスを作る源。
//!
//! **旧 workspace `crates/motolii-doc/src/pathgeom.rs` からの移植**(裁定10)。
//! `Point` / `Vertex` / `Contour` / `Path` / `rect` / `ellipse` は**そのまま**持ってきた。
//! 名前も変えていない — 変えると移植元を辿れなくなり、次に直す人が二重に読む。
//!
//! **落としたもの**(軸4「使う分だけ移植する」): `Point::rotate`(twist 専用)/
//! `Point::dot` 以外の未使用ヘルパ / `axis_aligned_rect` / `axis_aligned_ellipse`
//! (GPU OverlayRect 退化判定。この crate は板1枚を返すので要らない)/
//! `centroid_of`(pucker-bloat 専用)。要る日に旧 workspace から取る。
//!
//! **座標系**: 原点左上・Y 下向きの **AE comp 座標**(裁定14)。移植元は Y-up
//! 前提だったが、同じ回転行列を Y-down 空間に置くと画面上は**時計回り**になり、
//! それは Lottie/AE の rotation の向きそのもの(裁定58)。よって式は1行も変えず、
//! 空間の宣言だけを comp 座標へ合わせてある。

/// 正準空間の2Dベクトル/点。
///
/// Lottie の `s`(Size)は Vec2 の**単一 property** なので、幅・高さもこの型で持つ
/// (`x` = 幅 / `y` = 高さ)。x と y を別 property へ割らない理由は裁定61 と同じ —
/// 割ると片方だけキーが打たれた不正状態が作れる。
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

/// パス頂点。`in_tangent`/`out_tangent` は頂点相対の cubic bezier ハンドルで、
/// **Lottie `bezier` の `v`/`i`/`o` と同型**。両方ゼロなら直線(コーナー)。
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

/// 1輪郭。`closed` は Lottie `bezier.c`。
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

/// 複数輪郭からなるパス。各輪郭は独立に処理する。
pub type Path = Vec<Contour>;

/// 局所原点中央の軸平行矩形(`rectangle.s`)。
///
/// 位置・回転・角丸を持たないのは裁定74 — 層の transform と repeater の anchor、
/// および `rounded-corners` 演算子で賄えるので、持つと正本が2つになる。
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

/// 4-cubic 楕円(`ellipse.s`)。
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

// ---------------------------------------------------------------------------
// ベジエの共通部品(移植)。trim と rounded-corners が共有する。
// ---------------------------------------------------------------------------

/// タンジェントが両方ゼロ(`Vertex::corner`)の直線区間か。
/// 直線は等速(弧長∝t)でパラメータ化する — ゼロ長ハンドルの退化 cubic は
/// 数式上イーズ曲線になり弧長サンプリングが不正確になるため特殊扱いする。
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

/// 弧(center, radius, a0→a1)を cubic bezier 近似で頂点列化する(90°以下ごとに分割)。
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
    // 浮動小数の丸め誤差でちょうど90°がわずかに超過して segments が1つ増えないよう許容誤差を入れる。
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
