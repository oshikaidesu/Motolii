//! ビルド可否・TextInput使用・set_ime_allowed静的検査。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use walkdir::WalkDir;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn spike_source_uses_text_input() {
    let main_rs = fs::read_to_string(manifest_dir().join("src/main.rs")).unwrap();
    assert!(
        main_rs.contains("TextInput"),
        "IMEスパイクは Slint TextInput を使うこと (仕様: M3ガード1)"
    );
}

#[test]
fn cargo_build_succeeds() {
    let status = Command::new("cargo")
        .args(["build", "--quiet"])
        .current_dir(manifest_dir())
        .status()
        .expect("cargo build");
    assert!(status.success(), "spikes/ime-acceptance must build");
}

#[test]
fn winit_or_slint_sources_reference_set_ime_allowed() {
    let lock = fs::read_to_string(manifest_dir().join("Cargo.lock")).unwrap();
    let winit_ver = extract_dep_version(&lock, "winit").expect("winit in Cargo.lock");
    let slint_ver = extract_dep_version(&lock, "slint").expect("slint in Cargo.lock");

    let registry = cargo_registry_src();
    let winit_dir = find_crate_dir(&registry, "winit", &winit_ver);
    let slint_dir = find_crate_dir(&registry, "slint", &slint_ver);

    let hits = find_set_ime_allowed_in_tree(&winit_dir)
        .into_iter()
        .chain(find_set_ime_allowed_in_tree(&slint_dir))
        .collect::<Vec<_>>();

    assert!(
        !hits.is_empty(),
        "set_ime_allowed not found in winit-{winit_ver} or slint-{slint_ver} under {}",
        registry.display()
    );
}

fn extract_dep_version(lock: &str, name: &str) -> Option<String> {
    let pattern = format!("name = \"{name}\"\nversion = \"([^\"]+)\"");
    let re = Regex::new(&pattern).unwrap();
    re.captures(lock).map(|c| c[1].to_string())
}

fn cargo_registry_src() -> PathBuf {
    if let Ok(dir) = std::env::var("CARGO_HOME") {
        let p = PathBuf::from(dir).join("registry/src");
        if p.is_dir() {
            return p;
        }
    }
    for candidate in [
        "/usr/local/cargo/registry/src",
        "/home/ubuntu/.cargo/registry/src",
    ] {
        let p = PathBuf::from(candidate);
        if p.is_dir() {
            return p;
        }
    }
    PathBuf::from("/usr/local/cargo/registry/src")
}

fn find_crate_dir(registry: &Path, name: &str, version: &str) -> PathBuf {
    if let Ok(entries) = fs::read_dir(registry) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let candidate = path.join(format!("{name}-{version}"));
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    registry.join(format!("{name}-{version}"))
}

fn find_set_ime_allowed_in_tree(root: &Path) -> Vec<String> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "rs" | "cpp" | "cc" | "c") {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if content.contains("set_ime_allowed") {
            hits.push(path.display().to_string());
        }
    }
    hits
}
