//! `normal-map.tsv` の `採用済` に裏が取れるか(自己申告を増やさない柵)。
//!
//! 裁定212 が「`normal-map` は Lottie の4条件を1つも満たさない」と指摘し、実測として
//! 「main が395コミット進む間、採用済227が1行も動かなかった」を挙げていた。
//! 2026-08-23 も同じことが起き(140コミット進んで229のまま)、実際に測ると
//! **229件のうち109件は理由に実在識別子が無い**= 自己申告だった。
//!
//! `lottie-coverage` の作法を写す — **`採用済` の行は実在識別子を持ち、試験が確かめる。**
//! ただし既存109件を即座に赤にすると、緑にするために**嘘の識別子を書く圧力**がかかる
//! (裁定215 の owns 柵で立証不足を合格にしてある理由と同じ)。よって**上限を固定し、
//! これ以上増やさない**形にする。**減らすのはよい。**
//!
//! 落ちたら: `python3 scripts/check_adoption.py <リポ根>` を回し、
//! **`採用済` にした行の `理由` 列へ在庫表に載っている識別子を書く**。

use std::process::Command;

#[test]
fn adopted_rows_do_not_grow_without_evidence() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("リポジトリ根")
        .to_path_buf();
    let out = Command::new("python3")
        .arg(root.join("scripts/check_adoption.py"))
        .arg(&root)
        .output()
        .expect("check_adoption.py を起動できない");
    assert!(
        out.status.success(),
        "`採用済` の自己申告が増えた:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
