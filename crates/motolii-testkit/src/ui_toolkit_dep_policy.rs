//! M3E-1: UI toolkit直接依存の許可リスト走査。
//!
//! 製品クレート(`crates/`)では`motolii-ui`だけがegui/winit系へ直接依存できる。
//! slint系クレートは`crates/`配下で直接依存ゼロ(許可リストなし)。
//! `spikes/`はworkspace外の隔離spikeのため対象外。
//!
//! 2026-08-18(icedホスト移行 M-0)から**柵は2本**になった。iced系を持てるのは
//! `motolii-shell-iced`だけ、という向きと、**その`motolii-shell-iced`がegui系を
//! 持てない**という逆向きである。後者は`UI_TOOLKIT_CRATE_ALLOWLIST`へ
//! 新しい殻を**入れないこと**だけで成立する — 絞め殺し移行の途中で、新しい殻へ
//! 古いtoolkitが滲み戻るのを止める柵で、名簿1行の不在がその実体である。

use std::path::{Path, PathBuf};

use cargo_metadata::{DependencyKind, Metadata, MetadataCommand};

/// egui/winit系を直接依存にできるcrate。**`motolii-shell-iced`は入れない。**
pub const UI_TOOLKIT_CRATE_ALLOWLIST: &[&str] = &["motolii-ui"];

/// iced系を直接依存にできるcrate。
///
/// 2026-08-18の[icedホスト移行裁定](../../../../docs/reviews/2026-08-18-iced-host-migration-decision.md)
/// のM-0。egui shellは並走したままなので、名簿は2つ並ぶ。
pub const ICED_CRATE_ALLOWLIST: &[&str] = &["motolii-shell-iced"];

/// egui/winitエコシステムの直接依存名か。
pub fn is_ui_toolkit_dependency(name: &str) -> bool {
    name == "egui"
        || name.starts_with("egui-")
        || name == "egui_tiles"
        || name == "eframe"
        || name == "winit"
        || matches!(name, "ecolor" | "emath" | "epaint" | "epaint_default_fonts")
}

/// slintエコシステムの直接依存名か。
pub fn is_slint_dependency(name: &str) -> bool {
    name == "slint" || name.starts_with("slint-") || name.starts_with("i-slint-")
}

/// icedエコシステムの直接依存名か。
///
/// 区切り(`_` / `-`)を要求するので、`icedream`のような**無関係にicedで始まる名前**は
/// 巻き込まない。逆に`motolii-shell-iced`のようなcrate名にも当たらない
/// (当たると自分自身を違反にしてしまう)。
pub fn is_iced_dependency(name: &str) -> bool {
    name == "iced" || name.starts_with("iced_") || name.starts_with("iced-")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiToolkitDepViolation {
    pub crate_name: String,
    pub manifest_path: PathBuf,
    pub dependency: String,
    pub section: String,
}

/// workspaceのCargo metadataを取得する。
pub fn load_workspace_metadata(workspace_root: &Path) -> Result<Metadata, cargo_metadata::Error> {
    MetadataCommand::new().current_dir(workspace_root).exec()
}

fn dependency_section(kind: DependencyKind, target: Option<String>) -> String {
    let base = match kind {
        DependencyKind::Normal => "dependencies",
        DependencyKind::Development => "dev-dependencies",
        DependencyKind::Build => "build-dependencies",
        _ => "dependencies",
    };
    match target {
        Some(platform) => format!("target.{platform}.{base}"),
        None => base.to_string(),
    }
}

/// Cargo.tomlキー名ではなく解決済みpackage名で審判する(`package = "egui"` renameを含む)。
fn resolved_dependency_name(dep: &cargo_metadata::Dependency) -> &str {
    dep.name.as_ref()
}

fn is_product_crate_manifest(workspace_root: &Path, manifest_path: &Path) -> bool {
    manifest_path.starts_with(workspace_root.join("crates"))
}

fn collect_violations<F>(
    metadata: &Metadata,
    workspace_root: &Path,
    allowlist: &[&str],
    is_target: F,
) -> Vec<UiToolkitDepViolation>
where
    F: Fn(&str) -> bool,
{
    let mut violations = Vec::new();

    for package in metadata.workspace_packages() {
        let manifest_path = package.manifest_path.as_std_path();
        if !is_product_crate_manifest(workspace_root, manifest_path) {
            continue;
        }
        if allowlist.contains(&package.name.as_ref()) {
            continue;
        }

        for dep in &package.dependencies {
            let resolved = resolved_dependency_name(dep);
            if !is_target(resolved) {
                continue;
            }
            violations.push(UiToolkitDepViolation {
                crate_name: package.name.to_string(),
                manifest_path: manifest_path.to_path_buf(),
                dependency: resolved.to_string(),
                section: dependency_section(dep.kind, dep.target.as_ref().map(ToString::to_string)),
            });
        }
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

/// Cargo metadataから、許可リスト外クレートのUI toolkit直接依存を返す。
pub fn find_ui_toolkit_violations(
    metadata: &Metadata,
    workspace_root: &Path,
    allowlist: &[&str],
) -> Vec<UiToolkitDepViolation> {
    collect_violations(
        metadata,
        workspace_root,
        allowlist,
        is_ui_toolkit_dependency,
    )
}

/// Cargo metadataから、許可リスト外クレートのiced直接依存を返す。
pub fn find_iced_violations(
    metadata: &Metadata,
    workspace_root: &Path,
    allowlist: &[&str],
) -> Vec<UiToolkitDepViolation> {
    collect_violations(metadata, workspace_root, allowlist, is_iced_dependency)
}

/// `crates/`配下を走査し、許可リスト外クレートのiced直接依存を返す。
pub fn find_iced_violations_in_crates(
    workspace_root: &Path,
    allowlist: &[&str],
) -> Vec<UiToolkitDepViolation> {
    let metadata = load_workspace_metadata(workspace_root).unwrap_or_else(|err| {
        panic!(
            "cargo metadata failed for {}: {err}",
            workspace_root.display()
        )
    });
    find_iced_violations(&metadata, workspace_root, allowlist)
}

/// Cargo metadataから、全クレートのslint直接依存を返す(許可リストなし)。
pub fn find_slint_violations(
    metadata: &Metadata,
    workspace_root: &Path,
) -> Vec<UiToolkitDepViolation> {
    collect_violations(metadata, workspace_root, &[], is_slint_dependency)
}

/// `crates/`配下を走査し、許可リスト外クレートのUI toolkit直接依存を返す。
pub fn find_ui_toolkit_violations_in_crates(
    workspace_root: &Path,
    allowlist: &[&str],
) -> Vec<UiToolkitDepViolation> {
    let metadata = load_workspace_metadata(workspace_root).unwrap_or_else(|err| {
        panic!(
            "cargo metadata failed for {}: {err}",
            workspace_root.display()
        )
    });
    find_ui_toolkit_violations(&metadata, workspace_root, allowlist)
}

/// `crates/`配下を走査し、全クレートのslint直接依存を返す(許可リストなし)。
pub fn find_slint_violations_in_crates(workspace_root: &Path) -> Vec<UiToolkitDepViolation> {
    let metadata = load_workspace_metadata(workspace_root).unwrap_or_else(|err| {
        panic!(
            "cargo metadata failed for {}: {err}",
            workspace_root.display()
        )
    });
    find_slint_violations(&metadata, workspace_root)
}
