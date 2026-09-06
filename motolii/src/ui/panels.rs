use crate::ui::browser::{browser_panel, BrowserMarquee};
use crate::ui::context_menu::MenuRequest;
use crate::ui::dock::Panel;
use crate::ui::fixture;
use crate::ui::inspector::{inspector_panel, ChoiceId};
use crate::ui::mount::SurfaceView;
use crate::ui::semantic_menu::{FocusableItems, SemanticButton};
use crate::ui::session::Session;
use crate::ui::stage_widget::{StageBindings, StageState};
use crate::ui::timeline_shell::timeline_shell;
use crate::ui::timeline_widget::{TimelineBindings, TimelineMsg, TimelineState};
use dioxus_native::prelude::*;

#[component]
pub(super) fn BrowserPanel(
    session: Session,
    echo: u32,
    panel: Panel,
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    revision: Signal<u32>,
    menu: Signal<Option<MenuRequest>>,
) -> Element {
    let rail = use_signal(|| Option::<fixture::AssetFamily>::None);
    let search_node = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut marquee = use_signal(|| None::<BrowserMarquee>);
    let cancel_seen = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(0u32)));
    let mut seen = cancel_seen.get();
    if session.gesture.cancelled(&mut seen) {
        cancel_seen.set(seen);
        marquee.set(None);
    }
    if !matches!(panel, Panel::Media | Panel::Effects) && marquee.peek().is_some() {
        session.gesture.end();
        marquee.set(None);
    }
    let _focusable_items = use_context_provider(FocusableItems::new);
    let poke = consume_context::<crate::ui::host::Host>().poker();
    browser_panel(
        &session,
        session.doc.clone(),
        layer_rows,
        attrs_state,
        session.timeline_tx.clone(),
        selected,
        revision,
        echo,
        panel,
        rail,
        poke,
        search_node,
        marquee,
        menu,
    )
}

#[component]
pub(super) fn InspectorPanel(
    session: Session,
    echo: u32,
    selected: Option<crate::doc::store::LayerId>,
    revision: Signal<u32>,
    playhead: Signal<f64>,
    choice_open: Signal<Option<ChoiceId>>,
) -> Element {
    inspector_panel(
        &session.doc,
        selected,
        &session.clock,
        revision,
        &session,
        choice_open,
        playhead,
        &session.selected_bounds,
        &session.focus,
        session.live_focus(),
    )
}

