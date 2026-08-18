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
//! - **素材の入口が2つ、経路は1本**。Finder から窓へ動画/音声を落とすか、
//!   **Browser のカードをダブルクリック**すると、どちらも同じ `admit_dropped_paths`
//!   を通って probe → import → **playhead の位置**へ clip が立つ。Browser 側は
//!   パネルが `browser_panel::BrowserRequest` を返すだけで Document を書かず、
//!   流す先を決めるのは `app.rs` である(2つ目の import 経路を作らない)。
//!   probe できないファイルは理由つきで飛ばし、黙って捨てない
//! - **project の入口**。`Cmd+N` で新規 project を作って開き、`Cmd+O` で開き直す
//!   (`create_project_file` / `reseat_project`)。つまり `--project` 無しで
//!   起動しても後から project を持てる
//! - **入力**。Blitz のペイン(chrome の3枚)はマウスを受けない — ポインタの
//!   振り分けは後続capsule(C2)。Timeline / Stage / Browser / Inspector は
//!   ホストの egui へ直接描く native 面なので、素直にマウスが通る
//! - **保存と信用の可視化**。`Cmd+S` が writer snapshot を project ファイルへ
//!   書き戻す(`ProjectSeat::save`。経路は既存の `ProjectSession::save_document`)。
//!   下の status 帯は project が居るあいだ常設で、保存状態(未保存なら ● 付き)と
//!   Undo/Redo ボタン(Cmd+Z / Shift+Cmd+Z と同じ入口)を持つ。未保存のまま
//!   座席を捨てる操作(Cmd+O / Cmd+N / 窓を閉じる)は 保存 / 破棄 / キャンセル の
//!   確認を挟む(判断は `decide_unsaved`、dialog は `rfd` に集約)。
//!   自動保存と journal 常時追記は**しない**(明示保存のみ)
//! - **書き出し面**。status 帯の Export ボタン → 保存先 dialog(既定
//!   `{project名}.mp4`)→ **現 writer snapshot** から既存の
//!   `export_document_video` が**別 thread**で走る(headless GpuCtx。CLI と
//!   同じ形。dirty でも書き出せる)。実行中は「Exporting… {経過}s」+ Cancel が
//!   帯に出て UI は固まらず、完了/キャンセル/失敗は同じ帯の一言になる。
//!   キャンセルは部分出力を残さない。実行中は Export が消える = 二重起動なし。
//!   判断と thread は `crate::export_seat`、dialog は `rfd` に集約
//! - **レイアウトの永続化**は無い。起動するたび既定の並び
//! - **失敗の言い場所は1つ**。窓の一言も面(pane)の失敗も `drive::ShellTranscript` を
//!   通り、帯には最新の1行が出て、`--status-log <path>` を付ければ全文が JSONL
//!   (`{"seq":n,"text":"…"}`)で追記される。`eprintln!` で消える失敗は無い
//!   (フェンス: `tests/shell_error_fence.rs`)。New / Open / Export / 未保存確認の
//!   dialog も `drive::ShellPrompts` の後ろに集約してあり、窓は `NativePrompts`(rfd)、
//!   テスト・CLI 駆動は台本(`ScriptedPrompts`)が答える

mod app;
/// 運転席(transcript / prompts / headless 駆動)。**公開しない** — U0a の境界規律で、
/// egui / kittest の型を公開APIへ出さないため。外へ出るのは `--status-log` の JSONL だけ。
mod drive;
#[cfg(test)]
mod drive_tests;
mod pane;
mod runner;

pub(crate) use app::BlitzShellApp;
pub use app::{
    admit_dropped_paths, create_project_file, decide_unsaved, reseat_project, ProjectSeat,
    UnsavedChoice, UnsavedDecision,
};
pub use pane::{BlitzPane, PaneKind};
pub use runner::{run_blitz_shell, BlitzShellLaunch, ScreenshotRequest, DEFAULT_SCREENSHOT_FRAMES};
