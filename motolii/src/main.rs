use std::any::Any;

fn main() {
    re_log::setup_logging();
    let config: Vec<Box<dyn Any>> = vec![Box::new(
        dioxus_native::WindowAttributes::default().with_title("Motolii"),
    )];
    dioxus_native::launch_cfg(motolii::ui::app::app, Vec::new(), config);
}
