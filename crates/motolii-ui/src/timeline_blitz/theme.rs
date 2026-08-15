//! `timeline_egui/theme.rs` の**写し**。値をここで決めないため、名前と並びを1対1で保つ。
//!
//! 色はCSSへ出すので `Color32` ではなく16進文字列で持つ。
//! 値が一致していることは `tests::mirrors_timeline_egui_theme` が
//! `timeline_egui/theme.rs` の原文に対して機械照合する。

// timeline_egui/theme.rs:5
pub const OVERVIEW_H: f64 = 22.0;
// timeline_egui/theme.rs:6
pub const RULER_H: f64 = 20.0;
// timeline_egui/theme.rs:7
pub const LOCATOR_H: f64 = 16.0;

/// 行高。**固定・最小 20px**(2026-08-08 Timeline設計決定(3))。
/// 「コンポジットのタイムラインは縦が情報を持たない」ため可変にしない。
/// Ableton由来の可変トラック高／Optimize Height は同決定で不要とされている。
pub const ROW_H: f64 = 20.0;
// timeline_egui/theme.rs:8
pub const DESKTOP: &str = "#2a2a2a";
// timeline_egui/theme.rs:9
pub const SURFACE: &str = "#363636";
// timeline_egui/theme.rs:10
pub const SURFACE_HI: &str = "#464646";
// timeline_egui/theme.rs:11
pub const SURFACE_LO: &str = "#242424";
// timeline_egui/theme.rs:12
pub const CONTRAST: &str = "#111111";
// timeline_egui/theme.rs:13
pub const DIM: &str = "#757575";
// timeline_egui/theme.rs:14
pub const RULER: &str = "#919191";
// timeline_egui/theme.rs:15
pub const ACCENT: &str = "#ffad56";
// timeline_egui/theme.rs:16
pub const INK: &str = "#d6d6d6";
// timeline_egui/theme.rs:17
pub const BAR_INK: &str = "#141414";
// timeline_egui/theme.rs:18-25
pub const PALETTE: [&str; 6] = [
    "#96aadb", "#6fb9c1", "#bfa973", "#89b992", "#d69a8b", "#c39bc5",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// 意味の出所である `timeline_egui/theme.rs` の原文と突き合わせる。
    /// 写し間違いと、向こうが変わってこちらが取り残された事故の両方をここで落とす。
    const EGUI_THEME: &str = include_str!("../timeline_egui/theme.rs");

    fn from_rgb_literal(hex: &str) -> String {
        let bytes = hex.trim_start_matches('#');
        format!(
            "Color32::from_rgb(0x{}, 0x{}, 0x{})",
            &bytes[0..2],
            &bytes[2..4],
            &bytes[4..6]
        )
    }

    #[test]
    fn mirrors_timeline_egui_theme() {
        for (name, hex) in [
            ("DESKTOP", DESKTOP),
            ("SURFACE", SURFACE),
            ("SURFACE_HI", SURFACE_HI),
            ("SURFACE_LO", SURFACE_LO),
            ("CONTRAST", CONTRAST),
            ("DIM", DIM),
            ("RULER", RULER),
            ("ACCENT", ACCENT),
            ("INK", INK),
            ("BAR_INK", BAR_INK),
        ] {
            let expected = format!("pub(super) const {name}: Color32 = {};", from_rgb_literal(hex));
            assert!(
                EGUI_THEME.contains(&expected),
                "timeline_egui/theme.rs に {expected} が無い"
            );
        }

        for hex in PALETTE {
            let expected = from_rgb_literal(hex);
            assert!(
                EGUI_THEME.contains(&expected),
                "timeline_egui/theme.rs の PALETTE に {expected} が無い"
            );
        }

        for (name, value) in [
            ("OVERVIEW_H", OVERVIEW_H),
            ("RULER_H", RULER_H),
            ("LOCATOR_H", LOCATOR_H),
        ] {
            let expected = format!("pub(super) const {name}: f32 = {value:.1};");
            assert!(
                EGUI_THEME.contains(&expected),
                "timeline_egui/theme.rs に {expected} が無い"
            );
        }
    }
}
