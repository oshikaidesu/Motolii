//! 裁定142の柵(利用者裁定「全パネルのルールにして、あとから変更すると全部
//! 変わるやつ」): motolii-shell の pane コードへの **raw 色値・px 直書きを
//! 落ちるテストで禁止する**。`crates/motolii-testkit/tests/ui_toolkit_dep_policy.rs`
//! と同型のソース走査型の柵(依存グラフではなく、ソーステキストを読んで違反を
//! 具体的な行で落とす)。
//!
//! ## 走査対象
//! `next/DECISIONS.md` 裁定142 EXACT TARGET が名指しした5ファイル(すべて
//! `src/` 直下): [`SCANNED_FILES`]。**`tokens.rs` 自身・`fixture.rs`(テスト
//! データ)は対象外**(発注書 KNOWN) — 値の正本と、正本を読むための試験データは
//! この柵の対象ではない。`inspector_pane.rs`/`settings_pane.rs` はソース内に
//! `#[cfg(test)] mod tests { .. }` を持つ — [`scannable_prefix`] がその手前で
//! 切り、test 内の生値(assert の期待値等)は対象にしない。
//!
//! ## 何を「違反」とするか(境界線はここで決める — 発注書 KNOWN 4)
//! 全ての raw 数値・raw 色を1文字残らず禁止すると、ループ範囲・opacity%換算・
//! drag 感度表(`inspector_pane.rs::drag_step_per_pixel`)・RGBA→u8 変換の
//! `255.0`・`decimals` 桁数のような**ドメインロジック**まで巻き込んで誤検出
//! だらけになる。この柵が縛るのは **widget/描画の構築呼び出しへ直接渡る
//! 色・寸法値だけ**:
//!
//! - **色**: [`COLOR_MARKERS`] — `iced::Color::from_rgb[8]`/`from_rgba[8]` 呼び出し・
//!   `Color { .. }` 構造体リテラル。マッチしたら**無条件に違反**(中身が変数でも
//!   pane 側で `Color` を組み立てていること自体が裁定142違反 — 色は
//!   `tokens::Colors` 経由でしか持ち出せない)。
//! - **寸法**: [`DIMENSION_MARKERS`] — `.size(`/`.padding(`/`.spacing(`/
//!   `Length::Fixed(`/`fill_rect(`/`stroke_rect(`/`stroke_v(`/`stroke_h(` の
//!   呼び出し引数、および `iced::Border { .. }`/`Rect { .. }` リテラルの
//!   `width:`/`radius:`/`w:`/`h:` フィールド。呼び出し・リテラルは複数行へ
//!   またがることがある([`bracket_region_end`] が丸/波括弧の深さで実際の
//!   終端行を追う — 固定行数の当てずっぽうにしない)。
//!
//! **`0`/`0.0`/`1`/`1.0`(符号つき含む)は対象外**([`is_exempt_literal`])。
//! 0 は「寸法の不在」(padding 無し・spacing 無し・角丸無し)であって独立した
//! 意匠値ではない。1.0 は `.max(1.0)` 系の**物理1px床**(除外リスト(1)の実例
//! — `inspector_pane.rs::value_cell_height`/`glyph_height`、`screenshot.rs::
//! stroke_v`/`stroke_h`)の値そのものが仕様であり、raw 値として個別に問い直す
//! 対象ではない。
//!
//! **同じ行に `dims.`/`colors.` があればその行は触らない**
//! ([`line_is_token_derived`]): `dims.border_width * 2.0`(`screenshot.rs`
//! のマーカー太さの倍率)のような「token 由来の値への係数」は発注書 KNOWN の
//! 「レイアウト計算の中間値(`*0.5`/`+1`等)まで縛ると誤検出だらけ」に該当する。
//!
//! **この境界の外**(マーカーの呼び出し引数・リテラルフィールドの**外側**にある
//! 独立した `let`/`const` 宣言)は意図的にこの柵の対象外として残す —
//! `screenshot.rs::CANVAS_WIDTH`(診断器具のキャンバス幅)・`settings_pane.rs::
//! CHECKERBOARD_TILE_PX`(市松タイル、hairline 床と同型の「装飾は物理px床」
//! opt-out だが裁定142の3種そのものではない)・`screenshot.rs::render` 内の
//! `let button_w = 72.0_f32.min(..)`(header ボタン幅の近似)・stage 高さの
//! 安全上限 `700.0` は、いずれも**実 UI の意匠値ではなく instrument/フォール
//! バック固有のジオメトリ**で、対応する token 概念が存在しない(NON-GOALS:
//! 新規 token ロール追加はしない)。判断に迷う項目として作業報告へ列挙する。
//!
//! ## 明示除外リスト(裁定142 の3種)
//! [`EXCLUSIONS`] にファイル名+識別子単位で列挙する。上記の走査設計自体が
//! 0/1 の一般免除と「呼び出し引数・リテラルフィールドだけを見る」限定に
//! よってこの3種の大半を自然に対象外にする(=能動的な抑制ロジックとして
//! 個々に発火するわけではない)ため、[`EXCLUSIONS`] は**この3種を意識的に
//! 分類した監査記録**として持つ。[`exclusion_identifiers_still_exist_in_their_named_file`]
//! が各エントリの識別子が実ソースに存在し続けることを機械的に見張り、表が
//! silently stale にならないようにする。

