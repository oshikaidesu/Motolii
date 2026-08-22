//! [`crate::registry`] から `motolii_menubar::{Item, Menu}` を生成する。
//!
//! シグネチャは既存 `motolii_menubar::menus`/`motolii_menubar::context` の
//! 対応する関数と**同じ**(引数の `XxxMessages<M>` 構造体もそのまま再利用 —
//! 定義し直さない)。次切片で shell の呼び出しをこちらへ差し替えても
//! 呼び手側のコードは変わらない(発注書「既存 API と同じ形の出力にする」)。
//!
//! この関数群が既存 `menus.rs`/`context.rs` の出力と一致することは
//! `tests/equivalence.rs` で検証済み(移行の安全証明)。ラベル/shortcut の
//! 正本は [`crate::registry`] の `Verb` 定数へ移った — この関数群はもう
//! 文字列リテラルを持たない(並び順と「どのフィールドがどの `Verb` に
//! 対応するか」の対応表だけを持つ)。

use crate::registry as v;
use crate::Verb;
use motolii_menubar::context::{
    CanvasContextMessages, ClipContextMessages, KeyframeContextMessages, LayerRowContextMessages,
};
use motolii_menubar::menus::{
    EditMenuMessages, HelpMenuMessages, LayerMenuMessages, WindowMenuMessages,
};
use motolii_menubar::{Item, Menu};

/// `Verb` の label/shortcut を、呼び手が渡した具体 `message` と組んで
/// `Item` にする。全生成関数がこの1関数だけを通る — ラベル/shortcut の
/// 文字列リテラルはここにも無い(`Verb` から読むだけ)。
fn item<M>(verb: &Verb, message: M) -> Item<M> {
    Item {
        label: verb.label,
        shortcut: verb.shortcut,
        message,
    }
}

/// `motolii_menubar::menus::edit_menu` と同じ出力(`registry::EDIT_MENU` の
/// 並びを踏襲)。
pub fn edit_menu<M>(items: EditMenuMessages<M>) -> Menu<M> {
    debug_assert_eq!(
        v::EDIT_MENU.len(),
        8,
        "registry::EDIT_MENU の項目数が変わった"
    );
    Menu {
        label: "Edit",
        items: vec![
            item(&v::UNDO, items.undo),
            item(&v::REDO, items.redo),
            item(&v::CUT, items.cut),
            item(&v::COPY, items.copy),
            item(&v::PASTE, items.paste),
            item(&v::DUPLICATE, items.duplicate),
            item(&v::SELECT_ALL, items.select_all),
            item(&v::DESELECT_ALL, items.deselect_all),
        ],
    }
}

/// `motolii_menubar::menus::layer_menu` と同じ出力(`registry::LAYER_MENU` の
/// 並びを踏襲)。
pub fn layer_menu<M>(items: LayerMenuMessages<M>) -> Menu<M> {
    debug_assert_eq!(
        v::LAYER_MENU.len(),
        9,
        "registry::LAYER_MENU の項目数が変わった"
    );
    Menu {
        label: "Layer",
        items: vec![
            item(&v::NEW_LAYER, items.new_layer),
            item(&v::GROUP, items.group),
            item(&v::UNGROUP, items.ungroup),
            item(&v::FREEZE, items.freeze),
            item(&v::UNFREEZE, items.unfreeze),
            item(&v::TOGGLE_HIDE, items.toggle_hide),
            item(&v::TOGGLE_SOLO, items.toggle_solo),
            item(&v::TOGGLE_LOCK, items.toggle_lock),
            item(&v::CYCLE_LABEL_COLOR, items.cycle_label_color),
        ],
    }
}

/// `motolii_menubar::menus::window_menu` と同じ出力(`registry::WINDOW_MENU`
/// の並びを踏襲)。
pub fn window_menu<M>(items: WindowMenuMessages<M>) -> Menu<M> {
    debug_assert_eq!(
        v::WINDOW_MENU.len(),
        6,
        "registry::WINDOW_MENU の項目数が変わった"
    );
    Menu {
        label: "Window",
        items: vec![
            item(&v::TOGGLE_BROWSER, items.toggle_browser),
            item(&v::FOCUS_INSPECTOR, items.focus_inspector),
            item(&v::FOCUS_STAGE, items.focus_stage),
            item(&v::FOCUS_TIMELINE, items.focus_timeline),
            item(&v::CYCLE_PANEL, items.cycle_panel),
            item(&v::CLOSE_PANEL, items.close_panel),
        ],
    }
}

