use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;

use crate::ui::browser::browser_panel;
use crate::ui::dock::{Dock, Panel, Zone};
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

/// 窓1枚ぶんの見えかたの状態。Document には入らない物だけ。
#[derive(Clone, Copy)]
struct Panes {
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    revision: Signal<u32>,
    text_editing: Signal<Option<String>>,
    renaming: Signal<Option<(crate::doc::store::LayerId, String)>>,
    scroll_y: Signal<f64>,
    /// 他の窓が書いた時に上がる。状態は全窓で1つなので、これで描き直す。
    echo: Signal<u32>,
}

fn panes_for(ui: &fixture::UiData) -> Panes {
    Panes {
        layer_rows: use_signal(|| ui.layer_rows.clone()),
        attrs_state: use_signal(|| {
            ui.layer_rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect::<Vec<_>>()
        }),
        selected: use_signal(|| None),
        revision: use_signal(|| 0u32),
        text_editing: use_signal(|| None),
        renaming: use_signal(|| None),
        scroll_y: use_signal(|| 0.0f64),
        echo: use_signal(|| 0u32),
    }
}

/// 窓をまたいだ描き直しを繋ぐ。自分が書いたら他の窓を起こし、他の窓に起こされたら
/// 自分を描き直す。起こされた側は `echo` だけが動くので、起こし返しは起きない。
fn wire_windows(host: &crate::ui::host::Host, panes: Panes) {
    let me = use_hook(|| {
        let runtime = dioxus_core::Runtime::current();
        let scope = dioxus_core::current_scope_id();
        let mut echo = panes.echo;
        host.listen(move || {
            runtime.in_scope(scope, move || *echo.write() += 1);
        })
    });
    let host = host.clone();
    use_effect(move || {
        // 書き込みの合図。選択は revision を上げない経路なので別に見る。
        let _ = (panes.revision)();
        let _ = (panes.selected)();
        host.wake_others(me);
    });
}

/// パネル1枚の中身。窓が変わっても同じ物を出す。
fn panel_body(panel: Panel, session: &Session, ui: &fixture::UiData, p: Panes) -> Element {
    let _ = (p.echo)();
    let selected = session.selection.get();
    match panel {
        Panel::Media | Panel::Effects | Panel::Create | Panel::Colors => rsx!(BrowserPanel {
            session: session.clone(),
            panel,
            layer_rows: p.layer_rows,
            attrs_state: p.attrs_state,
            selected: p.selected,
            revision: p.revision,
        }),
        Panel::Stage => rsx!(StagePanel {
            session: session.clone(),
            selected: p.selected,
            revision: p.revision,
            comp_line: ui.comp_line.clone(),
        }),
        Panel::Inspector => rsx!(InspectorPanel {
            session: session.clone(),
            selected,
            revision: p.revision,
            editing: p.text_editing,
        }),
        Panel::Utility => rsx!(UtilityPanel {
            session: session.clone(),
            selected,
            revision: p.revision,
        }),
        Panel::Ease => rsx!(EasePanel {
            session: session.clone(),
            revision: p.revision,
        }),
        Panel::Timeline => rsx!(TimelinePanel {
            session: session.clone(),
            layer_rows: p.layer_rows,
            attrs_state: p.attrs_state,
            selected: p.selected,
            scroll_y: p.scroll_y,
            renaming: p.renaming,
            revision: p.revision,
        }),
    }
}

/// 別窓。パネル1枚だけを出す。状態は窓をまたいで1つ(Session)。
pub fn detached() -> Element {
    let session = use_hook(|| consume_context::<Session>());
    let ui = session.ui.clone();
    let panel = use_hook(|| consume_context::<Panel>());
    let host = use_hook(|| consume_context::<crate::ui::host::Host>());
    let panes = panes_for(&ui);
    wire_windows(&host, panes);
    println!("PROBE room=detached verdict=render panel={panel}");
    let css = format!("{}{}", tokens::css_root(100), STYLES);
    rsx!(
        style { {css} }
        div { id: "detached", {panel_body(panel, &session, &ui, panes)} }
    )
}

#[component]
fn BrowserPanel(
    session: Session,
    panel: Panel,
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    revision: Signal<u32>,
) -> Element {
    let rail = use_signal(|| Option::<fixture::AssetFamily>::None);
    browser_panel(
        &session.ui,
        session.doc.clone(),
        session.clock.clone(),
        layer_rows,
        attrs_state,
        session.timeline_tx.clone(),
        selected,
        revision,
        panel,
        rail,
    )
}

