use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;
use crate::ui::browser::browser_panel;
use crate::ui::dock::Panel;
use crate::ui::inspector::{inspector_panel, ChoiceId};
use crate::ui::semantic_menu::SemanticButton;
use crate::ui::session::Session;
use crate::ui::stage_widget::StageWidget;
use crate::ui::timeline_shell::timeline_shell;
use crate::ui::timeline_widget::TimelineWidget;
use crate::ui::fixture;

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
    browser_panel(
        &session,
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

/// ウィジェットはコンポーネントの中で作る。置き場を移すと要素が作り直されるので、
/// `CustomWidgetAttr` を app と共有すると2枚目が空になる(中身は一度しか渡せない)。
#[component]
pub(super) fn StagePanel(
    session: Session,
    selected: Signal<Option<crate::doc::store::LayerId>>,
    revision: Signal<u32>,
    comp_line: String,
) -> Element {
    let view_pct = use_signal(|| 100u32);
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
            session.frame_dim.clone(),
            session.gesture.clone(),
            session.output_only.clone(),
            view_pct,
            session.view_request.clone(),
        ))
    });
    let rings = session.rings.clone();
    let mut rings_on = use_signal(|| rings.load(std::sync::atomic::Ordering::Relaxed));
    rsx!(
        div { id: "stagecol",
            if session.doc.lock().unwrap().view().layers().is_empty() {
                div { class: "stagehint", "Add a Text layer from Create to begin" }
            }
            div { id: "stage",
                object { "data": attr }
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
                .with_playhead_mirror(playhead)
                .with_revision(revision)
                .with_gesture(session.gesture.clone())
                .with_key_mirror(session.selected_keys.clone()),
        )
    });
    // 行は Document から引き直す。一覧を持ち回っていると、書き込みの度に
    // 引き直しを**忘れた手**の分だけ窓が古いまま残る(名前変更がそれだった)。
    let _ = revision();
    // 行の投影は Document の revision が動いた時だけ(擦りの transient では引き直さない)。
    let memo: std::rc::Rc<std::cell::RefCell<Option<(String, Vec<crate::ui::fixture::LayerRow>)>>> =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(None)));
    let rows_now = {
        let d = session.doc.lock().unwrap();
        let stamp = format!("{:?}", d.revision());
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
        attr,
        session.selection.clone(),
        selected,
        scroll_y,
        session.timeline_tx.clone(),
        &session,
        revision,
    )
}
