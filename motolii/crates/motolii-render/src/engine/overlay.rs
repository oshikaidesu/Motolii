//! Track Overlay を描く: このコマの塊から箱・印の形を組み、comp と同じ大きさの画布に描く(描くのは形の道 = fork の re_renderer)。
//! 欄は docs/reviews/2026-09-14-tracery2-port-spec.md の Box / Marker。

use crate::doc::core::CompSpec;
use crate::doc::store::analysis::BlobMark;
use crate::extensions::overlay::{color_of, number_of, switch_of};
use crate::doc::store::{LayerId, Value};
use crate::doc::vector::{Brush, Contour, Dash, Fill, LineCap, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeNode, StarType, Stroke};
use crate::render::compositor::LayerContent;
use crate::render::engine::{Engine, EngineError};

type Params = [(String, Value)];

fn rgb(c: [f64; 4]) -> Rgb {
    Rgb { r: c[0], g: c[1], b: c[2] }
}

fn at(center: [f32; 2], rotation: f64, child: Shape) -> ShapeNode {
    ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform { position: Point { x: center[0] as f64, y: center[1] as f64 }, rotation, ..RepeaterTransform::IDENTITY },
        children: vec![ShapeNode::Leaf(child)],
    })
}

/// 箱 1 つ。Square / Circle は長辺に揃え、Custom Size は半径で決める。Gap は 4 辺(4 弧)の真ん中を残し角を抜く。
fn box_shape(params: &Params, mark: &BlobMark) -> Option<ShapeNode> {
    let kind = number_of(params, "box_shape").round() as i64;
    let mut size = [mark.size[0] as f64, mark.size[1] as f64];
    if matches!(kind, 1 | 3) { let side = size[0].max(size[1]); size = [side, side]; }
    if switch_of(params, "custom_size") { let d = number_of(params, "custom_radius").max(0.0) * 2.0; size = [d, d]; }
    let source = if matches!(kind, 2 | 3) { PathSource::Ellipse { size: Point { x: size[0], y: size[1] } } } else { PathSource::Rectangle { size: Point { x: size[0], y: size[1] } } };
    let stroke = switch_of(params, "box_stroke").then(|| {
        let c = color_of(params, "box_stroke_color");
        let perimeter = if matches!(kind, 2 | 3) { std::f64::consts::PI * (size[0] + size[1]) * 0.5 } else { 2.0 * (size[0] + size[1]) };
        let gap = switch_of(params, "box_gap").then(|| number_of(params, "box_gap_size").clamp(0.0, 0.95));
        Stroke {
            brush: Brush::Solid(rgb(c)),
            width: number_of(params, "box_stroke_width").max(0.0),
            opacity: number_of(params, "box_stroke_opacity").clamp(0.0, 1.0) * c[3],
            dash: gap.map(|g| { let quarter = perimeter / 4.0; Dash { pattern: vec![quarter * (1.0 - g), quarter * g], offset: quarter * g * 0.5 } }),
            ..Default::default()
        }
    });
    let fill = switch_of(params, "box_fill").then(|| {
        let c = color_of(params, "box_fill_color");
        Fill { brush: Brush::Solid(rgb(c)), opacity: number_of(params, "box_fill_opacity").clamp(0.0, 1.0) * c[3], ..Default::default() }
    });
    (stroke.is_some() || fill.is_some()).then(|| at(mark.center, 0.0, Shape { source, ops: Vec::new(), fill, stroke }))
}

