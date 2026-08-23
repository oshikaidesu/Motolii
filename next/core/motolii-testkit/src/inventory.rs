//! `next/reference/generated/inventory.tsv` の生成ロジック(裁定 = 総監督
//! 2026-08-23 指示「機械抽出して、関数や意味が出せるリストが欲しい」)。
//!
//! ## 何をする物か
//!
//! `next/` 配下の Rust ソースを [`syn`] で構文解析し、**Motolii が今持っている
//! 公開シンボルの在庫**を1行1シンボルの TSV へ書き出す。手で書き写す台帳
//! (`KNOWN.md`・`axis/*.tsv`)と違い、**ソースが変わればこの生成物も変わる**
//! ——生成器を再実行して差分が出るかどうかを機械で見張れる(`tests/
//! inventory_fence.rs`)。
//!
//! 集める `kind`:
//! - `Intent` — `motolii-store` の `enum Intent` の各枝
//! - `Message` — ソース中に `enum Message` という名前で定義された各 enum の枝
//!   (pane ごとに同名の enum が複数在る。`crate`/`path:line` 列で区別する)
//! - `PropId` — `motolii-store::property` モジュール直下の `pub const NAME: &str = "値"`
//! - `fn` — トップレベルの `pub fn` と、trait 実装ではない `impl Type { pub fn .. }`
//!   の中の `pub fn`(`Type::method` の形で記録)
//! - `struct` / `enum` — 上記以外のトップレベル `pub struct`/`pub enum`
//!
//! ## 精度の限界(誤検出を作らないための正直な申告、裁定209)
//!
//! - **`callers` は「識別子のテキスト全文一致(単語境界)を数えて定義自身の1件を
//!   引いた数」**であって、型を解決した本物の呼び出しグラフではない。
//!   同名の別シンボル(例: 複数の型に `new` という pub fn がある)を数え違える
//!   ことが構造的にあり得る——**「0件=未参照の可能性が高い」の方向にだけ
//!   安全**(同名衝突は実際より多く数える方向にしか倒れない。「0 なのに実は
//!   使われている」という偽陽性は起こらない。逆に「N>0 だが実は別シンボルの
//!   ヒット」で偽陰性(未配線を見逃す)は起こり得る——**これは在庫表の限界と
//!   して正直に書く**
//! - **`impl Trait for Type` の中の関数は数えない**(trait 実装は「呼び手が
//!   trait 経由の間接呼び出し」で callers=0 になりがちで、既知の偽陽性源
//!   — A10 の hover/カーソルで実測済みの穴と同じ理由。トレイト実装を出すと
//!   同じ誤検出を作るので、トレイト実装は最初からスコープ外にする)
//! - **関数本体の中で定義されるローカル `fn`/`struct`/`enum` は数えない**
//!   (トップレベルと `impl`/`mod` の中だけを見る — ノイズ源)
//! - **`tests/`/`examples`/`benches` 配下のファイルは定義の収集源にしない**
//!   (呼び手カウントの corpus には含める — テストから呼ばれているかは
//!   「配線されているか」の一部の証拠として有効)
//! - **`vis` は `pub` のみを対象**(`pub(crate)` 等は対象外 — 在庫表は
//!   「外へ見えている物」に絞る)

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, ImplItem, Item, Lit, Meta, Type, Visibility};

/// この crate の `src/` から見て `next/` の根。
/// (`owns_justification_fence.rs::next_root()` と同じ深さの相対関係。)
pub fn next_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn generated_inventory_path() -> PathBuf {
    next_root().join("reference/generated/inventory.tsv")
}

#[derive(Debug, Clone)]
struct Row {
    kind: &'static str,
    /// 表示用シンボル名(PropId だけは文字列値、他は識別子)。
    symbol: String,
    /// callers を数える時に検索する語(PropId は const の識別子名、他は symbol と同じ)。
    search_key: String,
    crate_name: String,
    rel_path: String,
    line: usize,
    doc: String,
}

