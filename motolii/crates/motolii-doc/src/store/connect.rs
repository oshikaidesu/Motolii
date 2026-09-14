//! 箱と箱をつなぐ線(2026-09-15 利用者「箱と箱を繋ぐ線は第一級の機能。破線や垂れた線など物理を出すのに適切、
//! ネイティブになると組み合わせの数が膨大になる」)。先例: leader-line.js(start / end、socket、path の
//! straight / arc / grid、dash)、FigJam のコネクタ(直線・曲線・折れ線)、Keynote の接続線。
//!
//! 形の層が `Connect From` と `Connect To` を持つと、その輪郭は 2 つの箱から毎コマ解いた道に差し替わる。
//! 線の色・太さ・破線・Trim Paths は形のまま。道は線の層の親の空間で解き、層の変換は道をそのまま置く。
//! 時刻の純関数(垂れは懸垂線の閉じた式)。線の Transition は端を箱に付けたまま、腹だけを遅らせる。

use std::cell::RefCell;

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;
use crate::doc::store::layout::{CONNECT_FROM, CONNECT_TO, DASH, DASH_GAP, DASH_OFFSET, FROM_SIDE, HANDLE_SIZE, LINE_PATH, MARGIN, SLACK, TO_SIDE, TRACE};
use crate::doc::store::{LayerId, PropertyId, ShapeNode, StoreError, StoreView};
use crate::doc::vector::{Contour, Dash, PathSource, Point, Vertex};

const HANG_POINTS: usize = 33;

/// 解いた道の点(接線は点からの相対)。
#[derive(Clone, Debug, PartialEq)]
struct Route {
    points: Vec<[glam::Vec2; 3]>,
}

