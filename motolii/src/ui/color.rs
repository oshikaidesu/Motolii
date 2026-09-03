//! 色の輪 — Browser の Colors に常設。焦点の色(Inspector の COLOR 行)に付いて行き、
//! 書き戻しは property でなく shape / text の data(2026-09-03 利用者裁定: 机には出さない)。

use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::doc::store::{Document, Intent, LayerId, ShapeNode, StoreError};
use crate::doc::vector::{Brush, Fill, Gradient, GradientStop, GradientType, PathSource, Point, Rgb};
use crate::ui::semantic_menu::{Field, SemanticButton};
use crate::ui::session::{ColorSlot, FieldAt, Focus, OpenField, Session};

/// 輪が今指す色。焦点の色、無ければ選んでいる層の最初の色。
pub(super) fn wheel_slot(session: &Session) -> Option<ColorSlot> {
    let slot = match session.live_focus() {
        Some(Focus::Color(slot)) => Some(slot),
        _ => session.selection.get().and_then(|layer| {
            let d = session.doc.lock().unwrap();
            let t = crate::doc::store::RationalTime::ZERO;
            crate::ui::fixture::inspector_data_from_doc(&d.view(), layer, t)
                .colors
                .into_iter()
                .next()
                .map(|row| row.slot)
        }),
    };
    // 読めない slot(style が消えた文字層)は輪の相手にしない。黒を見せて書き込みを捨てる嘘を避ける。
    // 縁取り(stroke)はまだ無くても輪を出す — 無い物を足す口が要る。読めない shape の fill だけ捨てる。
    slot.filter(|s| matches!(s, ColorSlot::TextStroke { .. }) || read_color(&session.doc, s).is_some())
}

/// 不透明度だけを書く(文字の fill / stroke)。shape の塗りは α を持たない。
pub(super) fn write_alpha(doc: &Arc<Mutex<Document>>, slot: &ColorSlot, alpha: f64) -> Result<(), StoreError> {
    let mut d = doc.lock().unwrap();
    let (layer, style) = match slot {
        ColorSlot::TextFill { layer, style } | ColorSlot::TextStroke { layer, style } => (*layer, *style),
        ColorSlot::ShapeFill { .. } | ColorSlot::ShapeGradientStop { .. } => return Ok(()),
    };
    let Some(mut text) = d.view().text_document(layer)? else { return Ok(()) };
    let Some(found) = text.styles.iter_mut().find(|s| s.id == style) else { return Ok(()) };
    let a = alpha.clamp(0.0, 1.0);
    match slot {
        ColorSlot::TextFill { .. } => found.fill[3] = a,
        _ => {
            if let Some(c) = found.stroke_color.as_mut() {
                c[3] = a;
            }
        }
    }
    d.apply(Intent::SetTextDocument { layer, document: text }).map(|_| ())
}

/// 色相の輪の直径(scale 100% の px)。面はその 0.6 倍。
const RING: f64 = 128.0;
const SQUARE: f64 = RING * 0.6;

/// HSV → RGB。h は度、s・v は 0..1。
pub(super) fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [f64; 3] {
    let h = h.rem_euclid(360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [r + m, g + m, b + m]
}

pub(super) fn rgb_to_hsv([r, g, b]: [f64; 3]) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= f64::EPSILON {
        0.0
    } else if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let s = if max <= f64::EPSILON { 0.0 } else { d / max };
    (h, s, max)
}

fn hex_of([r, g, b]: [f64; 3]) -> String {
    let c = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b))
}

fn leaf_mut<'a>(nodes: &'a mut [ShapeNode], path: &[usize]) -> Option<&'a mut crate::doc::vector::Shape> {
    let (first, rest) = path.split_first()?;
    match nodes.get_mut(*first)? {
        ShapeNode::Leaf(shape) if rest.is_empty() => Some(shape),
        ShapeNode::Group(group) => leaf_mut(&mut group.children, rest),
        ShapeNode::Leaf(_) => None,
    }
}

fn shape_location(slot: &ColorSlot) -> Option<(LayerId, &[usize])> {
    match slot {
        ColorSlot::ShapeFill { layer, path } | ColorSlot::ShapeGradientStop { layer, path, .. } => {
            Some((*layer, path))
        }
        _ => None,
    }
}

