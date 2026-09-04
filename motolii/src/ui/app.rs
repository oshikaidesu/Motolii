use dioxus_dnd::prelude::GestureEffect;
use dioxus_native::prelude::*;
use dioxus_workbench::{LayoutNode, SplitAxis, SplitId, TileId};

use crate::ui::composition::CompositionSheet;
use crate::ui::context_menu::{ContextMenu, MenuRequest};
use crate::ui::dock::{splitter_delta, Dock, Panel, Side, TabDrag};
use crate::ui::dock_hit::{dock_target_at, dock_zone, node_has_id};
use crate::ui::export_sheet::ExportSheet;
use crate::ui::fixture;
use crate::ui::inspector::{ChoiceDismiss, ChoiceId};
use crate::ui::keymap::Intent;
use crate::ui::output::{OutputStatus, OutputSurface};
use crate::ui::panels::{BrowserPanel, InspectorPanel, StagePanel, TimelinePanel};
use crate::ui::semantic_menu::{MenuDismiss, MenuId, SemanticControl, SemanticMenu};
use crate::ui::session::Session;
use crate::ui::settings::SettingsSheet;
use crate::ui::timeline_widget::TimelineMsg;
use crate::ui::tokens;

/// 落とし先の当たり。**位置を計算しない** —— 5枚の当たりを重ねて置き、
/// どれに乗ったかで決める。
pub(super) const SIDES: [(Side, &str); 5] = [
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

pub(super) fn take_tab_release(
    drag: &mut Option<TabDrag>,
    x: f64,
    y: f64,
    pointer_id: Option<i32>,
) -> Option<(TabDrag, GestureEffect)> {
    let drag = drag.take()?;
    let (_, effect) = drag.release(x, y, pointer_id.unwrap_or_else(|| drag.pointer_id()));
    Some((drag, effect))
}

#[track_caller]
fn live_signal<T: 'static>(signal: Signal<T>) -> bool {
    let result = match signal.try_peek() {
        Ok(_) => true,
        Err(BorrowError::Dropped(error)) => {
            println!("PROBE room=callback verdict=stale-scope {error}");
            false
        }
        Err(error) => panic!("Host callback signal borrow failed: {error}"),
    };
    result
}

#[track_caller]
fn update_live_signal<T: 'static, R>(
    mut signal: Signal<T>,
    update: impl FnOnce(&mut T) -> R,
) -> Option<R> {
    let result = match signal.try_write() {
        Ok(mut value) => Some(update(&mut value)),
        Err(BorrowMutError::Dropped(error)) => {
            println!("PROBE room=callback verdict=stale-scope {error}");
            None
        }
        Err(error) => panic!("Host callback signal borrow failed: {error}"),
    };
    result
}

