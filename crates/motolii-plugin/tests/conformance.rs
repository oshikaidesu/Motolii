//! plugin-authoring §7 チェックリストの機械化(INF-7a/7b)。
//!
//! - ベンダー/OS固有GPU APIの依存・ソース参照をワークスペース全体で拒否する(F-9、§3-1)。
//! - `motolii-plugin` 公開面(非テストコード)のpanic経路を拒否する(§3-7。clippy lintの補完で
//!   `assert!` 系もここで落とす)。
//!
//! 「違反負例が赤になる」証明はフィクスチャ文字列に対する単体テストで行う(ツリーに違反を
//! 置くとCI自体が赤になるため)。実ツリーへの適用テストは違反ゼロを主張する。

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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
fn direct_dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_deps = false;
    for raw in manifest.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            let section = line.trim_matches(|c| c == '[' || c == ']');
            in_deps = section.ends_with("dependencies");
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            names.push(key.trim().trim_matches('"').to_string());
            if let Some(rest) = value.split("package").nth(1) {
                if let Some(pkg) = rest.split('"').nth(1) {
                    names.push(pkg.to_string());
                }
            }
        }
    }
    names
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