/// 印 1 つ。Plus / Cross は 2 本の線、Polygon は正多角形(Polygon Fill なら塗り)。
fn marker_shape(params: &Params, mark: &BlobMark) -> ShapeNode {
    let c = color_of(params, "marker_color");
    let opacity = number_of(params, "marker_opacity").clamp(0.0, 1.0) * c[3];
    let size = number_of(params, "marker_size").max(0.0);
    let thickness = number_of(params, "marker_thickness").max(0.0);
    let rotation = number_of(params, "marker_rotation");
    let stroke = Stroke { brush: Brush::Solid(rgb(c)), width: thickness, opacity, cap: LineCap::Butt, ..Default::default() };
    let fill = Fill { brush: Brush::Solid(rgb(c)), opacity, ..Default::default() };
    let h = size * 0.5;
    let bars = |extra: f64| {
        let path = vec![Contour::open([Point { x: -h, y: 0.0 }, Point { x: h, y: 0.0 }]), Contour::open([Point { x: 0.0, y: -h }, Point { x: 0.0, y: h }])];
        at(mark.center, rotation + extra, Shape { source: PathSource::Bezier(path), ops: Vec::new(), fill: None, stroke: Some(stroke.clone()) })
    };
    match number_of(params, "marker_type").round() as i64 {
        0 => at(mark.center, 0.0, Shape { source: PathSource::Ellipse { size: Point { x: size, y: size } }, ops: Vec::new(), fill: Some(fill), stroke: None }),
        1 => bars(0.0),
        2 => bars(45.0),
        _ => {
            let filled = switch_of(params, "polygon_fill");
            let polygon = PathSource::PolyStar { points: number_of(params, "polygon_sides").round().clamp(3.0, 12.0), outer_radius: h, inner_radius: h, star_type: StarType::Polygon };
            at(mark.center, rotation, Shape { source: polygon, ops: Vec::new(), fill: filled.then_some(fill), stroke: (!filled).then_some(stroke) })
        }
    }
}

/// Grid(Tracery 2 の Grid 節): Edge は塊の箱の辺から立てた線(`overlay::edge_lines`、近い辺は 1 本に)、Cartesian は等間隔。
/// 線は画面の端から端まで。濃さは線ごと。
fn grid_shapes(params: &Params, marks: &[BlobMark], comp: [f64; 2]) -> Vec<ShapeNode> {
    let c = color_of(params, "grid_color");
    let base = number_of(params, "grid_opacity").clamp(0.0, 1.0) * c[3];
    let width = number_of(params, "grid_thickness").max(0.0);
    let line = |from: Point, to: Point, opacity: f64| ShapeNode::Leaf(Shape {
        source: PathSource::Bezier(vec![Contour::open([from, to])]),
        ops: Vec::new(),
        fill: None,
        stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width, opacity: base * opacity, cap: LineCap::Butt, ..Default::default() }),
    });
    let mut out = Vec::new();
    if number_of(params, "grid_mode").round() as i64 == 1 {
        let (columns, rows) = (number_of(params, "grid_columns").round().max(1.0), number_of(params, "grid_rows").round().max(1.0));
        for i in 0..=columns as i64 {
            let x = comp[0] * i as f64 / columns;
            out.push(line(Point { x, y: 0.0 }, Point { x, y: comp[1] }, 1.0));
        }
        for i in 0..=rows as i64 {
            let y = comp[1] * i as f64 / rows;
            out.push(line(Point { x: 0.0, y }, Point { x: comp[0], y }, 1.0));
        }
        return out;
    }
    let merge = number_of(params, "grid_merge").max(0.0) as f32;
    let xs: Vec<f32> = marks.iter().flat_map(|m| [m.center[0] - m.size[0] * 0.5, m.center[0] + m.size[0] * 0.5]).collect();
    let ys: Vec<f32> = marks.iter().flat_map(|m| [m.center[1] - m.size[1] * 0.5, m.center[1] + m.size[1] * 0.5]).collect();
    for (x, opacity) in crate::extensions::overlay::edge_lines(&xs, merge) {
        out.push(line(Point { x: f64::from(x), y: 0.0 }, Point { x: f64::from(x), y: comp[1] }, f64::from(opacity)));
    }
    for (y, opacity) in crate::extensions::overlay::edge_lines(&ys, merge) {
        out.push(line(Point { x: 0.0, y: f64::from(y) }, Point { x: comp[0], y: f64::from(y) }, f64::from(opacity)));
    }
    out
}

