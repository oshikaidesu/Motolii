//! 移植済みの Blitz パネルを1つの窓へ合体させる殻。
//!
//! 構成は [P7](../../../../docs/reviews/2026-08-15-blitz-ui-runtime-probe.md) の実走そのもの。
//! **窓と wgpu デバイスを持つのは `eframe(egui)`** で、Blitz は毎フレームそのデバイス上の
//! テクスチャへ描き、egui がそれを合成する。ドッキングは
//! [2026-08-15裁定](../../../../docs/blitz-port-order-capsules.md)により **egui の責任**で、
//! `egui_tiles` を移植せずそのまま使う。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-blitz-shell
//! ```
//!
//! ## ここに在るもの / 無いもの
//!
//! - **Timeline の編集**。`--project` で実プロジェクトを開くと Timeline pane は
//!   Blitz テクスチャではなく native エディタ(`timeline_editor::TimelineEditor`)になり、
//!   移動・トリム・選択・Undo/Redo(Cmd+Z / Shift+Cmd+Z)が `ProjectSeat` の唯一の
//!   writer を通る。編集後の snapshot は同じフレームで Stage へも配り直される
//! - **入力(Timeline 以外)**。Blitz のペインはマウスを受けない。ポインタの
//!   振り分けは後続capsule(C2)。Browser / Inspector / chrome は固定サンプルのまま
//! - **レイアウトの永続化**は無い。起動するたび既定の並び

mod app;
mod pane;
mod runner;

pub(crate) use app::BlitzShellApp;
pub use app::ProjectSeat;
pub use pane::{BlitzPane, PaneKind};
pub use runner::{run_blitz_shell, BlitzShellLaunch, ScreenshotRequest, DEFAULT_SCREENSHOT_FRAMES};
