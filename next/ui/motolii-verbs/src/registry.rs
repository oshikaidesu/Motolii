//! 動詞の正本(38動詞)。出典は `motolii_menubar::menus`/`motolii_menubar::context`
//! のモジュール冒頭 doc(表)をそのまま転記した — この crate はそれらの表の
//! **機械可読版**であって、新しい判断は増やしていない(発注書「新しい動詞は
//! 作らない、写すだけ」)。
//!
//! 各 `Verb` の `entries` は [`crate::s6_checked`] を通す(通さないと
//! コンパイルが失敗する構造 — モジュール冒頭 doc 参照)。1動詞が複数の
//! 入口(例: `Copy` はメニュー+右クリック)を持つ場合、[`ALL_VERBS`] には
//! 1回しか現れないが、[`EDIT_MENU`]/[`CLIP_CONTEXT`] のような**並び順スライス**
//! には(同じ `&'static Verb` として)複数回現れる — これが「1回書いて
//! 複数の入口リストから参照する」という発注書の狙いそのもの。

use crate::{s6_checked, ContextSlot, Entry, MenuSlot, PanelSlot, Verb};

// ---------------------------------------------------------------------------
// Edit(motolii_menubar::menus::edit_menu、bundle B33/B31)
// ---------------------------------------------------------------------------

/// normal-map 437(437/467で shortcut確認)。
pub static UNDO: Verb = Verb {
    id: "edit.undo",
    label: "Undo",
    shortcut: Some("Cmd+Z"),
    entries: s6_checked(&[Entry::Menu(MenuSlot::Edit)]),
    map_ids: &[437],
    bundle: Some("B33"),
};

/// normal-map 435(467で shortcut確認)。
pub static REDO: Verb = Verb {
    id: "edit.redo",
    label: "Redo",
    shortcut: Some("Cmd+Shift+Z"),
    entries: s6_checked(&[Entry::Menu(MenuSlot::Edit)]),
    map_ids: &[435],
    bundle: Some("B33"),
};

/// normal-map 432。クリップ右クリックの唯一の削除動詞でもある
/// (`motolii_menubar::context` モジュール冒頭 doc「Delete についての注記」)。
pub static CUT: Verb = Verb {
    id: "edit.cut",
    label: "Cut",
    shortcut: Some("Cmd+X"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Edit),
        Entry::Context(ContextSlot::Clip),
    ]),
    map_ids: &[432],
    bundle: Some("B33"),
};

/// normal-map 429。
pub static COPY: Verb = Verb {
    id: "edit.copy",
    label: "Copy",
    shortcut: Some("Cmd+C"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Edit),
        Entry::Context(ContextSlot::Clip),
    ]),
    map_ids: &[429],
    bundle: Some("B33"),
};

/// normal-map 430。
pub static PASTE: Verb = Verb {
    id: "edit.paste",
    label: "Paste",
    shortcut: Some("Cmd+V"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Edit),
        Entry::Context(ContextSlot::Clip),
    ]),
    map_ids: &[430],
    bundle: Some("B33"),
};

/// normal-map 434。
pub static DUPLICATE: Verb = Verb {
    id: "edit.duplicate",
    label: "Duplicate",
    shortcut: Some("Cmd+D"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Edit),
        Entry::Context(ContextSlot::Clip),
    ]),
    map_ids: &[434],
    bundle: Some("B33"),
};

/// clip を再生ヘッド位置で2本に割る(E-1、GOALS M6 の最後の1件)。
/// 入口は Cmd+K と clip 右クリックの2つ(S6 併存)。
pub static SPLIT: Verb = Verb {
    id: "edit.split",
    label: "Split",
    shortcut: Some("Cmd+K"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/input.rs(Cmd+K → SplitAtPlayhead)"),
        Entry::Context(ContextSlot::Clip),
    ]),
    map_ids: &[],
    bundle: Some("B39"),
};

