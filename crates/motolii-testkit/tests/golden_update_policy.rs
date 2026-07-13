//! D1i-4 / S16: 意味論ゴールデン更新禁止のCI執行テスト。
//!
//! - (a) semantic: 既存内容の変更はマーカー有無に関わらず拒否
//! - (b) provisional: `MOTOLII_REGENERATE_WHEN` 付きでのみ更新可
//!
//! ライブ台帳は `crates/motolii-testkit/golden_policy/classification.tsv`。
//! 負例は `tests/fixtures/golden_policy/` + CLASSIFICATION_FILE 上書き。

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-testkit -> workspace root")
        .to_path_buf()
}

fn run_policy(classification: Option<&str>, skip_consistency: bool, files: &str) -> (bool, String) {
    let root = workspace_root();
    let script = root.join("scripts/check-golden-update-policy.sh");
    let mut cmd = Command::new("bash");
    cmd.arg(&script)
        .arg("--files-from")
        .arg("-")
        .current_dir(&root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(c) = classification {
        cmd.env("CLASSIFICATION_FILE", c);
    }
    if skip_consistency {
        cmd.env("GOLDEN_POLICY_SKIP_CONSISTENCY", "1");
    } else {
        cmd.env_remove("GOLDEN_POLICY_SKIP_CONSISTENCY");
    }
    let mut child = cmd.spawn().expect("spawn check-golden-update-policy.sh");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(files.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), msg)
}

fn run_live_policy_no_changes() -> (bool, String) {
    let root = workspace_root();
    let script = root.join("scripts/check-golden-update-policy.sh");
    // 存在しない base で差分ゼロ扱い — 台帳一貫性だけを見る
    let out = Command::new("bash")
        .arg(&script)
        .arg("refs/does-not-exist-d1i4")
        .current_dir(&root)
        .output()
        .expect("run policy");
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), msg)
}

#[test]
fn live_classification_is_consistent_and_nonempty() {
    let (ok, msg) = run_live_policy_no_changes();
    assert!(ok, "live classification must be consistent: {msg}");
    assert!(
        msg.contains("golden-update-policy OK") || msg.contains("treating as no changes"),
        "unexpected: {msg}"
    );
}

#[test]
fn d1i2_pathop_geometry_is_classified_semantic() {
    let root = workspace_root();
    let tsv = std::fs::read_to_string(
        root.join("crates/motolii-testkit/golden_policy/classification.tsv"),
    )
    .expect("read classification.tsv");
    assert!(
        tsv.lines().any(|l| {
            l.starts_with("semantic\t")
                && l.contains("crates/motolii-doc/tests/d1i2_pathop_geometry.rs")
        }),
        "D1i-2 geometry golden must be classified semantic: {tsv}"
    );
    let src = std::fs::read_to_string(
        root.join("crates/motolii-doc/tests/d1i2_pathop_geometry.rs"),
    )
    .expect("read geometry golden");
    assert!(
        src.contains("MOTOLII_GOLDEN_CLASS: semantic"),
        "geometry golden must carry class marker"
    );
}

#[test]
fn semantic_golden_modification_is_rejected() {
    let (ok, msg) = run_policy(
        None,
        false,
        "M\tcrates/motolii-doc/tests/d1i2_pathop_geometry.rs\n",
    );
    assert!(!ok, "expected fail for semantic modify: {msg}");
    assert!(
        msg.contains("semantic golden modified") || msg.contains("golden-update-policy gate FAILED"),
        "unexpected: {msg}"
    );
}

#[test]
fn semantic_golden_modification_rejected_even_with_regenerate_marker() {
    let class = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_semantic_with_marker.tsv";
    let (ok, msg) = run_policy(
        Some(class),
        false,
        "M\tcrates/motolii-testkit/tests/fixtures/golden_policy/semantic_with_regenerate_marker.txt\n",
    );
    assert!(!ok, "regenerate marker must not bypass semantic lock: {msg}");
    assert!(
        msg.contains("semantic golden modified"),
        "unexpected: {msg}"
    );
}

#[test]
fn semantic_golden_addition_is_allowed() {
    let class = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_ok.tsv";
    // 台帳に載っていない新パスは分類外扱い。追加許可は「台帳上 semantic かつ status=A」。
    // フィクスチャの semantic_sample を A として渡す(新規追加シナリオ)。
    let (ok, msg) = run_policy(
        Some(class),
        false,
        "A\tcrates/motolii-testkit/tests/fixtures/golden_policy/semantic_sample.txt\n",
    );
    assert!(ok, "new semantic golden file must be allowed: {msg}");
}

#[test]
fn provisional_update_with_marker_is_allowed() {
    let class = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_ok.tsv";
    let (ok, msg) = run_policy(
        Some(class),
        false,
        "M\tcrates/motolii-testkit/tests/fixtures/golden_policy/provisional_with_marker.txt\n",
    );
    assert!(ok, "provisional+marker must allow update: {msg}");
}

#[test]
fn provisional_update_without_marker_is_rejected() {
    let class =
        "crates/motolii-testkit/tests/fixtures/golden_policy/classification_provisional_no_marker.tsv";
    let (ok, msg) = run_policy(
        Some(class),
        true, // マーカー欠落台帳は consistency を意図的に破る
        "M\tcrates/motolii-testkit/tests/fixtures/golden_policy/provisional_without_marker.txt\n",
    );
    assert!(!ok, "provisional without marker must fail: {msg}");
    assert!(
        msg.contains("provisional golden lacks") || msg.contains("MOTOLII_REGENERATE_WHEN"),
        "unexpected: {msg}"
    );
}

#[test]
fn unclassified_src_change_is_allowed() {
    let (ok, msg) = run_policy(
        None,
        false,
        "M\tcrates/motolii-doc/src/lib.rs\nA\tcrates/motolii-doc/tests/new_test.rs\n",
    );
    assert!(ok, "unclassified changes must pass: {msg}");
}

#[test]
fn empty_semantic_classification_is_rejected() {
    let root = workspace_root();
    let empty = root.join("target/d1i4-empty-classification.tsv");
    std::fs::create_dir_all(empty.parent().unwrap()).ok();
    std::fs::write(&empty, "# empty on purpose\n").unwrap();
    let rel = empty
        .strip_prefix(&root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let (ok, msg) = run_policy(Some(&rel), false, "");
    assert!(!ok, "empty semantic set must fail: {msg}");
    assert!(
        msg.contains("semantic classification is empty"),
        "unexpected: {msg}"
    );
}
