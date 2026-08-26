//! 起動時の座席 — `--project` と引数なし起動の「続きが開く」— 赤先行。
//!
//! `crate::last_project`(egui 側 wave D)を**共用**する。ここでは2つ目の
//! 判断は書かない — `motolii_ui::blitz_shell::{resume_last_project, ProjectSeat}`
//! をそのまま呼ぶ `motolii_shell_iced::resume::decide_resume` の合流点だけを審判する。
//!
//! | | 覚え方 |
//! |---|---|
//! | `--project <path>` | 開けたら座る。開けなければ**理由を帯へ**残してスタート画面 |
//! | 引数なし | 覚えていればそこへ座り直す(F-01)。覚えていない/開けないはスタート画面 |
//!
//! どちらの「開けない」も**黙って fixture にも空の窓にも落ちない** — egui shell の
//! `resume_last_project` が既に持っている3択の言い方をそのまま流用する。

mod common;

use std::path::PathBuf;

use common::{drain, press};
use motolii_shell_iced::resume::decide_resume;
use motolii_shell_iced::{view, Launch, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{create_project_file, Resume};
use motolii_ui::remember_last_project;

fn launch_with_project(path: Option<PathBuf>) -> Launch {
    Launch {
        project: path,
        ..Launch::default()
    }
}

/// `--project` が開ければ、`decide_resume` はその座席をそのまま返す。
#[test]
fn the_project_flag_opens_and_seats() {
    let dir = motolii_testkit::tmp_dir("launch_resume_project_ok");
    let project = dir.join("launch.json");
    create_project_file(&project).expect("project を作る");

    let resume = decide_resume(&launch_with_project(Some(project.clone())), None);
    match resume {
        Resume::Seated(seat) => {
            assert_eq!(
                seat.path(),
                project.as_path(),
                "--project で開いた座席のパスが違う"
            );
        }
        _ => panic!("--project が開けるのに座っていない"),
    }
}

/// `--project` が開けなければ、理由付きでスタート画面へ回る(黙って落ちない)。
/// egui 版の起動失敗(`run_blitz_shell` の `Err`)とはここが違う — iced は
/// 窓を開いてから帯で言う。
#[test]
fn the_project_flag_that_cannot_open_explains_instead_of_seating() {
    let dir = motolii_testkit::tmp_dir("launch_resume_project_missing");
    let missing = dir.join("does-not-exist.json");

    let resume = decide_resume(&launch_with_project(Some(missing.clone())), None);
    match resume {
        Resume::Explained(reason) => {
            assert!(
                reason.contains("does-not-exist.json"),
                "開けなかった path を名指しで言う。実際: {reason:?}"
            );
        }
        Resume::Seated(_) => panic!("開けないはずの --project で座ってしまった"),
        Resume::Nothing => panic!("開けない --project は黙ってはならない"),
    }
}

/// 引数なし起動は、覚えていた project へ座り直す(F-01)。
#[test]
fn no_arguments_resumes_the_remembered_project() {
    let dir = motolii_testkit::tmp_dir("launch_resume_no_args_remembered");
    let store = dir.join("last-project.json");
    let project = dir.join("continue.json");
    create_project_file(&project).expect("project を作る");
    remember_last_project(&store, &project).expect("覚える");

    let resume = decide_resume(&launch_with_project(None), Some(&store));
    match resume {
        Resume::Seated(seat) => {
            assert_eq!(seat.path(), project.as_path(), "続きが開いていない");
        }
        _ => panic!("覚えていた project へ座り直していない"),
    }
}

/// 初回起動(まだ何も覚えていない)は、何も言わずスタート画面。
#[test]
fn no_arguments_with_nothing_remembered_is_silent_start_screen() {
    let dir = motolii_testkit::tmp_dir("launch_resume_no_args_first");
    let store = dir.join("last-project.json");

    let resume = decide_resume(&launch_with_project(None), Some(&store));
    assert!(
        matches!(resume, Resume::Nothing),
        "初回起動は Nothing でなければならない"
    );
}

/// `--project` が指定されていれば、覚えていた project があってもそちらへは
/// 座らない(この分岐に入らない)。
#[test]
fn the_project_flag_takes_priority_over_the_remembered_project() {
    let dir = motolii_testkit::tmp_dir("launch_resume_project_priority");
    let store = dir.join("last-project.json");
    let remembered = dir.join("remembered.json");
    let explicit = dir.join("explicit.json");
    create_project_file(&remembered).expect("remembered project を作る");
    create_project_file(&explicit).expect("explicit project を作る");
    remember_last_project(&store, &remembered).expect("覚える");

    let resume = decide_resume(&launch_with_project(Some(explicit.clone())), Some(&store));
    match resume {
        Resume::Seated(seat) => {
            assert_eq!(
                seat.path(),
                explicit.as_path(),
                "--project より覚えていた方へ座ってしまった"
            );
        }
        _ => panic!("--project が開けるのに座っていない"),
    }
}

// ---------------------------------------------------------------------------
// `Shell::resumed` 経由 — 窓の view まで見て、座席・帯・remembering を確かめる。
// ---------------------------------------------------------------------------

/// `Shell::resumed` に `Resume::Seated` を渡すと、窓を開く前から座っている。
/// スタート画面のボタンは出ない。
#[test]
fn shell_resumed_with_a_seated_project_skips_the_start_screen() {
    let dir = motolii_testkit::tmp_dir("shell_resume_seated");
    let project = dir.join("launch.json");
    create_project_file(&project).expect("project を作る");
    let seat = motolii_ui::blitz_shell::ProjectSeat::open(&project).expect("open");

    let shell = Shell::resumed(ScriptedPrompts::default(), Resume::Seated(seat), None);

    assert!(shell.is_seated(), "--project 起動なのに座っていない");
    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(view::NEW_PROJECT).is_err(),
        "座っているのにスタート画面の New が出ている"
    );
}