/// Shapeのlocal boundsを横切る既定軸。Brushの座標もshape-localなので、
/// layer/親groupのtransformを混ぜない。
fn gradient_axis(source: &PathSource) -> (Point, Point) {
    let bounds = match source {
        PathSource::Rectangle { size } | PathSource::Ellipse { size } => {
            Some([-size.x * 0.5, -size.y * 0.5, size.x * 0.5, size.y * 0.5])
        }
        PathSource::PolyStar { outer_radius, .. } => {
            let r = outer_radius.abs();
            Some([-r, -r, r, r])
        }
        PathSource::Bezier(path) => {
            let mut bounds: Option<[f64; 4]> = None;
            for vertex in path.iter().flat_map(|contour| &contour.vertices) {
                // Bezier曲線は端点とcontrol pointの凸包内にある。既定軸には十分で、
                // 曲線を小さく見積もってgradientが途中で終わることもない。
                for point in [
                    vertex.point,
                    Point { x: vertex.point.x + vertex.in_tangent.x, y: vertex.point.y + vertex.in_tangent.y },
                    Point { x: vertex.point.x + vertex.out_tangent.x, y: vertex.point.y + vertex.out_tangent.y },
                ] {
                    bounds = Some(match bounds {
                        None => [point.x, point.y, point.x, point.y],
                        Some(b) => [b[0].min(point.x), b[1].min(point.y), b[2].max(point.x), b[3].max(point.y)],
                    });
                }
            }
            bounds
        }
    }
    .unwrap_or([-50.0, -50.0, 50.0, 50.0]);
    let [x0, y0, x1, y1] = bounds;
    if (x1 - x0).abs() > f64::EPSILON {
        let y = (y0 + y1) * 0.5;
        (Point { x: x0, y }, Point { x: x1, y })
    } else {
        let x = (x0 + x1) * 0.5;
        (Point { x, y: y0 }, Point { x, y: y1 })
    }
}

fn endpoint_color(gradient: &Gradient, end: bool) -> Option<Rgb> {
    let choose = if end {
        gradient.stops.iter().max_by(|a, b| a.offset.total_cmp(&b.offset))
    } else {
        gradient.stops.iter().min_by(|a, b| a.offset.total_cmp(&b.offset))
    };
    choose.map(|stop| stop.color)
}

fn write_endpoint(gradient: &mut Gradient, end: bool, color: Rgb) {
    match gradient.stops.len() {
        0 => gradient.stops.extend([
            GradientStop { offset: 0.0, color },
            GradientStop { offset: 1.0, color },
        ]),
        1 => {
            let first = gradient.stops[0].color;
            gradient.stops[0].offset = 0.0;
            gradient.stops.push(GradientStop { offset: 1.0, color: first });
        }
        _ => {}
    }
    let index = if end {
        gradient.stops.iter().enumerate().max_by(|(_, a), (_, b)| a.offset.total_cmp(&b.offset))
    } else {
        gradient.stops.iter().enumerate().min_by(|(_, a), (_, b)| a.offset.total_cmp(&b.offset))
    }
    .map(|(index, _)| index);
    if let Some(index) = index {
        gradient.stops[index].color = color;
    }
}

/// Solid/2色Linear Gradientを切り替える。変換自体もSetShapes 1手なのでUndo可能。
/// Gradient化では両端を現在色にして、切替だけで作品の見た目を変えない。
pub(super) fn set_shape_gradient(
    doc: &Arc<Mutex<Document>>,
    slot: &ColorSlot,
    enabled: bool,
) -> Result<(), StoreError> {
    let Some((layer, path)) = shape_location(slot) else { return Ok(()) };
    let mut d = doc.lock().unwrap();
    let mut shapes = d.view().shapes(layer)?;
    let Some(shape) = leaf_mut(&mut shapes, path) else { return Ok(()) };
    let mut fill = shape.fill.take().unwrap_or_default();
    fill.brush = match (enabled, fill.brush) {
        (true, Brush::Solid(color)) => {
            let (start, end) = gradient_axis(&shape.source);
            Brush::Gradient(Gradient {
                kind: GradientType::Linear,
                start,
                end,
                stops: vec![
                    GradientStop { offset: 0.0, color },
                    GradientStop { offset: 1.0, color },
                ],
            })
        }
        (true, brush @ Brush::Gradient(_)) => brush,
        (false, Brush::Gradient(gradient)) => {
            Brush::Solid(endpoint_color(&gradient, false).unwrap_or(Rgb::BLACK))
        }
        (false, brush @ Brush::Solid(_)) => brush,
    };
    shape.fill = Some(fill);
    d.apply(Intent::SetShapes { layer, shapes }).map(|_| ())
}

