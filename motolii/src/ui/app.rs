use dioxus_native::prelude::*;
use dioxus_native::CustomWidgetAttr;
use dioxus_dnd::prelude::{transition, GestureEffect, GestureEvent, GesturePhase, Point};
use dioxus_workbench::{LayoutNode, SplitAxis, SplitId, TileId};

use crate::ui::browser::browser_panel;
use crate::ui::dock::{Dock, Panel, Side};
use crate::ui::inspector::{inspector_panel, ChoiceDismiss, ChoiceId};
use crate::ui::keymap::Intent;
use crate::ui::output::{OutputStatus, OutputSurface};
use crate::ui::semantic_menu::{
    MenuDismiss, MenuId, SemanticButton, SemanticControl, SemanticMenu,
};
use crate::ui::session::Session;
use crate::ui::stage_widget::StageWidget;
use crate::ui::timeline_shell::timeline_shell;
use crate::ui::timeline_widget::{split_layer, TimelineMsg, TimelineWidget};
use crate::ui::fixture;
use crate::ui::tokens;

static STYLES: Asset = asset!("/src/ui/styles.css");

#[cfg(test)]
static TEST_STYLES: &str = include_str!("styles.css");

/// 落とし先の当たり。**位置を計算しない** —— 5枚の当たりを重ねて置き、
/// どれに乗ったかで決める。
const SIDES: [(Side, &str); 5] = [
    (Side::Top, "top"),
    (Side::Left, "left"),
    (Side::Center, "center"),
    (Side::Right, "right"),
    (Side::Bottom, "bottom"),
];

/// 節の中で隣り合う2つの割合を動かす。
#[derive(Clone)]
struct GripDrag {
    split: SplitId,
    axis: SplitAxis,
    start: f64,
    extent: f64,
    start_ratio: f64,
}

const TAB_DRAG_THRESHOLD: f64 = 6.0;

#[derive(Clone, Copy, PartialEq, Debug)]
struct TabDrag {
    panel: Panel,
    phase: GesturePhase,
    cursor: Point,
}

