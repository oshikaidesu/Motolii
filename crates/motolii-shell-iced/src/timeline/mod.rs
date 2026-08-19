//! Timeline pane — M-3。spike(`spikes/iced-rerun-embed-probe/timeline`)を種に、
//! 実 Document の投影・Q1 文法(click=選択 / drag=移動 / 端drag=trim /
//! release=確定 / Esc=復元 / Delete=削除 / Cmd+Z=undo)・playhead scrub・
//! zoom / pan・波形帯を持つ。
//!
//! 層の分担:
//! - [`semantics`] … egui 版から移植した**意味関数**(純関数のみ)
//! - [`pane`] … ジェスチャの状態機械。**release の1件だけが `UiIntent` になる**
//! - [`canvas`] … 絵と、生イベント → [`pane::TimelineMsg`] の翻訳
//! - [`structure`] … 構造操作(fold・rename・lock)の描画・補助
//!   (2026-08-19 構造操作レーン。柵: 新しい描画・hit のコードはここへ)
//! - [`waveform`] … soundtrack 波形の生成座席(縮約の意味は egui 版と共用)
//!
//! 編集が Document へ届く道は `Shell::update` の
//! `ShellGateway::dispatch(UiIntent::…)` **だけ**である
//! (フェンス: `tests/intent_gateway_fence.rs` — `editor_mut(` はこの crate で禁止)。

pub mod canvas;
pub mod pane;
pub mod semantics;
pub mod structure;
pub mod waveform;

pub use canvas::TimelineProgram;
pub use pane::{GrabZone, TimelineCtx, TimelineDrag, TimelineMsg, TimelinePane};
pub use waveform::{WaveformBandState, WaveformSeat};