#[component]
fn InspectorPanel(
    session: Session,
    selected: Option<crate::doc::store::LayerId>,
    revision: Signal<u32>,
    editing: Signal<Option<String>>,
) -> Element {
    let drag = use_signal(|| None);
    let blend_open = use_signal(|| false);
    let parent_open = use_signal(|| false);
    inspector_panel(
        &session.doc,
        selected,
        &session.clock,
        revision,
        editing,
        drag,
        blend_open,
        parent_open,
    )
}

#[component]
fn EasePanel(session: Session, revision: Signal<u32>) -> Element {
    let shape = use_hook(|| {
        std::sync::Arc::new(std::sync::Mutex::new(crate::ui::ease_widget::DEFAULT))
    });
    let editor = use_hook(|| {
        CustomWidgetAttr::new(crate::ui::ease_widget::EaseWidget::new(
            shape.clone(),
            session.clone(),
        ))
    });
    let kinds = use_hook(|| {
        CustomWidgetAttr::new(crate::ui::ease_widget::KindsWidget::new(
            shape.clone(),
            session.clone(),
        ))
    });
    crate::ui::ease::ease_panel(&session, editor, kinds, revision)
}

#[component]
fn UtilityPanel(
    session: Session,
    selected: Option<crate::doc::store::LayerId>,
    revision: Signal<u32>,
) -> Element {
    crate::ui::utility::utility_panel(
        &session.doc,
        selected,
        &session.selected_size,
        &session.clock,
        revision,
    )
}

/// ウィジェットはコンポーネントの中で作る。置き場を移すと要素が作り直されるので、
/// `CustomWidgetAttr` を app と共有すると2枚目が空になる(中身は一度しか渡せない)。
#[component]
fn StagePanel(
    session: Session,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    revision: Signal<u32>,
    comp_line: String,
) -> Element {
    let attr = use_hook(|| {
        CustomWidgetAttr::new(StageWidget::new(
            session.clock.clone(),
            session.doc.clone(),
            session.selection.clone(),
            selected,
            revision,
            session.selected_size.clone(),
            session.view_camera.clone(),
            session.rings.clone(),
        ))
    });
    let rings = session.rings.clone();
    let mut rings_on = use_signal(|| rings.load(std::sync::atomic::Ordering::Relaxed));
    rsx!(
        div { id: "stagecol",
            div { id: "stage",
                object { "data": attr }
            }
            div { id: "stagefoot",
                div {
                    class: if rings_on() { "chip on" } else { "chip" },
                    onclick: move |_| {
                        let next = !rings_on();
                        rings.store(next, std::sync::atomic::Ordering::Relaxed);
                        rings_on.set(next);
                        revision += 1;
                    },
                    "◎"
                }
                span { "{comp_line}" }
            }
        }
    )
}

#[component]
#[allow(clippy::too_many_arguments)]
fn TimelinePanel(
    session: Session,
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    scroll_y: Signal<f64>,
    renaming: Signal<Option<(crate::doc::store::LayerId, String)>>,
    revision: Signal<u32>,
) -> Element {
    let attr = use_hook(|| {
        let rows = fixture::canvas_rows_from_doc(&session.doc.lock().unwrap());
        CustomWidgetAttr::new(
            TimelineWidget::new(rows, session.timeline_rx.clone())
                .with_clock(session.clock.clone())
                .with_scale(session.scale.clone())
                .with_document(session.doc.clone(), fixture::canvas_rows_from_doc)
                .with_selection(session.selection.clone(), selected)
                .with_scroll_mirror(scroll_y)
                .with_key_mirror(session.selected_keys.clone()),
        )
    });
    timeline_shell(
        session.doc.clone(),
        attrs_state,
        &layer_rows.read(),
        layer_rows,
        attr,
        session.selection.clone(),
        selected,
        scroll_y,
        session.timeline_tx.clone(),
        renaming,
        revision,
    )
}

