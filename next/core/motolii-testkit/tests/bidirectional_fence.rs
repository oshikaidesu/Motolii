//! 台帳と実装を**双方向**で突き合わせる(Lottie の条件2の再演)。
//!
//! 裁定212 が Lottie の4条件の (2) を「閉じているので網羅が機械判定」と書いた —
//! `lottie_coverage.rs` は**双方向**(スキーマに在って表に無い / 表に在ってスキーマに無い)
//! で落とす。`normal-map` は片方向(`採用済` → 実在識別子、裁定229)しか見ていなかった。
//!
//! 逆向き = **実装に在るのに台帳が要求していない物**。**先回りの検出**そのもので、
//! 「段階が要求するまで作らない」(裁定226)の機械的な裏取りになる。
//! 鍵は `motolii-verbs` の `Verb::map_ids`。
//!
//! 導入時(2026-08-23)の実測: 動詞39 / `map_ids` 空19 / 実装済みなのに `採用予定` 9。
//! 後者9件はその場で解消した(verdict が実装に追いついていなかっただけ)。
//! 前者19件は**判定が要る**(定義の漏れ か 先回り)ので上限を固定して増やさない形にする。
//!
//! 落ちたら: `python3 scripts/check_bidirectional.py <リポ根>`。

use std::process::Command;

#[test]
fn ledger_and_implementation_agree_in_both_directions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/check_bidirectional.py"))
        .arg(&root)
        .output()
        .expect("check_bidirectional.py を起動できない");
    assert!(
        out.status.success(),
        "台帳と実装が食い違っている:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
