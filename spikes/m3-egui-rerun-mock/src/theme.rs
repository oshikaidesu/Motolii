use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontTweak, Stroke};

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
pub const WAY_TIMELINE: Color32 = Color32::from_rgb(204, 149, 135);
pub const OBJECT_AUDIO: Color32 = Color32::from_rgb(118, 170, 166);
pub const OBJECT_GROUP: Color32 = Color32::from_rgb(154, 154, 192);
pub const OBJECT_TITLE: Color32 = Color32::from_rgb(177, 155, 120);
pub const OBJECT_CHILD: Color32 = Color32::from_rgb(141, 167, 135);
pub const OBJECT_VIDEO_A: Color32 = Color32::from_rgb(190, 147, 136);
pub const OBJECT_VIDEO_B: Color32 = Color32::from_rgb(133, 162, 192);

pub fn display_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("motolii-display-family".into()))
}

pub fn interface_bold_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, FontFamily::Name("motolii-bold-family".into()))
}

pub fn install(context: &egui::Context) {
    install_mock_fonts(context);
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

fn install_mock_fonts(context: &egui::Context) {
    const CJK_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];
    let mut fonts = FontDefinitions::default();

    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/SFNS.ttf") {
        fonts.font_data.insert(
            "motolii-interface".into(),
            FontData::from_owned(bytes)
                .tweak(FontTweak {
                    scale: 1.18,
                    y_offset_factor: 0.02,
                    hinting: Some(true),
                    ..Default::default()
                })
                .into(),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "motolii-interface".into());
    }
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/SFNSMono.ttf") {
        fonts.font_data.insert(
            "motolii-technical".into(),
            FontData::from_owned(bytes)
                .tweak(FontTweak {
                    scale: 1.14,
                    y_offset_factor: 0.02,
                    hinting: Some(true),
                    ..Default::default()
                })
                .into(),
        );
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "motolii-technical".into());
    }
    if let Some(bytes) = CJK_CANDIDATES
        .iter()
        .find_map(|path| std::fs::read(path).ok())
    {
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
    }
    let display_family = FontFamily::Name("motolii-display-family".into());
    let mut display_fonts = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/Supplemental/Arial Black.ttf") {
        fonts.font_data.insert(
            "motolii-display".into(),
            FontData::from_owned(bytes)
                .tweak(FontTweak {
                    hinting: Some(true),
                    ..Default::default()
                })
                .into(),
        );
        display_fonts.insert(0, "motolii-display".into());
    }
    fonts.families.insert(display_family, display_fonts);
    let bold_family = FontFamily::Name("motolii-bold-family".into());
    let mut bold_fonts = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    if let Ok(bytes) = std::fs::read("/System/Library/Fonts/Supplemental/Arial Bold.ttf") {
        fonts.font_data.insert(
            "motolii-bold".into(),
            FontData::from_owned(bytes)
                .tweak(FontTweak {
                    hinting: Some(true),
                    ..Default::default()
                })
                .into(),
        );
        bold_fonts.insert(0, "motolii-bold".into());
    }
    fonts.families.insert(bold_family, bold_fonts);
    context.set_fonts(fonts);
}
