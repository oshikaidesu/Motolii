//! 運転席(driver seat)の合格条件 — red 先行。
//!
//! CLI・テストから GUI を**決定的に**駆動・観測できる座席の契約。ここが通るまで
//! 「窓の検証は人が画素を見るしかない」状態が続く。2026-08-16 の Timeline 再選定で
//! 決めた開発動線の第2層(`egui_kittest 0.35`)を shell 全体へ広げる実行部。
//!
//! 契約(このテストが定義する名前が正):
//! - `super::drive::ShellTranscript` — 窓の一言(status)の唯一の言い場所。
//!   `report()` された全文が順に残り、帯は `latest()` を映す。何も黙って消えない。
//! - `super::drive::ScriptedPrompts` — rfd dialog の台本版。`Default` は全て
//!   「答えない」。窓の外(テスト・CLI)からの駆動では native dialog を一切開かない。
//! - `super::drive::DrivenShell` — egui_kittest で `BlitzShellApp` を回す運転席。
//!   `seatless()` はスタート画面から始める。GPU が無い環境は `None`(skip、
//!   `motolii_testkit::gpu_or_skip` と同じポリシー)。
//!
//! kittest の接続詳細(RenderState の作り方等)は `drive` module の内側に閉じる。
//! ここでは AccessKit ラベルと transcript だけで合否を言う。

use std::path::{Path, PathBuf};

use super::drive::{DrivenShell, ScriptedPrompts, ShellTranscript};
use super::intent::{ShellGateway, UiIntent};

/// repo に入っている実 media(starter kit)。動画・画像・音声が1本ずつ在る。
/// Browser をここへ座らせると、窓を開かずにカードを触る道が試せる。
fn starter_media_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/mocks-ui/starter-media/media")
        .canonicalize()
        .expect("starter media lives in the repo")
}

/// 言う場所は1つ: report された全文が残り、帯は最新を映す。
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
    let dir = motolii_testkit::tmp_dir("drive_seat_new");
    let project = dir.join("fresh.json");
    let prompts = ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless(prompts) else {
        return; // GPU 無し: gpu_or_skip と同じスキップポリシー
    };

    assert!(
        shell.label_visible("New Project"),
        "スタート画面に New Project が無い"
    );
    assert!(shell.label_visible("Open"), "スタート画面に Open が無い");

    shell.click_label_containing("New Project");
    shell.run_frames(2);

    assert!(
        shell.seated(),
        "New は台本のパスへ座席を作る(dialog 待ちで止まらない)"
    );
    assert!(
        shell.latest_report().contains("opened"),
        "座席の成立は transcript に言われる。実際: {:?}",
        shell.latest_report()
    );
    assert!(project.exists(), "project ファイルが実際に作られる");
}

/// probe 不能な file のドロップは、理由が file 名ごと帯に言われる(黙って消えない)。
/// docs/ux-check-first-ten-minutes.md P5 の「理由つきで skip」の機械版。
#[test]
fn an_unprobeable_drop_reports_its_reason_with_the_file_name() {
    let dir = motolii_testkit::tmp_dir("drive_drop_skip");
    let project = dir.join("fresh.json");
    let notes = dir.join("notes.txt");
    std::fs::write(&notes, "not a media file").unwrap();

    let prompts = ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless(prompts) else {
        return;
    };
    shell.click_label_containing("New Project");
    shell.run_frames(2);
    assert!(shell.seated());

    shell.drop_file(&notes);
    shell.run_frames(2);

    let report = shell.latest_report();
    assert!(
        report.contains("notes.txt"),
        "skip は file 名を名指しで言う。実際: {report:?}"
    );
}

