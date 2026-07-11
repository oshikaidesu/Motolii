//! M2E-2: 保護テスト資産の集約とdiffゲートの負例。

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-testkit -> workspace root")
        .to_path_buf()
}

fn rust_sources_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name == "target" {
                    continue;
                }
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_protected_oracle_path(rel_path: &str) -> bool {
    rel_path.starts_with("crates/motolii-testkit/src/cpu_reference/")
}

/// 受け入れオラクル定義行か。`pub`の有無を問わない(private再導入を検出する)。
fn oracle_defn_name(line: &str) -> Option<&str> {
    let t = line.trim_start();
    let t = t.strip_prefix("pub ").unwrap_or(t);
    let rest = t.strip_prefix("fn ")?;
    let name = rest.split('(').next()?.trim();
    if name.starts_with("expected_")
        || name == "yuv_to_rgba_reference"
        || name == "premul_over_u8"
        || name == "premul_add_u8"
        || name == "premul_multiply_u8"
    {
        Some(name)
    } else {
        None
    }
}

fn find_oracle_defns_outside_protected(root: &Path) -> Vec<(String, String)> {
    let mut hits = Vec::new();
    for path in rust_sources_under(root) {
        let rel_path = rel(root, &path);
        if is_protected_oracle_path(&rel_path) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            if let Some(name) = oracle_defn_name(line) {
                hits.push((rel_path.clone(), name.to_string()));
            }
        }
    }
    hits.sort();
    hits
}

#[test]
fn oracle_defn_scanner_detects_private_expected_fn() {
    assert_eq!(
        oracle_defn_name("fn expected_sneaky(desc: FrameDesc) -> Vec<u8> {"),
        Some("expected_sneaky")
    );
    assert_eq!(
        oracle_defn_name("    fn expected_fixed_graph(desc: FrameDesc) -> Vec<u8> {"),
        Some("expected_fixed_graph")
    );
    assert_eq!(
        oracle_defn_name("pub fn yuv_to_rgba_reference(frame: &CpuFrame) -> Vec<u8> {"),
        Some("yuv_to_rgba_reference")
    );
    assert_eq!(oracle_defn_name("// fn expected_commented() {"), None);
    assert_eq!(
        oracle_defn_name("let expected_px = premul_over_u8(a, b);"),
        None
    );
}

/// 既存受け入れオラクル定義は保護領域外に存在してはならない。
#[test]
fn acceptance_oracles_live_only_in_protected_area() {
    let root = workspace_root();
    let hits = find_oracle_defns_outside_protected(&root);
    assert!(
        hits.is_empty(),
        "oracle definitions outside cpu_reference/: {hits:?}"
    );
}

fn run_protected_diff(files: &str) -> (bool, String) {
    let root = workspace_root();
    let script = root.join("scripts/check-protected-diff.sh");
    let mut child = Command::new("bash")
        .arg(&script)
        .arg("--files-from")
        .arg("-")
        .current_dir(&root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn check-protected-diff.sh");
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

#[test]
fn protected_diff_gate_rejects_simultaneous_protected_and_src() {
    let (ok, msg) = run_protected_diff(
        "crates/motolii-testkit/src/tol/mod.rs\ncrates/motolii-gpu/src/yuv.rs\n",
    );
    assert!(!ok, "expected fail, got ok: {msg}");
    assert!(
        msg.contains("protected-diff gate FAILED"),
        "unexpected message: {msg}"
    );
}

#[test]
fn protected_diff_gate_rejects_protected_with_ordinary_tests() {
    let (ok, msg) = run_protected_diff(
        "crates/motolii-testkit/src/tol/mod.rs\ncrates/motolii-gpu/tests/yuv_golden.rs\n",
    );
    assert!(!ok, "expected fail for protected+tests: {msg}");
}

#[test]
fn protected_diff_gate_rejects_protected_with_cargo_toml() {
    let (ok, msg) = run_protected_diff(
        "crates/motolii-testkit/src/cpu_reference/luma.rs\ncrates/motolii-gpu/Cargo.toml\n",
    );
    assert!(!ok, "expected fail for protected+Cargo.toml: {msg}");
}

#[test]
fn protected_diff_gate_rejects_protected_with_ci() {
    let (ok, msg) =
        run_protected_diff("crates/motolii-testkit/src/tol/mod.rs\n.github/workflows/ci.yml\n");
    assert!(!ok, "expected fail for protected+CI: {msg}");
}

#[test]
fn protected_diff_gate_allows_protected_only_test_update() {
    let (ok, msg) = run_protected_diff(
        "crates/motolii-testkit/src/tol/mod.rs\ncrates/motolii-testkit/golden/README.md\n",
    );
    assert!(ok, "expected ok for protected-only: {msg}");
}

#[test]
fn protected_diff_gate_allows_src_only_implementation() {
    let (ok, msg) =
        run_protected_diff("crates/motolii-gpu/src/yuv.rs\ncrates/motolii-render/src/lib.rs\n");
    assert!(ok, "expected ok for src-only: {msg}");
}

#[test]
fn protected_diff_gate_allows_ordinary_tdd_tests_with_src() {
    // 通常TDD(実装+新規テスト)は保護対象外 — 保護パスに触れなければ許可
    let (ok, msg) = run_protected_diff(
        "crates/motolii-gpu/src/yuv.rs\ncrates/motolii-gpu/tests/yuv_golden.rs\n",
    );
    assert!(ok, "expected ok for TDD (src+tests): {msg}");
}

