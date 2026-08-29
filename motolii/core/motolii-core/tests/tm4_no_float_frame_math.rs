//! TM-4 の**構造版**。
//!
//! 既存の `tm4_no_scattered_frame_conversion.rs` は禁止パターンを**5個の文字列**で
//! 探していた。2026-08-20 の敵対的レビューで、`motolii-engine` が
//! `t.num() as f64 / t.den() as f64 * fps.num() as f64 / fps.den() as f64` と書いており
//! **どの文字列にも一致せず素通りしていた**ことが判明した。
//! リセット裁定 §1 が旧 workspace を「柵が構造でなく宣言」と断罪したのと同じ型の失敗である。
//!
//! そこでパターン列挙をやめ、**「fps と f64 が同じ式に出てくること」自体**を禁じる。
//! 正準口(`try_to_frame_floor` / `try_to_frame_round` / `try_from_frame`)を使えば
//! f64 は要らないので、出てきたら手書きの写像が生えた合図である。

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" {
                continue;
            }
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_float_math_between_time_and_frames_outside_core() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    // core 自身は正準口の実装を持つので対象外。
    let core = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut files = Vec::new();
    collect_rs(&workspace, &mut files);
    assert!(!files.is_empty(), "走査対象が無い: {}", workspace.display());

    let mut violations = Vec::new();
    for path in files {
        if path.starts_with(core) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // この試験自身の説明文を拾わない。
        if path.file_name().and_then(|n| n.to_str()) == Some("tm4_no_float_frame_math.rs") {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let mentions_fps = code.contains("fps") || code.contains("Fps");
            let mentions_float = code.contains("f64") || code.contains("f32");
            if mentions_fps && mentions_float {
                violations.push(format!(
                    "{}:{}: {}",
                    path.strip_prefix(&workspace).unwrap_or(&path).display(),
                    index + 1,
                    code.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "時刻とフレームの写像を浮動小数で書いている箇所がある。\n\
         正準口(`RationalTime::try_to_frame_floor` / `try_to_frame_round` / \
         `try_from_frame`)を使うこと:\n{}",
        violations.join("\n")
    );
}