/// Browser のカードのダブルクリックが、**ドロップと同じ一本の経路**で
/// playhead へ clip を立てる。
///
/// 2026-08-18 の実機一撃(`docs/reviews/2026-08-18-user-first-touch-observations.md`
/// 原因(1))で「押しても何も起きない」と分かった所。合否は帯の一言だけでなく
/// **Document に item が増えたこと**まで見る(言うだけで置かない、を通さない)。
#[test]
fn double_clicking_a_browser_card_places_that_file_at_the_playhead() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("drive_browser_place");
    let project = dir.join("fresh.json");
    let prompts = ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless_browsing(prompts, &starter_media_dir()) else {
        return; // GPU 無し: gpu_or_skip と同じスキップポリシー
    };
    shell.click_label_containing("New Project");
    shell.run_frames(2);
    assert!(shell.seated());
    assert_eq!(
        shell.track_item_count(),
        0,
        "新規 project に clip はまだ無い"
    );
    let revision_before = shell.revision();

    assert!(
        shell.label_visible("starter-clip.mp4"),
        "Browser のカードが名乗っていない(触れる物として木に出ていない)"
    );
    shell.double_click_label_containing("starter-clip.mp4");
    shell.run_frames(2);

    let report = shell.latest_report();
    assert!(
        report.contains("placed") && report.contains("starter-clip.mp4"),
        "配置の成立は帯が名指しで言う。実際: {report:?}"
    );
    assert_eq!(
        shell.track_item_count(),
        1,
        "帯が言うだけで Document が動かないのは不合格"
    );
    assert!(
        shell.revision() > revision_before,
        "配置は writer を通る(revision が進む)"
    );
}

/// **静止画のダブルクリックも clip を立てる。**
///
/// このテストは元々「画像 admission はまだ開いていない」を固定していた
/// (レーンB が置いた合図)。2026-08-18 レーンAで扉が開いたので、赤くなった所を
/// 新しい事実へ書き換えた — もう1つの入口も同じ一本の経路を通る。
#[test]
fn double_clicking_a_still_image_card_places_it_too() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("drive_browser_still");
    let project = dir.join("fresh.json");
    let prompts = ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless_browsing(prompts, &starter_media_dir()) else {
        return;
    };
    shell.click_label_containing("New Project");
    shell.run_frames(2);
    assert!(shell.seated());

    shell.double_click_label_containing("starter-still.png");
    shell.run_frames(2);

    let report = shell.latest_report();
    assert!(
        report.contains("placed") && report.contains("starter-still.png"),
        "配置の成立は帯が名指しで言う。実際: {report:?}"
    );
    assert_eq!(
        shell.track_item_count(),
        1,
        "帯が言うだけで Document が動かないのは不合格"
    );
}

/// 入らない file のダブルクリックは、**理由つきで帯に出て** Document を動かさない。
///
/// SVG はラスタライザが要るので admission に載っていない(レーンAの NON-GOAL)。
/// 入らない物が黙って消えるのが一番分からないので、ここで固定する。
#[test]
fn double_clicking_a_card_we_cannot_admit_says_why_and_leaves_the_document_alone() {
    let dir = motolii_testkit::tmp_dir("drive_browser_skip");
    let project = dir.join("fresh.json");
    let prompts = ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless_browsing(prompts, &starter_media_dir()) else {
        return;
    };
    shell.click_label_containing("New Project");
    shell.run_frames(2);
    assert!(shell.seated());

    shell.double_click_label_containing("starter-mark.svg");
    shell.run_frames(2);

    let report = shell.latest_report();
    assert!(
        report.contains("starter-mark.svg"),
        "入らなかった file は名指しで言う(無反応にしない)。実際: {report:?}"
    );
    assert!(
        report.contains("skipped"),
        "入らなかったことが分かる語で言う。実際: {report:?}"
    );
    assert_eq!(
        shell.track_item_count(),
        0,
        "入らなかったなら Document は動かない"
    );
}

// ---------------------------------------------------------------------------
// ログと構造の強制(2026-08-18裁定)
// ---------------------------------------------------------------------------

