//! Turns a work into pictures and sound: the bridge from `motolii-doc` to the GPU.
//!
//! `frame_graph` compiles one revision of a work into a graph of nodes and evaluates it
//! once per time; `engine` and `compositor` run that graph's GPU work on the rerun
//! renderer (`re_renderer`); `picture` resolves layers into drawable form; `media`,
//! `audio` and `playback` decode and keep time; `export` writes files; `extensions`
//! holds the bundled effects, placement programs and text. GPU, decoding and colour
//! come from upstream crates; this crate only maps the work onto them.
pub mod picture;
pub mod audio;
pub mod compositor;
pub mod engine;
pub mod export;
pub mod extensions;
pub mod frame_graph;
pub mod media;
pub mod playback;

pub use motolii_doc as doc;

/// 家の中の道は `crate::render::…` のまま。
pub mod render {
    pub use crate::{audio, compositor, engine, export, media};
}
