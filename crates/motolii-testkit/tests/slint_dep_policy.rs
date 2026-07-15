//! M3E-1: slint 依存方向 CI のライブ走査と負例。

use std::path::{Path, PathBuf};

use motolii_testkit::slint_dep_policy::{
    find_slint_violations_in_crates, is_slint_dependency, scan_cargo_toml, SLINT_UI_CRATE_ALLOWLIST,
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-testkit -> workspace root")
        .to_path_buf()
}

fn fixture_manifest(name: &str) -> PathBuf {
    workspace_root().join(format!(
        "crates/motolii-testkit/tests/fixtures/slint_dep_policy/{name}/Cargo.toml"
    ))
}

fn read_fixture(name: &str) -> (PathBuf, String) {
    let path = fixture_manifest(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("fixture {} missing: {err}", path.display());
    });
    (path, text)
}

#[test]
fn slint_dependency_names_cover_ecosystem() {
    assert!(is_slint_dependency("slint"));
    assert!(is_slint_dependency("slint-build"));
    assert!(is_slint_dependency("slint-interpreter"));
    assert!(is_slint_dependency("i-slint-core"));
    assert!(!is_slint_dependency("wgpu"));
    assert!(!is_slint_dependency("motolii-core"));
}

#[test]
fn workspace_has_no_slint_outside_ui_allowlist() {
    let root = workspace_root();
    let violations = find_slint_violations_in_crates(&root, SLINT_UI_CRATE_ALLOWLIST);
    assert!(
        violations.is_empty(),
        "slint must be limited to {:?}; violations: {violations:#?}",
        SLINT_UI_CRATE_ALLOWLIST
    );
}

#[test]
fn detector_flags_non_ui_crate_with_slint() {
    let (path, text) = read_fixture("violation_core");
    let hits = scan_cargo_toml(&path, &text);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].crate_name, "motolii-core");
    assert_eq!(hits[0].dependency, "slint");
    assert_eq!(hits[0].section, "dependencies");
}

#[test]
fn detector_flags_dev_dependency_slint() {
    let (path, text) = read_fixture("violation_dev_dep");
    let hits = scan_cargo_toml(&path, &text);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].section, "dev-dependencies");
    assert_eq!(hits[0].dependency, "slint-build");
}

#[test]
fn detector_allows_ui_crate_fixture() {
    let (path, text) = read_fixture("allowed_ui");
    let hits = scan_cargo_toml(&path, &text);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].crate_name, "motolii-ui");
    assert!(
        SLINT_UI_CRATE_ALLOWLIST.contains(&hits[0].crate_name.as_str()),
        "ui crate slint dep is allowlisted"
    );
}

#[test]
fn allowlist_skips_ui_crate_in_scanner() {
    let root = workspace_root();
    let synthetic_root = root.join("target/m3e1-slint-dep-fixture");
    let crates_root = synthetic_root.join("crates");
    let _ = std::fs::remove_dir_all(&synthetic_root);
    std::fs::create_dir_all(crates_root.join("motolii-ui")).unwrap();
    std::fs::create_dir_all(crates_root.join("motolii-core")).unwrap();
    std::fs::copy(
        fixture_manifest("allowed_ui"),
        crates_root.join("motolii-ui/Cargo.toml"),
    )
    .unwrap();
    std::fs::copy(
        fixture_manifest("violation_core"),
        crates_root.join("motolii-core/Cargo.toml"),
    )
    .unwrap();

    let violations = find_slint_violations_in_crates(&synthetic_root, SLINT_UI_CRATE_ALLOWLIST);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].crate_name, "motolii-core");

    let _ = std::fs::remove_dir_all(&synthetic_root);
}
