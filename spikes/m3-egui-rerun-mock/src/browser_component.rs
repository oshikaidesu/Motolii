use crate::{components, theme};
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, Vec2,
};
use std::sync::Arc;

const EFFECTS: [Effect; 3] = [
    Effect {
        id: "echo-bloom",
        name: "Echo Bloom",
        category: "Effect",
        subtype: "Light",
        search: "echo bloom light pulse glow effect installed",
        source: Source::Used,
        collection: Some(Collection::Favorites),
        tags: &[Tag::GoTo, Tag::Atmosphere],
        pack: Some(Pack::MotionKitAlpha),
        badge: None,
        has_motion: true,
    },
    Effect {
        id: "type-pulse",
        name: "Type Pulse",
        category: "Effect",
        subtype: "Typography",
        search: "type pulse kinetic text motion effect",
        source: Source::Recent,
        collection: Some(Collection::Type),
        tags: &[Tag::Kinetic],
        pack: Some(Pack::MotionKitAlpha),
        badge: Some("◆ 12 KEYS"),
        has_motion: true,
    },
    Effect {
        id: "fold-field",
        name: "Fold Field",
        category: "Effect",
        subtype: "Spatial",
        search: "fold field space geometry effect incompatible unavailable",
        source: Source::All,
        collection: None,
        tags: &[Tag::Review],
        pack: None,
        badge: Some("Unavailable"),
        has_motion: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultView {
    Visual,
    Thumb,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserTab {
    Media,
    Effects,
    Create,
}

impl BrowserTab {
    fn index(self) -> usize {
        match self {
            Self::Media => 0,
            Self::Effects => 1,
            Self::Create => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    All,
    Used,
    Recent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Collection {
    Favorites,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
    GoTo,
    Atmosphere,
    Kinetic,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pack {
    MotionKitAlpha,
}

#[derive(Debug, Clone, Copy)]
struct Effect {
    id: &'static str,
    name: &'static str,
    category: &'static str,
    subtype: &'static str,
    search: &'static str,
    source: Source,
    collection: Option<Collection>,
    tags: &'static [Tag],
    pack: Option<Pack>,
    badge: Option<&'static str>,
    has_motion: bool,
}

#[derive(Debug)]
pub(crate) struct BrowserState {
    tab: BrowserTab,
    query: String,
    view: ResultView,
    source: Source,
    collection: Option<Collection>,
    tag: Option<Tag>,
    pack: Option<Pack>,
    selected: &'static str,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            tab: BrowserTab::Effects,
            query: String::new(),
            view: ResultView::Thumb,
            source: Source::All,
            collection: None,
            tag: None,
            pack: None,
            selected: "echo-bloom",
        }
    }
}

impl BrowserState {
    pub(crate) fn selected(&self) -> &'static str {
        self.selected
    }
}

pub(crate) enum BrowserAction {
    EffectSelected(&'static str),
    Status(&'static str),
}

pub(crate) fn browser_ui(ui: &mut egui::Ui, state: &mut BrowserState) -> Option<BrowserAction> {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.painter().rect_filled(ui.max_rect(), 0.0, theme::PANEL);

    let mut action = browser_header(ui, state);
    if state.tab != BrowserTab::Effects {
        simple_browser(ui, state);
        return action;
    }
    let mut changed = None;
    search_row(ui, state, "Search");

    let available = ui.available_size();
    ui.allocate_ui_with_layout(available, Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.allocate_ui_with_layout(
            Vec2::new(103.0, available.y),
            Layout::top_down(Align::Min),
            |ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, theme::APP);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::same(3))
                    .show(ui, |ui| source_rail(ui, state));
            },
        );
        vertical_divider(ui, available.y);
        ui.allocate_ui_with_layout(ui.available_size(), Layout::top_down(Align::Min), |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, theme::BORDER);
            changed = results(ui, state);
        });
    });
    if let Some(id) = changed {
        action = Some(BrowserAction::EffectSelected(id));
    }
    action
}