#[component]
pub(super) fn StagePanel(
    session: Session,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    revision: Signal<u32>,
    comp_line: String,
    menu: Signal<Option<MenuRequest>>,
) -> Element {
    let _ = revision();
    let view_pct = use_signal(|| 100u32);
    let host = consume_context::<crate::ui::host::Host>();
    let handle = host.mounts.get(surface_window(), "stage", || {
        StageState::new(
            session.clock.clone(),
            session.doc.clone(),
            session.selection.clone(),
            session.selected_bounds.clone(),
            session.view_camera.clone(),
            session.rings.clone(),
            session.frame_dim.clone(),
            session.gesture.clone(),
            session.surface_capture.clone(),
            session.output_only.clone(),
            session.view_request.clone(),
        )
    });
    let bindings = StageBindings {
        selected,
        revision,
        view_pct,
        context_menu: menu,
    };
    let rings = session.rings.clone();
    let mut rings_on = use_signal(|| rings.load(std::sync::atomic::Ordering::Relaxed));
    let stage_items = {
        let doc = session.doc.lock().unwrap();
        let view = doc.view();
        let mut layers = view.layers();
        layers.sort_by_key(|layer| {
            std::cmp::Reverse(
                view.meta(*layer)
                    .ok()
                    .flatten()
                    .map(|meta| meta.order)
                    .unwrap_or(0),
            )
        });
        let solo = layers.iter().any(|layer| {
            view.attrs(*layer)
                .ok()
                .flatten()
                .is_some_and(|attrs| attrs.solo && !attrs.hidden)
        });
        layers
            .into_iter()
            .filter_map(|layer| {
                let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
                let meta = view.meta(layer).ok().flatten()?;
                (!attrs.hidden
                    && (!solo || attrs.solo)
                    && meta.timing.covers(session.clock.current_frame()))
                .then_some((layer, attrs.name, attrs.locked))
            })
            .collect::<Vec<_>>()
    };
    let stage_item_count = stage_items.len();
    let selected_layers = session.selection.all();
    rsx!(
        div { id: "stagecol",
            // 行は常に 3 つ(grid の行数を揺らさない)。層が在れば空の 0px 行。
            if session.doc.lock().unwrap().view().layers().is_empty() {
                // 最初の問いは 1 つ「曲は?」。答えは操作そのもの(落とす)。比率は 3 つの chip。
                div { class: "stagehint",
                    span { "Drop a song or video here to begin · or add a Text layer from Create" }
                    for (label , w , h) in [("16:9", 1920u32, 1080u32), ("9:16", 1080, 1920), ("1:1", 1080, 1080)] {
                        SemanticButton {
                            class: "chip",
                            title: "Set the frame to {label}",
                            onclick: {
                                let session = session.clone();
                                move |_| {
                                    let mut d = session.doc.lock().unwrap();
                                    if let Ok(Some(comp)) = d.view().composition() {
                                        let next = crate::doc::store::Composition { width: w, height: h, ..comp };
                                        crate::ui::session::noted(&session.project_notice, d.apply(crate::doc::store::Intent::SetComposition(next)), revision);
                                    }
                                    drop(d);
                                    *revision.write() += 1;
                                }
                            },
                            "{label}"
                        }
                    }
                }
            } else {
                div { class: "stagehint empty" }
            }
            div { id: "stage", class: "surface-stack",
                SurfaceView::<StageState> { handle, bindings, label: "Stage canvas" }
                div { class: "canvas-a11y", role: "listbox", aria_label: "Visible Stage layers",
                    for (index, (layer, name, locked)) in stage_items.into_iter().enumerate() {
                        SemanticButton {
                            class: "canvas-a11y-item",
                            role: "option",
                            tabindex: Some("-1".to_owned()),
                            selected: selected_layers.contains(&layer),
                            aria_selected: Some(if selected_layers.contains(&layer) { "true" } else { "false" }.to_owned()),
                            aria_posinset: Some((index + 1).to_string()),
                            aria_setsize: Some(stage_item_count.to_string()),
                            aria_label: Some(format!(
                                "{name}, layer {} of {}{}",
                                index + 1,
                                stage_item_count,
                                if locked { ", locked" } else { "" },
                            )),
                            onclick: {
                                let selection = session.selection.clone();
                                move |_| {
                                    selection.set(Some(layer));
                                    selected.set(Some(layer));
                                    *revision.write() += 1;
                                }
                            },
                            oncontextmenu: {
                                let selection = session.selection.clone();
                                move |evt: MouseEvent| {
                                    evt.prevent_default();
                                    evt.stop_propagation();
                                    if !selection.contains(layer) {
                                        selection.set(Some(layer));
                                        selected.set(Some(layer));
                                    } else {
                                        selection.activate(layer);
                                    }
                                    let point = evt.client_coordinates();
                                    menu.set(Some(MenuRequest {
                                        x: point.x,
                                        y: point.y,
                                        target: crate::ui::context_menu::MenuTarget::StageLayer(layer),
                                    }));
                                }
                            },
                            "{name}"
                        }
                    }
                }
            }
            div { id: "stagefoot",
                SemanticButton {
                    class: if rings_on() { "chip on" } else { "chip" },
                    selected: rings_on(),
                    aria_label: "3D handles",
                    onclick: move |_| {
                        let next = !rings_on();
                        rings.store(next, std::sync::atomic::Ordering::Relaxed);
                        rings_on.set(next);
                        revision += 1;
                    },
                    title: "3D handles",
                    "3D"
                }
                SemanticButton {
                    class: "chip zoomchip",
                    aria_label: "View zoom {view_pct()}% · fit to window",
                    title: "Fit to window · ⌘0",
                    onclick: {
                        let request = session.view_request.clone();
                        move |_| {
                            *request.lock().unwrap() = Some(crate::ui::session::ViewRequest::Fit);
                            revision += 1;
                        }
                    },
                    "{view_pct()}%"
                }
                span { "{comp_line}" }
            }
        }
    )
}

