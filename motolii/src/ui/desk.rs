//! 机 — Document を覗くレンズ。誰にも呼ばれない。
//! 常設なのは細い顔(書き置き・取っ手・鳥)。引き出しは机の中に出て、その間だけ机が広がる。

use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::doc::store::{Document, Intent, LayerId, Marker, ShapeNode, StoreError};
use crate::doc::vector::{Brush, Fill, Rgb};
use crate::ui::inspector::{write_blend, BLEND_MODES};
use crate::ui::semantic_menu::{Field, SemanticButton};
use crate::ui::session::{ColorSlot, DeskState, FieldAt, Focus, OpenField, Session};

/// 引き出しは焦点の型に一つ。履歴だけが型を持たない例外。
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Drawer {
    Key,
    Blend,
    Color,
    Text,
    History,
}

const DRAWERS: &[(Drawer, &str)] = &[
    (Drawer::Key, "Ease"),
    (Drawer::Blend, "Blend"),
    (Drawer::Color, "Color"),
    (Drawer::Text, "Text"),
    (Drawer::History, "History"),
];

/// 焦点が導く引き出し。
fn derived(session: &Session) -> Option<Drawer> {
    match session.live_focus() {
        Some(Focus::Blend(_)) => Some(Drawer::Blend),
        Some(Focus::Color(_)) => Some(Drawer::Color),
        None if !session.selected_keys.lock().unwrap().is_empty() => Some(Drawer::Key),
        None => None,
    }
}

/// 今開く引き出し。手で開けた物が先。手で閉じた物は、焦点が導く物が変わるまで開かない。
pub(super) fn drawer_of(session: &Session) -> Option<Drawer> {
    let derived = derived(session);
    match *session.desk.lock().unwrap() {
        DeskState::Open(drawer) => Some(drawer),
        DeskState::Follow => derived,
        DeskState::Shut(seen) if seen == derived => None,
        DeskState::Shut(_) => derived,
    }
}

/// 手で閉じる。焦点も手放す。
fn shut(session: &Session) {
    *session.focus.lock().unwrap() = None;
    let derived = derived(session);
    *session.desk.lock().unwrap() = DeskState::Shut(derived);
}

/// 今の時刻に効いているマーカー。並びは時刻順なので、時刻以前の最後の物。
fn current_marker(markers: &[Marker], now: f64) -> Option<usize> {
    markers
        .iter()
        .enumerate()
        .filter(|(_, m)| m.time.as_seconds_f64() <= now + 1e-9)
        .last()
        .map(|(i, _)| i)
}

fn write_marker_body(
    doc: &Arc<Mutex<Document>>,
    index: usize,
    body: String,
) -> Result<(), StoreError> {
    let mut d = doc.lock().unwrap();
    let mut markers = d.view().markers()?;
    let Some(marker) = markers.get_mut(index) else {
        return Ok(());
    };
    marker.body = body;
    d.apply(Intent::SetMarkers { markers })
}

const STAND: [&str; 8] = [
    ".KKK....", "KKCKKK..", ".KKKKKK.", ".CCKKKKK", ".CCCKKKK", "..CCKKKK", "..C.C.KK", ".......K",
];
const SINK: [&str; 8] = [
    "........", ".KKK....", "KKCKKK..", ".KKKKKK.", ".CCKKKKK", ".CCCKKKK", "..CCKKKK", ".......K",
];

/// 8×8・2色のカササギ(2026-08-08 決定)。目は行の中で最初の `C`。
fn pet_frame(rows: [&str; 8], class: &str) -> Element {
    let mut eye_done = false;
    let cells = rows.iter().enumerate().flat_map(|(y, row)| {
        row.chars().enumerate().map(move |(x, ch)| (y, x, ch))
    });
    let px = cells
        .map(|(_, _, ch)| {
            let class = match ch {
                'K' => "pet-px k",
                'C' if !eye_done => {
                    eye_done = true;
                    "pet-px c eye"
                }
                'C' => "pet-px c",
                _ => "pet-px",
            };
            rsx!(div { class: "{class}" })
        })
        .collect::<Vec<_>>();
    rsx!(div { class: "pet-frame {class}", {px.into_iter()} })
}