/// normal-map 717(B19)。マーカーを playhead へ即置き(Add Marker)。
/// 入口は M キーとルーラ locator lane 右クリックの2つ(S6 併存、裁定195・
/// S2 発注 #22「追加 UI が無い」の穴埋め)。**両方とも「隠れていない」
/// 入口**(`ShortcutOnly` — `Entry::is_hidden` は `Context`/`PanelControl`
/// だけを隠れた入口とみなす)なので、S6 判定自体は単独でも通る形だが、
/// 発注(裁定222)が明示的に2入口の実装を要求したため両方を転記する。
/// M キーは normal-map 717 のコメント「4製品慣習」どおり(AE/Premiere/
/// Resolve/CapCut とも M=Add Marker at playhead)。右クリックの方は
/// クリック位置ではなく playhead を使う(`ruler.rs` の doc コメント引用元
/// 参照 — Premiere/Resolve とも右クリック追加は playhead 基準)。
pub static ADD_MARKER: Verb = Verb {
    id: "timeline.add_marker",
    label: "Add Marker",
    shortcut: Some("M"),
    entries: s6_checked(&[
        Entry::ShortcutOnly(
            "next/shell/motolii-shell/src/input.rs(M → Message::Marker(MarkerMessage::AddAtPlayhead))",
        ),
        Entry::ShortcutOnly(
            "next/ui/motolii-timeline-pane/src/ruler.rs(ルーラ locator lane 右クリック → Message::AddMarkerAt(playhead))",
        ),
    ]),
    map_ids: &[717],
    bundle: Some("B19"),
};

/// normal-map 436、bundle B31(B33外 — 既存 shell 定義の移送)。
pub static SELECT_ALL: Verb = Verb {
    id: "edit.select_all",
    label: "Select All",
    shortcut: Some("Cmd+A"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Edit),
        Entry::Context(ContextSlot::Canvas),
    ]),
    map_ids: &[436],
    bundle: Some("B31"),
};

/// normal-map 433、bundle B31(同上)。
pub static DESELECT_ALL: Verb = Verb {
    id: "edit.deselect_all",
    label: "Deselect All",
    shortcut: Some("Cmd+Shift+A"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Edit),
        Entry::Context(ContextSlot::Canvas),
    ]),
    map_ids: &[433],
    bundle: Some("B31"),
};

/// Edit メニューの並び(`motolii_menubar::menus::edit_menu` と一致)。
pub static EDIT_MENU: &[&Verb] = &[
    &UNDO,
    &REDO,
    &CUT,
    &COPY,
    &PASTE,
    &DUPLICATE,
    &SELECT_ALL,
    &DESELECT_ALL,
];

// ---------------------------------------------------------------------------
// Layer(motolii_menubar::menus::layer_menu、bundle B34/裁定119/裁定195)
// ---------------------------------------------------------------------------

/// normal-map出典ゼロ(旧 "+Layer" 箱ボタン)。
pub static NEW_LAYER: Verb = Verb {
    id: "layer.new_layer",
    label: "New Layer",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Layer)]),
    map_ids: &[],
    bundle: None,
};

/// normal-map 455/456/457(3行が同じ動詞へ収束、G1・裁定174)。
pub static GROUP: Verb = Verb {
    id: "layer.group",
    label: "Group",
    shortcut: Some("Cmd+G"),
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Layer),
        Entry::Context(ContextSlot::Clip),
        Entry::Context(ContextSlot::Canvas),
    ]),
    map_ids: &[455, 456, 457],
    bundle: Some("B34"),
};

/// normal-map 468/469/470(同上)。
pub static UNGROUP: Verb = Verb {
    id: "layer.ungroup",
    label: "Ungroup",
    shortcut: Some("Cmd+Shift+G"),
    entries: s6_checked(&[Entry::Menu(MenuSlot::Layer)]),
    map_ids: &[468, 469, 470],
    bundle: Some("B34"),
};