/// 今の色。無ければ黒。
/// `#ff8800` / `ff8800` / `#f80` を読む。読めなければ None(書かない)。
pub(super) fn parse_hex(text: &str) -> Option<[f64; 3]> {
    let t = text.trim().trim_start_matches('#');
    let expanded: String = match t.len() {
        3 => t.chars().flat_map(|c| [c, c]).collect(),
        6 => t.to_owned(),
        _ => return None,
    };
    let byte = |i: usize| u8::from_str_radix(&expanded[i..i + 2], 16).ok().map(|b| b as f64 / 255.0);
    Some([byte(0)?, byte(2)?, byte(4)?])
}

pub(super) fn read_color(doc: &Arc<Mutex<Document>>, slot: &ColorSlot) -> Option<[f64; 4]> {
    let d = doc.lock().unwrap();
    let view = d.view();
    match slot {
        ColorSlot::TextFill { layer, style } | ColorSlot::TextStroke { layer, style } => {
            let text = view.text_document(*layer).ok()??;
            let found = text.styles.iter().find(|s| s.id == *style)?;
            match slot {
                ColorSlot::TextFill { .. } => Some(found.fill),
                _ => found.stroke_color,
            }
        }
        ColorSlot::ShapeFill { layer, path } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            match shape.fill.as_ref().map(|f| &f.brush) {
                Some(Brush::Solid(rgb)) => Some([rgb.r, rgb.g, rgb.b, 1.0]),
                _ => None,
            }
        }
        ColorSlot::ShapeGradientStop { layer, path, end } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            match shape.fill.as_ref().map(|fill| &fill.brush) {
                Some(Brush::Gradient(gradient)) => endpoint_color(gradient, *end)
                    .map(|rgb| [rgb.r, rgb.g, rgb.b, 1.0]),
                _ => None,
            }
        }
    }
}

/// 掴んでいる間の下見。文字の色は property を持つので transient で Stage に出る。
/// shape の塗りは data だけなので、放した時の 1 回まで待つ。
fn preview_color(doc: &Arc<Mutex<Document>>, slot: &ColorSlot, [r, g, b]: [f64; 3]) {
    let (layer, property, a) = match slot {
        ColorSlot::TextFill { layer, style } => (
            *layer,
            crate::doc::store::PropertyId::text_style_fill_color(*style),
            read_color(doc, slot).map_or(1.0, |c| c[3]),
        ),
        ColorSlot::TextStroke { layer, style } => (
            *layer,
            crate::doc::store::PropertyId::text_style_stroke_color(*style),
            read_color(doc, slot).map_or(1.0, |c| c[3]),
        ),
        ColorSlot::ShapeFill { .. } | ColorSlot::ShapeGradientStop { .. } => return,
    };
    doc.lock().unwrap().set_transient(layer, property, crate::doc::eval::Value::Color([r, g, b, a]));
}

fn clear_preview(doc: &Arc<Mutex<Document>>, slot: &ColorSlot) {
    let (layer, property) = match slot {
        ColorSlot::TextFill { layer, style } => (*layer, crate::doc::store::PropertyId::text_style_fill_color(*style)),
        ColorSlot::TextStroke { layer, style } => (*layer, crate::doc::store::PropertyId::text_style_stroke_color(*style)),
        ColorSlot::ShapeFill { .. } | ColorSlot::ShapeGradientStop { .. } => return,
    };
    doc.lock().unwrap().clear_transient(layer, &property);
}

