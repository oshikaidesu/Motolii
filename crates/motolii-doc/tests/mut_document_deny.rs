//! M2E-5: motolii-doc 外の `&mut Document` を deny する走査。
//!
//! 単一writer(F-2)は型だけではすり抜け可能(Document が pub)。
//! コメント・文字列内は誤検出しない。`&mut` と `Document` の改行分割は見逃さない。

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/motolii-doc -> workspace root")
        .to_path_buf()
}

/// コメント・文字/文字列リテラルを空白に潰す(改行は残して行番号を保つ)。
fn mask_non_code(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        // 行コメント
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            out.push(b' ');
            out.push(b' ');
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(b' ');
                i += 1;
            }
            continue;
        }
        // ブロックコメント(入れ子対応)
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            out.push(b' ');
            out.push(b' ');
            i += 2;
            let mut depth = 1i32;
            while i < bytes.len() && depth > 0 {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    out.push(b' ');
                    out.push(b' ');
                    i += 2;
                    depth += 1;
                } else if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    out.push(b' ');
                    out.push(b' ');
                    i += 2;
                    depth -= 1;
                } else {
                    out.push(if bytes[i] == b'\n' { b'\n' } else { b' ' });
                    i += 1;
                }
            }
            continue;
        }
        // raw string: r#"..."# / r##"..."## / br"..." 等
        if let Some((prefix_len, hashes)) = raw_string_prefix(bytes, i) {
            for _ in 0..prefix_len {
                out.push(b' ');
            }
            i += prefix_len;
            let closer = format!("\"{}", "#".repeat(hashes));
            let closer_bytes = closer.as_bytes();
            while i < bytes.len() {
                if bytes[i..].starts_with(closer_bytes) {
                    for _ in 0..closer_bytes.len() {
                        out.push(b' ');
                    }
                    i += closer_bytes.len();
                    break;
                }
                out.push(if bytes[i] == b'\n' { b'\n' } else { b' ' });
                i += 1;
            }
            continue;
        }
        // 通常文字列 / バイト文字列
        if bytes[i] == b'"' || (bytes[i] == b'b' && i + 1 < bytes.len() && bytes[i + 1] == b'"') {
            if bytes[i] == b'b' {
                out.push(b' ');
                i += 1;
            }
            out.push(b' ');
            i += 1;
            while i < bytes.len() {
                let b = bytes[i];
                if b == b'\\' && i + 1 < bytes.len() {
                    out.push(b' ');
                    out.push(b' ');
                    i += 2;
                    continue;
                }
                out.push(if b == b'\n' { b'\n' } else { b' ' });
                i += 1;
                if b == b'"' {
                    break;
                }
            }
            continue;
        }
        // 文字リテラルはマスクしない — `'a` 寿命と区別が必要で、`&'a mut Document` 検出を壊す。
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).expect("mask preserves UTF-8 structure via ASCII ops")
}

/// `r#"...` / `br##"...` の開始。戻り値は (プレフィックス長, `#` 個数)。
fn raw_string_prefix(bytes: &[u8], i: usize) -> Option<(usize, usize)> {
    let mut j = i;
    if j < bytes.len() && bytes[j] == b'b' {
        j += 1;
    }
    if j >= bytes.len() || bytes[j] != b'r' {
        return None;
    }
    j += 1;
    let hash_start = j;
    while j < bytes.len() && bytes[j] == b'#' {
        j += 1;
    }
    if j >= bytes.len() || bytes[j] != b'"' {
        return None;
    }
    let hashes = j - hash_start;
    let prefix_len = j + 1 - i;
    Some((prefix_len, hashes))
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// `&` の直後のオプション寿命(`'a` / `'_`)を読み飛ばす。
fn skip_optional_lifetime(bytes: &[u8], mut i: usize) -> usize {
    i = skip_ws(bytes, i);
    if i >= bytes.len() || bytes[i] != b'\'' {
        return i;
    }
    i += 1;
    if i < bytes.len() && (is_ident_start(bytes[i]) || bytes[i] == b'_') {
        i += 1;
        while i < bytes.len() && is_ident_continue(bytes[i]) {
            i += 1;
        }
    }
    skip_ws(bytes, i)
}

/// `&mut Document` / `&'a mut path::Document`(空白・改行可)のヒット行(1-origin)を返す。
fn mut_document_hit_lines(src: &str) -> Vec<usize> {
    let masked = mask_non_code(src);
    let bytes = masked.as_bytes();
    let mut hits = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            i += 1;
            continue;
        }
        let amp_at = i;
        i += 1;
        i = skip_optional_lifetime(bytes, i);
        if !bytes[i..].starts_with(b"mut") {
            continue;
        }
        let after_mut = i + 3;
        if after_mut < bytes.len() && is_ident_continue(bytes[after_mut]) {
            continue;
        }
        i = skip_ws(bytes, after_mut);
        // 任意の `ident::` パス接頭辞
        loop {
            if i >= bytes.len() || !is_ident_start(bytes[i]) {
                break;
            }
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let name = &bytes[start..i];
            i = skip_ws(bytes, i);
            if i + 1 < bytes.len() && bytes[i] == b':' && bytes[i + 1] == b':' {
                i = skip_ws(bytes, i + 2);
                continue;
            }
            // 最終識別子が Document で、識別子境界
            if name == b"Document" && (i >= bytes.len() || !is_ident_continue(bytes[i])) {
                let line = masked[..amp_at].bytes().filter(|&b| b == b'\n').count() + 1;
                hits.push(line);
            }
            break;
        }
    }
    hits
}

