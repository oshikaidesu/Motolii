//! plugin-authoring §7 チェックリストの機械化(INF-7a/7b)。
//!
//! - ベンダー/OS固有GPU APIの依存・ソース参照をワークスペース全体で拒否する(F-9、§3-1)。
//! - `motolii-plugin` 公開面(非テストコード)のpanic経路を拒否する(§3-7。clippy lintの補完で
//!   `assert!` 系もここで落とす)。
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
/// M3のSlint/cpal等の正当なOS依存まで赤にすると基盤が進めなくなる。
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

/// Cargo.tomlのdependencies系セクションから直接依存クレート名を抜く。
/// `foo = { package = "bar" }` のリネームは実クレート名(bar)も返す。
/// `[dependencies.cudarc]` のようなネスト表形式もクレート名として拾う。
fn dependency_entries(manifest: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut section = None;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            let name = line.trim_matches(|c| c == '[' || c == ']');
            if let Some(crate_name) = nested_dependency_crate(name) {
                // [dependencies.cudarc] / [dev-dependencies.metal] / [target.…\.dependencies.ash]
                entries.push((name.to_string(), crate_name.to_string()));
                section = None;
            } else {
                section = is_dependencies_table(name).then(|| name.to_string());
            }
            continue;
        }
        let Some(section) = &section else {
            continue;
        };
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let dependency = key
                .trim()
                .trim_matches('"')
                .split('.')
                .next()
                .unwrap_or("")
                .to_string();
            entries.push((section.clone(), dependency));
            if let Some(rest) = value.split("package").nth(1) {
                if let Some(pkg) = rest.split('"').nth(1) {
                    entries.push((section.clone(), pkg.to_string()));
                }
            }
        }
    }
    entries
}

fn direct_dependency_names(manifest: &str) -> Vec<String> {
    dependency_entries(manifest)
        .into_iter()
        .map(|(_, name)| name)
        .collect()
}

fn is_dependencies_table(section: &str) -> bool {
    section == "dependencies"
        || section.ends_with(".dependencies")
        || section == "dev-dependencies"
        || section.ends_with(".dev-dependencies")
        || section == "build-dependencies"
        || section.ends_with(".build-dependencies")
}

