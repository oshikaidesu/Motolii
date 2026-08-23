//! `Tokens`(`Dimensions` + `Colors` + `ui_scale` の起動時の姿)の読み込みと
//! `ui_scale` の書き戻し(`replace_ui_scale` 等)。`lib.rs` から分割
//! (SP-8、中身は移送のみ)。

use std::path::Path;

use crate::{Colors, Dimensions};

/// 全 pane が読む、この起動時点でのデザイン値の姿。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub dims: Dimensions,
    pub colors: Colors,
    /// mock `--s` 相当の UI 拡大率。**正本は `dims.ui_scale`**(JSON トークン
    /// ファイル経由でホットリロードされる) — ここへは [`Tokens::load`]/
    /// [`Default`] がその値をそのまま写す(発注書が指定した置き場 `Tokens.ui_scale`
    /// を公開しつつ、実体は1つに保つ)。
    pub ui_scale: f32,
}

impl Default for Tokens {
    fn default() -> Self {
        let dims = Dimensions::default();
        Self {
            ui_scale: dims.ui_scale,
            dims,
            colors: Colors::default(),
        }
    }
}

// release ビルドは正本 JSON をコンパイル時に埋め込む。**file I/O ゼロ**。
#[cfg(not(debug_assertions))]
const DIMENSIONS_JSON: &str = include_str!("../tokens/dimensions.json");
#[cfg(not(debug_assertions))]
const COLOR_TOKENS_JSON: &str =
    include_str!("../../../../ui/motolii-tokens/sources/motolii-dark.json");

impl Tokens {
    /// 起動時の読み込み。debug はファイルから、release は埋め込み文字列から。
    pub fn load() -> Self {
        #[cfg(debug_assertions)]
        {
            let dims =
                Dimensions::load_from_path(&Dimensions::debug_source_path()).unwrap_or_default();
            let colors = Colors::load_from_path(&Colors::debug_source_path()).unwrap_or_default();
            Self {
                ui_scale: dims.ui_scale,
                dims,
                colors,
            }
        }
        #[cfg(not(debug_assertions))]
        {
            let dims = Dimensions::parse(DIMENSIONS_JSON).unwrap_or_default();
            let colors = Colors::parse(COLOR_TOKENS_JSON).unwrap_or_default();
            Self {
                ui_scale: dims.ui_scale,
                dims,
                colors,
            }
        }
    }
}

/// [`Dimensions::ui_scale`] だけを書き戻す surgical replace(テキストレベル)。
/// **`serde_json::to_string` で丸ごと書き直さない** — 正本 JSON は `_note_*` キー
/// (コメント代わり、`Dimensions` 構造体に対応フィールドが無い)を持つので、struct
/// 経由の再シリアライズはそれを消してしまう。`"ui_scale"` キーの値部分だけを
/// テキストとして置換し、それ以外の1バイトも変えない。
pub fn replace_ui_scale(json: &str, ui_scale: f32) -> Result<String, String> {
    let key = "\"ui_scale\"";
    let key_pos = json
        .find(key)
        .ok_or_else(|| "ui_scale キーが無い".to_owned())?;
    let after_key = &json[key_pos + key.len()..];
    let colon_offset = after_key
        .find(':')
        .ok_or_else(|| "ui_scale キーの直後に : が無い".to_owned())?;
    let value_start = key_pos + key.len() + colon_offset + 1;
    let rest = &json[value_start..];
    let end_offset = rest
        .find(|c: char| c == ',' || c == '}')
        .ok_or_else(|| "ui_scale の値の終端(, か })が見つからない".to_owned())?;

    let mut result = String::with_capacity(json.len() + 8);
    result.push_str(&json[..value_start]);
    result.push_str(&format!(" {ui_scale:.2}"));
    result.push_str(&json[value_start + end_offset..]);
    Ok(result)
}

/// [`replace_ui_scale`] を実ファイルへ適用する(read-modify-write)。**path 引数を
/// 取る** — `tokens/dimensions.json` は複数 worktree・並列試験間で共有される
/// delicate なファイル(`../reference/KNOWN.md`「レーン運用」)なので、試験は
/// `motolii_testkit::tmp_dir()` の隔離コピーでこの関数を叩く(実ファイルは
/// [`save_ui_scale`] からしか触らない)。
pub fn write_ui_scale_to_path(path: &Path, ui_scale: f32) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let replaced = replace_ui_scale(&text, ui_scale)?;
    std::fs::write(path, replaced).map_err(|error| error.to_string())
}

