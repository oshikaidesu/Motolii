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
    run_policy_with_base(classification, None, skip_consistency, files)
}

fn run_policy_with_base(
    classification: Option<&str>,
    base_classification: Option<&str>,
    skip_consistency: bool,
    files: &str,
) -> (bool, String) {
    run_policy_with_base_opts(
        classification,
        base_classification,
        None,
        skip_consistency,
        false,
        files,
    )
}

fn run_policy_with_base_opts(
    classification: Option<&str>,
    base_classification: Option<&str>,
    migration: Option<&str>,
    skip_consistency: bool,
    base_lookup_only: bool,
    files: &str,
) -> (bool, String) {
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
        cmd.env(
            "MIGRATION_FILE",
            "crates/motolii-testkit/tests/fixtures/golden_policy/migrations_empty.tsv",
        );
    } else {
        cmd.env_remove("CLASSIFICATION_FILE");
        cmd.env_remove("MIGRATION_FILE");
    }
    if let Some(m) = migration {
        cmd.env("MIGRATION_FILE", m);
    }
    if let Some(b) = base_classification {
        cmd.env("GOLDEN_POLICY_BASE_CLASSIFICATION", b);
    } else {
        cmd.env_remove("GOLDEN_POLICY_BASE_CLASSIFICATION");
    }
    if base_lookup_only {
        cmd.env("GOLDEN_POLICY_BASE_LOOKUP_ONLY", "1");
    } else {
        cmd.env_remove("GOLDEN_POLICY_BASE_LOOKUP_ONLY");
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
    // 差分ゼロ(stdin空)で台帳一貫性だけを見る。存在しないbaseはfail-closedのため使わない。
    run_policy(None, false, "")
}

#[test]
fn live_classification_is_consistent_and_nonempty() {
    let (ok, msg) = run_live_policy_no_changes();
    assert!(ok, "live classification must be consistent: {msg}");
    assert!(msg.contains("golden-update-policy OK"), "unexpected: {msg}");
}

#[test]
fn missing_base_ref_is_fail_closed() {
    let root = workspace_root();
    let script = root.join("scripts/check-golden-update-policy.sh");
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
    assert!(
        !out.status.success(),
        "missing base must fail closed: {msg}"
    );
    assert!(
        msg.contains("base ref") && msg.contains("not found"),
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
    // 本体編集なしで分類できること(正本は台帳)。
    assert!(root
        .join("crates/motolii-doc/tests/d1i2_pathop_geometry.rs")
        .is_file());
}

#[test]
fn blend_mode_oracle_is_semantic_and_harness_is_not() {
    let root = workspace_root();
    let tsv = std::fs::read_to_string(
        root.join("crates/motolii-testkit/golden_policy/classification.tsv"),
    )
    .expect("read classification.tsv");
    assert!(tsv
        .lines()
        .any(|line| { line == "semantic\tcrates/motolii-doc/tests/oracles/d1i3_blend_mode.tsv" }));
    assert!(!tsv
        .lines()
        .any(|line| { line == "semantic\tcrates/motolii-doc/tests/d1i3_blend_mode.rs" }));
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
        msg.contains("semantic golden modified")
            || msg.contains("golden-update-policy gate FAILED"),
        "unexpected: {msg}"
    );
}

#[test]
fn blend_mode_oracle_modification_is_rejected() {
    let (ok, msg) = run_policy(
        None,
        false,
        "M\tcrates/motolii-doc/tests/oracles/d1i3_blend_mode.tsv\n",
    );
    assert!(!ok, "expected fail for semantic oracle modify: {msg}");
    assert!(
        msg.contains("semantic golden modified"),
        "unexpected: {msg}"
    );
}

#[test]
fn blend_mode_harness_runtime_wiring_change_is_allowed() {
    let (ok, msg) = run_policy(
        None,
        false,
        "M\tcrates/motolii-doc/tests/d1i3_blend_mode.rs\n",
    );
    assert!(ok, "semantic harness change must be allowed: {msg}");
}

/// 台帳ブートストラップPR相当: HEADでsemantic登録済みの既存ファイルを同時に書き換えても拒否する。
/// (base未登録を理由に M を許可するとS16が初回PRで空洞化する)
#[test]
fn bootstrap_registration_cannot_rewrite_existing_semantic_file() {
    let (ok, msg) = run_policy(
        None,
        false,
        "A\tcrates/motolii-testkit/golden_policy/classification.tsv\nM\tcrates/motolii-doc/tests/d1i2_pathop_geometry.rs\n",
    );
    assert!(
        !ok,
        "bootstrap must not allow rewriting existing semantic golden: {msg}"
    );
    assert!(
        msg.contains("semantic golden modified"),
        "unexpected: {msg}"
    );
}

/// 台帳だけ追加し既存ゴールデンを触らない経路は許可。
#[test]
fn classification_tsv_only_change_is_allowed() {
    let (ok, msg) = run_policy(
        None,
        false,
        "A\tcrates/motolii-testkit/golden_policy/classification.tsv\n",
    );
    assert!(ok, "classification-only bootstrap must be allowed: {msg}");
}

