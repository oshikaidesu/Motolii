//! U1a-1: 組み込み既定 layout（static egui panel + placeholder のみ）。

use egui::{TextureId, Ui, Vec2};

use crate::browser_panel_spike::{self, BrowserPanelState};

pub(crate) const BROWSER_PANEL: &str = "motolii-browser";
pub(crate) const INSPECTOR_PANEL: &str = "motolii-inspector";
pub(crate) const TIMELINE_PANEL: &str = "motolii-timeline";
pub(crate) const STATUS_PANEL: &str = "motolii-status";
pub(crate) const STAGE_LABEL: &str = "Stage (Preview)";

pub(crate) fn paint(
    ui: &mut Ui,
    viewport_texture: TextureId,
    viewport_size: Vec2,
    browser: &mut BrowserPanelState,
) {
    egui::Panel::left(BROWSER_PANEL)
        .resizable(true)
        .default_size(330.0)
        .min_size(220.0)
        .show(ui, |ui| {
            browser_panel_spike::paint(ui, browser);
        });

    egui::Panel::right(INSPECTOR_PANEL)
        .resizable(false)
        .default_size(260.0)
        .show(ui, |ui| {
            ui.heading("Inspector");
            ui.label("Properties placeholder");
        });

    egui::Panel::bottom(TIMELINE_PANEL)
        .resizable(false)
        .default_size(120.0)
        .show(ui, |ui| {
            ui.heading("Timeline");
            ui.label("Tracks placeholder");
        });

    egui::Panel::bottom(STATUS_PANEL)
        .resizable(false)
        .exact_size(24.0)
        .show(ui, |ui| {
            ui.label("Status");
        });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading(STAGE_LABEL);
        ui.add(
            egui::Image::from_texture((viewport_texture, viewport_size))
                .fit_to_exact_size(viewport_size),
        );
    });
}