/// `next/` 配下の全 `*.rs` を再帰収集する(corpus 用 — 定義収集はこの後 filter する)。
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name.starts_with('.') {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// 定義の収集源として使ってよいファイルか(`tests`/`examples`/`benches` は除く)。
fn is_definition_source(rel: &str) -> bool {
    let comps: Vec<&str> = rel.split('/').collect();
    !comps
        .iter()
        .any(|c| *c == "tests" || *c == "examples" || *c == "benches")
}

/// `core/motolii-testkit/src/inventory.rs` から見た `next/` からの相対パス
/// (`/` 区切りに正規化)。
fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// `next/<category>/<crate-name>/...` の2番目の要素を crate 名として使う。
fn crate_name_from_rel(rel: &str) -> String {
    rel.split('/').nth(1).unwrap_or("?").to_string()
}

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn first_doc_line(attrs: &[Attribute]) -> String {
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &attr.meta {
            if let Expr::Lit(el) = &nv.value {
                if let Lit::Str(s) = &el.lit {
                    let line = s.value();
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

fn type_head_name(ty: &Type) -> String {
    let tokens = ty.to_token_stream().to_string();
    tokens
        .split(['<', '(', ' ', '&'])
        .find(|s| !s.is_empty())
        .unwrap_or(&tokens)
        .to_string()
}

fn str_literal_value(expr: &Expr) -> Option<String> {
    if let Expr::Lit(el) = expr {
        if let Lit::Str(s) = &el.lit {
            return Some(s.value());
        }
    }
    None
}

fn is_str_type(ty: &Type) -> bool {
    ty.to_token_stream().to_string().contains("str")
}

fn walk_items(items: &[Item], rel: &str, crate_name: &str, rows: &mut Vec<Row>) {
    for item in items {
        match item {
            Item::Fn(f) => {
                if is_pub(&f.vis) {
                    rows.push(Row {
                        kind: "fn",
                        symbol: f.sig.ident.to_string(),
                        search_key: f.sig.ident.to_string(),
                        crate_name: crate_name.to_string(),
                        rel_path: rel.to_string(),
                        line: f.span().start().line,
                        doc: first_doc_line(&f.attrs),
                    });
                }
            }
            Item::Struct(s) => {
                if is_pub(&s.vis) {
                    rows.push(Row {
                        kind: "struct",
                        symbol: s.ident.to_string(),
                        search_key: s.ident.to_string(),
                        crate_name: crate_name.to_string(),
                        rel_path: rel.to_string(),
                        line: s.span().start().line,
                        doc: first_doc_line(&s.attrs),
                    });
                }
            }
            Item::Enum(e) => {
                let name = e.ident.to_string();
                if name == "Intent" {
                    for v in &e.variants {
                        rows.push(Row {
                            kind: "Intent",
                            symbol: v.ident.to_string(),
                            search_key: v.ident.to_string(),
                            crate_name: crate_name.to_string(),
                            rel_path: rel.to_string(),
                            line: v.span().start().line,
                            doc: first_doc_line(&v.attrs),
                        });
                    }
                } else if name == "Message" {
                    for v in &e.variants {
                        rows.push(Row {
                            kind: "Message",
                            symbol: v.ident.to_string(),
                            search_key: v.ident.to_string(),
                            crate_name: crate_name.to_string(),
                            rel_path: rel.to_string(),
                            line: v.span().start().line,
                            doc: first_doc_line(&v.attrs),
                        });
                    }
                } else if is_pub(&e.vis) {
                    rows.push(Row {
                        kind: "enum",
                        symbol: name,
                        search_key: e.ident.to_string(),
                        crate_name: crate_name.to_string(),
                        rel_path: rel.to_string(),
                        line: e.span().start().line,
                        doc: first_doc_line(&e.attrs),
                    });
                }
            }
            Item::Impl(imp) => {
                // trait 実装(`impl Trait for Type`)は対象外(doc 冒頭の理由)。
                if imp.trait_.is_none() {
                    let type_name = type_head_name(&imp.self_ty);
                    for it in &imp.items {
                        if let ImplItem::Fn(m) = it {
                            if is_pub(&m.vis) {
                                rows.push(Row {
                                    kind: "fn",
                                    symbol: format!("{type_name}::{}", m.sig.ident),
                                    search_key: m.sig.ident.to_string(),
                                    crate_name: crate_name.to_string(),
                                    rel_path: rel.to_string(),
                                    line: m.span().start().line,
                                    doc: first_doc_line(&m.attrs),
                                });
                            }
                        }
                    }
                }
            }
            Item::Mod(m) => {
                if let Some((_, content)) = &m.content {
                    if m.ident == "property" {
                        for it in content {
                            if let Item::Const(c) = it {
                                if is_pub(&c.vis) && is_str_type(&c.ty) {
                                    if let Some(val) = str_literal_value(&c.expr) {
                                        rows.push(Row {
                                            kind: "PropId",
                                            symbol: val,
                                            search_key: c.ident.to_string(),
                                            crate_name: crate_name.to_string(),
                                            rel_path: rel.to_string(),
                                            line: c.span().start().line,
                                            doc: first_doc_line(&c.attrs),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // 他の pub fn/struct/enum が mod の中に在ってもよいので、
                    // 種別を問わず引き続き再帰する。
                    walk_items(content, rel, crate_name, rows);
                }
            }
            _ => {}
        }
    }
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `haystack` 中の `word` を**単語境界付きで**数える(部分文字列の誤爆を防ぐ
/// ——例えば `zoom_step` を数える時に `zoom_step_out` を1件として誤爆しない)。
fn count_whole_word(haystack: &str, word: &str) -> usize {
    if word.is_empty() {
        return 0;
    }
    let bytes = haystack.as_bytes();
    let wlen = word.len();
    let mut count = 0;
    let mut start = 0;
    while start <= haystack.len() {
        let Some(pos) = haystack[start..].find(word) else {
            break;
        };
        let idx = start + pos;
        let before_ok = idx == 0 || !is_word_byte(bytes[idx - 1]);
        let after_idx = idx + wlen;
        let after_ok = after_idx >= bytes.len() || !is_word_byte(bytes[after_idx]);
        if before_ok && after_ok {
            count += 1;
        }
        start = idx + 1;
    }
    count
}

fn tsv_escape(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ")
}

/// 在庫表(TSV テキスト)を生成する。ファイルへは書かない
/// (生成器 bin と柵テストの両方から同じロジックを呼ぶための分離)。
pub fn generate() -> String {
    let root = next_root();
    let mut all_files = Vec::new();
    collect_rs_files(&root, &mut all_files);

    // corpus: callers を数えるための全文(定義源フィルタなし — テストからの
    // 参照も「配線されているか」の証拠として数える)。
    let corpus: Vec<(String, String)> = all_files
        .iter()
        .filter_map(|p| {
            let rel = rel_path(&root, p);
            fs::read_to_string(p).ok().map(|text| (rel, text))
        })
        .collect();

    let mut rows: Vec<Row> = Vec::new();
    for (rel, text) in &corpus {
        if !is_definition_source(rel) {
            continue;
        }
        let file = match syn::parse_file(text) {
            Ok(f) => f,
            Err(err) => {
                // 構文解析に失敗するファイルは黙って飛ばす(生成器の壊れやすさより
                // 一部欠落の方が安全)——ただし失敗した事実は stderr へ残す
                // (無言で欠落すると「網羅した気になる」偽陽性の温床になる、裁定209)。
                eprintln!("gen_inventory: parse failed, skipped: {rel}: {err}");
                continue;
            }
        };
        let crate_name = crate_name_from_rel(rel);
        walk_items(&file.items, rel, &crate_name, &mut rows);
    }

    // 決定的な順序(生成のたびに順序が揺れると diff 柵が誤爆する)。
    rows.sort_by(|a, b| {
        (a.kind, &a.crate_name, &a.rel_path, a.line, &a.symbol).cmp(&(
            b.kind,
            &b.crate_name,
            &b.rel_path,
            b.line,
            &b.symbol,
        ))
    });

    let mut out = String::new();
    out.push_str(
        "kind\tsymbol\tcrate\tpath:line\tvis\tcallers\tdoc1行\t到達経路\n",
    );

    for row in &rows {
        let mut total = 0usize;
        let mut eval_total = 0usize;
        for (crel, ctext) in &corpus {
            let n = count_whole_word(ctext, &row.search_key);
            total += n;
            if crel.starts_with("core/motolii-eval/") {
                eval_total += n;
            }
        }
        // 定義自身の1件を引く(定義行にも識別子が現れるため)。
        let callers = total.saturating_sub(1);

        let reachability = match row.kind {
            "Intent" => {
                if callers == 0 {
                    "入口ゼロ"
                } else {
                    ""
                }
            }
            "Message" | "fn" | "struct" | "enum" => {
                if callers == 0 {
                    "呼び手ゼロ"
                } else {
                    ""
                }
            }
            "PropId" => {
                if eval_total > 0 {
                    "評価あり"
                } else {
                    "評価なし"
                }
            }
            _ => "",
        };

        out.push_str(&format!(
            "{}\t{}\t{}\t{}:{}\tpub\t{}\t{}\t{}\n",
            row.kind,
            tsv_escape(&row.symbol),
            row.crate_name,
            row.rel_path,
            row.line,
            callers,
            tsv_escape(&row.doc),
            reachability,
        ));
    }

    out
}

/// 未配線候補(`callers == 0` の行)だけを抜き出す——`generate()` の出力
/// (ヘッダ込み)を渡す。呼び手側での二次生成に使う小さな補助。
pub fn unwired_symbols(tsv: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (i, line) in tsv.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 8 {
            let kind = cols[0];
            let path_line = cols[3];
            let reachability = cols[7];
            if (kind == "fn" || kind == "Intent" || kind == "Message")
                && (reachability == "呼び手ゼロ" || reachability == "入口ゼロ")
            {
                out.insert(format!("{kind}\t{path_line}"));
            }
        }
    }
    out
}
