//! 運転席 — AUTOSAVE(SET+ B12 第2切片の shell 結線、2026-08-22)。
//!
//! `motolii-settings-pane` 側(AS レーン)が `AutoSaveConfig`/
//! `Document::auto_save` を着地させ、この結線レーンが `Message::AutoSaveTick`
//! (`auto_save::tick_subscription` の受け口、`Shell::run_auto_save`)を配線
//! した。ここでは実タイマー(`auto_save::tick_subscription`、OS スレッド駆動)
//! は一切叩かない — `Message::AutoSaveTick` を直接 `shell.update()` へ渡す
//! (`playback_drive.rs`が`Message::PlaybackTick`を直接叩くのと同じ「tick購読
//! そのものはテストしない、受け口だけを見る」手口)。
//!
//! 発注書の3点をそのまま柵にする: 有効/無効・dirty でない時のスキップ・
//! 再生中はスキップ(正典 §2 拘束5と同型)。実書き込みの oracle は
//! `motolii_store::Document::auto_save_dir` — スキップされた場合はこの
//! ディレクトリ自体が作られない(`Document::auto_save` 本体がdirty判定より
//! 後でしか `create_dir_all` しないため)。

use std::path::PathBuf;
use std::sync::Arc;

use motolii_audio::{DeviceWaitLatency, PlaybackClock, PlaybackCounters, PlaybackSession};
use motolii_core::{Fps, RationalTime};
use motolii_shell::file_dialogs::FileDialogs;
use motolii_shell::settings_pane::sections;
use motolii_shell::{Message, Shell};
use motolii_store::Document;

// ---------------------------------------------------------------------------
// fake FileDialogs(`file_drive.rs` と同じ理由 — Save As の実 OS dialog を
// 開かせない。ここでは `pick_save_path` の缶詰応答だけが要る)。
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FakeDialogs {
    save_path: PathBuf,
}

impl FileDialogs for FakeDialogs {
    fn confirm_discard(&self) -> bool {
        true
    }

    fn pick_save_path(&self) -> Option<PathBuf> {
        Some(self.save_path.clone())
    }

    fn quit(&self) {}
}

/// Save As 済み(`current_path` が実在)の Shell を組む。`AddLayer` を1回
/// 挟んで最初の Save As 自体が「意味のある保存」になるようにしてから、
/// **もう1回 `AddLayer`** して Save As 後にさらに編集(= dirty かつ
/// `last_auto_saved` と revision がずれた状態)を作る — AUTOSAVE tick の
/// 本命条件(発注書「dirty」)を満たす下ごしらえ。
///
/// **`tag` は呼び出し側ごとに違う文字列を渡すこと**: `motolii_testkit::
/// tmp_dir` はプロセス id だけを分岐に使う(`{tag}-{process::id()}`)ので、
/// 同じ `tag` を複数 test 関数から呼ぶと**同一プロセス内で並列に走る他の
/// test と同じ `project.motolii`/自動保存 dir を取り合ってしまう**
/// (`cargo test` は既定でスレッド並列、`file_drive.rs` が test ごとに違う
/// tag を使っているのと同じ理由)。
fn shell_saved_and_dirty(tag: &str) -> (Shell, PathBuf) {
    let dir = motolii_testkit::tmp_dir(tag);
    let path = dir.join("project.motolii");
    let (mut shell, _task) =
        Shell::new_with_dialogs(Box::new(FakeDialogs { save_path: path.clone() }));

    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::SaveAsRequested);
    assert_eq!(shell.current_path(), Some(path.as_path()), "Save As が current_path を立てていない");
    assert!(!shell.is_project_dirty(), "Save As 直後なのに dirty のまま");

    let _ = shell.update(Message::AddLayer);
    assert!(shell.is_project_dirty(), "2本目の AddLayer 後なのに dirty になっていない");

    (shell, path)
}

fn fixture_fps() -> Fps {
    Fps::try_new(30, 1).expect("30fps")
}

