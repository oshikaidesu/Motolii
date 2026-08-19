//! iced の運転席 第1号 — 窓を開かずに押して、読む。
//!
//! egui shell の `blitz_shell/drive_tests.rs` 最初の2テスト
//! (`every_report_lands_in_the_transcript_and_the_band_shows_the_latest` と
//! `the_start_screen_seats_a_project_without_a_native_dialog`)の iced 版である。
//! 合否の言い方は**一字も変えていない** — 変わったのは駆動器だけ:
//!
//! | | egui 版 | iced 版 |
//! |---|---|---|
//! | 駆動 | `egui_kittest::Harness` | `iced_test::Simulator` |
//! | 探す | AccessKit ラベル(部分一致) | `Selector`(`&str` は**完全一致**) |
//! | GPU | 無ければ skip(`DrivenShell::seatless` が `None`) | 要らない(無ければ tiny-skia へ落ちる) |
//! | 読む | `ShellTranscript` | `ShellTranscript`(**同じ型**) |
//!
//! 2026-08-18 の裁定が「段差ゼロが公式不変量」と言ったのはこの差である。
//! kittest は自前で繋いだ第2層だが、`iced_test` は iced 本体が持っている。

mod common;

use common::{drain, press};
use motolii_shell_iced::{view, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::ShellTranscript;

/// 言う場所は1つ: report された全文が残り、帯は最新を映す。
///
/// **egui 版と同じ型を触っている。** transcript は host 非依存の契約なので、
/// このテストは移行のどちら側でも同じ文で書ける。
#[test]
fn every_report_lands_in_the_transcript_and_the_band_shows_the_latest() {
    let transcript = ShellTranscript::default();
    transcript.report("opened a.json");
    transcript.report("save failed: disk full");

    let entries = transcript.entries();
    assert_eq!(entries.len(), 2, "report が transcript から欠けてはならない");
    assert_eq!(entries[0].text, "opened a.json");
    assert_eq!(entries[1].text, "save failed: disk full");
    assert_eq!(
        transcript.latest().as_deref(),
        Some("save failed: disk full"),
        "帯(最新表示)は最後の report を映す"
    );
}

/// スタート画面 → New が、native dialog を開かずに台本のパスへ座席を作る。
/// 失敗も成功も transcript に言われる。
#[test]
fn the_start_screen_seats_a_project_without_a_native_dialog() {
    let dir = motolii_testkit::tmp_dir("iced_drive_seat_new");
    let project = dir.join("fresh.json");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });

    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(motolii_shell_iced::view::NEW_PROJECT).is_ok(),
        "スタート画面に New Project が無い"
    );
    assert!(
        ui.find(motolii_shell_iced::view::OPEN_PROJECT).is_ok(),
        "スタート画面に Open が無い"
    );

    let pressed = press(ui, motolii_shell_iced::view::NEW_PROJECT);
    drain(&mut shell, pressed);

    assert!(
        shell.is_seated(),
        "New は台本のパスへ座席を作る(dialog 待ちで止まらない)"
    );
    assert!(
        shell
            .latest_report()
            .is_some_and(|report| report.contains("opened")),
        "座席の成立は transcript に言われる。実際: {:?}",
        shell.latest_report()
    );
    assert!(project.exists(), "project ファイルが実際に作られる");

    // 帯は同じフレームで最新を映す。**絵まで見て**言う
    // (「言ったつもりで出ていない」を通さない)。
    let mut after = iced_test::simulator(view(&shell));
    let latest = shell.latest_report().expect("帯に出る一言がある");
    assert!(
        after.find(latest.as_str()).is_ok(),
        "status 帯が最新の一言を映していない。実際: {latest:?}"
    );

    // legend 行の文法(2026-08-19 トンマナ統一 campaign レーンB): 左 = 最新の
    // 一言、右 = 効くキーのヒント。手本
    // (`docs/mocks-ui/public/timeline-library.html` の footer)と同じ
    // 「左=状況・右=ヒント」を、絵の実座標(bounds)まで見て確かめる —
    // 「同じ帯に両方出ている」だけでは「左右が入れ替わっていない」を
    // 保証しないため。
    let status = after
        .find(latest.as_str())
        .expect("最新の一言の bounds が取れる");
    let hint_text = motolii_shell_iced::legend_line();
    let hint = after
        .find(hint_text.as_str())
        .expect("近道キーのヒントが legend 行に出ていない");
    assert!(
        status.bounds().x < hint.bounds().x,
        "最新の一言(左)がヒント(右)より右に出ている。status.x={} hint.x={}",
        status.bounds().x,
        hint.bounds().x
    );
}

