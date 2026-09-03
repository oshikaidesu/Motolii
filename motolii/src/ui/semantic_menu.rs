use dioxus_native::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuId {
    File,
    View,
    Settings,
}

#[component]
pub(super) fn MenuDismiss(mut open: Signal<Option<MenuId>>) -> Element {
    if open().is_none() {
        return rsx! {};
    }
    rsx!(div {
        class: "menu-dismiss",
        role: "presentation",
        onmousedown: move |evt| {
            evt.stop_propagation();
            open.set(None);
        },
    })
}

impl MenuId {
    fn slug(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::View => "view",
            Self::Settings => "settings",
        }
    }
}

#[component]
pub(super) fn SemanticMenu(
    id: MenuId,
    label: String,
    mut open: Signal<Option<MenuId>>,
    children: Element,
) -> Element {
    let mut trigger = use_signal(|| None::<std::rc::Rc<MountedData>>);
    use_effect(move || {
        if open() == Some(id) {
            if let Some(handle) = trigger() {
                dioxus_core::spawn(async move {
                    let _ = handle.set_focus(true).await;
                });
            }
        }
    });
    let shown = open() == Some(id);
    let trigger_id = format!("menu-{}", id.slug());
    let list_id = format!("{trigger_id}-list");

    rsx!(
        div {
            class: if shown { "menu-shell on" } else { "menu-shell" },
            onkeydown: move |evt: KeyboardEvent| {
                if evt.key() == Key::Escape {
                    evt.prevent_default();
                    evt.stop_propagation();
                    open.set(None);
                }
            },
            button {
                id: "{trigger_id}",
                class: "menu",
                aria_haspopup: "menu",
                aria_expanded: if shown { "true" } else { "false" },
                onmounted: move |evt: MountedEvent| trigger.set(Some(evt.data())),
                onclick: move |_| {
                    open.set(if open() == Some(id) { None } else { Some(id) });
                },
                "{label}"
            }
            if shown {
                div {
                    id: "{list_id}",
                    class: "vmenu",
                    role: "menu",
                    onmousedown: move |evt| evt.stop_propagation(),
                    {children}
                }
            }
        }
    )
}

#[component]
pub(super) fn SemanticControl(
    label: String,
    onclick: EventHandler<MouseEvent>,
    #[props(default)] selected: bool,
    #[props(default)] secondary: bool,
    #[props(default)] disabled: bool,
) -> Element {
    let class = match (secondary, selected) {
        (true, true) => "vout on",
        (true, false) => "vout",
        (false, true) => "vitem on",
        (false, false) => "vitem",
    };
    rsx!(button {
        class: class,
        role: "menuitem",
        disabled: disabled,
        onclick: move |evt| onclick.call(evt),
        "{label}"
    })
}

#[component]
pub(super) fn SemanticButton(
    class: String,
    onclick: EventHandler<MouseEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] selected: Option<bool>,
    #[props(default)] aria_label: Option<String>,
    children: Element,
) -> Element {
    rsx!(button {
        class: "semantic-button {class}",
        disabled,
        aria_pressed: selected.map(|on| if on { "true" } else { "false" }),
        aria_label,
        onclick: move |evt| onclick.call(evt),
        {children}
    })
}

/// 欄。押した間だけ在り、Enter で確定、Escape で消える。外を押した時と焦点喪失は
/// host が Enter を送る(`host::commit_field_outside`)ので、閉じ方はここ 1 つ。
#[component]
pub(super) fn Field(
    class: String,
    #[props(default)] style: String,
    value: String,
    oninput: EventHandler<String>,
    oncommit: EventHandler<()>,
    oncancel: EventHandler<()>,
) -> Element {
    rsx!(input {
        class,
        style,
        value,
        autofocus: "true",
        oninput: move |evt| oninput.call(evt.value()),
        onkeydown: move |evt| {
            evt.stop_propagation();
            match evt.key() {
                Key::Enter => {
                    evt.prevent_default();
                    oncommit.call(());
                }
                Key::Escape => {
                    evt.prevent_default();
                    oncancel.call(());
                }
                _ => {}
            }
        },
    })
}