fn browser_header(ui: &mut egui::Ui, state: &mut BrowserState) -> Option<BrowserAction> {
    components::panel_header(ui, "Browser", "MEDIA / CREATE / EFFECTS", theme::SHAPE);
    let clicked = components::tabs(
        ui,
        &["Media", "Effects", "Create"],
        state.tab.index(),
        theme::SHAPE,
    );
    clicked.map(|index| {
        state.tab = match index {
            0 => BrowserTab::Media,
            2 => BrowserTab::Create,
            _ => BrowserTab::Effects,
        };
        state.query.clear();
        state.view = match state.tab {
            BrowserTab::Media => ResultView::Visual,
            BrowserTab::Effects | BrowserTab::Create => ResultView::Thumb,
        };
        match state.tab {
            BrowserTab::Media => BrowserAction::Status("Media browser · 6 items"),
            BrowserTab::Effects => BrowserAction::Status("Effects browser · 3 results"),
            BrowserTab::Create => BrowserAction::Status("Create browser · 8 registered items"),
        }
    })
}

fn simple_browser(ui: &mut egui::Ui, state: &mut BrowserState) {
    let (placeholder, title, scope, items): (&str, &str, &str, &[(&str, &str, &str)]) =
        match state.tab {
            BrowserTab::Media => (
                "Search media",
                "All Media",
                "6 ITEMS",
                &[
                    ("♪", "night_drive.wav", "PROJECT · USED"),
                    ("◇", "logo.svg", "PROJECT · UNPLACED"),
                    ("▧", "grain.png", "PROJECT · USED"),
                    ("▶", "city_loop.mp4", "PROJECT · INBOX"),
                    ("♪", "impact_04.wav", "AUDIO LIBRARY"),
                    ("▧", "paper.png", "BRAND KIT"),
                ],
            ),
            BrowserTab::Create => (
                "Search create items",
                "All Create items",
                "REGISTERED PROVIDERS · 8",
                &[
                    ("□", "Rectangle", "SHAPE · BUILT-IN"),
                    ("○", "Ellipse", "SHAPE · BUILT-IN"),
                    ("T", "Text", "LAYER · BUILT-IN"),
                    ("■", "Solid", "LAYER · BUILT-IN"),
                    ("G", "Glyph Current", "GENERATOR · MOTION KIT"),
                    ("T", "Type Pulse", "TEXT · MOTION KIT"),
                    ("≋", "Ribbon Array", "MISSING · MOTION KIT"),
                    ("✣", "Particle Field", "GENERATOR · ORBIT FORGE"),
                ],
            ),
            BrowserTab::Effects => unreachable!(),
        };

    search_row(ui, state, placeholder);
    let footer_height = 28.0;
    let body_size = Vec2::new(
        ui.available_width(),
        (ui.available_height() - footer_height).max(80.0),
    );
    ui.allocate_ui_with_layout(body_size, Layout::left_to_right(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.allocate_ui_with_layout(
            Vec2::new(103.0, body_size.y),
            Layout::top_down(Align::Min),
            |ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, theme::APP);
                egui::Frame::NONE
                    .inner_margin(egui::Margin::same(3))
                    .show(ui, |ui| simple_source_rail(ui, state.tab));
            },
        );
        vertical_divider(ui, body_size.y);
        ui.allocate_ui_with_layout(ui.available_size(), Layout::top_down(Align::Min), |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, theme::BORDER);
            simple_results(ui, state.tab, title, scope, items, &state.query);
        });
    });
    simple_footer(ui, state.tab, footer_height);
}

