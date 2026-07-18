//! plugin-authoring §7 チェックリストの機械化(INF-7a/7b)。
//!
//! - ベンダー/OS固有GPU APIの依存・ソース参照をワークスペース全体で拒否する(F-9、§3-1)。
//! - `motolii-plugin` 公開面(非テストコード)のpanic経路を拒否する(§3-7。clippy lintの補完で
//!   `assert!` 系もここで落とす)。
//! - A1S §3: `plugins/motolii-plugin-*` の依存allowlist、source閉集合、panic走査。
//!
//! 「違反負例が赤になる」証明はフィクスチャ文字列に対する単体テストで行う(ツリーに違反を
//! 置くとCI自体が赤になるため)。実ツリーへの適用テストは違反ゼロを主張する。

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// 製品経路で直接依存を禁止するクレート(F-9)。公認のGPU抽象はwgpu/WGSLのみ。
/// wgpu内部のバックエンド実装が持つ推移的依存は対象外(直接依存のみ見る)。
///
/// `windows` / `windows-sys` は入れない — F-9の本命はGPUベンダーAPIであり、
/// M3のegui/cpal等の正当なOS依存まで赤にすると基盤が進めなくなる。
const DENIED_CRATES: &[&str] = &[
    // CUDA
    "cudarc",
    "cust",
    "rustacuda",
    "cuda-runtime-sys",
    "cuda-sys",
    // Metal (wgpu経由以外の直依存)
    "metal",
    "objc2-metal",
    // Direct3D / Vulkan直叩き
    "d3d12",
    "ash",
    "vulkano",
    "erupt",
    // OpenCL
    "opencl3",
    "ocl",
];

/// ソース中で禁止するベンダーAPIトークン(コメント除去後・識別子境界つきで判定)。
/// OS窓口(`windows::`)は禁止しない(上記クレート方針と同じ)。GPUベンダー経路のみ。
const DENIED_SRC_TOKENS: &[&str] = &[
    "cudarc",
    "rustacuda",
    "cuda_runtime",
    "cuda_sys",
    "metal::",
    "objc2_metal",
    "d3d12::",
    "vulkano::",
    "erupt::",
    "opencl::",
    "ocl::",
    "ash::",
];

/// 公開面(非テスト)で禁止するpanic経路。`debug_assert!`は境界判定で許容される。
const DENIED_PANIC_TOKENS: &[&str] = &[
    "panic!(",
    "assert!(",
    "assert_eq!(",
    "assert_ne!(",
    ".unwrap()",
    ".expect(",
    "todo!(",
    "unimplemented!(",
    "unreachable!(",
];

const PLUGIN_CRATE_ALLOWED_DEPS: &[&str] = &["motolii-plugin"];

/// Opacity crate が実際に名指しした `motolii_plugin::` 公開pathの閉集合(A1S §3)。
const OPACITY_ALLOWED_PUBLIC_PATHS: &[&str] = &[
    "motolii_plugin::bytemuck",
    "motolii_plugin::wgpu",
    "motolii_plugin::F64Domain",
    "motolii_plugin::FilterPlugin",
    "motolii_plugin::GpuCtx",
    "motolii_plugin::NodeDesc",
    "motolii_plugin::ParamDef",
    "motolii_plugin::PipelineCache",
    "motolii_plugin::PipelineCacheKey",
    "motolii_plugin::PluginContract",
    "motolii_plugin::PluginError",
    "motolii_plugin::PluginId",
    "motolii_plugin::PluginKind",
    "motolii_plugin::RenderCtx",
    "motolii_plugin::ResolvedParams",
    "motolii_plugin::TextureRef",
    "motolii_plugin::Value",
    "motolii_plugin::ValueType",
];

