//! 裁定142の柵、この crate 側の再構築(裁定160 切片8)。
//!
//! `motolii-shell/tests/suite/tonmana_token_fence.rs` が元々 `inspector_pane.rs`
//! を走査していたが、切片8で当該ファイルがこの crate(`motolii-inspector-pane`)の
//! `src/lib.rs` へ抽出された。**この柵はその走査対象をここへ移設したもの**——
//! 切片9(`settings_pane.rs` → `motolii-settings-pane`)では同型の柵を抽出先に
//! 再構築せず「fence 走査ギャップ」として記録するに留めたが(元ファイルの
//! doc comment 参照)、この切片ではその前例を繰り返さない
//! (`docs/reviews/2026-08-21-pane-split-survey.md` 切片8「柵は緩めない・消さない」
//! の指示どおり)。走査ロジック・境界線の判断はすべて元の柵からの機械的な複製 —
//! 新しい規則は増やしていない。
//!
//! ## 走査対象
//! この crate の `src/*.rs` 全部([`SCANNED_FILES`])。**2026-08-22 追記**:
//! 利用者指摘(4,907行 lib.rs が merge 事故の原因)を受け、`lib.rs` 単一ファイル
//! だった内容を section 単位モジュールへ分割した(裁定160 pane 分割と同じ
//! 「積む前に割る」) — 柵は「lib.rs だけ」から「pane crate の全 `.rs`」へ
//! 走査対象を広げて追随する(柵は緩めない・消さない、分割前と同じ検出力を保つ)。
//! `#[cfg(test)] mod tests { .. }` 以降は対象外([`scannable_prefix`]、
//! `lib.rs` の末尾に集約された唯一の inline test module に効く)。
//!
//! ## 何を「違反」とするか(境界線は元の柵と同一 — 詳細は
//! `motolii-shell/tests/suite/tonmana_token_fence.rs` のモジュール doc 参照)
//! - **色**: [`COLOR_MARKERS`] — `iced::Color::from_rgb[8]`/`from_rgba[8]` 呼び出し・
//!   `Color { .. }` 構造体リテラル。
//! - **寸法**: [`CALL_MARKERS`]/[`STRUCT_LITERAL_MARKERS`] — `.size(`/`.padding(`/
//!   `.spacing(`/`Length::Fixed(` 等の呼び出し引数、`Border { .. }` の `width:`/
//!   `radius:` フィールド。
//! - `0`/`0.0`/`1`/`1.0`(符号つき含む)は対象外、`dims.`/`colors.`/`_px` を含む
//!   行は「token 由来」として対象外。

use std::path::{Path, PathBuf};

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// section 単位モジュール分割(2026-08-22)後の全ソースファイル。`lib.rs` は
/// 組み立てと再輸出+末尾の inline test module だけ、実装本体は7つの section
/// モジュールへ散った — 柵はどのファイルに書いても同じ検出力であるべきなので、
/// 分割で増えたファイルをそのまま追加した(`motolii-shell` 側の
/// `SCANNED_FILES` から `inspector_pane.rs` を落とし、この1エントリへ
/// 引き継いだ経緯は変わらない)。
const SCANNED_FILES: &[&str] = &[
    "lib.rs",
    "attrs.rs",
    "audio.rs",
    "chrome.rs",
    "effects.rs",
    "mask.rs",
    "projection.rs",
    "text/mod.rs",
    "transform/mod.rs",
];

/// `#[cfg(test)]\nmod tests {` の手前までを返す(inline test module は対象外)。
fn scannable_prefix(source: &str) -> &str {
    const MARKER: &str = "#[cfg(test)]\nmod tests {";
    match source.find(MARKER) {
        Some(idx) => &source[..idx],
        None => source,
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find("//") {
        Some(idx) => &line[..idx],
        None => line,
    }
}

/// `dims.`/`colors.` 参照が同じ行にあれば「token 由来の式」とみなす。`_px`
/// 接尾辞も同格に扱う(元の柵の判断をそのまま引き継ぐ)。
fn line_is_token_derived(line: &str) -> bool {
    line.contains("dims.") || line.contains("colors.") || line.contains("_px")
}

/// 「寸法の不在」(0)・「物理1px床」(1.0、`.max(1.0)` 系)の値そのものは
/// 個別に問い直さない。
fn is_exempt_literal(token: &str) -> bool {
    matches!(token.trim_start_matches('-'), "0" | "0.0" | "1" | "1.0")
}

/// `line` 中の「裸の数値リテラル」を左から拾う。識別子の一部(`f32` の `32`
/// 等、直前が英数字/`_`)や、既に拾った小数点以下の続きは拾わない。
fn bare_numeric_literals(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let prev_is_ident = i > 0
                && (chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.');
            if prev_is_ident {
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                continue;
            }
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let mut token: String = chars[start..i].iter().collect();
            if start > 0 && chars[start - 1] == '-' {
                let before_minus = if start >= 2 { Some(chars[start - 2]) } else { None };
                let is_unary = !matches!(
                    before_minus,
                    Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == ')' || c == ']'
                );
                if is_unary {
                    token.insert(0, '-');
                }
            }
            out.push(token);
        } else {
            i += 1;
        }
    }
    out
}

