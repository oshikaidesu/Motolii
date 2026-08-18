//! `--project <path>` の読み方 — 加法(既存の `--intent-log` / `--status-log` の
//! 行は書き換えない、新しい枝を足すだけ)。

use std::path::Path;

use motolii_shell_iced::Launch;

#[test]
fn the_project_flag_is_read_with_a_separate_argument() {
    let parsed = Launch::parse(
        ["--project", "/tmp/p.json"]
            .into_iter()
            .map(str::to_owned),
    )
    .expect("--project <path> は読める");
    assert_eq!(parsed.project.as_deref(), Some(Path::new("/tmp/p.json")));
}

#[test]
fn the_project_flag_is_read_with_an_equals_sign() {
    let parsed = Launch::parse(["--project=/tmp/p.json".to_owned()].into_iter())
        .expect("--project=<path> は読める");
    assert_eq!(parsed.project.as_deref(), Some(Path::new("/tmp/p.json")));
}

#[test]
fn a_project_flag_without_a_value_fails_to_launch() {
    assert!(
        Launch::parse(["--project".to_owned()].into_iter()).is_err(),
        "path の無い --project を通してはならない"
    );
}

#[test]
fn the_project_flag_combines_with_the_existing_logs() {
    let parsed = Launch::parse(
        [
            "--project",
            "/tmp/p.json",
            "--intent-log",
            "/tmp/i.jsonl",
            "--status-log=/tmp/s.jsonl",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .expect("3本とも読める");
    assert_eq!(parsed.project.as_deref(), Some(Path::new("/tmp/p.json")));
    assert_eq!(parsed.intent_log.as_deref(), Some(Path::new("/tmp/i.jsonl")));
    assert_eq!(parsed.status_log.as_deref(), Some(Path::new("/tmp/s.jsonl")));
}

#[test]
fn no_arguments_leaves_project_unset() {
    assert_eq!(
        Launch::parse(std::iter::empty()).expect("引数なしで起動できる"),
        Launch::default()
    );
    assert_eq!(Launch::default().project, None);
}