/// Sine crate が実際に名指しした `motolii_plugin::` 公開pathの閉集合(A2S / A1S §3同型)。
const SINE_ALLOWED_PUBLIC_PATHS: &[&str] = &[
    "motolii_plugin::DataTrack",
    "motolii_plugin::Fps",
    "motolii_plugin::MigrationOp",
    "motolii_plugin::MigrationStep",
    "motolii_plugin::NodeDesc",
    "motolii_plugin::ParamDef",
    "motolii_plugin::ParamDriverContext",
    "motolii_plugin::ParamDriverPlugin",
    "motolii_plugin::PluginContract",
    "motolii_plugin::PluginError",
    "motolii_plugin::PluginId",
    "motolii_plugin::PluginKind",
    "motolii_plugin::RationalTime",
    "motolii_plugin::ResolvedParams",
    "motolii_plugin::Value",
    "motolii_plugin::ValueType",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-plugin は workspace 直下にある")
        .to_path_buf()
}

/// 直前の文字が識別子構成文字でない位置でのみトークン一致とみなす
/// (`std::hash::` が `ash::` に誤爆しない、`debug_assert!` が `assert!` に誤爆しない)。
fn contains_token(haystack: &str, token: &str) -> bool {
    // 境界判定は識別子で始まるトークンのみ(`.unwrap()`等は直前が必ず識別子なので常に一致)。
    let needs_boundary = token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(token) {
        let at = from + pos;
        let boundary = !needs_boundary
            || at == 0
            || !haystack[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        if boundary {
            return true;
        }
        from = at + 1;
    }
    false
}

/// 行コメント(`//`)以降を落とす。文字列内URLの`//`も落ちるが、走査目的では過剰除去で問題ない。
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DependencyTableKind {
    Normal,
    DevOrBuild,
}

/// Cargo.tomlのdependencies系セクションから直接依存クレート名を抜く。
/// `foo = { package = "bar" }` のリネームは実クレート名(bar)も返す。
/// `[dependencies.cudarc]` のようなネスト表形式もクレート名として拾う。
fn direct_dependency_names(manifest: &str, kind: DependencyTableKind) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_deps = false;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            let section = line.trim_matches(|c| c == '[' || c == ']');
            if let Some(crate_name) = nested_dependency_crate(section, kind) {
                // [dependencies.cudarc] / [dev-dependencies.metal] / [target.…\.dependencies.ash]
                names.push(crate_name.to_string());
                in_deps = false;
            } else {
                in_deps = is_dependencies_table(section, kind);
            }
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let mut name = key.trim().trim_matches('"').to_string();
            if name.ends_with(".workspace") {
                name.truncate(name.len() - ".workspace".len());
            }
            names.push(name);
            if let Some(rest) = value.split("package").nth(1) {
                if let Some(pkg) = rest.split('"').nth(1) {
                    names.push(pkg.to_string());
                }
            }
        }
    }
    names
}

fn is_dependencies_table(section: &str, kind: DependencyTableKind) -> bool {
    match kind {
        DependencyTableKind::Normal => {
            section == "dependencies" || section.ends_with(".dependencies")
        }
        DependencyTableKind::DevOrBuild => {
            section == "dev-dependencies"
                || section.ends_with(".dev-dependencies")
                || section == "build-dependencies"
                || section.ends_with(".build-dependencies")
        }
    }
}

/// `[dependencies.cudarc]` → `Some("cudarc")`。通常の`[dependencies]`は`None`。
fn nested_dependency_crate(section: &str, kind: DependencyTableKind) -> Option<&str> {
    let markers: &[&str] = match kind {
        DependencyTableKind::Normal => &["dependencies."],
        DependencyTableKind::DevOrBuild => &["dev-dependencies.", "build-dependencies."],
    };
    for marker in markers {
        if let Some(rest) = section.rsplit_once(marker) {
            // target.'cfg(…)'.dependencies.foo → marker照合は dependencies. で foo
            let name = rest.1;
            if !name.is_empty() && !name.contains('.') {
                return Some(name);
            }
        }
    }
    None
}

fn manifest_violations(manifest: &str) -> Vec<String> {
    direct_dependency_names(manifest, DependencyTableKind::Normal)
        .into_iter()
        .filter(|name| DENIED_CRATES.contains(&name.as_str()))
        .collect()
}

