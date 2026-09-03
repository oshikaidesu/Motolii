use dioxus_native::prelude::*;

use crate::ui::session::{OpenField, Session};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuId {
    File,
    Edit,
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
            Self::Edit => "edit",
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
    /// 右端の加速鍵(⌘S)。menu は鍵の名簿でもある(Finder・VS Code)。
    #[props(default)] hint: Option<String>,
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
        if let Some(hint) = hint {
            span { class: "khint", "{hint}" }
        }
    })
}

#[component]
pub(super) fn SemanticButton(
    class: String,
    onclick: EventHandler<MouseEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] selected: Option<bool>,
    #[props(default)] aria_label: Option<String>,
    /// hover で下見する物(blend の格子)だけが持つ。
    #[props(default)] onmouseenter: Option<EventHandler<MouseEvent>>,
    #[props(default)] onmouseleave: Option<EventHandler<MouseEvent>>,
    /// 名札(hover で出る)。文字を持たない chip だけが持つ。
    #[props(default)] title: Option<String>,
    children: Element,
) -> Element {
    rsx!(button {
        class: "semantic-button {class}",
        disabled,
        title,
        aria_pressed: selected.map(|on| if on { "true" } else { "false" }),
        aria_label,
        onclick: move |evt| onclick.call(evt),
        onmouseenter: move |evt| if let Some(h) = &onmouseenter { h.call(evt) },
        onmouseleave: move |evt| if let Some(h) = &onmouseleave { h.call(evt) },
        {children}
    })
}

/// 欄。押した間だけ在り、Enter で確定、Escape で消える。外を押した時は host が
/// Cmd+Enter を送る(`host::commit_field_outside`)ので、閉じ方はここ 1 つ。
/// 複数行(書き置き)は Enter が改行で、確定は Cmd+Enter。
#[component]
pub(super) fn Field(
    session: Session,
    class: String,
    #[props(default)] style: String,
    #[props(default)] multiline: bool,
    revision: Signal<u32>,
    oncommit: EventHandler<OpenField>,
) -> Element {
    let draft = session.field().map(|f| f.draft).unwrap_or_default();
    let mut revision = revision;
    let edit = session.clone();
    let oninput = move |evt: FormEvent| edit.edit_field(evt.value());
    let onkeydown = move |evt: KeyboardEvent| {
        evt.stop_propagation();
        // 変換中の Enter は変換の確定。欄の確定ではない。
        if evt.is_composing() {
            return;
        }
        match evt.key() {
            Key::Enter if !multiline || evt.modifiers().intersects(Modifiers::META | Modifiers::SUPER) => {
                evt.prevent_default();
                if let Some(field) = session.close_field() {
                    oncommit.call(field);
                }
                *revision.write() += 1;
            }
            Key::Escape => {
                evt.prevent_default();
                session.close_field();
                *revision.write() += 1;
            }
            _ => {}
        }
    };
    if multiline {
        rsx!(textarea {
            class,
            style,
            value: draft.clone(),
            autofocus: "true",
            oninput,
            onkeydown,
            "{draft}"
        })
    } else {
        rsx!(input {
            class,
            style,
            value: draft,
            autofocus: "true",
            oninput,
            onkeydown,
        })
    }
}