/// Blend の引き出しが見る層。生きている焦点、無ければ選んでいる層。
fn blend_target(session: &Session) -> Option<LayerId> {
    match session.live_focus() {
        Some(Focus::Blend(layer)) => Some(layer),
        _ => session.selection.get(),
    }
}

/// 机。顔は常に細く、引き出しは開いた時だけ中に出る。
#[component]
pub(super) fn DeskPanel(
    session: Session,
    revision: Signal<u32>,
    playhead: Signal<f64>,
    on_history: EventHandler<i32>,
) -> Element {
    let _ = revision();
    let _ = playhead();
    let mut revision = revision;

    let now = session.clock.now_sec();
    let markers = session.doc.lock().unwrap().view().markers().unwrap_or_default();
    let current = current_marker(&markers, now);
    let drawer = drawer_of(&session);

    let face_name = current
        .map(|i| markers[i].name.clone())
        .unwrap_or_else(|| "No marker yet".to_owned());
    let note = drawer.filter(|d| *d == Drawer::Text).map(|_| match current {
        Some(i) => {
            let marker = &markers[i];
            let editing = session.field_at(&FieldAt::Note(i)).is_some();
            let doc = session.doc.clone();
            let body = marker.body.clone();
            let opener = session.clone();
            rsx!(div { class: "desk-note",
                span { class: "mname", "{marker.name}" }
                // 欄は押した間だけ在る。Enter・Escape・外を押す、のどれでも欄ごと消える。
                if editing {
                    Field {
                        session: session.clone(),
                        class: "mbody",
                        multiline: true,
                        revision,
                        oncommit: move |f: OpenField| {
                            let FieldAt::Note(at) = f.at else { return };
                            match write_marker_body(&doc, at, f.draft) {
                                Ok(()) => *revision.write() += 1,
                                Err(err) => println!("PROBE room=write verdict=apply-error {err}"),
                            }
                        },
                    }
                } else {
                    SemanticButton {
                        class: if body.is_empty() { "mbody idle" } else { "mbody" },
                        aria_label: "Edit note",
                        onclick: move |_| {
                            opener.open_field(FieldAt::Note(i), body.clone());
                            *revision.write() += 1;
                        },
                        if marker.body.is_empty() { "Write what happens here" } else { "{marker.body}" }
                    }
                }
            })
        }
        None => rsx!(div { class: "desk-note idle", span { class: "mname", "No marker yet" } }),
    });

    let drawer_body = drawer.map(|drawer| match drawer {
        Drawer::Text => note.clone().unwrap_or_else(|| rsx! {}),
        other => drawer_body(&session, other, revision, on_history),
    });
    let drawer_label = drawer
        .and_then(|d| DRAWERS.iter().find(|(x, _)| *x == d))
        .map(|(_, l)| *l)
        .unwrap_or("");

    rsx!(div { id: "desk",
        if let Some(body) = drawer_body {
            div { class: "desk-drawer",
                div { class: "dhead",
                    span { "{drawer_label}" }
                    SemanticButton {
                        class: "chip",
                        aria_label: "Close drawer",
                        onclick: {
                            let session = session.clone();
                            move |_| {
                                shut(&session);
                                *revision.write() += 1;
                            }
                        },
                        "×"
                    }
                }
                {body}
            }
        }
        div { class: "desk-face",
        span { class: "mname", "{face_name}" }
        div { class: "desk-foot",
            for (which , label) in DRAWERS.iter().copied() {
                SemanticButton {
                    class: if drawer == Some(which) { "chip on" } else { "chip" },
                    selected: drawer == Some(which),
                    aria_label: "Open {label}",
                    onclick: {
                        let session = session.clone();
                        move |_| {
                            // 開いている物を押せば閉じる。焦点で開いた物も同じ手で閉じる。
                            if drawer == Some(which) {
                                shut(&session);
                            } else {
                                *session.desk.lock().unwrap() = DeskState::Open(which);
                            }
                            *revision.write() += 1;
                        }
                    },
                    "{label}"
                }
            }
        }
        div { class: "pet",
            {pet_frame(STAND, "stand")}
            {pet_frame(SINK, "sink")}
        }
        }
    })
}