fn source_violations(src: &str) -> Vec<&'static str> {
    let code = strip_line_comments(src);
    DENIED_SRC_TOKENS
        .iter()
        .copied()
        .filter(|token| contains_token(&code, token))
        .collect()
}

fn panic_violations(src: &str) -> Vec<&'static str> {
    // `#[cfg(test)]` 以降はテストコード(このリポジトリの規約: テストmodは末尾)。
    let non_test = src.split("#[cfg(test)]").next().unwrap_or("");
    let code = strip_line_comments(non_test);
    DENIED_PANIC_TOKENS
        .iter()
        .copied()
        .filter(|token| contains_token(&code, token))
        .collect()
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn plugin_crate_dirs() -> Vec<PathBuf> {
    let root = workspace_root().join("plugins");
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("motolii-plugin-"))
        })
        .collect()
}

fn manifest_has_proc_macro(manifest: &str) -> bool {
    manifest.contains("proc-macro = true") || manifest.contains("proc_macro = true")
}

fn plugin_normal_dependency_violations(manifest: &str) -> Vec<String> {
    let names: BTreeSet<_> = direct_dependency_names(manifest, DependencyTableKind::Normal)
        .into_iter()
        .collect();
    let mut violations: Vec<_> = names
        .iter()
        .filter(|name| !PLUGIN_CRATE_ALLOWED_DEPS.contains(&name.as_str()))
        .cloned()
        .collect();
    for required in PLUGIN_CRATE_ALLOWED_DEPS {
        if !names.contains(*required) {
            violations.push(format!("missing required dependency `{required}`"));
        }
    }
    violations
}

fn plugin_dev_build_dependency_violations(manifest: &str) -> Vec<String> {
    direct_dependency_names(manifest, DependencyTableKind::DevOrBuild)
}

fn motolii_path_prefixes(src: &str) -> Vec<String> {
    let code = strip_line_comments(src);
    let mut paths = BTreeSet::new();
    let mut search_from = 0usize;
    while let Some(pos) = code[search_from..].find("motolii_") {
        let at = search_from + pos;
        let rest = &code[at..];
        let crate_end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let crate_name = &rest[..crate_end];
        if rest.get(crate_end..crate_end + 2) == Some("::") {
            let after = &rest[crate_end + 2..];
            let item_end = after
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(after.len());
            paths.insert(format!("{}::{}", crate_name, &after[..item_end]));
        } else {
            paths.insert(crate_name.to_string());
        }
        search_from = at + 1;
    }
    paths.into_iter().collect()
}

fn is_valid_motolii_plugin_path(path: &str) -> bool {
    path != "motolii_plugin"
        && path.starts_with("motolii_plugin::")
        && !path.contains('{')
        && path.len() > "motolii_plugin::".len()
}

fn plugin_src_motolii_violations(src: &str) -> Vec<String> {
    let code = strip_line_comments(src);
    let mut violations = Vec::new();
    if code.contains("motolii_plugin::{") {
        violations.push("motolii_plugin::{".to_string());
    }
    violations.extend(
        motolii_path_prefixes(src)
            .into_iter()
            .filter(|path| !is_valid_motolii_plugin_path(path)),
    );
    violations
}

fn plugin_test_motolii_violations(src: &str, own_crate: &str) -> Vec<String> {
    let allowed_own = format!("{own_crate}::");
    motolii_path_prefixes(src)
        .into_iter()
        .filter(|path| {
            !is_valid_motolii_plugin_path(path)
                && (!path.starts_with(&allowed_own) || path.len() <= allowed_own.len())
        })
        .collect()
}

fn opacity_observed_public_paths(src: &str) -> BTreeSet<String> {
    let code = strip_line_comments(src);
    let mut paths = BTreeSet::new();
    for line in code.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("use motolii_plugin::") else {
            continue;
        };
        if rest.starts_with('{') {
            continue;
        }
        let item = rest
            .trim_end_matches(';')
            .split([' ', '{'])
            .next()
            .unwrap_or("");
        if !item.is_empty() {
            paths.insert(format!("motolii_plugin::{item}"));
        }
    }
    for path in motolii_path_prefixes(src) {
        if is_valid_motolii_plugin_path(&path) {
            paths.insert(path);
        }
    }
    paths
}

