// 家: 窓。blitz/dioxus-native の上に置く面と、面が持つ表示状態だけ。
// 編集状態は Document(doc)が持ち、書き込みは Intent 経由のみ。
pub mod app;
mod autosave;
mod blend_preview;
mod browser;
mod browser_selection;
mod clipboard;
mod clipping;
mod color;
mod commands;
mod composition;
mod context_menu;
mod contracts;
mod desk;
mod dock;
mod dock_hit;
mod ease;
mod ease_model;
mod ease_widget;
mod export_sheet;
mod fixture;
mod functions;
#[cfg(test)]
mod gui;
pub mod host;
mod inspector;
mod keymap;
pub(crate) mod keys;
mod mount;
mod output;
mod panels;
mod playback;
mod poke;
mod project;
mod property_edit;
mod semantic_menu;
mod session;
mod settings;
mod stage_widget;
mod thumbnail;
mod timeline_edit;
mod timeline_shell;
mod timeline_widget;
pub(crate) mod tokens;
mod utility;
mod window_frame;

pub use crate::doc::store::blank_project;
