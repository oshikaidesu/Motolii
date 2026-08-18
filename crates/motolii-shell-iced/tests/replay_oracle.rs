//! replay oracle(常設)— red 先行。
//!
//! egui 版 `blitz_shell/drive_tests.rs` の
//! `a_recorded_session_replays_into_the_same_shell_state` の対応物である。
//! 駆動器だけが違い(kittest → `iced_test::Simulator`)、**審判は同じ型の同じ問い**:
//! 記録した intent 列を窓の無い新しい `ShellGateway` へ再実行すると、座席・
//! revision・track item 数・言われたことが一致するか。
//!
//! 前提は「初期状態が同じ」ことなので、駆動が作った project ファイルは消してから
//! replay する(`NewProject` は既にあるファイルを踏まないのが決定済みの意味であり、
//! その意味ごと再現するために world を初期状態へ戻す)。

mod common;

use common::{command_key, drain, feed, file_dropped, press, redraw, starter_media_dir};
use motolii_shell_iced::{view, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{ShellGateway, UiIntent};

#[test]
fn a_recorded_session_replays_into_the_same_shell_state() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_intent_replay");
    let project = dir.join("fresh.json");
    let clip = starter_media_dir().join("starter-clip.mp4");
    let still = starter_media_dir().join("starter-still.png");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });

    // 運転席で1セッション回す: 作る → 落とす → もう1本落とす → 保存する。
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), redraw()],
    );
    drain(&mut shell, dropped);
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&still), redraw()],
    );
    drain(&mut shell, dropped);
    let typed = feed(iced_test::simulator(view(&shell)), command_key('s'));
    drain(&mut shell, typed);

    let intents: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .collect();
    let driven_path = shell.project_path();
    let driven_revision = shell.revision();
    let driven_items = shell.track_item_count();
    let driven_reports = shell.reports();
    assert_eq!(
        intents,
        vec![
            UiIntent::NewProject {
                path: project.clone()
            },
            UiIntent::AdmitPaths {
                paths: vec![clip.clone()]
            },
            UiIntent::AdmitPaths {
                paths: vec![still.clone()]
            },
            UiIntent::SaveProject,
        ],
        "運転席の1セッションが intent 列として素直に読めない"
    );
    assert!(driven_items > 0, "replay で比べる中身がまず要る");

    // 座席は project ファイルの OS lock を握っている。replay が同じ project を
    // 開けるよう、先に殻ごと落とす(lock を返す)。
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
    // **同じ順で**現れる。
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