/// **原因のログ**: 利用者が触った操作は、入口が違っても全部 `UiIntent` として
/// journal に載る。
///
/// ここが空なら「何が起きたか(transcript)」はあっても「なぜ起きたか」が無い =
/// replay を組めない。ドロップと Browser のダブルクリックは別の入口だが、
/// 合流点である `AdmitPaths` として**同じ形**で残る。
#[test]
fn every_shell_action_lands_in_the_intent_journal() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("drive_intent_journal");
    let project = dir.join("fresh.json");
    let still = starter_media_dir().join("starter-still.png");
    let prompts = ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless_browsing(prompts, &starter_media_dir()) else {
        return; // GPU 無し: gpu_or_skip と同じスキップポリシー
    };

    shell.click_label_containing("New Project");
    shell.run_frames(2);
    shell.double_click_label_containing("starter-clip.mp4");
    shell.run_frames(2);
    shell.drop_file(&still);
    shell.run_frames(2);

    let intents = shell.intents();
    assert_eq!(
        intents.len(),
        3,
        "New / ダブルクリック / ドロップ で 3 件。実際: {intents:#?}"
    );
    assert_eq!(
        intents[0],
        UiIntent::NewProject {
            path: project.clone()
        },
        "dialog の**答え**が intent の中に入る(replay が訊き直さないため)"
    );
    assert!(
        matches!(&intents[1], UiIntent::AdmitPaths { paths }
            if paths.len() == 1 && paths[0].ends_with("starter-clip.mp4")),
        "Browser のダブルクリックは AdmitPaths として残る。実際: {:?}",
        intents[1]
    );
    assert_eq!(
        intents[2],
        UiIntent::AdmitPaths { paths: vec![still] },
        "OS ドロップも同じ合流点の intent になる"
    );
}

/// **replay oracle(常設)**: 駆動セッションで記録した intent 列を、窓を持たない
/// 新しい shell へ再実行すると、座席・revision・track item 数・言われたことが一致する。
///
/// iced の time-travel に相当する検証をこれで自前に持つ。前提は「初期状態が同じ」
/// ことなので、駆動が作った project ファイルは消してから replay する
/// (`NewProject` は既にあるファイルを踏まないのが決定済みの意味であり、
/// その意味ごと再現するために world を初期状態へ戻す)。
#[test]
fn a_recorded_session_replays_into_the_same_shell_state() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("drive_intent_replay");
    let project = dir.join("fresh.json");
    let still = starter_media_dir().join("starter-still.png");
    let prompts = ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    };
    let Some(mut shell) = DrivenShell::seatless_browsing(prompts, &starter_media_dir()) else {
        return;
    };

    shell.click_label_containing("New Project");
    shell.run_frames(2);
    shell.double_click_label_containing("starter-clip.mp4");
    shell.run_frames(2);
    shell.drop_file(&still);
    shell.run_frames(2);

    let intents = shell.intents();
    let driven_path = shell.project_path();
    let driven_revision = shell.revision();
    let driven_items = shell.track_item_count();
    let driven_reports = shell.reports();
    assert!(driven_items > 0, "replay で比べる中身がまず要る");
    assert!(!intents.is_empty(), "記録が空なら replay は何も言えない");

    // 座席は project ファイルの OS lock を握っている。replay が同じ project を
    // 開けるよう、先に窓ごと落とす(lock を返す)。
    drop(shell);
    // world を初期状態へ戻す: NewProject が作ったファイルだけを消す。
    std::fs::remove_file(&project).expect("駆動が作った project を消して初期状態へ戻す");

    let replayed = ShellGateway::replay(&intents);

    assert_eq!(
        replayed.project().map(|seat| seat.path().to_path_buf()),
        driven_path,
        "replay は同じ project へ座り直す"
    );
    assert_eq!(
        replayed.revision(),
        driven_revision,
        "writer 世代が一致する(同じ command 列が通った)"
    );
    assert_eq!(
        replayed.track_item_count(),
        driven_items,
        "帯だけ揃って Document が違うのは不合格"
    );

    // 結果のログの要点: replay が言ったことは、駆動が言ったことの中に
    // **同じ順で**現れる(駆動側には面が言った一言も混ざるので部分列で見る)。
    let replayed_reports: Vec<String> = replayed
        .transcript()
        .entries()
        .into_iter()
        .map(|event| event.text)
        .collect();
    assert!(!replayed_reports.is_empty(), "replay も言うべきことを言う");
    let mut driven = driven_reports.iter();
    for line in &replayed_reports {
        assert!(
            driven.any(|seen| seen == line),
            "replay の一言 {line:?} が駆動セッションの台帳に同じ順で無い。\n\
             駆動: {driven_reports:#?}\nreplay: {replayed_reports:#?}"
        );
    }
}
