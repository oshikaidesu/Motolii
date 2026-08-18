//! 人に訊く口 — **egui shell の trait をそのまま共用する**。
//!
//! M-0 ではこの crate に New / Open の2本だけを持つ小さな trait を建てた。M-1 で
//! Export と未保存の3択が要るようになったところで、それは egui shell
//! (`motolii_ui::blitz_shell::ShellPrompts`)が既に持っている4本と**同じ物**だと
//! 分かった。同じ物を2つ持てば、いつか文言か既定値がずれる — dialog は
//! 「利用者から見て窓が替わったと分からない」ことが値打ちなので、ここでは
//! 2つ目を作らずに向こうを差す。
//!
//! そのために `motolii-ui` 側で `ShellPrompts` / `NativePrompts` / `ScriptedPrompts` を
//! `pub` にした(意味は不変。どれも toolkit の型を1つも持たないので U0a の境界は
//! 破れない)。結果として:
//!
//! - **この crate は `rfd` を直接呼ばない。** native dialog の呼び出しは repo に
//!   1箇所(`blitz_shell/drive.rs` の `NativePrompts`)だけになった
//! - 台本(`ScriptedPrompts`)も共通なので、egui 版の運転席テストと iced 版の
//!   運転席テストが**同じ台本の書き方**をする
//!
//! **なぜ intent の外に居るのか。** 決まった答え(path・3択)だけが `UiIntent` の
//! 中へ入るので、journal を replay しても dialog は二度と開かない。この規律は
//! host を替えても変わらない。

pub use motolii_ui::blitz_shell::{NativePrompts, ScriptedPrompts, ShellPrompts};