/// `playback_drive.rs::fake_session_at` と同じ手口(実 cpal/producer を開かない
/// フェイク再生セッション)。ここでは「再生中である」状態を作れれば十分なので
/// 供給の手動進行は使わない。
fn fake_playback_session() -> PlaybackSession {
    let counters = Arc::new(PlaybackCounters::default());
    let wait = Arc::new(DeviceWaitLatency::default());
    let mut clock =
        PlaybackClock::new(counters, wait, 48_000).expect("48kHz は有効な sample_rate");
    let at = RationalTime::try_from_frame(0, fixture_fps()).expect("0 frame は有効");
    clock.start(at);
    PlaybackSession::for_simulation(clock)
}

// ---------------------------------------------------------------------------
// (a) 本命 — 有効・dirty・非再生中なら実際に自動保存する
// ---------------------------------------------------------------------------

#[test]
fn auto_save_tick_writes_a_generation_file_when_enabled_and_dirty() {
    let (mut shell, path) = shell_saved_and_dirty("auto-save-drive-writes");
    let auto_save_dir = Document::auto_save_dir(&path);
    assert!(!auto_save_dir.exists(), "tick 前から自動保存 dir がある(下ごしらえが汚れている)");

    let _ = shell.update(Message::AutoSaveTick);

    assert!(
        auto_save_dir.is_dir(),
        "有効・dirty・非再生中の tick で自動保存 dir が作られていない"
    );
    let entries: Vec<_> = std::fs::read_dir(&auto_save_dir)
        .expect("自動保存 dir を読める")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1, "自動保存 dir の中身が1世代でない: {entries:?}");
    assert!(
        shell.status().is_some_and(|s| s.contains("自動保存")),
        "自動保存の一行が status 帯へ出ていない(M13 — 無反応ゼロ): {:?}",
        shell.status()
    );
}

// ---------------------------------------------------------------------------
// (b) 無効なら tick が来ても何もしない
// ---------------------------------------------------------------------------

#[test]
fn auto_save_tick_is_a_no_op_when_disabled() {
    let (mut shell, path) = shell_saved_and_dirty("auto-save-drive-disabled");
    let auto_save_dir = Document::auto_save_dir(&path);

    let _ = shell.update(Message::Settings(sections::Message::AutoSaveToggle(false)));
    let _ = shell.update(Message::AutoSaveTick);

    assert!(
        !auto_save_dir.exists(),
        "auto_save_enabled=false なのに自動保存 dir が作られた"
    );
}

// ---------------------------------------------------------------------------
// (c) dirty でなければ tick は no-op(revision が動いていない)
// ---------------------------------------------------------------------------

#[test]
fn auto_save_tick_is_a_no_op_when_not_dirty_since_the_last_auto_save() {
    let dir = motolii_testkit::tmp_dir("auto-save-drive-clean");
    let path = dir.join("project.motolii");
    let (mut shell, _task) =
        Shell::new_with_dialogs(Box::new(FakeDialogs { save_path: path.clone() }));
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::SaveAsRequested);
    assert!(!shell.is_project_dirty(), "Save As 直後なのに dirty のまま");

    let auto_save_dir = Document::auto_save_dir(&path);
    let _ = shell.update(Message::AutoSaveTick);

    assert!(
        !auto_save_dir.exists(),
        "Save As 直後(未編集)の tick で自動保存 dir が作られた — dirty 判定が効いていない"
    );
    assert!(shell.status().is_none(), "何もしていないのに status が出た(無反応ゼロの逆 — 雑音): {:?}", shell.status());
}

// ---------------------------------------------------------------------------
// (d) 再生中はスキップ(正典 §2 拘束5と同型)
// ---------------------------------------------------------------------------

#[test]
fn auto_save_tick_skips_while_playing() {
    let (mut shell, path) = shell_saved_and_dirty("auto-save-drive-playing");
    let auto_save_dir = Document::auto_save_dir(&path);

    shell.debug_start_playback_with_session(fake_playback_session());
    assert!(shell.is_playing(), "セッション採用直後は再生中のはず");

    let _ = shell.update(Message::AutoSaveTick);

    assert!(
        !auto_save_dir.exists(),
        "再生中なのに自動保存 dir が作られた(正典 §2 拘束5違反)"
    );
}
