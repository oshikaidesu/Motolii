//! M3E-1: slint 直接依存の許可リスト走査。
//!
//! 製品クレート(`crates/`)は UI クレートのみが slint 系クレートへ直接依存できる。
//! `spikes/` は workspace 外の隔離スパイクのため対象外。

use std::path::{Path, PathBuf};

/// 将来の UI クレート名。未作成でも許可リストに予約する。
pub const SLINT_UI_CRATE_ALLOWLIST: &[&str] = &["motolii-ui"];

/// slint エコシステムの直接依存名か。
pub fn is_slint_dependency(name: &str) -> bool {
    name == "slint" || name.starts_with("slint-") || name.starts_with("i-slint-")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlintDepViolation {
    pub crate_name: String,
    pub manifest_path: PathBuf,
    pub dependency: String,
    pub section: String,
}

fn is_dependency_section(section: &str) -> bool {
    section == "dependencies"
        || section == "dev-dependencies"
        || section == "build-dependencies"
        || section.ends_with(".dependencies")
}

fn parse_package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == "name" {
                return Some(unquote_toml_value(value.trim()));
            }
        }
    }
    None
}

fn unquote_toml_value(value: &str) -> String {
    value.trim_matches('"').to_string()
}

fn dependency_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let key = trimmed.split('=').next()?.trim();
    if key.is_empty() || key.starts_with('[') {
        return None;
    }
    Some(key)
}

/// 単一 `Cargo.toml` の直接 slint 依存を列挙する。
pub fn scan_cargo_toml(manifest_path: &Path, manifest_text: &str) -> Vec<SlintDepViolation> {
    let crate_name = parse_package_name(manifest_text).unwrap_or_else(|| {
        manifest_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    let mut section = String::new();
    let mut violations = Vec::new();

    for line in manifest_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }
        if !is_dependency_section(&section) {
            continue;
        }
        let Some(key) = dependency_key(trimmed) else {
            continue;
        };
        if is_slint_dependency(key) {
            violations.push(SlintDepViolation {
                crate_name: crate_name.clone(),
                manifest_path: manifest_path.to_path_buf(),
                dependency: key.to_string(),
                section: section.clone(),
            });
        }
    }

    violations
}

/// `crates/` 配下を走査し、許可リスト外クレートの slint 直接依存を返す。
pub fn find_slint_violations_in_crates(
    workspace_root: &Path,
    allowlist: &[&str],
) -> Vec<SlintDepViolation> {
    let crates_dir = workspace_root.join("crates");
    let Ok(entries) = std::fs::read_dir(&crates_dir) else {
        return Vec::new();
    };

    let mut violations = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest_path) else {
            continue;
        };
        let crate_name = parse_package_name(&text).unwrap_or_else(|| {
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });
        if allowlist.contains(&crate_name.as_str()) {
            continue;
        }
        violations.extend(scan_cargo_toml(&manifest_path, &text));
    }

    violations.sort_by(|a, b| {
        (&a.crate_name, &a.dependency, &a.section, &a.manifest_path).cmp(&(
            &b.crate_name,
            &b.dependency,
            &b.section,
            &b.manifest_path,
        ))
    });
    violations
}