pub fn app() -> Element {
    let mut playing = use_signal(|| false);
    let mut browser_w = use_signal(|| 300.0f64);
    let mut inspector_w = use_signal(|| 270.0f64);
    let mut timeline_h = use_signal(|| 300.0f64);
    let mut drag = use_signal(|| Option::<DragSplit>::None);
    let mut dock = use_signal(Dock::default);
    let mut tab_drag = use_signal(|| Option::<Panel>::None);
    let mut drop_zone = use_signal(|| Option::<Zone>::None);
    let mut view_open = use_signal(|| false);
    let mut scale_pct = use_signal(|| 100u32);

    let session = use_hook(|| consume_context::<Session>()).clone();
    let loaded = session.ui.clone();
    let host = use_hook(|| consume_context::<crate::ui::host::Host>());
    // 別窓が閉じたら置き場へ戻す。別の窓からの合図なので、自分の runtime を包んで渡す。
    use_hook(|| {
        let runtime = dioxus_core::Runtime::current();
        let scope = dioxus_core::current_scope_id();
        host.on_close(move |panel| {
            let mut dock = dock;
            runtime.in_scope(scope, move || dock.write().reattach(panel));
        });
    });
    let panes = panes_for(&loaded);
    wire_windows(&host, panes);
    let Panes {
        layer_rows,
        attrs_state,
        selected: selected_sig,
        revision,
        text_editing,
        ..
    } = panes;
    let mut selected = selected_sig;

    let timeline_tx = session.timeline_tx.clone();
    let doc = session.doc.clone();
    let clock = session.clock.clone();
    let ui_scale = session.scale.clone();
    let selection = session.selection.clone();

    let bw = browser_w();
    let iw = inspector_w();
    let th = timeline_h();
    let css = format!("{}{}", tokens::css_root(scale_pct()), STYLES);

    let d = dock();
    // 掴んでいる間は空の置き場も開けておく。畳んだままだと戻す場所が無くなる。
    let holding = tab_drag().is_some();
    let px = |zone: Zone, v: f64| {
        let filled = !d.panels(zone).is_empty();
        match (filled, holding) {
            (true, _) => format!("{v}px"),
            (false, true) => format!("{}px", v.min(96.0)),
            (false, false) => "0px".to_string(),
        }
    };
    let left_w = px(Zone::Left, bw);
    let right_w = px(Zone::Right, iw);
    let grip_l = px(Zone::Left, 8.0);
    let grip_r = px(Zone::Right, 8.0);
    let bottom_h = px(Zone::Bottom, th);
    let grip_b = px(Zone::Bottom, 8.0);

    let body = |panel: Panel| -> Element { panel_body(panel, &session, &loaded, panes) };
    let zone_view = |zone: Zone| -> Element {
        let d = dock();
        let panels = d.panels(zone).to_vec();
        let dropping = tab_drag().is_some();
        let here = dropping && drop_zone() == Some(zone);
        let strip_class = if here { "ptabs drop" } else { "ptabs" };
        let zone_class = match (panels.is_empty(), here) {
            (_, true) => "zone here",
            (true, false) => "zone empty",
            (false, false) => "zone",
        };
        rsx!(
            div {
                class: "{zone_class}",
                onmousemove: move |_| {
                    if tab_drag.peek().is_some() && *drop_zone.peek() != Some(zone) {
                        drop_zone.set(Some(zone));
                    }
                },
                onmouseup: move |_| {
                    if let Some(panel) = tab_drag.write().take() {
                        dock.write().place(panel, zone);
                    }
                },
                div { class: "{strip_class}",
                    for panel in panels.iter().copied() {
                        span {
                            class: if d.is_active(zone, panel) { "ptab on" } else { "ptab" },
                            style: if d.is_active(zone, panel) { format!("border-bottom-color: {};", panel.way()) } else { String::new() },
                            onmousedown: move |_| {
                                *tab_drag.write() = Some(panel);
                                dock.write().set_active(zone, panel);
                            },
                            "{panel}"
                        }
                    }
                    if d.active(zone) == Some(Panel::Timeline) {
                        {crate::ui::timeline_shell::transport(clock.clone(), playing, layer_rows.read().len())}
                    }
                }
                if let Some(panel) = d.active(zone) {
                    div { class: "zbody", {body(panel)} }
                } else if dropping {
                    div { class: "zhint", "Drop here" }
                }
            }
        )
    };

    rsx!(
        style { {css} }
        div {
            id: "app",
            tabindex: "0",
            autofocus: "true",
            style: "grid-template-rows: var(--section) 1fr {grip_b} {bottom_h} calc(20 * var(--s) * 1px);",
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
                *tab_drag.write() = None;
                *drop_zone.write() = None;
            },
            onmouseleave: {
                let host = host.clone();
                move |_| {
                    let Some(panel) = tab_drag.write().take() else { return };
                    println!("PROBE room=dock verdict=detach panel={panel}");
                    *drop_zone.write() = None;
                    dock.write().detach(panel);
                    host.open(panel);
                }
            },
            onkeyup: move |evt: dioxus_native::prelude::Event<dioxus_native::prelude::KeyboardData>| {
                crate::ui::keymap::note_key_up(&evt.key());
            },
            onfocusout: move |_| crate::ui::keymap::forget_modifiers(),
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
                    crate::ui::keymap::note_key_down(&evt.key());
                    if text_editing.read().is_some() {
                        println!("PROBE room=input verdict=text-editing key={:?}", evt.key());
                        return;
                    }
                    let modifiers = evt.modifiers();
                    let primary = if cfg!(target_os = "macos") { modifiers.meta() } else { modifiers.ctrl() };
                    let Some(intent) = crate::ui::keymap::lookup_held(
                        &evt.key(),
                        primary,
                        modifiers.shift(),
                        modifiers.alt(),
                    ) else {
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
                        Intent::Duplicate => {
                            let Some(layer) = selected() else { return };
                            let Some(copy) =
                                crate::ui::timeline_widget::duplicate_layer(&doc, layer)
                            else {
                                return;
                            };
                            let d = doc.lock().unwrap();
                            let rows = fixture::layer_rows_from_doc(&d);
                            let canvas = fixture::canvas_rows_from_doc(&d);
                            drop(d);
                            attrs_state.set(rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect());
                            layer_rows.set(rows);
                            let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                            selection.set(Some(copy));
                            selected.set(Some(copy));
                            *revision.write() += 1;
                            println!("PROBE room=write verdict=applied Duplicate layer={copy:?}");
                        }
                        Intent::EasyEase(side) => {
                            let starts = crate::ui::ease::segments(
                                &session.selected_keys.lock().unwrap(),
                            );
                            if starts.is_empty() {
                                return;
                            }
                            let shape = crate::ui::ease::easy_ease(side);
                            match crate::ui::ease::apply(&session, &starts, shape) {
                                Ok(n) => println!(
                                    "PROBE room=write verdict=applied EasyEase tracks={n}"
                                ),
                                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                            }
                            *revision.write() += 1;
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
                span {
                    class: if view_open() { "menu on" } else { "menu" },
                    onclick: move |_| {
                        let open = view_open();
                        view_open.set(!open);
                    },
                    "View"
                    if view_open() {
                        div { class: "vmenu",
                            for panel in Panel::all() {
                                div { class: "vrow",
                                    span {
                                        class: if d.is_visible(panel) { "vitem on" } else { "vitem" },
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            dock.write().toggle(panel);
                                        },
                                        if d.is_visible(panel) { "✓ " } else { "  " }
                                        "{panel}"
                                    }
                                    span {
                                        class: if d.is_detached(panel) { "vout on" } else { "vout" },
                                        onclick: {
                                            let host = host.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                if dock.peek().is_detached(panel) {
                                                    return;
                                                }
                                                dock.write().detach(panel);
                                                host.open(panel);
                                            }
                                        },
                                        "Window"
                                    }
                                }
                            }
                        }
                    }
                }
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
                style: "grid-template-columns: {left_w} {grip_l} 1fr {grip_r} {right_w};",

                {zone_view(Zone::Left)}
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
                {zone_view(Zone::Center)}
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
                {zone_view(Zone::Right)}
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

            {zone_view(Zone::Bottom)}

            div { id: "status", "{loaded.status}" }
        }
    )
}