/// 3D の見つけた格子(提案 2026-09-15、利用者「グリッドはオブジェクト関係に付与される」「3d のグリッド表現」):
/// 物の箱の辺から x・y の面を、物の奥行きから z の面を立て(どれも `overlay::edge_lines` で近い物を 1 つに)、面どうしの交線を空間の線にする。
/// 奥行きの面の上に x・y の線(画面の幅・高さいっぱい)、x と y の交点に奥行きの柱(一番手前から一番奥まで)。
/// 線は寄り合っても消さず、重みで薄める(寄り合う途中で線が出たり消えたりしない)。奥行きが 1 つなら柱は無く、平面の格子と同じ。
pub(crate) fn lattice(params: &Params, marks: &[BlobMark], depths: &[f32], comp: [f32; 2]) -> crate::render::compositor::CloudLinks {
    use crate::extensions::overlay::edge_lines;
    let c = color_of(params, "grid_color");
    let base = (number_of(params, "grid_opacity").clamp(0.0, 1.0) * c[3]) as f32;
    let merge = number_of(params, "grid_merge").max(0.0) as f32;
    let xs = edge_lines(&marks.iter().flat_map(|m| [m.center[0] - m.size[0] * 0.5, m.center[0] + m.size[0] * 0.5]).collect::<Vec<_>>(), merge);
    let ys = edge_lines(&marks.iter().flat_map(|m| [m.center[1] - m.size[1] * 0.5, m.center[1] + m.size[1] * 0.5]).collect::<Vec<_>>(), merge);
    let zs = edge_lines(depths, merge);
    const LEVELS: usize = 16;
    let mut levels: Vec<Vec<([f32; 3], [f32; 3])>> = vec![Vec::new(); LEVELS];
    let mut put = |weight: f32, a: [f32; 3], b: [f32; 3]| {
        let level = ((weight.clamp(0.0, 1.0) * LEVELS as f32).ceil() as usize).clamp(1, LEVELS) - 1;
        levels[level].push((a, b));
    };
    for &(z, wz) in &zs {
        for &(x, wx) in &xs { put(wx * wz, [x, 0.0, z], [x, comp[1], z]); }
        for &(y, wy) in &ys { put(wy * wz, [0.0, y, z], [comp[0], y, z]); }
    }
    let (near, far) = zs.iter().fold((f32::MAX, f32::MIN), |(lo, hi), (z, _)| (lo.min(*z), hi.max(*z)));
    if far - near > 1e-3 {
        for &(x, wx) in &xs {
            for &(y, wy) in &ys { put(wx * wy, [x, y, near], [x, y, far]); }
        }
    }
    let rgb = [c[0], c[1], c[2]].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    crate::render::compositor::CloudLinks {
        levels: levels.into_iter().enumerate().filter(|(_, s)| !s.is_empty()).map(|(k, segments)| {
            let alpha = (base * (k as f32 + 1.0) / LEVELS as f32 * 255.0).round() as u8;
            ([rgb[0], rgb[1], rgb[2], alpha], segments)
        }).collect(),
        width: number_of(params, "grid_thickness").max(0.0) as f32,
    }
}

/// 押された跡(Push、Layers の時): いたかった箱(今の箱を押された分だけ戻した所)と、そこから今の中心への矢印。
/// 押されていなければ箱は今の箱に重なり、矢印は長さと一緒に 0 へ縮む(出たり消えたりしない)。
fn push_shapes(params: &Params, mark: &BlobMark, push: [f32; 2]) -> Vec<ShapeNode> {
    let c = color_of(params, "push_color");
    let width = number_of(params, "push_thickness").max(0.0);
    let stroke = Stroke {
        brush: Brush::Solid(rgb(c)),
        width,
        opacity: number_of(params, "push_opacity").clamp(0.0, 1.0) * c[3],
        cap: LineCap::Butt,
        dash: switch_of(params, "push_dash").then(|| Dash { pattern: vec![width * 3.0, width * 2.5], offset: 0.0 }),
        ..Default::default()
    };
    let shift = glam::Vec2::from(push);
    let now = glam::Vec2::from(mark.center);
    let was = now - shift;
    let p = |v: glam::Vec2| Point { x: f64::from(v.x), y: f64::from(v.y) };
    let half = glam::Vec2::from(mark.size) * 0.5;
    let mut path = vec![Contour::closed([p(was - half), p(was + glam::vec2(half.x, -half.y)), p(was + half), p(was + glam::vec2(-half.x, half.y))])];
    let length = shift.length();
    if length > 1e-3 {
        let dir = shift / length;
        let side = glam::vec2(-dir.y, dir.x);
        let head = (width as f32 * 6.0).min(length * 0.5);
        path.push(Contour::open([p(was), p(now)]));
        path.push(Contour::open([p(now - dir * head + side * head * 0.6), p(now), p(now - dir * head - side * head * 0.6)]));
    }
    vec![ShapeNode::Leaf(Shape { source: PathSource::Bezier(path), ops: Vec::new(), fill: None, stroke: Some(stroke) })]
}

