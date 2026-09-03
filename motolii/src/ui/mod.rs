// 家: 窓。blitz/dioxus-native の上に置く面と、面が持つ表示状態だけ。
// 編集状態は Document(doc)が持ち、書き込みは Intent 経由のみ。
pub mod app;
mod browser;
mod color;
mod desk;
mod dock;
mod ease;
mod ease_model;
mod ease_widget;
mod fixture;
#[cfg(test)]
mod gui;
pub mod host;
mod inspector;
mod keymap;
mod output;
mod playback;
mod project;
mod semantic_menu;
mod session;
mod stage_widget;
mod thumbnail;
mod timeline_shell;
mod timeline_widget;
pub(crate) mod tokens;
mod utility;

/// 白紙。**枠だけは要る** —— 枠が無いと何も描けず、窓が空を出す。
/// 大きさは既定の 1920x1080 30fps 60秒。
pub fn blank_project() -> crate::doc::store::Document {
    use crate::doc::store::{Composition, Document, Fps, Intent};
    let mut doc = Document::new();
    let comp = Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 1800,
        background: [0.0, 0.0, 0.0, 1.0],
    };
    let _ = doc.apply(Intent::SetComposition(comp));
    doc
}
