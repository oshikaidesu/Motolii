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
use dioxus_native::prelude::VirtualDom;
use dioxus_native::DioxusDocument;

use crate::ui::app::app;
use crate::ui::fixture::{load_fixture, Loaded};
use crate::ui::session::Session;

const W: u32 = 1600;
const H: u32 = 1000;

struct Gui {
    h: blitz_test_harness::Harness<DioxusDocument>,
    /// 窓と同じ状態への取っ手。**触った結果を数で見る**ために持つ。
    session: Session,
    host: crate::ui::host::Host,
}

impl Gui {
    fn open() -> Self {
        Self::open_at(W, H)
    }

    fn open_at(width: u32, height: u32) -> Self {
        let Loaded {
            doc,
            ui,
            duration_sec,
        } = load_fixture();
        let session = Session::new(doc, duration_sec, ui);
        let host = crate::ui::host::Host::for_tests();
        let mut vdom = VirtualDom::new(app);
        vdom.insert_any_root_context(Box::new(session.clone()));
        vdom.insert_any_root_context(Box::new(host.clone()));
        let mut doc = DioxusDocument::new(
            vdom,
            DocumentConfig {
                viewport: Some(Viewport::new(width, height, 1.0, ColorScheme::Dark)),
                ..Default::default()
            },
        );
        doc.initial_build();
        doc.inner_mut().resolve(0.0);
        Self {
            h: blitz_test_harness::Harness::wrap(doc),
            session,
            host,
        }
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

    fn count(&self, selector: &str) -> usize {
        self.h
            .base()
            .query_selector_all(selector)
            .unwrap_or_default()
            .len()
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
                let children = doc
                    .get_node(strip)
                    .map(|n| n.children.clone())
                    .unwrap_or_default();
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

    fn opacity_of_nth(&self, selector: &str, nth: usize) -> f32 {
        let doc = self.h.base();
        let nodes = doc.query_selector_all(selector).unwrap_or_default();
        let node = *nodes
            .get(nth)
            .unwrap_or_else(|| panic!("`{selector}` の {nth} 番が居ない"));
        let opacity = doc
            .get_node(node)
            .and_then(|node| node.primary_styles())
            .expect("computed style")
            .clone_opacity();
        opacity
    }

    fn tick(&mut self, seconds: f64) {
        self.h.tick(seconds);
    }

    fn tab_strip_overflow(&self) -> Vec<(usize, f32, f32)> {
        let doc = self.h.base();
        let strips = doc.query_selector_all(".ptabs").unwrap_or_default();
        let mut out = Vec::new();
        for (index, strip) in strips.into_iter().enumerate() {
            let Some(strip_node) = doc.get_node(strip) else {
                continue;
            };
            let strip_pos = strip_node.absolute_position(0.0, 0.0);
            let strip_right = strip_pos.x as f32 + strip_node.final_layout().size.width;
            let tab_right = strip_node
                .children
                .iter()
                .filter_map(|child| doc.get_node(*child))
                .filter(|child| {
                    child.attrs().is_some_and(|attrs| {
                        attrs.iter().any(|attr| {
                            attr.name.local.as_ref() == "class"
                                && attr.value.split_whitespace().any(|class| class == "ptab")
                        })
                    })
                })
                .map(|tab| {
                    let pos = tab.absolute_position(0.0, 0.0);
                    pos.x as f32 + tab.final_layout().size.width
                })
                .fold(strip_pos.x as f32, f32::max);
            let scrollable = strip_node
                .primary_styles()
                .is_some_and(|styles| styles.clone_overflow_x().is_scrollable());
            if tab_right > strip_right + 0.5 && !scrollable {
                out.push((index, tab_right, strip_right));
            }
        }
        out
    }

    fn center_of(&mut self, selector: &str, nth: usize) -> (f32, f32) {
        let nodes = self
            .h
            .base()
            .query_selector_all(selector)
            .unwrap_or_default();
        let node = *nodes.get(nth).unwrap_or_else(|| {
            panic!("`{selector}` の {nth} 番が居ない(居るのは {})", nodes.len())
        });
        let doc = self.h.base();
        let node = doc.get_node(node).expect("node");
        let layout = node.final_layout();
        let pos = node.absolute_position(0.0, 0.0);
        (
            pos.x as f32 + layout.size.width / 2.0,
            pos.y as f32 + layout.size.height / 2.0,
        )
    }

    fn center_of_text(&mut self, selector: &str, text: &str) -> (f32, f32) {
        let nodes = self
            .h
            .base()
            .query_selector_all(selector)
            .unwrap_or_default();
        let node = nodes
            .into_iter()
            .find(|node| self.text_of(*node).trim() == text)
            .unwrap_or_else(|| panic!("`{selector}` に `{text}` が居ない"));
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
        self.h.move_mouse_to(x, y);
        crate::ui::host::commit_field_outside(&mut self.h.doc, x, y);
        self.h.click_at(x, y);
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

    /// 打鍵を1つ流す。窓の shell と同じ順: 先に打鍵の当て先を決める(`host::aim_keystrokes`)。
    fn key(&mut self, key: keyboard_types::Key, mods: keyboard_types::Modifiers) {
        crate::ui::host::aim_keystrokes(&mut self.h.doc);
        self.h.press_with(key, mods);
        self.settle();
    }

    /// 窓の shell と同じ順: 欄の外を押したら先に欄を確定させる(`host::Windows::window_event`)。
    fn press(&mut self, x: f32, y: f32) {
        self.h.move_mouse_to(x, y);
        crate::ui::host::commit_field_outside(&mut self.h.doc, x, y);
        self.h.mouse_down_at(x, y);
    }

    fn motion(&mut self, x: f32, y: f32) {
        let event = self.pointer_raw(x, y, MouseEventButtons::Primary);
        self.h.dispatch(UiEvent::PointerMove(event));
    }

    /// 窓の shell と同じ順: 持ち主へ配ってから blitz へ流す(`host::Windows::window_event`)。
    fn release(&mut self, x: f32, y: f32) {
        self.host.primary_pointer_released(
            crate::ui::host::Host::HEADLESS,
            f64::from(x),
            f64::from(y),
        );
        self.h.mouse_up_at(x, y);
    }

    fn lose_focus(&mut self) {
        self.host.focus_lost();
        self.settle();
    }

    fn enter_files(&mut self, paths: &[std::path::PathBuf]) {
        self.host.focus_lost();
        self.session.file_drop.enter(paths);
        self.host.wake_all();
        self.settle();
    }

    fn leave_files(&mut self) {
        self.session.file_drop.leave();
        self.host.wake_all();
        self.settle();
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

    fn chord_motion(&mut self, x: f32, y: f32) {
        let buttons = MouseEventButtons::Primary | MouseEventButtons::Secondary;
        self.h
            .dispatch(UiEvent::PointerMove(self.pointer_raw(x, y, buttons)));
    }
}

fn collect_text(doc: &blitz_dom::BaseDocument, node: blitz_dom::NodeId, out: &mut String) {
    let Some(node) = doc.get_node(node) else {
        return;
    };
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
    gui.key(
        keyboard_types::Key::Enter,
        keyboard_types::Modifiers::empty(),
    );
    gui.settle();

    let after = gui.texts(".lsurface");
    assert_ne!(
        after, before,
        "打った文字が層の名前に入っていない: {after:?}"
    );
    assert!(
        after.iter().any(|n| n.contains("ZZ")),
        "打った文字が層の名前に入っていない: {after:?}"
    );
}

/// 書き置きは複数行。Enter は改行で欄は残り、外を押すと確定する。
#[test]
fn a_note_is_typed_in_a_textarea_and_committed_by_clicking_outside() {
    let mut gui = Gui::open();
    {
        use crate::doc::store::{Intent, Marker, RationalTime};
        let marker = Marker {
            name: "intro".into(),
            time: RationalTime::ZERO,
            duration: RationalTime::ZERO,
            body: String::new(),
        };
        gui.session.doc.lock().unwrap().apply(Intent::SetMarkers { markers: vec![marker] }).unwrap();
    }
    let text = gui.center_of_text(".desk-foot .chip", "Text");
    gui.click(text.0, text.1);
    let note = gui.center_of(".desk-note .mbody", 0);
    gui.click(note.0, note.1);
    assert_eq!(gui.count("textarea.mbody"), 1, "the note did not open as a textarea");

    for ch in ["L", "a"] {
        gui.key(
            keyboard_types::Key::Character(ch.into()),
            keyboard_types::Modifiers::empty(),
        );
    }
    gui.key(keyboard_types::Key::Enter, keyboard_types::Modifiers::empty());
    assert_eq!(gui.count("textarea.mbody"), 1, "Enter closed a multi-line note");
    assert!(gui.session.field().is_some());

    let stage = gui.center_of("#stage", 0);
    gui.click(stage.0, stage.1);
    assert_eq!(gui.count("textarea"), 0, "clicking outside left the note open");
    assert!(gui.session.field().is_none(), "the field owner still holds a closed field");
    let markers = gui.session.doc.lock().unwrap().view().markers().unwrap();
    assert!(
        markers.iter().any(|m| m.body.starts_with("La")),
        "typed note did not reach the marker: {:?}",
        markers.iter().map(|m| m.body.clone()).collect::<Vec<_>>()
    );
}

/// 色は Inspector の行を押すと焦点になり、机の輪と面で変わって data へ戻る。
#[test]
fn picking_on_the_desk_wheel_writes_the_focused_color_back() {
    let mut gui = Gui::open();
    let rows = gui.count(".lsurface");
    let mut found = false;
    for i in 1..rows {
        let (x, y) = gui.center_of(".lsurface", i);
        gui.click(x, y);
        if gui.count(".prow.color") > 0 {
            found = true;
            break;
        }
    }
    assert!(found, "no layer in the fixture shows a COLOR row");
    let (x, y) = gui.center_of(".prow.color", 0);
    gui.click(x, y);
    assert_eq!(gui.count(".color-drawer"), 1, "the color focus did not open the desk drawer");
    let before = gui.session.field();
    assert!(before.is_none());

    // 面の右上 = 彩度 1・明度 1 の純色。輪の色相はそのまま。
    let (sx, sy) = gui.size_of_nth(".sv-square", 0);
    let (cx, cy) = gui.center_of(".sv-square", 0);
    let (px, py) = (cx + sx / 2.0 - 4.0, cy - sy / 2.0 + 4.0);
    gui.press(px, py);
    gui.motion(px, py);
    gui.release(px, py);
    gui.settle();

    let slot = match gui.session.live_focus() {
        Some(crate::ui::session::Focus::Color(slot)) => slot,
        other => panic!("focus drifted: {other:?}"),
    };
    let after = crate::ui::desk::read_color(&gui.session.doc, &slot).unwrap();
    let (_, s, v) = crate::ui::desk::rgb_to_hsv([after[0], after[1], after[2]]);
    assert!(s > 0.9 && v > 0.9, "the pick did not reach the document: {after:?}");
}

/// 参考画像は素材と同じ口で入るが、役目は落とした場所で決まる。机なら参考、他は素材。
#[test]
fn a_file_dropped_on_the_desk_is_a_reference_and_stays_off_the_browser() {
    let mut gui = Gui::open();
    let desk = gui.center_of("#desk", 0);
    let stage = gui.center_of("#stage", 0);
    assert_eq!(
        crate::ui::host::drop_role_at(&gui.h.doc, desk.0, desk.1),
        crate::doc::store::AssetRole::Reference
    );
    assert_eq!(
        crate::ui::host::drop_role_at(&gui.h.doc, stage.0, stage.1),
        crate::doc::store::AssetRole::Material
    );

    let dir = tempfile::tempdir().unwrap();
    let png = dir.path().join("ref.png");
    image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 40, 40, 255])).save(&png).unwrap();
    let before = gui.count(".tcard");
    let summary = crate::ui::fixture::admit_paths(
        &mut gui.session.doc.lock().unwrap(),
        &[png],
        crate::doc::store::AssetRole::Reference,
    );
    assert_eq!(summary.admitted, 1, "{}", summary.notice());
    gui.session.desk.lock().unwrap().clone_from(&crate::ui::session::DeskState::Follow);
    let text = gui.center_of_text(".desk-foot .chip", "Text");
    gui.click(text.0, text.1);
    gui.click(text.0, text.1);
    assert_eq!(gui.count(".desk-refs .ref"), 1, "the reference image is not on the desk");
    assert_eq!(gui.count(".tcard"), before, "a reference image leaked into the browser");
}

#[test]
fn menus_are_one_semantic_family() {
    let mut gui = Gui::open();
    let (file_x, file_y) = gui.center_of("#menu-file", 0);
    gui.click(file_x, file_y);
    assert_eq!(gui.count("#menu-file-list"), 1, "File menu did not open");

    let (view_x, view_y) = gui.center_of("#menu-view", 0);
    gui.click(view_x, view_y);
    assert_eq!(
        gui.count("#menu-file-list"),
        0,
        "opening View left File open"
    );
    assert_eq!(gui.count("#menu-view-list"), 1, "View menu did not open");

    gui.key(
        keyboard_types::Key::Escape,
        keyboard_types::Modifiers::empty(),
    );
    gui.settle();
    assert_eq!(gui.count(".vmenu"), 0, "Escape did not close the open menu");
}

#[test]
fn losing_window_focus_dismisses_an_open_menu() {
    let mut gui = Gui::open();
    let file = gui.center_of("#menu-file", 0);
    gui.click(file.0, file.1);
    assert_eq!(gui.count("#menu-file-list"), 1);

    gui.lose_focus();

    assert_eq!(gui.count(".vmenu"), 0, "focus loss left a menu visible");
}

#[test]
fn menu_motion_has_a_real_intermediate_frame_without_delaying_the_command() {
    let mut gui = Gui::open();
    let (file_x, file_y) = gui.center_of("#menu-file", 0);
    gui.click(file_x, file_y);

    assert_eq!(gui.count("#menu-file-list"), 1, "menu command was delayed");
    let mut samples = vec![gui.opacity_of_nth(".vmenu", 0)];
    for _ in 0..12 {
        gui.tick(1.0 / 60.0);
        samples.push(gui.opacity_of_nth(".vmenu", 0));
    }
    assert!(
        samples.windows(2).all(|pair| pair[0] <= pair[1]),
        "menu opacity went backwards: {samples:?}"
    );
    let moving_frames = samples
        .windows(2)
        .filter(|pair| (pair[1] - pair[0]).abs() > 0.0001)
        .count();
    assert!(
        moving_frames >= 6,
        "menu jumped instead of producing smooth frames: {samples:?}"
    );
    let end = *samples.last().unwrap();
    assert!(
        (end - 1.0).abs() < 0.001,
        "menu did not finish opaque: {end}"
    );
}

#[test]
fn outside_menu_click_is_consumed_before_the_stage() {
    let mut gui = Gui::open();
    let (file_x, file_y) = gui.center_of("#menu-file", 0);
    gui.click(file_x, file_y);
    assert_eq!(
        gui.count(".menu-dismiss"),
        1,
        "outside-click control is missing"
    );
    let dismiss_size = gui.size_of_nth(".menu-dismiss", 0);
    assert!(
        dismiss_size.0 >= W as f32 && dismiss_size.1 >= H as f32,
        "outside-click control does not cover the window: {dismiss_size:?}"
    );

    let (stage_x, stage_y) = gui.center_of("#stage", 0);
    gui.press(stage_x, stage_y);
    assert!(
        !gui.session.gesture.is_active(),
        "the click passed through the menu dismissal surface into Stage"
    );
    gui.release(stage_x, stage_y);
    gui.settle();
    assert_eq!(
        gui.count(".vmenu"),
        0,
        "outside click did not close the menu"
    );
}

#[test]
fn product_chrome_does_not_advertise_unimplemented_controls() {
    let mut gui = Gui::open();
    let menubar = gui.texts("#menubar").join(" ");
    for dead in ["Edit", "Layer", "Effect", "Help"] {
        assert!(
            !menubar.contains(dead),
            "dead menu is still visible: {dead}"
        );
    }
    assert_eq!(
        gui.count(".btoolbar"),
        0,
        "dead Browser toolbar is still visible"
    );
    let browser = gui.texts("#browser").join(" ");
    for dead in [
        "Search files and tags",
        "Filters",
        "Tags",
        "Comp 1",
        "Edit tags",
    ] {
        assert!(
            !browser.contains(dead),
            "dead Browser control is still visible: {dead}"
        );
    }
}

#[test]
fn click_only_chrome_uses_real_buttons_and_real_disabled_state() {
    let mut gui = Gui::open();

    let create = gui.center_of("#dock-tab-Create", 0);
    gui.click(create.0, create.1);
    assert_eq!(gui.count(".tcard"), 4);
    assert_eq!(gui.count("button.semantic-button.tcard"), 4);

    let effects = gui.center_of("#dock-tab-Effects", 0);
    gui.click(effects.0, effects.1);
    let disabled = gui.count("button.semantic-button.tcard.disabled");
    assert!(
        disabled > 0,
        "effect cards did not expose disabled button semantics"
    );
    assert_eq!(
        gui.count("button.semantic-button.tcard.disabled[disabled]"),
        disabled
    );

    let settings = gui.center_of("#menu-settings", 0);
    gui.click(settings.0, settings.1);
    assert_eq!(gui.count(".zbtn"), 4);
    assert_eq!(gui.count("button.semantic-button.zbtn"), 4);
}

#[test]
fn the_mask_card_adds_one_mask_to_the_selected_layer() {
    let mut gui = Gui::open();
    let layer_row = gui.center_of(".lsurface", 1);
    gui.click(layer_row.0, layer_row.1);
    gui.settle();
    let selected = gui
        .session
        .selection
        .get()
        .expect("Timeline layer selection");
    assert!(gui
        .session
        .doc
        .lock()
        .unwrap()
        .view()
        .masks(selected)
        .unwrap()
        .is_empty());

    let create = gui.center_of("#dock-tab-Create", 0);
    gui.click(create.0, create.1);
    gui.settle();
    let mask = gui.center_of(".tcard", 3);
    gui.click(mask.0, mask.1);
    gui.settle();

    assert_eq!(
        gui.session
            .doc
            .lock()
            .unwrap()
            .view()
            .masks(selected)
            .unwrap()
            .len(),
        1
    );
    assert!(gui.texts(".tcard")[3].contains("1 attached"));
}

#[test]
fn edited_status_tracks_the_saved_document_revision() {
    let mut gui = Gui::open();
    assert!(!gui.texts("#status").join(" ").contains("Edited"));

    let create = gui.center_of("#dock-tab-Create", 0);
    gui.click(create.0, create.1);
    let text = gui.center_of(".tcard", 0);
    gui.click(text.0, text.1);
    gui.settle();
    assert!(gui.texts("#status").join(" ").contains("Edited"));

    gui.session.mark_saved(std::path::PathBuf::from("song.rrd"));
    gui.host.wake_all();
    gui.settle();
    assert!(!gui.texts("#status").join(" ").contains("Edited"));
}

#[test]
fn file_drop_hover_and_cancel_are_visible_in_the_product_tree() {
    let mut gui = Gui::open();
    gui.enter_files(&["clip.mov".into(), "sound.wav".into()]);
    assert_eq!(gui.count(".file-drop-overlay"), 1);
    assert_eq!(gui.texts(".file-drop-card"), vec!["Drop 2 files to import"]);

    gui.leave_files();
    assert_eq!(gui.count(".file-drop-overlay"), 0);
}

#[test]
fn timeline_uses_the_shared_focus_loss_cancellation() {
    let mut gui = Gui::open();
    let timeline = gui.center_of("#timeline", 0);
    gui.press(timeline.0, timeline.1);
    assert!(gui.session.gesture.is_active());
    gui.lose_focus();
    assert!(!gui.session.gesture.is_active());
    gui.release(timeline.0, timeline.1);
}

#[test]
/// 掴んでいる間、窓の他の部品は押せない(`.dock-capture` が全面を取る)。
/// 部品の上で放しても menu は開かず、掴みだけが終わる。
fn a_held_tab_keeps_the_rest_of_the_window_inert() {
    let mut gui = Gui::open();
    let tab = gui.center_of("#dock-tab-Media", 0);
    gui.press(tab.0, tab.1);
    gui.motion(tab.0 + 30.0, tab.1 + 30.0);
    gui.settle();
    assert_eq!(gui.count(".dock-ghost"), 1);

    let file = gui.center_of("#menu-file", 0);
    gui.motion(file.0, file.1);
    gui.release(file.0, file.1);
    gui.settle();

    assert_eq!(gui.count("#menu-file-list"), 0, "a release over chrome opened a menu");
    assert_eq!(gui.count(".dock-ghost"), 0);
    assert_eq!(gui.count(".dropmap"), 0);
}

#[test]
fn file_drag_entry_cancels_a_splitter_before_showing_the_overlay() {
    let mut gui = Gui::open();
    let splitter = gui.center_of(".vgrip", 0);
    let before = gui.size_of_nth(".tslot", 1).0;
    gui.press(splitter.0, splitter.1);
    gui.enter_files(&["chaos.mov".into()]);
    gui.motion(splitter.0 + 120.0, splitter.1);
    gui.release(splitter.0 + 120.0, splitter.1);
    gui.settle();

    assert_eq!(gui.count(".file-drop-overlay"), 1);
    assert_eq!(gui.size_of_nth(".tslot", 1).0, before);
}

#[test]
fn rename_owns_shortcuts_and_escape_without_mutating_the_document() {
    let mut gui = Gui::open();
    let row = gui.center_of(".lsurface", 1);
    gui.click(row.0, row.1);
    gui.click(row.0, row.1);
    let selected = gui.session.selection.get();
    let revision = gui.session.doc.lock().unwrap().revision();
    let layers = gui.session.doc.lock().unwrap().view().layers();
    assert_eq!(gui.count("input.lsurface"), 1);

    gui.key(
        keyboard_types::Key::Character("d".into()),
        keyboard_types::Modifiers::SUPER,
    );
    gui.key(
        keyboard_types::Key::Delete,
        keyboard_types::Modifiers::empty(),
    );
    assert_eq!(gui.session.doc.lock().unwrap().view().layers(), layers);
    assert_eq!(gui.session.doc.lock().unwrap().revision(), revision);
    assert_eq!(gui.count("input.lsurface"), 1);

    gui.key(
        keyboard_types::Key::Escape,
        keyboard_types::Modifiers::empty(),
    );
    assert_eq!(gui.count("input.lsurface"), 0);
    assert_eq!(gui.session.selection.get(), selected);
    assert_eq!(gui.session.doc.lock().unwrap().revision(), revision);
}

#[test]
fn repeated_escape_and_undo_redo_boundaries_remain_idempotent() {
    let mut gui = Gui::open();
    let create = gui.center_of("#dock-tab-Create", 0);
    gui.click(create.0, create.1);
    let rectangle = gui.center_of(".tcard", 1);
    gui.click(rectangle.0, rectangle.1);
    let layers = gui.session.doc.lock().unwrap().view().layers();

    for _ in 0..100 {
        gui.key(
            keyboard_types::Key::Character("z".into()),
            keyboard_types::Modifiers::SUPER,
        );
        gui.key(
            keyboard_types::Key::Character("Z".into()),
            keyboard_types::Modifiers::SUPER | keyboard_types::Modifiers::SHIFT,
        );
    }
    assert_eq!(gui.session.doc.lock().unwrap().view().layers(), layers);
    let revision = gui.session.doc.lock().unwrap().revision();
    for _ in 0..20 {
        gui.key(
            keyboard_types::Key::Escape,
            keyboard_types::Modifiers::empty(),
        );
    }
    assert_eq!(gui.session.doc.lock().unwrap().revision(), revision);
}
// ---- 生成した嵐 ------------------------------------------------------------

/// 窓へ送れる一手。**利用者は決められた順番では触らない**ので、順番も座標も
/// 種類も生成させる。
#[derive(Debug, Clone)]
enum Poke {
    Press(f32, f32),
    Motion(f32, f32),
    ChordMotion(f32, f32),
    Release(f32, f32),
    Key(usize),
    Shortcut(usize),
    Wheel(f32, f32, f32, f32),
    FocusLoss,
    FileEnter(u8),
    FileLeave,
    Tick(u8),
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

fn storm_shortcut(i: usize) -> (keyboard_types::Key, keyboard_types::Modifiers) {
    use keyboard_types::{Key, Modifiers};
    match i {
        0 => (Key::Character("a".into()), Modifiers::SUPER),
        1 => (Key::Character("d".into()), Modifiers::SUPER),
        2 => (Key::Character("g".into()), Modifiers::SUPER),
        3 => (
            Key::Character("G".into()),
            Modifiers::SUPER | Modifiers::SHIFT,
        ),
        4 => (Key::Character("z".into()), Modifiers::SUPER),
        5 => (
            Key::Character("Z".into()),
            Modifiers::SUPER | Modifiers::SHIFT,
        ),
        _ => (Key::Character("[".into()), Modifiers::ALT),
    }
}

const STORM_SHORTCUT_COUNT: usize = 7;

fn poke() -> impl proptest::strategy::Strategy<Value = Poke> {
    use proptest::prelude::*;
    // 窓の外も混ぜる。掴んだまま外へ出るのは実際に起きる。
    let x = -200.0f32..(W as f32 + 200.0);
    let y = -200.0f32..(H as f32 + 200.0);
    prop_oneof![
        (x.clone(), y.clone()).prop_map(|(x, y)| Poke::Press(x, y)),
        (x.clone(), y.clone()).prop_map(|(x, y)| Poke::Motion(x, y)),
        (x.clone(), y.clone()).prop_map(|(x, y)| Poke::ChordMotion(x, y)),
        (x, y).prop_map(|(x, y)| Poke::Release(x, y)),
        (0..STORM_KEY_COUNT).prop_map(Poke::Key),
        (0..STORM_SHORTCUT_COUNT).prop_map(Poke::Shortcut),
        (
            -200.0f32..(W as f32 + 200.0),
            -200.0f32..(H as f32 + 200.0),
            -120.0f32..120.0,
            -120.0f32..120.0,
        )
            .prop_map(|(x, y, dx, dy)| Poke::Wheel(x, y, dx, dy)),
        proptest::strategy::Just(Poke::FocusLoss),
        (1u8..5).prop_map(Poke::FileEnter),
        proptest::strategy::Just(Poke::FileLeave),
        (1u8..13).prop_map(Poke::Tick),
    ]
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig {
        cases: 64,
        max_shrink_iters: 128,
        ..proptest::prelude::ProptestConfig::default()
    })]

