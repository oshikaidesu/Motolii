// 家は 5 つ。build も家ごと: doc(契約)→ render(道路の中身)→ ui(この crate、hotpatch が追う tip)。
pub use motolii_doc as doc;
pub use motolii_render as render;
pub mod ui;

#[cfg(feature = "dynamic")]
#[allow(unused_imports)]
use motolii_road;
