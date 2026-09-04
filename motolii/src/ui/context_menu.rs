use dioxus_native::prelude::*;

use crate::ui::app::Panes;
use crate::ui::contracts::{Intent, MenuRow};
pub(super) use crate::ui::contracts::{MenuRequest, MenuTarget};
#[cfg(test)]
use crate::ui::functions::table::entries;
use crate::ui::functions::{table, view::ContextPopup};
use crate::ui::session::Session;

#[component]
pub(super) fn ContextMenu(session: Session, panes: Panes) -> Element {
    let mut open = panes.context_menu;
    let Some(request) = open() else {
        return rsx! {};
    };
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
                Intent::Split | Intent::Duplicate | Intent::Rename | Intent::DeleteLayer => {
                    !editable
                }
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
