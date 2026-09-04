//! 机 — Document を覗くレンズ。誰にも呼ばれない。
//! 常設なのは細い顔(書き置き・取っ手・鳥)。引き出しは机の中に出て、その間だけ机が広がる。

const NO_MARKER: &str = "No marker yet";

/// Blend の札に載せる色。層の最初の塗り(gradientなら始端)、無ければ accent。
fn blend_tint(session: &Session, layer: crate::doc::store::LayerId) -> [f32; 3] {
    let d = session.doc.lock().unwrap();
    let view = d.view();
    // 文字層は本文の塗り(歌詞は文字なので、札が層の色を映す)。
    if let Ok(Some(text)) = view.text_document(layer) {
        if let Some(style) = text.styles.first() {
            return [
                style.fill[0] as f32,
                style.fill[1] as f32,
                style.fill[2] as f32,
            ];
        }
    }
    let shapes = view.shapes(layer).unwrap_or_default();
    match crate::ui::fixture::first_shape_fill(&shapes, Vec::new()) {
        Some((_, crate::doc::vector::Brush::Solid(rgb))) => {
            [rgb.r as f32, rgb.g as f32, rgb.b as f32]
        }
        Some((_, crate::doc::vector::Brush::Gradient(gradient))) => gradient
            .stops
            .iter()
            .min_by(|a, b| a.offset.total_cmp(&b.offset))
            .map(|stop| {
                [
                    stop.color.r as f32,
                    stop.color.g as f32,
                    stop.color.b as f32,
                ]
            })
            .unwrap_or_else(|| {
                let a = crate::ui::tokens::ACCENT;
                [
                    a[0] as f32 / 255.0,
                    a[1] as f32 / 255.0,
                    a[2] as f32 / 255.0,
                ]
            }),
        None => {
            let a = crate::ui::tokens::ACCENT;
            [
                a[0] as f32 / 255.0,
                a[1] as f32 / 255.0,
                a[2] as f32 / 255.0,
            ]
        }
    }
}
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::doc::store::{Document, Intent, LayerId, Marker, StoreError};
use crate::ui::inspector::{write_blend, BLEND_MODES};
use crate::ui::semantic_menu::{
    page_scroll, Field, FocusableItems, SemanticButton, SpatialDirection,
};
use crate::ui::session::{DeskState, FieldAt, Focus, OpenField, Session};

/// 引き出しは焦点の型に一つ。履歴だけが型を持たない例外。
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Drawer {
    Key,
    Blend,
    Text,
    History,
}

const DRAWERS: &[(Drawer, &str)] = &[
    (Drawer::Key, "Ease"),
    (Drawer::Blend, "Blend"),
    (Drawer::Text, "Text"),
    (Drawer::History, "History"),
];

#[derive(Clone, Copy)]
struct Roving {
    items: FocusableItems,
    cursor: Signal<usize>,
}

impl Roving {
    fn tabindex(self, index: usize) -> String {
        if (self.cursor)() == index { "0" } else { "-1" }.to_owned()
    }

    fn point_to(mut self, keys: &[String], index: usize) {
        let Some(key) = keys.get(index) else { return };
        self.cursor.set(index);
        self.items.focus(key);
    }

    fn key(self, evt: &KeyboardEvent, keys: &[String]) -> bool {
        if keys.is_empty() {
            return false;
        }
        match evt.key() {
            Key::Home => self.point_to(keys, 0),
            Key::End => self.point_to(keys, keys.len() - 1),
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                let direction = match evt.key() {
                    Key::ArrowLeft => SpatialDirection::Left,
                    Key::ArrowRight => SpatialDirection::Right,
                    Key::ArrowUp => SpatialDirection::Up,
                    Key::ArrowDown => SpatialDirection::Down,
                    _ => unreachable!(),
                };
                let index = (self.cursor)().min(keys.len() - 1);
                let current = keys[index].clone();
                let ordered = keys.to_vec();
                let mut cursor = self.cursor;
                self.items
                    .move_spatial(&current, keys, direction, move |next| {
                        if let Some(index) = ordered.iter().position(|key| key == &next) {
                            cursor.set(index);
                        }
                    });
            }
            _ => return false,
        }
        evt.prevent_default();
        evt.stop_propagation();
        true
    }
}

type MountedSlot = std::rc::Rc<std::cell::RefCell<Option<std::rc::Rc<MountedData>>>>;

