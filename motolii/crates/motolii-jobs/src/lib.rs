//! Background authoring jobs: export and freeze. A job takes a snapshot of a work
//! (`motolii-doc` + `motolii-render`), runs on its own thread and never touches the live editor.
use motolii_doc as doc;
use motolii_render as render;
pub mod export;
pub mod freeze;
