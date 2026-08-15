//! Blitz(HTML/CSS)によるUI面。
//!
//! 起案 `docs/reviews/2026-08-15-blitz-ui-runtime-adoption-proposal.md` の構成:
//!   ホスト窓とwgpuデバイスは eframe が持ち、Stage は Rerun、
//!   UI面(Timeline / Inspector / Browser)は Blitz が自前テクスチャへ描く。
//!   入力ルーティングは Motolii 側が持つ。
//!
//! 各面は [`surface::BlitzSurface`] を1つ持つ。意味は既存実装から**写す**もので、
//! ここで新設しない(色・寸法は `timeline_egui/theme.rs`、操作意味は
//! `timeline_egui/clip_band.rs` と `timeline_skia/hit.rs`、役割は
//! `docs/ui-interaction-language.md`)。

pub(crate) mod surface;

pub(crate) use surface::{BlitzSurface, SurfacePointer};
