//! 自前で持っている技術の機構を数え、**天井(reference/owned-budget.tsv)と突き合わせる**。
//!
//! 憲法「技術は rerun の部品」を文ではなく数で見張る仕掛け。上でも下でも落ちるので、
//! 天井はいつも現実と一致する:
//!
//! - **超えた** = 自前を増やした。直すには天井を書き換えて commit するしかない(履歴に残る)
//! - **下回った** = 減らしたのに天井が古い。下げる commit を書く
//!
//! 走査するのは `src/`(製品のコード)だけ。`tests/` にはこの番人自身が居て、
//! 禁じている文字列を持っているので数に入れない。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn budget_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("reference/owned-budget.tsv")
}

fn collect_rs(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push((path, text));
            }
        }
    }
}

struct Budgeted {
    pattern: String,
    budget: usize,
    note: String,
}

fn budget_rows() -> Vec<Budgeted> {
    let text = std::fs::read_to_string(budget_path()).expect("owned-budget.tsv が無い");
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 2, "列が足りない行がある: {line}");
        out.push(Budgeted {
            pattern: cols[0].to_owned(),
            budget: cols[1].parse().expect("budget 列が数でない"),
            note: (*cols.get(2).unwrap_or(&"")).to_owned(),
        });
    }
    assert!(!out.is_empty(), "天井が1行も読めていない");
    out
}

fn counts() -> BTreeMap<String, (usize, Vec<String>)> {
    let mut files = Vec::new();
    collect_rs(&src_dir(), &mut files);
    assert!(!files.is_empty(), "走査対象のコードが無い");

    let mut out = BTreeMap::new();
    for row in budget_rows() {
        let mut hits = Vec::new();
        for (path, text) in &files {
            for (index, line) in text.lines().enumerate() {
                if line.contains(&row.pattern) {
                    let rel = path
                        .strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
                        .unwrap_or(path);
                    hits.push(format!("{}:{}", rel.display(), index + 1));
                }
            }
        }
        out.insert(row.pattern.clone(), (hits.len(), hits));
    }
    out
}

#[test]
fn owned_mechanisms_stay_within_the_budget() {
    let counted = counts();
    let mut over = Vec::new();
    for row in budget_rows() {
        let (count, hits) = counted.get(&row.pattern).expect("数えた");
        if *count > row.budget {
            over.push(format!(
                "  {} : {} > 天井 {}  ({})\n{}",
                row.pattern,
                count,
                row.budget,
                row.note,
                hits.iter()
                    .map(|h| format!("      {h}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
    assert!(
        over.is_empty(),
        "**自前の機構が増えている。**上流に同じ物が無いか先に確かめること\n\
         (憲法「作る前に幹から逆引きする」)。増やすと決めたなら \
         reference/owned-budget.tsv の天井を自分で上げて commit する:\n{}",
        over.join("\n")
    );
}

#[test]
fn the_budget_is_not_stale() {
    let counted = counts();
    let mut under = Vec::new();
    for row in budget_rows() {
        let (count, _) = counted.get(&row.pattern).expect("数えた");
        if *count < row.budget {
            under.push(format!(
                "  {} : {} < 天井 {}  — 天井を {} へ下げること",
                row.pattern, count, row.budget, count
            ));
        }
    }
    assert!(
        under.is_empty(),
        "**自前が減っている。**天井が現実より緩いままなので、下げて確定させる \
         (減らした事を履歴に残す。足すなら消す と同じ形):\n{}",
        under.join("\n")
    );
}

/// 今の在庫。**落ちない** — `cargo test -- --nocapture owned_inventory` で読む。
#[test]
fn report_inventory() {
    for (pattern, (count, _)) in counts() {
        println!("自前 {pattern}: {count}");
    }
    let borrowed = std::fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("reference"))
        .map(|d| {
            d.flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|x| x == "wgsl" || x == "fs" || x == "glsl")
                })
                .count()
        })
        .unwrap_or(0);
    println!("借り物の式(reference/): {borrowed}");
}
