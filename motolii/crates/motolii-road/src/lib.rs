//! 道路。doc と render(wgpu・rerun・Vello を含む重い依存)を 1 本の dylib に 1 回だけ link する。
//! 開発の 1 周で link し直すのは ui(tip crate)だけになる。Bevy の dynamic_linking と同じ運用。
#![allow(unused_imports)]
pub use motolii_doc;
pub use motolii_render;