impl StoreView<'_> {
    /// つなぐ線なら、その 2 つの相手。
    pub(crate) fn connection(&self, layer: LayerId, t: RationalTime) -> Result<Option<(LayerId, LayerId)>, StoreError> {
        let id = |name: &str| -> Result<Option<LayerId>, StoreError> {
            Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
                Some(Value::LayerId(id)) if id != 0 => Some(LayerId(id)),
                Some(Value::F64(v)) if v >= 1.0 => Some(LayerId(v.round() as u64)),
                _ => None,
            })
        };
        let (Some(from), Some(to)) = (id(CONNECT_FROM)?, id(CONNECT_TO)?) else { return Ok(None) };
        if from == layer || to == layer || !self.here(from, t)? || !self.here(to, t)? {
            return Ok(None);
        }
        Ok(Some((from, to)))
    }

    /// なぞる形なら、その相手と形の種類(`Connect To` が無く `Trace` が None でない)。
    pub(crate) fn tracing(&self, layer: LayerId, t: RationalTime) -> Result<Option<(LayerId, i64)>, StoreError> {
        let kind = self.choice(layer, TRACE, t)?;
        if kind <= 0 {
            return Ok(None);
        }
        let target = match self.value_at(layer, &PropertyId::new(CONNECT_FROM)?, t)? {
            Some(Value::LayerId(id)) if id != 0 => LayerId(id),
            Some(Value::F64(v)) if v >= 1.0 => LayerId(v.round() as u64),
            _ => return Ok(None),
        };
        if target == layer || !self.here(target, t)? {
            return Ok(None);
        }
        Ok(Some((target, kind)))
    }

    /// 形の層の姿を、つなぐ線なら解いた道に、なぞる形ならその輪郭に差し替える(色・太さ・効果の積みは形のまま、破線は Dash の欄)。
    pub(crate) fn connect_shapes(&self, layer: LayerId, t: RationalTime, shapes: Vec<ShapeNode>) -> Result<Vec<ShapeNode>, StoreError> {
        let path = if let Some(route) = self.route(layer, t)? {
            vec![Contour {
                closed: false,
                vertices: route.points.iter().map(|[p, i, o]| Vertex {
                    point: Point { x: f64::from(p.x), y: f64::from(p.y) },
                    in_tangent: Point { x: f64::from(i.x), y: f64::from(i.y) },
                    out_tangent: Point { x: f64::from(o.x), y: f64::from(o.y) },
                }).collect(),
            }]
        } else if let Some(path) = self.trace_path(layer, t)? {
            path
        } else {
            return Ok(shapes);
        };
        let dash = self.number(layer, DASH, 0.0, t)?.max(0.0);
        let gap = self.number(layer, DASH_GAP, dash, t)?.max(0.0);
        let offset = self.number(layer, DASH_OFFSET, 0.0, t)?;
        fn replace(nodes: Vec<ShapeNode>, path: &crate::doc::vector::Path, dash: Option<&Dash>) -> Vec<ShapeNode> {
            nodes.into_iter().map(|node| match node {
                ShapeNode::Leaf(mut shape) => {
                    shape.source = PathSource::Bezier(path.clone());
                    // 塗りは閉じた形(掴み・外枠・円)だけ。
                    if !path.iter().any(|c| c.closed) {
                        shape.fill = None;
                    }
                    if let (Some(stroke), Some(dash)) = (shape.stroke.as_mut(), dash) {
                        stroke.dash = Some(dash.clone());
                    }
                    ShapeNode::Leaf(shape)
                }
                ShapeNode::Group(mut group) => {
                    group.transform = crate::doc::vector::RepeaterTransform::IDENTITY;
                    group.children = replace(group.children, path, dash);
                    ShapeNode::Group(group)
                }
            }).collect()
        }
        let dash = (dash > 0.0).then(|| Dash { pattern: vec![dash, gap.max(1e-3)], offset });
        Ok(replace(shapes, &path, dash.as_ref()))
    }

    /// つなぐ線の置き場所(親の空間での Position)。道は親の空間で解いてあるので、素材座標の原点のずれだけ戻す。
    pub(crate) fn connector_position(&self, layer: LayerId, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
        if self.connection(layer, t)?.is_none() && self.tracing(layer, t)?.is_none() {
            return Ok(None);
        }
        let shapes = self.shapes_at(layer, t)?;
        let Ok(Some(canvas)) = crate::doc::vector::content_canvas(&shapes) else { return Ok(None) };
        Ok(Some([-(canvas.origin_x as f32), -(canvas.origin_y as f32)]))
    }

    /// なぞる形の輪郭(線の層の親の空間)。
    fn trace_path(&self, layer: LayerId, t: RationalTime) -> Result<Option<crate::doc::vector::Path>, StoreError> {
        let Some((target, kind)) = self.tracing(layer, t)? else { return Ok(None) };
        let Some((lo, hi)) = self.box_seen_from(target, layer, t)? else { return Ok(None) };
        let m = self.number(layer, MARGIN, 0.0, t)? as f32;
        let (lo, hi) = (lo - glam::Vec2::splat(m), hi + glam::Vec2::splat(m));
        let p = |v: glam::Vec2| Point { x: f64::from(v.x), y: f64::from(v.y) };
        let rect = |lo: glam::Vec2, hi: glam::Vec2| Contour::closed([p(lo), p(glam::vec2(hi.x, lo.y)), p(hi), p(glam::vec2(lo.x, hi.y))]);
        Ok(Some(match kind {
            2 => {
                let h = self.number(layer, HANDLE_SIZE, 10.0, t)?.max(0.0) as f32 * 0.5;
                [lo, glam::vec2(hi.x, lo.y), hi, glam::vec2(lo.x, hi.y)].into_iter().map(|c| rect(c - glam::Vec2::splat(h), c + glam::Vec2::splat(h))).collect()
            }
            3 => vec![Contour::open([p(lo), p(hi)]), Contour::open([p(glam::vec2(hi.x, lo.y)), p(glam::vec2(lo.x, hi.y))])],
            4 => {
                // 箱に内接する楕円(4 つの 3 次で円弧を近似)。
                let (c, r) = ((lo + hi) * 0.5, (hi - lo) * 0.5);
                let k = 0.552_284_8;
                let v = |q: glam::Vec2, i: glam::Vec2, o: glam::Vec2| Vertex { point: p(q), in_tangent: p(i), out_tangent: p(o) };
                vec![Contour { closed: true, vertices: vec![
                    v(c + glam::vec2(r.x, 0.0), glam::vec2(0.0, -r.y * k), glam::vec2(0.0, r.y * k)),
                    v(c + glam::vec2(0.0, r.y), glam::vec2(r.x * k, 0.0), glam::vec2(-r.x * k, 0.0)),
                    v(c - glam::vec2(r.x, 0.0), glam::vec2(0.0, r.y * k), glam::vec2(0.0, -r.y * k)),
                    v(c - glam::vec2(0.0, r.y), glam::vec2(-r.x * k, 0.0), glam::vec2(r.x * k, 0.0)),
                ] }]
            }
            5 => {
                // 箱の 4 辺を、画面(comp)の端から端まで伸ばす。
                let Some(comp) = self.composition()? else { return Ok(None) };
                let screen = [glam::Vec2::ZERO, glam::vec2(comp.width as f32, comp.height as f32)];
                let (s_lo, s_hi) = match self.attrs(layer)?.unwrap_or_default().parent {
                    Some(parent) => {
                        let inverse = self.world_2d(parent, t)?.inverse();
                        let corners = [screen[0], glam::vec2(screen[1].x, 0.0), screen[1], glam::vec2(0.0, screen[1].y)].map(|q| inverse.transform_point2(q));
                        corners.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(a, b), q| (a.min(*q), b.max(*q)))
                    }
                    None => (screen[0], screen[1]),
                };
                vec![
                    Contour::open([p(glam::vec2(s_lo.x, lo.y)), p(glam::vec2(s_hi.x, lo.y))]),
                    Contour::open([p(glam::vec2(s_lo.x, hi.y)), p(glam::vec2(s_hi.x, hi.y))]),
                    Contour::open([p(glam::vec2(lo.x, s_lo.y)), p(glam::vec2(lo.x, s_hi.y))]),
                    Contour::open([p(glam::vec2(hi.x, s_lo.y)), p(glam::vec2(hi.x, s_hi.y))]),
                ]
            }
            _ => vec![rect(lo, hi)],
        }))
    }

    /// 道: 今の端に、移り方で遅れた腹(弦からのずれ)を足す。
    fn route(&self, layer: LayerId, t: RationalTime) -> Result<Option<Route>, StoreError> {
        // 線が線につながって輪になれば、2 度目は解かない。
        thread_local! { static ROUTING: RefCell<std::collections::HashSet<(u64, i64, i64)>> = RefCell::new(Default::default()); }
        let key = (layer.0, t.num(), t.den());
        if !ROUTING.with(|r| r.borrow_mut().insert(key)) {
            return Ok(None);
        }
        let result = (|| {
            let Some(now) = self.route_at(layer, t)? else { return Ok(None) };
            let samples = self.transition_samples(layer, t)?;
            if samples.is_empty() {
                return Ok(Some(now));
            }
            let n = now.points.len();
            let chord = |route: &Route, i: usize| {
                let u = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.0 };
                route.points[0][0].lerp(route.points[n - 1][0], u)
            };
            let mut belly = vec![[glam::Vec2::ZERO; 3]; n];
            let mut total = 0.0f32;
            for (at, w) in samples {
                let Some(past) = self.route_at(layer, at)? else { continue };
                if past.points.len() != n {
                    continue;
                }
                for i in 0..n {
                    belly[i][0] += (past.points[i][0] - chord(&past, i)) * w;
                    belly[i][1] += past.points[i][1] * w;
                    belly[i][2] += past.points[i][2] * w;
                }
                total += w;
            }
            if total <= 1e-6 {
                return Ok(Some(now));
            }
            let points = (0..n).map(|i| {
                // 端は今の箱に付けたまま。
                if i == 0 || i == n - 1 {
                    return [now.points[i][0], belly[i][1] / total, belly[i][2] / total];
                }
                [chord(&now, i) + belly[i][0] / total, belly[i][1] / total, belly[i][2] / total]
            }).collect();
            Ok(Some(Route { points }))
        })();
        ROUTING.with(|r| r.borrow_mut().remove(&key));
        result
    }

    /// その時刻の箱だけから解いた道(線の層の親の空間)。
    fn route_at(&self, layer: LayerId, t: RationalTime) -> Result<Option<Route>, StoreError> {
        let Some((from, to)) = self.connection(layer, t)? else { return Ok(None) };
        let (Some(a), Some(b)) = (self.box_seen_from(from, layer, t)?, self.box_seen_from(to, layer, t)?) else { return Ok(None) };
        let margin = self.number(layer, MARGIN, 0.0, t)? as f32;
        let (ca, cb) = ((a.0 + a.1) * 0.5, (b.0 + b.1) * 0.5);
        let (p0, n0) = socket(a, cb, self.choice(layer, FROM_SIDE, t)?, margin);
        let (p1, n1) = socket(b, ca, self.choice(layer, TO_SIDE, t)?, margin);
        let corner = |p: glam::Vec2| [p, glam::Vec2::ZERO, glam::Vec2::ZERO];
        let distance = (p1 - p0).length();
        let points = match self.choice(layer, LINE_PATH, t)? {
            // Curved: 端の向きに伸ばした 3 次(leader-line の fluid、FigJam の曲線)。
            1 => {
                let k = distance * 0.4;
                vec![[p0, glam::Vec2::ZERO, n0 * k], [p1, n1 * k, glam::Vec2::ZERO]]
            }
            // Elbow: 軸に沿って折る(leader-line の grid、FigJam の折れ線)。
            2 => {
                if n0.x.abs() >= n0.y.abs() {
                    let mx = (p0.x + p1.x) * 0.5;
                    vec![corner(p0), corner(glam::vec2(mx, p0.y)), corner(glam::vec2(mx, p1.y)), corner(p1)]
                } else {
                    let my = (p0.y + p1.y) * 0.5;
                    vec![corner(p0), corner(glam::vec2(p0.x, my)), corner(glam::vec2(p1.x, my)), corner(p1)]
                }
            }
            // Hang: 縄の長さ = 端の距離 × (1 + Slack %)、懸垂線(重力は親の空間の下)。
            3 => hang(p0, p1, distance * (1.0 + self.number(layer, SLACK, 20.0, t)?.max(0.0) as f32 / 100.0)),
            _ => vec![corner(p0), corner(p1)],
        };
        Ok(Some(Route { points }))
    }
}

