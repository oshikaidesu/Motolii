use std::any::Any;

mod app;
mod browser;
mod fixture;
mod inspector;
mod keymap;
mod playback;
mod session;
mod stage_widget;
mod thumbnail;
mod timeline_shell;
mod timeline_widget;
mod tokens;

fn main() {
    re_log::setup_logging();
    let config: Vec<Box<dyn Any>> = vec![Box::new(
        dioxus_native::WindowAttributes::default().with_title("Motolii"),
    )];
    dioxus_native::launch_cfg(app::app, Vec::new(), config);
}
