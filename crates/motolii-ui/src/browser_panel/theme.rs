//! Browser native pane の色・寸法。**全部 `docs/mocks-ui/public/browser-library.css` の写し。**
//! ここで新しい値を1つも決めない。単位は CSS px = egui point。

#![allow(clippy::unreadable_literal)]

use egui::Color32;

const fn rgb(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

// ---- 面 ----
/// browser-library.css:25 `.libraryBrowser { background: #202225 }`
pub(crate) const PANEL_BG: Color32 = rgb(0x202225);
/// browser-library.css:11 `body { color: #e5e6e2 }`
pub(crate) const TEXT: Color32 = rgb(0xe5e6e2);
/// browser-library.css:34 `.browserHeader { border-bottom: 1px solid #3a3c40 }`
pub(crate) const BORDER: Color32 = rgb(0x3a3c40);
/// browser-library.css:47 `.browserToolbar { border-bottom: 1px solid #383b3e }`
pub(crate) const TOOLBAR_BORDER: Color32 = rgb(0x383b3e);

// ---- header ----
/// browser-library.css:29 `.browserHeader { height: 26px }`
pub(crate) const HEADER_H: f32 = 26.0;
/// browser-library.css:33 `padding: 0 8px`
pub(crate) const HEADER_PAD_X: f32 = 8.0;
/// browser-library.css:37 `.browserHeader strong { font-size: 10px }`
pub(crate) const FS_HEADER: f32 = 10.0;
/// browser-library.css:38 `.browserHeader span { color: #7d8284; font-size: 6px }`
pub(crate) const HEADER_SPAN: Color32 = rgb(0x7d8284);
pub(crate) const FS_HEADER_SPAN: f32 = 6.0;

// ---- toolbar ----
/// browser-library.css:41 `.browserToolbar { height: 30px }`
pub(crate) const TOOLBAR_H: f32 = 30.0;
/// browser-library.css:45-46 `gap: 3px; padding: 3px 5px`
pub(crate) const TOOLBAR_GAP: f32 = 3.0;
pub(crate) const TOOLBAR_PAD_X: f32 = 5.0;
/// browser-library.css:53-57 共通 button: h21 / border #45494d / bg #1a1c1f / #bec1bf
pub(crate) const BUTTON_H: f32 = 21.0;
pub(crate) const BUTTON_BORDER: Color32 = rgb(0x45494d);
pub(crate) const BUTTON_BG: Color32 = rgb(0x1a1c1f);
pub(crate) const BUTTON_FG: Color32 = rgb(0xbec1bf);
/// browser-library.css:60 `.historyButton { width: 18px; color: #8b8e8d }`
pub(crate) const HISTORY_W: f32 = 18.0;
pub(crate) const HISTORY_FG: Color32 = rgb(0x8b8e8d);
/// browser-library.css:62 `.toolbarButton { font-size: 7px }`
pub(crate) const FS_TOOLBAR: f32 = 7.0;
/// browser-library.css:64-77 `#library-search`
pub(crate) const SEARCH_BG: Color32 = rgb(0x15171a);
pub(crate) const SEARCH_BORDER: Color32 = rgb(0x474b50);
pub(crate) const SEARCH_FG: Color32 = rgb(0xeeefeb);
pub(crate) const SEARCH_PLACEHOLDER: Color32 = rgb(0x74797f);
pub(crate) const FS_SEARCH: f32 = 8.0;

// ---- tabs ----
/// browser-library.css:80 `.libraryTabs { height: 26px }`
pub(crate) const TABS_H: f32 = 26.0;
/// browser-library.css:92-94 `color: #afb1b0; font-size: 8px`
pub(crate) const TAB_FG: Color32 = rgb(0xafb1b0);
pub(crate) const FS_TAB: f32 = 8.0;
/// browser-library.css:97 selected: 下線 #b9a660 / bg #191b1e / #f0f0ec
pub(crate) const ACCENT: Color32 = rgb(0xb9a660);
pub(crate) const TAB_SELECTED_BG: Color32 = rgb(0x191b1e);
pub(crate) const TAB_SELECTED_FG: Color32 = rgb(0xf0f0ec);

// ---- sidebar ----
/// browser-library.css:102-103 `.librarySidebar { width: 112px }`
pub(crate) const SIDEBAR_W: f32 = 112.0;
/// browser-library.css:107 `background: #181a1d`
pub(crate) const SIDEBAR_BG: Color32 = rgb(0x181a1d);
/// browser-library.css:111-118 h2: h16 / #727679 / 6px
pub(crate) const SIDEBAR_H2_H: f32 = 16.0;
pub(crate) const SIDEBAR_H2_FG: Color32 = rgb(0x727679);
pub(crate) const FS_SIDEBAR_H2: f32 = 6.0;
/// browser-library.css:126-141 `.locationRow`: h19 / pad-left 7 / #b9bcba / 8px
pub(crate) const ROW_H: f32 = 19.0;
pub(crate) const ROW_PAD_X: f32 = 7.0;
pub(crate) const ROW_FG: Color32 = rgb(0xb9bcba);
pub(crate) const FS_ROW: f32 = 8.0;
/// browser-library.css:143 `.locationRow.indent { padding-left: 13px }`
pub(crate) const ROW_INDENT: f32 = 13.0;
/// browser-library.css:144 selected: 左帯 #b9a660 / bg #353738 / #f1f1ed
pub(crate) const ROW_SELECTED_BG: Color32 = rgb(0x353738);
pub(crate) const ROW_SELECTED_FG: Color32 = rgb(0xf1f1ed);
/// browser-library.css:145 hover bg #292c2f
pub(crate) const ROW_HOVER_BG: Color32 = rgb(0x292c2f);
/// browser-library.css:146 `.addFolder { color: #767a7c }`
pub(crate) const ROW_DISABLED_FG: Color32 = rgb(0x767a7c);

// ---- catalog header ----
/// browser-library.css:156 `.catalogHeader { height: 31px }`
pub(crate) const CATALOG_HEADER_H: f32 = 31.0;
/// browser-library.css:161 `border-bottom: 1px solid #373a3d`
pub(crate) const CATALOG_HEADER_BORDER: Color32 = rgb(0x373a3d);
/// browser-library.css:164 strong: #e2e3df / 9px
pub(crate) const CATALOG_TITLE_FG: Color32 = rgb(0xe2e3df);
pub(crate) const FS_CATALOG_TITLE: f32 = 9.0;
/// browser-library.css:165 span: #808487 / 6px
pub(crate) const CATALOG_PATH_FG: Color32 = rgb(0x808487);
pub(crate) const FS_CATALOG_PATH: f32 = 6.0;
/// browser-library.css:167 `.viewModes button { width: 21px }`
pub(crate) const VIEW_BUTTON_W: f32 = 21.0;
/// browser-library.css:168 pressed: border #b9a660 / bg #38362c / #f0ebce
pub(crate) const VIEW_PRESSED_BG: Color32 = rgb(0x38362c);
pub(crate) const VIEW_PRESSED_FG: Color32 = rgb(0xf0ebce);

// ---- filter shelf ----
/// browser-library.css:171-179 shelf: min-h24 / pad 3 5 / border #34373a / bg #1b1d20
pub(crate) const SHELF_H: f32 = 24.0;
pub(crate) const SHELF_BORDER: Color32 = rgb(0x34373a);
pub(crate) const SHELF_BG: Color32 = rgb(0x1b1d20);
/// browser-library.css:187 label: #777c7e / 6px
pub(crate) const SHELF_LABEL_FG: Color32 = rgb(0x777c7e);
pub(crate) const FS_SHELF_LABEL: f32 = 6.0;
/// browser-library.css:188-199 chip: min-h17 / border #484b4e / radius 8 / bg #26292c / #c9ccca / 7px
pub(crate) const CHIP_H: f32 = 17.0;
pub(crate) const CHIP_BORDER: Color32 = rgb(0x484b4e);
pub(crate) const CHIP_RADIUS: f32 = 8.0;
pub(crate) const CHIP_BG: Color32 = rgb(0x26292c);
pub(crate) const CHIP_FG: Color32 = rgb(0xc9ccca);
pub(crate) const FS_CHIP: f32 = 7.0;
/// browser-library.css:201 selected chip: border #b9a660 / bg #3c382b / #f1e8bd
pub(crate) const CHIP_SELECTED_BG: Color32 = rgb(0x3c382b);
pub(crate) const CHIP_SELECTED_FG: Color32 = rgb(0xf1e8bd);
/// browser-library.css:202 `.clearFilter { color: #9b9e9c }`
pub(crate) const CHIP_CLEAR_FG: Color32 = rgb(0x9b9e9c);

// ---- result summary ----
/// browser-library.css:205 `.resultSummary { height: 21px }`
pub(crate) const SUMMARY_H: f32 = 21.0;
/// browser-library.css:212 strong 8px
pub(crate) const FS_SUMMARY: f32 = 8.0;
/// browser-library.css:213 count: #898d8e / 7px
pub(crate) const SUMMARY_COUNT_FG: Color32 = rgb(0x898d8e);
pub(crate) const FS_SUMMARY_COUNT: f32 = 7.0;

// ---- cards ----
/// browser-library.css:227 `.libraryCard { padding: 3px }`
pub(crate) const CARD_PAD: f32 = 3.0;
/// browser-library.css:237 selected card bg #36393a
pub(crate) const CARD_SELECTED_BG: Color32 = rgb(0x36393a);
/// browser-library.css:246 thumb border #3e4245
pub(crate) const THUMB_BORDER: Color32 = rgb(0x3e4245);
/// browser-library.css:249 hover border #85898b
pub(crate) const THUMB_HOVER_BORDER: Color32 = rgb(0x85898b);
/// browser-library.css:251 thumb glyph: #f2f2ed / 8px
pub(crate) const THUMB_GLYPH_FG: Color32 = rgb(0xf2f2ed);
pub(crate) const FS_THUMB_GLYPH: f32 = 8.0;
/// browser-library.css:252 `.thumb-blue`(mock の video 例: starter-clip.mp4)
pub(crate) const THUMB_VIDEO: Color32 = rgb(0x5d7899);
/// browser-library.css:253 `.thumb-purple`(mock の svg 例: starter-mark.svg)
pub(crate) const THUMB_SVG: Color32 = rgb(0x746398);
/// browser-library.css:254 `.thumb-ochre`(mock の image 例: starter-still.png)
pub(crate) const THUMB_IMAGE: Color32 = rgb(0x88704e);
/// browser-library.css:255 `.thumb-green`(mock の audio 例: starter-tone.wav)
pub(crate) const THUMB_AUDIO: Color32 = rgb(0x557f6d);
/// browser-library.css:266 name: #edede9 / 8px
pub(crate) const CARD_NAME_FG: Color32 = rgb(0xedede9);
pub(crate) const FS_CARD_NAME: f32 = 8.0;
/// browser-library.css:267 meta: #9ea2a3 / 6.5px
pub(crate) const CARD_META_FG: Color32 = rgb(0x9ea2a3);
pub(crate) const FS_CARD_META: f32 = 6.5;
/// browser-library.css:275 list view thumb: width 46px
pub(crate) const LIST_THUMB_W: f32 = 46.0;

// ---- selection tray ----
/// browser-library.css:279 `.selectionTray { height: 27px }`
pub(crate) const TRAY_H: f32 = 27.0;
/// browser-library.css:286-287 border-top #3a3d40 / bg #191b1e
pub(crate) const TRAY_BORDER: Color32 = rgb(0x3a3d40);
pub(crate) const TRAY_BG: Color32 = rgb(0x191b1e);
/// browser-library.css:291 dot 5x5 #b9a660(= ACCENT)
pub(crate) const TRAY_DOT: f32 = 5.0;
/// browser-library.css:292 name: #e8e9e4 / 7px
pub(crate) const TRAY_NAME_FG: Color32 = rgb(0xe8e9e4);
pub(crate) const FS_TRAY_NAME: f32 = 7.0;
/// browser-library.css:293 meta: #83888a / 6px
pub(crate) const TRAY_META_FG: Color32 = rgb(0x83888a);
pub(crate) const FS_TRAY_META: f32 = 6.0;

/// 16:9(browser-library.css:241 `.libraryThumb { aspect-ratio: 16 / 9 }`)。
pub(crate) const THUMB_ASPECT: f32 = 16.0 / 9.0;
