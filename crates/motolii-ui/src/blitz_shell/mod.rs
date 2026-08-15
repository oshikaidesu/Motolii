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
//! - **Document接続**。Timeline以外は固定サンプルを描く。Timelineも参照Documentを読むだけで、
//!   編集は1つも通らない
//! - **Stage**。Blitzパネルが無いので席も置かない。**代わりの絵を描かない**
//! - **レイアウトの永続化**。起動するたび既定の並び
//!
//! つまりこれは「合わさった絵を見る」ための実物であって、製品の殻ではない。

mod app;
mod pane;

pub use app::BlitzShellApp;
pub use pane::{BlitzPane, PaneKind};
