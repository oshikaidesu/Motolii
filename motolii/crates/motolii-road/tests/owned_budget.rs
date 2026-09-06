//! 自前で持っている技術の機構を数える。
//!
//! 憲法「作ってよいのは編集の意味とエフェクトのデータだけ。技術は rerun の部品」を、
//! 文ではなく**数**で見張る。天井は `reference/owned-budget.tsv`。
//!
//! **超えても、下回っても落ちる。**
//! 超えた = 自前を増やした。通すには天井を自分で書き換えて commit するしかなく、
//! 履歴に残る。下回った = 減らしたのに天井が古い。下げる commit が仕事の成果。
//!
//! これが無いと、**自作が常に一番軽い道**になる。今日1日でも、上流に在る物を
//! 3度自作した(窓を触る器具・画素の覚え書き・静止画の読み込み)。

use std::path::Path;

fn rust_sources(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push(text);
            }
        }
    }
}

#[test]
fn the_machinery_we_own_matches_its_ceiling() {
    // motolii-road は doc と render を 1 本に link する道路。家 5 つのうち Rust の 3 つ
    // (doc・render・ui/native)を、旧 src/ui を数えずにここから見る。
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let table = std::fs::read_to_string(root.join("reference/owned-budget.tsv"))
        .expect("reference/owned-budget.tsv");

    let mut sources = Vec::new();
    rust_sources(&root.join("crates"), &mut sources);
    rust_sources(&root.join("ui/native/src"), &mut sources);
    assert!(!sources.is_empty(), "crates・ui/native を読めていない");

    let mut off = Vec::new();
    for line in table.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let (Some(pattern), Some(budget)) = (parts.next(), parts.next()) else {
            continue;
        };
        let budget: usize = budget.trim().parse().unwrap_or_else(|_| {
            panic!("天井が数でない: {line}");
        });
        let found: usize = sources
            .iter()
            .map(|text| text.matches(pattern).count())
            .sum();
        if found != budget {
            let word = if found > budget { "増やした" } else { "減らしたのに天井が古い" };
            off.push(format!("{pattern}: 天井 {budget} / 実測 {found} — {word}"));
        }
    }

    assert!(
        off.is_empty(),
        "自前の機構が天井と合っていない。数を直すか、天井を書き換えて commit する:\n  {}",
        off.join("\n  ")
    );
}