    /// どんな順番で何を叩かれても、窓は操作を受け付け続ける。
    ///
    /// 落ちない事だけでは弱い —— 掴みが外れず固まる、置き場が空になる、
    /// タブが効かなくなる、が実際に起きる壊れ方。**最後に普通の一手が
    /// 通るか**まで見る。
    #[test]
    fn the_window_still_takes_orders_after_any_storm(storm in proptest::collection::vec(poke(), 1..96)) {
        let mut gui = Gui::open();
        for p in &storm {
            match *p {
                Poke::Press(x, y) => gui.press(x, y),
                Poke::Motion(x, y) => gui.motion(x, y),
                Poke::ChordMotion(x, y) => gui.chord_motion(x, y),
                Poke::Release(x, y) => gui.release(x, y),
                Poke::Key(i) => gui.key(storm_key(i), keyboard_types::Modifiers::empty()),
                Poke::Shortcut(i) => {
                    let (key, modifiers) = storm_shortcut(i);
                    gui.key(key, modifiers);
                }
                Poke::Wheel(x, y, dx, dy) => {
                    gui.h.wheel_at(x, y, f64::from(dx), f64::from(dy));
                    gui.settle();
                }
                Poke::FocusLoss => gui.lose_focus(),
                Poke::FileEnter(count) => {
                    let paths = (0..count)
                        .map(|index| std::path::PathBuf::from(format!("chaos-{index}.mov")))
                        .collect::<Vec<_>>();
                    gui.enter_files(&paths);
                }
                Poke::FileLeave => gui.leave_files(),
                Poke::Tick(frames) => gui.tick(f64::from(frames) / 60.0),
            }
        }
        // 指を上げて、掴みを解く。ここから先は普通の窓でなければならない。
        gui.leave_files();
        gui.release(10.0, 10.0);
        gui.lose_focus();
        for _ in 0..2 {
            gui.key(keyboard_types::Key::Escape, keyboard_types::Modifiers::empty());
        }
        gui.settle();

        proptest::prop_assert!(gui.drawing_panels() > 0, "描く panel が居なくなった: {storm:?}");
        proptest::prop_assert!(!gui.session.gesture.is_active(), "gestureが閉じていない: {storm:?}");
        proptest::prop_assert_eq!(gui.count(".dock-ghost"), 0, "dock ghostが残った: {:?}", storm);
        proptest::prop_assert_eq!(gui.count(".dropmap"), 0, "drop targetが残った: {:?}", storm);
        proptest::prop_assert_eq!(gui.count(".file-drop-overlay"), 0, "file overlayが残った: {:?}", storm);
        proptest::prop_assert_eq!(gui.count(".vmenu"), 0, "menuが残った: {:?}", storm);

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

        // 置き場と通常操作を復旧し、UIからDocumentへもう一手書けることまで見る。
        let view = gui.center_of("#menu-view", 0);
        gui.click(view.0, view.1);
        let reset = gui.center_of(".menu-section .vitem", 0);
        gui.click(reset.0, reset.1);
        let create = gui.center_of("#dock-tab-Create", 0);
        gui.click(create.0, create.1);
        let before_layers = gui.session.doc.lock().unwrap().view().layers().len();
        let rectangle = gui.center_of(".tcard", 1);
        gui.click(rectangle.0, rectangle.1);
        let after_layers = gui.session.doc.lock().unwrap().view().layers().len();
        proptest::prop_assert_eq!(after_layers, before_layers + 1, "嵐後のCreateが書けない: {:?}", storm);

        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("chaos.rrd");
        gui.session.doc.lock().unwrap().save(&project).unwrap();
        let loaded = crate::doc::store::Document::load(&project).unwrap();
        proptest::prop_assert_eq!(loaded.view().layers().len(), after_layers);
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

#[test]
fn dragging_a_tab_out_of_the_window_detaches_on_release() {
    let mut gui = Gui::open();
    let before = gui.count(".ptab");
    let (x, y) = gui.center_of("#dock-tab-Inspector", 0);

    gui.press(x, y);
    gui.motion(x, y + 40.0);
    gui.motion(-80.0, -80.0);
    assert_eq!(gui.count(".ptab"), before, "leaving detached before release");
    gui.release(-80.0, -80.0);
    gui.settle();

    assert_eq!(gui.count(".ptab"), before - 1, "outside release did not detach");
    assert_eq!(gui.count("#dock-tab-Inspector"), 0);
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

#[test]
fn clicking_a_tab_does_not_enter_docking_mode() {
    let mut gui = Gui::open();
    let (x, y) = gui.center_of("#dock-tab-Media", 0);

    gui.press(x, y);
    gui.settle();

    assert_eq!(
        gui.count(".dropmap.dragging"),
        0,
        "a click exposed dock targets before any drag"
    );
    assert_eq!(gui.count(".dock-ghost"), 0, "a click created a drag ghost");
    gui.release(x, y);
}

#[test]
fn escape_cancels_a_dock_drag_without_moving_the_panel() {
    let mut gui = Gui::open();
    let before = gui.texts(".ptab");
    let (x, y) = gui.center_of("#dock-tab-Media", 0);

    gui.press(x, y);
    gui.motion(x + 24.0, y + 24.0);
    gui.settle();
    assert!(
        gui.count(".dropmap.dragging") > 0,
        "dock targets did not appear after the drag threshold"
    );
    assert_eq!(
        gui.count(".dock-ghost"),
        1,
        "the dragged panel has no visible ghost"
    );

    gui.key(
        keyboard_types::Key::Escape,
        keyboard_types::Modifiers::empty(),
    );
    gui.release(x + 24.0, y + 24.0);
    gui.settle();

    assert_eq!(gui.count(".dropmap"), 0, "Escape left dock targets armed");
    assert_eq!(
        gui.count(".dock-ghost"),
        0,
        "Escape left the drag ghost open"
    );
    assert_eq!(gui.texts(".ptab"), before, "Escape moved a panel");
}

#[test]
fn a_split_out_panel_keeps_the_same_draggable_tab() {
    let mut gui = Gui::open();
    let tab_count = gui.count(".ptab");
    let zone_count = gui.count(".ptabs");
    let (x, y) = gui.center_of("#dock-tab-Colors", 0);

    gui.press(x, y);
    gui.motion(x + 24.0, y + 24.0);
    gui.settle();
    let target = gui.center_of(".dz.left", 1);
    gui.motion(target.0, target.1);
    gui.release(target.0, target.1);
    gui.settle();

    assert_eq!(
        gui.count(".ptab"),
        tab_count,
        "the split-out panel lost its tab handle"
    );
    assert_eq!(
        gui.count("#dock-tab-Colors"),
        1,
        "the split-out panel is not uniquely reachable"
    );
    assert_eq!(
        gui.count(".ptabs"),
        zone_count + 1,
        "the edge drop did not create a new dock zone"
    );
}

#[test]
fn every_right_hand_panel_uses_the_same_dock_path() {
    for panel in ["Inspector", "Desk"] {
        let mut gui = Gui::open();
        let (x, y) = gui.center_of(&format!("#dock-tab-{panel}"), 0);
        gui.press(x, y);
        let target = gui.center_of("#stage", 0);
        gui.motion(target.0, target.1);
        gui.release(target.0, target.1);
        gui.settle();

        let zones = gui.zone_tabs();
        assert!(
            zones[1].iter().any(|tab| tab == panel),
            "{panel} did not dock into the Stage zone: {zones:?}"
        );
    }
}

#[test]
fn every_panel_can_recreate_the_bottom_after_timeline_is_closed() {
    let panels = crate::ui::dock::Panel::all()
        .filter(|panel| *panel != crate::ui::dock::Panel::Timeline)
        .collect::<Vec<_>>();
    for moving in panels {
        let mut gui = Gui::open();
        let zones = gui.count(".ptabs");
        let tabs = gui.count(".ptab");
        let view = gui.center_of("#menu-view", 0);
        gui.click(view.0, view.1);
        let timeline = gui.center_of_text("#menu-view-list .vitem", "✓ Timeline");
        gui.click(timeline.0, timeline.1);
        assert_eq!(gui.count(".ptabs"), zones - 1, "Timeline row did not collapse");

        let tab = gui.center_of(&format!("#dock-tab-{moving}"), 0);
        let source_zone = gui
            .zone_tabs()
            .iter()
            .position(|tabs| tabs.iter().any(|tab| tab == moving.label()))
            .expect("moving panel source zone");
        // 独りの tab を自分の下へ落としても列は増えない(元の箱が消える)。
        let alone = gui.zone_tabs()[source_zone].len() == 1;
        gui.press(tab.0, tab.1);
        gui.motion(tab.0 + 24.0, tab.1 + 24.0);
        gui.settle();
        let bottom = gui.center_of(".dz.bottom", source_zone);
        gui.motion(bottom.0, bottom.1);
        gui.release(bottom.0, bottom.1);
        gui.settle();

        assert_eq!(
            gui.count(".ptabs"),
            if alone { zones - 1 } else { zones },
            "{moving} did not recreate the bottom"
        );
        assert_eq!(gui.count(".ptab"), tabs - 1, "{moving} made another panel disappear");
        assert!(gui.drawing_panels() > 0, "{moving} removed all rendered content");
        for panel in crate::ui::dock::Panel::all() {
            assert_eq!(
                gui.count(&format!("#dock-tab-{panel}")),
                usize::from(panel != crate::ui::dock::Panel::Timeline),
                "moving {moving} changed {panel} reachability"
            );
        }
    }
}

#[test]
fn reset_layout_is_a_visible_way_back_from_a_custom_dock() {
    let mut gui = Gui::open();
    let default_zones = gui.count(".ptabs");
    let (x, y) = gui.center_of("#dock-tab-Colors", 0);
    gui.press(x, y);
    gui.motion(x + 24.0, y + 24.0);
    gui.settle();
    let target = gui.center_of(".dz.left", 1);
    gui.motion(target.0, target.1);
    gui.release(target.0, target.1);
    gui.settle();
    assert_eq!(gui.count(".ptabs"), default_zones + 1);

    let view = gui.center_of("#menu-view", 0);
    gui.click(view.0, view.1);
    let reset = gui.center_of(".menu-section .vitem", 0);
    gui.click(reset.0, reset.1);

    assert_eq!(
        gui.count(".ptabs"),
        default_zones,
        "Reset Layout did not restore the default dock"
    );
    assert_eq!(gui.count(".ptab"), crate::ui::dock::Panel::all().count());
}

#[test]
fn splitters_have_a_normal_hit_target() {
    let mut gui = Gui::open();
    let vertical = gui.size_of_nth(".vgrip", 0);
    let horizontal = gui.size_of_nth(".hgrip", 0);
    assert!(
        vertical.0 >= 6.0,
        "vertical splitter is too thin to grab: {vertical:?}"
    );
    assert!(
        horizontal.1 >= 6.0,
        "horizontal splitter is too thin to grab: {horizontal:?}"
    );
}

#[test]
fn a_splitter_uses_the_live_taffy_extent_without_borrowing_the_document_twice() {
    let mut gui = Gui::open();
    let before = gui.size_of_nth(".tslot", 1).0;
    let (x, y) = gui.center_of(".vgrip", 0);

    gui.press(x, y);
    gui.motion(x + 100.0, y);
    gui.release(x + 100.0, y);
    gui.settle();

    let after = gui.size_of_nth(".tslot", 1).0;
    assert!(
        after > before,
        "splitter did not follow the pointer: {before} -> {after}"
    );
}

#[test]
fn every_panel_tab_remains_reachable_at_ordinary_small_window_sizes() {
    let mut clipped = Vec::new();
    for (width, height) in [(640, 480), (900, 600), (1280, 800), (1600, 1000)] {
        let gui = Gui::open_at(width, height);
        let overflow = gui.tab_strip_overflow();
        if !overflow.is_empty() {
            clipped.push(((width, height), overflow));
        }
    }
    assert!(clipped.is_empty(), "panel tabs are clipped: {clipped:?}");
}

#[test]
fn losing_window_focus_cancels_tab_drag() {
    let mut gui = Gui::open();
    let tab = gui.center_of("#dock-tab-Media", 0);
    gui.press(tab.0, tab.1);
    gui.motion(tab.0 + 24.0, tab.1 + 24.0);
    gui.settle();
    assert_eq!(gui.count(".dock-ghost"), 1);

    gui.lose_focus();
    assert_eq!(
        gui.count(".dock-ghost"),
        0,
        "focus loss left a dock drag active"
    );
    assert_eq!(
        gui.count(".dropmap"),
        0,
        "focus loss left dock targets armed"
    );
}

#[test]
fn losing_window_focus_cancels_splitter_drag() {
    let mut gui = Gui::open();
    let splitter = gui.center_of(".vgrip", 0);
    gui.press(splitter.0, splitter.1);
    gui.lose_focus();
    let before = gui.size_of_nth(".tslot", 1).0;
    gui.motion(splitter.0 + 100.0, splitter.1);
    gui.release(splitter.0 + 100.0, splitter.1);
    gui.settle();
    let after = gui.size_of_nth(".tslot", 1).0;
    assert_eq!(after, before, "focus loss left a splitter drag active");
}

