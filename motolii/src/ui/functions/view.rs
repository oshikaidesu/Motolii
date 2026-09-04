use dioxus_native::prelude::*;

use crate::ui::contracts::{Intent, MenuRequest, MenuRow};
use crate::ui::semantic_menu::{MenuItems, SemanticControl};

#[component]
pub(crate) fn ContextPopup(
    request: MenuRequest,
    rows: Vec<MenuRow>,
    onaction: EventHandler<Intent>,
    ondismiss: EventHandler<()>,
) -> Element {
    let items = use_context_provider(|| MenuItems::new(true));
    let count = rows.len();
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
            aria_label: "Context menu",
            style: "left: max(0px, min({request.x}px, calc(100vw - 220 * var(--s) * 1px))); top: max(0px, min({request.y}px, calc(100vh - {count} * var(--row) - 2 * var(--sp1) - 2px)));",
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
            for row in rows {
                SemanticControl {
                    label: row.label,
                    hint: row.hint,
                    disabled: row.disabled,
                    onclick: move |evt: MouseEvent| {
                        evt.stop_propagation();
                        onaction.call(row.intent);
                    },
                }
            }
        }
    )
}