/// 札の書体: 等幅の OS の書体(無ければ OS の既定へ落ちる)。
fn label_family() -> &'static str {
    static FAMILY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FAMILY.get_or_init(|| {
        let families = crate::picture::shaping::font_families();
        ["SF Mono", "Menlo", "Consolas", "DejaVu Sans Mono", "Noto Sans Mono"].into_iter().find(|f| families.iter().any(|g| g == f)).unwrap_or("").to_owned()
    })
}

/// 札(Tracery 2 の Labels): 箱の左下から Offset だけずらして、Display Mode の値を文字の輪郭で。
fn label_shape(params: &Params, mark: &BlobMark, push: [f32; 2]) -> Option<ShapeNode> {
    let content = match number_of(params, "label_mode").round() as i64 {
        0 => format!("{}, {}", mark.center[0].round() as i64, mark.center[1].round() as i64),
        1 => format!("{} \u{d7} {}", mark.size[0].round() as i64, mark.size[1].round() as i64),
        2 => format!("{} px", glam::Vec2::from(push).length().round() as i64),
        // Speed: 物理の速さ(px/秒)。止まっている物には札を出さない。
        _ => {
            let speed = glam::Vec2::from(push).length();
            if speed < 12.0 { return None; }
            format!("{} px/s", speed.round() as i64)
        }
    };
    let font = crate::doc::vector::text::GlyphFont { path: String::new(), family: label_family().to_owned() };
    let shaped = crate::picture::shaping::shape_text(&content, &font, &crate::doc::vector::text::TextLayout::new(number_of(params, "font_size").max(1.0) as f32)).ok()?;
    let c = color_of(params, "label_color");
    let at = [mark.center[0] - mark.size[0] * 0.5 + number_of(params, "label_offset_x") as f32, mark.center[1] + mark.size[1] * 0.5 + number_of(params, "label_offset_y") as f32];
    Some(self::at(at, 0.0, Shape {
        source: PathSource::Bezier(shaped.contours),
        ops: Vec::new(),
        fill: Some(Fill { brush: Brush::Solid(rgb(c)), opacity: number_of(params, "label_opacity").clamp(0.0, 1.0) * c[3], ..Default::default() }),
        stroke: None,
    }))
}

/// 触れ合っている組の線(物理の可視)。関係が見えるように見える事が芯(2026-09-15 の裁定)。
fn link_shapes(params: &Params, links: &[([f32; 2], [f32; 2])]) -> Vec<ShapeNode> {
    let c = color_of(params, "links_color");
    let width = number_of(params, "links_width").max(0.0);
    let point = |p: [f32; 2]| Point { x: p[0] as f64, y: p[1] as f64 };
    links.iter().map(|(a, b)| ShapeNode::Leaf(Shape {
        source: PathSource::Bezier(vec![Contour::open([point(*a), point(*b)])]),
        ops: Vec::new(),
        fill: None,
        stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width, opacity: c[3], cap: LineCap::Round, ..Default::default() }),
    })).collect()
}

/// 触れ合っている点と、その法線の短い髭。どこで当たっているかが見えるのが芯。
fn contact_shapes(params: &Params, contacts: &[([f32; 2], [f32; 2])]) -> Vec<ShapeNode> {
    let c = color_of(params, "links_color");
    let point = |p: [f32; 2]| Point { x: p[0] as f64, y: p[1] as f64 };
    contacts.iter().flat_map(|(at, normal)| {
        let tick = [at[0] + normal[0] * 9.0, at[1] + normal[1] * 9.0];
        let line = |a: [f32; 2], b: [f32; 2], width: f64, opacity: f64| ShapeNode::Leaf(Shape {
            source: PathSource::Bezier(vec![Contour::open([point(a), point(b)])]),
            ops: Vec::new(),
            fill: None,
            stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width, opacity: c[3] * opacity, cap: LineCap::Butt, ..Default::default() }),
        });
        let dot = self::at(*at, 0.0, Shape {
            source: PathSource::Ellipse { size: Point { x: 3.0, y: 3.0 } },
            ops: Vec::new(),
            fill: Some(Fill { brush: Brush::Solid(rgb(c)), opacity: c[3], ..Default::default() }),
            stroke: None,
        });
        [dot, line(*at, tick, 1.0, 0.6)]
    }).collect()
}

