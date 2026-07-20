use crate::browser_component::{browser_ui, BrowserState};
use crate::components::{self, ToolIcon};
use crate::inspector_component::{inspector_ui, InspectorEffect, InspectorState};
use crate::stage_component::{stage_ui, StageState};
use crate::theme;
use crate::timeline_component::{timeline_ui, TimelineState};
use eframe::egui::{self, Align, Layout, RichText, Stroke};
use egui_tiles::{Container, Tile, Tree, UiResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneKind {
    Browser,
    Stage,
    Inspector,
}

pub struct MotoliiMock {
    tree: Tree<PaneKind>,
    state: MockState,
}

#[derive(Debug)]
struct MockState {
    browser: BrowserState,
    stage: StageState,
    inspector: InspectorState,
    timeline: TimelineState,
    settings_open: bool,
    browser_thumbnail_size: f32,
    status: String,
    last_timeline_status: String,
}

impl Default for MockState {
    fn default() -> Self {
        let timeline = TimelineState::default();
        Self {
            browser: BrowserState::default(),
            stage: StageState::default(),
            inspector: InspectorState::default(),
            last_timeline_status: timeline.status.clone(),
            timeline,
            settings_open: false,
            browser_thumbnail_size: 80.0,
            status: "Echo Bloom · selected effect · mock state only".into(),
        }
    }
}

impl MotoliiMock {
    pub fn new() -> Self {
        Self {
            tree: create_tree(),
            state: MockState::default(),
        }
    }

    fn title_bar(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("motolii-title")
            .default_size(34.0)
            .size_range(34.0..=34.0)
            .frame(panel_frame(theme::RAISED, true))
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("MOTOLII").strong().size(12.0));
                    ui.label(RichText::new("night_drive.mtl").strong());
                    ui.label(RichText::new("/ Main composition").color(theme::TEXT_SECONDARY));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(7.0);
                        if ui.button("Export").clicked() {
                            self.state.status =
                                "Export is outside this comparison prototype".into();
                        }
                        let _ = ui.add_enabled(false, egui::Button::new("Redo"));
                        let _ = ui.add_enabled(false, egui::Button::new("Undo"));
                        if ui.button("Settings").clicked() {
                            self.state.settings_open = !self.state.settings_open;
                        }
                    });
                });
            });
    }

    fn command_bar(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("motolii-commands")
            .default_size(32.0)
            .size_range(32.0..=32.0)
            .frame(panel_frame(theme::APP, true))
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(4.0);
                    for (icon, hint) in [
                        (ToolIcon::Select, "Select"),
                        (ToolIcon::Hand, "Hand"),
                        (ToolIcon::Shape, "Shape"),
                        (ToolIcon::Text, "Text"),
                        (ToolIcon::Connect, "Connect"),
                        (ToolIcon::Relative, "Relative Move"),
                        (ToolIcon::Camera, "Camera"),
                    ] {
                        let selected = hint == "Select";
                        if components::tool_button(ui, icon, selected)
                            .on_hover_text(hint)
                            .clicked()
                        {
                            self.state.status = format!("{hint} tool · prototype only");
                        }
                    }
                    ui.separator();
                    ui.label("COLOR BOOK");
                    ui.separator();
                    ui.label(RichText::new("Pulse rings / ").color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new(effect_name(self.state.browser.selected())).strong());
                });
            });
    }

    fn status_bar(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("motolii-status")
            .default_size(23.0)
            .size_range(23.0..=23.0)
            .frame(panel_frame(theme::APP, true))
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);
                    let (marker, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(8.0), egui::Sense::hover());
                    ui.painter().rect_filled(marker, 0.0, theme::WAY_INSPECTOR);
                    ui.label(RichText::new(&self.state.status).color(theme::TEXT_SECONDARY));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("egui 0.35 · egui_tiles 0.16 · comparison prototype")
                                .monospace()
                                .color(theme::TEXT_MUTED),
                        );
                    });
                });
            });
    }

    fn timeline_panel(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("motolii-timeline")
            .default_size(270.0)
            .size_range(190.0..=430.0)
            .resizable(true)
            .frame(panel_frame(theme::PANEL, true))
            .show(root, |ui| timeline_ui(ui, &mut self.state.timeline));
        if self.state.last_timeline_status != self.state.timeline.status {
            self.state.last_timeline_status = self.state.timeline.status.clone();
            self.state.status = self.state.timeline.status.clone();
        }
    }

    fn settings_window(&mut self, context: &egui::Context) {
        if !self.state.settings_open {
            return;
        }
        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .open(&mut self.state.settings_open)
            .show(context, |ui| {
                ui.label("Browser thumbnail size");
                ui.add(
                    egui::Slider::new(&mut self.state.browser_thumbnail_size, 56.0..=128.0)
                        .suffix(" px"),
                );
                ui.separator();
                ui.label(
                    RichText::new("Workspace-session candidate · Document unchanged")
                        .color(theme::TEXT_MUTED),
                );
            });
    }
}

