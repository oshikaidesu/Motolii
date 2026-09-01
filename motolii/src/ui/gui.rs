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
    h: blitz_test_harness::Harness<DioxusDocument>,
    /// 窓と同じ状態への取っ手。**触った結果を数で見る**ために持つ。
    session: Session,
}

impl Gui {
    fn open() -> Self {
        let Loaded { doc, ui, duration_sec } = load_fixture();
        let session = Session::new(doc, duration_sec, ui);
        let mut vdom = VirtualDom::new(app);
        vdom.insert_any_root_context(Box::new(session.clone()));
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
        Self { h: blitz_test_harness::Harness::wrap(doc), session }
    }

    fn text_of(&self, node: blitz_dom::NodeId) -> String {
        let mut out = String::new();
        collect_text(&self.h.base(), node, &mut out);
        out
    }

    /// 選択子に当たる全部の見出し。並びは DOM の順。
    fn texts(&mut self, selector: &str) -> Vec<String> {
        self.h
            .base()
            .query_selector_all(selector)
            .unwrap_or_default()
            .into_iter()
            .map(|n| self.text_of(n).trim().to_string())
            .collect()
    }

    /// 置き場ごとのタブの見出し。並びは DOM の順(左・中・右・下)。
    fn zone_tabs(&mut self) -> Vec<Vec<String>> {
        let strips = self
            .h
            .base()
            .query_selector_all(".ptabs")
            .unwrap_or_default();
        strips
            .into_iter()
            .map(|strip| {
                let doc = self.h.base();
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
        self.h.base().custom_widget_node_ids().len()
    }

    /// 選択子に当たる要素の class 属性。
    fn classes(&mut self, selector: &str) -> Vec<String> {
        self.h
            .base()
            .query_selector_all(selector)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|n| {
                self.h.base().get_node(n).and_then(|node| {
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
        let inner = self.h.base();
        let nodes = inner.query_selector_all(selector).unwrap_or_default();
        let node = *nodes
            .get(nth)
            .unwrap_or_else(|| panic!("`{selector}` の {nth} 番が居ない"));
        let layout = inner.get_node(node).expect("node").final_layout();
        (layout.size.width, layout.size.height)
    }

    fn center_of(&mut self, selector: &str, nth: usize) -> (f32, f32) {
        let nodes = self
            .h
            .base()
            .query_selector_all(selector)
            .unwrap_or_default();
        let node = *nodes
            .get(nth)
            .unwrap_or_else(|| panic!("`{selector}` の {nth} 番が居ない(居るのは {})", nodes.len()));
        let doc = self.h.base();
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
            self.h.pump();
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
        
        self.h.dispatch(UiEvent::KeyDown(event));
        self.settle();
    }

    fn press(&mut self, x: f32, y: f32) {
        let moved = self.pointer_raw(x, y, MouseEventButtons::None);
        self.h.dispatch(UiEvent::PointerMove(moved));
        let event = self.pointer_raw(x, y, MouseEventButtons::Primary);
        self.h.dispatch(UiEvent::PointerDown(event));
    }

    fn motion(&mut self, x: f32, y: f32) {
        let event = self.pointer_raw(x, y, MouseEventButtons::Primary);
        self.h.dispatch(UiEvent::PointerMove(event));
    }

    fn release(&mut self, x: f32, y: f32) {
        let event = self.pointer_raw(x, y, MouseEventButtons::None);
        self.h.dispatch(UiEvent::PointerUp(event));
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
/// 利用者の道 —— 名前をダブルクリックして、打って、Enter。
///
/// この道の試験が1本も無かったので、**打鍵のたび焦点を根へ引き戻していて
/// 欄に1字も入らない**状態が長く残った。窓に「double-click to type」と
/// 書いてあるのに打てない、という嘘まで出ていた。
#[test]
fn a_name_can_be_typed_after_double_clicking_it() {
    let mut gui = Gui::open();
    // 0番はカメラの行(層ではない)。最初の層は1番。
    let (x, y) = gui.center_of(".lsurface", 1);
    let before = gui.texts(".lsurface");

    gui.click(x, y);
    gui.click(x, y);
    gui.settle();
    for ch in ["Z", "Z"] {
        gui.key(
            keyboard_types::Key::Character(ch.into()),
            keyboard_types::Modifiers::empty(),
        );
    }
    gui.key(keyboard_types::Key::Enter, keyboard_types::Modifiers::empty());
    gui.settle();

    let after = gui.texts(".lsurface");
    assert_ne!(after, before, "打った文字が層の名前に入っていない: {after:?}");
    assert!(
        after.iter().any(|n| n.contains("ZZ")),
        "打った文字が層の名前に入っていない: {after:?}"
    );
}
// ---- 生成した嵐 ------------------------------------------------------------

/// 窓へ送れる一手。**利用者は決められた順番では触らない**ので、順番も座標も
/// 種類も生成させる。
#[derive(Debug, Clone)]
enum Poke {
    Press(f32, f32),
    Motion(f32, f32),
    Release(f32, f32),
    Key(usize),
}

/// 素の打鍵で意味を持つ物を混ぜる。押されて困る物ほど入れる価値がある。
const STORM_CHARS: &[&str] = &[" ", "u", "[", "]", "d", "*"];

fn storm_key(i: usize) -> keyboard_types::Key {
    match i.checked_sub(STORM_CHARS.len()) {
        None => keyboard_types::Key::Character(STORM_CHARS[i].to_owned()),
        Some(0) => keyboard_types::Key::Escape,
        Some(1) => keyboard_types::Key::Delete,
        Some(2) => keyboard_types::Key::ArrowLeft,
        _ => keyboard_types::Key::ArrowRight,
    }
}

const STORM_KEY_COUNT: usize = STORM_CHARS.len() + 4;

fn poke() -> impl proptest::strategy::Strategy<Value = Poke> {
    use proptest::prelude::*;
    // 窓の外も混ぜる。掴んだまま外へ出るのは実際に起きる。
    let x = -200.0f32..(W as f32 + 200.0);
    let y = -200.0f32..(H as f32 + 200.0);
    prop_oneof![
        (x.clone(), y.clone()).prop_map(|(x, y)| Poke::Press(x, y)),
        (x.clone(), y.clone()).prop_map(|(x, y)| Poke::Motion(x, y)),
        (x, y).prop_map(|(x, y)| Poke::Release(x, y)),
        (0..STORM_KEY_COUNT).prop_map(Poke::Key),
    ]
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig {
        cases: 24,
        max_shrink_iters: 64,
        ..proptest::prelude::ProptestConfig::default()
    })]

    /// どんな順番で何を叩かれても、窓は操作を受け付け続ける。
    ///
    /// 落ちない事だけでは弱い —— 掴みが外れず固まる、置き場が空になる、
    /// タブが効かなくなる、が実際に起きる壊れ方。**最後に普通の一手が
    /// 通るか**まで見る。
    #[test]
    fn the_window_still_takes_orders_after_any_storm(storm in proptest::collection::vec(poke(), 1..40)) {
        let mut gui = Gui::open();
        for p in &storm {
            match *p {
                Poke::Press(x, y) => gui.press(x, y),
                Poke::Motion(x, y) => gui.motion(x, y),
                Poke::Release(x, y) => gui.release(x, y),
                Poke::Key(i) => gui.key(storm_key(i), keyboard_types::Modifiers::empty()),
            }
        }
        // 指を上げて、掴みを解く。ここから先は普通の窓でなければならない。
        gui.release(10.0, 10.0);
        gui.key(keyboard_types::Key::Escape, keyboard_types::Modifiers::empty());
        gui.settle();

        proptest::prop_assert!(gui.drawing_panels() > 0, "描く panel が居なくなった: {storm:?}");

        // タブは窓の外へ引くと別窓へ出る(仕様)。全部は出られない。
        let tabs = gui.texts(".ptab");
        proptest::prop_assert!(!tabs.is_empty(), "置き場からタブが全部消えた: {storm:?}");

        // 素の一押しは**タブの並びを変えない**。変わるなら掴みが残っている。
        let (x, y) = gui.center_of(".ptab", 0);
        gui.click(x, y);
        gui.settle();
        proptest::prop_assert_eq!(
            gui.texts(".ptab"),
            tabs,
            "嵐のあと、タブを押しただけで並びが変わった(掴みが残っている): {:?}",
            storm
        );
        proptest::prop_assert!(
            gui.drawing_panels() > 0,
            "嵐のあとの一押しで描く panel が居なくなった: {storm:?}"
        );
    }
}


/// ホイールの向き。**今日はこれを実機で目視するしかなかった** —— 自作の器具に
/// ホイールが無かったため。上流のハーネスには `wheel_at` が在る。
///
/// 上へ回すと近づく(枠が広がる)。地図でも紙でも絵でもこの向き。
fn gui_zoom(gui: &Gui) -> f32 {
    gui.session.view_camera.lock().unwrap().zoom
}

#[test]
fn rolling_the_wheel_up_moves_the_view_closer() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of("#stage", 0);

    let before = gui_zoom(&gui);
    gui.h.wheel_at(x, y, 0.0, 40.0);
    gui.settle();
    let after = gui_zoom(&gui);

    assert!(
        after > before,
        "上へ回したのに遠ざかった: {before} → {after}"
    );
}

/// 掴んだまま窓を出ても、タブは消えない。
///
/// 前は `onmouseleave` が**離す前に**引きちぎって別窓にしていた。掴みは
/// 離した時だけ効く —— 同じ手つきで判定の瞬間が違うのが不統一の芯だった。
#[test]
fn dragging_a_tab_out_of_the_window_does_not_take_it_away() {
    let mut gui = Gui::open();
    let before = gui.texts(".ptab");
    let (x, y) = gui.center_of(".ptab", 0);

    gui.press(x, y);
    gui.motion(x, y + 40.0);
    gui.motion(-80.0, -80.0);
    gui.release(-80.0, -80.0);
    gui.settle();

    assert_eq!(gui.texts(".ptab"), before, "窓を出ただけでタブが消えた");
}

/// 掴んでいる間は、掴んでいると分かる。
#[test]
fn a_held_tab_looks_held() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of(".ptab", 0);

    gui.press(x, y);
    gui.motion(x + 30.0, y + 30.0);
    gui.settle();

    assert!(
        gui.classes(".ptab").iter().any(|c| c.contains("held")),
        "掴んでいるのに見た目が変わらない: {:?}",
        gui.classes(".ptab")
    );
}