/// 色を data へ書き戻す。α は触らない。
pub(super) fn write_color(
    doc: &Arc<Mutex<Document>>,
    slot: &ColorSlot,
    [r, g, b]: [f64; 3],
) -> Result<(), StoreError> {
    let mut d = doc.lock().unwrap();
    let intent = match slot {
        ColorSlot::TextFill { layer, style } | ColorSlot::TextStroke { layer, style } => {
            let Some(mut text) = d.view().text_document(*layer)? else { return Ok(()) };
            let Some(found) = text.styles.iter_mut().find(|s| s.id == *style) else { return Ok(()) };
            match slot {
                ColorSlot::TextFill { .. } => found.fill = [r, g, b, found.fill[3]],
                _ => {
                    let a = found.stroke_color.map_or(1.0, |c| c[3]);
                    found.stroke_color = Some([r, g, b, a]);
                    // 幅 0 の縁取りは描かれない。色を付けた時点で見える幅を入れる(級数の 5%)。
                    if found.stroke_width <= 0.0 {
                        found.stroke_width = (found.size * 0.05).max(1.0);
                    }
                }
            }
            Intent::SetTextDocument { layer: *layer, document: text }
        }
        ColorSlot::ShapeFill { layer, path } => {
            let mut shapes = d.view().shapes(*layer)?;
            let Some(shape) = leaf_mut(&mut shapes, path) else { return Ok(()) };
            // gradient の塗りは単色で潰さない(輪の相手は read_color が Solid の時だけ)。
            if matches!(shape.fill.as_ref().map(|f| &f.brush), Some(Brush::Gradient(_))) {
                return Ok(());
            }
            let mut fill = shape.fill.take().unwrap_or_default();
            fill.brush = Brush::Solid(Rgb { r, g, b });
            shape.fill = Some(Fill { ..fill });
            Intent::SetShapes { layer: *layer, shapes }
        }
        ColorSlot::ShapeGradientStop { layer, path, end } => {
            let mut shapes = d.view().shapes(*layer)?;
            let Some(shape) = leaf_mut(&mut shapes, path) else { return Ok(()) };
            let Some(fill) = shape.fill.as_mut() else { return Ok(()) };
            let Brush::Gradient(gradient) = &mut fill.brush else { return Ok(()) };
            write_endpoint(gradient, *end, Rgb { r, g, b });
            Intent::SetShapes { layer: *layer, shapes }
        }
    };
    d.apply(intent).map(|_| ())
}