/// 覚えていない project は開けないので、帯に理由が出たままスタート画面が残る。
#[test]
fn shell_resumed_with_explained_reason_shows_it_on_the_band_and_stays_on_start_screen() {
    let dir = motolii_testkit::tmp_dir("shell_resume_explained");
    let store = dir.join("last-project.json");
    let vanished = dir.join("vanished.json");
    create_project_file(&vanished).expect("project を作る");
    remember_last_project(&store, &vanished).expect("覚える");
    std::fs::remove_file(&vanished).expect("消す");

    let resume = decide_resume(&launch_with_project(None), Some(&store));
    let shell = Shell::resumed(ScriptedPrompts::default(), resume, None);

    assert!(!shell.is_seated(), "開けなかったのに座っている");
    assert!(
        shell
            .latest_report()
            .is_some_and(|report| report.contains("vanished.json")),
        "開けなかった project を帯が名指ししていない。実際: {:?}",
        shell.latest_report()
    );

    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(view::NEW_PROJECT).is_ok(),
        "開けなかったのだからスタート画面のはず"
    );
}

/// `last_project` の置き場を渡した `Shell` は、New で座り直すたびそこへ書く。
/// egui shell と**同じ座席**(`crate::last_project::remember_last_project`)を通る
/// ことの証拠 — 保存の経路も含めて2つ目の覚え方を作っていない。
#[test]
fn shell_resumed_with_a_store_remembers_the_next_seat() {
    let dir = motolii_testkit::tmp_dir("shell_resume_remembers");
    let store = dir.join("last-project.json");
    let project = dir.join("fresh.json");

    let mut shell = Shell::resumed(
        ScriptedPrompts {
            new_project_path: Some(project.clone()),
            ..ScriptedPrompts::default()
        },
        Resume::Nothing,
        Some(store.clone()),
    );

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);

    assert!(shell.is_seated(), "New で座れていない");
    assert_eq!(
        motolii_ui::load_last_project(&store).expect("読める"),
        Some(project),
        "New で座った project が last_project に書かれていない"
    );
}

/// `last_project` を渡していない `Shell`(通常の `Shell::new` を含む)は、
/// 座り直しても user 設定を1バイトも書かない — テスト・replay が利用者の
/// 「続きが開く」を踏まないための境目(egui 版と同じ規律)。
#[test]
fn shell_without_a_store_remembers_nothing() {
    let dir = motolii_testkit::tmp_dir("shell_resume_no_store");
    let store = dir.join("last-project.json");
    let project = dir.join("fresh.json");

    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);

    assert!(shell.is_seated(), "New で座れていない");
    assert!(!store.exists(), "覚え先を渡していないのに書いている");
}
