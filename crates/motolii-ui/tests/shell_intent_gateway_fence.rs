//! 構造フェンス: shell 層の副作用は**単一ゲートウェイ**の外から起こせない。
//!
//! [2026-08-18裁定](../../../docs/reviews/2026-08-18-log-and-structure-enforcement.md)
//! の「構造の強制」。iced では `update()` を通らない状態変化が**書けない**が、egui では
//! 何処からでも writer を呼べてしまう。そこで「journal を通らずに同じ副作用へ着く道」を
//! 走査で塞ぐ — 破ると落ちるテスト、という本リポジトリの実証済みの型
//! (`shell_error_fence.rs` / `diagnostic.rs` と同型)。
//!
//! ## 禁止リストの根拠(実測)
//!
//! 2026-08-18 に `blitz_shell/` を `rg` して出た、**製品状態を進める呼び出しの全部**:
//!
//! ```text
//! app.rs:123  DocumentWriter::new(     ← 座席の writer 生成
//! app.rs:155  .save_document(          ← Cmd+S の書き戻し
//! app.rs:252  .save_document(          ← New の初期保存
//! app.rs:303  .import_dropped_media(   ← 取り込みの実体
//! app.rs:475  create_project_file(     ← New
//! app.rs:508  reseat_project(          ← Open / New の座り直し
//! app.rs:604  admit_dropped_paths(     ← OS ドロップ
//! app.rs:626  ExportRun::start(        ← 書き出し開始
//! app.rs:755  run.cancel()             ← 書き出しキャンセル
//! app.rs:845  admit_dropped_paths(     ← Browser のダブルクリック
//! ```
//!
//! これらは `intent.rs`(ゲートウェイ)の中だけに在ってよい。外に1つでも残ると、
//! その操作は `--intent-log` に載らず replay で再現できない = ログの強制が破れる。
//!
//! ## いま塞いでいないもの(正直に名指しする)
//!
//! - `ShellGateway::project_mut()` 経由の Timeline エディタ編集と Undo/Redo。
//!   あれは `motolii-doc` 側の D2 journal が受けている。shell 層の intent 化は wave E
//! - view(camera・選択・panel)の操作。本レーンのスコープ外
//!
//! ## 走査の規約
//!
//! 見るのは**製品コードだけ**: `#[cfg(test)]` から先(テストが temp project を組み立てる
//! ための `save_document` 等が居る)と、`//` で始まる行(この禁止リストを説明する
//! doc コメント自身に引っかからないため)は落とす。

/// ゲートウェイ本体。ここだけが禁止 API を呼んでよい。
const GATEWAY: &str = "intent.rs";

/// 走査する blitz_shell の製品ソース(ゲートウェイ以外の全部)。
const SCANNED: &[(&str, &str)] = &[
    ("app.rs", include_str!("../src/blitz_shell/app.rs")),
    ("pane.rs", include_str!("../src/blitz_shell/pane.rs")),
    ("drive.rs", include_str!("../src/blitz_shell/drive.rs")),
    ("runner.rs", include_str!("../src/blitz_shell/runner.rs")),
    ("main.rs", include_str!("../src/blitz_shell/main.rs")),
    ("mod.rs", include_str!("../src/blitz_shell/mod.rs")),
];

/// 禁止する呼び出しと、代わりに通すべき intent。
const FORBIDDEN: &[(&str, &str)] = &[
    ("create_project_file(", "UiIntent::NewProject"),
    ("reseat_project(", "UiIntent::NewProject / OpenProject"),
    ("admit_dropped_paths(", "UiIntent::AdmitPaths"),
    ("import_dropped_media(", "UiIntent::AdmitPaths"),
    ("ExportRun::start(", "UiIntent::BeginExport"),
    (".cancel()", "UiIntent::CancelExport"),
    (
        "DocumentWriter::new(",
        "ShellGateway(座席の生成はゲートウェイの中)",
    ),
    ("save_document(", "UiIntent::SaveProject"),
];

/// 製品コードだけを残す。`#[cfg(test)]` から先と `//` 行は落とす。
fn product_source(source: &str) -> String {
    let body = match source.find("#[cfg(test)]") {
        Some(at) => &source[..at],
        None => source,
    };
    body.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn no_shell_code_outside_the_gateway_touches_product_state_directly() {
    let mut breaches: Vec<String> = Vec::new();
    for (name, source) in SCANNED {
        let product = product_source(source);
        for (pattern, replacement) in FORBIDDEN {
            let hits = product.matches(pattern).count();
            if hits > 0 {
                breaches.push(format!(
                    "  blitz_shell/{name}: `{pattern}` が {hits} 箇所 — {replacement} を dispatch する"
                ));
            }
        }
    }
    assert!(
        breaches.is_empty(),
        "ゲートウェイ({GATEWAY})の外から製品状態を直接動かしている:\n{}\n\
         journal を通らない副作用は --intent-log に載らず replay で再現できない。\
         `ShellGateway::dispatch(UiIntent::…)` へ寄せること",
        breaches.join("\n")
    );
}

/// 禁止リストが**空振りしていない**ことの担保。ゲートウェイ本体には
/// これらの呼び出しが実際に在る = パターンが現実の API 名と噛み合っている。
/// (綴りを間違えた禁止リストは、何も守らないのに緑になる)
#[test]
fn the_forbidden_list_matches_real_calls_inside_the_gateway() {
    let gateway = product_source(include_str!("../src/blitz_shell/intent.rs"));
    let missing: Vec<&str> = FORBIDDEN
        .iter()
        .map(|(pattern, _)| *pattern)
        .filter(|pattern| !gateway.contains(pattern))
        .collect();
    assert!(
        missing.is_empty(),
        "禁止リストの {missing:?} が {GATEWAY} の中に1つも無い — \
         API 名が変わった(綴り違いの禁止リストは何も守らない)。\
         現行の呼び出し名を rg で測り直して更新すること"
    );
}