fn simple_source_rail(ui: &mut egui::Ui, tab: BrowserTab) {
    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
    let _ = nav_row(
        ui,
        "A",
        if tab == BrowserTab::Media {
            "All Media"
        } else {
            "All"
        },
        true,
        None,
    );
    if tab == BrowserTab::Media {
        let _ = nav_row(ui, "◆", "Project", false, None);
        let _ = nav_row(ui, "↺", "Recent", false, None);
        section_title(ui, "Registered folders");
        for label in ["City Source", "Audio Library", "Brand Kit"] {
            let _ = nav_row(ui, "■", label, false, None);
        }
        let _ = nav_row(ui, "+", "Add folder", false, None);
        section_title(ui, "Collections");
        let _ = nav_row(ui, "◎", "Favorites", false, None);
        let _ = nav_row(ui, "Aa", "Brand", false, None);
        section_title(ui, "Tags");
        for (glyph, label) in [
            ("◎", "Favorite"),
            ("Aa", "Brand"),
            ("✓", "Review"),
            ("~", "Audio"),
        ] {
            let _ = nav_row(ui, glyph, label, false, Some(0));
        }
        let _ = nav_row(ui, "+", "New tag", false, None);
    } else {
        let _ = nav_row(ui, "↺", "Recent", false, None);
        section_title(ui, "Type");
        for (glyph, label) in [("○", "Shapes"), ("□", "Layers"), ("+", "Generators")] {
            let _ = nav_row(ui, glyph, label, false, None);
        }
        section_title(ui, "Provider");
        let _ = nav_row(ui, "M", "Built-in", false, None);
        let _ = nav_row(ui, "O", "Orbit Forge", false, None);
        section_title(ui, "Tags");
        for (glyph, label, count) in [
            ("□", "Layout", 1),
            ("Aa", "Brand kit", 3),
            ("~", "Animated", 3),
            ("◇", "Prototype", 1),
        ] {
            let _ = nav_row(ui, glyph, label, false, Some(count));
        }
    }
    section_title(ui, "Packs");
    let _ = nav_row(ui, "P", "Motion Kit α", false, None);
}

fn simple_results(
    ui: &mut egui::Ui,
    tab: BrowserTab,
    title: &str,
    scope: &str,
    items: &[(&str, &str, &str)],
    query: &str,
) {
    components::Block {
        height: 27.0,
        fill: theme::PANEL,
        border_top: false,
        border_bottom: true,
        inset_x: 5.0,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        let rect = ui.available_rect_before_wrap();
        let scope_galley = ellipsized_galley(
            ui.painter(),
            scope,
            FontId::monospace(7.0),
            theme::TEXT_MUTED,
            rect.width() * 0.62,
        );
        let scope_pos = Pos2::new(
            rect.right() - 5.0 - scope_galley.size().x,
            rect.center().y - scope_galley.size().y * 0.5,
        );
        ui.painter()
            .galley(scope_pos, scope_galley, theme::TEXT_MUTED);

        let title_galley = ellipsized_galley(
            ui.painter(),
            title,
            theme::interface_bold_font(10.0),
            theme::TEXT,
            (scope_pos.x - rect.left() - 5.0).max(1.0),
        );
        ui.painter().galley(
            Pos2::new(rect.left(), rect.center().y - title_galley.size().y * 0.5),
            title_galley,
            theme::TEXT,
        );
        ui.allocate_rect(rect, Sense::hover());
    });

    let query = query.to_ascii_lowercase();
    let visible = items.iter().filter(|(_, name, detail)| {
        query.is_empty()
            || name.to_ascii_lowercase().contains(&query)
            || detail.to_ascii_lowercase().contains(&query)
    });
    let gap = 1.0;
    let width = ((ui.available_width() - gap) * 0.5).max(60.0);
    egui::Grid::new(("simple-browser-grid", tab.index()))
        .spacing([gap, 1.0])
        .show(ui, |ui| {
            for (index, (glyph, name, detail)) in visible.enumerate() {
                simple_result_card(ui, tab, glyph, name, detail, width);
                if index % 2 == 1 {
                    ui.end_row();
                }
            }
        });
}