#[derive(Debug, Clone)]
struct Violation {
    file: &'static str,
    line: usize,
    text: String,
    kind: &'static str,
}

/// 無条件違反(中身の値を問わない) — pane 側で `Color` を直接組み立てること
/// 自体が裁定142違反。
const COLOR_MARKERS: &[&str] = &[
    "Color::from_rgb(",
    "Color::from_rgba(",
    "Color::from_rgb8(",
    "Color::from_rgba8(",
    "Color {",
];

/// 呼び出し引数(丸括弧)の中を見るマーカー。
const CALL_MARKERS: &[&str] = &[
    ".size(",
    ".padding(",
    ".spacing(",
    "Length::Fixed(",
    "fill_rect(",
    "stroke_rect(",
    "stroke_v(",
    "stroke_h(",
];

/// 構造体リテラル(波括弧)の中の特定フィールドだけを見るマーカー。
/// `(open_marker, open_char, close_char, field_markers)`。
const STRUCT_LITERAL_MARKERS: &[(&str, char, char, &[&str])] = &[
    ("Border {", '{', '}', &["width:", "radius:"]),
    ("Rect {", '{', '}', &[" w: ", " h: "]),
];

/// `text` 内で `marker` の直後にある開き括弧(`open`)から、対応する閉じ括弧
/// (`close`)までの行 index(0-indexed, inclusive)を、深さで追って返す。
fn bracket_region_end(lines: &[&str], start_line: usize, marker: &str, open: char, close: char) -> usize {
    let marker_col = strip_comment(lines[start_line])
        .find(marker)
        .expect("caller only invokes this where marker is present");
    let open_col = marker_col + marker.len() - 1; // marker の最後の文字が開き括弧そのもの。
    let mut depth: i32 = 0;
    let mut started = false;
    for (li, raw) in lines.iter().enumerate().skip(start_line) {
        let line = strip_comment(raw);
        let from = if li == start_line { open_col } else { 0 };
        for ch in line.chars().skip(from) {
            if ch == open {
                depth += 1;
                started = true;
            } else if ch == close {
                depth -= 1;
            }
            if started && depth == 0 {
                return li;
            }
        }
    }
    lines.len().saturating_sub(1)
}

/// [`bracket_region_end`] で決めた行範囲を、行ごとに(`dims.`/`colors.` を含む
/// 行は除いて)[`bare_numeric_literals`] で走査し、免除以外を [`Violation`] へ
/// 積む。
fn scan_region(
    file: &'static str,
    lines: &[&str],
    start_line: usize,
    end_line: usize,
    kind: &'static str,
    field_filter: Option<&[&str]>,
    violations: &mut Vec<Violation>,
) {
    for (offset, raw) in lines[start_line..=end_line].iter().enumerate() {
        let line_no = start_line + offset + 1;
        let line = strip_comment(raw);
        if line_is_token_derived(line) {
            continue;
        }
        if let Some(fields) = field_filter {
            if !fields.iter().any(|f| line.contains(f)) {
                continue;
            }
        }
        for literal in bare_numeric_literals(line) {
            if !is_exempt_literal(&literal) {
                violations.push(Violation {
                    file,
                    line: line_no,
                    text: raw.trim().to_owned(),
                    kind,
                });
            }
        }
    }
}