/// `ui_scale` の実行時の書き戻し口。**debug ビルドだけが実際に正本 JSON へ触る**
/// (`watch_subscription` と同じ判断) — release は `include_str!` で埋め込み済み
/// なので、書いても次回起動には反映されない(file I/O 自体をしない)。
///
/// **この関数自体は自動試験の対象にしない**: `tokens/dimensions.json` は複数
/// worktree・並列試験間で共有されるファイルで、ここを試験で書き換えると他の
/// 試験(このファイルを読む `tests/drive.rs` 等)とレースする。書き戻しの実質
/// (テキスト置換の正しさ)は [`replace_ui_scale`]/[`write_ui_scale_to_path`] が
/// 隔離された文字列・一時ファイルで検分済み — ここは経路を1行つなぐだけ。
#[cfg(debug_assertions)]
pub fn save_ui_scale(ui_scale: f32) -> Result<(), String> {
    write_ui_scale_to_path(&Dimensions::debug_source_path(), ui_scale)
}

#[cfg(not(debug_assertions))]
pub fn save_ui_scale(_ui_scale: f32) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod ui_scale_writeback_tests {
    use super::replace_ui_scale;

    const SAMPLE: &str = r#"{
  "row_height": 20,
  "_note_row_height": "note",

  "ui_scale": 1.00,
  "_note_ui_scale": "仮の置き場うんぬん、カンマも波括弧も含まない",

  "border_width": 1.0
}"#;

    /// **本命**: 値だけが変わり、直後の `_note_ui_scale`(コメント代わりのキー、
    /// `Dimensions` 構造体には存在しない)は1バイトも変わらない。
    #[test]
    fn replacing_ui_scale_changes_only_that_value() {
        let replaced = replace_ui_scale(SAMPLE, 1.5).expect("置換できない");
        assert!(
            replaced.contains("\"ui_scale\": 1.50"),
            "新しい値が入っていない: {replaced}"
        );
        assert!(
            replaced.contains("_note_ui_scale"),
            "note キーが消えている: {replaced}"
        );
        assert!(
            replaced.contains("\"row_height\": 20"),
            "無関係なキーまで変わっている: {replaced}"
        );
        assert!(
            replaced.contains("\"border_width\": 1.0"),
            "ui_scale より後ろのキーが壊れている: {replaced}"
        );
    }

    /// 最後のキー(直後が `}`)でも壊れない — `,` 前提の実装だと落ちる境界。
    #[test]
    fn replacing_the_last_key_before_the_closing_brace_still_works() {
        let json = r#"{"a": 1, "ui_scale": 1.0}"#;
        let replaced = replace_ui_scale(json, 2.0).expect("置換できない");
        assert_eq!(replaced, r#"{"a": 1, "ui_scale": 2.00}"#);
    }

    #[test]
    fn missing_key_is_a_clear_error_not_a_panic() {
        assert!(replace_ui_scale("{}", 1.0).is_err());
    }

    /// 書き戻した文字列は `Dimensions::parse` でそのまま読み直せて、他フィールドは
    /// 元の `Dimensions::default()` と一致する(構造としても壊れていない)。
    #[test]
    fn the_rewritten_text_round_trips_through_dimensions_parse() {
        use super::Dimensions;
        let full = r#"{"row_height": 20, "transport_band": 30, "title_text": 12,
            "body_text": 11, "caption_text": 9, "micro_text": 8,
            "spacing_xs": 2, "spacing_s": 4, "spacing_m": 8, "spacing_l": 12,
            "border_width": 1.0, "panel_header_height": 29,
            "inspector_panel_width": 496, "inspector_row_height": 20,
            "inspector_section_header_height": 26, "inspector_value_width": 38,
            "inspector_glyph_width": 18, "ui_scale": 1.0}"#;
        let replaced = replace_ui_scale(full, 1.75).expect("置換できない");
        let dims = Dimensions::parse(&replaced).expect("書き戻した JSON を読めない");
        assert_eq!(dims.ui_scale, 1.75);
        assert_eq!(dims.row_height, 20.0, "無関係なフィールドが壊れている");
    }
}

