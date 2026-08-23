//! 「入口が在るか」の台帳(`axis/A01-entry.tsv`)が実コードと食い違っていないか。
//!
//! 2026-08-23: `plan_waves.py` のグラフが手書きの列に依存していると MC-1 が
//! 指摘した(コードを直しても台帳が古いままだとグラフが動かない)。判定は
//! `scripts/derive_entries.py` が実コードから導けるので、**手書きと導出の
//! 食い違いをここで落とす**。
//!
//! 落ちたら: `python3 scripts/derive_entries.py <リポ根>` を回し、
//! 出力の「食い違い」節に従って **台帳側**を直す(実コードが正)。

use std::process::Command;

#[test]
fn ledger_entry_verdicts_match_the_code() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/derive_entries.py"))
        .arg(&root)
        .output()
        .expect("derive_entries.py を起動できない");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        !text.contains("食い違い(台帳が古い可能性)"),
        "`axis/A01-entry.tsv` の判定が実コードと食い違っている。\
         実コードが正なので台帳を直すこと:\n{text}"
    );
}
