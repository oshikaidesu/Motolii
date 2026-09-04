use crate::ui::app::TileNodes;
use crate::ui::app::{panel_body, Panes, SIDES};
use crate::ui::dock::{Dock, Panel, Side, TabDrag};
use crate::ui::fixture;
use crate::ui::session::Session;
use dioxus_native::prelude::*;
use dioxus_workbench::TileId;

pub(super) fn dock_side_at(x: f64, y: f64, width: f64, height: f64) -> Side {
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

pub(super) fn node_has_id(node: &blitz_dom::Node, expected: &str) -> bool {
    node.attr(blitz_dom::local_name!("id"))
        .is_some_and(|id| id == expected)
}

pub(super) fn dock_target_at(tile_nodes: &TileNodes, x: f64, y: f64) -> Option<(TileId, Side)> {
    let mounted = tile_nodes
        .borrow()
        .iter()
        .map(|(id, node)| (id.clone(), node.clone()))
        .collect::<Vec<_>>();
    for (id, handle) in mounted {
        let Some(doc) = handle.try_doc() else {
            continue;
        };
        let Some(node) = doc.get_node(handle.node_id()) else {
            continue;
        };
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
pub(super) fn dock_zone(
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
    let mut tab_handles =
        use_signal(std::collections::BTreeMap::<Panel, std::rc::Rc<MountedData>>::new);
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
            div { class: "ptabs", role: "tablist",
                // ←→ Home End で面を切り替える(tablist の作法。Tab は 1 停止だけ)。
                onkeydown: {
                    let panels = panels.clone();
                    move |evt: KeyboardEvent| {
                        let n = panels.len();
                        if n == 0 {
                            return;
                        }
                        let cur = panels.iter().position(|p| dock.read().is_active(*p)).unwrap_or(0);
                        let next = match evt.key() {
                            Key::ArrowLeft => (cur + n - 1) % n,
                            Key::ArrowRight => (cur + 1) % n,
                            Key::Home => 0,
                            Key::End => n - 1,
                            _ => return,
                        };
                        evt.stop_propagation();
                        evt.prevent_default();
                        dock.write().set_active(panels[next]);
                        if let Some(handle) = tab_handles.read().get(&panels[next]).cloned() {
                            crate::ui::semantic_menu::focus_and_reveal(handle);
                        }
                    }
                },
                for panel in panels.iter().copied() {
                    button {
                        id: "dock-tab-{panel}",
                        role: "tab",
                        aria_selected: if d.is_active(panel) { "true" } else { "false" },
                        aria_controls: "zbody-{panel}",
                        tabindex: if d.is_active(panel) { "0" } else { "-1" },
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
                        onmounted: move |evt: MountedEvent| {
                            tab_handles.write().insert(panel, evt.data());
                        },
                        onpointerdown: move |evt: PointerEvent| {
                            if evt.data().trigger_button()
                                != Some(
                                    dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary,
                                )
                            {
                                return;
                            }
                            if let Some(handle) = tab_handles.read().get(&panel).cloned() {
                                crate::ui::semantic_menu::focus_mounted(handle);
                            }
                            let p = evt.data().client_coordinates();
                            tab_drag.set(Some(TabDrag::pressed(
                                panel,
                                p.x,
                                p.y,
                                evt.data().pointer_id(),
                            )));
                        },
                        onpointerup: move |evt: PointerEvent| {
                            if evt.data().is_primary()
                                && evt.data().trigger_button()
                                    == Some(
                                        dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary,
                                    )
                            {
                                if let Some(handle) = tab_handles.read().get(&panel).cloned() {
                                    crate::ui::semantic_menu::focus_mounted(handle);
                                }
                            }
                        },
                        onclick: move |evt| {
                            evt.prevent_default();
                            if let Some(handle) = tab_handles.read().get(&panel).cloned() {
                                crate::ui::semantic_menu::focus_mounted(handle);
                            }
                        },
                        "{panel}"
                        if d.is_active(panel) { span { class: "a11y", "selected" } }
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
                div { class: "zbody", id: "zbody-{panel}", role: "tabpanel", aria_labelledby: "dock-tab-{panel}", {panel_body(panel, session, ui, panes)} }
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