/// 物の輪郭(当たりに使っている形そのもの)。
fn hull_shapes(params: &Params, hulls: &[Vec<[f32; 2]>]) -> Vec<ShapeNode> {
    let c = color_of(params, "box_stroke_color");
    hulls.iter().filter(|h| h.len() >= 3).map(|h| ShapeNode::Leaf(Shape {
        source: PathSource::Bezier(vec![Contour::closed(h.iter().map(|p| Point { x: p[0] as f64, y: p[1] as f64 }))]),
        ops: Vec::new(),
        fill: None,
        stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width: 1.0, opacity: c[3] * 0.75, cap: LineCap::Butt, ..Default::default() }),
    })).collect()
}

/// 今の速さ(向きと大きさ)。止まっている物には出ない。
fn velocity_shapes(params: &Params, velocities: &[([f32; 2], [f32; 2])]) -> Vec<ShapeNode> {
    let c = color_of(params, "velocity_color");
    let point = |p: [f32; 2]| Point { x: p[0] as f64, y: p[1] as f64 };
    velocities.iter().filter_map(|(at, v)| {
        let speed = (v[0] * v[0] + v[1] * v[1]).sqrt();
        if speed < 12.0 {
            return None;
        }
        // 長さは速さに比例させつつ頭打ちに(画面を糸で埋めない)。
        let scale = (0.09_f32).min(110.0 / speed);
        let tip = [at[0] + v[0] * scale, at[1] + v[1] * scale];
        Some(ShapeNode::Leaf(Shape {
            source: PathSource::Bezier(vec![Contour::open([point(*at), point(tip)])]),
            ops: Vec::new(),
            fill: None,
            stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width: 1.0, opacity: c[3] * (0.4 + (speed / 900.0).min(0.6) as f64), cap: LineCap::Butt, ..Default::default() }),
        }))
    }).collect()
}

/// 場の元: 届く輪(点線)と、一様な向きの矢。
fn well_shapes(params: &Params, wells: &[([f32; 2], f32, [f32; 2])]) -> Vec<ShapeNode> {
    let c = color_of(params, "well_color");
    let point = |p: [f32; 2]| Point { x: p[0] as f64, y: p[1] as f64 };
    let mut out = Vec::new();
    for (spot, reach, gravity) in wells {
        if *reach > 0.0 {
            out.push(at(*spot, 0.0, Shape {
                source: PathSource::Ellipse { size: Point { x: (*reach * 2.0) as f64, y: (*reach * 2.0) as f64 } },
                ops: Vec::new(),
                fill: None,
                stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width: 1.5, opacity: c[3] * 0.7, dash: Some(Dash { pattern: vec![10.0, 10.0], offset: 0.0 }), cap: LineCap::Butt, ..Default::default() }),
            }));
        }
        let len = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
        if len > 1e-3 {
            let d = [gravity[0] / len, gravity[1] / len];
            let tip = [spot[0] + d[0] * 70.0, spot[1] + d[1] * 70.0];
            let side = [-d[1] * 4.0, d[0] * 4.0];
            let back = [tip[0] - d[0] * 9.0, tip[1] - d[1] * 9.0];
            for (a, b) in [(*spot, tip), ([back[0] + side[0], back[1] + side[1]], tip), ([back[0] - side[0], back[1] - side[1]], tip)] {
                out.push(ShapeNode::Leaf(Shape {
                    source: PathSource::Bezier(vec![Contour::open([point(a), point(b)])]),
                    ops: Vec::new(),
                    fill: None,
                    stroke: Some(Stroke { brush: Brush::Solid(rgb(c)), width: 1.0, opacity: c[3], cap: LineCap::Butt, ..Default::default() }),
                }));
            }
        }
    }
    out
}

/// このコマの塊から、Grid → Box → Push → Marker → Labels の順に形を組む(comp の座標)。`pushes` は Layers の時だけ(塊と同じ順)。
pub(crate) fn overlay_shapes(params: &Params, marks: &[BlobMark], pushes: &[[f32; 2]], comp: [f64; 2]) -> Vec<ShapeNode> {
    overlay_shapes_with(params, marks, pushes, &PhysicsTrace::default(), comp)
}

