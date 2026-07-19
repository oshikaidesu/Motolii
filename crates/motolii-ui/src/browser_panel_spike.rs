//! SPIKE: 制限付きfixtureをeguiへ投影できるかだけを検証する。

use egui::{RichText, TextEdit, Ui};
use egui_taffy::taffy;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{tui, TuiBuilderLogic};

#[derive(Debug, Clone, Copy)]
struct BrowserCardSpec {
    id: &'static str,
    glyph: &'static str,
    name: &'static str,
    kind: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct BrowserPanelSpec {
    id: &'static str,
    label: &'static str,
    sources: &'static [&'static str],
    cards: &'static [BrowserCardSpec],
}

include!(concat!(env!("OUT_DIR"), "/browser_panel_fixture.rs"));

#[derive(Debug)]
pub(crate) struct BrowserPanelState {
    surface_index: usize,
    source_index: usize,
    query: String,
}

impl Default for BrowserPanelState {
    fn default() -> Self {
        Self {
            surface_index: GENERATED_SURFACES
                .iter()
                .position(|surface| surface.id == "gen")
                .unwrap_or_default(),
            source_index: 0,
            query: String::new(),
        }
    }
}

pub(crate) fn paint(ui: &mut Ui, state: &mut BrowserPanelState) {
    ui.options_mut(|options| {
        options.max_passes = std::num::NonZeroUsize::new(2).expect("two is non-zero");
    });

    ui.heading("Browser");
    ui.horizontal(|ui| {
        for (index, surface) in GENERATED_SURFACES.iter().enumerate() {
            if ui
                .selectable_label(state.surface_index == index, surface.label)
                .clicked()
            {
                state.surface_index = index;
                state.source_index = 0;
            }
        }
    });
    ui.add(TextEdit::singleline(&mut state.query).hint_text("Search"));
    ui.separator();

    let spec = &GENERATED_SURFACES[state.surface_index.min(GENERATED_SURFACES.len() - 1)];
    let query = state.query.to_lowercase();
    let cards: Vec<_> = spec
        .cards
        .iter()
        .filter(|card| query.is_empty() || card.name.to_lowercase().contains(&query))
        .collect();

    tui(ui, ui.id().with(("browser-grid", spec.id)))
        .reserve_available_width()
        .style(taffy::Style {
            size: taffy::Size {
                width: percent(1.0),
                height: auto(),
            },
            flex_direction: taffy::FlexDirection::Row,
            align_items: Some(taffy::AlignItems::FlexStart),
            gap: length(6.0),
            ..Default::default()
        })
        .show(|tui| {
            tui.id("source-rail")
                .style(taffy::Style {
                    size: taffy::Size {
                        width: length(86.0),
                        height: auto(),
                    },
                    padding: length(4.0),
                    flex_direction: taffy::FlexDirection::Column,
                    gap: length(2.0),
                    ..Default::default()
                })
                .add_with_border(|tui| {
                    tui.ui(|ui| {
                        ui.label(RichText::new(spec.label.to_uppercase()).small().strong());
                    });
                    for (index, source) in spec.sources.iter().enumerate() {
                        if tui
                            .id(egui::Id::new(("source", *source)))
                            .style(taffy::Style {
                                size: taffy::Size {
                                    width: length(74.0),
                                    height: auto(),
                                },
                                padding: length(3.0),
                                ..Default::default()
                            })
                            .wrap_mode(egui::TextWrapMode::Truncate)
                            .ui(|ui| {
                                ui.set_width(68.0);
                                ui.selectable_label(state.source_index == index, *source)
                            })
                            .clicked()
                        {
                            state.source_index = index;
                        }
                    }
                });

            tui.id("results")
                .style(taffy::Style {
                    flex_grow: 1.0,
                    min_size: taffy::Size {
                        width: length(90.0),
                        height: auto(),
                    },
                    flex_direction: taffy::FlexDirection::Column,
                    gap: length(4.0),
                    ..Default::default()
                })
                .add(|tui| {
                    tui.ui(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Results").small().strong());
                            ui.label(RichText::new(cards.len().to_string()).small().weak());
                        });
                    });
                    tui.id("result-grid")
                        .style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Row,
                            flex_wrap: taffy::FlexWrap::Wrap,
                            align_items: Some(taffy::AlignItems::Stretch),
                            gap: length(5.0),
                            ..Default::default()
                        })
                        .add(|tui| {
                            for card in &cards {
                                tui.id(card.id)
                                    .style(taffy::Style {
                                        size: taffy::Size {
                                            width: length(86.0),
                                            height: length(78.0),
                                        },
                                        padding: length(6.0),
                                        flex_direction: taffy::FlexDirection::Column,
                                        justify_content: Some(taffy::JustifyContent::SpaceBetween),
                                        ..Default::default()
                                    })
                                    .add_with_border(|tui| {
                                        tui.ui(|ui| {
                                            ui.label(RichText::new(card.glyph).size(22.0));
                                        });
                                        tui.ui(|ui| {
                                            ui.label(RichText::new(card.name).small().strong());
                                            ui.label(RichText::new(card.kind).small().weak());
                                        });
                                    });
                            }
                        });
                });
        });
}

#[cfg(test)]
mod tests {
    use super::GENERATED_SURFACES;
    use std::collections::BTreeSet;

    #[test]
    fn generated_fixture_has_unique_non_empty_surfaces_and_cards() {
        let mut surface_ids = BTreeSet::new();
        let mut card_ids = BTreeSet::new();
        assert!(!GENERATED_SURFACES.is_empty());
        for surface in GENERATED_SURFACES {
            assert!(surface_ids.insert(surface.id));
            assert!(!surface.sources.is_empty());
            assert!(!surface.cards.is_empty());
            for card in surface.cards {
                assert!(!card.name.is_empty());
                assert!(card_ids.insert(card.id));
            }
        }
    }
}
