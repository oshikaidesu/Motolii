//! 机 — Document を覗くレンズ。誰にも呼ばれない。
//! 常設なのは細い顔(書き置き・取っ手・鳥)。引き出しは机の中に出て、その間だけ机が広がる。

use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::doc::store::{Document, Intent, LayerId, Marker, StoreError};
use crate::ui::inspector::{write_blend, BLEND_MODES};
use crate::ui::semantic_menu::{Field, SemanticButton};
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

/// 机が見せる書き置き。開けている欄が在ればその印(再生が進んでも欄は逃げない)、
/// 無ければ今の時刻の印。
fn note_index(session: &Session, markers: &[Marker], now: f64) -> Option<usize> {
    match session.field().map(|f| f.at) {
        Some(FieldAt::Note(i)) if i < markers.len() => Some(i),
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
    echo: u32,
    revision: Signal<u32>,
    playhead: Signal<f64>,
    on_history: EventHandler<i32>,
) -> Element {
    let _ = revision();
    let _ = playhead();
    let mut revision = revision;

    let now = session.clock.now_sec();
    let markers = session.doc.lock().unwrap().view().markers().unwrap_or_default();
    let current = note_index(&session, &markers, now);
    let drawer = drawer_of(&session);

    let face_name = current
        .map(|i| markers[i].name.clone())
        .unwrap_or_else(|| "No marker yet".to_owned());
    let refs = crate::ui::fixture::reference_images_from_view(&session.doc.lock().unwrap().view());
    let note = drawer.filter(|d| *d == Drawer::Text).map(|_| match current {
        Some(i) => {
            let marker = &markers[i];
            let editing = session.field_at(&FieldAt::Note(i)).is_some();
            let doc = session.doc.clone();
            let body = marker.body.clone();
            let unchanged = marker.body.clone();
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
        div { class: if refs.is_empty() { "desk-refs empty" } else { "desk-refs" },
            if refs.is_empty() {
                "Drop reference images here"
            }
            for (id , name , uri) in refs.iter() {
                if let Some(uri) = uri {
                    div { class: "refi",
                        img { class: "ref", src: "{uri}", alt: "{name}", title: "{name}" }
                        SemanticButton {
                            class: "chip refx",
                            aria_label: "Remove {name}",
                            onclick: {
                                let doc = session.doc.clone();
                                let id = *id;
                                move |_| {
                                    match doc.lock().unwrap().apply(Intent::RemoveAsset { asset: id }) {
                                        Ok(_) => *revision.write() += 1,
                                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                    }
                                }
                            },
                            "×"
                        }
                    }
                }
            }
        }
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
                                    doc.lock().unwrap().clear_transient(layer, &crate::doc::store::PropertyId::blend_mode());
                                    match write_blend(&doc, layer, mode) {
                                        Ok(_) => *revision.write() += 1,
                                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                    }
                                }
                            },
                            span { class: "blend-swatch" }
                            span { class: "blend-name", "{label}" }
                        }
                    }
                }),
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
    fn the_note_under_edit_does_not_follow_the_playhead() {
        let loaded = crate::ui::fixture::load_fixture();
        let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
        let markers = [marker(0), marker(1)];
        assert_eq!(note_index(&session, &markers, 1.5), Some(1));
        session.open_field(FieldAt::Note(0), String::new());
        assert_eq!(note_index(&session, &markers, 1.5), Some(0));
        session.open_field(FieldAt::Note(7), String::new());
        assert_eq!(note_index(&session, &markers, 1.5), Some(1), "a stale note index must not panic the desk");
    }

    #[test]
    fn the_magpie_has_one_eye_per_frame() {
        for rows in [STAND, SINK] {
            let eyes = rows.iter().filter(|row| row.contains('C')).count();
            assert!(eyes >= 1, "a frame without a motif pixel has no eye to blink");
        }
    }
}