fn opacity_public_path_violations(src: &str) -> Vec<String> {
    public_path_violations(src, OPACITY_ALLOWED_PUBLIC_PATHS)
}

fn sine_public_path_violations(src: &str) -> Vec<String> {
    public_path_violations(src, SINE_ALLOWED_PUBLIC_PATHS)
}

fn public_path_violations(src: &str, expected_paths: &[&str]) -> Vec<String> {
    let mut violations = Vec::new();
    if strip_line_comments(src).contains("motolii_plugin::{") {
        violations.push("brace import motolii_plugin::{...} is forbidden".to_string());
    }
    let observed = opacity_observed_public_paths(src);
    let expected: BTreeSet<_> = expected_paths
        .iter()
        .map(|path| (*path).to_string())
        .collect();
    for path in observed.difference(&expected) {
        violations.push(format!("unknown public path: {path}"));
    }
    for path in expected.difference(&observed) {
        violations.push(format!("missing expected public path: {path}"));
    }
    violations
}

const CLI_DENIED_SINE_CRATES: &[&str] = &["motolii-plugin-sine"];

const CLI_DENIED_SINE_SRC_TOKENS: &[&str] = &["motolii_plugin_sine", "motolii-plugin-sine"];

fn cli_sine_dependency_violations(manifest: &str) -> Vec<String> {
    let mut violations = direct_dependency_names(manifest, DependencyTableKind::Normal)
        .into_iter()
        .filter(|name| CLI_DENIED_SINE_CRATES.contains(&name.as_str()))
        .collect::<Vec<_>>();
    violations.extend(
        direct_dependency_names(manifest, DependencyTableKind::DevOrBuild)
            .into_iter()
            .filter(|name| CLI_DENIED_SINE_CRATES.contains(&name.as_str())),
    );
    violations
}

fn cli_sine_source_violations(src: &str) -> Vec<&'static str> {
    let code = strip_line_comments(src);
    CLI_DENIED_SINE_SRC_TOKENS
        .iter()
        .copied()
        .filter(|token| contains_token(&code, token))
        .collect()
}

fn cli_crate_dir() -> PathBuf {
    workspace_root().join("crates/motolii-cli")
}

#[test]
fn workspace_has_no_denied_vendor_crate_dependency() {
    let root = workspace_root();
    let mut manifests = vec![root.join("Cargo.toml")];
    for entry in fs::read_dir(root.join("crates")).unwrap().flatten() {
        let manifest = entry.path().join("Cargo.toml");
        if manifest.is_file() {
            manifests.push(manifest);
        }
    }
    assert!(manifests.len() > 5, "クレート列挙に失敗している");
    for manifest in manifests {
        let text = fs::read_to_string(&manifest).unwrap();
        let violations = manifest_violations(&text);
        assert!(
            violations.is_empty(),
            "{} にベンダーAPIクレートへの直接依存: {:?}(F-9。wgpu/WGSLのみ公認)",
            manifest.display(),
            violations
        );
    }
}

#[test]
fn product_sources_have_no_vendor_api_tokens() {
    let root = workspace_root();
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join("crates")).unwrap().flatten() {
        collect_rust_files(&entry.path().join("src"), &mut files);
        collect_rust_files(&entry.path().join("tests"), &mut files);
    }
    assert!(files.len() > 10, "ソース列挙に失敗している");
    for file in files {
        // このファイル自身は禁止トークンをフィクスチャとして含むため対象外。
        if file.ends_with("motolii-plugin/tests/conformance.rs") {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap();
        let violations = source_violations(&text);
        assert!(
            violations.is_empty(),
            "{} にベンダーAPI参照: {:?}(F-9。wgpu/WGSLのみ公認)",
            file.display(),
            violations
        );
    }
}

