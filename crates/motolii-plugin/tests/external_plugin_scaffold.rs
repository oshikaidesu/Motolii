//! VSM-A4I: 外部作者crateは公開façadeだけで生成・Host検査できる。

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-plugin is below workspace root")
        .to_path_buf()
}

fn tool() -> PathBuf {
    workspace_root().join("scripts/new-plugin-crate.sh")
}

fn temp_root(label: &str) -> PathBuf {
    let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "motolii-external-plugin-scaffold-{label}-{}-{serial}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create temporary test root");
    root
}

fn run(args: &[&str]) -> Output {
    Command::new("bash")
        .arg(tool())
        .args(args)
        .output()
        .expect("run new-plugin-crate")
}

fn generate(root: &Path, vendor: &str, name: &str) -> PathBuf {
    let crate_dir = root.join("candidate");
    let crate_dir_arg = crate_dir.to_string_lossy().into_owned();
    let output = run(&[
        "--from",
        "core.layer_source.radial_repeater",
        "--vendor",
        vendor,
        "--name",
        name,
        "--out-dir",
        &crate_dir_arg,
    ]);
    assert!(
        output.status.success(),
        "generation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    crate_dir
}

fn check(crate_dir: &Path) -> Output {
    let crate_dir_arg = crate_dir.to_string_lossy().into_owned();
    run(&["--check", &crate_dir_arg])
}

fn assert_rejected(output: Output, expected: &str) {
    assert!(
        !output.status.success(),
        "check unexpectedly passed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("check[hygiene]"), "stderr: {stderr}");
    assert!(stderr.contains(expected), "stderr: {stderr}");
}

#[test]
fn generated_external_crate_is_isolated_and_passes_host_check() {
    let root = temp_root("positive");
    let crate_dir = generate(&root, "acme", "radial_fork");

    let files: BTreeSet<PathBuf> = fs::read_dir(&crate_dir)
        .expect("read generated root")
        .map(|entry| entry.expect("entry").file_name().into())
        .collect();
    assert_eq!(
        files,
        BTreeSet::from([
            PathBuf::from("AUTHORING.md"),
            PathBuf::from("Cargo.toml"),
            PathBuf::from("src"),
        ])
    );

    let manifest = fs::read_to_string(crate_dir.join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains("edition = \"2021\""));
    assert!(manifest.contains("license = \"MIT OR Apache-2.0\""));
    assert!(manifest.contains("motolii-plugin = { path = \""));
    assert!(manifest.contains("[workspace]"));
    assert!(!manifest.contains("workspace = true"));
    assert!(!manifest.contains("dev-dependencies"));
    assert!(!manifest.contains("build-dependencies"));
    assert!(!crate_dir.join("build.rs").exists());

    let source = fs::read_to_string(crate_dir.join("src/lib.rs")).expect("source");
    assert!(source.contains("acme.layer_source.radial_fork"));
    assert!(source.contains("use motolii_plugin::"));
    assert!(!source.contains("motolii_core"));
    assert!(!source.contains("motolii_eval"));
    assert!(!source.contains("motolii_gpu"));
    assert!(!source.contains("motolii_testkit"));

    let output = check(&crate_dir);
    assert!(
        output.status.success(),
        "Host check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!crate_dir.join("Cargo.lock").exists());
    assert!(!crate_dir.join("target").exists());

    fs::remove_dir_all(root).expect("remove temporary test root");
}

#[test]
fn generator_rejects_reserved_vendor_and_repository_output() {
    let root = temp_root("input");
    let reserved = root.join("reserved").to_string_lossy().into_owned();
    let output = run(&[
        "--from",
        "core.layer_source.radial_repeater",
        "--vendor",
        "core",
        "--name",
        "fork",
        "--out-dir",
        &reserved,
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("reserved"));

    let inside = workspace_root().join("target/external-plugin-scaffold-rejected-output");
    let inside_arg = inside.to_string_lossy().into_owned();
    let output = run(&[
        "--from",
        "core.layer_source.radial_repeater",
        "--vendor",
        "acme",
        "--name",
        "fork",
        "--out-dir",
        &inside_arg,
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("outside the Motolii repository"));
    assert!(!inside.exists());

    let output = check(&workspace_root().join("plugins/motolii-plugin-radial-repeater"));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("outside the Motolii repository"));

    fs::remove_dir_all(root).expect("remove temporary test root");
}

#[test]
fn host_check_rejects_dependency_and_authority_escapes() {
    let root = temp_root("negative");
    let crate_dir = generate(&root, "acme", "radial_fork");
    let manifest_path = crate_dir.join("Cargo.toml");
    let source_path = crate_dir.join("src/lib.rs");
    let base_manifest = fs::read_to_string(&manifest_path).expect("manifest");
    let base_source = fs::read_to_string(&source_path).expect("source");

    fs::write(
        &manifest_path,
        base_manifest.replace(
            "[workspace]",
            "motolii-core = { path = \"/forbidden\" }\n\n[workspace]",
        ),
    )
    .expect("add dependency");
    assert_rejected(check(&crate_dir), "only regular dependency");

    fs::write(
        &manifest_path,
        format!("{base_manifest}\n[dev-dependencies]\nserde = \"1\"\n"),
    )
    .expect("add dev dependency");
    assert_rejected(check(&crate_dir), "[dev-dependencies]");

    fs::write(&manifest_path, base_manifest.replace("[workspace]", ""))
        .expect("remove workspace boundary");
    assert_rejected(check(&crate_dir), "empty [workspace]");

    fs::write(&manifest_path, &base_manifest).expect("restore manifest");
    fs::write(&source_path, format!("{base_source}\nuse std::fs;\n"))
        .expect("add ambient authority");
    assert_rejected(check(&crate_dir), "std::fs");

    fs::write(&source_path, &base_source).expect("restore source");
    fs::write(crate_dir.join("build.rs"), "fn main() {}\n").expect("add build script");
    assert_rejected(check(&crate_dir), "build.rs");

    fs::remove_dir_all(root).expect("remove temporary test root");
}
