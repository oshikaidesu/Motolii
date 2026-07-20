//! M3E-1: UI toolkit依存方向CIのlive走査と負例。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use motolii_testkit::ui_toolkit_dep_policy::{
    find_slint_violations, find_slint_violations_in_crates, find_ui_toolkit_violations,
    find_ui_toolkit_violations_in_crates, is_slint_dependency, is_ui_toolkit_dependency,
    load_workspace_metadata, UI_TOOLKIT_CRATE_ALLOWLIST,
};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-testkit -> workspace root")
        .to_path_buf()
}

fn fixtures_root() -> PathBuf {
    workspace_root().join("crates/motolii-testkit/tests/fixtures/ui_toolkit_dep_policy")
}

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct SyntheticWorkspace {
    root: PathBuf,
}

impl Drop for SyntheticWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) {
    std::fs::create_dir_all(dest).unwrap_or_else(|err| {
        panic!("create_dir_all {} failed: {err}", dest.display());
    });
    for entry in std::fs::read_dir(src).unwrap_or_else(|err| {
        panic!("read_dir {} failed: {err}", src.display());
    }) {
        let entry = entry.unwrap();
        let dest_path = dest.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path);
        } else {
            std::fs::copy(entry.path(), &dest_path).unwrap_or_else(|err| {
                panic!(
                    "copy {} -> {} failed: {err}",
                    entry.path().display(),
                    dest_path.display()
                );
            });
        }
    }
}

/// `crates/<name>/` へfixtureを配置し、Cargo metadataが解決できる最小workspaceを作る。
fn synthetic_workspace(members: &[(&str, &str)]) -> SyntheticWorkspace {
    let seq = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = workspace_root().join(format!("target/ui-toolkit-dep-fixture-{seq}"));
    let _ = std::fs::remove_dir_all(&root);

    copy_dir_recursive(
        &fixtures_root().join("local_packages"),
        &root.join("local_packages"),
    );

    let crates_dir = root.join("crates");
    std::fs::create_dir_all(&crates_dir).unwrap();

    let member_paths: Vec<String> = members
        .iter()
        .map(|(crate_dir, fixture_name)| {
            let src = fixtures_root().join(fixture_name);
            let dest = crates_dir.join(crate_dir);
            std::fs::create_dir_all(dest.join("src")).unwrap();
            std::fs::copy(src.join("src/lib.rs"), dest.join("src/lib.rs")).unwrap_or_else(|err| {
                panic!("fixture {fixture_name} src/lib.rs missing: {err}");
            });

            let toml = std::fs::read_to_string(src.join("Cargo.toml")).unwrap_or_else(|err| {
                panic!("fixture {fixture_name} Cargo.toml missing: {err}");
            });
            let toml = toml.replace("../local_packages/", "../../local_packages/");
            std::fs::write(dest.join("Cargo.toml"), toml).unwrap();
            format!("crates/{crate_dir}")
        })
        .collect();

    let workspace_toml = format!(
        "[workspace]\nmembers = [{}]\nresolver = \"2\"\n",
        member_paths
            .iter()
            .map(|member| format!("\"{member}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    std::fs::write(root.join("Cargo.toml"), workspace_toml).unwrap();

    SyntheticWorkspace { root }
}

#[test]
fn ui_toolkit_dependency_names_cover_ecosystem() {
    assert!(is_ui_toolkit_dependency("egui"));
    assert!(is_ui_toolkit_dependency("egui-wgpu"));
    assert!(is_ui_toolkit_dependency("egui_tiles"));
    assert!(is_ui_toolkit_dependency("eframe"));
    assert!(is_ui_toolkit_dependency("winit"));
    assert!(is_ui_toolkit_dependency("epaint"));
    assert!(!is_ui_toolkit_dependency("wgpu"));
    assert!(!is_ui_toolkit_dependency("motolii-core"));
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
fn workspace_has_no_ui_toolkit_outside_ui_allowlist() {
    let root = workspace_root();
    let violations = find_ui_toolkit_violations_in_crates(&root, UI_TOOLKIT_CRATE_ALLOWLIST);
    assert!(
        violations.is_empty(),
        "UI toolkit must be limited to {:?}; violations: {violations:#?}",
        UI_TOOLKIT_CRATE_ALLOWLIST
    );
}

#[test]
fn workspace_has_no_slint_in_crates() {
    let root = workspace_root();
    let violations = find_slint_violations_in_crates(&root);
    assert!(
        violations.is_empty(),
        "slint must not appear in crates/; violations: {violations:#?}"
    );
}

#[test]
fn metadata_flags_non_ui_crate_with_egui() {
    let ws = synthetic_workspace(&[("motolii-core", "violation_core")]);
    let metadata = load_workspace_metadata(&ws.root).unwrap_or_else(|err| {
        panic!("cargo metadata failed for {}: {err}", ws.root.display());
    });
    let violations = find_ui_toolkit_violations(&metadata, &ws.root, UI_TOOLKIT_CRATE_ALLOWLIST);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].crate_name, "motolii-core");
    assert_eq!(violations[0].dependency, "egui");
    assert_eq!(violations[0].section, "dependencies");
}

#[test]
fn metadata_flags_renamed_egui_dependency() {
    let ws = synthetic_workspace(&[("motolii-core", "violation_rename")]);
    let metadata = load_workspace_metadata(&ws.root).unwrap_or_else(|err| {
        panic!("cargo metadata failed for {}: {err}", ws.root.display());
    });
    let violations = find_ui_toolkit_violations(&metadata, &ws.root, UI_TOOLKIT_CRATE_ALLOWLIST);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].crate_name, "motolii-core");
    assert_eq!(violations[0].dependency, "egui");
    assert_eq!(violations[0].section, "dependencies");
}

#[test]
fn metadata_flags_dev_dependency_eframe() {
    let ws = synthetic_workspace(&[("motolii-render", "violation_dev_dep")]);
    let metadata = load_workspace_metadata(&ws.root).unwrap_or_else(|err| {
        panic!("cargo metadata failed for {}: {err}", ws.root.display());
    });
    let violations = find_ui_toolkit_violations(&metadata, &ws.root, UI_TOOLKIT_CRATE_ALLOWLIST);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].crate_name, "motolii-render");
    assert_eq!(violations[0].section, "dev-dependencies");
    assert_eq!(violations[0].dependency, "eframe");
}

#[test]
fn metadata_flags_slint_dependency() {
    let ws = synthetic_workspace(&[("motolii-core", "violation_slint")]);
    let metadata = load_workspace_metadata(&ws.root).unwrap_or_else(|err| {
        panic!("cargo metadata failed for {}: {err}", ws.root.display());
    });
    let violations = find_slint_violations(&metadata, &ws.root);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].crate_name, "motolii-core");
    assert_eq!(violations[0].dependency, "slint");
    assert_eq!(violations[0].section, "dependencies");
}

#[test]
fn allowlist_skips_ui_crate_in_metadata_scan() {
    let ws = synthetic_workspace(&[
        ("motolii-ui", "allowed_ui"),
        ("motolii-core", "violation_core"),
    ]);
    let metadata = load_workspace_metadata(&ws.root).unwrap_or_else(|err| {
        panic!("cargo metadata failed for {}: {err}", ws.root.display());
    });
    let violations = find_ui_toolkit_violations(&metadata, &ws.root, UI_TOOLKIT_CRATE_ALLOWLIST);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].crate_name, "motolii-core");
    assert_eq!(violations[0].dependency, "egui");
}
