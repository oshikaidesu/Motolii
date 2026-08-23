//! **在庫表(inventory)の柵**(総監督 2026-08-23 指示「機械抽出して、関数や
//! 意味が出せるリストが欲しい」への実装)。
//!
//! `next/reference/generated/inventory.tsv` は**人が編集しない生成物**
//! (`motolii_testkit::inventory::generate()` が吐く)。この柵は
//! **生成器を今すぐ再実行した結果と、コミット済みファイルを突き合わせ、
//! 食い違ったら赤にする**——`axis_ledger_fence.rs`/`owns_justification_fence.rs`
//! と同じ「ファイル置き場」(`core/motolii-testkit/tests/`)・同じ作法
//! (禁止パターンの grep ではなく、あるべき状態そのものとの一致を要求する、
//! 裁定209 の形の延長)。
//!
//! **この柵が赤くなったら**: ソースは変わったのに `inventory.tsv` を
//! 再生成してコミットし忘れている、ということ。直し方は
//! `cargo run --manifest-path next/Cargo.toml -p motolii-testkit --bin gen_inventory`
//! を回して差分をコミットするだけ(判断は要らない——生成物だから)。
//!
//! 生成ロジック自体の精度限界(callers のテキスト一致ヒューリスティック等)は
//! `next/core/motolii-testkit/src/inventory.rs` の doc に書いてある。この柵は
//! 「生成器とコミット済みファイルが一致しているか」だけを見る——生成ロジック
//! そのものの正しさの柵ではない。

use motolii_testkit::inventory;

#[test]
fn committed_inventory_matches_a_fresh_generation() {
    let path = inventory::generated_inventory_path();
    let committed = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "`{}` が無い — 初回は `cargo run --manifest-path next/Cargo.toml -p motolii-testkit --bin gen_inventory` で作ってコミットすること",
            path.display()
        )
    });
    let fresh = inventory::generate();

    if committed == fresh {
        let lines = fresh.lines().count().saturating_sub(1);
        println!("inventory fence: 一致(データ行 {lines} 件)");
        return;
    }

    let committed_lines: Vec<&str> = committed.lines().collect();
    let fresh_lines: Vec<&str> = fresh.lines().collect();

    let committed_set: std::collections::BTreeSet<&str> = committed_lines.iter().copied().collect();
    let fresh_set: std::collections::BTreeSet<&str> = fresh_lines.iter().copied().collect();

    let only_in_committed: Vec<&&str> = committed_set.difference(&fresh_set).take(20).collect();
    let only_in_fresh: Vec<&&str> = fresh_set.difference(&committed_set).take(20).collect();

    panic!(
        "`{}` がソースの今の状態と食い違う(生成器を回してコミットし忘れている疑い)。\n\
         コミット済み {} 行 / 新規生成 {} 行\n\
         コミット済みにだけ在る行(最大20件・古い可能性): {:#?}\n\
         新規生成にだけ在る行(最大20件・未反映の可能性): {:#?}\n\
         直し方: `cargo run --manifest-path next/Cargo.toml -p motolii-testkit --bin gen_inventory` を回して差分をコミットする。",
        path.display(),
        committed_lines.len(),
        fresh_lines.len(),
        only_in_committed,
        only_in_fresh,
    );
}