fn ellipsized_galley(
    painter: &egui::Painter,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<egui::Galley> {
    let full = painter.layout_no_wrap(text.to_owned(), font.clone(), color);
    if full.size().x <= max_width {
        return full;
    }

    let mut prefix = text.to_owned();
    while !prefix.is_empty() {
        prefix.pop();
        let shortened = painter.layout_no_wrap(format!("{prefix}…"), font.clone(), color);
        if shortened.size().x <= max_width {
            return shortened;
        }
    }
    painter.layout_no_wrap("…".to_owned(), font, color)
}

fn simple_result_card(
    ui: &mut egui::Ui,
    tab: BrowserTab,
    glyph: &str,
    name: &str,
    detail: &str,
    width: f32,
) {
    let thumbnail_height = if tab == BrowserTab::Media { 50.0 } else { 58.0 };
    let name_height = if tab == BrowserTab::Media { 0.0 } else { 28.0 };
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, thumbnail_height + name_height),
        Sense::click(),
    );
    let thumb = Rect::from_min_size(rect.min, Vec2::new(width, thumbnail_height));
    ui.painter().rect_filled(thumb, 0.0, theme::APP);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );
    paint_simple_preview(ui.painter(), thumb.shrink(3.0), glyph, name, detail);
    if name_height > 0.0 {
        let name_rect = Rect::from_min_max(Pos2::new(rect.left(), thumb.bottom()), rect.max);
        ui.painter().rect_filled(name_rect, 0.0, theme::PANEL);
        ui.painter().text(
            name_rect.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            name,
            theme::interface_bold_font(9.0),
            theme::TEXT_SECONDARY,
        );
    }
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), name));
    response.on_hover_text(format!("{name} · {detail}"));
}

fn paint_simple_preview(
    painter: &egui::Painter,
    rect: Rect,
    glyph: &str,
    name: &str,
    detail: &str,
) {
    if name.contains("wav") {
        let points = (0..9)
            .map(|index| {
                let x = egui::lerp(rect.x_range(), index as f32 / 8.0);
                let y = rect.center().y + if index % 2 == 0 { 5.0 } else { -5.0 };
                Pos2::new(x, y)
            })
            .collect();
        painter.add(egui::Shape::line(points, Stroke::new(1.0, theme::DATA)));
    } else if name == "Rectangle" || name == "Solid" {
        let square = Rect::from_center_size(rect.center(), Vec2::splat(22.0));
        if name == "Solid" {
            painter.rect_filled(square, 0.0, theme::TEXT);
        } else {
            painter.rect_stroke(
                square,
                0.0,
                Stroke::new(2.0, theme::TEXT),
                StrokeKind::Inside,
            );
        }
    } else if name == "Ellipse" {
        painter.circle_stroke(rect.center(), 17.0, Stroke::new(2.0, theme::TEXT));
    } else if name.contains("mp4") || name.contains("png") {
        for offset in (-40..=40).step_by(8) {
            painter.line_segment(
                [
                    Pos2::new(rect.left(), rect.center().y + offset as f32),
                    Pos2::new(rect.right(), rect.center().y + offset as f32 - rect.width()),
                ],
                Stroke::new(1.0, theme::BORDER),
            );
        }
        if name.contains("mp4") {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "▶ 00:12",
                FontId::monospace(7.0),
                theme::TEXT_SECONDARY,
            );
        }
    } else if name == "Ribbon Array" {
        for row in -1..=1 {
            let points = (0..=18)
                .map(|index| {
                    let t = index as f32 / 18.0;
                    Pos2::new(
                        egui::lerp(rect.left() + 14.0..=rect.right() - 14.0, t),
                        rect.center().y
                            + row as f32 * 6.0
                            + (t * std::f32::consts::TAU).sin() * 2.0,
                    )
                })
                .collect();
            painter.add(egui::Shape::line(points, Stroke::new(1.8, theme::TEXT)));
        }
        painter.text(
            rect.left_top(),
            Align2::LEFT_TOP,
            "MISSING",
            FontId::monospace(6.0),
            theme::WARNING,
        );
    } else if name == "Particle Field" {
        painter.line_segment(
            [
                rect.center() + Vec2::new(-7.0, 0.0),
                rect.center() + Vec2::new(7.0, 0.0),
            ],
            Stroke::new(1.5, theme::TEXT),
        );
        painter.line_segment(
            [
                rect.center() + Vec2::new(0.0, -7.0),
                rect.center() + Vec2::new(0.0, 7.0),
            ],
            Stroke::new(1.5, theme::TEXT),
        );
        for offset in [
            Vec2::new(-10.0, -8.0),
            Vec2::new(10.0, -5.0),
            Vec2::new(-8.0, 9.0),
        ] {
            painter.circle_filled(rect.center() + offset, 2.0, theme::ACCENT);
        }
        painter.circle_filled(rect.center(), 3.0, theme::TEXT);
    } else {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            theme::interface_bold_font(if glyph.len() == 1 { 26.0 } else { 16.0 }),
            if detail.contains("MISSING") {
                theme::WARNING
            } else {
                theme::TEXT
            },
        );
    }
}