/// `[dependencies.cudarc]` → `Some("cudarc")`。通常の`[dependencies]`は`None`。
fn nested_dependency_crate(section: &str) -> Option<&str> {
    for marker in ["dependencies.", "dev-dependencies.", "build-dependencies."] {
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
    direct_dependency_names(manifest)
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

/// `plugins/motolii-plugin-*` 配下の外部参照crateを列挙する。
fn external_plugin_crates() -> Vec<PathBuf> {
    let plugins_dir = workspace_root().join("plugins");
    let Ok(entries) = fs::read_dir(&plugins_dir) else {
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

fn external_plugin_manifests() -> Vec<PathBuf> {
    external_plugin_crates()
        .into_iter()
        .map(|dir| dir.join("Cargo.toml"))
        .collect()
}

fn external_dependency_violations(manifest: &str) -> Vec<String> {
    let mut violations = Vec::new();
    for (section, name) in dependency_entries(manifest) {
        let is_dev_or_build =
            section.contains("dev-dependencies") || section.contains("build-dependencies");
        if is_dev_or_build {
            violations.push(format!("{section} must be empty: `{name}`"));
            continue;
        }
        if name != "motolii-plugin" {
            violations.push(format!("{section}: disallowed dependency `{name}`"));
        }
    }
    violations
}

fn external_manifest_violations(manifest: &str, has_build_rs: bool) -> Vec<String> {
    let mut violations = external_dependency_violations(manifest);
    if has_build_rs {
        violations.push("build.rs is forbidden".to_string());
    }
    if strip_line_comments(manifest).contains("proc-macro = true") {
        violations.push("proc-macro = true is forbidden".to_string());
    }
    violations
}

/// `motolii_` pathは`motolii_plugin::`だけを許す(src)。testsでは自crate名も許す。
fn motolii_path_violations(src: &str, own_crate: Option<&str>) -> Vec<String> {
    let code = strip_line_comments(src);
    let mut violations = Vec::new();
    let mut from = 0;
    while let Some(pos) = code[from..].find("motolii_") {
        let at = from + pos;
        let rest = &code[at..];
        if rest.starts_with("motolii_plugin::") {
            from = at + "motolii_plugin::".len();
            continue;
        }
        if let Some(own) = own_crate {
            if rest.starts_with(own) {
                from = at + own.len();
                continue;
            }
        }
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .unwrap_or(rest.len());
        violations.push(rest[..end].to_string());
        from = at + 1;
    }
    violations
}

/// Opacity crateが名指しした`motolii_plugin::`公開pathの閉集合(A1S §3)。
const ALLOWED_OPACITY_PUBLIC_PATHS: &[&str] = &[
    "bytemuck",
    "F64Domain",
    "FilterPlugin",
    "GpuCtx",
    "NodeDesc",
    "ParamDef",
    "PipelineCache",
    "PipelineCacheKey",
    "PluginContract",
    "PluginError",
    "PluginId",
    "PluginKind",
    "RenderCtx",
    "ResolvedParams",
    "TextureRef",
    "Value",
    "ValueType",
    "wgpu",
];

fn collect_motolii_plugin_paths(src: &str) -> BTreeSet<String> {
    let code = strip_line_comments(src);
    let mut paths = BTreeSet::new();
    let mut from = 0;
    while let Some(pos) = code[from..].find("motolii_plugin::") {
        let at = from + pos + "motolii_plugin::".len();
        let rest = &code[at..];
        if rest.starts_with('{') {
            if let Some(close) = rest.find('}') {
                for item in rest[1..close].split(',') {
                    let ident = item.trim().split(" as ").next().unwrap_or("").trim();
                    if !ident.is_empty() {
                        paths.insert(ident.to_string());
                    }
                }
            }
            from = at + 1;
            continue;
        }
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if end > 0 {
            paths.insert(rest[..end].to_string());
        }
        from = at;
    }
    paths
}

fn opacity_public_api_violations(src: &str) -> Vec<String> {
    collect_motolii_plugin_paths(src)
        .into_iter()
        .filter(|path| !ALLOWED_OPACITY_PUBLIC_PATHS.contains(&path.as_str()))
        .collect()
}

fn top_level_public_items(src: &str) -> BTreeSet<String> {
    strip_line_comments(src)
        .lines()
        .filter(|line| *line == line.trim_start())
        .filter_map(|line| {
            let declaration = line.strip_prefix("pub ")?;
            let (kind, rest) = declaration.split_once(' ')?;
            let name = rest
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .next()
                .unwrap_or("");
            (!name.is_empty()).then(|| format!("{kind} {name}"))
        })
        .collect()
}

#[test]
fn workspace_has_external_reference_plugin_crates() {
    let crates = external_plugin_crates();
    assert!(
        !crates.is_empty(),
        "plugins/motolii-plugin-* に外部参照crateが1件以上必要(A1S §2.2)"
    );
    for crate_dir in crates {
        assert!(
            crate_dir.join("Cargo.toml").is_file(),
            "{} にCargo.tomlが必要",
            crate_dir.display()
        );
    }
}

#[test]
fn external_plugin_crates_have_allowlisted_dependencies() {
    for manifest in external_plugin_manifests() {
        assert!(manifest.is_file(), "{} is missing", manifest.display());
        let text = fs::read_to_string(&manifest).unwrap();
        let has_build_rs = manifest.parent().unwrap().join("build.rs").exists();
        let violations = external_manifest_violations(&text, has_build_rs);
        assert!(
            violations.is_empty(),
            "{}: {:?}",
            manifest.display(),
            violations
        );
    }
}

#[test]
fn external_plugin_sources_use_only_motolii_plugin_paths() {
    for crate_dir in external_plugin_crates() {
        let manifest = fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        let own_crate = manifest
            .lines()
            .find(|line| line.starts_with("name = "))
            .and_then(|line| line.split('"').nth(1))
            .map(|name| format!("{}::", name.replace('-', "_")))
            .unwrap();
        let mut source_files = Vec::new();
        collect_rust_files(&crate_dir.join("src"), &mut source_files);
        for file in source_files {
            let text = fs::read_to_string(&file).unwrap();
            let violations = motolii_path_violations(&text, None);
            assert!(
                violations.is_empty(),
                "{} has disallowed motolii_* paths: {:?}",
                file.display(),
                violations
            );
        }
        let mut test_files = Vec::new();
        collect_rust_files(&crate_dir.join("tests"), &mut test_files);
        for file in test_files {
            let text = fs::read_to_string(&file).unwrap();
            let violations = motolii_path_violations(&text, Some(&own_crate));
            assert!(
                violations.is_empty(),
                "{} has disallowed motolii_* paths: {:?}",
                file.display(),
                violations
            );
        }
    }
}

#[test]
fn external_plugin_public_api_has_no_panic_paths() {
    for crate_dir in external_plugin_crates() {
        let mut files = Vec::new();
        collect_rust_files(&crate_dir.join("src"), &mut files);
        for file in files {
            let text = fs::read_to_string(&file).unwrap();
            let violations = panic_violations(&text);
            assert!(
                violations.is_empty(),
                "{} の非テストコードにpanic経路: {:?}",
                file.display(),
                violations
            );
        }
    }
}

#[test]
fn opacity_public_api_usage_is_closed_set() {
    let opacity_src = workspace_root().join("plugins/motolii-plugin-opacity/src/lib.rs");
    let text = fs::read_to_string(opacity_src).unwrap();
    let violations = opacity_public_api_violations(&text);
    assert!(
        violations.is_empty(),
        "unknown motolii_plugin:: paths in opacity crate: {violations:?}"
    );
    let used = collect_motolii_plugin_paths(&text);
    for required in ALLOWED_OPACITY_PUBLIC_PATHS {
        assert!(
            used.contains(*required),
            "expected opacity crate to use motolii_plugin::{required}"
        );
    }
}

#[test]
fn opacity_public_surface_is_exactly_three_items() {
    let opacity_src = workspace_root().join("plugins/motolii-plugin-opacity/src/lib.rs");
    let text = fs::read_to_string(opacity_src).unwrap();
    let expected = [
        "struct OpacityFilter",
        "static OPACITY_FILTER",
        "fn opacity_contract",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(top_level_public_items(&text), expected);
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
    for crate_dir in external_plugin_crates() {
        collect_rust_files(&crate_dir.join("src"), &mut files);
        collect_rust_files(&crate_dir.join("tests"), &mut files);
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
fn external_dependency_scanner_flags_non_plugin_dependency() {
    let manifest = "[dependencies]\nserde = \"1\"\n";
    let violations = external_dependency_violations(manifest);
    assert!(violations.iter().any(|v| v.contains("serde")));
}

#[test]
fn external_dependency_scanner_flags_renamed_package() {
    let manifest =
        "[dependencies]\nop = { package = \"motolii-gpu\", path = \"../crates/motolii-gpu\" }\n";
    let violations = external_dependency_violations(manifest);
    assert!(violations.iter().any(|v| v.contains("motolii-gpu")));
}

#[test]
fn external_dependency_scanner_flags_nonempty_dev_dependencies() {
    let manifest = "[dev-dependencies]\nserde = \"1\"\n";
    let violations = external_dependency_violations(manifest);
    assert!(violations.iter().any(|v| v.contains("dev-dependencies")));
}

#[test]
fn external_dependency_scanner_flags_nonempty_build_dependencies() {
    let manifest = "[build-dependencies]\nserde = \"1\"\n";
    let violations = external_dependency_violations(manifest);
    assert!(violations.iter().any(|v| v.contains("build-dependencies")));
}

#[test]
fn external_dependency_scanner_flags_target_nested_dependency() {
    let manifest = "[target.'cfg(unix)'.dependencies.serde]\nversion = \"1\"\n";
    let violations = external_dependency_violations(manifest);
    assert!(violations.iter().any(|v| v.contains("serde")));
}

#[test]
fn external_manifest_scanner_flags_proc_macro_setting() {
    let manifest = "[lib]\nproc-macro = true\n";
    let violations = external_manifest_violations(manifest, false);
    assert!(violations.iter().any(|v| v.contains("proc-macro")));
}

#[test]
fn external_manifest_scanner_flags_build_script() {
    let violations = external_manifest_violations("[package]\nname = \"bad\"\n", true);
    assert!(violations.iter().any(|v| v.contains("build.rs")));
}

#[test]
fn top_level_public_item_scanner_flags_extra_builder() {
    let src = "pub struct OpacityFilter;\npub fn opacity_contract() {}\npub static OPACITY_FILTER: u8 = 0;\npub fn builder() {}\n";
    assert!(top_level_public_items(src).contains("fn builder"));
}

#[test]
fn motolii_path_scanner_flags_internal_crate_alias() {
    let src = "use motolii_gpu::PipelineCacheKey;\n";
    let violations = motolii_path_violations(src, None);
    assert!(
        violations.iter().any(|v| v.starts_with("motolii_gpu::")),
        "{violations:?}"
    );
}

#[test]
fn opacity_public_api_scanner_flags_unknown_path() {
    let src = "fn f(_: motolii_plugin::PluginRegistry) {}\n";
    assert_eq!(
        opacity_public_api_violations(src),
        vec!["PluginRegistry".to_string()]
    );
}