#[test]
fn plugin_public_api_has_no_panic_paths() {
    let root = workspace_root();
    let mut files = Vec::new();
    collect_rust_files(&root.join("crates/motolii-plugin/src"), &mut files);
    assert!(!files.is_empty());
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let violations = panic_violations(&text);
        assert!(
            violations.is_empty(),
            "{} の非テストコードにpanic経路: {:?}(§3-7。入力起因はPluginErrorで返す)",
            file.display(),
            violations
        );
    }
}

// ---- 負例(スキャナが違反を赤にできる証明)と誤爆回避の検証 ----

#[test]
fn deny_scanner_flags_cuda_dependency() {
    let manifest = "[package]\nname = \"evil\"\n[dependencies]\ncudarc = \"0.11\"\n";
    assert_eq!(manifest_violations(manifest), vec!["cudarc".to_string()]);
}

#[test]
fn deny_scanner_flags_renamed_vendor_package() {
    let manifest = "[dependencies]\ngfx = { package = \"metal\", version = \"0.27\" }\n";
    assert_eq!(manifest_violations(manifest), vec!["metal".to_string()]);
}

#[test]
fn deny_scanner_flags_nested_dependency_table() {
    let manifest = "[package]\nname = \"evil\"\n[dependencies.cudarc]\nversion = \"0.11\"\n";
    assert_eq!(manifest_violations(manifest), vec!["cudarc".to_string()]);
}

#[test]
fn deny_scanner_flags_target_nested_vendor_table() {
    let manifest = "[target.'cfg(windows)'.dependencies.d3d12]\nversion = \"0.7\"\n";
    assert_eq!(manifest_violations(manifest), vec!["d3d12".to_string()]);
}

#[test]
fn deny_scanner_ignores_wgpu_and_non_dep_sections() {
    let manifest = "[dependencies]\nwgpu = \"29\"\n[package.metadata]\nmetal = \"not-a-dep\"\n";
    assert!(manifest_violations(manifest).is_empty());
}

#[test]
fn src_scanner_flags_vendor_api_use() {
    let src = "fn boot() {\n    let dev = metal::Device::system_default();\n}\n";
    assert_eq!(source_violations(src), vec!["metal::"]);
}

#[test]
fn src_scanner_ignores_comments_and_hash_paths() {
    let src = "// CUDAやmetal::の話はコメントなら許される\nuse std::hash::Hash;\n";
    assert!(source_violations(src).is_empty());
}

#[test]
fn panic_scanner_flags_unwrap_in_public_api() {
    let src = "pub fn f(v: Option<u32>) -> u32 {\n    v.unwrap()\n}\n";
    assert_eq!(panic_violations(src), vec![".unwrap()"]);
}

#[test]
fn panic_scanner_allows_debug_assert_and_test_code() {
    let src = "pub fn f(x: u32) {\n    debug_assert!(x > 0);\n}\n#[cfg(test)]\nmod tests {\n    fn g() { panic!(\"ok in tests\"); }\n}\n";
    assert!(panic_violations(src).is_empty());
}

#[test]
fn workspace_has_external_plugin_crates() {
    let plugin_dirs = plugin_crate_dirs();
    assert!(
        !plugin_dirs.is_empty(),
        "plugins/motolii-plugin-* が1件以上必要(A1S §3)"
    );
}

#[test]
fn external_plugin_crates_only_depend_on_motolii_plugin() {
    for dir in plugin_crate_dirs() {
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        let violations = plugin_normal_dependency_violations(&manifest);
        assert!(
            violations.is_empty(),
            "{} の[dependencies]がallowlist外: {:?}",
            dir.display(),
            violations
        );
        let dev_build = plugin_dev_build_dependency_violations(&manifest);
        assert!(
            dev_build.is_empty(),
            "{} のdev/build依存は禁止(A1S §2.2): {:?}",
            dir.display(),
            dev_build
        );
        assert!(
            !dir.join("build.rs").is_file(),
            "{} に build.rs は禁止(A1S §2.2)",
            dir.display()
        );
        assert!(
            !manifest_has_proc_macro(&manifest),
            "{} に proc-macro は禁止(A1S §2.2)",
            dir.display()
        );
    }
}

