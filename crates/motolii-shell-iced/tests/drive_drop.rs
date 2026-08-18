//! OS ドロップ — red 先行。
//!
//! 台本 P3 の第1歩「起動 → 何も読まずに動画をドロップ」を iced 殻でも通す。
//! 経路は egui shell と**同じ1本**(`UiIntent::AdmitPaths` → `admit_dropped_paths`)で、
//! 帯の文言も同じ物をそのまま映す。
//!
//! ## iced と egui の唯一の差: 事象の粒度
//!
//! winit は**ファイル1つにつき1事象**(`window::Event::FileDropped`)を出す。
//! egui は1フレームぶんの `dropped_files` をまとめて渡してくるので、そのままだと
//! 「3本落として3回の取り込み」と「3本落として1回の取り込み」で帯の言い方が
//! 変わってしまう。そこで殻は**フレームの区切り**(`RedrawRequested`)まで path を
//! 溜めてから1つの `AdmitPaths` にする — 言われることが host で揺れない。

mod common;

use common::{drain, feed, file_dropped, press, redraw, starter_media_dir};
use motolii_shell_iced::{view, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::UiIntent;

/// project が無いところへ落ちた素材は、**黙って消えず**に「先に作る」と案内される
/// (台本 P3 の1行目)。
#[test]
fn a_drop_without_a_project_says_to_make_one_first() {
    let clip = starter_media_dir().join("starter-clip.mp4");
    let mut shell = Shell::new(ScriptedPrompts::default());

    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), redraw()],
    );
    drain(&mut shell, dropped);

    assert_eq!(
        shell.latest_report().as_deref(),
        Some("open a project first — Cmd+N to create one, Cmd+O to open one"),
        "案内の文言は egui shell と同じでなければならない"
    );
    assert_eq!(
        shell
            .intents()
            .into_iter()
            .map(|event| event.intent)
            .collect::<Vec<_>>(),
        vec![UiIntent::AdmitPaths { paths: vec![clip] }],
        "案内で終わった取り込みも『やろうとしたこと』として journal に残る"
    );

    // 帯は同じフレームで案内を映す。**絵まで見て**言う。
    let mut after = iced_test::simulator(view(&shell));
    let latest = shell.latest_report().expect("帯に出る一言がある");
    assert!(
        after.find(latest.as_str()).is_ok(),
        "案内が帯に出ていない。実際: {latest:?}"
    );
}

/// 座席がある窓へ落ちた動画は playhead へ立ち、帯が名指しで言う。
#[test]
fn a_dropped_clip_lands_at_the_playhead_and_the_band_names_it() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_drop_place");
    let project = dir.join("fresh.json");
    let clip = starter_media_dir().join("starter-clip.mp4");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    assert_eq!(shell.track_item_count(), 0, "新規 project に clip はまだ無い");
    let revision_before = shell.revision();

    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), redraw()],
    );
    drain(&mut shell, dropped);

    let report = shell.latest_report().unwrap_or_default();
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

/// **1回のドロップは1回の取り込み。** 2本同時に落ちても intent は1件で、
/// 帯の一言も1つ(egui shell と同じ言い方)。
#[test]
fn two_files_dropped_together_are_one_admission() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_drop_batch");
    let project = dir.join("fresh.json");
    let clip = starter_media_dir().join("starter-clip.mp4");
    let still = starter_media_dir().join("starter-still.png");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    let said_before = shell.reports().len();

    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), file_dropped(&still), redraw()],
    );
    drain(&mut shell, dropped);

    let admissions: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .filter(|intent| matches!(intent, UiIntent::AdmitPaths { .. }))
        .collect();
    assert_eq!(
        admissions,
        vec![UiIntent::AdmitPaths {
            paths: vec![clip, still]
        }],
        "同じフレームに落ちた2本は1件の AdmitPaths にまとまる"
    );
    assert_eq!(
        shell.reports().len() - said_before,
        1,
        "1回のドロップに帯の一言も1つ。実際: {:?}",
        shell.reports()
    );
    assert_eq!(shell.track_item_count(), 2, "2本とも立っていない");
}

/// 取り込めない file は**理由つきで**帯に出て、Document を動かさない。
#[test]
fn a_drop_we_cannot_admit_says_why_and_leaves_the_document_alone() {
    let dir = motolii_testkit::tmp_dir("iced_drop_skip");
    let project = dir.join("fresh.json");
    let notes = dir.join("notes.txt");
    std::fs::write(&notes, "not a media file").expect("メモを書く");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);

    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&notes), redraw()],
    );
    drain(&mut shell, dropped);

    let report = shell.latest_report().unwrap_or_default();
    assert!(
        report.contains("notes.txt"),
        "入らなかった file は名指しで言う(無反応にしない)。実際: {report:?}"
    );
    assert_eq!(
        shell.track_item_count(),
        0,
        "入らなかったなら Document は動かない"
    );
}