fn simple_footer(ui: &mut egui::Ui, tab: BrowserTab, height: f32) {
    components::Block {
        height,
        fill: theme::APP,
        border_top: true,
        border_bottom: false,
        inset_x: 5.0,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        ui.label(
            RichText::new(if tab == BrowserTab::Media {
                "1 selected"
            } else {
                "8 registered · 3 providers · D&D or double-click"
            })
            .monospace()
            .size(7.0)
            .color(theme::TEXT_MUTED),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(4.0);
            let _ = ui.add_sized(
                [42.0, 22.0],
                egui::Button::new(if tab == BrowserTab::Media {
                    "Select"
                } else {
                    "Save…"
                }),
            );
        });
    });
}

fn vertical_divider(ui: &mut egui::Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::BORDER);
}

fn search_row(ui: &mut egui::Ui, state: &mut BrowserState, placeholder: &str) {
    components::Block {
        height: 36.0,
        fill: theme::APP,
        border_top: false,
        border_bottom: true,
        inset_x: 5.0,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        let field_width = (ui.available_width() - 89.0).max(52.0);
        ui.add_sized(
            [field_width, 27.0],
            egui::TextEdit::singleline(&mut state.query)
                .hint_text(placeholder)
                .desired_width(field_width),
        );
        view_button(ui, state, ResultView::Visual, "S", "Thumbnail-only view");
        view_button(ui, state, ResultView::Thumb, "G", "Thumbnail and name view");
        view_button(ui, state, ResultView::Detail, "L", "List view");
        ui.add_space(2.0);
    });
}

fn view_button(
    ui: &mut egui::Ui,
    state: &mut BrowserState,
    view: ResultView,
    glyph: &str,
    hint: &str,
) {
    let selected = state.view == view;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::splat(components::TOKENS.control_height),
        Sense::click(),
    );
    ui.painter().rect_filled(
        rect,
        0.0,
        if selected {
            theme::RAISED
        } else {
            theme::PANEL
        },
    );
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(
            1.0,
            if selected {
                theme::SHAPE
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    paint_view_icon(ui.painter(), rect, view, selected);
    if response.on_hover_text(hint).clicked() {
        state.view = view;
    }
    let _ = glyph;
}

fn paint_view_icon(painter: &egui::Painter, rect: Rect, view: ResultView, selected: bool) {
    let color = if selected {
        theme::TEXT
    } else {
        theme::TEXT_MUTED
    };
    match view {
        ResultView::Visual => {
            painter.rect_filled(
                Rect::from_center_size(rect.center(), Vec2::splat(4.0)),
                0.0,
                color,
            );
        }
        ResultView::Thumb => {
            painter.rect_filled(
                Rect::from_center_size(rect.center(), Vec2::splat(6.0)),
                0.0,
                color,
            );
        }
        ResultView::Detail => {
            for offset in [-3.0, 0.0, 3.0] {
                painter.line_segment(
                    [
                        Pos2::new(rect.center().x - 4.0, rect.center().y + offset),
                        Pos2::new(rect.center().x + 4.0, rect.center().y + offset),
                    ],
                    Stroke::new(1.0, color),
                );
            }
        }
    }
}

fn source_rail(ui: &mut egui::Ui, state: &mut BrowserState) {
    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);

    for (source, glyph, label) in [
        (Source::All, "A", "All"),
        (Source::Used, "◇", "Used"),
        (Source::Recent, "↺", "Recent"),
    ] {
        if nav_row(ui, glyph, label, state.source == source, None).clicked() {
            state.source = source;
            state.collection = None;
            state.pack = None;
        }
    }

    section_title(ui, "Collections");
    for (collection, glyph, label) in [
        (Collection::Favorites, "◎", "Favorites"),
        (Collection::Type, "Aa", "Type"),
    ] {
        let selected = state.collection == Some(collection);
        if nav_row(ui, glyph, label, selected, None).clicked() {
            state.collection = if selected { None } else { Some(collection) };
            state.pack = None;
        }
    }

    section_title(ui, "Tags");
    for (tag, glyph, label) in [
        (Tag::GoTo, "◎", "Go-to"),
        (Tag::Atmosphere, "@", "Atmosphere"),
        (Tag::Kinetic, "~", "Kinetic"),
        (Tag::Review, "✓", "Review"),
    ] {
        let selected = state.tag == Some(tag);
        if nav_row(ui, glyph, label, selected, Some(tag_count(tag))).clicked() {
            state.tag = if selected { None } else { Some(tag) };
        }
    }

    section_title(ui, "Packs");
    let selected = state.pack == Some(Pack::MotionKitAlpha);
    if nav_row(ui, "P", "Motion Kit α", selected, None).clicked() {
        state.pack = if selected {
            None
        } else {
            Some(Pack::MotionKitAlpha)
        };
        state.collection = None;
    }

    let (save, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 22.0), Sense::click());
    paint_dashed_rect(ui.painter(), save, theme::BORDER_STRONG);
    ui.painter().text(
        save.center(),
        Align2::CENTER_CENTER,
        "+ Save current…",
        FontId::proportional(9.0),
        theme::TEXT_SECONDARY,
    );
    response.on_hover_text("Save current effect");
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.add_space(3.0);
    ui.label(
        RichText::new(title.to_ascii_uppercase())
            .monospace()
            .size(8.0)
            .color(theme::TEXT_MUTED),
    );
    ui.add_space(1.0);
}

