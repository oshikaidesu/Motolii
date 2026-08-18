//! `--intent-log` / `--status-log` の2本 — red 先行。
//!
//! 行の形は egui runner(`blitz_shell/runner.rs`)と**同型**である:
//!
//! ```text
//! {"seq":1,"intent":{"kind":"new_project","path":"…"}}   ← 原因
//! {"seq":1,"text":"opened …"}                             ← 結果
//! ```
//!
//! 同じ形で出るからこそ、2本を並べて「何をしようとして(intent)、何が起きたか
//! (status)」が読める。原因の列はそのまま `ShellGateway::replay` へ食わせられる。

mod common;

use common::{drain, press};
use motolii_shell_iced::{view, IntentLog, Launch, ScriptedPrompts, Shell, StatusLog};

/// 結果のログが JSONL で外へ出る。egui shell の `--status-log` と同じ1行。
#[test]
fn the_status_log_writes_the_same_jsonl_shape_as_the_egui_runner() {
    let dir = motolii_testkit::tmp_dir("iced_status_log");
    let project = dir.join("fresh.json");
    // 親ディレクトリが無くても作る(記録先の不在で起動を落とさない)。
    let log_path = dir.join("nested/statuses.jsonl");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });
    let mut log = StatusLog::create(&log_path).expect("status log を開ける");

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    log.flush(&shell);

    let written = std::fs::read_to_string(&log_path).expect("status log が書けている");
    assert_eq!(
        written,
        format!(
            "{{\"seq\":1,\"text\":\"opened {}\"}}\n",
            project.display()
        ),
        "結果の1行は egui runner と同型でなければならない"
    );
}

/// 増えた分だけ追記する(既に流した行を二度書かない)。
#[test]
fn both_logs_append_only_what_is_new() {
    let dir = motolii_testkit::tmp_dir("iced_logs_append");
    let project = dir.join("fresh.json");
    let intents = dir.join("intents.jsonl");
    let statuses = dir.join("statuses.jsonl");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        ..ScriptedPrompts::default()
    });
    let mut intent_log = IntentLog::create(&intents).expect("intent log を開ける");
    let mut status_log = StatusLog::create(&statuses).expect("status log を開ける");

    // まだ何も起きていないうちに1回流しても、空のまま。
    intent_log.flush(&shell);
    status_log.flush(&shell);

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    intent_log.flush(&shell);
    status_log.flush(&shell);
    // 何も起きていない所でもう一度流す = 二度書きの有無を見る。
    intent_log.flush(&shell);
    status_log.flush(&shell);

    assert_eq!(
        std::fs::read_to_string(&intents).unwrap().lines().count(),
        1,
        "同じ intent を二度書いている"
    );
    assert_eq!(
        std::fs::read_to_string(&statuses).unwrap().lines().count(),
        1,
        "同じ status を二度書いている"
    );
}

/// 引数の読み方。**知らない引数は黙って無視しない**(起動失敗にする)。
#[test]
fn the_launch_arguments_name_both_logs() {
    let parsed = Launch::parse(
        [
            "--intent-log",
            "/tmp/i.jsonl",
            "--status-log=/tmp/s.jsonl",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("2本とも読める");
    assert_eq!(
        parsed.intent_log.as_deref(),
        Some(std::path::Path::new("/tmp/i.jsonl"))
    );
    assert_eq!(
        parsed.status_log.as_deref(),
        Some(std::path::Path::new("/tmp/s.jsonl"))
    );

    assert_eq!(
        Launch::parse(std::iter::empty()).expect("引数なしで起動できる"),
        Launch::default(),
        "引数なしの起動はどちらも記録しない"
    );
    assert!(
        Launch::parse(["--status-log".to_owned()].into_iter()).is_err(),
        "path の無い --status-log を通してはならない"
    );
    assert!(
        Launch::parse(["--wat".to_owned()].into_iter()).is_err(),
        "知らない引数は起動失敗にする"
    );
}