/// 色の輪。Browser の Colors に常設。輪で色相、面で彩度と明度。掴んでいる間は下書きで、
/// 放した時に 1 回だけ書く(Undo が 1 手になる)。
#[component]
pub(super) fn ColorWheel(session: Session, slot: ColorSlot, revision: Signal<u32>) -> Element {
    let mut revision = revision;
    let current = read_color(&session.doc, &slot).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let mut draft: Signal<Option<(f64, f64, f64)>> = use_signal(|| None);
    // 不透明度の下書き。掴んで滑らせ、放した時に 1 回だけ書く。
    let mut alpha_draft: Signal<Option<f64>> = use_signal(|| None);
    let (h, s, v) = draft().unwrap_or_else(|| rgb_to_hsv([current[0], current[1], current[2]]));
    let k = session.scale.factor();
    let ring = RING * k;
    let square = SQUARE * k;
    let inset = (ring - square) / 2.0;
    let hue_hex = hex_of(hsv_to_rgb(h, 1.0, 1.0));
    let shown = hex_of(hsv_to_rgb(h, s, v));
    // 輪は赤が 12 時、時計回り(Photoshop・Resolve)。画面の角度(3 時が 0)とは 90° ずれる。
    let a = (h - 90.0).to_radians();
    let r = ring / 2.0 - (ring - square) / 4.0;
    let (mx, my) = (ring / 2.0 + r * a.cos(), ring / 2.0 + r * a.sin());
    let (sx, sy) = (inset + s * square, inset + (1.0 - v) * square);

    let commit = {
        let session = session.clone();
        let slot = slot.clone();
        move || {
            let Some((h, s, v)) = draft.write().take() else { return };
            clear_preview(&session.doc, &slot);
            if !session.writable(slot.layer()) {
                return;
            }
            let rgb = hsv_to_rgb(h, s, v);
            let same = read_color(&session.doc, &slot)
                .is_some_and(|c| (0..3).all(|i| (c[i] - rgb[i]).abs() < 1e-9));
            if same {
                return;
            }
            match write_color(&session.doc, &slot, rgb) {
                Ok(()) => *revision.write() += 1,
                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
            }
        }
    };
    let preview_hue = (session.clone(), slot.clone());
    let pick_hue = move |evt: PointerEvent| {
        let p = evt.data().element_coordinates();
        let deg = ((p.y - ring / 2.0).atan2(p.x - ring / 2.0).to_degrees() + 90.0).rem_euclid(360.0);
        draft.set(Some((deg, s, v)));
        preview_color(&preview_hue.0.doc, &preview_hue.1, hsv_to_rgb(deg, s, v));
        *revision.write() += 1;
    };
    let preview_sv = (session.clone(), slot.clone());
    let pick_sv = move |evt: PointerEvent| {
        let p = evt.data().element_coordinates();
        let s = (p.x / square).clamp(0.0, 1.0);
        let v = (1.0 - p.y / square).clamp(0.0, 1.0);
        draft.set(Some((h, s, v)));
        preview_color(&preview_sv.0.doc, &preview_sv.1, hsv_to_rgb(h, s, v));
        *revision.write() += 1;
    };
    let commit_alpha = {
        let session = session.clone();
        let slot = slot.clone();
        move || {
            let Some(a) = alpha_draft.write().take() else { return };
            if !session.writable(slot.layer()) {
                return;
            }
            match write_alpha(&session.doc, &slot, a) {
                Ok(()) => *revision.write() += 1,
                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
            }
        }
    };
    let held = |evt: &PointerEvent| {
        evt.data()
            .held_buttons()
            .contains(dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary)
    };
    let mut commit_up = commit.clone();
    let mut commit_far = commit.clone();
    let mut commit_leave = commit.clone();
    rsx!(div { class: "color-pick",
        onpointerup: { let mut alpha = commit_alpha.clone(); move |_| { commit_up(); alpha() } },
        // 引き出しの外へ出たら、そこまでの色で確定(掴んだまま彷徨わせない)。
        onpointerleave: { let mut alpha = commit_alpha.clone(); move |_| { commit_leave(); alpha() } },
        // 外で放して戻ってきた時。押していないのに下書きが残っていれば、それが放した印。
        onpointermove: move |evt: PointerEvent| {
            if !held(&evt) && draft.peek().is_some() {
                commit_far();
            }
        },
        div { class: "color-wheel", style: "width: {ring}px; height: {ring}px;",
            div {
                class: "hue-ring",
                onpointerdown: pick_hue.clone(),
                onpointermove: {
                    let mut pick = pick_hue.clone();
                    move |evt: PointerEvent| if held(&evt) { pick(evt) }
                },
            }
            div {
                class: "sv-square",
                style: "left: {inset}px; top: {inset}px; width: {square}px; height: {square}px; background: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, {hue_hex});",
                onpointerdown: pick_sv.clone(),
                onpointermove: {
                    let mut pick = pick_sv.clone();
                    move |evt: PointerEvent| if held(&evt) { pick(evt) }
                },
            }
            span { class: "color-mark", style: "left: {mx}px; top: {my}px;" }
            span { class: "color-mark", style: "left: {sx}px; top: {sy}px;" }
        }
        div { class: "color-now",
            span { class: "dot", style: "background: {shown};" }
            // hex は打てる(Figma・Photoshop)。押して打ち、Enter で書く。
            if session.field_at(&FieldAt::Hex(slot.clone())).is_some() {
                Field {
                    label: "Hex color",
                    session: session.clone(),
                    class: "hex typing",
                    revision,
                    oncommit: {
                        let session = session.clone();
                        move |f: OpenField| {
                            let FieldAt::Hex(slot) = f.at else { return };
                            let Some(rgb) = parse_hex(&f.draft) else { return };
                            if !session.writable(slot.layer()) {
                                return;
                            }
                            match write_color(&session.doc, &slot, rgb) {
                                Ok(()) => *revision.write() += 1,
                                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                            }
                        }
                    },
                }
            } else {
                SemanticButton {
                    class: "hex",
                    title: "Type a hex color",
                    onclick: {
                        let session = session.clone();
                        let slot = slot.clone();
                        let shown = shown.clone();
                        move |_| {
                            session.open_field(FieldAt::Hex(slot.clone()), shown.clone());
                            *revision.write() += 1;
                        }
                    },
                    "{shown}"
                }
            }
        }
        // 不透明度。歌詞のフェードは色でもやる(Transform の opacity だけに頼らない)。
        if !slot.is_shape_fill() {
            div {
                class: "alpha-bar",
                style: "width: {ring}px; background: linear-gradient(to right, transparent, {shown});",
                onpointerdown: move |evt: PointerEvent| {
                    alpha_draft.set(Some((evt.data().element_coordinates().x / ring).clamp(0.0, 1.0)));
                },
                // 帯から縦にぶれても追う。押していないのに下書きが残っていれば、それが放した印(色と同じ保険)。
                onpointermove: {
                    let mut commit = commit_alpha.clone();
                    move |evt: PointerEvent| {
                        if alpha_draft.peek().is_none() {
                            return;
                        }
                        if held(&evt) {
                            alpha_draft.set(Some((evt.data().element_coordinates().x / ring).clamp(0.0, 1.0)));
                        } else {
                            commit();
                        }
                    }
                },
                onpointerup: { let mut commit = commit_alpha.clone(); move |_| commit() },
                span { class: "color-mark", style: "left: {alpha_draft().unwrap_or(current[3]) * ring}px; top: 50%;" }
            }
        }
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_wheel_math_round_trips() {
        for (h, s, v) in [(0.0, 1.0, 1.0), (120.0, 0.5, 0.75), (300.0, 0.2, 0.1), (0.0, 0.0, 0.5)] {
            let (h2, s2, v2) = rgb_to_hsv(hsv_to_rgb(h, s, v));
            let h_ok = s <= f64::EPSILON || (h - h2).abs() < 1e-6;
            assert!(h_ok && (s - s2).abs() < 1e-6 && (v - v2).abs() < 1e-6, "{h} {s} {v} -> {h2} {s2} {v2}");
        }
        assert_eq!(hex_of(hsv_to_rgb(0.0, 1.0, 1.0)), "#ff0000");
    }

    /// 色は shape / text の data へ戻る。Inspector が見せる行と同じ場所を書く。
    #[test]
    fn a_color_written_through_its_slot_is_the_color_the_inspector_shows() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        let layers = session.doc.lock().unwrap().view().layers();
        let t = crate::doc::store::RationalTime::ZERO;
        let mut slots = Vec::new();
        for layer in layers {
            let d = session.doc.lock().unwrap();
            for row in crate::ui::fixture::inspector_data_from_doc(&d.view(), layer, t).colors {
                slots.push(row.slot);
            }
        }
        assert!(!slots.is_empty(), "the fixture has no colored layer to test against");
        for slot in slots {
            write_color(&session.doc, &slot, [0.25, 0.5, 0.75]).unwrap();
            let back = read_color(&session.doc, &slot).unwrap();
            assert!((back[0] - 0.25).abs() < 1e-9 && (back[1] - 0.5).abs() < 1e-9 && (back[2] - 0.75).abs() < 1e-9, "{slot:?} {back:?}");
            let layer = slot.layer();
            let d = session.doc.lock().unwrap();
            let rows = crate::ui::fixture::inspector_data_from_doc(&d.view(), layer, t).colors;
            assert!(rows.iter().any(|r| r.slot == slot && r.hex.starts_with("#4080bf")), "{slot:?} {:?}", rows.iter().map(|r| r.hex.clone()).collect::<Vec<_>>());
        }
    }

    #[test]
    fn a_shape_fill_switches_to_two_editable_gradient_ends_without_changing_color() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        let t = crate::doc::store::RationalTime::ZERO;
        let (layer, solid) = {
            let d = session.doc.lock().unwrap();
            d.view()
                .layers()
                .into_iter()
                .find_map(|layer| {
                    crate::ui::fixture::inspector_data_from_doc(&d.view(), layer, t)
                        .colors
                        .into_iter()
                        .find(|row| matches!(row.slot, ColorSlot::ShapeFill { .. }))
                        .map(|row| (layer, row.slot))
                })
                .expect("fixture has no solid shape fill")
        };
        let original = read_color(&session.doc, &solid).unwrap();

        set_shape_gradient(&session.doc, &solid, true).unwrap();
        let rows = {
            let d = session.doc.lock().unwrap();
            crate::ui::fixture::inspector_data_from_doc(&d.view(), layer, t).colors
        };
        assert_eq!(rows.len(), 2);
        let start = rows.iter().find(|row| matches!(row.slot, ColorSlot::ShapeGradientStop { end: false, .. })).unwrap().slot.clone();
        let end = rows.iter().find(|row| matches!(row.slot, ColorSlot::ShapeGradientStop { end: true, .. })).unwrap().slot.clone();
        assert_eq!(read_color(&session.doc, &start), Some(original));
        assert_eq!(read_color(&session.doc, &end), Some(original));

        write_color(&session.doc, &end, [0.1, 0.2, 0.3]).unwrap();
        let changed = read_color(&session.doc, &end).unwrap();
        assert!((changed[0] - 0.1).abs() < 1e-9 && (changed[1] - 0.2).abs() < 1e-9 && (changed[2] - 0.3).abs() < 1e-9);

        set_shape_gradient(&session.doc, &start, false).unwrap();
        let rows = {
            let d = session.doc.lock().unwrap();
            crate::ui::fixture::inspector_data_from_doc(&d.view(), layer, t).colors
        };
        assert_eq!(rows.len(), 1);
        assert!(matches!(rows[0].slot, ColorSlot::ShapeFill { .. }));
    }


}
