//! 台帳の**証拠が今も実在するか**を全12軸へ一律に当てる。
//!
//! 2026-08-23 利用者指摘「『効きそうな所』という部分が少し怖い、まだ人の手が
//! 残っている証拠と思います」への対処。**どの軸を機械化すると効くか**を
//! supervisor の勘で選んでいたので、優先順位も導出する形にした。
//!
//! 軸ごとの規則を書く前に、**全軸へ一律に当てられる検査**が1つある —
//! `証拠` 列の `file:line` が今も実在するか。証拠が腐っている行は判定も
//! 腐っている可能性が高く、**腐りの多い軸が次に機械化すべき軸**。
//!
//! 導入時に15件の腐りを検出した。うち大半は同日の `shell/lib.rs` 分割
//! (6,228→2,127行)で行番号が失効した巻き添え。**分割した本人が台帳を
//! 追随させていなかった** — まさにこの柵が要る理由。
//!
//! 落ちたら: `python3 scripts/check_evidence.py <リポ根>` を回して直す。

use std::process::Command;

#[test]
fn ledger_evidence_still_exists() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/check_evidence.py"))
        .arg(&root)
        .output()
        .expect("check_evidence.py を起動できない");
    assert!(
        out.status.success(),
        "台帳の証拠が実在しない(file:line が消えている)。\
         コードを動かしたら台帳も追随させること:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
