mod app;
mod browser_component;
mod components;
mod fixture;
mod inspector_component;
mod stage_component;
mod theme;
mod timeline_component;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Motolii · egui / Rerun pattern mock")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1080.0, 680.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Motolii · egui / Rerun pattern mock",
        options,
        Box::new(|creation_context| {
            theme::install(&creation_context.egui_ctx);
            Ok(Box::new(app::MotoliiMock::new()))
        }),
    )
}
