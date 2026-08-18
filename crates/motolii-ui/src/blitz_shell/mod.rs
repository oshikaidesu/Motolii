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
//! - **続きが開く**。座り直しが成立するたび、その path を user 設定層
//!   (`crate::last_project`。palette settings と同じ家・同じ保存方式)へ1本だけ
//!   覚える。次に `--project` 無しで起動すると**そこへ座り直してから窓が出る** —
//!   `--project` 起動と同じ座り方なので、journal の第1行も同じ `OpenProject` に
//!   なる。覚えていない(初回)はスタート画面、覚えていたのに開けない(消された・
//!   他プロセスが握っている)は**理由を帯に言ってから**スタート画面で、黙って
//!   fixture へは落ちない。覚えるのは最後の1本だけで、recent files の一覧も
//!   autosave も無い
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
//! - **原因の言い場所も1つ**。利用者の操作(New / Open / Save / Export 開始・
//!   キャンセル / 素材の取り込み)は型付きの `intent::UiIntent` になって
//!   `intent::IntentJournal` へ**行動の前に**載り、`intent::ShellGateway` だけが
//!   実行する。`--intent-log <path>` を付ければ全文が JSONL
//!   (`{"seq":n,"intent":{"kind":"…",…}}`)で追記され、その列は
//!   `ShellGateway::replay` に食わせればそのまま再現する。ゲートウェイを迂回して
//!   製品状態へ着く道はフェンス(`tests/shell_intent_gateway_fence.rs`)が塞ぐ。
//!   Timeline エディタの中の編集と Undo/Redo はまだ通っていない(wave E。あちらは
//!   `motolii-doc` の D2 Command journal が受けている)
//! - **失敗の言い場所は1つ**。窓の一言も面(pane)の失敗も `drive::ShellTranscript` を
//!   通り、帯には最新の1行が出て、`--status-log <path>` を付ければ全文が JSONL
//!   (`{"seq":n,"text":"…"}`)で追記される。`eprintln!` で消える失敗は無い
//!   (フェンス: `tests/shell_error_fence.rs`)。New / Open / Export / 未保存確認の
//!   dialog も `drive::ShellPrompts` の後ろに集約してあり、窓は `NativePrompts`(rfd)、
//!   テスト・CLI 駆動は台本(`ScriptedPrompts`)が答える

mod app;
/// 運転席(transcript / prompts / headless 駆動)。**module は公開しない** — U0a の
/// 境界規律で、egui / kittest の型(`DrivenShell`)を公開APIへ出さないため。
/// 出るのは下の `pub use` が名指しする `ShellTranscript` / `StatusEvent` の2つだけで、
/// どちらも toolkit の型を1つも持たない(2026-08-18 iced ホスト移行 M-0)。
mod drive;
#[cfg(test)]
mod drive_tests;
/// **原因のログとその唯一の実行口**(2026-08-18裁定「ログと構造の強制」の第1弾)。
/// 利用者の操作は `UiIntent` になって journal に載り、`ShellGateway` だけが実行する。
/// journal を通らずに製品状態へ着く道は `tests/shell_intent_gateway_fence.rs` が塞ぐ。
mod intent;
mod pane;
mod runner;

pub(crate) use app::BlitzShellApp;
pub use app::{decide_unsaved, UnsavedChoice, UnsavedDecision};
/// **結果のログ**と**訊き方**。host 非依存の契約なので、egui shell と iced shell が
/// 同じ型・同じ文言を通す(2026-08-18 iced ホスト移行 M-0 / M-1)。
pub use drive::{NativePrompts, ScriptedPrompts, ShellPrompts, ShellTranscript, StatusEvent};
pub use intent::{
    admit_dropped_paths, create_project_file, reseat_project, resume_last_project,
    seconds_to_us, IntentEvent, IntentJournal, ProjectSeat, Resume, ShellGateway, UiEditParam,
    UiIntent, UiItemFlag, UiKeyInterp,
};
pub use pane::{BlitzPane, PaneKind};
pub use runner::{run_blitz_shell, BlitzShellLaunch, ScreenshotRequest, DEFAULT_SCREENSHOT_FRAMES};
