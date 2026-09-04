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
use crate::ui::timeline_widget::{TimelineBindings, TimelineState};
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
        &session.selected_size,
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
            session.selected_size.clone(),
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
                                        crate::ui::session::noted(d.apply(crate::doc::store::Intent::SetComposition(next)), revision);
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
            div { id: "stage",
                SurfaceView::<StageState> { handle, bindings }
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
    let surface = rsx! { SurfaceView::<TimelineState> { handle, bindings } };
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