fn paint_dashed_rect(painter: &egui::Painter, rect: Rect, color: Color32) {
    let dash = 3.0;
    let gap = 2.0;
    let mut x = rect.left();
    while x < rect.right() {
        let end = (x + dash).min(rect.right());
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(end, rect.top())],
            Stroke::new(1.0, color),
        );
        painter.line_segment(
            [Pos2::new(x, rect.bottom()), Pos2::new(end, rect.bottom())],
            Stroke::new(1.0, color),
        );
        x += dash + gap;
    }
    let mut y = rect.top();
    while y < rect.bottom() {
        let end = (y + dash).min(rect.bottom());
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.left(), end)],
            Stroke::new(1.0, color),
        );
        painter.line_segment(
            [Pos2::new(rect.right(), y), Pos2::new(rect.right(), end)],
            Stroke::new(1.0, color),
        );
        y += dash + gap;
    }
}

fn nav_row(
    ui: &mut egui::Ui,
    glyph: &str,
    label: &str,
    selected: bool,
    count: Option<usize>,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 23.0), Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            0.0,
            if selected {
                theme::RAISED
            } else {
                theme::HOVER
            },
        );
    }
    if selected {
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
            0.0,
            theme::SHAPE,
        );
    }
    ui.painter().text(
        Pos2::new(rect.left() + 7.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyph,
        FontId::proportional(10.0),
        if selected {
            theme::TEXT
        } else {
            theme::TEXT_SECONDARY
        },
    );
    ui.painter().text(
        Pos2::new(rect.left() + 28.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(10.0),
        if selected {
            theme::TEXT
        } else {
            theme::TEXT_SECONDARY
        },
    );
    if let Some(count) = count {
        ui.painter().text(
            Pos2::new(rect.right() - 5.0, rect.center().y),
            Align2::RIGHT_CENTER,
            count,
            FontId::monospace(8.0),
            theme::TEXT_MUTED,
        );
    }
    response
}

fn results(ui: &mut egui::Ui, state: &mut BrowserState) -> Option<&'static str> {
    let visible = filtered(state);
    components::Block {
        height: 27.0,
        fill: theme::PANEL,
        border_top: false,
        border_bottom: true,
        inset_x: 5.0,
    }
    .show(ui, Layout::left_to_right(Align::Center), |ui| {
        ui.label(RichText::new("Results").font(theme::interface_bold_font(11.0)));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(5.0);
            ui.label(
                RichText::new(visible.len().to_string())
                    .monospace()
                    .color(theme::TEXT_MUTED),
            );
        });
    });

    if visible.is_empty() {
        ui.label(RichText::new("No matching effects").color(theme::TEXT_MUTED));
        return None;
    }

    let mut selected = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| match state.view {
            ResultView::Visual | ResultView::Thumb => {
                let gap = 1.0;
                let columns = if ui.available_width() >= 150.0 { 2 } else { 1 };
                let width = ((ui.available_width() - gap * (columns - 1) as f32) / columns as f32)
                    .max(66.0);
                egui::Grid::new("candidate-effect-card-grid")
                    .spacing([gap, 1.0])
                    .show(ui, |ui| {
                        for (index, effect) in visible.iter().enumerate() {
                            if effect_card(ui, effect, state, width).clicked() {
                                selected = Some(effect.id);
                            }
                            if (index + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            }
            ResultView::Detail => {
                for effect in visible {
                    if detail_row(ui, effect, state.selected == effect.id).clicked() {
                        selected = Some(effect.id);
                    }
                }
            }
        });

    if let Some(id) = selected {
        state.selected = id;
    }
    selected
}

fn filtered(state: &BrowserState) -> Vec<&'static Effect> {
    let query = state.query.trim().to_ascii_lowercase();
    EFFECTS
        .iter()
        .filter(|effect| {
            (query.is_empty()
                || effect.search.contains(&query)
                || effect.name.to_ascii_lowercase().contains(&query))
                && (state.source == Source::All || effect.source == state.source)
                && state
                    .collection
                    .is_none_or(|value| effect.collection == Some(value))
                && state.tag.is_none_or(|value| effect.tags.contains(&value))
                && state.pack.is_none_or(|value| effect.pack == Some(value))
        })
        .collect()
}

fn tag_count(tag: Tag) -> usize {
    EFFECTS
        .iter()
        .filter(|effect| effect.tags.contains(&tag))
        .count()
}

fn effect_card(ui: &mut egui::Ui, effect: &Effect, state: &BrowserState, width: f32) -> Response {
    let show_name = state.view == ResultView::Thumb;
    let thumbnail_height = if show_name {
        (width * 9.0 / 16.0).max(42.0)
    } else {
        width.max(62.0)
    };
    let name_height = if show_name { 33.0 } else { 0.0 };
    let height = thumbnail_height + name_height;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let selected = state.selected == effect.id;
    let painter = ui.painter();

    painter.rect_filled(rect, 1.0, if selected { theme::RAISED } else { theme::APP });
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(
            1.0,
            if selected {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );

    let thumb = Rect::from_min_size(rect.min, Vec2::new(rect.width(), thumbnail_height));
    paint_thumbnail(painter, thumb.shrink(2.0), effect.id);
    painter.text(
        thumb.left_top() + Vec2::new(4.0, 4.0),
        Align2::LEFT_TOP,
        "FX",
        FontId::monospace(8.0),
        theme::TEXT,
    );
    if effect.has_motion {
        let mark_center = thumb.right_top() + Vec2::new(-12.5, 12.5);
        painter.circle_filled(mark_center, 8.5, theme::APP.gamma_multiply(0.88));
        painter.circle_stroke(mark_center, 8.5, Stroke::new(1.0, theme::BORDER_STRONG));
        painter.text(
            mark_center,
            Align2::CENTER_CENTER,
            "▶",
            FontId::proportional(8.0),
            theme::TEXT_SECONDARY,
        );
    }
    if let Some(badge) = effect.badge {
        let unavailable = badge == "Unavailable";
        let badge_width = if unavailable { 58.0 } else { 53.0 };
        let badge_rect = Rect::from_min_size(
            if unavailable {
                thumb.right_top() + Vec2::new(-badge_width - 4.0, 4.0)
            } else {
                thumb.left_bottom() + Vec2::new(4.0, -18.0)
            },
            Vec2::new(badge_width, 14.0),
        );
        painter.rect_filled(badge_rect, 1.0, theme::APP.gamma_multiply(0.92));
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge,
            FontId::monospace(if unavailable { 7.0 } else { 6.5 }),
            if unavailable {
                theme::WARNING
            } else {
                theme::ACCENT
            },
        );
    }

    if show_name {
        let display_name = match effect.id {
            "echo-bloom" => "Echo Bloom…",
            "type-pulse" => "Type Pulse",
            "fold-field" => "Fold Field",
            _ => effect.name,
        };
        painter.text(
            Pos2::new(rect.left() + 5.0, thumb.bottom() + name_height * 0.5),
            Align2::LEFT_CENTER,
            display_name,
            theme::interface_bold_font(10.0),
            theme::TEXT,
        );
    }

    response.on_hover_text(format!("{} · {}", effect.name, effect.subtype))
}

fn detail_row(ui: &mut egui::Ui, effect: &Effect, selected: bool) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 64.0), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        1.0,
        if selected {
            theme::ACCENT.gamma_multiply(0.14)
        } else if response.hovered() {
            theme::HOVER
        } else {
            theme::APP
        },
    );
    painter.rect_stroke(
        rect,
        1.0,
        Stroke::new(
            1.0,
            if selected {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ),
        StrokeKind::Inside,
    );
    let thumb = Rect::from_min_size(rect.min, Vec2::new(62.0, 64.0));
    paint_thumbnail(painter, thumb, effect.id);
    painter.text(
        Pos2::new(rect.left() + 68.0, rect.top() + 15.0),
        Align2::LEFT_CENTER,
        effect.name,
        FontId::proportional(10.0),
        theme::TEXT,
    );
    painter.text(
        Pos2::new(rect.left() + 68.0, rect.bottom() - 15.0),
        Align2::LEFT_CENTER,
        format!("{}  ›  {}", effect.category, effect.subtype),
        FontId::proportional(8.0),
        theme::TEXT_MUTED,
    );
    if let Some(badge) = effect.badge {
        painter.text(
            Pos2::new(rect.right() - 6.0, rect.center().y),
            Align2::RIGHT_CENTER,
            badge,
            FontId::monospace(7.0),
            if badge == "Unavailable" {
                theme::WARNING
            } else {
                theme::ACCENT
            },
        );
    }
    response
}