/// 引き出し。要る時だけ前へ浮かび、配置を押し広げない。
/// Ease の盤。widget は一度しか渡せないので、引き出しと一緒に生まれて一緒に消える。
#[component]
fn EaseDrawer(session: Session, revision: Signal<u32>) -> Element {
    let shape = use_hook(|| Arc::new(Mutex::new(crate::ui::ease_widget::DEFAULT)));
    let editor = use_hook(|| {
        CustomWidgetAttr::new(crate::ui::ease_widget::EaseWidget::new(
            shape.clone(),
            session.clone(),
        ))
    });
    let kind_name = use_signal(String::new);
    let kinds = use_hook(|| {
        CustomWidgetAttr::new(
            crate::ui::ease_widget::KindsWidget::new(shape.clone(), session.clone())
                .with_name_mirror(kind_name),
        )
    });
    crate::ui::ease::ease_panel(&session, editor, kinds, kind_name, revision)
}

/// 引き出しの中身。型に一つ。
fn drawer_body(
    session: &Session,
    drawer: Drawer,
    revision: Signal<u32>,
    on_history: EventHandler<i32>,
) -> Element {
    let mut revision = revision;
    match drawer {
        Drawer::Key => rsx!(EaseDrawer { session: session.clone(), revision }),
        Drawer::Blend => {
            let target = blend_target(session).and_then(|layer| {
                let d = session.doc.lock().unwrap();
                d.view().attrs(layer).ok().flatten().map(|a| (layer, a.blend_mode))
            });
            match target {
                None => rsx!(div { class: "dempty", "Pick a layer" }),
                Some((layer, current)) => rsx!(div { class: "blend-grid",
                    for (mode , label) in BLEND_MODES.iter().copied() {
                        SemanticButton {
                            class: if mode == current { "blend-cell on" } else { "blend-cell" },
                            selected: mode == current,
                            aria_label: "Blend {label}",
                            onclick: {
                                let doc = session.doc.clone();
                                move |_| match write_blend(&doc, layer, mode) {
                                    Ok(_) => *revision.write() += 1,
                                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                }
                            },
                            span { class: "blend-swatch" }
                            span { class: "blend-name", "{label}" }
                        }
                    }
                }),
            }
        }
        Drawer::Color => {
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
            match slot {
                None => rsx!(div { class: "dempty", "Pick a color" }),
                Some(slot) => rsx!(ColorDrawer { session: session.clone(), slot, revision }),
            }
        }
        Drawer::Text => rsx! {},
        Drawer::History => {
            let (back, forward) = session.doc.lock().unwrap().history_depth();
            rsx!(div { class: "history-strip",
                for step in (1..=back).rev() {
                    SemanticButton {
                        class: "hstep past",
                        aria_label: "Back {step}",
                        onclick: move |_| on_history.call(-(step as i32)),
                    }
                }
                span { class: "hstep now" }
                for step in 1..=forward {
                    SemanticButton {
                        class: "hstep ahead",
                        aria_label: "Forward {step}",
                        onclick: move |_| on_history.call(step as i32),
                    }
                }
            })
        }
    }
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

/// 今の色。無ければ黒。
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
    }
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
                }
            }
            Intent::SetTextDocument { layer: *layer, document: text }
        }
        ColorSlot::ShapeFill { layer, path } => {
            let mut shapes = d.view().shapes(*layer)?;
            let Some(shape) = leaf_mut(&mut shapes, path) else { return Ok(()) };
            let mut fill = shape.fill.take().unwrap_or_default();
            fill.brush = Brush::Solid(Rgb { r, g, b });
            shape.fill = Some(Fill { ..fill });
            Intent::SetShapes { layer: *layer, shapes }
        }
    };
    d.apply(intent).map(|_| ())
}

