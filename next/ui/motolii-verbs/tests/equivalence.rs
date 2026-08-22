//! 移行の安全証明: `motolii_verbs::generate::*` の出力が既存
//! `motolii_menubar::menus`/`motolii_menubar::context` の出力と
//! **label・shortcut・message の並び**まで一致することを確認する
//! (発注書「生成結果が現行の menus.rs/context.rs の出力と一致すること」)。
//!
//! 次切片で shell の呼び出しをこの crate の関数へ差し替える判断は、この
//! テストが green であることを根拠にする — 落ちたら移行してはいけない。

use motolii_menubar::context::{
    CanvasContextMessages, ClipContextMessages, KeyframeContextMessages, LayerRowContextMessages,
};
use motolii_menubar::menus::{
    EditMenuMessages, HelpMenuMessages, LayerMenuMessages, WindowMenuMessages,
};
use motolii_menubar::{context as old_context, menus as old_menus, Item, Menu};
use motolii_verbs::generate as new;

#[derive(Debug, Clone, PartialEq)]
struct FakeMessage(&'static str);

fn edit_messages() -> EditMenuMessages<FakeMessage> {
    EditMenuMessages {
        undo: FakeMessage("undo"),
        redo: FakeMessage("redo"),
        cut: FakeMessage("cut"),
        copy: FakeMessage("copy"),
        paste: FakeMessage("paste"),
        duplicate: FakeMessage("duplicate"),
        select_all: FakeMessage("select_all"),
        deselect_all: FakeMessage("deselect_all"),
    }
}

fn layer_messages() -> LayerMenuMessages<FakeMessage> {
    LayerMenuMessages {
        new_layer: FakeMessage("new_layer"),
        group: FakeMessage("group"),
        ungroup: FakeMessage("ungroup"),
        freeze: FakeMessage("freeze"),
        unfreeze: FakeMessage("unfreeze"),
        toggle_hide: FakeMessage("toggle_hide"),
        toggle_solo: FakeMessage("toggle_solo"),
        toggle_lock: FakeMessage("toggle_lock"),
        cycle_label_color: FakeMessage("cycle_label_color"),
    }
}

fn window_messages() -> WindowMenuMessages<FakeMessage> {
    WindowMenuMessages {
        toggle_browser: FakeMessage("toggle_browser"),
        focus_inspector: FakeMessage("focus_inspector"),
        focus_stage: FakeMessage("focus_stage"),
        focus_timeline: FakeMessage("focus_timeline"),
        cycle_panel: FakeMessage("cycle_panel"),
        close_panel: FakeMessage("close_panel"),
    }
}

fn help_messages() -> HelpMenuMessages<FakeMessage> {
    HelpMenuMessages {
        open_documentation: FakeMessage("open_documentation"),
        open_community_forum: FakeMessage("open_community_forum"),
        send_feedback: FakeMessage("send_feedback"),
    }
}

fn clip_messages() -> ClipContextMessages<FakeMessage> {
    ClipContextMessages {
        copy: FakeMessage("copy"),
        paste: FakeMessage("paste"),
        duplicate: FakeMessage("duplicate"),
        cut: FakeMessage("cut"),
        group: FakeMessage("group"),
        freeze: FakeMessage("freeze"),
        cycle_label_color: FakeMessage("cycle_label_color"),
    }
}

fn layer_row_messages() -> LayerRowContextMessages<FakeMessage> {
    LayerRowContextMessages {
        rename: FakeMessage("rename"),
        bring_forward: FakeMessage("bring_forward"),
        send_backward: FakeMessage("send_backward"),
        bring_to_front: FakeMessage("bring_to_front"),
        send_to_back: FakeMessage("send_to_back"),
        toggle_hide: FakeMessage("toggle_hide"),
        toggle_solo: FakeMessage("toggle_solo"),
        toggle_lock: FakeMessage("toggle_lock"),
    }
}

fn canvas_messages() -> CanvasContextMessages<FakeMessage> {
    CanvasContextMessages {
        select_all: FakeMessage("select_all"),
        deselect_all: FakeMessage("deselect_all"),
        group: FakeMessage("group"),
    }
}

fn keyframe_messages() -> KeyframeContextMessages<FakeMessage> {
    KeyframeContextMessages {
        hold: FakeMessage("hold"),
        linear: FakeMessage("linear"),
        easy_ease: FakeMessage("easy_ease"),
        easy_ease_in: FakeMessage("easy_ease_in"),
        easy_ease_out: FakeMessage("easy_ease_out"),
        delete: FakeMessage("delete"),
    }
}

/// `Menu<M>` を (label, [(item_label, shortcut, message)]) へ潰して比較しやすくする。
fn flatten_menu(
    menu: Menu<FakeMessage>,
) -> (
    &'static str,
    Vec<(&'static str, Option<&'static str>, FakeMessage)>,
) {
    (
        menu.label,
        menu.items
            .into_iter()
            .map(
                |Item {
                     label,
                     shortcut,
                     message,
                 }| (label, shortcut, message),
            )
            .collect(),
    )
}

fn flatten_items(
    items: Vec<Item<FakeMessage>>,
) -> Vec<(&'static str, Option<&'static str>, FakeMessage)> {
    items
        .into_iter()
        .map(
            |Item {
                 label,
                 shortcut,
                 message,
             }| (label, shortcut, message),
        )
        .collect()
}

#[test]
fn edit_menu_matches_existing_output() {
    let old = flatten_menu(old_menus::edit_menu(edit_messages()));
    let generated = flatten_menu(new::edit_menu(edit_messages()));
    assert_eq!(old, generated);
}

#[test]
fn layer_menu_matches_existing_output() {
    let old = flatten_menu(old_menus::layer_menu(layer_messages()));
    let generated = flatten_menu(new::layer_menu(layer_messages()));
    assert_eq!(old, generated);
}

#[test]
fn window_menu_matches_existing_output() {
    let old = flatten_menu(old_menus::window_menu(window_messages()));
    let generated = flatten_menu(new::window_menu(window_messages()));
    assert_eq!(old, generated);
}

#[test]
fn help_menu_matches_existing_output() {
    let old = flatten_menu(old_menus::help_menu(help_messages()));
    let generated = flatten_menu(new::help_menu(help_messages()));
    assert_eq!(old, generated);
}

#[test]
fn clip_context_items_match_existing_output() {
    let old = flatten_items(old_context::clip_context_items(clip_messages()));
    let generated = flatten_items(new::clip_context_items(clip_messages()));
    assert_eq!(old, generated);
}

#[test]
fn layer_row_context_items_match_existing_output() {
    let old = flatten_items(old_context::layer_row_context_items(layer_row_messages()));
    let generated = flatten_items(new::layer_row_context_items(layer_row_messages()));
    assert_eq!(old, generated);
}

#[test]
fn canvas_context_items_match_existing_output() {
    let old = flatten_items(old_context::canvas_context_items(canvas_messages()));
    let generated = flatten_items(new::canvas_context_items(canvas_messages()));
    assert_eq!(old, generated);
}

#[test]
fn keyframe_context_items_match_existing_output() {
    let old = flatten_items(old_context::keyframe_context_items(keyframe_messages()));
    let generated = flatten_items(new::keyframe_context_items(keyframe_messages()));
    assert_eq!(old, generated);
}