impl TabDrag {
    fn pressed(panel: Panel, x: f64, y: f64, pointer_id: i32) -> Self {
        let at = Point::new(x, y);
        let (phase, _) = transition(
            GesturePhase::Idle,
            GestureEvent::Down { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        Self { panel, phase, cursor: at }
    }

    fn move_to(mut self, x: f64, y: f64, pointer_id: i32) -> (Self, GestureEffect) {
        let at = Point::new(x, y);
        let (phase, effect) = transition(
            self.phase,
            GestureEvent::Move { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        self.phase = phase;
        self.cursor = at;
        (self, effect)
    }

    fn release(mut self, x: f64, y: f64, pointer_id: i32) -> (Self, GestureEffect) {
        let at = Point::new(x, y);
        let (phase, _) = transition(
            self.phase,
            GestureEvent::Move { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        let (phase, effect) = transition(
            phase,
            GestureEvent::Up { at, pointer_id },
            TAB_DRAG_THRESHOLD,
        );
        self.phase = phase;
        self.cursor = at;
        (self, effect)
    }

    fn cancel(mut self) -> (Self, GestureEffect) {
        let (phase, effect) = transition(
            self.phase,
            GestureEvent::Cancel,
            TAB_DRAG_THRESHOLD,
        );
        self.phase = phase;
        (self, effect)
    }

    fn dragging(self) -> bool {
        matches!(self.phase, GesturePhase::Dragging { .. })
    }

    fn pointer_id(self) -> i32 {
        match self.phase {
            GesturePhase::Pressed { pointer_id, .. }
            | GesturePhase::Dragging { pointer_id, .. } => pointer_id,
            GesturePhase::Idle => 0,
        }
    }
}

fn take_tab_release(
    drag: &mut Option<TabDrag>,
    x: f64,
    y: f64,
    pointer_id: Option<i32>,
) -> Option<(TabDrag, GestureEffect)> {
    let drag = drag.take()?;
    let (_, effect) = drag.release(x, y, pointer_id.unwrap_or_else(|| drag.pointer_id()));
    Some((drag, effect))
}

fn finish_tab_release(
    mut dock: Signal<Dock>,
    mut tab_drag: Signal<Option<TabDrag>>,
    tile_nodes: &TileNodes,
    host: &crate::ui::host::Host,
    x: f64,
    y: f64,
    pointer_id: Option<i32>,
) {
    let terminal = take_tab_release(&mut tab_drag.write(), x, y, pointer_id);
    let Some((drag, effect)) = terminal else { return };
    match effect {
        GestureEffect::Tap => dock.write().set_active(drag.panel),
        GestureEffect::Drop { .. } => {
            if let Some((target, side)) = dock_target_at(tile_nodes, x, y) {
                dock.write().drop_onto(drag.panel, &target, side);
            } else if dock.write().detach(drag.panel) {
                host.open(drag.panel);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[test]
fn tab_release_has_one_terminal_action_and_cancel_has_none() {
    let drag = TabDrag::pressed(Panel::Inspector, 10.0, 10.0, 7)
        .move_to(30.0, 10.0, 7)
        .0;
    let mut armed = Some(drag);
    assert!(matches!(
        take_tab_release(&mut armed, -20.0, -20.0, None),
        Some((_, GestureEffect::Drop { .. }))
    ));
    assert!(take_tab_release(&mut armed, -20.0, -20.0, None).is_none());

    let drag = TabDrag::pressed(Panel::Inspector, 10.0, 10.0, 7)
        .move_to(30.0, 10.0, 7)
        .0;
    let mut cancelled = Some(drag);
    let _ = cancelled.take().unwrap().cancel();
    assert!(take_tab_release(&mut cancelled, -20.0, -20.0, None).is_none());
}

fn splitter_delta(pointer_delta: f64, extent: f64) -> f32 {
    if extent.is_finite() && extent > 0.0 {
        (pointer_delta / extent) as f32
    } else {
        0.0
    }
}

#[cfg(test)]
#[test]
fn splitter_delta_uses_the_measured_container_extent() {
    assert!((splitter_delta(120.0, 600.0) - 0.2).abs() < f32::EPSILON);
    assert!((splitter_delta(120.0, 1_200.0) - 0.1).abs() < f32::EPSILON);
    assert_eq!(splitter_delta(120.0, 0.0), 0.0);
}

#[cfg(test)]
#[test]
fn ui_motion_never_transitions_direct_manipulation_geometry() {
    assert!(!TEST_STYLES.contains("transition: all"));
    for property in ["left", "top", "width", "height", "transform"] {
        assert!(
            !TEST_STYLES.contains(&format!("transition-property: {property}")),
            "direct manipulation property entered the transition contract: {property}"
        );
    }
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
    /// 窓の文字の大きさ(%)。設定を1枚へ集めるため Settings が触る。
    scale_pct: Signal<u32>,
    /// 再生位置。値を出す側はこれを見て描き直す。
    playhead: Signal<f64>,
    inspector_choice: Signal<Option<ChoiceId>>,
}

fn panes_for(ui: &fixture::UiData) -> Panes {
    Panes {
        scale_pct: use_signal(|| 100),
        playhead: use_signal(|| 0.0),
        inspector_choice: use_signal(|| None),
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
            runtime.in_scope(scope, move || {
                // 面が作り直された後(hotpatch・再描画)に古い信号へ書くと落ちる。捨てる。
                if let Ok(mut echo) = echo.try_write() {
                    *echo += 1;
                }
            });
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
            playhead: p.playhead,
            choice_open: p.inspector_choice,
        }),
        Panel::Desk => rsx!(crate::ui::desk::DeskPanel {
            session: session.clone(),
            revision: p.revision,
            playhead: p.playhead,
            on_history: {
                let doc = session.doc.clone();
                let timeline_tx = session.timeline_tx.clone();
                move |steps: i32| {
                    history_step(&doc, p.layer_rows, p.attrs_state, &timeline_tx, p.revision, steps);
                }
            },
        }),
        Panel::Timeline => rsx!(TimelinePanel {
            session: session.clone(),
            layer_rows: p.layer_rows,
            attrs_state: p.attrs_state,
            selected: p.selected,
            scroll_y: p.scroll_y,
            playhead: p.playhead,
            renaming: p.renaming,
            revision: p.revision,
        }),
    }
}

fn file_drop_overlay(session: &Session) -> Element {
    let count = session.file_drop.count();
    if count == 0 {
        return rsx! {};
    }
    let noun = if count == 1 { "file" } else { "files" };
    rsx!(div {
        class: "file-drop-overlay",
        div { class: "file-drop-card", "Drop {count} {noun} to import" }
    })
}

/// 別窓。パネル1枚だけを出す。状態は窓をまたいで1つ(Session)。
pub fn detached() -> Element {
    let session = use_hook(|| consume_context::<Session>());
    let ui = session.ui.clone();
    let panel = use_hook(|| consume_context::<Panel>());
    let host = use_hook(|| consume_context::<crate::ui::host::Host>());
    let panes = panes_for(&ui);
    wire_windows(&host, panes);
    let reduced_motion = dioxus_core::try_consume_context::<
        std::sync::Arc<dyn dioxus_native::winit::window::Window>,
    >()
    .is_some()
        && tokens::system_prefers_reduced_motion();
    let css = tokens::css_root(100, reduced_motion);
    rsx!(
        style { {css} }
        link { rel: "stylesheet", href: STYLES }
        ChoiceDismiss { open: panes.inspector_choice }
        div { id: "detached", {panel_body(panel, &session, &ui, panes)} }
        {file_drop_overlay(&session)}
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
    playhead: Signal<f64>,
    choice_open: Signal<Option<ChoiceId>>,
) -> Element {
    let drag = use_signal(|| None);
    let num_edit = use_signal(|| None);
    inspector_panel(
        &session.doc,
        selected,
        &session.clock,
        revision,
        editing,
        drag,
        num_edit,
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
            session.frame_dim.clone(),
            session.gesture.clone(),
            false,
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
                SemanticButton {
                    class: if rings_on() { "chip on" } else { "chip" },
                    selected: rings_on(),
                    aria_label: "Toggle 3D handles",
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

/// 見る側の設定。**作品には入らない**物だけを置く。
/// 散らばっていると探せないので、窓の設定はここへ集める。
#[component]
fn SettingsSheet(session: Session, scale_pct: Signal<u32>) -> Element {
    let dim = session.frame_dim.clone();
    let pct = use_signal(|| dim.load(std::sync::atomic::Ordering::Relaxed));
    let dim_step = move |dim: std::sync::Arc<std::sync::atomic::AtomicU32>, mut pct: Signal<u32>, by: i32| {
        let next = (pct() as i32 + by).clamp(0, 100) as u32;
        dim.store(next, std::sync::atomic::Ordering::Relaxed);
        pct.set(next);
    };
    let (dim_a, dim_b) = (dim.clone(), dim.clone());

    let scale_step = move |ui: std::sync::Arc<crate::ui::tokens::UiScale>, mut sig: Signal<u32>, by: i32| {
        let next = (sig() as i32 + by).clamp(50, 200) as u32;
        ui.set_percent(next);
        sig.set(ui.percent());
    };
    let (ui_a, ui_b) = (session.scale.clone(), session.scale.clone());

    rsx!(
        div { class: "settings-sheet",
            div { class: "sec", "VIEW" }
            div { class: "prow",
                span { class: "pname", "Outside dim" }
                div { class: "zoomctl",
                    SemanticButton { class: "zbtn", aria_label: "Decrease outside dim", onclick: move |_| dim_step(dim_a.clone(), pct, -5), "−" }
                    span { class: "zval", "{pct()}%" }
                    SemanticButton { class: "zbtn", aria_label: "Increase outside dim", onclick: move |_| dim_step(dim_b.clone(), pct, 5), "+" }
                }
            }
            div { class: "sec", "WINDOW" }
            div { class: "prow",
                span { class: "pname", "Scale" }
                div { class: "zoomctl",
                    SemanticButton { class: "zbtn", aria_label: "Decrease interface scale", onclick: move |_| scale_step(ui_a.clone(), scale_pct, -5), "−" }
                    span { class: "zval", "{scale_pct()}%" }
                    SemanticButton { class: "zbtn", aria_label: "Increase interface scale", onclick: move |_| scale_step(ui_b.clone(), scale_pct, 5), "+" }
                }
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
    playhead: Signal<f64>,
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
                .with_playhead_mirror(playhead)
                .with_revision(revision)
                .with_gesture(session.gesture.clone())
                .with_key_mirror(session.selected_keys.clone()),
        )
    });
    // 行は Document から引き直す。一覧を持ち回っていると、書き込みの度に
    // 引き直しを**忘れた手**の分だけ窓が古いまま残る(名前変更がそれだった)。
    let _ = revision();
    let rows_now = fixture::layer_rows_from_doc(&session.doc.lock().unwrap());
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
        renaming,
        revision,
    )
}

/// 履歴を進める・戻す手は 1 つ。キーも机の履歴も同じ手を使う(行の目・solo・鍵も追従する)。
fn history_step(
    doc: &std::sync::Arc<std::sync::Mutex<crate::doc::store::Document>>,
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &std::sync::mpsc::Sender<TimelineMsg>,
    revision: Signal<u32>,
    steps: i32,
) {
    let mut moved = 0;
    {
        let mut d = doc.lock().unwrap();
        for _ in 0..steps.unsigned_abs() {
            let ok = if steps < 0 { d.undo() } else { d.redo() };
            if !ok {
                break;
            }
            moved += 1;
        }
    }
    if moved > 0 {
        refresh_layer_projection(doc, layer_rows, attrs_state, timeline_tx, revision);
    }
    println!("PROBE room=write verdict=history moved={moved}");
}

fn refresh_layer_projection(
    doc: &std::sync::Arc<std::sync::Mutex<crate::doc::store::Document>>,
    mut layer_rows: Signal<Vec<fixture::LayerRow>>,
    mut attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &std::sync::mpsc::Sender<TimelineMsg>,
    mut revision: Signal<u32>,
) {
    let snapshot = doc.lock().unwrap();
    let rows = fixture::layer_rows_from_doc(&snapshot);
    let canvas = fixture::canvas_rows_from_doc(&snapshot);
    drop(snapshot);
    attrs_state.set(rows.iter().map(|row| (row.hidden, row.solo, row.locked)).collect());
    layer_rows.set(rows);
    let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
    *revision.write() += 1;
}

/// 作品を仕舞う。行き先が決まっていなければ聞く。
async fn put_away(
    session: Session,
    poke: crate::ui::host::Poke,
    ask: bool,
) -> bool {
    let known = session.project_path.lock().unwrap().clone();
    let out = match known.filter(|_| !ask) {
        Some(path) => path,
        None => {
            let picked = rfd::AsyncFileDialog::new()
                .set_file_name("song.rrd")
                .save_file()
                .await;
            let Some(file) = picked else { return false };
            let mut out = file.path().to_path_buf();
            if out.extension().is_none_or(|e| !e.eq_ignore_ascii_case("rrd")) {
                out.set_extension("rrd");
            }
            out
        }
    };
    let save_result = session.doc.lock().unwrap().save(&out);
    let (word, saved) = match save_result {
        Ok(()) => {
            session.mark_saved(out.clone());
            (format!("Saved {}", out.display()), true)
        }
        Err(e) => (format!("Save failed: {e}"), false),
    };
    println!("PROBE room=project verdict=save {word}");
    *session.project_notice.lock().unwrap() = word;
    poke.poke();
    saved
}

async fn allow_project_replacement(
    session: Session,
    poke: crate::ui::host::Poke,
    window: Option<std::sync::Arc<dyn dioxus_native::winit::window::Window>>,
) -> bool {
    if !session.is_dirty() {
        return true;
    }
    let mut dialog = rfd::AsyncMessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title("Save changes?")
        .set_description("This project has changes that are not saved.")
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
            "Save".to_owned(),
            "Don't Save".to_owned(),
            "Cancel".to_owned(),
        ));
    if let Some(window) = window.as_deref() {
        dialog = dialog.set_parent(window);
    }
    let answer = dialog.show().await;
    match answer {
        rfd::MessageDialogResult::Custom(label) if label == "Save" => {
            put_away(session, poke, false).await
        }
        rfd::MessageDialogResult::Custom(label) if label == "Don't Save" => true,
        _ => false,
    }
}

type SplitNodes = std::rc::Rc<
    std::cell::RefCell<std::collections::BTreeMap<SplitId, dioxus_native::NodeHandle>>,
>;
type TileNodes = std::rc::Rc<
    std::cell::RefCell<std::collections::BTreeMap<TileId, dioxus_native::NodeHandle>>,
>;

fn dock_side_at(x: f64, y: f64, width: f64, height: f64) -> Side {
    if y < height * 0.25 {
        Side::Top
    } else if y > height * 0.75 {
        Side::Bottom
    } else if x < width * 0.25 {
        Side::Left
    } else if x > width * 0.75 {
        Side::Right
    } else {
        Side::Center
    }
}

fn node_has_id(node: &blitz_dom::Node, expected: &str) -> bool {
    node.attr(blitz_dom::local_name!("id")).is_some_and(|id| id == expected)
}

fn dock_target_at(tile_nodes: &TileNodes, x: f64, y: f64) -> Option<(TileId, Side)> {
    let mounted = tile_nodes
        .borrow()
        .iter()
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect::<Vec<_>>();
    for (id, handle) in mounted {
        let Some(doc) = handle.try_doc() else { continue };
        let Some(node) = doc.get_node(handle.node_id()) else { continue };
        // 消えた箱の id は次に作られた節へ再利用される。本人でなければ古い取っ手。
        if !node_has_id(node, &format!("tile-{}", id.as_str())) {
            continue;
        }
        let origin = node.absolute_position(0.0, 0.0);
        let size = node.final_layout().size;
        let local_x = x - f64::from(origin.x);
        let local_y = y - f64::from(origin.y);
        let width = f64::from(size.width);
        let height = f64::from(size.height);
        if local_x >= 0.0 && local_y >= 0.0 && local_x <= width && local_y <= height {
            return Some((id, dock_side_at(local_x, local_y, width, height)));
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn dock_zone(
    id: TileId,
    panels: Vec<Panel>,
    shown: Option<Panel>,
    d: &Dock,
    mut dock: Signal<Dock>,
    mut tab_drag: Signal<Option<TabDrag>>,
    tile_nodes: TileNodes,
    session: &Session,
    ui: &fixture::UiData,
    panes: Panes,
    playing: Signal<bool>,
) -> Element {
    let gesture = tab_drag();
    let dragging = gesture.is_some_and(TabDrag::dragging);
    rsx!(
        div {
            class: "zone",
            key: "{id}",
            id: "tile-{id}",
            onmounted: {
                let tile_nodes = tile_nodes.clone();
                let id = id.clone();
                move |evt: MountedEvent| {
                    let mounted = evt.data();
                    if let Some(node) = mounted.downcast::<dioxus_native::NodeHandle>() {
                        tile_nodes.borrow_mut().insert(id.clone(), node.clone());
                    }
                }
            },
            div { class: "ptabs",
                for panel in panels.iter().copied() {
                    span {
                        id: "dock-tab-{panel}",
                        aria_label: "{panel} panel tab",
                        class: match gesture.filter(|drag| drag.panel == panel) {
                            Some(drag) if drag.dragging() => "ptab held",
                            Some(_) => "ptab pressed",
                            None if d.is_active(panel) => "ptab on",
                            None => "ptab",
                        },
                        style: if d.is_active(panel) {
                            format!("border-bottom-color: {};", panel.way())
                        } else {
                            String::new()
                        },
                        onpointerdown: move |evt: PointerEvent| {
                            if evt.data().trigger_button()
                                != Some(
                                    dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary,
                                )
                            {
                                return;
                            }
                            evt.prevent_default();
                            let p = evt.data().client_coordinates();
                            tab_drag.set(Some(TabDrag::pressed(
                                panel,
                                p.x,
                                p.y,
                                evt.data().pointer_id(),
                            )));
                        },
                        "{panel}"
                    }
                }
                if shown == Some(Panel::Timeline) {
                    {crate::ui::timeline_shell::transport(
                        session.clock.clone(),
                        playing,
                        panes.playhead,
                        panes.layer_rows.read().len(),
                    )}
                }
            }
            if let Some(panel) = shown {
                div { class: "zbody", {panel_body(panel, session, ui, panes)} }
            }
            if dragging {
                div { class: "dropmap dragging",
                    for (_, class) in SIDES {
                        div {
                            class: "dz {class}",
                        }
                    }
                    div { class: "dock-guide", aria_label: "Dock position guide",
                        for (_, class) in SIDES {
                            span { class: "dock-guide-{class}" }
                        }
                    }
                }
            }
        }
    )
}

#[allow(clippy::too_many_arguments)]
fn tile_view(
    node: LayoutNode,
    dock: Signal<Dock>,
    tab_drag: Signal<Option<TabDrag>>,
    mut grip: Signal<Option<GripDrag>>,
    split_nodes: SplitNodes,
    tile_nodes: TileNodes,
    session: &Session,
    ui: &fixture::UiData,
    panes: Panes,
    playing: Signal<bool>,
) -> Element {
    let d = dock();
    match node {
        LayoutNode::Split { id, axis, ratio, first, second } => {
            let flow = if axis == SplitAxis::Horizontal { "row" } else { "column" };
            let first_share = ratio;
            let second_share = 1.0 - ratio;
            rsx!(
                div {
                    class: "tlinear",
                    key: "{id}",
                    id: "split-{id}",
                    style: "flex-direction: {flow};",
                    onmounted: {
                        let split_nodes = split_nodes.clone();
                        let id = id.clone();
                        move |evt: MountedEvent| {
                            let mounted = evt.data();
                            if let Some(node) = mounted.downcast::<dioxus_native::NodeHandle>() {
                                split_nodes.borrow_mut().insert(id.clone(), node.clone());
                            }
                        }
                    },
                    div {
                        class: "tslot",
                        style: "flex: {first_share};",
                        {tile_view(
                            *first,
                            dock,
                            tab_drag,
                            grip,
                            split_nodes.clone(),
                            tile_nodes.clone(),
                            session,
                            ui,
                            panes,
                            playing,
                        )}
                    }
                    div {
                        class: if axis == SplitAxis::Horizontal { "vgrip" } else { "hgrip" },
                        onpointerdown: {
                            let split_nodes = split_nodes.clone();
                            let split = id.clone();
                            move |evt: PointerEvent| {
                                let p = evt.data().client_coordinates();
                                let start = if axis == SplitAxis::Horizontal { p.x } else { p.y };
                                let Some(handle) = split_nodes.borrow().get(&split).cloned() else {
                                    return;
                                };
                                let Some(doc) = handle.try_doc() else { return };
                                let Some(node) = doc.get_node(handle.node_id()) else { return };
                                if !node_has_id(node, &format!("split-{}", split.as_str())) {
                                    return;
                                }
                                let layout = node.final_layout();
                                let extent = if axis == SplitAxis::Horizontal {
                                    f64::from(layout.size.width)
                                } else {
                                    f64::from(layout.size.height)
                                };
                                drop(doc);
                                if extent > 0.0 {
                                    grip.set(Some(GripDrag {
                                        split: split.clone(),
                                        axis,
                                        start,
                                        extent,
                                        start_ratio: ratio,
                                    }));
                                }
                            }
                        }
                    }
                    div {
                        class: "tslot",
                        style: "flex: {second_share};",
                        {tile_view(
                            *second,
                            dock,
                            tab_drag,
                            grip,
                            split_nodes.clone(),
                            tile_nodes.clone(),
                            session,
                            ui,
                            panes,
                            playing,
                        )}
                    }
                }
            )
        }
        LayoutNode::Tile(tile) => {
            let panels = d.panels(&tile);
            let shown = d.active(&tile);
            dock_zone(
                tile.id,
                panels,
                shown,
                &d,
                dock,
                tab_drag,
                tile_nodes,
                session,
                ui,
                panes,
                playing,
            )
        }
    }
}

pub fn app() -> Element {
    let mut playing = use_signal(|| false);
    let mut dock = use_signal(Dock::default);

    let mut tab_drag = use_signal(|| Option::<TabDrag>::None);
    let mut grip = use_signal(|| Option::<GripDrag>::None);
    let split_nodes = use_hook(|| {
        std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new()))
    });
    let tile_nodes = use_hook(|| {
        std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new()))
    });
    let mut open_menu = use_signal(|| None::<MenuId>);

    let session = use_hook(|| consume_context::<Session>()).clone();
    let loaded = session.ui.clone();
    let host = use_hook(|| consume_context::<crate::ui::host::Host>());
    let window = use_hook(|| {
        dioxus_core::try_consume_context::<
            std::sync::Arc<dyn dioxus_native::winit::window::Window>,
        >()
    });
    // 別窓が閉じたら置き場へ戻す。別の窓からの合図なので、自分の runtime を包んで渡す。
    use_hook(|| {
        let runtime = dioxus_core::Runtime::current();
        let scope = dioxus_core::current_scope_id();
        host.on_close(move |panel| {
            let mut dock = dock;
            runtime.in_scope(scope, move || dock.write().reattach(panel));
        });
    });
    use_hook(|| {
        let Some(window_id) = window.as_ref().map(|window| window.id()) else { return };
        let runtime = dioxus_core::Runtime::current();
        let scope = dioxus_core::current_scope_id();
        let tile_nodes = tile_nodes.clone();
        let dock_host = host.clone();
        host.on_primary_pointer_release(window_id, move |x, y| {
            runtime.in_scope(scope, || {
                finish_tab_release(dock, tab_drag, &tile_nodes, &dock_host, x, y, None);
            });
        });
    });
    use_hook(|| {
        let runtime = dioxus_core::Runtime::current();
        let scope = dioxus_core::current_scope_id();
        let gesture = session.gesture.clone();
        host.on_focus_lost(move || {
            gesture.cancel();
            // Cmd+Tab で離れると修飾の keyup が届かない。戻った時の Space が Cmd+Space になる。
            crate::ui::keymap::forget_modifiers();
            runtime.in_scope(scope, move || {
                if let Some(drag) = *tab_drag.peek() {
                    let _ = drag.cancel();
                }
                tab_drag.set(None);
                grip.set(None);
                open_menu.set(None);
            });
        });
    });
    let panes = panes_for(&loaded);
    let output_generation = (panes.echo)();
    let project_notice = session.project_notice.lock().unwrap().clone();
    let status_line = match (project_notice.is_empty(), session.is_dirty()) {
        (true, true) => "Edited".to_owned(),
        (false, true) => format!("{project_notice} · Edited"),
        _ => project_notice,
    };
    let scale_pct = panes.scale_pct;
    wire_windows(&host, panes);
    // 引き出しの開閉を机の tile の広さへ写す。机は呼ばれない — 開閉は Document と焦点から読む。
    {
        let session = session.clone();
        use_effect(move || {
            let _ = (panes.revision)();
            let _ = (panes.echo)();
            let open = crate::ui::desk::drawer_of(&session).is_some();
            if dock.peek().desk_open() != open {
                dock.write().open_desk(open);
            }
        });
    }

    let Panes {
        layer_rows,
        attrs_state,
        selected: selected_sig,
        revision,
        text_editing,
        renaming,
        ..
    } = panes;
    let mut selected = selected_sig;

    let timeline_tx = session.timeline_tx.clone();
    let doc = session.doc.clone();
    let clock = session.clock.clone();
    let selection = session.selection.clone();
    let sync_doc = doc.clone();
    let sync_clock = clock.clone();
    use_effect(move || {
        let _ = revision();
        sync_clock.sync_document(&sync_doc.lock().unwrap());
    });

    let reduced_motion = window.is_some() && tokens::system_prefers_reduced_motion();
    let css = tokens::css_root(scale_pct(), reduced_motion);

    let d = dock();
    let menubar_gesture = session.gesture.clone();
    let pointer_tiles = tile_nodes.clone();
    let dock_host = host.clone();
    rsx!(
        style { {css} }
        link { rel: "stylesheet", href: STYLES }
        div {
            id: "app",
            tabindex: "0",
            autofocus: "true",
            style: "grid-template-rows: var(--section) 1fr calc(20 * var(--s) * 1px);",
            onpointermove: move |evt: PointerEvent| {
                let p = evt.data().client_coordinates();
                let current = *tab_drag.peek();
                if let Some(drag) = current {
                    let (next, _) = drag.move_to(p.x, p.y, evt.data().pointer_id());
                    if next != drag {
                        tab_drag.set(Some(next));
                    }
                }
                let held = grip.peek().clone();
                if let Some(g) = held {
                    let now = if g.axis == SplitAxis::Horizontal { p.x } else { p.y };
                    let delta = splitter_delta(now - g.start, g.extent);
                    if delta != 0.0 {
                        dock.write()
                            .set_split_ratio(&g.split, g.start_ratio + f64::from(delta));
                    }
                }
            },
            onpointerup: move |evt: PointerEvent| {
                *grip.write() = None;
                let p = evt.data().client_coordinates();
                finish_tab_release(
                    dock,
                    tab_drag,
                    &pointer_tiles,
                    &dock_host,
                    p.x,
                    p.y,
                    Some(evt.data().pointer_id()),
                );
            },
            onpointercancel: move |_| {
                if let Some(drag) = tab_drag.write().take() {
                    let _ = drag.cancel();
                }
                *grip.write() = None;
            },
            onkeyup: move |evt: dioxus_native::prelude::Event<dioxus_native::prelude::KeyboardData>| {
                crate::ui::keymap::note_key_up(&evt.key());
            },
            onfocusout: {
                move |_| {
                    crate::ui::keymap::forget_modifiers();
                }
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
                    if evt.key() == Key::Escape && tab_drag.peek().is_some() {
                        evt.prevent_default();
                        evt.stop_propagation();
                        if let Some(drag) = tab_drag.write().take() {
                            let _ = drag.cancel();
                        }
                        return;
                    }
                    if evt.key() == Key::Escape && open_menu.peek().is_some() {
                        evt.prevent_default();
                        evt.stop_propagation();
                        open_menu.set(None);
                        return;
                    }
                    crate::ui::keymap::note_key_down(&evt.key());
                    // 打っているかは窓が DOM で決める(`aim_keystrokes`)。flag は欄より長生きする。
                    if crate::ui::keymap::is_typing()
                        || text_editing.read().is_some()
                        || renaming.read().is_some()
                    {
                        println!("PROBE room=input verdict=text-editing key={:?}", evt.key());
                        return;
                    }
                    let modifiers = evt.modifiers();
                    let primary = crate::ui::keymap::primary_modifier(modifiers);
                    let Some(intent) = crate::ui::keymap::lookup_held(
                        &evt.key(),
                        evt.code(),
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
                            let comp_frame = clock.current_frame();
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
                            clock.seek_frame(clock.current_frame() + delta);
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
                        Intent::Group => {
                            let targets = selection.all();
                            let result = doc.lock().unwrap().group_layers(&targets);
                            match result {
                                Ok(Some(group)) => {
                                    fixture::expand(group);
                                    selection.set(Some(group));
                                    selected.set(Some(group));
                                    *session.project_notice.lock().unwrap() = "Grouped".to_owned();
                                    refresh_layer_projection(
                                        &doc,
                                        layer_rows,
                                        attrs_state,
                                        &timeline_tx,
                                        revision,
                                    );
                                    println!("PROBE room=write verdict=applied Group layer={group:?}");
                                }
                                Ok(None) => println!("PROBE room=write verdict=group-noop reason=no-selection"),
                                Err(error) => {
                                    *session.project_notice.lock().unwrap() = format!("Group failed: {error}");
                                    *revision.write() += 1;
                                    println!("PROBE room=write verdict=apply-error {error}");
                                }
                            }
                        }
                        Intent::Ungroup => {
                            let targets = selection.all();
                            let result = doc.lock().unwrap().ungroup_layers(&targets);
                            match result {
                                Ok(released) if !released.is_empty() => {
                                    selection.set(None);
                                    for layer in released {
                                        if !selection.contains(layer) {
                                            selection.toggle(layer);
                                        }
                                    }
                                    selected.set(selection.get());
                                    *session.project_notice.lock().unwrap() = "Ungrouped".to_owned();
                                    refresh_layer_projection(
                                        &doc,
                                        layer_rows,
                                        attrs_state,
                                        &timeline_tx,
                                        revision,
                                    );
                                    println!("PROBE room=write verdict=applied Ungroup");
                                }
                                Ok(_) => println!("PROBE room=write verdict=ungroup-noop reason=no-group-selection"),
                                Err(error) => {
                                    *session.project_notice.lock().unwrap() = format!("Ungroup failed: {error}");
                                    *revision.write() += 1;
                                    println!("PROBE room=write verdict=apply-error {error}");
                                }
                            }
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
                            let time = clock.current_time();
                            let sec = time.as_seconds_f64();
                            let mut d = doc.lock().unwrap();
                            let mut markers = d.view().markers().unwrap_or_default();
                            let hit = markers
                                .iter()
                                .position(|m| {
                                    (m.time.as_seconds_f64() - sec).abs()
                                        < 0.5 * clock.frame_duration_sec()
                                });
                            match hit {
                                Some(i) => {
                                    markers.remove(i);
                                }
                                None => {
                                    markers.push(crate::doc::store::Marker {
                                        name: format!("{}", markers.len() + 1),
                                        time,
                                        duration: crate::doc::store::RationalTime::ZERO,
                                        body: String::new(),
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
                        Intent::End => clock.seek(clock.duration()),
                        Intent::Deselect => {
                            // 掴んでいる間の `Esc` は**取り消し**。掴んでいない時だけ
                            // 選択を解く(規格が MUST で求める pointercancel の役)。
                            if session.gesture.cancel() {
                                *revision.write() += 1;
                            } else {
                                selection.set(None);
                                selected.set(None);
                                *session.focus.lock().unwrap() = None;
                            }
                        }
                        Intent::PlayPause => {
                            clock.toggle();
                            playing.set(clock.playing());
                        }
                        Intent::Undo | Intent::Redo => {
                            let steps = if matches!(intent, Intent::Undo) { -1 } else { 1 };
                            history_step(&doc, layer_rows, attrs_state, &timeline_tx, revision, steps);
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
                            let frame = clock.current_frame();
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
                                .and_then(|l| rows.iter().position(|r| r.layer == Some(l)))
                                .unwrap_or(0) as i32;
                            let next = (current + delta).clamp(0, rows.len() as i32 - 1) as usize;
                            selection.set(rows[next].layer);
                            selected.set(rows[next].layer);
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

            MenuDismiss { open: open_menu }
            ChoiceDismiss { open: panes.inspector_choice }
            {file_drop_overlay(&session)}

            if tab_drag().is_some() {
                div { class: "dock-capture", aria_hidden: "true" }
            }

            if let Some(drag) = tab_drag().filter(|drag| drag.dragging()) {
                div {
                    class: "dock-ghost",
                    style: "left: {drag.cursor.x + 12.0}px; top: {drag.cursor.y + 12.0}px;",
                    "{drag.panel}"
                }
            }

            div { id: "menubar",
                onmousedown: {
                    let gesture = menubar_gesture.clone();
                    move |_| {
                        gesture.cancel();
                        if let Some(drag) = tab_drag.write().take() {
                            let _ = drag.cancel();
                        }
                        grip.set(None);
                    }
                },
                span { class: "appname", "Motolii" }
                SemanticMenu {
                    id: MenuId::File,
                    label: "File",
                    open: open_menu,
                            div { class: "vrow",
                                SemanticControl {
                                    label: "New",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        let window = window.clone();
                                        let mut revision = panes.revision;
                                        let mut selected = panes.selected;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            let window = window.clone();
                                            dioxus_core::spawn(async move {
                                                if !allow_project_replacement(
                                                    session.clone(),
                                                    poke,
                                                    window,
                                                )
                                                .await
                                                {
                                                    return;
                                                }
                                                session.replace_project(crate::ui::blank_project(), None);
                                                session.selection.set(None);
                                                selected.set(None);
                                                *revision.write() += 1;
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Open…",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        let window = window.clone();
                                        let mut revision = panes.revision;
                                        let mut selected = panes.selected;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            let window = window.clone();
                                            dioxus_core::spawn(async move {
                                                if !allow_project_replacement(
                                                    session.clone(),
                                                    poke,
                                                    window,
                                                )
                                                .await
                                                {
                                                    return;
                                                }
                                                let picked = rfd::AsyncFileDialog::new()
                                                    .add_filter("Motolii", &["rrd"])
                                                    .pick_file()
                                                    .await;
                                                let Some(file) = picked else { return };
                                                match crate::doc::store::Document::load(file.path()) {
                                                    Ok(loaded) => {
                                                        session.replace_project(
                                                            loaded,
                                                            Some(file.path().to_path_buf()),
                                                        );
                                                        session.selection.set(None);
                                                        selected.set(None);
                                                        *session.project_notice.lock().unwrap() = String::new();
                                                        *revision.write() += 1;
                                                    }
                                                    Err(e) => {
                                                        *session.project_notice.lock().unwrap() =
                                                            format!("Open failed: {e}");
                                                        *revision.write() += 1;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Save",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            dioxus_core::spawn(async move {
                                                let _ = put_away(session, poke, false).await;
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Save As…",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            dioxus_core::spawn(async move {
                                                let _ = put_away(session, poke, true).await;
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Import…",
                                    onclick: {
                                        let doc = session.doc.clone();
                                        let project_notice = session.project_notice.clone();
                                        let mut revision = panes.revision;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let doc = doc.clone();
                                            let project_notice = project_notice.clone();
                                            // 選ぶ窓は事象処理の**外**で開ける。中から開けると
                                            // winit が事象の入れ子になって落ちる。
                                            dioxus_core::spawn(async move {
                                                let Some(files) = rfd::AsyncFileDialog::new().pick_files().await else {
                                                    return;
                                                };
                                                let paths = files
                                                    .iter()
                                                    .map(|file| file.path().to_path_buf())
                                                    .collect::<Vec<_>>();
                                                let summary = {
                                                    let mut d = doc.lock().unwrap();
                                                    fixture::admit_paths(&mut d, &paths)
                                                };
                                                println!("PROBE room=browser verdict=import admitted={} of={}", summary.admitted, summary.total);
                                                *project_notice.lock().unwrap() = summary.notice();
                                                if summary.admitted > 0 {
                                                    *revision.write() += 1;
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Export…",
                                    disabled: session.export.is_active(),
                                    onclick: {
                                        let doc = session.doc.clone();
                                        let export = session.export.clone();
                                        let poke = host.poker();
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let doc = doc.clone();
                                            let export = export.clone();
                                            let poke = poke.clone();
                                            dioxus_core::spawn(async move {
                                                // 種の絞りを付けると、OS が拡張子を**もう一度**足して
                                                // `x.mp4.mp4` になる。足すのはこちらの仕事に一本化する。
                                                let picked = rfd::AsyncFileDialog::new()
                                                    .set_file_name("comp.mp4")
                                                    .save_file()
                                                    .await;
                                                let Some(file) = picked else { return };
                                                // 打った名前に既に付いていたら足さない。
                                                let mut out = file.path().to_path_buf();
                                                if out.extension().is_none_or(|e| !e.eq_ignore_ascii_case("mp4")) {
                                                    out.set_extension("mp4");
                                                }
                                                if let Err(error) = export.start(doc, out, poke) {
                                                    println!("PROBE room=export verdict=start-error {error}");
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                }
                SemanticMenu {
                    id: MenuId::View,
                    label: "View",
                    open: open_menu,
                            div { class: "vrow menu-section",
                                SemanticControl {
                                    label: "Reset Layout",
                                    onclick: move |evt: Event<MouseData>| {
                                        evt.stop_propagation();
                                        dock.write().reset_layout();
                                        open_menu.set(None);
                                    }
                                }
                            }
                            for panel in Panel::all() {
                                div { class: "vrow",
                                    SemanticControl {
                                        label: if d.is_visible(panel) { format!("✓ {panel}") } else { format!("  {panel}") },
                                        selected: d.is_visible(panel),
                                        onclick: move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            dock.write().toggle(panel);
                                        }
                                    }
                                    SemanticControl {
                                        label: "Window",
                                        selected: d.is_detached(panel),
                                        secondary: true,
                                        disabled: d.is_detached(panel),
                                        onclick: {
                                            let host = host.clone();
                                            move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                if dock.write().detach(panel) {
                                                    host.open(panel);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                }
                // 窓の都合は作品でないので、面を持たずヘッダに仕舞う。
                SemanticMenu {
                    id: MenuId::Settings,
                    label: "Settings",
                    open: open_menu,
                    SettingsSheet { session: session.clone(), scale_pct: panes.scale_pct }
                }

            }

            div { id: "main",
                {tile_view(
                    dock().root(),
                    dock,
                    tab_drag,
                    grip,
                    split_nodes,
                    tile_nodes,
                    &session,
                    &loaded,
                    panes,
                    playing,
                )}
            }

            // 出した事は**今いる場所**に返す。Output パネルの中だけだと、
            // Stage を見ている人には起きていない事と同じになる。
            div { id: "status",
                span { "{status_line}" }
                OutputStatus { controller: session.export.clone(), surface: OutputSurface::StatusBar, generation: output_generation }
            }
        }
    )
}