pub(super) fn finish_tab_release(
    dock: Signal<Dock>,
    tab_drag: Signal<Option<TabDrag>>,
    tile_nodes: &TileNodes,
    host: &crate::ui::host::Host,
    x: f64,
    y: f64,
    pointer_id: Option<i32>,
    outside: bool,
) {
    if !live_signal(dock) || !live_signal(tab_drag) {
        return;
    }
    let terminal =
        update_live_signal(tab_drag, |drag| take_tab_release(drag, x, y, pointer_id)).flatten();
    let Some((drag, effect)) = terminal else {
        return;
    };
    match effect {
        GestureEffect::Tap => {
            let _ = update_live_signal(dock, |dock| dock.set_active(drag.panel));
        }
        GestureEffect::Drop { .. } => {
            // 置き場に落とせば移る。窓の外なら別窓へ出る。窓の中の余白(menubar 等)は何もしない。
            if let Some((target, side)) = dock_target_at(tile_nodes, x, y) {
                let _ = update_live_signal(dock, |dock| dock.drop_onto(drag.panel, &target, side));
            } else if outside
                && update_live_signal(dock, |dock| dock.detach(drag.panel)).unwrap_or(false)
            {
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

/// 窓1枚ぶんの見えかたの状態。Document には入らない物だけ。
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Panes {
    pub(super) context_menu: Signal<Option<MenuRequest>>,
    pub(super) layer_rows: Signal<Vec<fixture::LayerRow>>,
    pub(super) attrs_state: Signal<Vec<(bool, bool, bool)>>,
    pub(super) selected: Signal<Option<crate::doc::store::LayerId>>,
    pub(super) revision: Signal<u32>,
    pub(super) scroll_y: Signal<f64>,
    /// 他の窓が書いた時に上がる。状態は全窓で1つなので、これで描き直す。
    pub(super) echo: Signal<u32>,
    /// 窓の文字の大きさ(%)。設定を1枚へ集めるため Settings が触る。
    pub(super) scale_pct: Signal<u32>,
    /// 再生位置。値を出す側はこれを見て描き直す。
    pub(super) playhead: Signal<f64>,
    pub(super) inspector_choice: Signal<Option<ChoiceId>>,
}

fn panes_for(session: &Session) -> Panes {
    let rows = fixture::layer_rows_from_doc(&session.doc.lock().unwrap());
    Panes {
        context_menu: use_signal(|| None),
        scale_pct: use_signal(|| session.scale.percent()),
        playhead: use_signal(|| session.clock.now_sec()),
        inspector_choice: use_signal(|| None),
        layer_rows: use_signal(|| rows.clone()),
        attrs_state: use_signal(|| {
            rows.iter()
                .map(|r| (r.hidden, r.solo, r.locked))
                .collect::<Vec<_>>()
        }),
        selected: use_signal(|| session.selection.get()),
        revision: use_signal(|| 0u32),
        scroll_y: use_signal(|| 0.0f64),
        echo: use_signal(|| 0u32),
    }
}

type EpochRegistration = std::rc::Rc<std::cell::Cell<(Option<u64>, Option<u64>)>>;

fn use_epoch_registration(
    host: &crate::ui::host::Host,
    register: impl FnOnce() -> Option<u64>,
    remove: fn(&crate::ui::host::Host, u64),
) -> EpochRegistration {
    let lease = use_hook(|| std::rc::Rc::new(std::cell::Cell::new((None, None))));
    let epoch = host.patch_epoch();
    if lease.get().0 != Some(epoch) {
        if let Some(token) = lease.get().1 {
            remove(host, token);
        }
        lease.set((Some(epoch), register()));
    }
    let retained = lease.clone();
    let owner = host.clone();
    use_drop(move || {
        if let Some(token) = retained.take().1 {
            remove(&owner, token);
        }
    });
    lease
}

/// 窓をまたいだ描き直しを繋ぐ。自分が書いたら他の窓を起こし、他の窓に起こされたら
/// 自分を描き直す。起こされた側は `echo` だけが動くので、起こし返しは起きない。
fn wire_windows(host: &crate::ui::host::Host, panes: Panes) {
    let registration = use_epoch_registration(
        host,
        || {
            let runtime = dioxus_core::Runtime::current();
            let scope = dioxus_core::current_scope_id();
            let echo = panes.echo;
            Some(host.listen(move || {
                if !live_signal(echo) {
                    return;
                }
                runtime.in_scope(scope, move || {
                    let _ = update_live_signal(echo, |echo| *echo += 1);
                });
            }))
        },
        crate::ui::host::Host::unlisten,
    );
    let host = host.clone();
    use_effect(move || {
        // 書き込みの合図。選択は revision を上げない経路なので別に見る。
        let _ = (panes.revision)();
        let _ = (panes.selected)();
        if let Some(token) = registration.get().1 {
            host.wake_others(token);
        }
    });
}

/// パネル1枚の中身。窓が変わっても同じ物を出す。
pub(super) fn panel_body(
    panel: Panel,
    session: &Session,
    _ui: &fixture::UiData,
    p: Panes,
) -> Element {
    let _ = (p.selected)();
    let _ = (p.echo)();
    let selected = session.selection.get();
    match panel {
        Panel::Media | Panel::Effects | Panel::Create | Panel::Colors => rsx!(BrowserPanel {
            session: session.clone(),
            echo: (p.echo)(),
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
            comp_line: fixture::comp_line(&session.doc.lock().unwrap().view()),
            menu: p.context_menu,
        }),
        Panel::Inspector => rsx!(InspectorPanel {
            session: session.clone(),
            echo: (p.echo)(),
            selected,
            revision: p.revision,
            playhead: p.playhead,
            choice_open: p.inspector_choice,
        }),
        Panel::Desk => rsx!(crate::ui::desk::DeskPanel {
            session: session.clone(),
            echo: (p.echo)(),
            revision: p.revision,
            playhead: p.playhead,
            on_history: {
                let session = session.clone();
                let timeline_tx = session.timeline_tx.clone();
                move |steps: i32| {
                    history_step(
                        &session,
                        p.layer_rows,
                        p.attrs_state,
                        &timeline_tx,
                        p.revision,
                        steps,
                    );
                }
            },
        }),
        Panel::Timeline => rsx!(TimelinePanel {
            session: session.clone(),
            echo: (p.echo)(),
            layer_rows: p.layer_rows,
            attrs_state: p.attrs_state,
            selected: p.selected,
            scroll_y: p.scroll_y,
            playhead: p.playhead,
            revision: p.revision,
            menu: p.context_menu,
        }),
    }
}

fn file_drop_overlay(session: &Session) -> Element {
    let count = session.file_drop.count();
    if count == 0 {
        return rsx! {};
    }
    let noun = if count == 1 { "file" } else { "files" };
    // 受け付けられる物だけを数える。覆いが「入る」と言い切って後で断るのは嘘になる。
    let supported = session
        .file_drop
        .paths()
        .iter()
        .filter(|p| {
            p.is_dir()
                || p.extension()
                    .and_then(|e| e.to_str())
                    .and_then(crate::render::media::asset_type_for_extension)
                    .is_some()
        })
        .count();
    let line = if supported == 0 {
        "Nothing here can be imported".to_owned()
    } else if supported < count {
        format!("Drop to import {supported} of {count} {noun}")
    } else {
        format!("Drop {count} {noun} to import")
    };
    rsx!(div {
        class: "file-drop-overlay",
        div { class: "file-drop-card",
            span { class: "line", "{line}" }
            div { class: "sub", "Onto the Desk adds images as references" }
        }
    })
}

/// 別窓。パネル1枚だけを出す。状態は窓をまたいで1つ(Session)。
pub fn detached() -> Element {
    if let Some(runtime) = try_consume_context::<crate::render::engine::CatalogRuntime>() {
        crate::render::engine::bind_catalog_runtime(&runtime);
    }
    let session = use_hook(|| consume_context::<Session>());
    let ui = session.ui.clone();
    let panel = use_hook(|| consume_context::<Panel>());
    let host = use_hook(|| consume_context::<crate::ui::host::Host>());
    let panes = panes_for(&session);
    wire_windows(&host, panes);
    let window = dioxus_core::try_consume_context::<
        std::sync::Arc<dyn dioxus_native::winit::window::Window>,
    >();
    // 別窓で数値を擦って窓の外で放しても、そこで確定する(主窓と同じ線)。
    use_epoch_registration(
        &host,
        || {
            let window = window.as_ref()?;
            let scrub_session = session.clone();
            let waker = host.clone();
            Some(
                host.on_primary_pointer_release(window.id(), move |_, _, _| {
                    if !live_signal(panes.echo) {
                        return;
                    }
                    if crate::ui::inspector::end_scrub(&scrub_session) {
                        waker.wake_all();
                    }
                }),
            )
        },
        crate::ui::host::Host::remove_callback,
    );
    use_epoch_registration(
        &host,
        || {
            let menu = panes.context_menu;
            Some(
                host.on_focus_lost(
                    window
                        .as_ref()
                        .map_or(crate::ui::host::Host::HEADLESS, |w| w.id()),
                    move || {
                        let _ = update_live_signal(menu, |open| *open = None);
                    },
                ),
            )
        },
        crate::ui::host::Host::remove_callback,
    );
    let reduced_motion = window.is_some() && tokens::system_prefers_reduced_motion();
    // 別窓も同じ倍率(150% で左の名前列と右の帯がずれない)。
    let css = tokens::css_root(session.scale.percent(), reduced_motion);
    rsx!(
        style { {css} }
        {tokens::stylesheet()}
        ChoiceDismiss { open: panes.inspector_choice }
        div {
            id: "detached",
            tabindex: "0",
            onkeyup: move |evt: KeyboardEvent| crate::ui::keymap::note_key_up(&evt.key()),
            onkeydown: {
                let session = session.clone();
                let mut context = panes.context_menu;
                move |evt: KeyboardEvent| {
                    if context.peek().is_some() {
                        evt.stop_propagation();
                        if evt.key() == Key::Escape { evt.prevent_default(); context.set(None); }
                        return;
                    }
                    if session.field().is_some() || crate::ui::keymap::is_typing() { return; }
                    if crate::ui::keymap::is_on_control() && matches!(evt.key(), Key::Enter) { return; }
                    crate::ui::keymap::note_key_down(&evt.key());
                    let mods = evt.modifiers();
                    if let Some(intent) = crate::ui::keymap::lookup_held(&evt.key(), evt.code(),
                        crate::ui::keymap::primary_modifier(mods), mods.shift(), mods.alt()) {
                        if crate::ui::commands::run(&session, panes, intent) { evt.prevent_default(); }
                    }
                }
            },
            {panel_body(panel, &session, &ui, panes)}
            ContextMenu { session: session.clone(), panes }
        }
        {file_drop_overlay(&session)}
    )
}

/// 履歴を進める・戻す手は 1 つ。キーも机の履歴も同じ手を使う(行の目・solo・鍵も追従する)。
pub(super) fn history_step(
    session: &Session,
    layer_rows: Signal<Vec<fixture::LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &std::sync::mpsc::Sender<TimelineMsg>,
    revision: Signal<u32>,
    steps: i32,
) {
    let doc = &session.doc;
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
        session.forget_dead_layers();
        refresh_layer_projection(doc, layer_rows, attrs_state, timeline_tx, revision);
    }
    println!("PROBE room=write verdict=history moved={moved}");
}

pub(super) fn refresh_layer_projection(
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
    attrs_state.set(
        rows.iter()
            .map(|row| (row.hidden, row.solo, row.locked))
            .collect(),
    );
    layer_rows.set(rows);
    let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
    *revision.write() += 1;
}

type SplitNodes =
    std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<SplitId, dioxus_native::NodeHandle>>>;
pub(super) type TileNodes =
    std::rc::Rc<std::cell::RefCell<std::collections::BTreeMap<TileId, dioxus_native::NodeHandle>>>;

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
        LayoutNode::Split {
            id,
            axis,
            ratio,
            first,
            second,
        } => {
            let flow = if axis == SplitAxis::Horizontal {
                "row"
            } else {
                "column"
            };
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
                tile.id, panels, shown, &d, dock, tab_drag, tile_nodes, session, ui, panes, playing,
            )
        }
    }
}

