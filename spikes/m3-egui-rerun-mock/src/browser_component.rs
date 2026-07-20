use crate::theme;
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, Vec2,
};

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

pub(crate) fn browser_ui(ui: &mut egui::Ui, state: &mut BrowserState) -> Option<&'static str> {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.painter().rect_filled(ui.max_rect(), 0.0, theme::PANEL);

    browser_header(ui);
    let mut changed = None;
    search_row(ui, state);
    ui.add_space(5.0);
    ui.separator();

    let available = ui.available_size();
    ui.allocate_ui_with_layout(available, Layout::left_to_right(Align::Min), |ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(106.0, available.y),
            Layout::top_down(Align::Min),
            |ui| source_rail(ui, state),
        );
        ui.separator();
        ui.allocate_ui_with_layout(ui.available_size(), Layout::top_down(Align::Min), |ui| {
            changed = results(ui, state);
        });
    });
    changed
}

fn browser_header(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let header = Rect::from_min_size(ui.cursor().min, Vec2::new(width, 29.0));
    ui.painter().rect_filled(header, 0.0, theme::RAISED);
    ui.painter().line_segment(
        [header.left_bottom(), header.right_bottom()],
        Stroke::new(1.0, theme::BORDER),
    );
    ui.allocate_ui_with_layout(header.size(), Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(9.0);
        let (marker, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), Sense::hover());
        ui.painter().rect_filled(marker, 0.0, theme::SHAPE);
        ui.label(RichText::new("Browser").strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(7.0);
            ui.label(
                RichText::new("MEDIA / CREATE / EFFECTS")
                    .monospace()
                    .size(7.0)
                    .color(theme::TEXT_MUTED),
            );
        });
    });

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        for (label, active) in [("Media", false), ("Effects", true), ("Create", false)] {
            let response = ui.add_sized(
                [width / 3.0, 29.0],
                egui::Button::new(RichText::new(label).size(10.0))
                    .fill(theme::PANEL)
                    .stroke(Stroke::NONE)
                    .corner_radius(0.0),
            );
            if active {
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        Pos2::new(response.rect.left(), response.rect.bottom() - 2.0),
                        Vec2::new(response.rect.width(), 2.0),
                    ),
                    0.0,
                    theme::SHAPE,
                );
            }
        }
    });
}

fn search_row(ui: &mut egui::Ui, state: &mut BrowserState) {
    ui.horizontal(|ui| {
        let field_width = (ui.available_width() - 81.0).max(52.0);
        ui.add_sized(
            [field_width, 25.0],
            egui::TextEdit::singleline(&mut state.query)
                .hint_text("Search")
                .desired_width(field_width),
        );
        view_button(ui, state, ResultView::Visual, "S", "Thumbnail-only view");
        view_button(ui, state, ResultView::Thumb, "G", "Thumbnail and name view");
        view_button(ui, state, ResultView::Detail, "L", "List view");
    });
}

fn view_button(
    ui: &mut egui::Ui,
    state: &mut BrowserState,
    view: ResultView,
    glyph: &str,
    hint: &str,
) {
    if ui
        .add_sized(
            [25.0, 25.0],
            egui::Button::selectable(state.view == view, glyph),
        )
        .on_hover_text(hint)
        .clicked()
    {
        state.view = view;
    }
}

fn source_rail(ui: &mut egui::Ui, state: &mut BrowserState) {
    ui.spacing_mut().item_spacing = Vec2::new(2.0, 1.0);

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

    ui.add_space(6.0);
    ui.add_sized([100.0, 24.0], egui::Button::new("＋ Save current…"))
        .on_hover_text("Save current effect");
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.add_space(7.0);
    ui.label(
        RichText::new(title.to_ascii_uppercase())
            .monospace()
            .size(8.0)
            .color(theme::TEXT_MUTED),
    );
    ui.add_space(1.0);
}

fn nav_row(
    ui: &mut egui::Ui,
    glyph: &str,
    label: &str,
    selected: bool,
    count: Option<usize>,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 21.0), Sense::click());
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            1.0,
            if selected {
                theme::ACCENT.gamma_multiply(0.18)
            } else {
                theme::HOVER
            },
        );
    }
    if selected {
        ui.painter().rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
            0.0,
            theme::ACCENT,
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
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), 20.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(RichText::new("Results").strong().size(11.0));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(visible.len().to_string())
                        .monospace()
                        .color(theme::TEXT_MUTED),
                );
            });
        },
    );

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
    let name_height = if show_name { 30.0 } else { 0.0 };
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
        painter.text(
            Pos2::new(rect.left() + 5.0, thumb.bottom() + name_height * 0.5),
            Align2::LEFT_CENTER,
            effect.name,
            FontId::proportional(10.0),
            theme::TEXT,
        );
        let tags = effect
            .tags
            .iter()
            .map(|tag| format!("#{}", tag_label(*tag)))
            .collect::<Vec<_>>()
            .join(" ");
        painter.text(
            Pos2::new(rect.right() - 5.0, thumb.bottom() + name_height * 0.5),
            Align2::RIGHT_CENTER,
            tags,
            FontId::proportional(7.0),
            theme::TEXT_MUTED,
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

fn tag_label(tag: Tag) -> &'static str {
    match tag {
        Tag::GoTo => "Go-to",
        Tag::Atmosphere => "Atmosphere",
        Tag::Kinetic => "Kinetic",
        Tag::Review => "Review",
    }
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
        let mut state = BrowserState::default();
        state.query = "type".into();
        assert_eq!(filtered(&state)[0].id, "type-pulse");

        state.query.clear();
        state.tag = Some(Tag::Review);
        assert_eq!(filtered(&state)[0].id, "fold-field");
    }

    #[test]
    fn unmatched_search_returns_no_cards() {
        let mut state = BrowserState::default();
        state.query = "not-a-real-effect".into();
        assert!(filtered(&state).is_empty());
    }
}
