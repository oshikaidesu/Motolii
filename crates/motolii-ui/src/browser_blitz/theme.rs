//! Browserパネルが使う色と寸法。**ここで値を決めない。**
//!
//! 色は `crates/motolii-ui/src/timeline_egui/theme.rs` からの写し。
//! theme.rs の定数は `pub(super)` で timeline_egui の外から参照できないため、
//! 型変換ではなく同じリテラルを出所付きで写す。
//! 寸法は `spikes/blitz-probe/src/bin/browser_panel.rs`(実証済み)からの写し。

/// timeline_egui/theme.rs:8 `DESKTOP` = rgb(0x2a,0x2a,0x2a)
pub(super) const DESKTOP: &str = "#2a2a2a";
/// timeline_egui/theme.rs:9 `SURFACE` = rgb(0x36,0x36,0x36)
pub(super) const SURFACE: &str = "#363636";
/// timeline_egui/theme.rs:10 `SURFACE_HI` = rgb(0x46,0x46,0x46)
pub(super) const SURFACE_HI: &str = "#464646";
/// timeline_egui/theme.rs:11 `SURFACE_LO` = rgb(0x24,0x24,0x24)
pub(super) const SURFACE_LO: &str = "#242424";
/// timeline_egui/theme.rs:12 `CONTRAST` = rgb(0x11,0x11,0x11)
pub(super) const CONTRAST: &str = "#111111";
/// timeline_egui/theme.rs:14 `RULER` = rgb(0x91,0x91,0x91)
pub(super) const RULER: &str = "#919191";
/// timeline_egui/theme.rs:15 `ACCENT` = rgb(0xff,0xad,0x56)
pub(super) const ACCENT: &str = "#ffad56";
/// timeline_egui/theme.rs:16 `INK` = rgb(0xd6,0xd6,0xd6)
pub(super) const INK: &str = "#d6d6d6";
/// timeline_egui/theme.rs:23 `PALETTE[4]` = rgb(0xd6,0x9a,0x8b)
pub(super) const PALETTE_4: &str = "#d69a8b";

/// browser_panel.rs:34 `CELL_W`
pub(super) const CELL_W: f64 = 132.0;
/// browser_panel.rs:35 `CELL_H`
pub(super) const CELL_H: f64 = 108.0;
/// browser_panel.rs:36 `COLS`
pub(super) const COLS: usize = 6;
/// browser_panel.rs:37 `PAD`
pub(super) const PAD: f64 = 10.0;
/// browser_panel.rs:38 `TOP`(ヘッダ下端 = 格子の原点)
pub(super) const TOP: f64 = 34.0;
/// browser_panel.rs:105 ヘッダ帯の高さ
pub(super) const HEADER_H: f64 = 24.0;