/// 1ファイル分のテキストを走査する(実ファイル・合成 fixture 文字列の両方から
/// 呼べるよう、`text` を引数で受け取る)。
fn scan_text(file: &'static str, text: &str) -> Vec<Violation> {
    let scannable = scannable_prefix(text);
    let lines: Vec<&str> = scannable.lines().collect();
    let mut violations = Vec::new();

    for (idx, raw) in lines.iter().enumerate() {
        let line = strip_comment(raw);
        if COLOR_MARKERS.iter().any(|m| line.contains(m)) {
            violations.push(Violation {
                file,
                line: idx + 1,
                text: raw.trim().to_owned(),
                kind: "color",
            });
        }
    }

    for (idx, raw) in lines.iter().enumerate() {
        let line = strip_comment(raw);
        for marker in CALL_MARKERS {
            if line.contains(marker) {
                let end = bracket_region_end(&lines, idx, marker, '(', ')');
                scan_region(file, &lines, idx, end, "px", None, &mut violations);
            }
        }
        for (marker, open, close, fields) in STRUCT_LITERAL_MARKERS {
            if line.contains(marker) {
                let end = bracket_region_end(&lines, idx, marker, *open, *close);
                scan_region(file, &lines, idx, end, "px", Some(fields), &mut violations);
            }
        }
    }

    violations.sort_by_key(|v| v.line);
    violations.dedup_by(|a, b| a.line == b.line && a.text == b.text);
    violations
}

fn scan_file(file: &'static str) -> Vec<Violation> {
    let path = src_dir().join(file);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} を読めない: {err}", path.display()));
    scan_text(file, &text)
}