#[test]
fn external_plugin_sources_use_closed_motolii_paths() {
    for dir in plugin_crate_dirs() {
        let own_crate = dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .replace('-', "_");
        let src_dir = dir.join("src");
        let tests_dir = dir.join("tests");
        let mut files = Vec::new();
        collect_rust_files(&src_dir, &mut files);
        for file in &files {
            let text = fs::read_to_string(file).unwrap();
            let violations = plugin_src_motolii_violations(&text);
            assert!(
                violations.is_empty(),
                "{} のsrcが motolii_plugin:: 以外を参照: {:?}",
                file.display(),
                violations
            );
            let panic_hits = panic_violations(&text);
            assert!(
                panic_hits.is_empty(),
                "{} の非テストコードにpanic経路: {:?}",
                file.display(),
                panic_hits
            );
            let vendor_hits = source_violations(&text);
            assert!(
                vendor_hits.is_empty(),
                "{} にベンダーAPI参照: {:?}",
                file.display(),
                vendor_hits
            );
        }
        files.clear();
        collect_rust_files(&tests_dir, &mut files);
        for file in files {
            let text = fs::read_to_string(&file).unwrap();
            let violations = plugin_test_motolii_violations(&text, &own_crate);
            assert!(
                violations.is_empty(),
                "{} のtestsが許可外crateを参照: {:?}",
                file.display(),
                violations
            );
        }
    }
}

#[test]
fn opacity_plugin_public_api_paths_are_closed_set() {
    let opacity_src = workspace_root().join("plugins/motolii-plugin-opacity/src/lib.rs");
    let text = fs::read_to_string(opacity_src).unwrap();
    let violations = opacity_public_path_violations(&text);
    assert!(
        violations.is_empty(),
        "Opacity crate の未知公開path: {:?}",
        violations
    );
}

#[test]
fn sine_plugin_public_api_paths_are_closed_set() {
    let sine_src = workspace_root().join("plugins/motolii-plugin-sine/src/lib.rs");
    let text = fs::read_to_string(sine_src).unwrap();
    let violations = sine_public_path_violations(&text);
    assert!(
        violations.is_empty(),
        "Sine crate の未知公開path: {:?}",
        violations
    );
}

#[test]
fn cli_has_no_direct_sine_dependency_or_import() {
    let cli_dir = cli_crate_dir();
    let manifest = fs::read_to_string(cli_dir.join("Cargo.toml")).unwrap();
    let dep_violations = cli_sine_dependency_violations(&manifest);
    assert!(
        dep_violations.is_empty(),
        "motolii-cli must not depend on motolii-plugin-sine directly (VSM-A2): {:?}",
        dep_violations
    );

    let mut files = Vec::new();
    collect_rust_files(&cli_dir.join("src"), &mut files);
    collect_rust_files(&cli_dir.join("tests"), &mut files);
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        let violations = cli_sine_source_violations(&text);
        assert!(
            violations.is_empty(),
            "{} must not import motolii-plugin-sine directly (VSM-A2): {:?}",
            file.display(),
            violations
        );
    }
}

#[test]
fn cli_sine_dependency_scanner_flags_manifest_violation() {
    let manifest = "[dependencies]\nmotolii-plugin-sine.workspace = true\n";
    assert_eq!(
        cli_sine_dependency_violations(manifest),
        vec!["motolii-plugin-sine".to_string()]
    );
}

#[test]
fn cli_sine_dependency_scanner_flags_dev_dependency() {
    let manifest = "[dev-dependencies]\nmotolii-plugin-sine = { path = \"../../plugins/motolii-plugin-sine\" }\n";
    assert_eq!(
        cli_sine_dependency_violations(manifest),
        vec!["motolii-plugin-sine".to_string()]
    );
}

#[test]
fn cli_sine_import_scanner_flags_source_violation() {
    let src = "use motolii_plugin_sine::SINE_PARAM_DRIVER;\n";
    assert_eq!(cli_sine_source_violations(src), vec!["motolii_plugin_sine"]);
}

