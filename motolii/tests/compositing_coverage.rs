//! 技術の幹(W3C Compositing and Blending Level 1)からの逆引き。
//!
//! `lottie_coverage` が「意味の幹」(Bodymovin 経由の AE のデータモデル)に対して
//! やっていることを、「技術の幹」に対してやる。骨は上流そのままの
//! `reference/vello-blend.wgsl` から機械生成するので、地図の側で語彙を作れない。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const STATUSES: &[&str] = &["採用済", "採用予定", "不採用", "未判定", "該当なし"];

fn reference_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("reference")
}

/// 上流の実装が持つ語彙(`const MIX_* = n;` / `const COMPOSE_* = n;`)を骨にする。
fn vocabulary_from_upstream() -> BTreeSet<(String, String)> {
    let text = std::fs::read_to_string(reference_dir().join("vello-blend.wgsl"))
        .expect("vello-blend.wgsl が無い(PINNED_VELLO_BLEND.md の出所から置き直す)");
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("const ") else {
            continue;
        };
        let Some((name, _)) = rest.split_once(" = ") else {
            continue;
        };
        let (group, mode) = if let Some(m) = name.strip_prefix("MIX_") {
            ("mix", m)
        } else if let Some(m) = name.strip_prefix("COMPOSE_") {
            ("compose", m)
        } else {
            continue;
        };
        out.insert((group.to_owned(), mode.to_lowercase().replace('_', "-")));
    }
    out
}

struct Row {
    key: (String, String),
    status: String,
    evidence: String,
}

fn coverage_rows() -> Vec<Row> {
    let text = std::fs::read_to_string(reference_dir().join("compositing-coverage.tsv"))
        .expect("compositing-coverage.tsv が無い");
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.starts_with("group\t") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 5, "列が足りない行がある: {line}");
        let status = cols[4];
        assert!(
            STATUSES.contains(&status),
            "状態語が固定集合の外: 「{status}」({}/{})",
            cols[0],
            cols[1]
        );
        out.push(Row {
            key: (cols[0].to_owned(), cols[1].to_owned()),
            status: status.to_owned(),
            evidence: cols.get(6).unwrap_or(&"").trim().to_owned(),
        });
    }
    out
}

#[test]
fn the_map_covers_the_whole_vocabulary() {
    let upstream = vocabulary_from_upstream();
    assert!(!upstream.is_empty(), "上流から語彙を1つも読めていない");
    let mapped: BTreeSet<_> = coverage_rows().into_iter().map(|r| r.key).collect();

    let missing: Vec<_> = upstream.difference(&mapped).collect();
    assert!(
        missing.is_empty(),
        "上流にあって地図に無い語が {}件。**読み落とし**なので表へ足すこと:\n{}",
        missing.len(),
        missing
            .iter()
            .map(|(g, o)| format!("  {g}/{o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let stale: Vec<_> = mapped.difference(&upstream).collect();
    assert!(
        stale.is_empty(),
        "地図にあって上流に無い語が {}件。版が上がって消えた行なので落とすこと:\n{}",
        stale.len(),
        stale
            .iter()
            .map(|(g, o)| format!("  {g}/{o}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn adopted_rows_point_at_real_code() {
    let mut sources = String::new();
    collect_rs(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"), &mut sources);
    assert!(!sources.is_empty(), "走査対象のコードが無い");

    let mut bad = Vec::new();
    for row in coverage_rows() {
        if row.status != "採用済" {
            continue;
        }
        let (g, o) = &row.key;
        if row.evidence.is_empty() {
            bad.push(format!("  {g}/{o}: evidence 欄が空"));
        } else if !sources.contains(&row.evidence) {
            bad.push(format!(
                "  {g}/{o}: evidence 「{}」がコードに無い",
                row.evidence
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "「採用済」と書いた行の裏づけが取れない {}件:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// 未判定がどれだけ残っているか。**落ちない** — 距離を数えて出すだけ。
#[test]
fn report_coverage() {
    let rows = coverage_rows();
    for group in ["mix", "compose"] {
        let of_group: Vec<_> = rows.iter().filter(|r| r.key.0 == group).collect();
        let adopted = of_group.iter().filter(|r| r.status == "採用済").count();
        let undecided = of_group.iter().filter(|r| r.status == "未判定").count();
        println!(
            "{group}: 全{} 採用済{adopted} 未判定{undecided}",
            of_group.len()
        );
    }
}

fn collect_rs(dir: &Path, out: &mut String) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push_str(&text);
            }
        }
    }
}
