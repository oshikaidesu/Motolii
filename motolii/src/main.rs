pub use motolii_doc as doc;
pub use motolii_render as render;
mod ui;

#[cfg(feature = "dynamic")]
#[allow(unused_imports)]
use motolii_road;

fn main() {
    re_log::setup_logging();
    ui::host::launch("Motolii");
}
