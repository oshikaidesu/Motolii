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
