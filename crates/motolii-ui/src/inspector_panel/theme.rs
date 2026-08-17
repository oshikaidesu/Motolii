//! Inspector native pane の色・寸法。**全部 `docs/mocks-ui/public/inspector-library.css` の写し。**
//!
//! CSS は `var(--mock-role-*, fallback)` 形で、fallback 値は
//! `docs/mocks-ui/src/tokens/mock-candidates.css` の token 実値と一致することを確認済み
//! (例: `--mock-role-surface-app` = `--mock-candidate-color-neutral-950` = `#141414`)。
//! ここで新しい値を1つも決めない。単位は CSS px = egui point。

#![allow(clippy::unreadable_literal)]

use egui::Color32;

const fn rgb(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

// ---- 色(role token の fallback 値) ----
/// inspector-library.css:8 `--mock-role-surface-app, #141414`
pub(crate) const SURFACE_APP: Color32 = rgb(0x141414);
/// inspector-library.css:26 `--mock-role-surface-panel, #1a1a1a`
pub(crate) const SURFACE_PANEL: Color32 = rgb(0x1a1a1a);
/// inspector-library.css:83 `--mock-role-surface-raised, #222222`
pub(crate) const SURFACE_RAISED: Color32 = rgb(0x222222);
/// inspector-library.css:25 `--mock-role-border-default, #3b3b3b`
pub(crate) const BORDER_DEFAULT: Color32 = rgb(0x3b3b3b);
/// inspector-library.css:497 `--mock-role-border-strong, #686868`
pub(crate) const BORDER_STRONG: Color32 = rgb(0x686868);
/// inspector-library.css:9 `--mock-role-text-primary, #f0f0f0`
pub(crate) const TEXT_PRIMARY: Color32 = rgb(0xf0f0f0);
/// inspector-library.css:150 `--mock-role-text-secondary, #c6c6c6`
pub(crate) const TEXT_SECONDARY: Color32 = rgb(0xc6c6c6);
/// inspector-library.css:47 `--mock-role-text-muted, #929292`
pub(crate) const TEXT_MUTED: Color32 = rgb(0x929292);
/// inspector-library.css:103 `--mock-role-action-active, #d8b574`
pub(crate) const ACTION_ACTIVE: Color32 = rgb(0xd8b574);
/// inspector-library.css:297 `--mock-role-data, #78b5b0`(scalar param の帯色)
pub(crate) const ROLE_DATA: Color32 = rgb(0x78b5b0);
/// inspector-library.css:300 `--mock-role-shape, #aaa0d0`(integer param の帯色)
pub(crate) const ROLE_SHAPE: Color32 = rgb(0xaaa0d0);
/// inspector-library.css:199 `--mock-role-way-plugins, #9f9fcf`(effect 色の既定)
pub(crate) const WAY_PLUGINS: Color32 = rgb(0x9f9fcf);
/// inspector-library.css:42 `--mock-role-way-inspector, #8eb086`(panel accent)
pub(crate) const WAY_INSPECTOR: Color32 = rgb(0x8eb086);
/// inspector-library.css:298 `--mock-role-way-inspector`(vector param の帯色)
pub(crate) const ROLE_VECTOR: Color32 = WAY_INSPECTOR;

// ---- 寸法 ----
/// inspector-library.css:30 `--mock-role-panel-header-height, 29px`
pub(crate) const HEADER_H: f32 = 29.0;
/// inspector-library.css:39-41 `.panelHeader::before { width: 3px; height: 13px; }`
pub(crate) const HEADER_ACCENT_W: f32 = 3.0;
pub(crate) const HEADER_ACCENT_H: f32 = 13.0;
/// inspector-library.css:35 `padding: 0 var(--mock-role-inset-control, 9px)`
pub(crate) const HEADER_PAD_X: f32 = 9.0;
/// inspector-library.css:50 `--mock-role-tab-height, 28px`
pub(crate) const TABS_H: f32 = 28.0;
/// inspector-library.css:77 `.selectionSummary { height: 46px; }`
pub(crate) const SUMMARY_H: f32 = 46.0;
/// inspector-library.css:81 `.selectionSummary { padding: 6px 10px; }`
pub(crate) const SUMMARY_PAD_X: f32 = 10.0;
/// inspector-library.css:87-88 `.shapeGlyph { width: 27px; height: 20px; }`
pub(crate) const GLYPH_W: f32 = 27.0;
pub(crate) const GLYPH_H: f32 = 20.0;
/// inspector-library.css:99 `.layerStateButton { width: 22px; height: 21px; }`
pub(crate) const LAYER_STATE_W: f32 = 22.0;
pub(crate) const LAYER_STATE_H: f32 = 21.0;
/// inspector-library.css:126 `.columnHeader { height: 21px; }`
pub(crate) const COLUMN_HEADER_H: f32 = 21.0;
/// inspector-library.css:142 `.tableSection h2 { height: 23px; }`
pub(crate) const SECTION_H: f32 = 23.0;
/// inspector-library.css:285 `.propertyRow { min-height: 25px; }`
pub(crate) const ROW_H: f32 = 25.0;
/// inspector-library.css:291 `.propertyRow::before { width: 3px; }`
pub(crate) const ROW_BAR_W: f32 = 3.0;
/// inspector-library.css:118 `grid-template-columns: minmax(132px, 1fr) repeat(3, 64px) 26px`
pub(crate) const NAME_COL_MIN: f32 = 132.0;
pub(crate) const VALUE_COL_W: f32 = 64.0;
pub(crate) const KEY_COL_W: f32 = 26.0;
/// inspector-library.css:313-314 `.propertyName i { width: 15px; height: 15px; }`
pub(crate) const KIND_ICON: f32 = 15.0;
/// inspector-library.css:309 `.propertyName { padding: 0 8px 0 11px; }`
pub(crate) const NAME_PAD_LEFT: f32 = 11.0;
/// inspector-library.css:189 `.effectStackToolbar { min-height: 22px; }`
pub(crate) const FX_TOOLBAR_H: f32 = 22.0;
/// inspector-library.css:207 `.effectBadge { width: 17px; height: 13px; }`
pub(crate) const BADGE_W: f32 = 17.0;
pub(crate) const BADGE_H: f32 = 13.0;
/// inspector-library.css:519 `footer { min-height: 26px; }`
pub(crate) const FOOTER_H: f32 = 26.0;
/// inspector-library.css:520 `.statusDot { width: 5px; height: 5px; }`
pub(crate) const STATUS_DOT: f32 = 5.0;

// ---- 文字サイズ(CSS px) ----
/// inspector-library.css:46 `.panelHeader strong { font-size: 11px; }`
pub(crate) const FS_TITLE: f32 = 11.0;
/// inspector-library.css:47 `.panelHeader span { font-size: 9px; }`
pub(crate) const FS_HEADER_SPAN: f32 = 9.0;
/// inspector-library.css:62 `.modeTabs button { font-size: 9px; }`
pub(crate) const FS_TAB: f32 = 9.0;
/// inspector-library.css:96 `.selectionSummary strong { font-size: 11px; }`
pub(crate) const FS_SUMMARY: f32 = 11.0;
/// inspector-library.css:97 `.selectionSummary span { font-size: 9px; }`
pub(crate) const FS_SUMMARY_SPAN: f32 = 9.0;
/// inspector-library.css:130 `.columnHeader { font-size: 8px; }`
pub(crate) const FS_COLUMN: f32 = 8.0;
/// inspector-library.css:151 `.tableSection h2 { font-size: 8px; }`
pub(crate) const FS_SECTION: f32 = 8.0;
/// inspector-library.css:360 `.propertyName span { font-size: 10px; }`
pub(crate) const FS_ROW_NAME: f32 = 10.0;
/// inspector-library.css:367 `.propertyName.effectName span { font-size: 9px; }`
pub(crate) const FS_FX_ROW_NAME: f32 = 9.0;
/// inspector-library.css:404 `.valueCell input { font-size: 9px; }`(mono)
pub(crate) const FS_VALUE: f32 = 9.0;
/// inspector-library.css:452 `.keyButton { font-size: 10px; }`(mono)
pub(crate) const FS_KEY: f32 = 10.0;
/// inspector-library.css:215 `.effectSource { font-size: 8px; }`
pub(crate) const FS_FX_SOURCE: f32 = 8.0;
/// inspector-library.css:207 `.effectBadge { font-size: 7px; }`
pub(crate) const FS_BADGE: f32 = 7.0;
/// inspector-library.css:519 `footer { font-size: 9px; }`
pub(crate) const FS_FOOTER: f32 = 9.0;

/// CSS `color-mix(in srgb, A p%, B (100-p)%)` の写し。
/// inspector-library.css は透過側も `transparent` 相当を混ぜるが、
/// ここでは合成先が単色背景なので同じ見た目に落ちる sRGB 線形補間で写す。
pub(crate) fn mix(a: Color32, pct_a: f32, b: Color32) -> Color32 {
    let t = pct_a.clamp(0.0, 100.0) / 100.0;
    let ch = |x: u8, y: u8| ((x as f32) * t + (y as f32) * (1.0 - t)).round() as u8;
    Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
}