/// 裁定119(意図動詞、normal-map出典ゼロ)。
pub static FREEZE: Verb = Verb {
    id: "layer.freeze",
    label: "Freeze",
    shortcut: None,
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Layer),
        Entry::Context(ContextSlot::Clip),
    ]),
    map_ids: &[],
    bundle: None,
};

/// 裁定119(同上)。
pub static UNFREEZE: Verb = Verb {
    id: "layer.unfreeze",
    label: "Unfreeze",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Layer)]),
    map_ids: &[],
    bundle: None,
};

/// `timeline_pane::Message::ToggleMute`。裁定195で Menu/Context の第二入口を
/// 追加(`rail.rs:337` の mute glyph が唯一の入口だった単一入口を穴埋め)。
pub static TOGGLE_HIDE: Verb = Verb {
    id: "layer.toggle_hide",
    label: "Hide",
    shortcut: None,
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Layer),
        Entry::Context(ContextSlot::LayerRow),
        Entry::PanelControl(PanelSlot::RailGlyph),
    ]),
    map_ids: &[],
    bundle: Some("B32"),
};

/// `timeline_pane::Message::ToggleSolo`(`rail.rs:338`、同上)。
pub static TOGGLE_SOLO: Verb = Verb {
    id: "layer.toggle_solo",
    label: "Solo",
    shortcut: None,
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Layer),
        Entry::Context(ContextSlot::LayerRow),
        Entry::PanelControl(PanelSlot::RailGlyph),
    ]),
    map_ids: &[],
    bundle: Some("B32"),
};

/// `timeline_pane::Message::ToggleLock`(`rail.rs:339`、同上)。
pub static TOGGLE_LOCK: Verb = Verb {
    id: "layer.toggle_lock",
    label: "Lock",
    shortcut: None,
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Layer),
        Entry::Context(ContextSlot::LayerRow),
        Entry::PanelControl(PanelSlot::RailGlyph),
    ]),
    map_ids: &[],
    bundle: Some("B32"),
};

/// `inspector_pane::Message::CycleLabelColor`(`inspector-pane/src/lib.rs:2827`
/// の色 swatch button が唯一の入口だった単一入口を裁定195で穴埋め)。
pub static CYCLE_LABEL_COLOR: Verb = Verb {
    id: "layer.cycle_label_color",
    label: "Label Color",
    shortcut: None,
    entries: s6_checked(&[
        Entry::Menu(MenuSlot::Layer),
        Entry::Context(ContextSlot::Clip),
        Entry::PanelControl(PanelSlot::InspectorSwatch),
    ]),
    map_ids: &[],
    bundle: None,
};

/// Layer メニューの並び(`motolii_menubar::menus::layer_menu` と一致)。
pub static LAYER_MENU: &[&Verb] = &[
    &NEW_LAYER,
    &GROUP,
    &UNGROUP,
    &FREEZE,
    &UNFREEZE,
    &TOGGLE_HIDE,
    &TOGGLE_SOLO,
    &TOGGLE_LOCK,
    &CYCLE_LABEL_COLOR,
];

// ---------------------------------------------------------------------------
// Window(motolii_menubar::menus::window_menu、bundle B25)
// ---------------------------------------------------------------------------

/// normal-map 1525 "Open or close Project panel"。
pub static TOGGLE_BROWSER: Verb = Verb {
    id: "window.toggle_browser",
    label: "Browser",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Window)]),
    map_ids: &[1525],
    bundle: Some("B25"),
};

/// normal-map 801 "Active Window: Inspector"。
pub static FOCUS_INSPECTOR: Verb = Verb {
    id: "window.focus_inspector",
    label: "Inspector",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Window)]),
    map_ids: &[801],
    bundle: Some("B25"),
};

/// normal-map 1317 "Active Window: Timeline Viewer"。
pub static FOCUS_STAGE: Verb = Verb {
    id: "window.focus_stage",
    label: "Stage",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Window)]),
    map_ids: &[1317],
    bundle: Some("B25"),
};

