// 家: 窓。blitz/dioxus-native の上に置く面と、面が持つ表示状態だけ。
// 編集状態は Document(doc)が持ち、書き込みは Intent 経由のみ。
pub mod app;
mod browser;
mod fixture;
mod inspector;
mod keymap;
mod playback;
mod session;
mod stage_widget;
mod thumbnail;
mod timeline_shell;
mod timeline_widget;
mod tokens;
