//! 鍵の配り方 — 窓の shell(winit)と harness が同じ順で通す規則。欄・焦点・IME。
//! 規則は Document でも面でもなく shell の物なので、ここに 1 つ。

use blitz_dom::Document;
use dioxus_native::DioxusDocument;

/// 窓に開く欄。1 行は `input`、書き置きは `textarea`。同時に 1 つ(`Session.field`)。
/// 欄は Session の Field だけ(`.field`)。生の input(枠の設定)は打鍵の道を奪わない。
pub(crate) const FIELD: &str = "input.field, textarea.field";

/// 欄は許可制。押すまで無く、Enter・Escape・**外を押す**のどれでも欄ごと消える。
/// 欄を持つ面がそれぞれ閉じ方を書くのではなく、外を押した時は欄へ Cmd+Enter を送る
/// (書き置きは Enter が改行なので、確定は Cmd 付き)。
/// 窓に欄が在る間は打鍵が全部そこへ行く(`aim_keystrokes`)ので、閉じ損ねは鍵の全喪失になる。
pub(crate) fn commit_field_outside(doc: &mut DioxusDocument, x: f32, y: f32) {
    let Some(field) = doc.inner().query_selector(FIELD).ok().flatten() else { return };
    if doc.inner().hit(x, y).is_some_and(|hit| hit.node_id == field) {
        return;
    }
    commit_field(doc);
}

/// 窓を離れる時も欄は確定して消える(§6b)。
pub(crate) fn commit_field(doc: &mut DioxusDocument) {
    if doc.inner().query_selector(FIELD).ok().flatten().is_none() {
        return;
    }
    aim_keystrokes(doc);
    send_chord(doc, keyboard_types::Key::Enter, keyboard_types::Code::Enter);
}

/// 開いたばかりの欄: 1 行(input)は全選択 — 打てば置き換わる(Finder・AE の名前と同じ)。
/// 書き置き(textarea)は caret を末尾へ。blitz は欄の editor を node より後に作り、その時
/// caret を先頭に置くので、editor が出来るまでは何もせず次の event で再び見る。
pub(crate) fn select_new_field(doc: &mut DioxusDocument, seen: &mut Option<blitz_dom::NodeId>) {
    let field = doc.inner().query_selector(FIELD).ok().flatten();
    if field == *seen {
        return;
    }
    let Some(node) = field else {
        *seen = None;
        return;
    };
    let ready = {
        let inner = doc.inner();
        inner.get_node(node).and_then(|n| {
            let element = n.element_data()?;
            element.text_input_data()?;
            Some(element.name.local.as_ref() == "textarea")
        })
    };
    let Some(multiline) = ready else { return };
    *seen = field;
    aim_keystrokes(doc);
    if multiline {
        send_chord(doc, keyboard_types::Key::End, keyboard_types::Code::End);
    } else {
        send_chord(doc, keyboard_types::Key::Character("a".into()), keyboard_types::Code::KeyA);
    }
}

fn send_chord(doc: &mut DioxusDocument, key: keyboard_types::Key, code: keyboard_types::Code) {
    let event = |state| blitz_traits::events::BlitzKeyEvent {
        key: key.clone(),
        code,
        modifiers: keyboard_types::Modifiers::SUPER,
        location: keyboard_types::Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state,
        text: None,
    };
    doc.handle_ui_event(blitz_traits::events::UiEvent::KeyDown(event(
        blitz_traits::events::KeyState::Pressed,
    )));
    doc.handle_ui_event(blitz_traits::events::UiEvent::KeyUp(event(
        blitz_traits::events::KeyState::Released,
    )));
}

/// 落とした先が机なら参考画像、他は素材。落とす口は 1 つで、役目だけが場所で決まる。
pub(crate) fn drop_role_at(doc: &DioxusDocument, x: f32, y: f32) -> crate::doc::store::AssetRole {
    let inner = doc.inner();
    let desk = inner.query_selector("#desk").ok().flatten();
    let mut cur = inner.hit(x, y).map(|hit| hit.node_id);
    while let (Some(node), Some(desk)) = (cur, desk) {
        if node == desk {
            return crate::doc::store::AssetRole::Reference;
        }
        cur = inner.get_node(node).and_then(|n| n.parent);
    }
    crate::doc::store::AssetRole::Material
}

thread_local! {
    /// 最後に焦点が居た control の場所(親と何番目か)。欄を確定して升が作り直された後も、
    /// 同じ場所に居る新しい節へ返す(NodeId は作り直しで変わる)。
    static LAST_CONTROL: std::cell::Cell<Option<(blitz_dom::NodeId, usize)>> = const { std::cell::Cell::new(None) };
}

fn place_of(doc: &blitz_dom::BaseDocument, node: blitz_dom::NodeId) -> Option<(blitz_dom::NodeId, usize)> {
    let parent = doc.get_node(node)?.parent?;
    let index = doc.get_node(parent)?.children.iter().position(|c| *c == node)?;
    Some((parent, index))
}

fn node_at(doc: &blitz_dom::BaseDocument, place: (blitz_dom::NodeId, usize)) -> Option<blitz_dom::NodeId> {
    doc.get_node(place.0)?.children.get(place.1).copied()
}