#[cfg(test)]
mod detached_tests {
    use super::*;
    use crate::ui::fixture::{load_fixture, Loaded};
    use blitz_dom::{Document, DocumentConfig};
    use blitz_traits::shell::{ColorScheme, Viewport};
    use dioxus_native::DioxusDocument;

    fn open(panel: Panel) -> DioxusDocument {
        let Loaded { doc, ui, duration_sec } = load_fixture();
        let mut vdom = VirtualDom::new(detached);
        vdom.insert_any_root_context(Box::new(Session::new(doc, duration_sec, ui)));
        vdom.insert_any_root_context(Box::new(panel));
        vdom.insert_any_root_context(Box::new(crate::ui::host::Host::for_tests()));
        let mut doc = DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport: Some(Viewport::new(900, 600, 1.0, ColorScheme::Dark)),
                ..Default::default()
            },
        );
        doc.initial_build();
        doc.inner_mut().resolve(0.0);
        doc
    }

    fn size_of(doc: &DioxusDocument, selector: &str) -> (f32, f32) {
        let inner = doc.inner();
        let node = inner
            .query_selector(selector)
            .ok()
            .flatten()
            .unwrap_or_else(|| panic!("{selector} が居ない"));
        let layout = inner.get_node(node).expect("node").final_layout();
        (layout.size.width, layout.size.height)
    }

    #[test]
    fn a_detached_stage_fills_its_window() {
        let doc = open(Panel::Stage);
        let (w, h) = size_of(&doc, "#stage");
        assert!(w > 100.0 && h > 100.0, "別窓の Stage が潰れている: {w}x{h}");
    }

    #[test]
    fn a_detached_timeline_fills_its_window() {
        let doc = open(Panel::Timeline);
        let (w, h) = size_of(&doc, "#timeline");
        assert!(w > 100.0 && h > 100.0, "別窓のタイムラインが潰れている: {w}x{h}");
    }
}
