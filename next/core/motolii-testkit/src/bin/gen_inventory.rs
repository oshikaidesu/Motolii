//! `next/reference/generated/inventory.tsv` を書き直す生成器。
//!
//! ロジックは `motolii_testkit::inventory` に在る(`tests/inventory_fence.rs`
//! と同じ関数を呼ぶ——生成と検査が別ロジックにならないようにするため)。
//!
//! 使い方:
//! ```text
//! cargo run --manifest-path next/Cargo.toml -p motolii-testkit --bin gen_inventory
//! ```

fn main() {
    let tsv = motolii_testkit::inventory::generate();
    let path = motolii_testkit::inventory::generated_inventory_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("reference/generated を作れない");
    }
    std::fs::write(&path, &tsv).expect("inventory.tsv を書けない");
    eprintln!(
        "gen_inventory: {} 行(ヘッダ込み)を {} へ書いた",
        tsv.lines().count(),
        path.display()
    );
}