#[test]
fn semantic_golden_modification_rejected_even_with_regenerate_marker() {
    let class = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_semantic_with_marker.tsv";
    let (ok, msg) = run_policy(
        Some(class),
        false,
        "M\tcrates/motolii-testkit/tests/fixtures/golden_policy/semantic_with_regenerate_marker.txt\n",
    );
    assert!(
        !ok,
        "regenerate marker must not bypass semantic lock: {msg}"
    );
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

/// provisional 行を台帳から消すだけでは通過できない(分類解除によるマーカー回避を塞ぐ)。
#[test]
fn dropping_provisional_from_classification_is_rejected() {
    let head = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_head_drop_provisional.tsv";
    let base = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_base_with_provisional.tsv";
    let (ok, msg) = run_policy_with_base(
        Some(head),
        Some(base),
        false,
        "M\tcrates/motolii-testkit/golden_policy/classification.tsv\n",
    );
    assert!(!ok, "dropping provisional classification must fail: {msg}");
    assert!(
        msg.contains("provisional entry removed") || msg.contains("declassification forbidden"),
        "unexpected: {msg}"
    );
}

/// 台帳から provisional を外し、マーカー無しで本体を書き換える迂回も拒否。
#[test]
fn declassify_provisional_then_modify_without_marker_is_rejected() {
    let head = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_head_drop_provisional.tsv";
    let base = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_base_with_provisional.tsv";
    let (ok, msg) = run_policy_with_base(
        Some(head),
        Some(base),
        false,
        "M\tcrates/motolii-testkit/golden_policy/classification.tsv\nM\tcrates/motolii-testkit/tests/fixtures/golden_policy/provisional_without_marker.txt\n",
    );
    assert!(
        !ok,
        "declassify+modify provisional without marker must fail: {msg}"
    );
    assert!(
        msg.contains("provisional entry removed")
            || msg.contains("declassification forbidden")
            || msg.contains("provisional golden lacks"),
        "unexpected: {msg}"
    );
}

/// HEAD台帳では未分類でも、base が provisional ならマーカー必須(effective class)。
/// 削り検査を意図的に外し、参照合成だけを審判する(本番CIでは LOOKUP_ONLY を付けない)。
#[test]
fn base_provisional_still_requires_marker_when_head_unclassified() {
    let head = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_head_drop_provisional.tsv";
    let base = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_base_provisional_unmarked.tsv";
    let (ok, msg) = run_policy_with_base_opts(
        Some(head),
        Some(base),
        None,
        false,
        true, // lookup-only: 台帳削り以外の effective class 経路
        "M\tcrates/motolii-testkit/tests/fixtures/golden_policy/provisional_without_marker.txt\n",
    );
    assert!(
        !ok,
        "base provisional must still be gated via effective class: {msg}"
    );
    assert!(
        msg.contains("provisional golden lacks") || msg.contains("MOTOLII_REGENERATE_WHEN"),
        "unexpected: {msg}"
    );
}

#[test]
fn semantic_harness_to_registered_oracle_migration_is_allowed() {
    let head = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_head_semantic_migration.tsv";
    let base = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_base_semantic_sample.tsv";
    let migration =
        "crates/motolii-testkit/tests/fixtures/golden_policy/migrations_semantic_sample.tsv";
    let (ok, msg) = run_policy_with_base_opts(
        Some(head),
        Some(base),
        Some(migration),
        false,
        false,
        "M\tcrates/motolii-testkit/golden_policy/classification.tsv\nM\tcrates/motolii-testkit/tests/fixtures/golden_policy/semantic_sample.txt\nA\tcrates/motolii-testkit/tests/fixtures/golden_policy/semantic_with_regenerate_marker.txt\n",
    );
    assert!(ok, "valid harness-to-oracle migration must pass: {msg}");
}

#[test]
fn semantic_declassification_without_migration_is_rejected() {
    let head = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_head_semantic_migration.tsv";
    let base = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_base_semantic_sample.tsv";
    let (ok, msg) = run_policy_with_base(
        Some(head),
        Some(base),
        false,
        "M\tcrates/motolii-testkit/golden_policy/classification.tsv\n",
    );
    assert!(!ok, "semantic declassification must fail: {msg}");
    assert!(
        msg.contains("removed without a valid harness-to-oracle migration"),
        "unexpected: {msg}"
    );
}

/// provisional → semantic 昇格(台帳のみ)は許可。
#[test]
fn promoting_provisional_to_semantic_is_allowed() {
    let head = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_head_promote_provisional.tsv";
    let base = "crates/motolii-testkit/tests/fixtures/golden_policy/classification_base_with_provisional.tsv";
    let (ok, msg) = run_policy_with_base(
        Some(head),
        Some(base),
        false,
        "M\tcrates/motolii-testkit/golden_policy/classification.tsv\n",
    );
    assert!(ok, "provisional→semantic promotion must be allowed: {msg}");
}