fn format_violations(violations: &[Violation]) -> String {
    violations
        .iter()
        .map(|v| format!("  {}:{} [{}] {}", v.file, v.line, v.kind, v.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **本命**: `lib.rs`(旧 `inspector_pane.rs`)に raw 色値・px 直書きが無いこと。
/// 見つかったら具体的な行を列挙して落ちる。
#[test]
fn pane_source_files_do_not_construct_raw_colors_or_dimension_literals() {
    let mut all_violations = Vec::new();
    for file in SCANNED_FILES {
        all_violations.extend(scan_file(file));
    }
    assert!(
        all_violations.is_empty(),
        "raw 色値・px 直書きが {} 件見つかった(裁定142違反 — tokens 経由へ寄せること):\n{}",
        all_violations.len(),
        format_violations(&all_violations)
    );
}

// ---------------------------------------------------------------------------
// 明示除外リスト(裁定142 の3種のうち、この crate が抱える分) — ファイル名+
// 識別子単位、理由コメント必須。`motolii-shell` 側の柵と同じ2エントリを
// `inspector_pane.rs` → `lib.rs` へ file だけ書き換えて引き継ぐ。
// ---------------------------------------------------------------------------

struct Exclusion {
    file: &'static str,
    identifier: &'static str,
    reason: &'static str,
}

const EXCLUSIONS: &[Exclusion] = &[
    // --- (1) 物理1px hairline 床 ---------------------------------------
    // 2026-08-22 section 分割で `value_cell_height`/`glyph_height` は
    // `lib.rs` → `chrome.rs`(値セル・共通 widget の意匠を持つモジュール)へ
    // 移設した — 除外リストは実体の置き場に追随する(表を stale にしない)。
    Exclusion {
        file: "chrome.rs",
        identifier: "fn value_cell_height",
        reason: "(dims.inspector_row_height - dims.spacing_s).max(1.0) — 物理1px床(裁定142除外1)。\
                 1.0はゼロ/負幅セルを防ぐ最小可視化の床であって独立した意匠値ではない \
                 (この柵は0/1リテラルを一般免除しているので実際には走査へ引っかからないが、\
                 除外(1)の実例としてここに記録する)。",
    },
    Exclusion {
        file: "chrome.rs",
        identifier: "fn glyph_height",
        reason: "(dims.inspector_row_height - dims.spacing_xs).max(1.0) — 同上(Key/M/S glyph 列の床)。",
    },
];

/// 除外リストの識別子が実ソースに存在し続けることを見張る — コードが変わって
/// 除外の前提が崩れた(リネーム・削除)場合に、表が silently stale になるのを防ぐ。
#[test]
fn exclusion_identifiers_still_exist_in_their_named_file() {
    for entry in EXCLUSIONS {
        let path = src_dir().join(entry.file);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{} を読めない: {err}", path.display()));
        assert!(
            text.contains(entry.identifier),
            "除外リストの識別子 `{}` が {} に見つからない — コードが変わって除外の前提が崩れている\
             (表を更新するか、除外を取り下げること)。理由: {}",
            entry.identifier,
            entry.file,
            entry.reason
        );
    }
}

// ---------------------------------------------------------------------------
// 柵の自己検分 — 合成 fixture 文字列に対して、拾うべき違反を実際に拾い、
// 免除すべきものを実際に免除することを確認する(元の柵からの機械的な複製)。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod fence_self_tests {
    use super::*;

    #[test]
    fn scannable_prefix_cuts_off_at_the_inline_test_module() {
        let src = "fn real() {}\n#[cfg(test)]\nmod tests {\n    fn t() { let x = 5.0; }\n}\n";
        let prefix = scannable_prefix(src);
        assert!(prefix.contains("fn real"));
        assert!(!prefix.contains("let x = 5.0"));
    }

    #[test]
    fn bare_numeric_literals_skips_identifier_digits_and_field_access() {
        assert_eq!(bare_numeric_literals("dims.spacing_m"), Vec::<String>::new());
        assert_eq!(bare_numeric_literals("w: f32,"), Vec::<String>::new());
        assert_eq!(bare_numeric_literals(".padding(dims.spacing_m)"), Vec::<String>::new());
        assert_eq!(bare_numeric_literals(".padding(12.5)"), vec!["12.5".to_owned()]);
        assert_eq!(bare_numeric_literals("width_px.max(1.0)"), vec!["1.0".to_owned()]);
    }

    #[test]
    fn zero_and_one_literals_are_exempt_but_others_are_not() {
        assert!(is_exempt_literal("0"));
        assert!(is_exempt_literal("0.0"));
        assert!(is_exempt_literal("1"));
        assert!(is_exempt_literal("1.0"));
        assert!(is_exempt_literal("-1.0"));
        assert!(!is_exempt_literal("2.0"));
        assert!(!is_exempt_literal("46"));
    }

    #[test]
    fn a_raw_color_construction_call_is_caught_unconditionally() {
        let synthetic = "let c = iced::Color::from_rgb(0.5, 0.2, 0.1);\n";
        let violations = scan_text("synthetic.rs", synthetic);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].kind, "color");
    }

    #[test]
    fn a_bare_color_struct_literal_is_caught_even_with_variable_fields() {
        let synthetic = "let c = Color { r: red, g: green, b: blue, a: 1.0 };\n";
        let violations = scan_text("synthetic.rs", synthetic);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].kind, "color");
    }

    #[test]
    fn a_raw_padding_literal_is_caught() {
        let synthetic = ".padding(17.0)\n";
        let violations = scan_text("synthetic.rs", synthetic);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].kind, "px");
    }

    #[test]
    fn a_token_derived_padding_is_not_caught() {
        let synthetic = ".padding([0.0, dims.spacing_m])\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }

    #[test]
    fn a_multiline_border_width_literal_is_caught() {
        let synthetic = "border: iced::Border {\n    color: colors.border_default,\n    width: 3.0,\n    radius: 0.0.into(),\n},\n";
        let violations = scan_text("synthetic.rs", synthetic);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert!(violations[0].text.contains("width: 3.0"), "{violations:#?}");
    }

    #[test]
    fn a_token_derived_border_width_is_not_caught() {
        let synthetic = "border: iced::Border {\n    color: colors.border_default,\n    width: dims.border_width,\n    radius: 0.0.into(),\n},\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }

    #[test]
    fn a_multiline_fill_rect_call_with_a_raw_argument_is_caught() {
        let synthetic = "fill_rect(\n    &mut canvas,\n    padding,\n    y,\n    99.0,\n    header_h,\n    to_rgba(colors.surface_panel, 1.0),\n);\n";
        let violations = scan_text("synthetic.rs", synthetic);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert!(violations[0].text.contains("99.0"), "{violations:#?}");
    }

    #[test]
    fn a_multiline_fill_rect_call_entirely_derived_from_tokens_is_clean() {
        let synthetic = "fill_rect(\n    &mut canvas,\n    padding,\n    y,\n    content_width,\n    header_h,\n    to_rgba(colors.surface_panel, 1.0),\n);\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }

    #[test]
    fn a_raw_rect_literal_field_is_caught_across_lines() {
        let synthetic = "let rect = Rect {\n    x: padding,\n    y,\n    w: 12.0,\n    h: stage_h,\n};\n";
        let violations = scan_text("synthetic.rs", synthetic);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert!(violations[0].text.contains("w: 12.0"), "{violations:#?}");
    }

    #[test]
    fn the_rect_struct_definition_itself_is_not_a_false_positive() {
        let synthetic = "struct Rect {\n    x: f32,\n    y: f32,\n    w: f32,\n    h: f32,\n}\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }

    #[test]
    fn domain_math_outside_any_marker_is_never_touched() {
        let synthetic = "opacity.value *= 100.0;\nlet step = 0.5;\nlet byte = (c * 255.0).round();\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }
}
