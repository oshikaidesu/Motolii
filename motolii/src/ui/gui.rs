//! 窓を開けずに GUI を動かす。blitz の document をそのまま headless で建て、
//! 座標でクリックする(当たり判定もレイアウトも本番と同じ経路)。
//!
//! **ここは器具で、何も主張しない。** 主張は道ごとに書き直す。

#![allow(dead_code)]

use std::sync::Arc;

use blitz_dom::{Document, DocumentConfig};
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use dioxus_native::DioxusDocument;
use dioxus_native::prelude::VirtualDom;

use crate::ui::app::app;
use crate::ui::fixture::{load_fixture, Loaded};
use crate::ui::session::Session;

const W: u32 = 1600;
const H: u32 = 1000;

struct Gui {
    doc: DioxusDocument,
}

impl Gui {
    fn open() -> Self {
        let Loaded { doc, ui, duration_sec } = load_fixture();
        let mut vdom = VirtualDom::new(app);
        vdom.insert_any_root_context(Box::new(Session::new(doc, duration_sec, ui)));
        vdom.insert_any_root_context(Box::new(crate::ui::host::Host::for_tests()));
        let mut doc = DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport: Some(Viewport::new(W, H, 1.0, ColorScheme::Dark)),
                ..Default::default()
            },
        );
        doc.initial_build();
        doc.inner_mut().resolve(0.0);
        Self { doc }
    }

    fn text_of(&self, node: blitz_dom::NodeId) -> String {
        let mut out = String::new();
        collect_text(&self.doc.inner(), node, &mut out);
        out
    }

    /// 選択子に当たる全部の見出し。並びは DOM の順。
    fn texts(&mut self, selector: &str) -> Vec<String> {
        self.doc
            .inner()
            .query_selector_all(selector)
            .unwrap_or_default()
            .into_iter()
            .map(|n| self.text_of(n).trim().to_string())
            .collect()
    }

    /// 置き場ごとのタブの見出し。並びは DOM の順(左・中・右・下)。
    fn zone_tabs(&mut self) -> Vec<Vec<String>> {
        let strips = self
            .doc
            .inner()
            .query_selector_all(".ptabs")
            .unwrap_or_default();
        strips
            .into_iter()
            .map(|strip| {
                let doc = self.doc.inner();
                let children = doc.get_node(strip).map(|n| n.children.clone()).unwrap_or_default();
                drop(doc);
                children
                    .into_iter()
                    .map(|c| self.text_of(c).trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect()
            })
            .collect()
    }

    /// 自前で描く widget が今いくつ生きているか。
    fn drawing_panels(&self) -> usize {
        self.doc.inner().custom_widget_node_ids().len()
    }

    /// 選択子に当たる要素の class 属性。
    fn classes(&mut self, selector: &str) -> Vec<String> {
        self.doc
            .inner()
            .query_selector_all(selector)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|n| {
                self.doc.inner().get_node(n).and_then(|node| {
                    node.attrs().map(|attrs| {
                        attrs
                            .iter()
                            .find(|a| a.name.local.as_ref() == "class")
                            .map(|a| a.value.clone())
                            .unwrap_or_default()
                    })
                })
            })
            .collect()
    }

    fn size_of_nth(&mut self, selector: &str, nth: usize) -> (f32, f32) {
        let inner = self.doc.inner();
        let nodes = inner.query_selector_all(selector).unwrap_or_default();
        let node = *nodes
            .get(nth)
            .unwrap_or_else(|| panic!("`{selector}` の {nth} 番が居ない"));
        let layout = inner.get_node(node).expect("node").final_layout();
        (layout.size.width, layout.size.height)
    }

    fn center_of(&mut self, selector: &str, nth: usize) -> (f32, f32) {
        let nodes = self
            .doc
            .inner()
            .query_selector_all(selector)
            .unwrap_or_default();
        let node = *nodes
            .get(nth)
            .unwrap_or_else(|| panic!("`{selector}` の {nth} 番が居ない(居るのは {})", nodes.len()));
        let doc = self.doc.inner();
        let node = doc.get_node(node).expect("node");
        let layout = node.final_layout();
        let pos = node.absolute_position(0.0, 0.0);
        (
            pos.x as f32 + layout.size.width / 2.0,
            pos.y as f32 + layout.size.height / 2.0,
        )
    }

    fn click(&mut self, x: f32, y: f32) {
        self.press(x, y);
        self.release(x, y);
        self.settle();
    }

    /// 掴んで運んで離す。タブを別の置き場へ移す操作。
    fn drag(&mut self, from: (f32, f32), to: (f32, f32)) {
        self.press(from.0, from.1);
        self.motion(to.0, to.1);
        self.release(to.0, to.1);
        self.settle();
    }

    fn settle(&mut self) {
        for _ in 0..4 {
            self.doc.poll(None);
            self.doc.inner_mut().resolve(0.0);
        }
    }

    /// 打鍵を1つ流す。
    fn key(&mut self, key: keyboard_types::Key, mods: keyboard_types::Modifiers) {
        use blitz_traits::events::{BlitzKeyEvent, KeyState};
        let event = BlitzKeyEvent {
            key,
            code: keyboard_types::Code::Unidentified,
            modifiers: mods,
            location: keyboard_types::Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Pressed,
            text: None,
        };
        // 本番と同じ規則で焦点を当ててから配る(`host.rs` と同じ1つの関数)。
        crate::ui::host::aim_keystrokes(&mut self.doc);
        self.doc.handle_ui_event(UiEvent::KeyDown(event));
        self.settle();
    }

    fn press(&mut self, x: f32, y: f32) {
        let moved = self.pointer_raw(x, y, MouseEventButtons::None);
        self.doc.handle_ui_event(UiEvent::PointerMove(moved));
        let event = self.pointer_raw(x, y, MouseEventButtons::Primary);
        self.doc.handle_ui_event(UiEvent::PointerDown(event));
    }

    fn motion(&mut self, x: f32, y: f32) {
        let event = self.pointer_raw(x, y, MouseEventButtons::Primary);
        self.doc.handle_ui_event(UiEvent::PointerMove(event));
    }

    fn release(&mut self, x: f32, y: f32) {
        let event = self.pointer_raw(x, y, MouseEventButtons::None);
        self.doc.handle_ui_event(UiEvent::PointerUp(event));
    }

    fn pointer_raw(&self, x: f32, y: f32, buttons: MouseEventButtons) -> BlitzPointerEvent {
        let event = |buttons| BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button: MouseEventButton::Main,
            buttons,
            mods: Default::default(),
            details: Default::default(),
            element: Default::default(),
            active_pointers: Arc::default(),
        };
        event(buttons)
    }
}

fn collect_text(doc: &blitz_dom::BaseDocument, node: blitz_dom::NodeId, out: &mut String) {
    let Some(node) = doc.get_node(node) else { return };
    if let Some(text) = node.text_data() {
        out.push_str(&text.content);
    }
    for child in &node.children {
        collect_text(doc, *child, out);
    }
}
