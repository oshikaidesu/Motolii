//! 台帳どうしが同じ対象について食い違っていないか(ジグソーの噛み合わせ)。
//!
//! 2026-08-23 利用者の観察: 「実装する各要素は独立しておらず蜘蛛の巣のように
//! マップを構造し、互いに相互補完している(まるでジグソーパズル)。訂正が必要な
//! 部分は、既に台帳から得れるのでは?」
//!
//! そのとおりだった。同じ対象は複数の軸に別々の行を持つので、**軸どうしが
//! 食い違ったらどちらかが嘘**。ピースを検分せず、隣と合わないことだけで分かる。
//!
//! **この検査はリポジトリを読まない**(台帳だけ)。だから安く、どこを読むかの
//! 選択バイアスも入らない。導入時に10件検出し、うち8件は「台帳の語彙が粗い」
//! (`採用済` が「1つに対して効く」しか意味していなかった)ことの露出だった。
//!
//! 落ちたら: `python3 scripts/check_coherence.py <リポ根>` を回して直す。

use std::process::Command;

#[test]
fn ledgers_do_not_contradict_each_other() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/check_coherence.py"))
        .arg(&root)
        .output()
        .expect("check_coherence.py を起動できない");
    assert!(
        out.status.success(),
        "台帳どうしが同じ対象について食い違っている。\
         どちらかが古いか、語彙が粗い:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
