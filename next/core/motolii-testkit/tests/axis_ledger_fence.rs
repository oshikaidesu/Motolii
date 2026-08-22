//! **軸台帳(axis ledger)の柵**(裁定 = `docs/reviews/2026-08-23-normal-map-trunk.md`
//! §5 発注、`next/reference/axis/README.md` の規約を機械化する)。
//!
//! `next/reference/axis/*.tsv` は「幹の軸(A01〜A12)を全葉(`normal-map.tsv`)に
//! 1本ずつ掛けた結果」を1軸1ファイルで置く台帳。README が定める形式を、
//! `owns_justification_fence.rs`(裁定209: 禁止パターンの grep でなく、
//! 明示マーカー/構造の存在を要求する形)に倣って機械で強制する。
//!
//! ## 縛る不変(README の規約、6点)
//!
//! 1. **軸idの3重一致** — ファイル名の `A##` == `#axis:` ヘッダの `A##` ==
//!    全データ行の第1列。取り違え(行を別軸ファイルへコピペ)を構造で封じる
//! 2. `#axis` / `#question` / `#verdicts` / `#sweep` の4ヘッダが在ること。
//!    `#sweep` は1本以上(網羅を主張しない規約の担保 — GESTURES.md 前例)
//! 3. `判定` 列の値は、そのファイルの `#verdicts:` で宣言された集合のどれか
//!    (宣言しない値は使えない)
//! 4. `map_id` は `-` か、`next/reference/normal-map.tsv` に実在する id
//! 5. 全データ行の列数が一定(タブ区切りの列ずれを検出)
//! 6. `穴` 列は `穴` か `-` の2値
//!
//! ## write-set 境界(2026-08-23 発注、他レーン同時走行)
//!
//! `next/reference/axis/*.tsv` は AX-1〜AX-6 の複数レーンが同時に書いている。
//! このレーンが見るのは merge 後の全ファイルだが、**発注時点ではまだ1本も
//! 揃っていない可能性がある**——この柵はファイルが1本も無くても緑になる
//! (存在するファイルだけを検査する。空集合は違反ゼロとして扱う)。
//!
//! ## 自己参照を避ける(裁定209 の教訓)
//!
//! この柵ファイル自身は `next/reference/axis/` 配下ではなく
//! `core/motolii-testkit/tests/` 配下にあるので、収集対象(`axis/*.tsv`)に
//! 自分自身が混ざる心配はもとから無い。

use std::fs;
use std::path::{Path, PathBuf};

/// この crate の `tests/` から見て `next/` の根。
/// (`owns_justification_fence.rs` の `next_root()` と同じ深さの相対関係
/// — `core/motolii-testkit` も `next/` から見て2階層下)。
fn next_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn axis_dir() -> PathBuf {
    next_root().join("reference/axis")
}

fn normal_map_path() -> PathBuf {
    next_root().join("reference/normal-map.tsv")
}

/// `normal-map.tsv` の `id` 列(第1列)を全部集める。ヘッダ行(`id` で始まる)は除く。
fn collect_normal_map_ids(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            // ヘッダ行(`id\tcategory\t...`)はスキップ。
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if let Some(id) = line.split('\t').next() {
            ids.push(id.to_string());
        }
    }
    ids
}

