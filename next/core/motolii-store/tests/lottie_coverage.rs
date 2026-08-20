//! **Lottie の全語彙を地図にして、抜けを機械で出す。**
//!
//! 「作る瞬間に読む」方式は、読まなかった物が**構造的に見えない**。
//! そこで上流のスキーマ(`next/reference/lottie.schema.json`、上流そのまま)を正本にして、
//! 語彙を全部列挙した表(`lottie-coverage.tsv`)と突き合わせる。
//!
//! この試験が落ちる条件:
//! - スキーマにあるのに表に無い(= **読み落とし**)
//! - 表にあるのにスキーマに無い(= 古い行が残っている。上流更新で起きる)
//! - 状態語が固定集合の外
//!
//! Lottie は Bodymovin が AE のデータ模型を吐いた物なので、**実質 OSS の AE 解析**である。
//! よってこの表の「未判定」の数は、**AE の意味のうち Motolii がまだ向き合っていない量**に近い。

use std::collections::BTreeSet;
use std::path::PathBuf;

const STATUSES: &[&str] = &["採用済", "採用予定", "不採用", "未判定", "該当なし"];

fn reference_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../reference")
}

/// 自分自身が定義する property だけ。`$ref` は解決せず `@extends` 行で見せる
/// (解決すると基底の field が全派生型へ重複して、地図が読めなくなる)。
fn own_props(node: &serde_json::Value, out: &mut Vec<(String, String)>) {
    let Some(map) = node.as_object() else {
        return;
    };
    if let Some(props) = map.get("properties").and_then(|p| p.as_object()) {
        for (name, value) in props {
            let title = value
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .replace('\t', " ");
            out.push((name.clone(), title));
        }
    }
    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(list) = map.get(key).and_then(|l| l.as_array()) {
            for sub in list {
                if sub.get("$ref").is_some() {
                    continue;
                }
                own_props(sub, out);
            }
        }
    }
}

fn refs_of(node: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let Some(map) = node.as_object() else {
        return out;
    };
    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(list) = map.get(key).and_then(|l| l.as_array()) {
            for sub in list {
                if let Some(r) = sub.get("$ref").and_then(|r| r.as_str()) {
                    out.push(r.replace("#/$defs/", ""));
                }
            }
        }
    }
    out
}

/// スキーマから語彙を導く。生成器と**同じ規則**でなければ意味がない。
fn vocabulary_from_schema() -> BTreeSet<(String, String, String)> {
    let text = std::fs::read_to_string(reference_dir().join("lottie.schema.json"))
        .expect("lottie.schema.json が無い(上流から取ってきて reference/ へ置く)");
    let schema: serde_json::Value = serde_json::from_str(&text).expect("スキーマが JSON でない");
    let defs = schema["$defs"].as_object().expect("$defs が無い");

    let mut out = BTreeSet::new();
    for (group, items) in defs {
        for (name, node) in items.as_object().expect("group が object でない") {
            for parent in refs_of(node) {
                out.insert((group.clone(), name.clone(), format!("@extends:{parent}")));
            }
            let mut props = Vec::new();
            own_props(node, &mut props);
            if props.is_empty() && refs_of(node).is_empty() {
                out.insert((group.clone(), name.clone(), String::new()));
            }
            for (field, _) in props {
                out.insert((group.clone(), name.clone(), field));
            }
        }
    }
    out
}

struct Row {
    key: (String, String, String),
    status: String,
}

fn coverage_rows() -> Vec<Row> {
    let text = std::fs::read_to_string(reference_dir().join("lottie-coverage.tsv"))
        .expect("lottie-coverage.tsv が無い");
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with("group\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 5, "列が足りない行がある: {line}");
        let (group, object, field, title, status) =
            (cols[0], cols[1], cols[2], cols[3], cols[4]);
        // `@extends` 行は field 欄が `@extends`、title 欄が継承元。
        let field = if field == "@extends" {
            format!("@extends:{title}")
        } else {
            field.to_owned()
        };
        assert!(
            STATUSES.contains(&status),
            "状態語が固定集合の外: 「{status}」({group}/{object}/{field})"
        );
        out.push(Row {
            key: (group.to_owned(), object.to_owned(), field),
            status: status.to_owned(),
        });
    }
    out
}

#[test]
fn the_map_covers_the_whole_schema() {
    let schema = vocabulary_from_schema();
    let rows = coverage_rows();
    let mapped: BTreeSet<_> = rows.iter().map(|r| r.key.clone()).collect();

    let missing: Vec<_> = schema.difference(&mapped).collect();
    assert!(
        missing.is_empty(),
        "スキーマにあって地図に無い項目が {}件。**読み落とし**なので表へ足すこと:\n{}",
        missing.len(),
        missing
            .iter()
            .take(20)
            .map(|(g, o, f)| format!("  {g}/{o}/{f}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let stale: Vec<_> = mapped.difference(&schema).collect();
    assert!(
        stale.is_empty(),
        "地図にあってスキーマに無い項目が {}件。上流更新で消えた行なので落とすこと:\n{}",
        stale.len(),
        stale
            .iter()
            .take(20)
            .map(|(g, o, f)| format!("  {g}/{o}/{f}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 地図の状態を毎回出す。**未判定の数が、まだ向き合っていない AE の意味の量**。
#[test]
fn report_coverage() {
    let rows = coverage_rows();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for row in &rows {
        *counts.entry(row.status.as_str()).or_default() += 1;
    }
    let real: usize = rows.iter().filter(|r| r.status != "該当なし").count();
    println!("Lottie 地図: 全 {} 項目(継承行を除く {real} が判断対象)", rows.len());
    for (status, count) in &counts {
        println!("  {status:<8} {count:>4}");
    }
    let undecided = counts.get("未判定").copied().unwrap_or(0);
    println!(
        "  → 未判定 {undecided} / {real} = {:.0}% がまだ向き合っていない",
        undecided as f64 / real as f64 * 100.0
    );
}
