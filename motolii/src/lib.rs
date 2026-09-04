pub use motolii_doc as doc;
pub use motolii_render as render;

pub mod ui {
    pub use motolii_doc::store::blank_project;
}

#[cfg(feature = "dynamic")]
#[allow(unused_imports)]
use motolii_road;
