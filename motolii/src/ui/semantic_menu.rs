use dioxus_native::prelude::*;

use crate::ui::session::{OpenField, Session};

struct DomWork {
    runtime: std::rc::Rc<dioxus_core::Runtime>,
    scope: dioxus_core::ScopeId,
    run: Box<dyn FnOnce()>,
}

thread_local! {
    /// MountedData synchronously borrows Blitz's BaseDocument even Dioxus is still
    /// polling that same document inside an event callback. Queue those operations
    /// until the shell/harness has returned from `DioxusDocument::poll`.
    static DOM_WORK: std::cell::RefCell<Vec<DomWork>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn queue_dom_work(run: impl FnOnce() + 'static) {
    DOM_WORK.with(|pending| {
        pending.borrow_mut().push(DomWork {
            runtime: dioxus_core::Runtime::current(),
            scope: dioxus_core::current_scope_id(),
            run: Box::new(run),
        });
    });
}

/// Run mounted-node operations only after Blitz/Dioxus released its document
/// borrow. Returns how many operations were applied so harnesses can settle again.
pub(crate) fn flush_dom_work() -> usize {
    let mut total = 0;
    loop {
        let work = DOM_WORK.with(|pending| std::mem::take(&mut *pending.borrow_mut()));
        if work.is_empty() {
            return total;
        }
        total += work.len();
        for DomWork {
            runtime,
            scope,
            run,
        } in work
        {
            runtime.in_scope(scope, run);
        }
    }
}

fn node_handle(handle: &std::rc::Rc<MountedData>) -> Option<dioxus_native::NodeHandle> {
    handle.downcast::<dioxus_native::NodeHandle>().cloned()
}

fn apply_focus_and_reveal(handle: &std::rc::Rc<MountedData>) {
    let Some(handle) = node_handle(handle) else {
        return;
    };
    let id = handle.node_id();
    let mut doc = handle.doc_mut();
    if doc.get_node(id).is_none() {
        return;
    }
    doc.set_focus_to(id);
    reveal_node(&mut doc, id);
}

pub(crate) fn reveal_node(doc: &mut blitz_dom::BaseDocument, id: blitz_dom::NodeId) {
    if doc.get_node(id).is_none() {
        return;
    }
    doc.scroll_into_view(
        id,
        blitz_dom::ScrollBehavior::Instant,
        blitz_dom::ScrollLogicalPosition::Nearest,
        blitz_dom::ScrollLogicalPosition::Nearest,
    );

    // The pinned implementation can leave the last item under a sticky footer
    // because nearest-edge scroll has no clearance. Center it in the first real
    // scroll ancestor when one exists.
    let Some(target) = doc.get_client_bounding_rect(id) else {
        return;
    };
    let mut parent = doc.get_node(id).and_then(|node| node.parent);
    while let Some(candidate) = parent {
        let info = doc.get_node(candidate).map(|node| {
            let layout = node.final_layout();
            (
                node.parent,
                layout.scroll_height() as f64,
                f64::from(layout.size.height),
                *node.scroll_offset(),
            )
        });
        let Some((next, scroll_height, _client_height, offset)) = info else {
            break;
        };
        if scroll_height > 0.5 {
            if let Some(viewport) = doc.get_client_bounding_rect(candidate) {
                let target_center = target.y + target.height * 0.5;
                let viewport_center = viewport.y + viewport.height * 0.5;
                doc.scroll_to(
                    candidate,
                    offset.x,
                    offset.y + target_center - viewport_center,
                    blitz_dom::ScrollBehavior::Instant,
                );
            }
            break;
        }
        parent = next;
    }
}

fn apply_focus(handle: &std::rc::Rc<MountedData>) {
    let Some(handle) = node_handle(handle) else {
        return;
    };
    let id = handle.node_id();
    let mut doc = handle.doc_mut();
    if doc.get_node(id).is_some() {
        doc.set_focus_to(id);
    }
}

fn mounted_rect(handle: &std::rc::Rc<MountedData>) -> Option<FocusRect> {
    let handle = node_handle(handle)?;
    let key = String::new();
    let rect = handle
        .doc_mut()
        .get_client_bounding_rect(handle.node_id())?;
    Some(FocusRect {
        key,
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    })
}

fn primary_pointer(event: &PointerEvent) -> Option<(String, i32)> {
    (event.data().is_primary()
        && event.data().trigger_button()
            == Some(dioxus_native::prelude::dioxus_elements::input_data::MouseButton::Primary))
    .then(|| (event.data().pointer_type(), event.data().pointer_id()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuId {
    File,
    Edit,
    View,
    /// 枠の設定(⌘K)。
    Composition,
    /// 書き出しの sheet(範囲・サマリ)。
    Export,
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
            Self::Composition => "composition",
            Self::Export => "export",
            Self::Settings => "settings",
        }
    }
}

/// 開いている menu の項目。↑↓ で焦点を回す為に、項目は載った順にここへ名乗る。
#[derive(Clone, Copy)]
pub(super) struct MenuItems {
    handles: Signal<Vec<std::rc::Rc<MountedData>>>,
    cursor: Signal<Option<usize>>,
    focus_on_mount: bool,
}

impl MenuItems {
    pub(super) fn new(focus_on_mount: bool) -> Self {
        Self {
            handles: Signal::new(Vec::new()),
            cursor: Signal::new(None),
            focus_on_mount,
        }
    }

    pub(super) fn focus_first(self) {
        self.focus(0);
    }

    fn focus(mut self, index: usize) {
        let handle = self.handles.read().get(index).cloned();
        if let Some(handle) = handle {
            self.cursor.set(Some(index));
            queue_dom_work(move || {
                apply_focus_and_reveal(&handle);
            });
        }
    }

    pub(super) fn navigate(self, key: &Key) -> bool {
        let count = self.handles.read().len();
        if count == 0 {
            return false;
        }
        let cursor = (self.cursor)();
        let next = match key {
            Key::ArrowDown => cursor.map_or(0, |c| (c + 1) % count),
            Key::ArrowUp => cursor.map_or(count - 1, |c| (c + count - 1) % count),
            Key::Home => 0,
            Key::End => count - 1,
            _ => return false,
        };
        self.focus(next);
        true
    }

    fn register(mut self, handle: std::rc::Rc<MountedData>) {
        let first = {
            let mut handles = self.handles.write();
            if handles.iter().any(|h| std::rc::Rc::ptr_eq(h, &handle)) {
                return;
            }
            handles.push(handle);
            handles.len() == 1
        };
        if first && self.focus_on_mount {
            self.focus_first();
        }
    }
}

/// Mounted controls in a keyboard-navigable grid. Selection stays with the grid's meaning owner;
/// this registry only moves focus and keeps the focused item visible.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FocusableItems {
    handles: Signal<Vec<(String, std::rc::Rc<MountedData>)>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpatialDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone)]