#[component]
#[allow(clippy::too_many_arguments)]
pub(super) fn TimelinePanel(
    session: Session,
    echo: u32,
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    scroll_y: Signal<f64>,
    playhead: Signal<f64>,
    revision: Signal<u32>,
    menu: Signal<Option<MenuRequest>>,
) -> Element {
    let host = consume_context::<crate::ui::host::Host>();
    let handle = host.mounts.get(surface_window(), "timeline", || {
        let rows = fixture::canvas_rows_from_doc(&session.doc.lock().unwrap());
        TimelineState::new(rows, session.timeline_rx.clone())
            .with_clock(session.clock.clone())
            .with_scale(session.scale.clone())
            .with_document(session.doc.clone(), fixture::canvas_rows_from_doc)
            .with_selection(session.selection.clone())
            .with_gesture(session.gesture.clone())
            .with_capture(session.surface_capture.clone())
            .with_key_mirror(session.selected_keys.clone())
            .with_notice(session.project_notice.clone())
    });
    let bindings = TimelineBindings {
        selected: Some(selected),
        scroll_y: Some(scroll_y),
        playhead: Some(playhead),
        revision: Some(revision),
        context_menu: Some(menu),
    };
    // 行は Document から引き直す。一覧を持ち回っていると、書き込みの度に
    // 引き直しを**忘れた手**の分だけ窓が古いまま残る(名前変更がそれだった)。
    let _ = revision();
    // 行の投影は Document の revision が動いた時だけ(擦りの transient では引き直さない)。
    let memo: std::rc::Rc<std::cell::RefCell<Option<(String, Vec<crate::ui::fixture::LayerRow>)>>> =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None)));
    let rows_now = {
        let d = session.doc.lock().unwrap();
        let stamp = fixture::memo_stamp(d.revision());
        let mut memo = memo.borrow_mut();
        match memo.as_ref() {
            Some((seen, rows)) if *seen == stamp => rows.clone(),
            _ => {
                let rows = fixture::layer_rows_from_doc(&d);
                *memo = Some((stamp, rows.clone()));
                rows
            }
        }
    };
    let canvas_rows = fixture::canvas_rows_from_doc(&session.doc.lock().unwrap());
    let first = ((scroll_y() / (crate::ui::tokens::ROW * session.scale.factor())).floor() as usize)
        .min(canvas_rows.len());
    let selected_keys = session.selected_keys.lock().unwrap().clone();
    let mut timeline_items = Vec::new();
    for (row, label) in canvas_rows
        .iter()
        .zip(rows_now.iter())
        .skip(first)
        .take(64)
    {
        let Some(layer) = row.layer else { continue };
        for &at_sec in &row.keys {
            timeline_items.push((
                layer,
                row.prop.clone(),
                at_sec,
                label.name.clone(),
                label.locked,
            ));
        }
    }
    let timeline_item_count = timeline_items.len();
    let surface = rsx! {
        div { class: "surface-stack",
            SurfaceView::<TimelineState> { handle, bindings, label: "Timeline canvas" }
            div { class: "canvas-a11y", role: "listbox", aria_label: "Visible Timeline keyframes",
                for (index, (layer, property, at_sec, name, locked)) in timeline_items.into_iter().enumerate() {
                    {
                        let key = crate::ui::session::KeySel {
                            layer,
                            property: property.clone(),
                            at_sec,
                        };
                        let is_selected = selected_keys.iter().any(|selected| {
                            selected.layer == key.layer
                                && selected.property == key.property
                                && (selected.at_sec - key.at_sec).abs()
                                    < 0.5 * session.clock.frame_duration_sec()
                        });
                        let choose = key.clone();
                        let choose_session = session.clone();
                        let context = key.clone();
                        let context_session = session.clone();
                        rsx!(SemanticButton {
                            class: "canvas-a11y-item",
                            role: "option",
                            tabindex: Some("-1".to_owned()),
                            selected: is_selected,
                            aria_selected: Some(if is_selected { "true" } else { "false" }.to_owned()),
                            aria_posinset: Some((index + 1).to_string()),
                            aria_setsize: Some(timeline_item_count.to_string()),
                            aria_label: Some(format!(
                                "{name} keyframe at {at_sec:.3} seconds, {} of {}{}",
                                index + 1,
                                timeline_item_count,
                                if locked { ", locked" } else { "" },
                            )),
                            onclick: move |_| {
                                choose_session.selection.replace_preserving_keys([choose.layer]);
                                *choose_session.selected_keys.lock().unwrap() = vec![choose.clone()];
                                let _ = choose_session.timeline_tx.send(TimelineMsg::SelectKeys(vec![choose.clone()]));
                                selected.set(Some(choose.layer));
                                *revision.write() += 1;
                            },
                            oncontextmenu: move |evt: MouseEvent| {
                                evt.prevent_default();
                                evt.stop_propagation();
                                context_session.selection.replace_preserving_keys([context.layer]);
                                *context_session.selected_keys.lock().unwrap() = vec![context.clone()];
                                let _ = context_session.timeline_tx.send(TimelineMsg::SelectKeys(vec![context.clone()]));
                                selected.set(Some(context.layer));
                                let point = evt.client_coordinates();
                                menu.set(Some(MenuRequest {
                                    x: point.x,
                                    y: point.y,
                                    target: crate::ui::context_menu::MenuTarget::TimelineKey { layer: context.layer },
                                }));
                                *revision.write() += 1;
                            },
                            "{name}"
                        })
                    }
                }
            }
        }
    };
    timeline_shell(
        session.doc.clone(),
        attrs_state,
        &rows_now,
        layer_rows,
        surface,
        session.selection.clone(),
        selected,
        scroll_y,
        session.timeline_tx.clone(),
        &session,
        revision,
        menu,
    )
}

fn surface_window() -> dioxus_native::winit::window::WindowId {
    try_consume_context::<std::sync::Arc<dyn dioxus_native::winit::window::Window>>()
        .map_or(crate::ui::host::Host::HEADLESS, |window| window.id())
}