#[test]
fn plugin_facade_has_no_central_migrate_plugin_params() {
    let lib_src =
        fs::read_to_string(workspace_root().join("crates/motolii-plugin/src/lib.rs")).unwrap();
    let code = strip_line_comments(&lib_src);
    assert!(
        !code.contains("fn migrate_plugin_params"),
        "central migrate_plugin_params must be removed (VSM-A2)"
    );
    assert!(
        !code.contains("\"core.param.sine\" =>"),
        "central Sine ID match must not remain in motolii-plugin facade"
    );
}

#[test]
fn plugin_allowlist_scanner_flags_extra_dependency() {
    let manifest = "[dependencies]\nmotolii-core = \"0.1\"\n";
    assert_eq!(
        plugin_normal_dependency_violations(manifest),
        vec![
            "motolii-core".to_string(),
            "missing required dependency `motolii-plugin`".to_string(),
        ]
    );
}

#[test]
fn plugin_allowlist_scanner_rejects_missing_normal_dependency() {
    let manifest = "[dependencies]\n";
    assert_eq!(
        plugin_normal_dependency_violations(manifest),
        vec!["missing required dependency `motolii-plugin`".to_string()]
    );
}

#[test]
fn plugin_allowlist_scanner_flags_dev_dependency_even_when_allowed_name() {
    let manifest = "[dev-dependencies]\nmotolii-plugin = { path = \"../crates/motolii-plugin\" }\n";
    assert_eq!(
        plugin_dev_build_dependency_violations(manifest),
        vec!["motolii-plugin".to_string()]
    );
    assert_eq!(
        plugin_normal_dependency_violations(manifest),
        vec!["missing required dependency `motolii-plugin`".to_string()]
    );
}

#[test]
fn plugin_allowlist_scanner_flags_build_dependency() {
    let manifest = "[build-dependencies]\nmotolii-plugin.workspace = true\n";
    assert_eq!(
        plugin_dev_build_dependency_violations(manifest),
        vec!["motolii-plugin".to_string()]
    );
}

#[test]
fn plugin_allowlist_scanner_flags_renamed_dev_dependency() {
    let manifest = "[dev-dependencies]\nfacade = { package = \"motolii-plugin\", path = \"../crates/motolii-plugin\" }\n";
    assert_eq!(
        plugin_dev_build_dependency_violations(manifest),
        vec!["facade".to_string(), "motolii-plugin".to_string()]
    );
}

#[test]
fn plugin_path_scanner_flags_private_crate_use() {
    let src = "use motolii_gpu::GpuCtx;\n";
    assert_eq!(
        plugin_src_motolii_violations(src),
        vec!["motolii_gpu::GpuCtx".to_string()]
    );
}

#[test]
fn plugin_path_scanner_rejects_bare_facade_name() {
    assert_eq!(
        plugin_src_motolii_violations("use motolii_plugin;\n"),
        vec!["motolii_plugin".to_string()]
    );
    assert_eq!(
        plugin_test_motolii_violations("use motolii_plugin_opacity;\n", "motolii_plugin_opacity"),
        vec!["motolii_plugin_opacity".to_string()]
    );
}

#[test]
fn opacity_public_path_scanner_flags_unknown_facade_path() {
    let src = "use motolii_plugin::PluginRuntime;\n";
    let violations = opacity_public_path_violations(src);
    assert!(
        violations
            .iter()
            .any(|v| v == "unknown public path: motolii_plugin::PluginRuntime"),
        "{violations:?}"
    );
}

#[test]
fn opacity_public_path_scanner_flags_brace_import() {
    let src = "use motolii_plugin::{FilterPlugin, PluginRuntime};\n";
    let violations = opacity_public_path_violations(src);
    assert!(
        violations.iter().any(|v| v.contains("brace import")),
        "{violations:?}"
    );
}

#[test]
fn opacity_public_path_scanner_flags_missing_expected_path() {
    let src = "use motolii_plugin::FilterPlugin;\n";
    let violations = opacity_public_path_violations(src);
    assert!(
        violations
            .iter()
            .any(|v| v.starts_with("missing expected public path:")),
        "{violations:?}"
    );
}