/// 物理の可視が描く物(解き手から取る)。
#[derive(Default)]
pub(crate) struct PhysicsTrace<'a> {
    pub links: &'a [([f32; 2], [f32; 2])],
    pub contacts: &'a [([f32; 2], [f32; 2])],
    pub velocities: &'a [([f32; 2], [f32; 2])],
    pub wells: &'a [([f32; 2], f32, [f32; 2])],
    pub hulls: &'a [Vec<[f32; 2]>],
}

/// 物理の可視は、同じ形の上に触れ合いの線と場の輪を足す。
pub(crate) fn overlay_shapes_with(params: &Params, marks: &[BlobMark], pushes: &[[f32; 2]], physics: &PhysicsTrace<'_>, comp: [f64; 2]) -> Vec<ShapeNode> {
    let mut out = Vec::new();
    if switch_of(params, "well") {
        out.extend(well_shapes(params, physics.wells));
    }
    if switch_of(params, "hull") {
        out.extend(hull_shapes(params, physics.hulls));
    }
    if switch_of(params, "velocity") {
        out.extend(velocity_shapes(params, physics.velocities));
    }
    if switch_of(params, "links") {
        out.extend(link_shapes(params, physics.links));
    }
    if switch_of(params, "contacts") {
        out.extend(contact_shapes(params, physics.contacts));
    }
    if switch_of(params, "grid") {
        out.extend(grid_shapes(params, marks, comp));
    }
    if switch_of(params, "box") {
        out.extend(marks.iter().filter_map(|m| box_shape(params, m)));
    }
    if switch_of(params, "push") {
        out.extend(marks.iter().zip(pushes).flat_map(|(m, p)| push_shapes(params, m, *p)));
    }
    if switch_of(params, "marker") {
        out.extend(marks.iter().map(|m| marker_shape(params, m)));
    }
    if switch_of(params, "label") {
        out.extend(marks.iter().enumerate().filter_map(|(k, m)| label_shape(params, m, pushes.get(k).copied().unwrap_or([0.0; 2]))));
    }
    out
}

impl Engine {
    /// Track Overlay を持つ層の中身(comp 大、左上が層の位置)。Show Mask なら解析の二値。
    pub(super) fn overlay_content(&mut self, layer: LayerId, comp: CompSpec) -> Result<Option<(LayerContent, [f32; 2])>, EngineError> {
        // 物理の可視は、絵を組む途中で解き手を進めてから読む(そうしないと 1 コマ遅れる・空になる)。
        if self.overlay_frames.get(&layer).is_some_and(|f| f.physics) {
            self.solve_physics_now();
            let (marks, links, wells) = (self.physics_marks(), self.physics_links(), self.physics_wells());
            let (contacts, velocities, hulls) = (self.physics_contacts_at(), self.physics_velocities(), self.physics_hulls());
            if let Some(frame) = self.overlay_frames.get_mut(&layer) {
                // 札の Speed は速さを読む(pushes の枠を借りる: 印と同じ順)。
                frame.pushes = marks.iter().map(|m| velocities.iter().min_by(|a, b| {
                    let da = (a.0[0] - m.center[0]).powi(2) + (a.0[1] - m.center[1]).powi(2);
                    let db = (b.0[0] - m.center[0]).powi(2) + (b.0[1] - m.center[1]).powi(2);
                    da.total_cmp(&db)
                }).map_or([0.0, 0.0], |v| v.1)).collect();
                frame.marks = marks;
                frame.links = links;
                frame.wells = wells;
                frame.contacts = contacts;
                frame.velocities = velocities;
                frame.hulls = hulls;
            }
        }
        let Some(frame) = self.overlay_frames.get(&layer) else { return Ok(None) };
        let natural = [comp.width as f32, comp.height as f32];
        if switch_of(&frame.params, "show_mask") {
            let Some((bits, w, h)) = frame.mask.clone() else { return Ok(None) };
            let bytes: Vec<u8> = bits.iter().flat_map(|b| {
                let v = half::f16::from_f32(if *b > 0 { 1.0 } else { 0.0 }).to_le_bytes();
                let one = half::f16::from_f32(1.0).to_le_bytes();
                [v[0], v[1], v[0], v[1], v[0], v[1], one[0], one[1]]
            }).collect();
            let texture = self.compositor.upload_rgba16f("motolii-overlay-mask", bytes, w, h)?;
            return Ok(Some((LayerContent::LinearTexture(texture), natural)));
        }
        // 3D の格子は焼かず、空間の線のまま view へ(粒子の Plexus と同じ道)。
        if let Some(depths) = frame.depths.as_ref().filter(|d| !d.is_empty() && switch_of(&frame.params, "grid") && number_of(&frame.params, "grid_mode").round() as i64 == 0) {
            let links = lattice(&frame.params, &frame.marks, depths, natural);
            return Ok(Some((LayerContent::Cloud {
                positions: std::sync::Arc::new(Vec::new()),
                colors: std::sync::Arc::new(Vec::new()),
                bounds: crate::render::media::SpatialBounds { min: [0.0, 0.0, 0.0], max: [natural[0], natural[1], 0.0] },
                point_size: 1.0,
                sizes: None,
                sprites: true,
                links: Some(std::sync::Arc::new(links)),
            }, natural)));
        }
        let trace = PhysicsTrace { links: &frame.links, contacts: &frame.contacts, velocities: &frame.velocities, wells: &frame.wells, hulls: &frame.hulls };
        let shapes = overlay_shapes_with(&frame.params, &frame.marks, &frame.pushes, &trace, [f64::from(comp.width), f64::from(comp.height)]);
        if shapes.is_empty() {
            return Ok(None);
        }
        let canvas = crate::picture::shapes_ops::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        let texture = self.compositor.render_paths("track-overlay", &shapes, &canvas, 1.0, super::texture::raster_pixel_budget(comp))?;
        Ok(texture.map(|t| (LayerContent::Texture(t), natural)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, Value)]) -> Vec<(String, Value)> {
        pairs.iter().map(|(n, v)| ((*n).to_owned(), v.clone())).collect()
    }