struct FocusRect {
    key: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl FocusableItems {
    pub(super) fn new() -> Self {
        Self {
            handles: Signal::new(Vec::new()),
        }
    }

    fn register(mut self, key: String, handle: std::rc::Rc<MountedData>) {
        let mut handles = self.handles.write();
        if let Some((_, current)) = handles.iter_mut().find(|(known, _)| *known == key) {
            *current = handle;
        } else {
            handles.push((key, handle));
        }
    }

    pub(super) fn focus(self, key: &str) -> bool {
        let handle = self
            .handles
            .read()
            .iter()
            .find_map(|(known, handle)| (known == key).then(|| handle.clone()));
        let Some(handle) = handle else { return false };
        focus_and_reveal(handle);
        true
    }

    pub(super) fn move_spatial(
        self,
        current: &str,
        allowed: &[String],
        direction: SpatialDirection,
        moved: impl FnOnce(String) + 'static,
    ) -> bool {
        let allowed = allowed
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let handles = self
            .handles
            .read()
            .iter()
            .filter(|(key, _)| allowed.contains(key.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !handles.iter().any(|(key, _)| key == current) {
            return false;
        }
        let current = current.to_owned();
        queue_dom_work(move || {
            let mut rects = Vec::with_capacity(handles.len());
            for (key, handle) in &handles {
                if let Some(mut rect) = mounted_rect(handle) {
                    rect.key = key.clone();
                    rects.push(rect);
                }
            }
            let Some(next) = spatial_neighbor(&rects, &current, direction) else {
                return;
            };
            moved(next.clone());
            if let Some((_, handle)) = handles.iter().find(|(key, _)| key == &next) {
                apply_focus_and_reveal(handle);
            }
        });
        true
    }

    pub(super) fn items_intersecting(
        self,
        allowed: &[String],
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        found: impl FnOnce(Vec<String>) + 'static,
    ) -> bool {
        let allowed = allowed
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        let handles = self
            .handles
            .read()
            .iter()
            .filter(|(key, _)| allowed.contains(key.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if handles.is_empty() {
            return false;
        }
        let (left, right) = (left.min(right), left.max(right));
        let (top, bottom) = (top.min(bottom), top.max(bottom));
        queue_dom_work(move || {
            let mut keys = Vec::new();
            for (key, handle) in handles {
                let Some(item) = mounted_rect(&handle) else {
                    continue;
                };
                if item.x + item.width >= left
                    && item.x <= right
                    && item.y + item.height >= top
                    && item.y <= bottom
                {
                    keys.push(key);
                }
            }
            found(keys);
        });
        true
    }
}

fn spatial_neighbor(
    items: &[FocusRect],
    current: &str,
    direction: SpatialDirection,
) -> Option<String> {
    let from = items.iter().find(|item| item.key == current)?;
    let center = |item: &FocusRect| (item.x + item.width / 2.0, item.y + item.height / 2.0);
    let (fx, fy) = center(from);
    let same_row =
        |item: &FocusRect| item.y < from.y + from.height && from.y < item.y + item.height;
    let mut candidates = items
        .iter()
        .filter(|item| item.key != current)
        .filter(|item| {
            let (x, y) = center(item);
            match direction {
                SpatialDirection::Left => same_row(item) && x < fx,
                SpatialDirection::Right => same_row(item) && x > fx,
                SpatialDirection::Up => y < fy,
                SpatialDirection::Down => y > fy,
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        let (ax, ay) = center(a);
        let (bx, by) = center(b);
        let rank = |x: f64, y: f64| match direction {
            SpatialDirection::Left | SpatialDirection::Right => ((x - fx).abs(), (y - fy).abs()),
            SpatialDirection::Up | SpatialDirection::Down => ((y - fy).abs(), (x - fx).abs()),
        };
        rank(ax, ay)
            .partial_cmp(&rank(bx, by))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.key.cmp(&b.key))
    });
    candidates.first().map(|item| item.key.clone())
}

pub(super) fn focus_and_reveal(handle: std::rc::Rc<MountedData>) {
    queue_dom_work(move || {
        apply_focus_and_reveal(&handle);
    });
}

/// Pointer focus must not scroll between down and up; moving the target would
/// turn a valid click into a cross-target release. Keyboard navigation uses the
/// reveal variant above.
pub(super) fn focus_mounted(handle: std::rc::Rc<MountedData>) {
    queue_dom_work(move || apply_focus(&handle));
}

pub(super) fn page_scroll(handle: std::rc::Rc<MountedData>, direction: i32) {
    if direction == 0 {
        return;
    }
    queue_dom_work(move || {
        let Some(handle) = node_handle(&handle) else {
            return;
        };
        let id = handle.node_id();
        let mut doc = handle.doc_mut();
        let Some(node) = doc.get_node(id) else { return };
        let offset = *node.scroll_offset();
        let height = f64::from(node.final_layout().size.height);
        doc.scroll_to(
            id,
            offset.x,
            offset.y + height * f64::from(direction.signum()),
            blitz_dom::ScrollBehavior::Instant,
        );
    });
}

#[component]
pub(super) fn SemanticMenu(
    id: MenuId,
    label: String,
    mut open: Signal<Option<MenuId>>,
    children: Element,
) -> Element {
    let mut trigger = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let trigger_focus = trigger;
    let mut trigger_armed = use_signal(|| None::<(String, i32)>);
    let mut trigger_ready = use_signal(|| None::<(String, i32)>);
    let items = use_context_provider(|| MenuItems::new(false));
    use_effect(move || {
        if open() == Some(id) {
            if let Some(handle) = trigger() {
                focus_and_reveal(handle);
            }
        } else {
            let mut items = items;
            items.handles.write().clear();
            items.cursor.set(None);
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
                    return;
                }
                // ↑↓ Home End で項目を回る(Mac の menu と同じ)。Enter / Space は焦点の項目を押す。
                if items.navigate(&evt.key()) {
                    evt.prevent_default();
                    evt.stop_propagation();
                }
            },
            button {
                id: "{trigger_id}",
                class: "menu",
                aria_haspopup: "menu",
                aria_expanded: if shown { "true" } else { "false" },
                aria_controls: "{list_id}",
                onmounted: move |evt: MountedEvent| trigger.set(Some(evt.data())),
                onpointerdown: move |evt: PointerEvent| {
                    let pointer = primary_pointer(&evt);
                    trigger_armed.set(pointer.clone());
                    trigger_ready.set(None);
                    if pointer.is_some() {
                        if let Some(handle) = trigger_focus.read().as_ref().cloned() {
                            focus_mounted(handle);
                        }
                    }
                },
                onpointerup: move |evt: PointerEvent| {
                    let pointer = primary_pointer(&evt);
                    trigger_ready.set((trigger_armed() == pointer).then_some(pointer).flatten());
                    trigger_armed.set(None);
                },
                onpointercancel: move |_| {
                    trigger_armed.set(None);
                    trigger_ready.set(None);
                },
                onclick: move |evt| {
                    evt.prevent_default();
                    let ready = trigger_ready().is_some();
                    trigger_ready.set(None);
                    if ready {
                        open.set(if open() == Some(id) { None } else { Some(id) });
                    }
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
    #[props(default)]
    hint: Option<String>,
    /// View menu の「出ている / 隠れている」。字の ✓ でなく状態として持つ。
    #[props(default)]
    checked: Option<bool>,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let selected = selected || checked == Some(true);
    let class = match (secondary, selected) {
        (true, true) => "vout on",
        (true, false) => "vout",
        (false, true) => "vitem on",
        (false, false) => "vitem",
    };
    let items = dioxus_core::try_consume_context::<MenuItems>();
    let mut mounted = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut armed = use_signal(|| None::<(String, i32)>);
    let mut click_ready = use_signal(|| None::<(String, i32)>);
    rsx!(button {
        class: class,
        role: if checked.is_some() { "menuitemcheckbox" } else { "menuitem" },
        aria_checked: checked.map(|on| if on { "true" } else { "false" }),
        aria_label,
        disabled: disabled.then_some("true"),
        onmounted: move |evt: MountedEvent| {
            let handle = evt.data();
            mounted.set(Some(handle.clone()));
            if let (Some(items), false) = (items, disabled) {
                items.register(handle);
            }
        },
        onpointerdown: move |evt: PointerEvent| if !disabled {
            let pointer = primary_pointer(&evt);
            armed.set(pointer.clone());
            click_ready.set(None);
            if pointer.is_some() {
                if let Some(handle) = mounted.read().as_ref().cloned() {
                    focus_mounted(handle);
                }
            }
        },
        onpointerup: move |evt: PointerEvent| if !disabled {
            let pointer = primary_pointer(&evt);
            click_ready.set((armed() == pointer).then_some(pointer).flatten());
            armed.set(None);
        },
        onpointercancel: move |_| {
            armed.set(None);
            click_ready.set(None);
        },
        onclick: move |evt| {
            evt.prevent_default();
            let ready = click_ready().is_some();
            click_ready.set(None);
            if !disabled && ready {
                onclick.call(evt)
            }
        },
        // adapter は可視の文字しか名前にしない(aria-* は捨てられる)。状態も文字で。
        if let Some(on) = checked {
            span { class: "vcheck", aria_hidden: "true", if on { "✓" } else { "" } }
            span { class: "a11y", if on { "on" } else { "off" } }
        }
        if let Some(name) = aria_label.clone() {
            span { class: "a11y", "{name}" }
        }
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
    #[props(default)] aria_expanded: Option<String>,
    #[props(default)] aria_haspopup: Option<String>,
    #[props(default)] aria_controls: Option<String>,
    #[props(default)] aria_selected: Option<String>,
    #[props(default)] role: Option<String>,
    #[props(default)] tabindex: Option<String>,
    #[props(default)] style: Option<String>,
    /// Stable identity within an optional [`FocusableItems`] owner.
    #[props(default)]
    focus_key: Option<String>,
    /// hover で下見する物(blend の格子)だけが持つ。
    #[props(default)]
    onmouseenter: Option<EventHandler<MouseEvent>>,
    #[props(default)] onmouseleave: Option<EventHandler<MouseEvent>>,
    #[props(default)] ondoubleclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// 名札(hover で出る)。文字を持たない chip だけが持つ。
    #[props(default)]
    title: Option<String>,
    children: Element,
) -> Element {
    let a11y_name = aria_label.clone();
    let items = dioxus_core::try_consume_context::<MenuItems>();
    let focusable = dioxus_core::try_consume_context::<FocusableItems>();
    let mut mounted = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut armed = use_signal(|| None::<(String, i32)>);
    let mut click_ready = use_signal(|| None::<(String, i32)>);
    let mut double_ready = use_signal(|| false);
    let semantic_class = if focus_key.is_some() {
        format!("semantic-button browser-focusable {class}")
    } else {
        format!("semantic-button {class}")
    };
    rsx!(button {
        class: "{semantic_class}",
        disabled: disabled.then_some("true"),
        tabindex,
        title,
        style,
        role,
        onmounted: move |evt: MountedEvent| {
            if !disabled {
                let handle = evt.data();
                mounted.set(Some(handle.clone()));
                if let Some(items) = items {
                    items.register(handle.clone());
                }
                if let (Some(items), Some(key)) = (focusable, focus_key.clone()) {
                    items.register(key, handle);
                }
            }
        },
        aria_pressed: selected.map(|on| if on { "true" } else { "false" }),
        aria_label,
        aria_expanded,
        aria_haspopup,
        aria_controls,
        aria_selected,
        onpointerdown: move |evt: PointerEvent| if !disabled {
            let pointer = primary_pointer(&evt);
            armed.set(pointer.clone());
            click_ready.set(None);
            double_ready.set(false);
            if pointer.is_some() {
                if let Some(handle) = mounted.read().as_ref().cloned() {
                    focus_mounted(handle);
                }
            }
        },
        onpointerup: move |evt: PointerEvent| if !disabled {
            let pointer = primary_pointer(&evt);
            click_ready.set((armed() == pointer).then_some(pointer).flatten());
            armed.set(None);
        },
        onpointercancel: move |_| {
            armed.set(None);
            click_ready.set(None);
            double_ready.set(false);
        },
        onclick: move |evt| {
            let ready = click_ready().is_some();
            click_ready.set(None);
            double_ready.set(ready);
            if disabled || !ready {
                evt.prevent_default();
                return;
            }
            if let Some(handle) = mounted.read().as_ref().cloned() {
                focus_and_reveal(handle);
            }
            onclick.call(evt)
        },
        onmouseenter: move |evt| if !disabled { if let Some(h) = &onmouseenter { h.call(evt) } },
        onmouseleave: move |evt| if !disabled { if let Some(h) = &onmouseleave { h.call(evt) } },
        ondoubleclick: move |evt| {
            evt.prevent_default();
            let ready = double_ready();
            double_ready.set(false);
            if !disabled && ready { if let Some(h) = &ondoubleclick { h.call(evt) } }
        },
        onkeydown: move |evt| if !disabled { if let Some(h) = &onkeydown { h.call(evt) } },
        // 見えない名前。adapter が aria-label を捨てるので、文字として置く(記号だけの button の為)。
        if let Some(name) = a11y_name.clone() {
            span { class: "a11y", "{name}" }
        }
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
    /// 欄の名前(読み上げ用)。見えない文字として欄の前に置く。
    #[props(default)]
    label: Option<String>,
) -> Element {
    let draft = session.field().map(|f| f.draft).unwrap_or_default();
    let mut revision = revision;
    // DOMから消えた欄は、全windowを見られるHost（harnessではGui::settle）が
    // Sessionと照合して閉じる。component dropは同じFieldの再mountでも走るため使わない。
    let edit = session.clone();
    let oninput = move |evt: FormEvent| edit.edit_field(evt.value());
    let onkeydown = move |evt: KeyboardEvent| {
        evt.stop_propagation();
        // 変換中の Enter は変換の確定。欄の確定ではない。
        if evt.is_composing() {
            return;
        }
        match evt.key() {
            Key::Enter
                if !multiline
                    || evt
                        .modifiers()
                        .intersects(Modifiers::META | Modifiers::SUPER) =>
            {
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
    let name = label.clone();
    if multiline {
        rsx!(if let Some(name) = name.clone() { span { class: "a11y", "{name}" } } textarea {
            class: "{class} field",
            style,
            value: draft.clone(),
            autofocus: "true",
            oninput,
            onkeydown,
            "{draft}"
        })
    } else {
        rsx!(if let Some(name) = name.clone() { span { class: "a11y", "{name}" } } input {
            class: "{class} field",
            style,
            value: draft,
            autofocus: "true",
            oninput,
            onkeydown,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    thread_local! {
        static CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }

    fn add_calls(by: u32) {
        CALLS.with(|calls| calls.set(calls.get() + by));
    }

    fn disabled_controls() -> Element {
        rsx!(
            SemanticButton {
                class: "disabled-button",
                disabled: true,
                onclick: move |_| add_calls(1),
                "Disabled button"
            }
            SemanticControl {
                label: "Disabled control",
                disabled: true,
                onclick: move |_| add_calls(1),
            }
            SemanticControl {
                label: "Enabled control",
                secondary: true,
                onclick: move |_| add_calls(100),
            }
            SemanticButton {
                class: "enabled-button",
                onclick: move |_| add_calls(1),
                "Enabled button"
            }
            SemanticButton {
                class: "other-button",
                onclick: move |_| add_calls(10),
                "Other button"
            }
        )
    }

    #[test]
    fn disabled_semantic_controls_ignore_dispatched_clicks() {
        CALLS.with(|calls| calls.set(0));
        let mut gui = blitz_test_harness::Harness::from_component(disabled_controls);

        // Pinned Blitz dispatches click listeners before its disabled default action.
        gui.click(".disabled-button");
        flush_dom_work();
        gui.pump();
        gui.click(".vitem");
        flush_dom_work();
        gui.pump();
        assert_eq!(CALLS.with(std::cell::Cell::get), 0);
        gui.click(".enabled-button");
        flush_dom_work();
        gui.pump();
        assert_eq!(CALLS.with(std::cell::Cell::get), 1);
        let focused = gui.base().query_selector(".enabled-button").ok().flatten();
        assert_eq!(gui.base().get_focussed_node_id(), focused);

        let from = gui.center_of(".enabled-button");
        let to = gui.center_of(".other-button");
        gui.mouse_down_at(from.0, from.1);
        gui.mouse_up_at(to.0, to.1);
        flush_dom_work();
        gui.pump();
        assert_eq!(
            CALLS.with(std::cell::Cell::get),
            1,
            "release over another button activated it"
        );

        gui.click(".vout");
        flush_dom_work();
        gui.pump();
        assert_eq!(CALLS.with(std::cell::Cell::get), 101);
        let focused = gui.base().query_selector(".vout").ok().flatten();
        assert_eq!(gui.base().get_focussed_node_id(), focused);
    }

    fn grid(columns: usize, count: usize, size: f64, gap: f64) -> Vec<FocusRect> {
        (0..count)
            .map(|index| FocusRect {
                key: index.to_string(),
                x: (index % columns) as f64 * (size + gap),
                y: (index / columns) as f64 * (size + gap),
                width: size,
                height: size,
            })
            .collect()
    }

    #[test]
    fn spatial_focus_follows_rendered_rows_after_a_grid_resize() {
        let colors_four_columns = grid(4, 12, 48.0, 4.0);
        assert_eq!(
            spatial_neighbor(&colors_four_columns, "1", SpatialDirection::Down).as_deref(),
            Some("5"),
        );
        assert_eq!(
            spatial_neighbor(&colors_four_columns, "5", SpatialDirection::Left).as_deref(),
            Some("4"),
        );

        let colors_three_columns = grid(3, 12, 48.0, 4.0);
        assert_eq!(
            spatial_neighbor(&colors_three_columns, "1", SpatialDirection::Down).as_deref(),
            Some("4"),
        );
        let ordinary_two_columns = grid(2, 7, 88.0, 1.0);
        assert_eq!(
            spatial_neighbor(&ordinary_two_columns, "2", SpatialDirection::Down).as_deref(),
            Some("4"),
        );
        assert_eq!(
            spatial_neighbor(&ordinary_two_columns, "4", SpatialDirection::Right).as_deref(),
            Some("5"),
        );
    }
}
