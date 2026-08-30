use std::sync::Arc;

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::browser::browser_panel;
use crate::fixture::{load_fixture, Loaded};
use crate::inspector::inspector_panel;
use crate::keymap::{lookup, Intent};
use crate::session::Session;
use crate::stage_widget::StageWidget;
use crate::timeline_shell::timeline_shell;
use crate::timeline_widget::{split_layer, TimelineMsg, TimelineWidget};
use crate::fixture;
use crate::tokens;

static STYLES: &str = include_str!("styles.css");

#[derive(Clone, Copy, PartialEq)]
enum DragTarget {
    Browser,
    Inspector,
    Timeline,
}

struct DragSplit {
    target: DragTarget,
    start: f64,
    orig: f64,
}

pub fn app() -> Element {
    let mut playing = use_signal(|| false);
    let mut browser_w = use_signal(|| 300.0f64);
    let mut inspector_w = use_signal(|| 270.0f64);
    let mut timeline_h = use_signal(|| 300.0f64);
    let mut drag = use_signal(|| Option::<DragSplit>::None);

    let mut scale_pct = use_signal(|| 100u32);
    let revision = use_signal(|| 0u32);
    let mut selected = use_signal(|| None);
    let timeline_scroll_y = use_signal(|| 0.0f64);
    let text_editing = use_signal(|| Option::<String>::None);

    let (clock, ui_scale, timeline_attr, timeline_tx, stage_attr, loaded, doc, selection) = use_hook(|| {
        let Loaded { doc, ui, duration_sec } = load_fixture();
        let session = Session::new(doc, duration_sec);
        let Session { doc, clock, scale: ui_scale, selection } = session;

        let canvas_rows = fixture::canvas_rows_from_doc(&doc.lock().unwrap());
        let timeline = TimelineWidget::new(canvas_rows)
            .with_clock(clock.clone())
            .with_scale(ui_scale.clone())
            .with_document(doc.clone(), fixture::canvas_rows_from_doc)
            .with_selection(selection.clone(), selected)
            .with_scroll_mirror(timeline_scroll_y);
        let timeline_tx = timeline.sender();
        let stage = StageWidget::new(clock.clone(), doc.clone(), selection.clone(), selected, revision);
        (
            clock,
            ui_scale,
            CustomWidgetAttr::new(timeline),
            timeline_tx,
            CustomWidgetAttr::new(stage),
            Arc::new(ui),
            doc,
            selection,
        )
    });
    let layer_rows = use_signal(|| loaded.layer_rows.clone());
    let attrs_state = use_signal(|| {
        loaded.layer_rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect::<Vec<_>>()
    });

    let bw = browser_w();
    let iw = inspector_w();
    let th = timeline_h();
    let css = format!("{}{}", tokens::css_root(scale_pct()), STYLES);

    rsx!(
        style { {css} }
        div {
            id: "app",
            tabindex: "0",
            autofocus: "true",
            style: "grid-template-rows: var(--section) 1fr 8px {th}px calc(20 * var(--s) * 1px);",
            onmousemove: move |evt| {
                if let Some(d) = drag.read().as_ref() {
                    let p = evt.data().client_coordinates();
                    match d.target {
                        DragTarget::Browser => {
                            *browser_w.write() = (d.orig + (p.x - d.start)).clamp(140.0, 420.0);
                        }
                        DragTarget::Inspector => {
                            *inspector_w.write() = (d.orig - (p.x - d.start)).clamp(180.0, 420.0);
                        }
                        DragTarget::Timeline => {
                            *timeline_h.write() = (d.orig - (p.y - d.start)).clamp(120.0, 600.0);
                        }
                    }
                }
            },
            onmouseup: move |_| {
                *drag.write() = None;
            },
            onkeydown: {
                let doc = doc.clone();
                let clock = clock.clone();
                let selection = selection.clone();
                let timeline_tx = timeline_tx.clone();
                let mut layer_rows = layer_rows;
                let mut attrs_state = attrs_state;
                let mut revision = revision;
                move |evt| {
                    println!("PROBE room=input verdict=keydown key={:?}", evt.key());
                    if text_editing.read().is_some() {
                        println!("PROBE room=input verdict=text-editing key={:?}", evt.key());
                        return;
                    }
                    let modifiers = evt.modifiers();
                    let Some(intent) = lookup(&evt.key(), modifiers.meta(), modifiers.shift()) else {
                        println!("PROBE room=input verdict=no-binding key={:?}", evt.key());
                        return;
                    };
                    println!("PROBE room=input verdict=intent key={:?}", evt.key());
                    evt.prevent_default();
                    match intent {
                        Intent::Split => {
                            let Some(layer) = selected() else {
                                println!("PROBE room=write verdict=split-noop reason=no-selection");
                                return;
                            };
                            let comp_frame = (clock.now_sec() * 30.0).round() as i64;
                            match split_layer(&doc, layer, comp_frame) {
                                Some(tail) => {
                                    let snapshot = doc.lock().unwrap();
                                    let rows = fixture::layer_rows_from_doc(&snapshot);
                                    let canvas = fixture::canvas_rows_from_doc(&snapshot);
                                    drop(snapshot);
                                    attrs_state.set(
                                        rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect(),
                                    );
                                    layer_rows.set(rows);
                                    let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                                    println!(
                                        "PROBE room=write verdict=applied Split layer={layer:?} tail={tail:?} comp_frame={comp_frame}"
                                    );
                                }
                                None => println!(
                                    "PROBE room=write verdict=split-noop layer={layer:?} comp_frame={comp_frame}"
                                ),
                            }
                        }
                        Intent::StepFrame(delta) => {
                            let frame = (clock.now_sec() * 30.0).round() as i64 + delta;
                            clock.seek(frame as f64 / 30.0);
                        }
                        Intent::Home => clock.seek(0.0),
                        Intent::End => clock.seek(clock.duration),
                        Intent::Deselect => {
                            selection.set(None);
                            selected.set(None);
                        }
                        Intent::PlayPause => {
                            clock.toggle();
                            playing.set(clock.playing());
                        }
                        Intent::SelectAll => {
                            let layers = doc.lock().unwrap().view().layers();
                            for layer in layers {
                                if !selection.contains(layer) {
                                    selection.toggle(layer);
                                }
                            }
                            selected.set(selection.get());
                            *revision.write() += 1;
                        }
                    }
                }
            },

            div { id: "menubar",
                span { class: "appname", "Motolii" }
                span { class: "menu", "File" }
                span { class: "menu", "Edit" }
                span { class: "menu", "Layer" }
                span { class: "menu", "Effect" }
                span { class: "menu", "View" }
                span { class: "menu", "Help" }
                div { class: "zoomctl",
                    span {
                        class: "zbtn",
                        onclick: {
                            let ui_scale = ui_scale.clone();
                            move |_| {
                                ui_scale.set_percent(ui_scale.percent().saturating_sub(1));
                                *scale_pct.write() = ui_scale.percent();
                            }
                        },
                        "−"
                    }
                    span { class: "zval", "{scale_pct()}%" }
                    span {
                        class: "zbtn",
                        onclick: {
                            let ui_scale = ui_scale.clone();
                            move |_| {
                                ui_scale.set_percent(ui_scale.percent() + 1);
                                *scale_pct.write() = ui_scale.percent();
                            }
                        },
                        "+"
                    }
                }
            }

            div {
                id: "main",
                style: "grid-template-columns: {bw}px 8px 1fr 8px {iw}px;",

                {browser_panel(&loaded, doc.clone(), clock.clone(), layer_rows, attrs_state, timeline_tx.clone(), selected, revision)}

                div {
                    class: "vgrip",
                    onmousedown: move |evt| {
                        let p = evt.data().client_coordinates();
                        *drag.write() = Some(DragSplit {
                            target: DragTarget::Browser,
                            start: p.x,
                            orig: browser_w(),
                        });
                    },
                }

                div { id: "stagecol",
                    div { id: "stagehead",
                        span { class: "way", style: "background:var(--way-stage);" }
                        "Stage"
                        em { "{loaded.comp_line}" }
                    }
                    div { id: "stage",
                        object { "data": stage_attr }
                    }
                }

                div {
                    class: "vgrip",
                    onmousedown: move |evt| {
                        let p = evt.data().client_coordinates();
                        *drag.write() = Some(DragSplit {
                            target: DragTarget::Inspector,
                            start: p.x,
                            orig: inspector_w(),
                        });
                    },
                }

                {inspector_panel(&doc, selected(), &clock, revision, text_editing)}
            }

            div {
                class: "hgrip",
                onmousedown: move |evt| {
                    let p = evt.data().client_coordinates();
                    *drag.write() = Some(DragSplit {
                        target: DragTarget::Timeline,
                        start: p.y,
                        orig: timeline_h(),
                    });
                },
            }

            {timeline_shell(clock.clone(), playing, doc.clone(), attrs_state, &layer_rows.read(), timeline_attr, selection.clone(), selected, timeline_scroll_y, timeline_tx.clone())}

            div { id: "status", "{loaded.status}" }
        }
    )
}