    #[test]
    fn boxes_follow_the_shape_choice_and_markers_are_optional() {
        let marks = [BlobMark { id: 0, center: [100.0, 50.0], size: [40.0, 20.0], age: 0 }];
        let shapes = overlay_shapes(&params(&[]), &marks, &[], [400.0, 300.0]);
        assert_eq!(shapes.len(), 1, "既定は箱だけ");
        let ShapeNode::Group(g) = &shapes[0] else { panic!() };
        assert_eq!((g.transform.position.x, g.transform.position.y), (100.0, 50.0));
        let ShapeNode::Leaf(leaf) = &g.children[0] else { panic!() };
        assert_eq!(leaf.source, PathSource::Rectangle { size: Point { x: 40.0, y: 20.0 } });
        let circle = overlay_shapes(&params(&[("box_shape", Value::F64(3.0)), ("marker", Value::F64(1.0))]), &marks, &[], [400.0, 300.0]);
        assert_eq!(circle.len(), 2, "箱 + 印");
        let ShapeNode::Group(g) = &circle[0] else { panic!() };
        let ShapeNode::Leaf(leaf) = &g.children[0] else { panic!() };
        assert_eq!(leaf.source, PathSource::Ellipse { size: Point { x: 40.0, y: 40.0 } }, "Circle は長辺の円");
        let gapped = overlay_shapes(&params(&[("box_gap", Value::F64(1.0))]), &marks, &[], [400.0, 300.0]);
        let ShapeNode::Group(g) = &gapped[0] else { panic!() };
        let ShapeNode::Leaf(leaf) = &g.children[0] else { panic!() };
        assert!(leaf.stroke.as_ref().and_then(|s| s.dash.as_ref()).is_some_and(|d| d.pattern.len() == 2), "Gap は破線");
        assert!(overlay_shapes(&params(&[("box", Value::F64(0.0))]), &marks, &[], [400.0, 300.0]).is_empty());
    }

