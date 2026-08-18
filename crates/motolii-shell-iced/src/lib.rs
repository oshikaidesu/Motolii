//! iced ホストの殻 — M-1。
//!
//! [2026-08-18裁定](../../../docs/reviews/2026-08-18-iced-host-migration-decision.md)
//! の絞め殺し移行の受け皿である。egui shell(`motolii_ui::blitz_shell`)は**並走したまま**で、
//! こちらが「窓としてやること」を1つずつ引き取っていく。
//!
//! ## ここに在るもの
//!
//! - **スタート画面**。New Project / Open の2ボタン。文言は egui 版と同じ
//!   ([`view::NEW_PROJECT`] / [`view::OPEN_PROJECT`])
//! - **背骨は持ち越し**。押した結果は `motolii_ui::blitz_shell::UiIntent` になって
//!   `ShellGateway` を通る。journal も transcript も egui shell と**同じ型の同じ契約**で、
//!   ここで2つ目の台帳を作らない(裁定文書「意味は不変のまま iced へ移る」)
//! - **保存と未保存 guard**。`Cmd+S` が保存し、帯は保存状態(未保存なら `● … — unsaved`)を
//!   出す。未保存のまま New / Open / 窓を閉じる をやると3択(Save / Discard / Cancel)で、
//!   判断は egui shell と**同じ `decide_unsaved`** を通る
//! - **OS ドロップ**。窓へ落ちた素材は `UiIntent::AdmitPaths` になって playhead へ立つ。
//!   project が無ければ「先に作る」と案内する(台本 P3)。winit はファイル1つにつき
//!   1事象を出すので、**フレームの区切りでまとめて**1件の取り込みにする
//! - **書き出し**。帯の Export → 保存先 dialog → 既存の `export_seat` が別 thread で走る。
//!   実行中は `Exporting… {経過}s` と Cancel が帯に出て、Export は消える(二重起動なし)
//! - **status 帯**。`ShellTranscript::latest()` の1行を下端に出す
//! - **見た目の token**([`theme`])。iced 既定 palette を残さず、
//!   `ui/motolii-tokens/generated/` の生成正本から写した [`theme::Tokens`] で
//!   塗る。照合は `tests/theme_matches_generated_tokens.rs`、絵そのものは
//!   `tests/snapshot_start_screen.rs` の golden が固定する
//! - **記録2本**。[`IntentLog`](`--intent-log`)と [`StatusLog`](`--status-log`)が
//!   journal / transcript を JSONL で追記する。行の形は egui runner と同型
//! - **運転席**。`iced_test::Simulator` で窓を開かずに押して・叩いて・落として読む
//!   (`tests/` 配下)。近道キーと OS ドロップが購読ではなく widget 木
//!   ([`window_input`])を通るのは、運転席から注入できる道を1本に保つためである
//!
//! ## M-2 で加わったもの
//!
//! - **Stage 島**([`stage_island`])。座席が在る間の中央 pane に Rerun
//!   `SpatialStage` の合成絵が立つ。入力は翻訳([`stage_bridge`])と
//!   調停3状態([`stage_arbiter`]: 素通し / orbit 中 / 掴み中)を通り、
//!   失敗は [`Message::StageReported`] として帯へ出る。bind group 床
//!   (iced fork seam 2)の実効 oracle も常設(`tests/stage_bind_groups_oracle.rs`)
//!
//! ## ここに無いもの(意図的に)
//!
//! - Timeline(M-3)、Browser / Inspector(M-4)。ギズモの絵と意味論・
//!   書き出しカメラ枠 overlay は M-2 後。Undo / Redo ボタンは編集面と一緒に来る
//! - 引数なし起動の「続きが開く」(egui 側 wave D の `last_project`)。M-2 以降
//! - egui。**この crate に egui 系の直接依存は1つも無い**
//!   (柵: `crates/motolii-testkit/src/ui_toolkit_dep_policy.rs` の
//!   `UI_TOOLKIT_CRATE_ALLOWLIST` にこの crate を**入れていない**)
//! - `rfd` の直接呼び出し。native dialog は egui shell と**同じ実装**
//!   (`motolii_ui::blitz_shell::NativePrompts`)の後ろに1箇所だけ在る

mod intent_log;
mod jsonl;
mod launch;
mod message;
mod prompts;
mod shell;
pub mod stage_arbiter;
pub mod stage_bridge;
pub mod stage_island;
mod status_log;
pub mod theme;
pub mod view;
mod window_input;

pub use intent_log::IntentLog;
pub use launch::Launch;
pub use message::Message;
pub use prompts::{NativePrompts, ScriptedPrompts, ShellPrompts};
pub use shell::{Outcome, Shell};
pub use status_log::StatusLog;
pub use view::view;
pub use window_input::window_input;
