use dioxus_native::prelude::*;

use crate::ui::app::Panes;
use crate::ui::contracts::{Intent, MenuRow};
pub(super) use crate::ui::contracts::{MenuRequest, MenuTarget};
#[cfg(test)]
use crate::ui::functions::table::entries;
use crate::ui::functions::{table, view::ContextPopup};
use crate::ui::semantic_menu::{MenuItems, SemanticControl};
use crate::ui::session::Session;

#[component]
pub(super) fn ContextMenu(session: Session, panes: Panes) -> Element {
    let mut open = panes.context_menu;
    let Some(request) = open() else {
        return rsx! {};
    };
    if let MenuTarget::BrowserMedia(asset) = request.target {
        let info = crate::ui::browser::media_context_info(&session, asset);
        let (name, path, in_use) = info.unwrap_or_else(|| ("Missing media".into(), None, false));
        let can_replace = path.is_some()
            && session
                .selection
                .get()
                .is_some_and(|layer| session.writable(layer));
        return rsx!(BrowserMediaContext {
            session,
            panes,
            request,
            asset,
            name,
            path,
            in_use,
            can_replace,
            ondismiss: move |_| open.set(None),
        });
    }
    let selected = session.selection.all();
    let keys = if matches!(request.target, MenuTarget::TimelineKey { .. }) {
        session.selected_keys.lock().unwrap().len()
    } else {
        0
    };
    let doc = session.doc.lock().unwrap();
    let store = doc.view();
    let editable = selected
        .iter()
        .any(|&layer| crate::ui::session::edit_rejection(&store, layer).is_none());
    let (undo, redo) = doc.history_depth();
    let empty = store.layers().is_empty();
    let rows = table::entries(request.target)
        .into_iter()
        .map(|(label, intent)| MenuRow {
            label: if matches!(intent, Intent::DeleteLayer) && keys > 0 {
                "Delete Keyframes"
            } else {
                label
            }
            .into(),
            hint: crate::ui::keymap::hint(intent),
            intent,
            disabled: match intent {
                Intent::Split
                | Intent::Duplicate
                | Intent::Rename
                | Intent::DeleteLayer
                | Intent::Copy
                | Intent::Cut => {
                    !editable
                }
                Intent::Paste => !session.clipboard.has_payload(),
                Intent::Deselect => selected.is_empty() && keys == 0,
                Intent::SelectAll => empty,
                Intent::Undo => undo == 0,
                Intent::Redo => redo == 0,
                _ => false,
            },
        })
        .collect::<Vec<_>>();
    drop(store);
    drop(doc);
    rsx!(ContextPopup {
        key: "{request:?}",
        request,
        rows,
        ondismiss: move |_| open.set(None),
        onaction: move |intent| {
            open.set(None);
            crate::ui::commands::run(&session, panes, intent);
        },
    })
}

#[derive(Clone, Copy)]
enum BrowserMediaAction {
    Place,
    Replace,
    Reveal,
    Remove,
}

#[component]
fn BrowserMediaContext(
    session: Session,
    panes: Panes,
    request: MenuRequest,
    asset: crate::doc::store::AssetId,
    name: String,
    path: Option<String>,
    in_use: bool,
    can_replace: bool,
    ondismiss: EventHandler<()>,
) -> Element {
    let items = use_context_provider(|| MenuItems::new(true));
    let rows = [
        ("Place as Layer".to_owned(), BrowserMediaAction::Place, path.is_none()),
        (
            "Replace Selected Layer".to_owned(),
            BrowserMediaAction::Replace,
            !can_replace,
        ),
        ("Reveal in Finder".to_owned(), BrowserMediaAction::Reveal, path.is_none()),
        (
            if in_use {
                "Remove from Library (In Use)".to_owned()
            } else {
                "Remove from Library".to_owned()
            },
            BrowserMediaAction::Remove,
            in_use,
        ),
    ];
    rsx!(
        div {
            class: "context-dismiss",
            onmousedown: move |evt| {
                evt.stop_propagation();
                evt.prevent_default();
                ondismiss.call(());
            },
            oncontextmenu: move |evt| {
                evt.stop_propagation();
                evt.prevent_default();
                ondismiss.call(());
            },
        }
        div {
            class: "vmenu context-menu",
            role: "menu",
            aria_label: "Media actions for {name}",
            style: "left: max(0px, min({request.x}px, calc(100vw - 220 * var(--s) * 1px))); top: max(0px, min({request.y}px, calc(100vh - 4 * var(--row) - 2 * var(--sp1) - 2px)));",
            onmounted: move |_| items.focus_first(),
            onmousedown: move |evt| evt.stop_propagation(),
            oncontextmenu: move |evt| { evt.prevent_default(); evt.stop_propagation(); },
            onkeydown: move |evt: KeyboardEvent| {
                evt.stop_propagation();
                if evt.key() == Key::Escape {
                    evt.prevent_default();
                    ondismiss.call(());
                } else if items.navigate(&evt.key()) {
                    evt.prevent_default();
                }
            },
            for (label, action, disabled) in rows {
                SemanticControl {
                    label,
                    disabled,
                    onclick: {
                        let session = session.clone();
                        let path = path.clone();
                        move |evt: MouseEvent| {
                            evt.stop_propagation();
                            ondismiss.call(());
                            match action {
                                BrowserMediaAction::Place => {
                                    crate::ui::browser::place_media_asset(&session, asset, panes)
                                }
                                BrowserMediaAction::Replace => {
                                    crate::ui::browser::replace_with_media_asset(&session, asset, panes)
                                }
                                BrowserMediaAction::Reveal => {
                                    if let Some(path) = &path {
                                        crate::ui::output::reveal_in_finder(std::path::Path::new(path));
                                    }
                                }
                                BrowserMediaAction::Remove => {
                                    crate::ui::browser::remove_media_asset(&session, asset, panes.revision)
                                }
                            }
                        }
                    },
                }
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_context_has_no_destructive_layer_operations() {
        for target in [MenuTarget::Stage, MenuTarget::Timeline] {
            assert!(entries(target).iter().all(|(_, intent)| !matches!(
                intent,
                Intent::Split | Intent::Duplicate | Intent::DeleteLayer | Intent::Rename
            )));
            assert!(entries(target)
                .iter()
                .all(|(_, intent)| crate::ui::keymap::hint(*intent).is_some()));
        }
    }

    #[test]
    fn context_origin_keeps_layer_and_key_domains_distinct() {
        let layer = crate::doc::store::LayerId(7);
        for target in [
            MenuTarget::StageLayer(layer),
            MenuTarget::TimelineLayer(layer),
            MenuTarget::TimelineKey { layer },
        ] {
            assert!(entries(target)
                .iter()
                .any(|(_, intent)| matches!(intent, Intent::DeleteLayer)));
        }
        assert_ne!(
            MenuTarget::StageLayer(layer),
            MenuTarget::TimelineKey { layer }
        );
    }
}