/// normal-map 1316 "Active Window: Timeline"。
pub static FOCUS_TIMELINE: Verb = Verb {
    id: "window.focus_timeline",
    label: "Timeline",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Window)]),
    map_ids: &[1316],
    bundle: Some("B25"),
};

/// normal-map 1503 "Cycle to previous or next panel in active frame"。
pub static CYCLE_PANEL: Verb = Verb {
    id: "window.cycle_panel",
    label: "Cycle Panel",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Window)]),
    map_ids: &[1503],
    bundle: Some("B25"),
};

/// normal-map 1499(1500と重複統合)。
pub static CLOSE_PANEL: Verb = Verb {
    id: "window.close_panel",
    label: "Close Panel",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Window)]),
    map_ids: &[1499],
    bundle: Some("B25"),
};

/// Window メニューの並び(`motolii_menubar::menus::window_menu` と一致)。
pub static WINDOW_MENU: &[&Verb] = &[
    &TOGGLE_BROWSER,
    &FOCUS_INSPECTOR,
    &FOCUS_STAGE,
    &FOCUS_TIMELINE,
    &CYCLE_PANEL,
    &CLOSE_PANEL,
];

// ---------------------------------------------------------------------------
// Help(motolii_menubar::menus::help_menu、bundle B06)
// ---------------------------------------------------------------------------

/// normal-map 582(572/579/580重複統合)。
pub static OPEN_DOCUMENTATION: Verb = Verb {
    id: "help.open_documentation",
    label: "Documentation",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Help)]),
    map_ids: &[582],
    bundle: Some("B06"),
};

/// normal-map 585(573重複統合)。
pub static OPEN_COMMUNITY_FORUM: Verb = Verb {
    id: "help.open_community_forum",
    label: "Community Forum",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Help)]),
    map_ids: &[585],
    bundle: Some("B06"),
};

/// normal-map 588。
pub static SEND_FEEDBACK: Verb = Verb {
    id: "help.send_feedback",
    label: "Send Feedback…",
    shortcut: None,
    entries: s6_checked(&[Entry::Menu(MenuSlot::Help)]),
    map_ids: &[588],
    bundle: Some("B06"),
};

/// Help メニューの並び(`motolii_menubar::menus::help_menu` と一致)。
pub static HELP_MENU: &[&Verb] = &[&OPEN_DOCUMENTATION, &OPEN_COMMUNITY_FORUM, &SEND_FEEDBACK];

// ---------------------------------------------------------------------------
// レイヤー行右クリック専用動詞(`motolii_menubar::context::SHORTCUT_ONLY_REGISTRY`
// + `layer_row_context_items`)。メニューには存在しない — shortcut(実装済み)
// が第二の入口。
// ---------------------------------------------------------------------------

/// 正典 §6。shortcut 出典: `next/shell/motolii-shell/src/lib.rs:5391`。
pub static RENAME: Verb = Verb {
    id: "layer_row.rename",
    label: "Rename",
    shortcut: Some("Enter"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/lib.rs:5391"),
        Entry::Context(ContextSlot::LayerRow),
    ]),
    map_ids: &[],
    bundle: None,
};

/// `StackDirection::Forward`。shortcut 出典:
/// `next/shell/motolii-shell/src/lib.rs:5399-5417`。
pub static BRING_FORWARD: Verb = Verb {
    id: "layer_row.bring_forward",
    label: "Bring Forward",
    shortcut: Some("Cmd+Opt+Up"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/lib.rs:5399-5417"),
        Entry::Context(ContextSlot::LayerRow),
    ]),
    map_ids: &[],
    bundle: None,
};