/// `axis/` 配下の `*.tsv` を(再帰せず、直下だけ)集める。README/その他の
/// 非 tsv ファイルは対象外。
fn collect_axis_tsv_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("tsv") {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// ファイル名から `A\d+` を抜く(例: `A10-affordance.tsv` → `A10`)。
fn axis_id_from_filename(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let prefix: String = stem.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    if prefix.len() >= 2 && prefix.starts_with('A') && prefix[1..].chars().all(|c| c.is_ascii_digit())
    {
        Some(prefix)
    } else {
        None
    }
}

/// `#axis: A## 軸名` ヘッダから `A##` を抜く。
fn axis_id_from_header(line: &str) -> Option<String> {
    let rest = line.strip_prefix("#axis:")?.trim();
    let token = rest.split_whitespace().next()?;
    Some(token.to_string())
}

#[derive(Debug, Default)]
struct FileReport {
    rel: String,
    errors: Vec<String>,
}

#[test]
fn axis_ledger_matches_readme_contract() {
    let normal_map_ids = collect_normal_map_ids(&normal_map_path());
    let dir = axis_dir();
    let files = collect_axis_tsv_files(&dir);

    // ファイルが1本も無くても緑になる(他レーンがまだ merge されていない時点の実行に備える、発注書の指示)。
    if files.is_empty() {
        println!("axis ledger fence: `next/reference/axis/*.tsv` が0本 — 検査対象なし、緑扱い");
        return;
    }

    assert!(
        !normal_map_ids.is_empty(),
        "normal-map.tsv から id を1件も読めなかった — パス計算またはファイル破損を疑う: {}",
        normal_map_path().display()
    );

    let mut reports: Vec<FileReport> = Vec::new();

    for path in &files {
        let root = next_root();
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let mut errors: Vec<String> = Vec::new();

        let Ok(text) = fs::read_to_string(path) else {
            errors.push("読み込み不能".to_string());
            reports.push(FileReport { rel, errors });
            continue;
        };

        // --- ヘッダ収集 ---
        let mut has_axis = false;
        let mut has_question = false;
        let mut has_verdicts = false;
        let mut sweep_count = 0usize;
        let mut header_axis_id: Option<String> = None;
        let mut verdict_set: Vec<String> = Vec::new();

        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("#axis:") {
                has_axis = true;
                header_axis_id = axis_id_from_header(line);
                let _ = rest;
            } else if line.starts_with("#question:") {
                has_question = true;
            } else if let Some(rest) = line.strip_prefix("#verdicts:") {
                has_verdicts = true;
                verdict_set = rest.trim().split('|').map(|s| s.trim().to_string()).collect();
            } else if line.starts_with("#sweep:") {
                sweep_count += 1;
            }
        }

        if !has_axis {
            errors.push("`#axis:` ヘッダが無い".to_string());
        }
        if !has_question {
            errors.push("`#question:` ヘッダが無い".to_string());
        }
        if !has_verdicts {
            errors.push("`#verdicts:` ヘッダが無い".to_string());
        }
        if sweep_count == 0 {
            errors.push("`#sweep:` ヘッダが1本も無い(網羅の主張禁止規約 — 使った探索を残すこと)".to_string());
        }

        // --- 軸id 3重一致(1: ファイル名 / ヘッダ) ---
        let filename_axis_id = axis_id_from_filename(path);
        match (&filename_axis_id, &header_axis_id) {
            (Some(f), Some(h)) if f != h => {
                errors.push(format!(
                    "軸id不一致: ファイル名から `{f}`、`#axis:` ヘッダから `{h}`"
                ));
            }
            (None, _) => {
                errors.push("ファイル名から `A##` を抽出できない(命名規約 `A##-*.tsv` に従うこと)".to_string());
            }
            (Some(_), None) => {
                // `#axis:` ヘッダ自体が無い場合は上ですでに報告済み。二重報告を避ける。
            }
            _ => {}
        }

        // --- データ行の検査 ---
        let mut data_rows: Vec<(usize, Vec<String>)> = Vec::new();
        for (idx, line) in text.lines().enumerate() {
            let line_no = idx + 1;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            // ヘッダ行(`axis\tmap_id\t...`)は列名そのものなのでスキップ。
            if line.starts_with("axis\t") {
                continue;
            }
            let cols: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
            data_rows.push((line_no, cols));
        }

        if data_rows.is_empty() {
            errors.push("データ行が1行も無い".to_string());
        }

        // 5: 列数が一定。
        if let Some((_, first_cols)) = data_rows.first() {
            let expected_ncols = first_cols.len();
            for (line_no, cols) in &data_rows {
                if cols.len() != expected_ncols {
                    errors.push(format!(
                        "{line_no}行目: 列数が{expected_ncols}でなく{}(タブ区切りの列ずれ)",
                        cols.len()
                    ));
                }
            }
        }

        for (line_no, cols) in &data_rows {
            // 1: 軸id3重一致(2: 全データ行の第1列)。
            if let Some(first) = cols.first() {
                if let Some(f) = &filename_axis_id {
                    if first != f {
                        errors.push(format!(
                            "{line_no}行目: 第1列 `{first}` がファイル名由来の `{f}` と不一致"
                        ));
                    }
                }
            }

            // 3: 判定列(第4列、`axis\tmap_id\t対象\t判定\t証拠\t穴\t...`)。
            if let Some(verdict) = cols.get(3) {
                if has_verdicts && !verdict_set.iter().any(|v| v == verdict) {
                    errors.push(format!(
                        "{line_no}行目: 判定 `{verdict}` が `#verdicts:` 宣言集合 {verdict_set:?} に無い"
                    ));
                }
            }

            // 4: map_id(第2列)は `-` か normal-map.tsv 実在id。
            if let Some(map_id) = cols.get(1) {
                if map_id != "-" && !normal_map_ids.iter().any(|id| id == map_id) {
                    errors.push(format!(
                        "{line_no}行目: map_id `{map_id}` が normal-map.tsv に実在しない(`-` か実在idであること)"
                    ));
                }
            }

            // 6: 穴列(第6列)は `穴` か `-`。
            if let Some(hole) = cols.get(5) {
                if hole != "穴" && hole != "-" {
                    errors.push(format!(
                        "{line_no}行目: 穴列の値 `{hole}` が `穴`/`-` の2値でない"
                    ));
                }
            }
        }

        reports.push(FileReport { rel, errors });
    }

    let failing: Vec<&FileReport> = reports.iter().filter(|r| !r.errors.is_empty()).collect();

    assert!(
        failing.is_empty(),
        "軸台帳がREADME規約(next/reference/axis/README.md)に違反しているファイルが見つかった:\n{}",
        failing
            .iter()
            .map(|r| format!("- {}:\n{}", r.rel, r.errors.iter().map(|e| format!("    {e}")).collect::<Vec<_>>().join("\n")))
            .collect::<Vec<_>>()
            .join("\n")
    );

    println!(
        "axis ledger fence: {}本すべて規約通過(normal-map.tsv id={}件を突き合わせ)",
        reports.len(),
        normal_map_ids.len()
    );
}