/// 色の引き出し。輪で色相、面で彩度と明度。掴んでいる間は下書きで、放した時に 1 回だけ書く
/// (Undo が 1 手になる)。
#[component]
fn ColorDrawer(session: Session, slot: ColorSlot, revision: Signal<u32>) -> Element {
    let mut revision = revision;
    let current = read_color(&session.doc, &slot).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let mut draft: Signal<Option<(f64, f64, f64)>> = use_signal(|| None);
    let (h, s, v) = draft().unwrap_or_else(|| rgb_to_hsv([current[0], current[1], current[2]]));
    let k = session.scale.factor();
    let ring = RING * k;
    let square = SQUARE * k;
    let inset = (ring - square) / 2.0;
    let hue_hex = hex_of(hsv_to_rgb(h, 1.0, 1.0));
    let shown = hex_of(hsv_to_rgb(h, s, v));
    let a = h.to_radians();
    let r = ring / 2.0 - (ring - square) / 4.0;
    let (mx, my) = (ring / 2.0 + r * a.cos(), ring / 2.0 + r * a.sin());
    let (sx, sy) = (inset + s * square, inset + (1.0 - v) * square);

    let commit = {
        let session = session.clone();
        let slot = slot.clone();
        move || {
            let Some((h, s, v)) = draft.write().take() else { return };
            match write_color(&session.doc, &slot, hsv_to_rgb(h, s, v)) {
                Ok(()) => *revision.write() += 1,
                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
            }
        }
    };
    let mut pick_hue = move |evt: PointerEvent| {
        let p = evt.data().element_coordinates();
        let deg = (p.y - ring / 2.0).atan2(p.x - ring / 2.0).to_degrees().rem_euclid(360.0);
        draft.set(Some((deg, s, v)));
    };
    let mut pick_sv = move |evt: PointerEvent| {
        let p = evt.data().element_coordinates();
        let s = (p.x / square).clamp(0.0, 1.0);
        let v = (1.0 - p.y / square).clamp(0.0, 1.0);
        draft.set(Some((h, s, v)));
    };
    let held = |evt: &PointerEvent| {
        evt.data()
            .held_buttons()
            .contains(dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary)
    };
    let mut commit_up = commit.clone();
    let mut commit_far = commit.clone();
    rsx!(div { class: "color-drawer",
        onpointerup: move |_| commit_up(),
        // 外で放して戻ってきた時。押していないのに下書きが残っていれば、それが放した印。
        onpointermove: move |evt: PointerEvent| {
            if !held(&evt) && draft.peek().is_some() {
                commit_far();
            }
        },
        div { class: "color-wheel", style: "width: {ring}px; height: {ring}px;",
            div {
                class: "hue-ring",
                onpointerdown: pick_hue,
                onpointermove: move |evt: PointerEvent| if held(&evt) { pick_hue(evt) },
            }
            div {
                class: "sv-square",
                style: "left: {inset}px; top: {inset}px; width: {square}px; height: {square}px; background: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, {hue_hex});",
                onpointerdown: pick_sv,
                onpointermove: move |evt: PointerEvent| if held(&evt) { pick_sv(evt) },
            }
            span { class: "color-mark", style: "left: {mx}px; top: {my}px;" }
            span { class: "color-mark", style: "left: {sx}px; top: {sy}px;" }
        }
        div { class: "color-now",
            span { class: "dot", style: "background: {shown};" }
            span { "{shown}" }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::RationalTime;

    fn marker(sec: i64) -> Marker {
        Marker {
            name: sec.to_string(),
            time: RationalTime::try_new(sec, 1).unwrap(),
            duration: RationalTime::ZERO,
            body: String::new(),
        }
    }

    #[test]
    fn the_note_follows_the_last_marker_at_or_before_the_playhead() {
        let markers = [marker(1), marker(5), marker(9)];
        assert_eq!(current_marker(&markers, 0.5), None);
        assert_eq!(current_marker(&markers, 5.0), Some(1));
        assert_eq!(current_marker(&markers, 8.9), Some(1));
        assert_eq!(current_marker(&markers, 30.0), Some(2));
    }

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
            assert!(rows.iter().any(|r| r.slot == slot && r.hex.starts_with("#3f7fbf")), "{slot:?} {:?}", rows.iter().map(|r| r.hex.clone()).collect::<Vec<_>>());
        }
    }

    #[test]
    fn the_magpie_has_one_eye_per_frame() {
        for rows in [STAND, SINK] {
            let eyes = rows.iter().filter(|row| row.contains('C')).count();
            assert!(eyes >= 1, "a frame without a motif pixel has no eye to blink");
        }
    }
}
