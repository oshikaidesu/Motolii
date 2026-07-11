//! M2E-4: `assert_rgba_close` の tolerance を deny する走査。
//!
//! 閾値を1上げる1文字diffは「テスト改変」に見えにくい(監査E-2)。
//! 許容は `motolii_testkit::tol` 定数経由のみ。許可定数と定義内転送以外は
//! すべて違反(識別子・`1_u8`・`0x1`・ローカル束縛の転送も不可)。

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-testkit -> workspace root")
        .to_path_buf()
}

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 呼び出し側で許す tolerance 引数(完全一致)。
const ALLOWED_TOL_CONSTS: &[&str] = &[
    "tol::EXACT",
    "tol::GPU_RASTER",
    "motolii_testkit::tol::EXACT",
    "motolii_testkit::tol::GPU_RASTER",
];

/// `assert_rgba_close` / `_with_artifacts` の tolerance 引数が許可外ならその式を返す。
fn disallowed_tolerance_hits(src: &str) -> Vec<(usize, String)> {
    let src = strip_line_comments(src);
    let forward_ranges = assert_rgba_close_forward_ranges(&src);
    let names = ["assert_rgba_close_with_artifacts", "assert_rgba_close"];
    let mut hits = Vec::new();
    // 日本語コメント等のマルチバイト文字で char 境界を踏まない。
    let mut indices: Vec<usize> = src.char_indices().map(|(i, _)| i).collect();
    indices.push(src.len());
    let mut pos = 0usize;
    while pos < indices.len() - 1 {
        let i = indices[pos];
        let mut found: Option<&str> = None;
        for name in names {
            if src[i..].starts_with(name) {
                let prev_ok = i == 0 || {
                    let prev = src[..i].chars().next_back().unwrap();
                    !prev.is_ascii_alphanumeric() && prev != '_'
                };
                let after = i + name.len();
                let next_ok = after >= src.len() || {
                    let next = src[after..].chars().next().unwrap();
                    !next.is_ascii_alphanumeric() && next != '_'
                };
                if prev_ok && next_ok {
                    found = Some(name);
                    break;
                }
            }
        }
        let Some(name) = found else {
            pos += 1;
            continue;
        };
        // `fn assert_rgba_close` シグネチャ行はスキップ(本体の転送は別判定)。
        let line_start = src[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let prefix = src[line_start..i].trim_start();
        if prefix.starts_with("fn ") || prefix.starts_with("pub fn ") {
            pos += name.len();
            continue;
        }

        let mut j = i + name.len();
        let bytes = src.as_bytes();
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'(' {
            pos += 1;
            continue;
        }
        let open = j;
        let mut depth = 0usize;
        let mut end = None;
        let mut k = open;
        while k < bytes.len() {
            match bytes[k] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(k);
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        let Some(close) = end else {
            pos += 1;
            continue;
        };
        let body = &src[open + 1..close];
        let args = split_top_level_args(body);
        let nonempty: Vec<&str> = args
            .iter()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .collect();
        let tol = if name.ends_with("with_artifacts") {
            nonempty.get(nonempty.len().wrapping_sub(2)).copied()
        } else {
            nonempty.last().copied()
        };
        let next_i = close + 1;
        pos = indices.partition_point(|&x| x < next_i);
        let Some(tol) = tol else {
            continue;
        };
        if ALLOWED_TOL_CONSTS.contains(&tol) {
            continue;
        }
        // 定義内転送: `assert_rgba_close` 本体から `_with_artifacts` へ `tolerance` を渡すだけ許可。
        if tol == "tolerance"
            && name == "assert_rgba_close_with_artifacts"
            && forward_ranges.iter().any(|&(a, b)| i >= a && i < b)
        {
            continue;
        }
        let line = src[..i].chars().filter(|c| *c == '\n').count() + 1;
        hits.push((line, tol.to_string()));
    }
    hits
}

/// `pub fn assert_rgba_close` / `fn assert_rgba_close` 本体のバイト範囲(転送許可用)。
/// `_with_artifacts` は含めない — そちらはパラメータ受け取り側で、外からの
/// `let tolerance = N` 転送の抜け道になるため。
fn assert_rgba_close_forward_ranges(src: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = src[search_from..].find("fn assert_rgba_close") {
        let at = search_from + rel;
        // `fn assert_rgba_close_with_artifacts` は除外
        let after_name = at + "fn assert_rgba_close".len();
        if src[after_name..].starts_with("_with_artifacts") {
            search_from = after_name;
            continue;
        }
        // 直前が識別子の続きならスキップ(念のため)
        if at > 0 {
            let prev = src[..at].chars().next_back().unwrap();
            if prev.is_ascii_alphanumeric() || prev == '_' {
                search_from = after_name;
                continue;
            }
        }
        // `{` まで進み、ブレース対応で本体終端を取る。
        let Some(brace_at) = src[after_name..].find('{') else {
            break;
        };
        let body_start = after_name + brace_at;
        let mut depth = 0i32;
        let mut body_end = None;
        for (off, ch) in src[body_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        body_end = Some(body_start + off + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = body_end else {
            break;
        };
        ranges.push((body_start, end));
        search_from = end;
    }
    ranges
}

fn split_top_level_args(body: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut brace = 0i32;
    let mut bracket = 0i32;
    let mut in_str: Option<char> = None;
    let mut esc = false;
    for ch in body.chars() {
        if let Some(q) = in_str {
            cur.push(ch);
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == q {
                in_str = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_str = Some(ch);
                cur.push(ch);
            }
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            '{' => {
                brace += 1;
                cur.push(ch);
            }
            '}' => {
                brace -= 1;
                cur.push(ch);
            }
            '[' => {
                bracket += 1;
                cur.push(ch);
            }
            ']' => {
                bracket -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 && brace == 0 && bracket == 0 => {
                args.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    args.push(cur);
    args
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
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

#[test]
fn fixture_literal_tolerance_is_detected() {
    let bad = r#"
        assert_rgba_close("x", desc, &a, &e, 0);
        assert_rgba_close(
            "y",
            desc,
            &a,
            &e,
            1,
        );
        assert_rgba_close_with_artifacts("z", desc, &a, &e, 2, None);
    "#;
    let hits = disallowed_tolerance_hits(bad);
    assert_eq!(hits.len(), 3, "hits={hits:?}");
    assert_eq!(hits[0].1, "0");
    assert_eq!(hits[1].1, "1");
    assert_eq!(hits[2].1, "2");
}

#[test]
fn fixture_bypass_forms_are_detected() {
    // 数字以外の迂回: 識別子・型付きリテラル・16進・ローカル束縛名の転送。
    let bad = r#"
        assert_rgba_close("a", desc, &a, &e, LOCAL_TOL);
        assert_rgba_close("b", desc, &a, &e, 1_u8);
        assert_rgba_close("c", desc, &a, &e, 0x1);
        assert_rgba_close("d", desc, &a, &e, tolerance);
        assert_rgba_close_with_artifacts("e", desc, &a, &e, bumped, None);
    "#;
    let hits = disallowed_tolerance_hits(bad);
    assert_eq!(hits.len(), 5, "hits={hits:?}");
    assert_eq!(hits[0].1, "LOCAL_TOL");
    assert_eq!(hits[1].1, "1_u8");
    assert_eq!(hits[2].1, "0x1");
    assert_eq!(hits[3].1, "tolerance");
    assert_eq!(hits[4].1, "bumped");
}

#[test]
fn fixture_definition_forwarding_tolerance_is_allowed() {
    // lib.rs と同じ形: assert_rgba_close 本体から _with_artifacts へ tolerance 転送。
    let good = r#"
        pub fn assert_rgba_close(
            label: &str,
            desc: RgbaImageDesc,
            actual: &[u8],
            expected: &[u8],
            tolerance: u8,
        ) {
            assert_rgba_close_with_artifacts(label, desc, actual, expected, tolerance, None);
        }
    "#;
    assert!(
        disallowed_tolerance_hits(good).is_empty(),
        "hits={:?}",
        disallowed_tolerance_hits(good)
    );
}

#[test]
fn fixture_tol_constants_are_allowed() {
    let good = r#"
        assert_rgba_close("x", desc, &a, &e, tol::EXACT);
        assert_rgba_close("y", desc, &a, &e, tol::GPU_RASTER);
        assert_rgba_close_with_artifacts("z", desc, &a, &e, motolii_testkit::tol::EXACT, None);
    "#;
    assert!(disallowed_tolerance_hits(good).is_empty());
}

#[test]
fn no_disallowed_tolerance_in_workspace_sources() {
    let root = workspace_root();
    let mut violations = Vec::new();
    for path in rust_sources(&root) {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        // 本ファイルの負例フィクスチャは走査対象外(判定は fixture_* テストが担保)。
        if rel.ends_with("tests/tol_literals.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line, expr) in disallowed_tolerance_hits(&text) {
            violations.push(format!("{rel}:{line}: disallowed tolerance {expr}"));
        }
    }
    assert!(
        violations.is_empty(),
        "assert_rgba_close tolerance must use tol::EXACT / tol::GPU_RASTER:\n{}",
        violations.join("\n")
    );
}