/// 箱の端の点と外向き。Auto は相手の中心へ向かう線が箱(+ 間)を出る所(相手が回っても点が跳ばない)。
fn socket(b: (glam::Vec2, glam::Vec2), toward: glam::Vec2, side: i64, margin: f32) -> (glam::Vec2, glam::Vec2) {
    let (lo, hi) = (b.0 - glam::Vec2::splat(margin), b.1 + glam::Vec2::splat(margin));
    let c = (b.0 + b.1) * 0.5;
    match side {
        1 => (glam::vec2(c.x, lo.y), glam::Vec2::NEG_Y),
        2 => (glam::vec2(hi.x, c.y), glam::Vec2::X),
        3 => (glam::vec2(c.x, hi.y), glam::Vec2::Y),
        4 => (glam::vec2(lo.x, c.y), glam::Vec2::NEG_X),
        5 => (c, (toward - c).normalize_or_zero()),
        _ => {
            let d = toward - c;
            if d.length() < 1e-4 {
                return (c, glam::Vec2::X);
            }
            let half = (hi - lo) * 0.5;
            let s = [half.x / d.x.abs().max(1e-6), half.y / d.y.abs().max(1e-6)].into_iter().fold(f32::INFINITY, f32::min);
            (c + d * s.min(1.0), d.normalize())
        }
    }
}

