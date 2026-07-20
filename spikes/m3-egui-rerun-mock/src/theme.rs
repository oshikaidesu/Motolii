use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Stroke};

pub const APP: Color32 = Color32::from_rgb(20, 20, 20);
pub const PANEL: Color32 = Color32::from_rgb(26, 26, 26);
pub const RAISED: Color32 = Color32::from_rgb(34, 34, 34);
pub const HOVER: Color32 = Color32::from_rgb(44, 44, 44);
pub const BORDER: Color32 = Color32::from_rgb(59, 59, 59);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(104, 104, 104);
pub const TEXT: Color32 = Color32::from_rgb(240, 240, 240);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(198, 198, 198);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(146, 146, 146);
pub const ACCENT: Color32 = Color32::from_rgb(216, 181, 116);
pub const DATA: Color32 = Color32::from_rgb(120, 181, 176);
pub const SHAPE: Color32 = Color32::from_rgb(170, 160, 208);
pub const WARNING: Color32 = Color32::from_rgb(225, 138, 109);
pub const WAY_STAGE: Color32 = Color32::from_rgb(188, 160, 114);
pub const WAY_INSPECTOR: Color32 = Color32::from_rgb(142, 176, 134);

pub fn install(context: &egui::Context) {
    install_cjk_fallback(context);
    context.set_theme(egui::Theme::Dark);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = APP;
    visuals.faint_bg_color = RAISED;
    visuals.code_bg_color = APP;
    visuals.override_text_color = Some(TEXT);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.bg_fill = PANEL;
    visuals.widgets.inactive.weak_bg_fill = PANEL;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.weak_bg_fill = HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    visuals.widgets.active.bg_fill = RAISED;
    visuals.widgets.active.weak_bg_fill = RAISED;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.open.bg_fill = RAISED;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.22);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    context.set_visuals(visuals);

    let mut style = (*context.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(5.0, 4.0);
    style.spacing.button_padding = egui::vec2(7.0, 3.0);
    style.spacing.interact_size.y = 24.0;
    style.spacing.slider_width = 102.0;
    style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(11.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(10.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(9.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(9.0, FontFamily::Monospace),
    );
    context.set_style_of(egui::Theme::Dark, style);
}

fn install_cjk_fallback(context: &egui::Context) {
    const CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];
    let Some(bytes) = CANDIDATES.iter().find_map(|path| std::fs::read(path).ok()) else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("motolii-cjk".into(), FontData::from_owned(bytes).into());
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("motolii-cjk".into());
    }
    context.set_fonts(fonts);
}