/// `StackDirection::Backward`(同上)。
pub static SEND_BACKWARD: Verb = Verb {
    id: "layer_row.send_backward",
    label: "Send Backward",
    shortcut: Some("Cmd+Opt+Down"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/lib.rs:5399-5417"),
        Entry::Context(ContextSlot::LayerRow),
    ]),
    map_ids: &[],
    bundle: None,
};

/// `StackDirection::ToFront`(同上)。
pub static BRING_TO_FRONT: Verb = Verb {
    id: "layer_row.bring_to_front",
    label: "Bring to Front",
    shortcut: Some("Cmd+Opt+Shift+Up"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/lib.rs:5399-5417"),
        Entry::Context(ContextSlot::LayerRow),
    ]),
    map_ids: &[],
    bundle: None,
};

/// `StackDirection::ToBack`(同上)。
pub static SEND_TO_BACK: Verb = Verb {
    id: "layer_row.send_to_back",
    label: "Send to Back",
    shortcut: Some("Cmd+Opt+Shift+Down"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/lib.rs:5399-5417"),
        Entry::Context(ContextSlot::LayerRow),
    ]),
    map_ids: &[],
    bundle: None,
};

/// レイヤー行右クリックの並び(`motolii_menubar::context::layer_row_context_items`
/// と一致 — Rename → restack(4項目)→ 可視性/ロック(rail の M/S/L 描画順))。
pub static LAYER_ROW_CONTEXT: &[&Verb] = &[
    &RENAME,
    &BRING_FORWARD,
    &SEND_BACKWARD,
    &BRING_TO_FRONT,
    &SEND_TO_BACK,
    &TOGGLE_HIDE,
    &TOGGLE_SOLO,
    &TOGGLE_LOCK,
];

// ---------------------------------------------------------------------------
// クリップ/キャンバス右クリック(既存動詞の再入口のみ、新規動詞ゼロ)
// ---------------------------------------------------------------------------

/// クリップ右クリックの並び(`motolii_menubar::context::clip_context_items`
/// と一致)。全項目が [`EDIT_MENU`]/[`LAYER_MENU`] の再入口 — 新規動詞ゼロ。
pub static CLIP_CONTEXT: &[&Verb] = &[
    &COPY,
    &PASTE,
    &DUPLICATE,
    &SPLIT,
    &CUT,
    &GROUP,
    &FREEZE,
    &CYCLE_LABEL_COLOR,
];

/// キャンバス右クリックの並び(`motolii_menubar::context::canvas_context_items`
/// と一致)。全項目が [`EDIT_MENU`]/[`LAYER_MENU`] の再入口 — 新規動詞ゼロ。
pub static CANVAS_CONTEXT: &[&Verb] = &[&SELECT_ALL, &DESELECT_ALL, &GROUP];

// ---------------------------------------------------------------------------
// キーフレーム右クリック専用動詞(`motolii_menubar::context::keyframe_context_items`)。
// メニュー実体は shell `menu.rs` の Edit タブ末尾(この crate にはまだ無い —
// `motolii_menubar::menus::edit_menu` doc 冒頭「この crate の edit_menu には
// まだ未統合」)。[`Entry::ExternalMenu`] で出典を転記する。
// ---------------------------------------------------------------------------

const INTERP_SOURCE: &str = "next/shell/motolii-shell/src/menu.rs:112-142(Edit メニュー末尾、\
    この crate の edit_menu にはまだ未統合)";

pub static INTERP_HOLD: Verb = Verb {
    id: "keyframe.interp_hold",
    label: "Interpolation: Hold",
    shortcut: None,
    entries: s6_checked(&[
        Entry::ExternalMenu(INTERP_SOURCE),
        Entry::Context(ContextSlot::Keyframe),
    ]),
    map_ids: &[],
    bundle: None,
};

pub static INTERP_LINEAR: Verb = Verb {
    id: "keyframe.interp_linear",
    label: "Interpolation: Linear",
    shortcut: None,
    entries: s6_checked(&[
        Entry::ExternalMenu(INTERP_SOURCE),
        Entry::Context(ContextSlot::Keyframe),
    ]),
    map_ids: &[],
    bundle: None,
};