/// `motolii_menubar::menus::help_menu` と同じ出力(`registry::HELP_MENU` の
/// 並びを踏襲)。
pub fn help_menu<M>(items: HelpMenuMessages<M>) -> Menu<M> {
    debug_assert_eq!(
        v::HELP_MENU.len(),
        3,
        "registry::HELP_MENU の項目数が変わった"
    );
    Menu {
        label: "Help",
        items: vec![
            item(&v::OPEN_DOCUMENTATION, items.open_documentation),
            item(&v::OPEN_COMMUNITY_FORUM, items.open_community_forum),
            item(&v::SEND_FEEDBACK, items.send_feedback),
        ],
    }
}

/// `motolii_menubar::context::clip_context_items` と同じ出力
/// (`registry::CLIP_CONTEXT` の並びを踏襲)。
pub fn clip_context_items<M>(items: ClipContextMessages<M>) -> Vec<Item<M>> {
    debug_assert_eq!(
        v::CLIP_CONTEXT.len(),
        7,
        "registry::CLIP_CONTEXT の項目数が変わった"
    );
    vec![
        item(&v::COPY, items.copy),
        item(&v::PASTE, items.paste),
        item(&v::DUPLICATE, items.duplicate),
        item(&v::CUT, items.cut),
        item(&v::GROUP, items.group),
        item(&v::FREEZE, items.freeze),
        item(&v::CYCLE_LABEL_COLOR, items.cycle_label_color),
    ]
}

/// `motolii_menubar::context::layer_row_context_items` と同じ出力
/// (`registry::LAYER_ROW_CONTEXT` の並びを踏襲)。
pub fn layer_row_context_items<M>(items: LayerRowContextMessages<M>) -> Vec<Item<M>> {
    debug_assert_eq!(
        v::LAYER_ROW_CONTEXT.len(),
        8,
        "registry::LAYER_ROW_CONTEXT の項目数が変わった"
    );
    vec![
        item(&v::RENAME, items.rename),
        item(&v::BRING_FORWARD, items.bring_forward),
        item(&v::SEND_BACKWARD, items.send_backward),
        item(&v::BRING_TO_FRONT, items.bring_to_front),
        item(&v::SEND_TO_BACK, items.send_to_back),
        item(&v::TOGGLE_HIDE, items.toggle_hide),
        item(&v::TOGGLE_SOLO, items.toggle_solo),
        item(&v::TOGGLE_LOCK, items.toggle_lock),
    ]
}

/// `motolii_menubar::context::canvas_context_items` と同じ出力
/// (`registry::CANVAS_CONTEXT` の並びを踏襲)。
pub fn canvas_context_items<M>(items: CanvasContextMessages<M>) -> Vec<Item<M>> {
    debug_assert_eq!(
        v::CANVAS_CONTEXT.len(),
        3,
        "registry::CANVAS_CONTEXT の項目数が変わった"
    );
    vec![
        item(&v::SELECT_ALL, items.select_all),
        item(&v::DESELECT_ALL, items.deselect_all),
        item(&v::GROUP, items.group),
    ]
}

/// `motolii_menubar::context::keyframe_context_items` と同じ出力
/// (`registry::KEYFRAME_CONTEXT` の並びを踏襲)。
pub fn keyframe_context_items<M>(items: KeyframeContextMessages<M>) -> Vec<Item<M>> {
    debug_assert_eq!(
        v::KEYFRAME_CONTEXT.len(),
        6,
        "registry::KEYFRAME_CONTEXT の項目数が変わった"
    );
    vec![
        item(&v::INTERP_HOLD, items.hold),
        item(&v::INTERP_LINEAR, items.linear),
        item(&v::INTERP_EASY_EASE, items.easy_ease),
        item(&v::INTERP_EASY_EASE_IN, items.easy_ease_in),
        item(&v::INTERP_EASY_EASE_OUT, items.easy_ease_out),
        item(&v::DELETE_KEYS, items.delete),
    ]
}
