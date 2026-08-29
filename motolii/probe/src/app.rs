use std::sync::Arc;

use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::browser::browser_panel;
use crate::fixture::{load_fixture, Loaded};
use crate::inspector::inspector_panel;
use crate::session::Session;
use crate::stage_widget::StageWidget;
use crate::timeline_shell::timeline_shell;
use crate::timeline_widget::TimelineWidget;
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
    let playing = use_signal(|| false);
    let mut browser_w = use_signal(|| 300.0f64);
    let mut inspector_w = use_signal(|| 270.0f64);
    let mut timeline_h = use_signal(|| 300.0f64);
    let mut drag = use_signal(|| Option::<DragSplit>::None);

    let mut scale_pct = use_signal(|| 100u32);

    let (clock, ui_scale, timeline_attr, timeline_tx, stage_attr, loaded, doc, _selection) = use_hook(|| {
        let Loaded { doc, ui, duration_sec } = load_fixture();
        let session = Session::new(doc, duration_sec);
        let Session { doc, clock, scale: ui_scale, selection } = session;

        let canvas_rows = fixture::canvas_rows_from_doc(&doc.lock().unwrap());
        let timeline = TimelineWidget::new(canvas_rows)
            .with_clock(clock.clone())
            .with_scale(ui_scale.clone())
            .with_document(doc.clone(), fixture::canvas_rows_from_doc);
        let timeline_tx = timeline.sender();
        let stage = StageWidget::new(clock.clone(), doc.clone());
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

                {browser_panel(&loaded, doc.clone(), clock.clone(), layer_rows, attrs_state, timeline_tx.clone())}

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

                {inspector_panel(&loaded.inspector)}
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

            {timeline_shell(clock.clone(), playing, doc.clone(), attrs_state, &layer_rows.read(), timeline_attr)}

            div { id: "status", "{loaded.status}" }
        }
    )
}
