//! Background authoring jobs consume document snapshots, never the live editor.
use motolii_doc as doc;
use motolii_render as render;
pub mod export;
pub mod freeze;
