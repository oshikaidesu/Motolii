
//! 窓を開けずに GUI を動かす。blitz の document をそのまま headless で建て、
//! 座標でクリックする(当たり判定もレイアウトも本番と同じ経路)。

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
        vdom.insert_any_root_context(Box::new(Session::new(doc, duration_sec)));
        vdom.insert_any_root_context(Box::new(std::sync::Arc::new(ui)));
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

#[test]
fn the_dock_opens_with_every_panel_reachable_by_a_tab() {
    let mut gui = Gui::open();
    let tabs = gui.texts(".ptab");
    for expected in ["Media", "Effects", "Create", "Colors", "Stage", "Inspector", "Utility", "Timeline"] {
        assert!(
            tabs.iter().any(|t| t == expected),
            "{expected} のタブが出ていない(出ているのは {tabs:?})"
        );
    }
}

#[test]
fn clicking_a_tab_swaps_the_body_without_losing_the_other_tabs() {
    let mut gui = Gui::open();
    let before = gui.texts(".ptab");
    let utility = before.iter().position(|t| t == "Utility").expect("Utility タブ");

    let (x, y) = gui.center_of(".ptab", utility);
    gui.click(x, y);

    let after = gui.texts(".ptab");
    assert_eq!(before, after, "タブを押しただけでタブの並びが変わった");
    let body = gui.texts("#inspector");
    assert!(
        body.iter().any(|t| t.contains("ANCHOR")),
        "Utility を押しても中身が入れ替わっていない: {body:?}"
    );
}

#[test]
fn every_panel_can_be_shown_without_breaking_the_next_render() {
    let mut gui = Gui::open();
    for i in 0..gui.texts(".ptab").len() {
        let (x, y) = gui.center_of(".ptab", i);
        gui.click(x, y);
    }
    assert_eq!(
        gui.texts(".ptab").len(),
        8,
        "順に押していったらタブが減った"
    );
}

#[test]
fn switching_back_and_forth_between_two_tabs_survives() {
    let mut gui = Gui::open();
    let tabs = gui.texts(".ptab");
    let inspector = tabs.iter().position(|t| t == "Inspector").expect("Inspector タブ");
    let utility = tabs.iter().position(|t| t == "Utility").expect("Utility タブ");

    for _ in 0..3 {
        let (x, y) = gui.center_of(".ptab", utility);
        gui.click(x, y);
        let (x, y) = gui.center_of(".ptab", inspector);
        gui.click(x, y);
    }

    assert!(
        gui.texts("#inspector").iter().any(|t| t.contains("TRANSFORM")),
        "往復したら Inspector が壊れた"
    );
}

#[test]
fn a_tab_dragged_into_another_zone_moves_there() {
    let mut gui = Gui::open();
    let tabs = gui.texts(".ptab");
    let colors = tabs.iter().position(|t| t == "Colors").expect("Colors タブ");
    let timeline = tabs.iter().position(|t| t == "Timeline").expect("Timeline タブ");

    let from = gui.center_of(".ptab", colors);
    let to = gui.center_of(".ptab", timeline);
    gui.drag(from, to);

    let strips = gui.zone_tabs();
    let bottom = strips.last().expect("下の置き場");
    assert!(
        bottom.contains(&"Colors".to_string()),
        "Colors が下の置き場へ移っていない: {strips:?}"
    );
}

/// 窓で落ちた順番そのもの: Utility を出す(Inspector の hook が消える) → Create に
/// 切り替える → Colors を下の置き場へ落とす。
#[test]
fn moving_a_panel_after_hiding_another_ones_body_survives() {
    let mut gui = Gui::open();
    let tabs = gui.texts(".ptab");
    let at = |name: &str| tabs.iter().position(|t| t == name).unwrap_or_else(|| panic!("{name} タブ"));

    let p = gui.center_of(".ptab", at("Utility"));
    gui.click(p.0, p.1);
    let p = gui.center_of(".ptab", at("Create"));
    gui.click(p.0, p.1);

    let from = gui.center_of(".ptab", at("Colors"));
    let to = gui.center_of(".ptab", at("Timeline"));
    gui.drag(from, to);

    let strips = gui.zone_tabs();
    assert!(
        strips.last().expect("下の置き場").contains(&"Colors".to_string()),
        "Colors が下の置き場へ移っていない: {strips:?}"
    );
}

/// 盤面(Stage・Timeline)は自前で描く widget を持つ。置き場を移すと要素が
/// 作り直されるので、移した先で widget が付いていないと絵が止まる。
#[test]
fn a_panel_that_draws_itself_still_draws_after_being_moved() {
    let mut gui = Gui::open();
    let before = gui.drawing_panels();
    assert_eq!(before, 2, "Stage と Timeline が自前で描いているはず: {before}");

    let tabs = gui.texts(".ptab");
    let at = |name: &str| tabs.iter().position(|t| t == name).unwrap_or_else(|| panic!("{name} タブ"));
    // 左へ移す。タイムラインが選ばれたままの置き場へ入れると、選ばれなかった方は
    // そもそも描かれないので、数が減るのが正しくなってしまう。
    let from = gui.center_of(".ptab", at("Stage"));
    let to = gui.center_of(".ptab", at("Media"));
    gui.drag(from, to);

    let strips = gui.zone_tabs();
    assert!(
        strips[0].contains(&"Stage".to_string()),
        "そもそも移っていない: {strips:?}"
    );
    assert_eq!(
        gui.drawing_panels(),
        2,
        "移した先で widget が付いていない(CustomWidgetAttr は一度しか中身を渡せない): {strips:?}"
    );
}
