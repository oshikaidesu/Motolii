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
//! ## ここに無いもの
//!
//! - **入力**。ペインはマウスを受けない。ポインタの振り分けは後続capsule(C2)
//! - **編集**。`--project` で実プロジェクトを開くと Timeline / Stage はその Document
//!   (`ProjectSeat` の writer snapshot)を映すが、編集は1つも通らない(次レーン)。
//!   Browser / Inspector / chrome は固定サンプルのまま
//! - **レイアウトの永続化**。起動するたび既定の並び
//!
//! つまりこれは「合わさった絵を見る」ための実物であって、製品の殻ではない。

mod app;
mod pane;

pub use app::{BlitzShellApp, ProjectSeat};
pub use pane::{BlitzPane, PaneKind};
