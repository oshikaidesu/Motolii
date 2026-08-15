//! Timeline席の色と帯の高さ。描画側で生の色定数を増やさないため。

use egui::Color32;

pub(super) const OVERVIEW_H: f32 = 22.0;
pub(super) const RULER_H: f32 = 20.0;
pub(super) const LOCATOR_H: f32 = 16.0;
pub(super) const DESKTOP: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub(super) const SURFACE: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
pub(super) const SURFACE_HI: Color32 = Color32::from_rgb(0x46, 0x46, 0x46);
pub(super) const SURFACE_LO: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
pub(super) const CONTRAST: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
pub(super) const DIM: Color32 = Color32::from_rgb(0x75, 0x75, 0x75);
pub(super) const RULER: Color32 = Color32::from_rgb(0x91, 0x91, 0x91);
pub(super) const ACCENT: Color32 = Color32::from_rgb(0xff, 0xad, 0x56);
pub(super) const INK: Color32 = Color32::from_rgb(0xd6, 0xd6, 0xd6);
pub(super) const BAR_INK: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
pub(super) const PALETTE: [Color32; 6] = [
    Color32::from_rgb(0x96, 0xaa, 0xdb),
    Color32::from_rgb(0x6f, 0xb9, 0xc1),
    Color32::from_rgb(0xbf, 0xa9, 0x73),
    Color32::from_rgb(0x89, 0xb9, 0x92),
    Color32::from_rgb(0xd6, 0x9a, 0x8b),
    Color32::from_rgb(0xc3, 0x9b, 0xc5),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineTheme;

impl Default for TimelineTheme {
    fn default() -> Self {
        Self
    }
}
