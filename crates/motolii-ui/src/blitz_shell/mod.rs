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
//! - **ファイルの入口が2つ**。Finder から窓へ動画/音声を落とすと probe → import →
//!   **playhead の位置**へ clip が立つ(`admit_dropped_paths`)。`Cmd+N` で新規
//!   project を作って開き、`Cmd+O` で開き直す(`create_project_file` /
//!   `reseat_project`)。つまり `--project` 無しで起動しても後から project を持てる。
//!   probe できないファイルは理由つきで飛ばし、黙って捨てない
//! - **入力(Timeline 以外)**。Blitz のペインはマウスを受けない。ポインタの
//!   振り分けは後続capsule(C2)。Browser / Inspector / chrome は固定サンプルのまま
//! - **保存は無い**。ドロップも編集も writer の中だけで、project ファイルへは
//!   書き戻らない(`Cmd+S` はまだ無い)。座席を差し替えるとその編集は消える
//! - **レイアウトの永続化**は無い。起動するたび既定の並び

mod app;
mod pane;
mod runner;

pub(crate) use app::BlitzShellApp;
pub use app::{admit_dropped_paths, create_project_file, reseat_project, ProjectSeat};
pub use pane::{BlitzPane, PaneKind};
pub use runner::{run_blitz_shell, BlitzShellLaunch, ScreenshotRequest, DEFAULT_SCREENSHOT_FRAMES};
