//! `write.rs` の8個の `#[cfg(test)]` モジュール(third_slice_fixtures 含む)を
//! そのまま移設(SP-2 分割、`write.rs` 1813-2726行を移設)。**中身は無改変**。
//! 元は `write` 直下の兄弟モジュールだったが、1ファイル800行以下のため
//! 内容ごとにさらに分けた(fixtures は共有、他は元の1モジュール=1ファイル)。
//! この `mod.rs` は `#[cfg(test)]`(親の `write::tests` 宣言側)の傘下なので
//! 個々のファイルに重複して付けていない。

mod fixtures;
mod fold;
mod key_interp;
mod key_selection_verbs;
mod rename;
mod restack;
mod reverse_selected_keys;
mod waveform;