fn paint_thumbnail(painter: &egui::Painter, rect: Rect, id: &str) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(13, 14, 17));
    match id {
        "echo-bloom" => {
            for (radius, alpha) in [(23.0_f32, 25), (16.0, 48), (10.0, 92)] {
                painter.circle_filled(
                    rect.center(),
                    radius.min(rect.width() * 0.34),
                    theme::ACCENT.gamma_multiply(alpha as f32 / 100.0),
                );
            }
            painter.circle_filled(rect.center(), 3.5, theme::TEXT);
        }
        "type-pulse" => {
            painter.line_segment(
                [rect.left_bottom(), rect.right_top()],
                Stroke::new(1.0, theme::DATA),
            );
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Aa",
                FontId::proportional(24.0),
                theme::TEXT,
            );
        }
        _ => {
            let center = rect.center();
            painter.add(egui::Shape::convex_polygon(
                vec![
                    center + Vec2::new(-24.0, 7.0),
                    center + Vec2::new(1.0, -17.0),
                    center + Vec2::new(22.0, 3.0),
                    center + Vec2::new(3.0, 19.0),
                ],
                theme::SHAPE,
                Stroke::NONE,
            ));
            painter.line_segment(
                [rect.left_center(), rect.right_center()],
                Stroke::new(1.0, theme::WARNING),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_and_tag_filters_use_the_fixed_mock_fixture() {
        let mut state = BrowserState {
            query: "type".into(),
            ..BrowserState::default()
        };
        assert_eq!(filtered(&state)[0].id, "type-pulse");

        state.query.clear();
        state.tag = Some(Tag::Review);
        assert_eq!(filtered(&state)[0].id, "fold-field");
    }

    #[test]
    fn unmatched_search_returns_no_cards() {
        let state = BrowserState {
            query: "not-a-real-effect".into(),
            ..BrowserState::default()
        };
        assert!(filtered(&state).is_empty());
    }
}