/// ⇧Tab は前の焦点へ(上流の Tab は前方だけ)。扱ったら true。
pub(crate) fn step_focus_back(doc: &mut DioxusDocument, key: &keyboard_types::Key, shift: bool) -> bool {
    if *key != keyboard_types::Key::Tab || !shift {
        return false;
    }
    doc.inner_mut().focus_prev_node();
    true
}

/// 打鍵をどこへ配るかを決める。**窓の側と試験の側で同じ規則を通す** —— 分けると、
/// 利用者が歩く道(欄を開けて打つ)の試験が書けない。
pub(crate) fn aim_keystrokes(doc: &mut DioxusDocument) {
    let field = doc.inner().query_selector(FIELD).ok().flatten();
    crate::ui::keymap::set_typing(field.is_some());
    let (target, on_control) = {
        let inner = doc.inner();
        let root = ["#app", "#detached"]
            .into_iter()
            .find_map(|s| inner.query_selector(s).ok().flatten());
        // Tab で button へ移った焦点は奪い返さない(奪うと鍵で何も押せない)。
        let focused = inner.get_focussed_node_id();
        let on_control = field.is_none()
            && focused.is_some_and(|f| {
                Some(f) != root
                    // tabindex="-1"(roving で休んでいる tab)も焦点の持ち主。奪って根へ戻さない。
                    && inner.get_node(f).is_some_and(|n| {
                        n.is_focussable()
                            || n.element_data().is_some_and(|e| e.attr(blitz_dom::local_name!("tabindex")).is_some())
                    })
                    && root.is_some_and(|r| descends_from(&inner, f, r))
            });
        // 欄が閉じて input が消えた直後は、開く前に居た control へ返す(根へ飛ばすと Tab をやり直す)。
        let remembered = LAST_CONTROL
            .with(|c| c.get())
            .filter(|_| field.is_none() && !on_control)
            .and_then(|place| node_at(&inner, place))
            .filter(|n| {
                Some(*n) != root
                    && root.is_some_and(|r| descends_from(&inner, *n, r))
                    && inner.get_node(*n).is_some_and(|x| {
                        x.is_focussable()
                            || x.element_data().is_some_and(|e| e.attr(blitz_dom::local_name!("tabindex")).is_some())
                    })
            });
        if on_control {
            LAST_CONTROL.with(|c| c.set(focused.and_then(|f| place_of(&inner, f))));
        }
        let (target, on_control) = if let Some(back) = remembered {
            LAST_CONTROL.with(|c| c.set(None));
            (Some(back), true)
        } else {
            (if on_control { focused } else { field.or(root) }, on_control)
        };
        (target, on_control)
    };
    crate::ui::keymap::set_on_control(on_control);
    let Some(target) = target else { return };
    let mut inner = doc.inner_mut();
    if inner.get_focussed_node_id() != Some(target) {
        inner.set_focus_to(target);
    }
}

fn descends_from(doc: &blitz_dom::BaseDocument, node: blitz_dom::NodeId, ancestor: blitz_dom::NodeId) -> bool {
    let mut cur = Some(node);
    while let Some(n) = cur {
        if n == ancestor {
            return true;
        }
        cur = doc.get_node(n).and_then(|x| x.parent);
    }
    false
}

/// 焦点のある button を Enter / Space で押す(blitz は鍵で button を押さない)。押したら true。
pub(crate) fn activate_focused_control(doc: &mut DioxusDocument, key: &keyboard_types::Key) -> bool {
    let pressing = matches!(key, keyboard_types::Key::Enter) || matches!(key, keyboard_types::Key::Character(c) if c == " ");
    if !pressing || !crate::ui::keymap::is_on_control() {
        return false;
    }
    let (center, focused) = {
        let inner = doc.inner();
        let Some(f) = inner.get_focussed_node_id() else { return false };
        let Some(node) = inner.get_node(f) else { return false };
        if node.element_data().map(|e| e.name.local.as_ref()) != Some("button") {
            return false;
        }
        let pos = node.absolute_position(0.0, 0.0);
        let size = node.final_layout().size;
        ((pos.x + size.width / 2.0, pos.y + size.height / 2.0), f)
    };

    let event = |buttons| blitz_traits::events::BlitzPointerEvent {
        id: blitz_traits::events::BlitzPointerId::Mouse,
        is_primary: true,
        coords: blitz_traits::events::PointerCoords {
            page_x: center.0,
            page_y: center.1,
            screen_x: center.0,
            screen_y: center.1,
            client_x: center.0,
            client_y: center.1,
        },
        button: blitz_traits::events::MouseEventButton::Main,
        buttons,
        mods: Default::default(),
        details: Default::default(),
        element: Default::default(),
        active_pointers: std::sync::Arc::default(),
    };
    // 上流の harness と同じ形(up でも buttons は Primary)。それ以外だと click にならない。
    doc.handle_ui_event(blitz_traits::events::UiEvent::PointerDown(event(blitz_traits::events::MouseEventButtons::Primary)));
    doc.handle_ui_event(blitz_traits::events::UiEvent::PointerUp(event(blitz_traits::events::MouseEventButtons::Primary)));
    if doc.inner().get_node(focused).is_some() {
        doc.inner_mut().set_focus_to(focused);
    }
    true
}