/// 2 点の間に長さ `length` の縄を垂らす(懸垂線)。点は Catmull-Rom の接線で滑らかにつなぐ。
fn hang(p0: glam::Vec2, p1: glam::Vec2, length: f32) -> Vec<[glam::Vec2; 3]> {
    let (left, right) = if p0.x <= p1.x { (p0, p1) } else { (p1, p0) };
    let h = right.x - left.x;
    // 上向きの y で解く(画面の y は下向き)。
    let v = -(right.y - left.y);
    let chord = (h * h + v * v).sqrt();
    let mut points: Vec<glam::Vec2> = if h < 1e-3 || length <= chord + 1e-3 {
        (0..HANG_POINTS).map(|i| left.lerp(right, i as f32 / (HANG_POINTS - 1) as f32)).collect()
    } else {
        // sinh(ξ)/ξ = √(L² − v²)/h を ξ について解き、a = h / 2ξ。
        let r = ((length * length - v * v).sqrt() / h) as f64;
        let (mut lo, mut hi) = (1e-9f64, 40.0f64);
        for _ in 0..80 {
            let mid = (lo + hi) * 0.5;
            if mid.sinh() / mid < r { lo = mid } else { hi = mid }
        }
        let xi = (lo + hi) * 0.5;
        let a = f64::from(h) / (2.0 * xi);
        let xc = f64::from(h) * 0.5 - a * (f64::from(v) / f64::from(length)).clamp(-0.999_999, 0.999_999).atanh();
        let base = (-xc / a).cosh();
        (0..HANG_POINTS).map(|i| {
            let x = f64::from(h) * i as f64 / (HANG_POINTS - 1) as f64;
            let up = a * (((x - xc) / a).cosh() - base);
            glam::vec2(left.x + x as f32, left.y - up as f32)
        }).collect()
    };
    if p0.x > p1.x {
        points.reverse();
    }
    let n = points.len();
    (0..n).map(|i| {
        let prev = points[i.saturating_sub(1)];
        let next = points[(i + 1).min(n - 1)];
        let tangent = (next - prev) / 6.0;
        let (inward, outward) = match i {
            0 => (glam::Vec2::ZERO, tangent),
            _ if i == n - 1 => (-tangent, glam::Vec2::ZERO),
            _ => (-tangent, tangent),
        };
        [points[i], inward, outward]
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{blank_project, property, rect_shape, Document, Intent, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};

    const T: RationalTime = RationalTime::ZERO;

    fn rect(doc: &mut Document, id: u64, size: [f32; 2], at: [f64; 2]) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: id as i16, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(None), ..Default::default() } },
            Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], size)] },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(at) },
        ]).unwrap();
        layer
    }

    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }

    /// 画面の上の箱(世界)。
    fn shown(doc: &Document, layer: LayerId) -> [f32; 4] {
        let view = doc.view();
        let b = view.layer_box(layer, T).unwrap().unwrap();
        let world = view.world_2d(layer, T).unwrap();
        let (lo, hi) = (world.transform_point2(glam::vec2(b[0], b[1])), world.transform_point2(glam::vec2(b[2], b[3])));
        [lo.x.min(hi.x), lo.y.min(hi.y), lo.x.max(hi.x), lo.y.max(hi.y)]
    }

    #[test]
    fn a_trace_draws_around_one_box_and_its_guides_cross_the_screen() {
        let mut doc = blank_project();
        let a = rect(&mut doc, 1, [200.0, 100.0], [500.0, 300.0]);
        let ba = shown(&doc, a);
        let hud = rect(&mut doc, 2, [10.0, 10.0], [0.0, 0.0]);
        put(&mut doc, hud, CONNECT_FROM, Value::LayerId(a.0));
        put(&mut doc, hud, MARGIN, Value::F64(4.0));
        put(&mut doc, hud, TRACE, Value::Enum(1));
        let h = shown(&doc, hud);
        assert!((0..4).all(|i| (h[i] - ba[i] + if i < 2 { 4.0 } else { -4.0 }).abs() < 0.5), "Outline: the box a margin out: {ba:?} {h:?}");
        put(&mut doc, hud, TRACE, Value::Enum(2));
        put(&mut doc, hud, HANDLE_SIZE, Value::F64(12.0));
        let h = shown(&doc, hud);
        assert!((h[0] - (ba[0] - 4.0 - 6.0)).abs() < 0.5 && (h[3] - (ba[3] + 4.0 + 6.0)).abs() < 0.5, "Handles: squares centred on the corners: {h:?}");
        put(&mut doc, hud, TRACE, Value::Enum(5));
        let comp = doc.view().composition().unwrap().unwrap();
        let h = shown(&doc, hud);
        assert!(h[0] <= 0.5 && h[2] >= comp.width as f32 - 0.5 && h[1] <= 0.5 && h[3] >= comp.height as f32 - 0.5, "Guides run edge to edge of the screen: {h:?}");
        put(&mut doc, a, property::POSITION, Value::Vec2([900.0, 600.0]));
        put(&mut doc, hud, TRACE, Value::Enum(4));
        let (ba, h) = (shown(&doc, a), shown(&doc, hud));
        assert!(((h[0] + h[2]) - (ba[0] + ba[2])).abs() < 1.0, "Circle: centred on the box, and follows it: {ba:?} {h:?}");
    }

    #[test]
    fn a_line_joins_two_boxes_edge_to_edge_and_follows_them() {
        let mut doc = blank_project();
        let a = rect(&mut doc, 1, [100.0, 100.0], [200.0, 300.0]);
        let b = rect(&mut doc, 2, [100.0, 100.0], [700.0, 300.0]);
        let (ba, bb) = (shown(&doc, a), shown(&doc, b));
        let line = rect(&mut doc, 3, [10.0, 10.0], [0.0, 0.0]);
        put(&mut doc, line, CONNECT_FROM, Value::LayerId(a.0));
        put(&mut doc, line, CONNECT_TO, Value::LayerId(b.0));
        put(&mut doc, line, MARGIN, Value::F64(5.0));
        let l = shown(&doc, line);
        assert!((l[0] - (ba[2] + 5.0)).abs() < 0.5 && (l[2] - (bb[0] - 5.0)).abs() < 0.5, "from the right edge of one to the left edge of the other, a margin off: {ba:?} {bb:?} {l:?}");
        assert!((l[1] - (ba[1] + ba[3]) * 0.5).abs() < 0.5, "level with their middles");

        put(&mut doc, b, property::POSITION, Value::Vec2([700.0, 700.0]));
        let (bb, l) = (shown(&doc, b), shown(&doc, line));
        assert!((l[3] - bb[1]).abs() < 60.0 && l[3] > 500.0, "moving a box carries the end: {bb:?} {l:?}");

        put(&mut doc, b, property::POSITION, Value::Vec2([700.0, 300.0]));
        put(&mut doc, line, LINE_PATH, Value::Enum(3));
        put(&mut doc, line, SLACK, Value::F64(30.0));
        let l = shown(&doc, line);
        assert!(l[3] > 300.0 + 100.0, "a hanging line sags below its ends: {l:?}");
        assert!(l[1] > 298.0, "and never rises above them");
    }

    #[test]
    fn a_hanging_rope_keeps_its_length_and_sags_down() {
        let (p0, p1) = (glam::vec2(0.0, 0.0), glam::vec2(400.0, 0.0));
        let rope = hang(p0, p1, 480.0);
        let length: f32 = rope.windows(2).map(|w| (w[1][0] - w[0][0]).length()).sum();
        assert!((length - 480.0).abs() < 4.0, "the polyline is as long as the rope: {length}");
        assert!(rope[HANG_POINTS / 2][0].y > 100.0, "the middle hangs down (screen y grows downward): {:?}", rope[HANG_POINTS / 2][0]);
        assert_eq!((rope[0][0], rope[HANG_POINTS - 1][0]), (p0, p1), "ends stay on the boxes");
        let tilted = hang(glam::vec2(0.0, 0.0), glam::vec2(300.0, -200.0), 500.0);
        let length: f32 = tilted.windows(2).map(|w| (w[1][0] - w[0][0]).length()).sum();
        assert!((length - 500.0).abs() < 4.0, "uneven ends keep the length too: {length}");
        let taut = hang(p0, p1, 400.0);
        assert!(taut.iter().all(|p| p[0].y.abs() < 1e-3), "no slack: a straight line");
    }
}