pub static INTERP_EASY_EASE: Verb = Verb {
    id: "keyframe.interp_easy_ease",
    label: "Interpolation: Easy Ease",
    shortcut: None,
    entries: s6_checked(&[
        Entry::ExternalMenu(INTERP_SOURCE),
        Entry::Context(ContextSlot::Keyframe),
    ]),
    map_ids: &[],
    bundle: None,
};

pub static INTERP_EASY_EASE_IN: Verb = Verb {
    id: "keyframe.interp_easy_ease_in",
    label: "Interpolation: Easy Ease In",
    shortcut: None,
    entries: s6_checked(&[
        Entry::ExternalMenu(INTERP_SOURCE),
        Entry::Context(ContextSlot::Keyframe),
    ]),
    map_ids: &[],
    bundle: None,
};

pub static INTERP_EASY_EASE_OUT: Verb = Verb {
    id: "keyframe.interp_easy_ease_out",
    label: "Interpolation: Easy Ease Out",
    shortcut: None,
    entries: s6_checked(&[
        Entry::ExternalMenu(INTERP_SOURCE),
        Entry::Context(ContextSlot::Keyframe),
    ]),
    map_ids: &[],
    bundle: None,
};

/// `timeline_pane::Message::DeleteSelectedKeys`。shortcut 出典:
/// `next/shell/motolii-shell/src/lib.rs:5146-5152`。
pub static DELETE_KEYS: Verb = Verb {
    id: "keyframe.delete",
    label: "Delete",
    shortcut: Some("Backspace"),
    entries: s6_checked(&[
        Entry::ShortcutOnly("next/shell/motolii-shell/src/lib.rs:5146-5152"),
        Entry::Context(ContextSlot::Keyframe),
    ]),
    map_ids: &[],
    bundle: None,
};

/// キーフレーム右クリックの並び(`motolii_menubar::context::keyframe_context_items`
/// と一致)。
pub static KEYFRAME_CONTEXT: &[&Verb] = &[
    &INTERP_HOLD,
    &INTERP_LINEAR,
    &INTERP_EASY_EASE,
    &INTERP_EASY_EASE_IN,
    &INTERP_EASY_EASE_OUT,
    &DELETE_KEYS,
];

// ---------------------------------------------------------------------------
// 全動詞(重複無し・38個)。S6 監査/map id 重複検査はこのスライスに対して行う
// (`tests/registry_invariants.rs`)。
// ---------------------------------------------------------------------------

pub static ALL_VERBS: &[&Verb] = &[
    &UNDO,
    &REDO,
    &CUT,
    &COPY,
    &PASTE,
    &DUPLICATE,
    &SPLIT,
    &ADD_MARKER,
    &SELECT_ALL,
    &DESELECT_ALL,
    &NEW_LAYER,
    &GROUP,
    &UNGROUP,
    &FREEZE,
    &UNFREEZE,
    &TOGGLE_HIDE,
    &TOGGLE_SOLO,
    &TOGGLE_LOCK,
    &CYCLE_LABEL_COLOR,
    &TOGGLE_BROWSER,
    &FOCUS_INSPECTOR,
    &FOCUS_STAGE,
    &FOCUS_TIMELINE,
    &CYCLE_PANEL,
    &CLOSE_PANEL,
    &OPEN_DOCUMENTATION,
    &OPEN_COMMUNITY_FORUM,
    &SEND_FEEDBACK,
    &RENAME,
    &BRING_FORWARD,
    &SEND_BACKWARD,
    &BRING_TO_FRONT,
    &SEND_TO_BACK,
    &INTERP_HOLD,
    &INTERP_LINEAR,
    &INTERP_EASY_EASE,
    &INTERP_EASY_EASE_IN,
    &INTERP_EASY_EASE_OUT,
    &DELETE_KEYS,
];
