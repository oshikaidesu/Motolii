//! iced ホストの殻 — M-0 の土台。
//!
//! [2026-08-18裁定](../../../docs/reviews/2026-08-18-iced-host-migration-decision.md)
//! の絞め殺し移行の受け皿である。egui shell(`motolii_ui::blitz_shell`)は**並走したまま**で、
//! こちらは M-1 以降が中身を足していく空の家として先に建てる。
//!
//! ## ここに在るもの
//!
//! - **スタート画面**。New Project / Open の2ボタン。文言は egui 版と同じ
//!   ([`view::NEW_PROJECT`] / [`view::OPEN_PROJECT`])
//! - **背骨は持ち越し**。押した結果は `motolii_ui::blitz_shell::UiIntent` になって
//!   `ShellGateway` を通る。journal も transcript も egui shell と**同じ型の同じ契約**で、
//!   ここで2つ目の台帳を作らない(裁定文書「意味は不変のまま iced へ移る」)
//! - **status 帯**。`ShellTranscript::latest()` の1行を下端に出す
//! - **`--intent-log` 相当**。[`IntentLog`] が journal を JSONL で追記する。
//!   行の形は `IntentEvent` の宣言そのもので、egui shell の `--intent-log` と同型
//! - **運転席**。`iced_test::Simulator` で窓を開かずに押して読む
//!   (`tests/drive_seat.rs`)。egui 版 `drive_tests.rs` の最初の2テストに対応する
//!
//! ## ここに無いもの(意図的に)
//!
//! - Rerun の Stage 島(M-2)、Timeline(M-3)、Browser / Inspector(M-4)
//! - Save / Export / 未保存確認 / ドロップ / キーボード近道(M-1)
//! - egui。**この crate に egui 系の直接依存は1つも無い**
//!   (柵: `crates/motolii-testkit/src/ui_toolkit_dep_policy.rs` の
//!   `UI_TOOLKIT_CRATE_ALLOWLIST` にこの crate を**入れていない**)

mod intent_log;
mod message;
mod prompts;
mod shell;
pub mod view;

pub use intent_log::IntentLog;
pub use message::Message;
pub use prompts::{NativePrompts, ScriptedPrompts, ShellPrompts};
pub use shell::Shell;
pub use view::view;