/// ヘッダ帯(2026-08-19 campaign 第2波レーンE)— Undo/Redo/Export は
/// window 最上部、Timeline pane より**上**に居る。利用者裁定「Undo とか
/// エクスポートはヘッダに出て欲しい」の絵での確認。
/// 座席前(スタート画面)にヘッダは無い(座席と一緒に来る)。
#[test]
fn undo_and_export_live_in_the_header_above_the_timeline() {
    let dir = motolii_testkit::tmp_dir("iced_drive_seat_header");
    let project = dir.join("fresh.json");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });

    // 座席前: スタート画面にはヘッダ(Undo/Export)が無い。
    let mut before = iced_test::simulator(view(&shell));
    assert!(
        before.find(motolii_shell_iced::view::UNDO).is_err(),
        "座席の無いスタート画面に Undo が出てはいけない"
    );
    drop(before); // `Simulator` は `&shell` を借りたまま Drop を持つ — 次の可変借用の前に手放す。

    let pressed = press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::NEW_PROJECT,
    );
    drain(&mut shell, pressed);
    assert!(shell.is_seated(), "前提: New で座席が要る");

    let mut ui = iced_test::simulator(view(&shell));
    let undo = ui
        .find(motolii_shell_iced::view::UNDO)
        .expect("座席後のヘッダに Undo が無い");
    let export = ui
        .find(motolii_shell_iced::view::EXPORT)
        .expect("座席後のヘッダに Export が無い");
    let timeline = ui
        .find(motolii_shell_iced::view::TIMELINE_PANE_ID)
        .expect("Timeline pane が座席上に立っていない");

    assert!(
        undo.bounds().y < timeline.bounds().y,
        "Undo は Timeline より上(ヘッダ)に居るはず。undo.y={} timeline.y={}",
        undo.bounds().y,
        timeline.bounds().y
    );
    assert!(
        export.bounds().y < timeline.bounds().y,
        "Export は Timeline より上(ヘッダ)に居るはず。export.y={} timeline.y={}",
        export.bounds().y,
        timeline.bounds().y
    );
}

/// 押したことは**行動の前に** journal へ載り、`--intent-log` 相当の JSONL は
/// egui shell と同じ形の行になる。
#[test]
fn the_press_lands_in_the_journal_as_a_replayable_line() {
    let dir = motolii_testkit::tmp_dir("iced_drive_seat_journal");
    let project = dir.join("fresh.json");
    let log_path = dir.join("intents.jsonl");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });
    let mut log = motolii_shell_iced::IntentLog::create(&log_path).expect("intent log を開ける");

    let pressed = press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::NEW_PROJECT,
    );
    drain(&mut shell, pressed);
    log.flush(&shell);

    let entries = shell.intents();
    assert_eq!(entries.len(), 1, "押した1回が journal の1行になる");
    assert_eq!(entries[0].seq, 1);
    assert_eq!(
        entries[0].intent,
        motolii_ui::blitz_shell::UiIntent::NewProject {
            path: project.clone()
        },
        "dialog の答えは intent の中に値として入る(replay が訊き直さないため)"
    );

    let written = std::fs::read_to_string(&log_path).expect("intent log が書けている");
    let line = written.lines().next().expect("1行は在る");
    assert!(
        line.starts_with(r#"{"seq":1,"intent":{"kind":"new_project""#),
        "行の形は egui shell の --intent-log と同型でなければならない。実際: {line}"
    );
}

/// 答えない dialog は**何も起こさない**。journal にも載らない
/// (intent は起こした行動の記録であって、失敗の記録ではない)。
#[test]
fn a_cancelled_dialog_leaves_no_trace() {
    let mut shell = Shell::new(ScriptedPrompts::default());

    let pressed = press(
        iced_test::simulator(view(&shell)),
        motolii_shell_iced::view::NEW_PROJECT,
    );
    drain(&mut shell, pressed);

    assert!(!shell.is_seated(), "答えなかった dialog で座ってはならない");
    assert_eq!(shell.intent_count(), 0, "起きなかった行動は journal に載らない");
    assert_eq!(shell.latest_report(), None, "帯も黙ったまま");
}