fn page_key(evt: &KeyboardEvent, mounted: &MountedSlot) -> bool {
    let direction = match evt.key() {
        Key::PageUp => -1,
        Key::PageDown => 1,
        _ => return false,
    };
    evt.prevent_default();
    evt.stop_propagation();
    if let Some(handle) = mounted.borrow().as_ref().cloned() {
        page_scroll(handle, direction);
    }
    true
}

fn mounted_slot() -> MountedSlot {
    std::rc::Rc::new(std::cell::RefCell::new(None))
}

/// 焦点が導く引き出し。
fn derived(session: &Session) -> Option<Drawer> {
    match session.live_focus() {
        Some(Focus::Blend(_)) => Some(Drawer::Blend),
        // 色は机に出さない。Browser の Colors に常設の輪が居て、焦点に付いて行く(2026-09-03 利用者)。
        Some(Focus::Color(_)) => None,
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
        // 手で閉じた引き出しは、手で開けるまで開かない(選択を変えただけで勝手に戻らない)。
        DeskState::Shut => None,
    }
}

/// 手で閉じる。焦点も手放す。
fn shut(session: &Session) {
    *session.focus.lock().unwrap() = None;
    *session.desk.lock().unwrap() = DeskState::Shut;
}

/// 机が見せる書き置き。開けている欄が在ればその印(再生が進んでも欄は逃げない)、
/// 無ければ今の時刻の印。
fn note_index(session: &Session, markers: &[Marker], now: f64) -> Option<usize> {
    match session.field().map(|f| f.at) {
        Some(FieldAt::Note(at)) => markers
            .iter()
            .position(|m| m.time == at)
            .or_else(|| current_marker(markers, now)),
        _ => current_marker(markers, now),
    }
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

/// 印は時刻で引く。index は打ち直しや Undo でずれる。
fn write_marker_body(
    doc: &Arc<Mutex<Document>>,
    at: crate::doc::store::RationalTime,
    body: String,
) -> Result<(), StoreError> {
    let mut d = doc.lock().unwrap();
    let mut markers = d.view().markers()?;
    let Some(marker) = markers.iter_mut().find(|m| m.time == at) else {
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
    let cells = rows
        .iter()
        .enumerate()
        .flat_map(|(y, row)| row.chars().enumerate().map(move |(x, ch)| (y, x, ch)));
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
    echo: u32,
    revision: Signal<u32>,
    playhead: Signal<f64>,
    on_history: EventHandler<i32>,
) -> Element {
    let items = use_context_provider(FocusableItems::new);
    let drawer_roving = Roving {
        items,
        cursor: use_signal(|| 0usize),
    };
    let blend_roving = Roving {
        items,
        cursor: use_signal(|| 0usize),
    };
    let history_roving = Roving {
        items,
        cursor: use_signal(|| 0usize),
    };
    let blend_scroll = use_hook(mounted_slot);
    let history_scroll = use_hook(mounted_slot);
    let refs_scroll = use_hook(mounted_slot);
    // 下見(blend の transient)は格子が消えたら落とす。mouseleave 無しで消える経路が幾つもある。
    let shown_blend: std::rc::Rc<std::cell::Cell<Option<crate::doc::store::LayerId>>> =
        use_hook(|| std::rc::Rc::new(std::cell::Cell::new(None)));
    {
        let doc = session.doc.clone();
        let shown = shown_blend.clone();
        let watch = session.clone();
        let rev = revision;
        use_effect(move || {
            // 反応源を読む(読まないと初回しか走らず、閉じた時の掃除が一度も来ない)。
            let _ = rev();
            let grid_open = matches!(*watch.desk.lock().unwrap(), DeskState::Open(Drawer::Blend))
                || (matches!(*watch.desk.lock().unwrap(), DeskState::Follow)
                    && matches!(*watch.focus.lock().unwrap(), Some(Focus::Blend(_))));
            if !grid_open {
                if let Some(layer) = shown.take() {
                    doc.lock()
                        .unwrap()
                        .clear_transient(layer, &crate::doc::store::PropertyId::blend_mode());
                }
            }
        });
    }
    let _ = revision();
    let _ = playhead();
    let mut revision = revision;

    let now = session.clock.now_sec();
    let markers = session
        .doc
        .lock()
        .unwrap()
        .view()
        .markers()
        .unwrap_or_default();
    let current = note_index(&session, &markers, now);
    let drawer = drawer_of(&session);

    let face_name = current
        .map(|i| markers[i].name.clone())
        .unwrap_or_else(|| NO_MARKER.to_owned());
    let refs = crate::ui::fixture::reference_images_from_view(&session.doc.lock().unwrap().view());
    let note = drawer.filter(|d| *d == Drawer::Text).map(|_| match current {
        Some(i) => {
            let marker = &markers[i];
            let at = marker.time;
            let editing = session.field_at(&FieldAt::Note(at)).is_some();
            let doc = session.doc.clone();
            let body = marker.body.clone();
            let unchanged = marker.body.clone();
            let opener = session.clone();
            // 書き置きを選んでいる文字層の本文へ。歌詞は机で打って層へ送る(二度打ちさせない)。
            let text_layer = session.selection.get().filter(|l| {
                session.doc.lock().unwrap().view().text_document(*l).ok().flatten().is_some()
            });
            let to_layer = text_layer.map(|layer| {
                let doc = session.doc.clone();
                let clock = session.clock.clone();
                let body = marker.body.clone();
                move |_| match crate::ui::inspector::write_content(&doc, layer, clock.current_time(), body.clone()) {
                    Ok(()) => *revision.write() += 1,
                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                }
            });
            rsx!(div { class: "desk-note",
                span { class: "mname", "{marker.name}" }
                // 欄は押した間だけ在る。Enter・Escape・外を押す、のどれでも欄ごと消える。
                if editing {
                    Field {
                        label: "Note",
                        session: session.clone(),
                        class: "mbody",
                        multiline: true,
                        revision,
                        oncommit: move |f: OpenField| {
                            let FieldAt::Note(at) = f.at else { return };
                            if f.draft == unchanged {
                                return;
                            }
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
                            opener.open_field(FieldAt::Note(at), body.clone());
                            *revision.write() += 1;
                        },
                        if marker.body.is_empty() { "Add a note" } else { "{marker.body}" }
                    }
                    if let (Some(mut send), false) = (to_layer, marker.body.is_empty()) {
                        SemanticButton {
                            class: "chip tolayer",
                            aria_label: "Send note to the selected text layer",
                            onclick: move |evt| send(evt),
                            "Send to layer"
                        }
                    }
                }
            })
        }
        None => rsx!(div { class: "desk-note idle", span { class: "mname", "{NO_MARKER}" } }),
    });

    let drawer_body = drawer.map(|drawer| match drawer {
        Drawer::Text => note.clone().unwrap_or_else(|| rsx! {}),
        other => drawer_body(
            &session,
            other,
            revision,
            on_history,
            shown_blend.clone(),
            blend_roving,
            history_roving,
            &blend_scroll,
            &history_scroll,
        ),
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
        div {
            class: "desk-foot",
            role: "toolbar",
            onkeydown: {
                let keys = DRAWERS
                    .iter()
                    .map(|(drawer, _)| format!("desk:drawer:{drawer:?}"))
                    .collect::<Vec<_>>();
                move |evt: KeyboardEvent| {
                    drawer_roving.key(&evt, &keys);
                }
            },
            for (index, (which , label)) in DRAWERS.iter().copied().enumerate() {
                SemanticButton {
                    class: if drawer == Some(which) { "chip on" } else { "chip" },
                    selected: drawer == Some(which),
                    aria_label: "Open {label}",
                    tabindex: Some(drawer_roving.tabindex(index)),
                    focus_key: Some(format!("desk:drawer:{which:?}")),
                    onclick: {
                        let session = session.clone();
                        move |_| {
                            let mut cursor = drawer_roving.cursor;
                            cursor.set(index);
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
        div { class: "desk-face",
        span { class: "mname", "{face_name}" }
        div {
            class: if refs.is_empty() { "desk-refs empty" } else { "desk-refs" },
            onmounted: {
                let refs_scroll = refs_scroll.clone();
                move |evt: MountedEvent| *refs_scroll.borrow_mut() = Some(evt.data())
            },
            onkeydown: {
                let refs_scroll = refs_scroll.clone();
                move |evt: KeyboardEvent| { page_key(&evt, &refs_scroll); }
            },
            if refs.is_empty() {
                "Drop reference images here"
            }
            for (id , name , uri) in refs.iter() {
                if let Some(uri) = uri {
                    div { class: "refi",
                        img { class: "ref", src: "{uri}", alt: "{name}" }
                        SemanticButton {
                            class: "chip refx",
                            aria_label: "Remove {name}",
                            onclick: {
                                let doc = session.doc.clone();
                                let id = *id;
                                move |_| {
                                    crate::ui::session::noted(doc.lock().unwrap().apply(Intent::RemoveAsset { asset: id }), revision)
                                }
                            },
                            "×"
                        }
                    }
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
    let preset_focus = use_hook(|| Arc::new(Mutex::new(0usize)));
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
                .with_focus(preset_focus.clone())
                .with_name_mirror(kind_name),
        )
    });
    crate::ui::ease::ease_panel(
        &session,
        editor,
        kinds,
        kind_name,
        shape,
        preset_focus,
        revision,
    )
}

/// 引き出しの中身。型に一つ。
fn drawer_body(
    session: &Session,
    drawer: Drawer,
    revision: Signal<u32>,
    on_history: EventHandler<i32>,
    shown_blend: std::rc::Rc<std::cell::Cell<Option<crate::doc::store::LayerId>>>,
    blend_roving: Roving,
    history_roving: Roving,
    blend_scroll: &MountedSlot,
    history_scroll: &MountedSlot,
) -> Element {
    let mut revision = revision;
    match drawer {
        Drawer::Key => rsx!(EaseDrawer {
            session: session.clone(),
            revision
        }),
        Drawer::Blend => {
            let target = blend_target(session).and_then(|layer| {
                let d = session.doc.lock().unwrap();
                d.view()
                    .attrs(layer)
                    .ok()
                    .flatten()
                    .map(|a| (layer, a.blend_mode))
            });
            match target {
                None => rsx!(div { class: "dempty", "Select a layer to change its blend" }),
                // 合成は線形光(裁定 498)。AE の既定(ガンマ)とは Multiply / Screen の絵が違う。将来の切替点はここ。
                Some((layer, current)) => {
                    let tint = blend_tint(&session, layer);
                    shown_blend.set(Some(layer));
                    let keys = BLEND_MODES
                        .iter()
                        .map(|(mode, _)| format!("desk:blend:{mode:?}"))
                        .collect::<Vec<_>>();
                    rsx!(div { class: "dnote", "Blend preview · linear light" } div {
                        class: "blend-grid",
                        role: "toolbar",
                        aria_label: "Blend modes",
                        onmounted: {
                            let blend_scroll = blend_scroll.clone();
                            move |evt: MountedEvent| *blend_scroll.borrow_mut() = Some(evt.data())
                        },
                        onkeydown: {
                            let keys = keys.clone();
                            let blend_scroll = blend_scroll.clone();
                            move |evt: KeyboardEvent| {
                                if !page_key(&evt, &blend_scroll) {
                                    blend_roving.key(&evt, &keys);
                                }
                            }
                        },
                        for (index, (mode , label)) in BLEND_MODES.iter().copied().enumerate() {
                            SemanticButton {
                                class: if mode == current { "blend-cell on" } else { "blend-cell" },
                                selected: mode == current,
                                aria_label: "Blend {label}",
                                tabindex: Some(blend_roving.tabindex(index)),
                                focus_key: Some(keys[index].clone()),
                                // hover で Stage が下見(§4)。blend は property なので transient で足りる。
                                onmouseenter: {
                                    let doc = session.doc.clone();
                                    move |_| {
                                        doc.lock().unwrap().set_transient(
                                            layer,
                                            crate::doc::store::PropertyId::blend_mode(),
                                            crate::doc::eval::Value::Enum(mode.to_enum_value()),
                                        );
                                        *revision.write() += 1;
                                    }
                                },
                                onmouseleave: {
                                    let doc = session.doc.clone();
                                    move |_| {
                                        doc.lock().unwrap().clear_transient(layer, &crate::doc::store::PropertyId::blend_mode());
                                        *revision.write() += 1;
                                    }
                                },
                                onclick: {
                                    let doc = session.doc.clone();
                                    move |_| {
                                        let mut cursor = blend_roving.cursor;
                                        cursor.set(index);
                                        doc.lock().unwrap().clear_transient(layer, &crate::doc::store::PropertyId::blend_mode());
                                        match write_blend(&doc, layer, mode) {
                                            Ok(_) => *revision.write() += 1,
                                            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                        }
                                    }
                                },
                                // 札の絵は実際にその式で混ざった色(暗・明・青・暖の 4 つの下地)。
                                span { class: "blend-swatch",
                                    for bed in crate::ui::blend_preview::BEDS {
                                        span { class: "blend-px", style: "background: {crate::ui::blend_preview::css(crate::ui::blend_preview::blend(mode, tint, bed))};" }
                                    }
                                }
                                span { class: "blend-name", "{label}" }
                            }
                        }
                    })
                }
            }
        }
        Drawer::Text => rsx! {},
        Drawer::History => {
            let (back, forward) = session.doc.lock().unwrap().history_depth();
            let steps = (1..=back)
                .rev()
                .map(|step| -(step as i32))
                .chain((1..=forward).map(|step| step as i32))
                .collect::<Vec<_>>();
            let keys = steps
                .iter()
                .map(|step| format!("desk:history:{step}"))
                .collect::<Vec<_>>();
            let active = (history_roving.cursor)().min(steps.len().saturating_sub(1));
            rsx!(div {
                class: "history-strip",
                role: "toolbar",
                aria_label: "Document history",
                onmounted: {
                    let history_scroll = history_scroll.clone();
                    move |evt: MountedEvent| *history_scroll.borrow_mut() = Some(evt.data())
                },
                onkeydown: {
                    let keys = keys.clone();
                    let history_scroll = history_scroll.clone();
                    move |evt: KeyboardEvent| {
                        if !page_key(&evt, &history_scroll) {
                            history_roving.key(&evt, &keys);
                        }
                    }
                },
                for (index, step) in steps.iter().copied().enumerate() {
                    if step > 0 && index == back {
                        span { class: "hstep now" }
                    }
                    SemanticButton {
                        class: if step < 0 { "hstep past" } else { "hstep ahead" },
                        aria_label: if step < 0 { format!("Back {}", step.unsigned_abs()) } else { format!("Forward {step}") },
                        tabindex: Some(if active == index { "0".to_owned() } else { "-1".to_owned() }),
                        focus_key: Some(keys[index].clone()),
                        onclick: move |_| {
                            let mut cursor = history_roving.cursor;
                            cursor.set(index);
                            on_history.call(step);
                        },
                    }
                }
                if forward == 0 {
                    span { class: "hstep now" }
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::RationalTime;

    fn roving_fixture() -> Element {
        let items = use_context_provider(FocusableItems::new);
        let roving = Roving {
            items,
            cursor: use_signal(|| 0usize),
        };
        let keys = (0..4)
            .map(|index| format!("test:roving:{index}"))
            .collect::<Vec<_>>();
        rsx!(div {
            class: "test-roving",
            style: "display:grid;grid-template-columns:repeat(2,40px);gap:4px;",
            onkeydown: {
                let keys = keys.clone();
                move |evt: KeyboardEvent| { roving.key(&evt, &keys); }
            },
            for index in 0..4 {
                SemanticButton {
                    class: "test-cell",
                    tabindex: Some(roving.tabindex(index)),
                    focus_key: Some(keys[index].clone()),
                    onclick: move |_| {
                        let mut cursor = roving.cursor;
                        cursor.set(index);
                    },
                    "{index}"
                }
            }
        })
    }

    #[test]
    fn desk_grid_arrows_and_home_end_move_one_roving_focus() {
        let mut gui = blitz_test_harness::Harness::from_component(roving_fixture);
        gui.click(".test-cell");
        crate::ui::semantic_menu::flush_dom_work();
        gui.pump();
        let cells = gui.query_all(".test-cell");
        assert_eq!(gui.focused(), cells.first().copied());
        gui.press(Key::ArrowRight);
        crate::ui::semantic_menu::flush_dom_work();
        gui.pump();
        assert_eq!(gui.focused(), cells.get(1).copied());
        gui.press(Key::ArrowDown);
        crate::ui::semantic_menu::flush_dom_work();
        gui.pump();
        assert_eq!(gui.focused(), cells.get(3).copied());
        gui.press(Key::Home);
        crate::ui::semantic_menu::flush_dom_work();
        gui.pump();
        assert_eq!(gui.focused(), cells.first().copied());
        gui.press(Key::End);
        crate::ui::semantic_menu::flush_dom_work();
        gui.pump();
        assert_eq!(gui.focused(), cells.last().copied());
        assert_eq!(
            gui.attr(".test-cell:last-child", "tabindex").as_deref(),
            Some("0")
        );
    }

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
    fn the_note_under_edit_does_not_follow_the_playhead() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        let markers = [marker(0), marker(1)];
        assert_eq!(note_index(&session, &markers, 1.5), Some(1));
        session.open_field(FieldAt::Note(markers[0].time), String::new());
        assert_eq!(note_index(&session, &markers, 1.5), Some(0));
        session.open_field(
            FieldAt::Note(RationalTime::try_new(7, 1).unwrap()),
            String::new(),
        );
        assert_eq!(
            note_index(&session, &markers, 1.5),
            Some(1),
            "a stale note must not panic the desk"
        );
    }

    #[test]
    fn the_magpie_has_one_eye_per_frame() {
        for rows in [STAND, SINK] {
            let eyes = rows.iter().filter(|row| row.contains('C')).count();
            assert!(
                eyes >= 1,
                "a frame without a motif pixel has no eye to blink"
            );
        }
    }
}