fn is_under_motolii_doc(rel: &str) -> bool {
    rel == "crates/motolii-doc" || rel.starts_with("crates/motolii-doc/")
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
fn fixture_same_line_mut_document_is_detected() {
    let bad = "fn f(doc: &mut Document) {}\n";
    assert_eq!(mut_document_hit_lines(bad), vec![1]);
}

#[test]
fn fixture_linebreak_mut_document_is_detected() {
    // 改行分割の見逃し防止
    let bad = "fn f(doc: &mut\nDocument) {}\n";
    assert_eq!(mut_document_hit_lines(bad), vec![1]);

    let spaced = "fn f(doc: &\nmut\n  Document) {}\n";
    assert_eq!(mut_document_hit_lines(spaced), vec![1]);
}

#[test]
fn fixture_lifetime_mut_document_is_detected() {
    let bad = "fn f<'a>(doc: &'a mut Document) {}\n";
    assert_eq!(mut_document_hit_lines(bad), vec![1]);
}

#[test]
fn fixture_path_qualified_mut_document_is_detected() {
    let bad = "fn f(doc: &mut motolii_doc::Document) {}\nfn g(x: &mut crate::foo::Document) {}\n";
    assert_eq!(mut_document_hit_lines(bad), vec![1, 2]);
}

#[test]
fn fixture_comments_are_not_detected() {
    let src = r#"
// &mut Document はコメント
/// doc: &mut Document
//! crate: &mut Document
fn ok() {
    /* &mut Document */
    let _ = 1;
    /* nested /* &mut Document */ still comment */
}
"#;
    assert!(
        mut_document_hit_lines(src).is_empty(),
        "hits={:?}",
        mut_document_hit_lines(src)
    );
}

#[test]
fn fixture_strings_are_not_detected() {
    // 外側は ### — 内側の r#"..."# / r##"..."## と衝突させない
    let src = r###"
fn ok() {
    let _ = "&mut Document";
    let _ = b"&mut Document";
    let _ = r#"&mut Document"#;
    let _ = r##"&mut
Document"##;
}
"###;
    assert!(
        mut_document_hit_lines(src).is_empty(),
        "hits={:?}",
        mut_document_hit_lines(src)
    );
}

#[test]
fn fixture_identifier_boundary_avoids_false_positive() {
    // Document の接頭/接尾で誤検出しない
    let src = "fn f(x: &mut DocumentWriter) {}\nfn g(y: &mut Documentation) {}\n";
    assert!(
        mut_document_hit_lines(src).is_empty(),
        "hits={:?}",
        mut_document_hit_lines(src)
    );
}

#[test]
fn no_mut_document_outside_motolii_doc() {
    let root = workspace_root();
    let mut sources = rust_sources(&root);
    sources.sort();
    assert!(
        sources.len() > 10,
        "ソース列挙に失敗している(件数={})",
        sources.len()
    );
    // 番兵: motolii-doc 自身に本物の &mut Document があること(走査が空振りしていない)
    let doc_lib = root.join("crates/motolii-doc/src/lib.rs");
    let doc_src = std::fs::read_to_string(&doc_lib).expect("read motolii-doc lib");
    assert!(
        !mut_document_hit_lines(&doc_src).is_empty(),
        "番兵: motolii-doc 内の &mut Document を検出できない — スキャナ退行"
    );

    let mut violations = Vec::new();
    let mut scanned_outside = 0usize;
    for path in &sources {
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if is_under_motolii_doc(&rel) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        scanned_outside += 1;
        for line in mut_document_hit_lines(&text) {
            violations.push(format!("{rel}:{line}: &mut Document outside motolii-doc"));
        }
    }
    assert!(
        scanned_outside > 5,
        "motolii-doc 外の走査が空振り(件数={scanned_outside})"
    );
    assert!(
        violations.is_empty(),
        "単一writer: &mut Document は motolii-doc 内のみ(DocumentWriter::edit):\n{}",
        violations.join("\n")
    );
}
