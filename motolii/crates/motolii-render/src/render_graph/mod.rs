//! Backend-neutral render work produced from evaluated Motolii semantics.
//!
//! This module deliberately does not know authored semantic node kinds. It is the
//! cassette contract between semantic evaluation/lowering and execution backends.

pub mod work;

pub use work::{Composed, Extrusion, ImageInput, LayerWork, MaterialRecipe, RasterSource, RenderGraph};
