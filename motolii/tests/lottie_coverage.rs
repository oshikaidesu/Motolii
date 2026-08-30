
use std::collections::BTreeSet;
use std::path::PathBuf;

const STATUSES: &[&str] = &["採用済", "採用予定", "不採用", "未判定", "該当なし"];

fn reference_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("reference")
}

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
    evidence: String,
    unit: String,
    source: String,
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
            evidence: cols.get(6).unwrap_or(&"").trim().to_owned(),
            unit: cols.get(7).unwrap_or(&"").trim().to_owned(),
            source: cols.get(8).unwrap_or(&"lottie").trim().to_owned(),
        });
    }
    out
}

#[test]
fn the_map_covers_the_whole_schema() {
    let schema = vocabulary_from_schema();
    let rows = coverage_rows();
    let mapped: BTreeSet<_> = rows
        .iter()
        .filter(|r| r.source == "lottie")
        .map(|r| r.key.clone())
        .collect();

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

#[test]
fn the_map_covers_the_rive_defs() {
    let dir = reference_dir().join("rive-text-defs");
    let mut defs: BTreeSet<(String, String)> = BTreeSet::new();
    let entries = std::fs::read_dir(&dir).expect("rive-text-defs が無い");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let text = std::fs::read_to_string(&path).expect("def を読めない");
            let def: serde_json::Value = serde_json::from_str(&text).expect("def が JSON でない");
            let name = def["name"].as_str().expect("name が無い");
            let object = kebab(name);
            match def.get("properties").and_then(|p| p.as_object()) {
                Some(props) if !props.is_empty() => {
                    for field in props.keys() {
                        defs.insert((object.clone(), field.clone()));
                    }
                }
                _ => {
                    defs.insert((object.clone(), String::new()));
                }
            }
        }
    }
    assert!(!defs.is_empty(), "def を1つも読めていない");

    let mapped: BTreeSet<(String, String)> = coverage_rows()
        .into_iter()
        .filter(|r| r.source == "rive")
        .map(|r| {
            let field = if r.key.2.starts_with('@') {
                String::new()
            } else {
                r.key.2.clone()
            };
            (r.key.1.clone(), field)
        })
        .collect();

    let missing: Vec<_> = defs.difference(&mapped).collect();
    assert!(
        missing.is_empty(),
        "Rive の defs にあって地図に無い項目が {}件。**読み落とし**なので表へ足すこと:\n{}",
        missing.len(),
        missing
            .iter()
            .take(20)
            .map(|(o, f)| format!("  {o}/{f}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let stale: Vec<_> = mapped.difference(&defs).collect();
    assert!(
        stale.is_empty(),
        "地図にあって Rive の defs に無い項目が {}件。上流更新で消えた行:\n{}",
        stale.len(),
        stale
            .iter()
            .take(20)
            .map(|(o, f)| format!("  {o}/{f}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.char_indices() {
        if ch.is_uppercase() && i != 0 {
            out.push('-');
        }
        out.extend(ch.to_lowercase());
    }
    out
}

#[test]
fn adopted_rows_point_at_real_code() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut sources = String::new();
    collect_sources(&root, &mut sources);
    assert!(!sources.is_empty(), "走査対象のコードが無い");

    let mut bad = Vec::new();
    for row in coverage_rows() {
        if row.status != "採用済" {
            continue;
        }
        let (g, o, f) = &row.key;
        if row.evidence.is_empty() {
            bad.push(format!("  {g}/{o}/{f}: evidence 欄が空"));
        } else if !sources.contains(&row.evidence) {
            bad.push(format!(
                "  {g}/{o}/{f}: evidence 「{}」がコードに無い",
                row.evidence
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "採用済 と書いてあるのに裏付けが無い行が {}件:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

fn collect_sources(dir: &std::path::Path, out: &mut String) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" || name == "reference" {
                continue;
            }
            collect_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push_str(&text);
            }
        }
    }
}

#[test]
fn report_work_packages() {
    use std::collections::BTreeMap;
    let rows = coverage_rows();
    let mut units: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for row in &rows {
        if row.status == "採用予定" {
            units.entry(row.unit.clone()).or_default().0 += 1;
        }
    }
    for row in &rows {
        if row.status == "採用済" && !row.unit.is_empty() {
            units.entry(row.unit.clone()).or_default().1 += 1;
        }
    }

    let mut sorted: Vec<_> = units.into_iter().collect();
    sorted.sort_by_key(|(_, (todo, _))| std::cmp::Reverse(*todo));

    println!("発注単位(採用予定 = 残り / 束の完了条件 = 全部 採用済 + evidence):");
    for (unit, (todo, done)) in &sorted {
        println!("  {unit:<16} 残り {todo:>3} / 済 {done:>3}");
    }
    println!("  合計 残り {}", sorted.iter().map(|(_, (t, _))| t).sum::<usize>());
}

#[test]
fn every_planned_row_has_a_work_package() {
    let orphans: Vec<_> = coverage_rows()
        .into_iter()
        .filter(|r| r.status == "採用予定" && r.unit.is_empty())
        .map(|r| format!("  {}/{}/{}", r.key.0, r.key.1, r.key.2))
        .collect();
    assert!(
        orphans.is_empty(),
        "採用予定 なのに発注単位が無い行が {}件。束ねないと誰の仕事か決まらない:\n{}",
        orphans.len(),
        orphans.join("\n")
    );
}

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