    /// 奥行きの違う物を読むと、奥行きの面ごとの線と、面を貫く柱が立つ。奥行きが 1 つなら柱は無い。
    #[test]
    fn a_lattice_stands_sheets_at_the_depths_and_pillars_between_them() {
        let marks = [
            BlobMark { id: 0, center: [100.0, 50.0], size: [40.0, 20.0], age: 0 },
            BlobMark { id: 1, center: [300.0, 150.0], size: [40.0, 20.0], age: 0 },
        ];
        let p = params(&[("grid", Value::F64(1.0)), ("grid_merge", Value::F64(4.0))]);
        let segments = |depths: &[f32]| -> Vec<([f32; 3], [f32; 3])> { lattice(&p, &marks, depths, [400.0, 300.0]).levels.into_iter().flat_map(|(_, s)| s).collect() };
        let flat = segments(&[0.0, 0.0]);
        assert!(flat.iter().all(|(a, b)| a[2] == b[2]), "one depth: no pillars");
        assert_eq!(flat.len(), 2 * (4 + 4), "each depth copy carries 4 vertical and 4 horizontal lines");
        let deep = segments(&[0.0, 500.0]);
        let pillars: Vec<_> = deep.iter().filter(|(a, b)| a[2] != b[2]).collect();
        assert_eq!(pillars.len(), 4 * 4, "a pillar at every crossing of an x line and a y line");
        assert!(pillars.iter().all(|(a, b)| a[2] == 0.0 && b[2] == 500.0), "from the nearest sheet to the farthest");
        assert!(deep.iter().any(|(a, b)| a[2] == 500.0 && b[2] == 500.0 && a[0] == b[0] && b[1] == 300.0), "the far sheet has its own lines, screen-high");
    }

    /// Push Trace の既定: 押された物に、いたかった箱と矢印と「N px」の札。押されていない物には跡の箱だけ(矢印は無い)。
    #[test]
    fn a_push_trace_draws_where_each_thing_wanted_to_be_and_how_far_it_went() {
        let marks = [
            BlobMark { id: 0, center: [100.0, 50.0], size: [40.0, 20.0], age: 0 },
            BlobMark { id: 1, center: [300.0, 150.0], size: [40.0, 20.0], age: 0 },
        ];
        let p = crate::extensions::overlay::with_defaults(crate::extensions::overlay::PUSH_TRACE, &[("box", Value::F64(0.0))].map(|(n, v)| (n.to_owned(), v)));
        let shapes = overlay_shapes(&p, &marks, &[[30.0, -40.0], [0.0, 0.0]], [400.0, 300.0]);
        let paths: Vec<&Vec<Contour>> = shapes.iter().filter_map(|n| match n { ShapeNode::Leaf(Shape { source: PathSource::Bezier(path), stroke: Some(_), .. }) => Some(path), _ => None }).collect();
        assert_eq!(paths.len(), 2, "one trace per thing");
        assert_eq!(paths[0].len(), 3, "pushed: the wanted box, the shaft and the head");
        assert_eq!((paths[0][0].vertices[0].point.x, paths[0][0].vertices[0].point.y), (50.0, 80.0), "the wanted box is the box moved back by the push");
        assert_eq!(paths[1].len(), 1, "not pushed: only the box, lying on the thing");
        let labels = shapes.iter().filter(|n| matches!(n, ShapeNode::Group(_))).count();
        assert_eq!(labels, 2, "a label for each thing (50 px, 0 px)");
    }

    #[test]
    fn an_edge_grid_raises_one_line_per_edge_across_the_screen() {
        let marks = [
            BlobMark { id: 0, center: [100.0, 50.0], size: [40.0, 20.0], age: 0 },
            BlobMark { id: 1, center: [102.0, 150.0], size: [40.0, 20.0], age: 0 },
        ];
        let grid = overlay_shapes(&params(&[("box", Value::F64(0.0)), ("grid", Value::F64(1.0)), ("grid_merge", Value::F64(10.0))]), &marks, &[], [400.0, 300.0]);
        assert_eq!(grid.len(), 8, "left, right, top, bottom of each box");
        let xs: Vec<f64> = grid.iter().filter_map(|n| match n {
            ShapeNode::Leaf(Shape { source: PathSource::Bezier(path), .. }) if path[0].vertices[0].point.x == path[0].vertices[1].point.x => Some(path[0].vertices[0].point.x),
            _ => None,
        }).collect();
        assert!((xs[0] - xs[2]).abs() < 0.5, "the two left edges 2 px apart share one vertical line: {xs:?}");
        let ShapeNode::Leaf(first) = &grid[0] else { panic!() };
        let PathSource::Bezier(path) = &first.source else { panic!() };
        assert_eq!((path[0].vertices[0].point.y, path[0].vertices[1].point.y), (0.0, 300.0), "a line runs edge to edge of the screen");
    }
}