use std::path::{Path, PathBuf};

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// 発注書 EXACT TARGET が名指しした pane 系ファイル(すべて `src/` 直下)。
///
/// `timeline_pane.rs` は第2波第1切片(純粋なファイル分割)で `src/timeline/`
/// (`mod.rs`/`projection.rs`/`hit.rs`/`canvas.rs`/`input.rs`)へ分かれ、
/// 第2波T1(裁定147)で `lane_bar.rs` が加わった —
/// 柵は緩めず、分割後の6ファイルへ対象を追随させる(色・寸法の直書きが
/// 実際に発生し得るのは主に `canvas.rs`/`lane_bar.rs` だが、将来の混入も
/// 拾えるよう分割後の全ファイルを対象にする)。
const SCANNED_FILES: &[&str] = &[
    "inspector_pane.rs",
    "timeline/mod.rs",
    "timeline/projection.rs",
    "timeline/hit.rs",
    "timeline/canvas.rs",
    "timeline/input.rs",
    "timeline/lane_bar.rs",
    "settings_pane.rs",
    "lib.rs",
    "screenshot.rs",
];

/// `#[cfg(test)]\nmod tests {` の手前までを返す(inline test module は対象外)。
/// 見つからなければ全文を返す(`timeline_pane.rs`/`lib.rs`/`screenshot.rs` は
/// inline test module を持たない)。
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

/// `dims.`/`colors.` 参照が同じ行にあれば「token 由来の式」とみなす。
///
/// **`_px` 接尾辞も同格に扱う**: `screenshot.rs::stroke_v`/`stroke_h`/
/// `stroke_rect` の `width_px` パラメータがその実例 — 呼び出し元では常に
/// `dims.border_width`(倍率つきのことも)を渡す(`stroke_v` 冒頭の doc
/// コメント「`width_px` は罫線幅 token」参照)。関数境界を1つまたぐだけで
/// `dims.`/`colors.` という文字列そのものは局所スコープから見えなくなるが、
/// 値の出自は変わらない — `x - width_px / 2.0`(線を中心に置く `/2.0`)は
/// 発注書 KNOWN 4「レイアウト計算の中間値(`*0.5`/`+1`等)まで縛ると誤検出
/// だらけ」に該当する、`width_px` という既に token 由来の値の上の計算。
fn line_is_token_derived(line: &str) -> bool {
    line.contains("dims.") || line.contains("colors.") || line.contains("_px")
}