pub fn app() -> Element {
    if let Some(runtime) = try_consume_context::<crate::render::engine::CatalogRuntime>() {
        crate::render::engine::bind_catalog_runtime(&runtime);
    }
    let mut playing = use_signal(|| false);
    let layout_file =
        use_hook(|| consume_context::<crate::ui::host::Host>().settings_file("layout.json"));
    let mut dock = use_signal(|| {
        layout_file
            .as_deref()
            .and_then(Dock::load)
            .unwrap_or_default()
    });
    let mut tab_drag = use_signal(|| Option::<TabDrag>::None);
    let mut grip = use_signal(|| Option::<GripDrag>::None);
    // 掴んでいる間は書かない。放した時に 1 回。
    use_effect(move || {
        if let (false, Some(path)) = (grip().is_some() || tab_drag().is_some(), &layout_file) {
            dock().save(path);
        }
    });

    let split_nodes =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new())));
    let tile_nodes =
        use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new())));
    let mut open_menu = use_signal(|| None::<MenuId>);

    let session = use_hook(|| consume_context::<Session>()).clone();
    let loaded = session.ui.clone();
    let panes = panes_for(&session);
    let host = use_hook(|| consume_context::<crate::ui::host::Host>());
    let window = use_hook(|| {
        dioxus_core::try_consume_context::<std::sync::Arc<dyn dioxus_native::winit::window::Window>>(
        )
    });
    // 別窓が閉じたら置き場へ戻す。別の窓からの合図なので、自分の runtime を包んで渡す。
    use_epoch_registration(
        &host,
        || {
            let runtime = dioxus_core::Runtime::current();
            let scope = dioxus_core::current_scope_id();
            Some(host.on_close(move |panel| {
                if !live_signal(dock) {
                    return;
                }
                runtime.in_scope(scope, move || {
                    let _ = update_live_signal(dock, |dock| dock.reattach(panel));
                });
            }))
        },
        crate::ui::host::Host::remove_callback,
    );
    use_epoch_registration(
        &host,
        || {
            let window_id = window
                .as_ref()
                .map_or(crate::ui::host::Host::HEADLESS, |w| w.id());
            let runtime = dioxus_core::Runtime::current();
            let scope = dioxus_core::current_scope_id();
            let tile_nodes = tile_nodes.clone();
            let dock_host = host.clone();
            let scrub_session = session.clone();
            Some(
                host.on_primary_pointer_release(window_id, move |x, y, outside| {
                    if !live_signal(dock) || !live_signal(tab_drag) {
                        return;
                    }
                    runtime.in_scope(scope, || {
                        finish_tab_release(
                            dock,
                            tab_drag,
                            &tile_nodes,
                            &dock_host,
                            x,
                            y,
                            None,
                            outside,
                        );
                    });
                    // 数値を擦ったまま窓の外で放しても、そこで確定する。
                    if crate::ui::inspector::end_scrub(&scrub_session) {
                        dock_host.wake_all();
                    }
                }),
            )
        },
        crate::ui::host::Host::remove_callback,
    );
    use_epoch_registration(
        &host,
        || {
            let runtime = dioxus_core::Runtime::current();
            let scope = dioxus_core::current_scope_id();
            let gesture = session.gesture.clone();
            let lost_session = session.clone();
            Some(
                host.on_focus_lost(
                    window
                        .as_ref()
                        .map_or(crate::ui::host::Host::HEADLESS, |w| w.id()),
                    move || {
                        if !live_signal(tab_drag)
                            || !live_signal(grip)
                            || !live_signal(open_menu)
                            || !live_signal(panes.context_menu)
                        {
                            return;
                        }
                        gesture.cancel();
                        // 下見(transient)は窓を離れたら全部落とす。書類に無い絵を描き続けない。
                        lost_session.doc.lock().unwrap().clear_all_transients();
                        // Cmd+Tab で離れると修飾の keyup が届かない。戻った時の Space が Cmd+Space になる。
                        crate::ui::keymap::forget_modifiers();
                        runtime.in_scope(scope, move || {
                            let _ = update_live_signal(tab_drag, |drag| {
                                if let Some(drag) = drag.take() {
                                    let _ = drag.cancel();
                                }
                            });
                            let _ = update_live_signal(grip, |grip| *grip = None);
                            let _ = update_live_signal(open_menu, |menu| *menu = None);
                            let _ = update_live_signal(panes.context_menu, |menu| *menu = None);
                        });
                    },
                ),
            )
        },
        crate::ui::host::Host::remove_callback,
    );
    // ffmpeg の有無は起動後 1 回だけ見る(毎 render に子プロセスを起こさない)。
    static FFMPEG_READY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let ffmpeg_ready = *FFMPEG_READY.get_or_init(crate::render::media::tools_available);
    let has_work = {
        let d = session.doc.lock().unwrap();
        let view = d.view();
        // 白紙(comp 無し・層 0)は書き出せない。真っ黒な mp4 を出して「壊れた」と思わせない。
        view.composition().ok().flatten().is_some() && !view.layers().is_empty()
    };
    let can_export = has_work && ffmpeg_ready;
    let export_hint = if !ffmpeg_ready {
        "Install ffmpeg to export video"
    } else if !has_work {
        "Nothing to export yet"
    } else {
        ""
    };
    // 面からの「この panel を前に出して」。revision の度に拾う(Inspector の COLOR 行 → Colors)。
    {
        let asker = session.clone();
        let mut revision = panes.revision;
        use_effect(move || {
            let _ = (panes.revision)();
            let _ = (panes.echo)();
            if let Some(panel) = asker.take_panel_ask() {
                dock.write().set_active(panel);
            }
            // 別の糸が指紋を取り終えた取り込みを棚へ入れる。
            let batches: Vec<_> = std::mem::take(&mut *asker.imports.lock().unwrap());
            if !batches.is_empty() {
                let mut d = asker.doc.lock().unwrap();
                let mut last = String::new();
                for batch in batches {
                    let summary = fixture::admit_prepared(&mut d, batch);
                    println!(
                        "PROBE room=browser verdict=import admitted={} of={}",
                        summary.admitted, summary.total
                    );
                    last = summary.notice();
                }
                drop(d);
                *asker.project_notice.lock().unwrap() = last;
                *revision.write() += 1;
            }
        });
    }
    // 選択が変わったら箱の寸法は一度捨てる(前の層の箱でアンカーの升を押させない)。
    {
        let sizer = session.clone();
        use_effect(move || {
            let _ = (panes.selected)();
            *sizer.selected_size.lock().unwrap() = None;
        });
    }
    use_effect(move || {
        if (panes.context_menu)().is_some() {
            open_menu.set(None);
            let mut choice = panes.inspector_choice;
            choice.set(None);
        }
    });
    let output_generation = (panes.echo)();
    let project_notice = session.project_notice.lock().unwrap().clone();
    let mut status_line = match (project_notice.is_empty(), session.is_dirty()) {
        (true, true) => "Edited".to_owned(),
        (false, true) => format!("{project_notice} · Edited"),
        _ => project_notice,
    };
    // 選んでいるキーは見せる。見えないまま Delete と ⌥矢印の意味が変わるのは予測できない。
    let keys_selected = session.selected_keys.lock().unwrap().len();
    if keys_selected > 0 {
        let word = if keys_selected == 1 {
            "keyframe"
        } else {
            "keyframes"
        };
        status_line = if status_line.is_empty() {
            format!("{keys_selected} {word} selected")
        } else {
            format!("{keys_selected} {word} selected · {status_line}")
        };
    }
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
        echo: layout_echo,
        ..
    } = panes;
    let mut selected = selected_sig;
    let mut layout_echo = layout_echo;

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
        {tokens::stylesheet()}
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
                        *layout_echo.write() += 1;
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
                    false,
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
            onkeydown: {
                let doc = doc.clone();
                let clock = clock.clone();
                let selection = selection.clone();
                let timeline_tx = timeline_tx.clone();
                let mut layer_rows = layer_rows;
                let mut attrs_state = attrs_state;
                let mut revision = revision;
                let poke = host.poker();
                let session = session.clone();
                let window = window.clone();
                let mut choice = panes.inspector_choice;
                let selected_sig = panes.selected;
                move |evt| {
                    if panes.context_menu.peek().is_some() {
                        evt.prevent_default();
                        evt.stop_propagation();
                        if evt.key() == Key::Escape { let mut menu = panes.context_menu; menu.set(None); }
                        return;
                    }
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
                    // Escape は一番内側から 1 枚ずつ(Figma・VS Code)。Inspector の選択肢も 1 枚。
                    if evt.key() == Key::Escape && choice.peek().is_some() {
                        evt.prevent_default();
                        evt.stop_propagation();
                        choice.set(None);
                        return;
                    }
                    // 机の引き出しも 1 枚。開いていれば閉じるだけで、選択には触らない。
                    if evt.key() == Key::Escape
                        && matches!(*session.desk.lock().unwrap(), crate::ui::session::DeskState::Open(_))
                    {
                        evt.prevent_default();
                        evt.stop_propagation();
                        *session.desk.lock().unwrap() = crate::ui::session::DeskState::Shut;
                        *revision.write() += 1;
                        return;
                    }
                    // 焦点が button に在る時の Enter / Space はその button の物。menu が開いている間は鍵を引かない。
                    if crate::ui::keymap::is_on_control()
                        && (evt.key() == Key::Enter || evt.key() == Key::Character(" ".into()))
                    {
                        return;
                    }
                    // menu を開けている間も修飾の記憶は続ける(Cmd を押したまま menu が閉じると Space が Cmd+Space になる)。
                    crate::ui::keymap::note_key_down(&evt.key());
                    let open_now = *open_menu.peek();
                    if let Some(open) = open_now {
                        // 開いた鍵でもう一度押せば閉じる(⌥⌘K で開けた枠の設定を ⌥⌘K で閉じる)。
                        let same_chord = matches!(open, MenuId::Composition)
                            && evt.modifiers().alt()
                            && evt.modifiers().intersects(Modifiers::META | Modifiers::SUPER)
                            && crate::ui::keymap::code_to_char(evt.code()) == Some('k');
                        if same_chord {
                            evt.prevent_default();
                            evt.stop_propagation();
                            open_menu.set(None);
                        }
                        return;
                    }
                    // 打っているかは窓が DOM で決める(`aim_keystrokes`)。flag は欄より長生きする。
                    if crate::ui::keymap::is_typing() || session.field().is_some() {
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
                        return;
                    };
                    evt.prevent_default();
                    match intent {
                        Intent::Split | Intent::Duplicate | Intent::StepFrame(_)
                        | Intent::Home | Intent::End | Intent::Deselect | Intent::PlayPause
                        | Intent::Undo | Intent::Redo | Intent::Rename | Intent::DeleteLayer
                        | Intent::SelectAll => {
                            crate::ui::commands::run(&session, panes, intent);
                            playing.set(clock.playing());
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
                            match crate::ui::ease::apply_easy(&session, &starts, side) {
                                Ok(n) => println!(
                                    "PROBE room=write verdict=applied EasyEase tracks={n}"
                                ),
                                Err(e) => {
                                    // 効かなかった事を黙らない(AE の F9 は必ず何かが起きる)。
                                    *session.project_notice.lock().unwrap() = "Select keyframes first".to_owned();
                                    println!("PROBE room=write verdict=apply-error {e}");
                                }
                            }
                            *revision.write() += 1;
                        }
                        Intent::Reveal(property) => {
                            fixture::toggle_reveal(property);
                            if fixture::reveal().is_some() {
                                if let Some(layer) = selected() {
                                    fixture::expand(layer);
                                }
                            }
                            refresh_layer_projection(&doc, layer_rows, attrs_state, &timeline_tx, revision);
                        }
                        Intent::ToggleKeyedOnly => {
                            fixture::toggle_keyed_only();
                            if fixture::keyed_only() {
                                if let Some(layer) = selected() {
                                    fixture::expand(layer);
                                }
                            }
                            refresh_layer_projection(&doc, layer_rows, attrs_state, &timeline_tx, revision);
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
                                // 止まっている時に同じ所を押せば外す。再生中の連打(拍を叩く)では消さない。
                                Some(i) if !clock.playing() => {
                                    markers.remove(i);
                                }
                                Some(_) => return,
                                None => {
                                    markers.push(crate::doc::store::Marker {
                                        // 名前は時刻から。採番は途中を消すと衝突する。
                                        name: clock.format_timecode(),
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
                                // 印を打ったら、その本文を書く場所が開いている(押し直しをさせない)。
                                // 本文を書く場所を開けるのは、机が焦点に付いて回っている時だけ。
                                // 手で閉じた/別の引き出しを開けている人の連打(拍を叩く)を邪魔しない。
                                if hit.is_none()
                                    && *session.desk.lock().unwrap() == crate::ui::session::DeskState::Follow
                                {
                                    *session.desk.lock().unwrap() =
                                        crate::ui::session::DeskState::Open(crate::ui::desk::Drawer::Text);
                                }
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
                        Intent::Save | Intent::SaveAs => {
                            let session = session.clone();
                            let poke = poke.clone();
                            let ask = matches!(intent, Intent::SaveAs);
                            dioxus_core::spawn(async move {
                                let _ = crate::ui::project::put_away(session, poke, ask).await;
                            });
                        }
                        Intent::NewProject => {
                            dioxus_core::spawn(crate::ui::project::new_project(session.clone(), poke.clone(), window.clone(), revision, selected_sig));
                        }
                        Intent::OpenProject => {
                            dioxus_core::spawn(crate::ui::project::open_project(session.clone(), poke.clone(), window.clone(), revision, selected_sig));
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
                            let rows: Vec<_> = fixture::layer_rows_from_doc(&d).into_iter().filter(|r| r.layer.is_some()).collect();
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
                        Intent::SelectExtend(delta) => {
                            let d = doc.lock().unwrap();
                            let rows = fixture::layer_rows_from_doc(&d);
                            drop(d);
                            let rows: Vec<_> = rows.into_iter().filter(|r| r.layer.is_some()).collect();
                            if rows.is_empty() {
                                return;
                            }
                            let Some(current) = selected().and_then(|l| rows.iter().position(|r| r.layer == Some(l))) else { return };
                            let next = (current as i32 + delta).clamp(0, rows.len() as i32 - 1) as usize;
                            if let Some(layer) = rows[next].layer {
                                if !selection.contains(layer) {
                                    selection.toggle(layer);
                                }
                                selected.set(Some(layer));
                                *revision.write() += 1;
                            }
                        }
                        Intent::CompositionSettings => {
                            open_menu.set(Some(MenuId::Composition));
                        }
                        Intent::Quit => {
                            let session = session.clone();
                            let poke = poke.clone();
                            let window = window.clone();
                            dioxus_core::spawn(async move {
                                if crate::ui::project::allow_project_replacement(session.clone(), poke.clone(), window).await {
                                    session.quit.store(true, std::sync::atomic::Ordering::Relaxed);
                                    poke.poke();
                                }
                            });
                        }
                        Intent::View(request) => {
                            *session.view_request.lock().unwrap() = Some(request);
                            *revision.write() += 1;
                        }
                        Intent::Nudge(dx, dy) => {
                            // キーを選んでいる時の Alt+←→ はキーをコマで動かす(AE)。
                            let keys: Vec<_> = session.selected_keys.lock().unwrap().clone();
                            let keys: Vec<_> = keys.into_iter().filter(|k| session.writable(k.layer)).collect();
                            if !keys.is_empty() && dy == 0.0 {
                                let mut d = doc.lock().unwrap();
                                let Ok(fps) = crate::ui::timeline_widget::document_fps(&d) else { return };
                                let by = dx.signum() as i64 * if dx.abs() >= 10.0 { 10 } else { 1 };
                                let mut intents = Vec::new();
                                for k in &keys {
                                    let frame = (k.at_sec * fps.as_f64()).round() as i64;
                                    match crate::ui::timeline_widget::keyframe_move_intents(&d, k.layer, k.property.as_ref(), &[frame], by) {
                                        Ok(more) => intents.extend(more),
                                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                    }
                                }
                                let applied = d.apply_all(intents).is_ok();
                                let canvas = fixture::canvas_rows_from_doc(&d);
                                drop(d);
                                if applied {
                                    for k in session.selected_keys.lock().unwrap().iter_mut() {
                                        k.at_sec += by as f64 / fps.as_f64();
                                    }
                                    let _ = timeline_tx.send(TimelineMsg::SetRows(canvas));
                                    *revision.write() += 1;
                                }
                                return;
                            }
                            let targets = session.editable_selection();
                            if targets.is_empty() {
                                return;
                            }
                            let at = clock.current_time();
                            let mut d = doc.lock().unwrap();
                            let result = crate::ui::stage_widget::nudge_intents(&d, &targets, (dx, dy), at).and_then(|intents| d.apply_all(intents));
                            drop(d);
                            match result {
                                Ok(()) => *revision.write() += 1,
                                Err(error) => *session.project_notice.lock().unwrap() = format!("Move failed: {error}"),
                            }
                        }

                    }
                }
            },

            ContextMenu { session: session.clone(), panes }
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

            div { id: "menubar", role: "menubar",
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
                SemanticMenu {
                    id: MenuId::File,
                    label: "File",
                    open: open_menu,
                            div { class: "vrow",
                                SemanticControl {
                                    label: "New",
                                    hint: "⌘N",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        let window = window.clone();
                                        let revision = panes.revision;
                                        let selected = panes.selected;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            let window = window.clone();
                                            dioxus_core::spawn(crate::ui::project::new_project(session, poke, window, revision, selected));
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Open…",
                                    hint: "⌘O",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        let window = window.clone();
                                        let revision = panes.revision;
                                        let selected = panes.selected;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            let window = window.clone();
                                            dioxus_core::spawn(crate::ui::project::open_project(session, poke, window, revision, selected));
                                        }
                                    }
                                }
                            }
                            // 最近の作品(Mac の Open Recent)。5 件。
                            for path in crate::ui::project::recents().into_iter().take(5) {
                                div { class: "vrow",
                                    SemanticControl {
                                        label: path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
                                        secondary: true,
                                        onclick: {
                                            let session = session.clone();
                                            let poke = host.poker();
                                            let window = window.clone();
                                            let path = path.clone();
                                            let selected_sig = panes.selected;
                                            move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                open_menu.set(None);
                                                let session = session.clone();
                                                let poke = poke.clone();
                                                let window = window.clone();
                                                let path = path.clone();
                                                dioxus_core::spawn(async move {
                                                    if crate::ui::project::allow_project_replacement(session.clone(), poke, window).await {
                                                        crate::ui::project::open_path(&session, path, revision, selected_sig);
                                                    }
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Save",
                                    hint: "⌘S",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            dioxus_core::spawn(async move {
                                                let _ = crate::ui::project::put_away(session, poke, false).await;
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Save As…",
                                    hint: "⇧⌘S",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            dioxus_core::spawn(async move {
                                                let _ = crate::ui::project::put_away(session, poke, true).await;
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
                                        let inbox = session.imports.clone();
                                        let poke = host.poker();
                                        let window = window.clone();
                                        let mut revision = panes.revision;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let doc = doc.clone();
                                            let project_notice = project_notice.clone();
                                            let inbox = inbox.clone();
                                            let poke = poke.clone();
                                            let window = window.clone();
                                            // 選ぶ窓は事象処理の**外**で開ける。中から開けると
                                            // winit が事象の入れ子になって落ちる。
                                            dioxus_core::spawn(async move {
                                                let Some(files) = crate::ui::project::sheet(window.as_deref()).pick_files().await else {
                                                    return;
                                                };
                                                let paths = files
                                                    .iter()
                                                    .map(|file| file.path().to_path_buf())
                                                    .collect::<Vec<_>>();
                                                let _ = &doc;
                                                let count = paths.len();
                                                *project_notice.lock().unwrap() =
                                                    format!("Importing {count} {}…", if count == 1 { "file" } else { "files" });
                                                *revision.write() += 1;
                                                // 指紋(全 byte の SHA-256)は別の糸。窓は止めない。
                                                let inbox = inbox.clone();
                                                let poke = poke.clone();
                                                std::thread::spawn(move || {
                                                    let prepared = fixture::prepare_paths(&paths, crate::doc::store::AssetRole::Material);
                                                    inbox.lock().unwrap().push(prepared);
                                                    poke.poke();
                                                });
                                            });
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Export…",
                                    // 白紙(comp 無し)や ffmpeg 無しは押せない。押せない理由は hint に。
                                    disabled: session.export.is_active() || !can_export,
                                    hint: export_hint,
                                    onclick: move |evt: Event<MouseData>| {
                                        evt.stop_propagation();
                                        open_menu.set(Some(MenuId::Export));
                                    }
                                }
                            }
                            div { class: "vrow menu-section",
                                SemanticControl {
                                    label: "Quit Motolii",
                                    hint: "⌘Q",
                                    onclick: {
                                        let session = session.clone();
                                        let poke = host.poker();
                                        let window = window.clone();
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            open_menu.set(None);
                                            let session = session.clone();
                                            let poke = poke.clone();
                                            let window = window.clone();
                                            dioxus_core::spawn(async move {
                                                if crate::ui::project::allow_project_replacement(session.clone(), poke.clone(), window).await {
                                                    session.quit.store(true, std::sync::atomic::Ordering::Relaxed);
                                                    poke.poke();
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                }
                SemanticMenu {
                    id: MenuId::Edit,
                    label: "Edit",
                    open: open_menu,
                            for (label , hint , steps) in [("Undo", "⌘Z", -1i32), ("Redo", "⇧⌘Z", 1i32)] {
                                div { class: "vrow",
                                    SemanticControl {
                                        label: label,
                                        hint: hint,
                                        onclick: {
                                            let session = session.clone();
                                            let timeline_tx = timeline_tx.clone();
                                            move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                open_menu.set(None);
                                                history_step(&session, layer_rows, attrs_state, &timeline_tx, revision, steps);
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "vrow menu-section",
                                SemanticControl {
                                    label: "History…",
                                    onclick: {
                                        let session = session.clone();
                                        let mut revision = revision;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            *session.desk.lock().unwrap() = crate::ui::session::DeskState::Open(crate::ui::desk::Drawer::History);
                                            session.ask_panel(Panel::Desk);
                                            *revision.write() += 1;
                                            open_menu.set(None);
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
                                    label: "Reset Panel Layout",
                                    onclick: {
                                        let notice = session.project_notice.clone();
                                        let mut revision = revision;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            dock.write().reset_layout();
                                            *notice.lock().unwrap() = "Panel layout reset".to_owned();
                                            *revision.write() += 1;
                                            open_menu.set(None);
                                        }
                                    }
                                }
                            }
                            div { class: "vrow",
                                SemanticControl {
                                    label: "Output Only",
                                    checked: session.output_only.load(std::sync::atomic::Ordering::Relaxed),
                                    hint: "Stage",
                                    onclick: {
                                        let flag = session.output_only.clone();
                                        let mut revision = revision;
                                        move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            let next = !flag.load(std::sync::atomic::Ordering::Relaxed);
                                            flag.store(next, std::sync::atomic::Ordering::Relaxed);
                                            *revision.write() += 1;
                                            open_menu.set(None);
                                        }
                                    }
                                }
                            }
                            for panel in Panel::all() {
                                div { class: "vrow",
                                    SemanticControl {
                                        label: "{panel}",
                                        checked: d.is_visible(panel),
                                        onclick: move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            dock.write().toggle(panel);
                                            open_menu.set(None);
                                        }
                                    }
                                    SemanticControl {
                                        label: "Window",
                                        aria_label: "Open {panel} in a new window",
                                        selected: d.is_detached(panel),
                                        secondary: true,
                                        onclick: {
                                            let host = host.clone();
                                            move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                let already_detached = dock.peek().is_detached(panel);
                                                if already_detached || dock.write().detach(panel) {
                                                    host.open(panel);
                                                }
                                                open_menu.set(None);
                                            }
                                        }
                                    }
                                }
                            }
                }
                // 枠は作品の物。File の隣に置く(AE の Composition ▸ Composition Settings ⌘K)。
                SemanticMenu {
                    id: MenuId::Composition,
                    label: "Composition",
                    open: open_menu,
                    CompositionSheet { session: session.clone(), revision }
                }
                SemanticMenu {
                    id: MenuId::Export,
                    label: "Export",
                    open: open_menu,
                    ExportSheet { session: session.clone(), revision, poke: host.poker(), window: window.clone(), open: open_menu }
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
            div { id: "status", role: "status", aria_live: "polite",
                span { "{status_line}" }
                OutputStatus { controller: session.export.clone(), surface: OutputSurface::StatusBar, generation: output_generation }
            }
        }
    )
}

#[cfg(test)]
mod host_callback_lifetimes {
    use super::*;

    fn empty() -> Element {
        rsx! {}
    }

    #[test]
    fn dropped_drag_scope_rejects_release_without_changing_live_dock() {
        let dock_dom = VirtualDom::new(empty);
        let dock = dock_dom.in_scope(ScopeId::ROOT, || Signal::new(Dock::default()));
        let before = dock.peek().clone();
        let drag_dom = VirtualDom::new(empty);
        let drag = drag_dom.in_scope(ScopeId::ROOT, || {
            Signal::new(Some(
                TabDrag::pressed(Panel::Inspector, 10.0, 10.0, 7)
                    .move_to(30.0, 10.0, 7)
                    .0,
            ))
        });
        drop(drag_dom);
        assert!(matches!(drag.try_peek(), Err(BorrowError::Dropped(_))));
        let host = crate::ui::host::Host::for_tests();
        finish_tab_release(
            dock,
            drag,
            &TileNodes::default(),
            &host,
            -100.0,
            -100.0,
            None,
            true,
        );
        assert!(*dock.peek() == before);
        assert!(update_live_signal(drag, |drag| drag.take()).is_none());
    }

    #[test]
    fn live_borrow_conflicts_are_not_treated_as_disposed_callbacks() {
        let dom = VirtualDom::new(empty);
        dom.in_scope(ScopeId::ROOT, || {
            let mut signal = Signal::new(7u32);
            let alias = signal;
            let borrowed = signal.peek();
            let write_failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                update_live_signal(alias, |value| *value += 1)
            }));
            assert!(write_failure.is_err());
            drop(borrowed);
            assert_eq!(*signal.peek(), 7);
            let borrowed = signal.write();
            let read_failure =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| live_signal(alias)));
            assert!(read_failure.is_err());
            drop(borrowed);
        });
    }
}