impl eframe::App for MotoliiMock {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.title_bar(root);
        self.command_bar(root);
        self.status_bar(root);
        self.timeline_panel(root);

        egui::CentralPanel::default()
            .frame(panel_frame(theme::PANEL, false))
            .show(root, |ui| {
                let mut behavior = WorkspaceBehavior {
                    state: &mut self.state,
                };
                self.tree.ui(&mut behavior, ui);
            });

        let context = root.ctx().clone();
        self.settings_window(&context);
    }
}

struct WorkspaceBehavior<'a> {
    state: &'a mut MockState,
}

impl egui_tiles::Behavior<PaneKind> for WorkspaceBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &PaneKind) -> egui::WidgetText {
        match pane {
            PaneKind::Browser => "Browser",
            PaneKind::Stage => "Stage",
            PaneKind::Inspector => "Inspector",
        }
        .into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut PaneKind,
    ) -> UiResponse {
        match pane {
            PaneKind::Browser => {
                if let Some(id) = browser_ui(ui, &mut self.state.browser) {
                    self.state.inspector.selected_effect = inspector_effect(id);
                    self.state.status =
                        format!("{} · selected effect · Document unchanged", effect_name(id));
                }
            }
            PaneKind::Stage => {
                stage_ui(ui, &mut self.state.stage);
                if self.state.stage.fit_requested {
                    self.state.status = "Stage fitted to output frame".into();
                } else if self.state.stage.previous_key_requested {
                    self.state.status = "Previous key".into();
                } else if self.state.stage.next_key_requested {
                    self.state.status = "Next key".into();
                }
            }
            PaneKind::Inspector => inspector_ui(ui, &mut self.state.inspector),
        }
        UiResponse::None
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        2.0
    }

    fn min_size(&self) -> f32 {
        180.0
    }
}

fn create_tree() -> Tree<PaneKind> {
    let mut tiles = egui_tiles::Tiles::default();
    let browser = tiles.insert_pane(PaneKind::Browser);
    let stage = tiles.insert_pane(PaneKind::Stage);
    let inspector = tiles.insert_pane(PaneKind::Inspector);
    let root = tiles.insert_horizontal_tile(vec![browser, stage, inspector]);
    if let Some(Tile::Container(Container::Linear(linear))) = tiles.get_mut(root) {
        linear.shares.set_share(browser, 284.0);
        linear.shares.set_share(stage, 830.0);
        linear.shares.set_share(inspector, 326.0);
    }
    Tree::new("motolii-workspace-tree", root, tiles)
}

fn inspector_effect(id: &str) -> InspectorEffect {
    match id {
        "type-pulse" => InspectorEffect::TypePulse,
        "fold-field" => InspectorEffect::FoldField,
        _ => InspectorEffect::EchoBloom,
    }
}

fn effect_name(id: &str) -> &'static str {
    match id {
        "type-pulse" => "Type Pulse",
        "fold-field" => "Fold Field",
        _ => "Echo Bloom",
    }
}

fn panel_frame(fill: egui::Color32, border: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(if border {
            Stroke::new(1.0, theme::BORDER)
        } else {
            Stroke::NONE
        })
        .inner_margin(egui::Margin::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "manual visual inspection helper"]
    fn capture_full_mock() {
        let output = std::env::var("MOTOLII_KITTEST_CAPTURE")
            .expect("set MOTOLII_KITTEST_CAPTURE to a PNG path");
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1440.0, 900.0))
            .build_eframe(|creation_context| {
                crate::theme::install(&creation_context.egui_ctx);
                MotoliiMock::new()
            });
        harness.run_steps(3);
        harness
            .render()
            .expect("mock should render")
            .save(output)
            .expect("mock capture should save");
    }
}
