use std::sync::Arc;

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::ui::browser::browser_panel;
use crate::ui::fixture::{load_fixture, Loaded};
use crate::ui::inspector::inspector_panel;
use crate::ui::keymap::{lookup, Intent};
use crate::ui::session::Session;
use crate::ui::stage_widget::StageWidget;
use crate::ui::timeline_shell::timeline_shell;
use crate::ui::timeline_widget::{split_layer, TimelineMsg, TimelineWidget};
use crate::ui::fixture;
use crate::ui::tokens;

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
    let panel_tab = use_signal(|| 0u8);

    let mut scale_pct = use_signal(|| 100u32);
    let revision = use_signal(|| 0u32);
    let mut selected = use_signal(|| None);
    let timeline_scroll_y = use_signal(|| 0.0f64);
    let text_editing = use_signal(|| Option::<String>::None);
    let renaming = use_signal(|| Option::<(crate::doc::store::LayerId, String)>::None);

    let (clock, ui_scale, timeline_attr, timeline_tx, stage_attr, loaded, doc, selection, selected_size) = use_hook(|| {
        let Loaded { doc, ui, duration_sec } = load_fixture();
        let session = Session::new(doc, duration_sec);
        let Session { doc, clock, scale: ui_scale, selection, selected_size } = session;

        let canvas_rows = fixture::canvas_rows_from_doc(&doc.lock().unwrap());
        let timeline = TimelineWidget::new(canvas_rows)
            .with_clock(clock.clone())
            .with_scale(ui_scale.clone())
            .with_document(doc.clone(), fixture::canvas_rows_from_doc)
            .with_selection(selection.clone(), selected)
            .with_scroll_mirror(timeline_scroll_y);
        let timeline_tx = timeline.sender();
        let stage = StageWidget::new(clock.clone(), doc.clone(), selection.clone(), selected, revision, selected_size.clone());
        (
            clock,
            ui_scale,
            CustomWidgetAttr::new(timeline),
            timeline_tx,
            CustomWidgetAttr::new(stage),
            Arc::new(ui),
            doc,
            selection,
            selected_size,
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
                    let Some(intent) = lookup(&evt.key(), modifiers.meta(), modifiers.shift(), modifiers.alt()) else {
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
                        Intent::ToggleKeyedOnly => {
                            fixture::toggle_keyed_only();
                            if fixture::keyed_only() {
                                if let Some(layer) = selected() {
                                    fixture::expand(layer);
                                }
                            }
                            let d = doc.lock().unwrap();
                            let rows = fixture::layer_rows_from_doc(&d);
                            let canvas = fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                            layer_rows.set(rows);
                            let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                            *revision.write() += 1;
                        }
                        Intent::ToggleMarker => {
                            let sec = clock.now_sec();
                            let mut d = doc.lock().unwrap();
                            let mut markers = d.view().markers().unwrap_or_default();
                            let hit = markers
                                .iter()
                                .position(|m| (m.time.as_seconds_f64() - sec).abs() < 0.5 / 30.0);
                            match hit {
                                Some(i) => {
                                    markers.remove(i);
                                }
                                None => {
                                    let frame = (sec * 30.0).round() as i64;
                                    let Ok(time) = crate::doc::store::RationalTime::try_new(frame, 30) else {
                                        return;
                                    };
                                    markers.push(crate::doc::store::Marker {
                                        name: format!("{}", markers.len() + 1),
                                        time,
                                        duration: crate::doc::store::RationalTime::ZERO,
                                    });
                                    markers.sort_by(|a, b| {
                                        a.time.as_seconds_f64().total_cmp(&b.time.as_seconds_f64())
                                    });
                                }
                            }
                            let applied = d
                                .apply(crate::doc::store::Intent::SetMarkers {
                                    markers: markers.clone(),
                                })
                                .is_ok();
                            drop(d);
                            if applied {
                                let secs = markers.iter().map(|m| m.time.as_seconds_f64()).collect();
                                let _ = timeline_tx.send(TimelineMsg::SetMarkers(secs));
                                *revision.write() += 1;
                            }
                        }
                        Intent::JumpMarker(dir) => {
                            let sec = clock.now_sec();
                            let d = doc.lock().unwrap();
                            let markers = d.view().markers().unwrap_or_default();
                            drop(d);
                            let mut times: Vec<f64> =
                                markers.iter().map(|m| m.time.as_seconds_f64()).collect();
                            times.sort_by(f64::total_cmp);
                            let next = if dir < 0 {
                                times.into_iter().rev().find(|t| *t < sec - 1e-6)
                            } else {
                                times.into_iter().find(|t| *t > sec + 1e-6)
                            };
                            if let Some(t) = next {
                                clock.seek(t);
                            }
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
                        Intent::Undo | Intent::Redo => {
                            let mut d = doc.lock().unwrap();
                            let moved = if matches!(intent, Intent::Undo) { d.undo() } else { d.redo() };
                            let rows = fixture::layer_rows_from_doc(&d);
                            let canvas = fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            if moved {
                                attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                                layer_rows.set(rows);
                                let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                                *revision.write() += 1;
                            }
                            println!("PROBE room=write verdict=history moved={moved}");
                        }
                        Intent::DeleteLayer => {
                            let targets = selection.all();
                            if targets.is_empty() {
                                println!("PROBE room=write verdict=delete-noop reason=no-selection");
                                return;
                            }
                            let mut d = doc.lock().unwrap();
                            let intents: Vec<_> = targets
                                .iter()
                                .map(|l| crate::doc::store::Intent::RemoveLayer(*l))
                                .collect();
                            let applied = d.apply_all(intents).is_ok();
                            let rows = fixture::layer_rows_from_doc(&d);
                            let canvas = fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            if applied {
                                selection.set(None);
                                selected.set(None);
                                attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                                layer_rows.set(rows);
                                let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                                *revision.write() += 1;
                            }
                            println!("PROBE room=write verdict=applied RemoveLayer n={} ok={applied}", targets.len());
                        }
                        Intent::Reorder(delta) => {
                            let Some(layer) = selected() else { return };
                            let mut d = doc.lock().unwrap();
                            let current = d.view().meta(layer).ok().flatten().map(|m| m.order).unwrap_or(0);
                            let applied = d
                                .apply(crate::doc::store::Intent::SetOrder {
                                    layer,
                                    order: current.saturating_add(delta),
                                })
                                .is_ok();
                            let rows = fixture::layer_rows_from_doc(&d);
                            let canvas = fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            if applied {
                                attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                                layer_rows.set(rows);
                                let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                                *revision.write() += 1;
                            }
                            println!("PROBE room=write verdict=applied SetOrder layer={layer:?} {current}->{} ok={applied}", current.saturating_add(delta));
                        }
                        Intent::SnapEdgeToPlayhead(tail) | Intent::TrimToPlayhead(tail) => {
                            let trim = matches!(intent, Intent::TrimToPlayhead(_));
                            let Some(layer) = selected() else { return };
                            let frame = (clock.now_sec() * 30.0).round() as i64;
                            let mut d = doc.lock().unwrap();
                            let Some(orig) = d.view().meta(layer).ok().flatten().map(|m| m.timing) else {
                                return;
                            };
                            let timing = crate::ui::timeline_widget::edge_to_frame(orig, frame, tail, trim);
                            let applied = d
                                .apply(crate::doc::store::Intent::SetTiming { layer, timing })
                                .is_ok();
                            let rows = fixture::layer_rows_from_doc(&d);
                            let canvas = fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            if applied {
                                attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                                layer_rows.set(rows);
                                let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                                *revision.write() += 1;
                            }
                            println!(
                                "PROBE room=write verdict=applied {} tail={tail} start {}->{} dur {}->{}",
                                if trim { "TrimToPlayhead" } else { "SnapEdgeToPlayhead" },
                                orig.start, timing.start, orig.duration, timing.duration
                            );
                        }
                        Intent::SelectStep(delta) => {
                            let d = doc.lock().unwrap();
                            let rows = fixture::layer_rows_from_doc(&d);
                            drop(d);
                            if rows.is_empty() {
                                return;
                            }
                            let current = selected()
                                .and_then(|l| rows.iter().position(|r| r.layer == l))
                                .unwrap_or(0) as i32;
                            let next = (current + delta).clamp(0, rows.len() as i32 - 1) as usize;
                            selection.set(Some(rows[next].layer));
                            selected.set(Some(rows[next].layer));
                            *revision.write() += 1;
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

                if panel_tab() == 0 {
                    {inspector_panel(&doc, selected(), &clock, revision, text_editing, panel_tab)}
                } else {
                    {crate::ui::utility::utility_panel(&doc, selected(), &selected_size, &clock, panel_tab, revision)}
                }
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

            {timeline_shell(clock.clone(), playing, doc.clone(), attrs_state, &layer_rows.read(), layer_rows, timeline_attr, selection.clone(), selected, timeline_scroll_y, timeline_tx.clone(), renaming, revision)}

            div { id: "status", "{loaded.status}" }
        }
    )
}