/// 「寸法の不在」(0)・「物理1px床」(1.0、`.max(1.0)` 系)の値そのものは
/// 個別に問い直さない(モジュール doc 参照)。
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
/// コメントは行ごとに [`strip_comment`] で落としてから数える。閉じが最後まで
/// 見つからなければ最終行を返す(壊れた入力への防波堤 — この柵はソースが
/// 構文的に正しいことを前提にしてよい、cargo が先に落ちる)。
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
/// 積む。`field_filter` が `Some` なら、そのマーカーを含む行だけを見る
/// (構造体リテラルの特定フィールドだけを問う場合)。
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
/// 呼べるよう、`text` を引数で受け取る — [`fence_self_tests`] が合成文字列で
/// この関数を直接検分する)。
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

/// **本命**: 発注書 EXACT TARGET の5ファイルに raw 色値・px 直書きが無いこと。
/// 見つかったら具体的な行を列挙して落ちる(発注書 EXACT TARGET 1)。
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
// 明示除外リスト(裁定142 の3種) — ファイル名+識別子単位、理由コメント必須。
// ---------------------------------------------------------------------------

struct Exclusion {
    file: &'static str,
    identifier: &'static str,
    reason: &'static str,
}

const EXCLUSIONS: &[Exclusion] = &[
    // --- (1) 物理1px hairline 床 ---------------------------------------
    Exclusion {
        file: "inspector_pane.rs",
        identifier: "fn value_cell_height",
        reason: "(dims.inspector_row_height - dims.spacing_s).max(1.0) — 物理1px床(裁定142除外1)。\
                 1.0はゼロ/負幅セルを防ぐ最小可視化の床であって独立した意匠値ではない \
                 (この柵は0/1リテラルを一般免除しているので実際には走査へ引っかからないが、\
                 除外(1)の実例としてここに記録する)。",
    },
    Exclusion {
        file: "inspector_pane.rs",
        identifier: "fn glyph_height",
        reason: "(dims.inspector_row_height - dims.spacing_xs).max(1.0) — 同上(Key/M/S glyph 列の床)。",
    },
    Exclusion {
        file: "screenshot.rs",
        identifier: "fn stroke_v",
        reason: "width_px.max(1.0) / (y1 - y0).max(1.0) — hairline 床と同型の最小1px保証。",
    },
    Exclusion {
        file: "screenshot.rs",
        identifier: "fn stroke_h",
        reason: "(x1 - x0).max(1.0) / width_px.max(1.0) — 同上。",
    },
    // --- (2) データ由来の色 ---------------------------------------------
    Exclusion {
        file: "settings_pane.rs",
        identifier: "pub fn preset_rgba",
        reason: "Composition.background(Stage 背景プリセット)の実値。UI chrome ではなく\
                 ユーザ作品(書き出しに乗る)の内容 — token化すると「テーマ変更で作品の色が\
                 変わる」逆事故になる(裁定142除外2)。",
    },
    // --- (3) 製品意味の定数 -----------------------------------------------
    Exclusion {
        file: "settings_pane.rs",
        identifier: "BackgroundPreset::Gray18 => [channel(46), channel(46), channel(46), 1.0]",
        reason: "「18%グレー」という写真用語をそのまま8bit値(255*0.18≈46)へ当てた値そのものが\
                 仕様(裁定142除外3)。散在させず preset_rgba 1箇所にしか定義しない。",
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
// 免除すべきものを実際に免除することを確認する(実ファイルへの赤→緑の
// 実地検分は作業報告に別記)。
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
        // `struct Rect { x: f32, y: f32, w: f32, h: f32 }` の型定義 — `f32` の
        // 数字は識別子の一部であって裸のリテラルではない。
        let synthetic = "struct Rect {\n    x: f32,\n    y: f32,\n    w: f32,\n    h: f32,\n}\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }

    #[test]
    fn domain_math_outside_any_marker_is_never_touched() {
        // opacity %換算・drag感度表・RGBA→u8変換等、widget構築呼び出しの外に
        // ある raw 数値はこの柵の対象外(モジュール doc の境界線どおり)。
        let synthetic = "opacity.value *= 100.0;\nlet step = 0.5;\nlet byte = (c * 255.0).round();\n";
        assert!(scan_text("synthetic.rs", synthetic).is_empty());
    }
}
