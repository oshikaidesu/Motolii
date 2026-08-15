//! `ui/motolii-rn/src/productStyles.ts` と、Extension panel が持つローカル StyleSheet の**写し**。
//!
//! Chrome(titlebar / splitter / ChromeModal / Extension panel)が使う StyleSheet エントリだけを、
//! **原文の並びのまま**持つ。ここで値を決めない。色も寸法も間隔も原文にあるものだけ。
//!
//! 表は出所ごとに3つある:
//! - `STYLES` … `productStyles.ts` の共有表(`chrome.tsx` が `styles.X` で参照するもの)。
//! - `ASSET_TAGS_STYLES` … `panels/AssetTaggingPanel.tsx` のファイルローカルな `StyleSheet.create`。
//! - `EXPORT_NOTES_STYLES` … `panels/registry.tsx` のファイルローカルな `StyleSheet.create`。
//!
//! 各エントリは3つ組で持つ:
//! - `name` … `styles.<name>` の名前。そのままCSSのclass名になる(camelCaseはclass名として合法)。
//!   `chrome.tsx` の `style={styles.parameterRow}` と `class="parameterRow"` が1対1で読める。
//! - `rn` … 原文**そのまま**の右辺。
//! - `css` … `rn` をRNの箱モデルからCSSへ機械的に写したもの。**新しい値は入れない**。
//!
//! `tests::mirrors_product_styles` が `include_str!` した `productStyles.ts` の原文に対して
//! `"  {name}: {rn},"` の**逐語一致**を要求する。エントリのどのフィールドが向こうで変わっても落ちる。
//! ローカル表の側は原文に複数行の宣言(`AssetTaggingPanel.tsx` の `input`)があるので、
//! `mirrors_asset_tags_styles` / `mirrors_export_notes_styles` が
//! **連続する空白を1つに畳んでから**同じ逐語一致を要求する。
//! `tests::css_introduces_no_new_literals` が `css` 側のリテラル(16進色・数値)が
//! すべて `rn` 側にもあることを3つの表すべてに要求する。写し間違いはここで落ちる。
//!
//! ## RN → CSS の写し規則(値ではなく箱モデルの対応)
//!
//! | RN | CSS |
//! |---|---|
//! | `paddingHorizontal: N` | `padding-left:Npx;padding-right:Npx` |
//! | `paddingVertical: N` | `padding-top:Npx;padding-bottom:Npx` |
//! | `borderWidth: N, borderColor: C` | `border:Npx solid C` |
//! | `borderBottomWidth: N, borderBottomColor: C` | `border-bottom:Npx solid C` |
//! | `borderLeftWidth: N, borderRightWidth: N, borderColor: C` | `border-left:Npx solid C;border-right:Npx solid C` |
//! | `borderTopWidth: N, borderBottomWidth: N, borderColor: C` | `border-top:Npx solid C;border-bottom:Npx solid C` |
//! | `borderRadius: N` | `border-radius:Npx` |
//! | `flex: 1` | `flex-grow:1;flex-shrink:1;flex-basis:0px` |
//! | `flexWrap: 'wrap'` / `gap: N` | `flex-wrap:wrap` / `gap:Npx` |
//! | `justifyContent: 'space-between'` | `justify-content:space-between` |
//! | `marginLeft: 'auto'` | `margin-left:auto` |
//! | `minHeight: N` / `minWidth: N` | `min-height:Npx` / `min-width:Npx` |
//! | `maxWidth: '90%'` | `max-width:90%` (百分率はそのまま) |
//! | `inset: 0` | `top:0px;right:0;bottom:0;left:0` (`inset` は使わず4辺へ展開) |
//! | `zIndex: N` | `z-index:N` |
//! | `opacity: N` | `opacity:N` |
//! | `backgroundColor: C` | `background:C` |
//! | `fontWeight: 'N'` | `font-weight:N` |
//! | `letterSpacing: N` / `lineHeight: N` | `letter-spacing:Npx` / `line-height:Npx` |
//! | 数値の長さ | `px` を付ける (RNの長さ単位はDIP) |

pub struct RnStyle {
    /// `styles.<name>`。CSSのclass名も同じ。
    pub name: &'static str,
    /// 原文の右辺。
    pub rn: &'static str,
    /// 上をCSSへ写したもの。
    pub css: &'static str,
}

/// `productStyles.ts` の出現順。CSSもこの順で出すので、
/// `titleActionDisabled` が `titleAction` を後勝ちで上書きする関係が原文の並びのまま保たれる。
pub const STYLES: &[RnStyle] = &[
    // productStyles.ts:4
    RnStyle {
        name: "shell",
        rn: "{flex: 1, minWidth: 980, minHeight: 650, backgroundColor: '#111315'}",
        css: "flex-grow:1;flex-shrink:1;flex-basis:0px;min-width:980px;min-height:650px;background:#111315;",
    },
    // productStyles.ts:5
    RnStyle {
        name: "chromeModalScrim",
        rn: "{position: 'absolute', inset: 0, backgroundColor: 'rgba(0,0,0,0.45)', alignItems: 'center', justifyContent: 'center', zIndex: 20}",
        css: "position:absolute;top:0px;right:0;bottom:0;left:0;background:rgba(0,0,0,0.45);align-items:center;justify-content:center;z-index:20;",
    },
    // productStyles.ts:6
    RnStyle {
        name: "chromeModalCard",
        rn: "{width: 420, maxWidth: '90%', backgroundColor: '#1b1d20', borderWidth: 1, borderColor: '#3a3d41'}",
        css: "width:420px;max-width:90%;background:#1b1d20;border:1px solid #3a3d41;",
    },
    // productStyles.ts:7
    RnStyle {
        name: "chromeModalHeader",
        rn: "{height: 31, flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', paddingHorizontal: 9, borderBottomWidth: 1, borderBottomColor: '#3a3d41'}",
        css: "height:31px;flex-direction:row;align-items:center;justify-content:space-between;padding-left:9px;padding-right:9px;border-bottom:1px solid #3a3d41;",
    },
    // productStyles.ts:8
    RnStyle {
        name: "chromeModalActions",
        rn: "{flexDirection: 'row', justifyContent: 'flex-end', paddingHorizontal: 8, paddingVertical: 10}",
        css: "flex-direction:row;justify-content:flex-end;padding-left:8px;padding-right:8px;padding-top:10px;padding-bottom:10px;",
    },
    // productStyles.ts:9
    RnStyle {
        name: "chromeModalStatus",
        rn: "{paddingHorizontal: 10, paddingBottom: 10, fontSize: 9, color: '#858a8d'}",
        css: "padding-left:10px;padding-right:10px;padding-bottom:10px;font-size:9px;color:#858a8d;",
    },
    // productStyles.ts:15
    RnStyle {
        name: "titleAction",
        rn: "{fontSize: 10, color: '#d8d8d5', paddingVertical: 6, paddingHorizontal: 10, marginLeft: 6, borderWidth: 1, borderColor: '#414448'}",
        css: "font-size:10px;color:#d8d8d5;padding-top:6px;padding-bottom:6px;padding-left:10px;padding-right:10px;margin-left:6px;border:1px solid #414448;",
    },
    // productStyles.ts:16
    RnStyle {
        name: "titleActionDisabled",
        rn: "{opacity: 0.4}",
        css: "opacity:0.4;",
    },
    // productStyles.ts:17
    RnStyle {
        name: "titleActionText",
        rn: "{fontSize: 10, color: '#d8d8d5'}",
        css: "font-size:10px;color:#d8d8d5;",
    },
    // productStyles.ts:28
    RnStyle {
        name: "panelHeader",
        rn: "{height: 31, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 9, borderBottomWidth: 1, borderBottomColor: '#3a3d41'}",
        css: "height:31px;flex-direction:row;align-items:center;padding-left:9px;padding-right:9px;border-bottom:1px solid #3a3d41;",
    },
    // productStyles.ts:29
    RnStyle {
        name: "panelTitle",
        rn: "{fontSize: 11, fontWeight: '700', color: '#dedfdd'}",
        css: "font-size:11px;font-weight:700;color:#dedfdd;",
    },
    // productStyles.ts:30
    RnStyle {
        name: "panelDetail",
        rn: "{marginLeft: 'auto', fontSize: 8, color: '#85898c'}",
        css: "margin-left:auto;font-size:8px;color:#85898c;",
    },
    // productStyles.ts:64
    RnStyle {
        name: "muted",
        rn: "{fontSize: 9, color: '#858a8d'}",
        css: "font-size:9px;color:#858a8d;",
    },
    // productStyles.ts:65
    RnStyle {
        name: "vSplitter",
        rn: "{width: 8, backgroundColor: '#272a2d', borderLeftWidth: 1, borderRightWidth: 1, borderColor: '#3e4246'}",
        css: "width:8px;background:#272a2d;border-left:1px solid #3e4246;border-right:1px solid #3e4246;",
    },
    // productStyles.ts:66
    RnStyle {
        name: "hSplitter",
        rn: "{height: 8, backgroundColor: '#272a2d', borderTopWidth: 1, borderBottomWidth: 1, borderColor: '#3e4246'}",
        css: "height:8px;background:#272a2d;border-top:1px solid #3e4246;border-bottom:1px solid #3e4246;",
    },
    // productStyles.ts:91
    RnStyle {
        name: "parameterRow",
        rn: "{minHeight: 32, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 8, borderBottomWidth: 1, borderBottomColor: '#33363a'}",
        css: "min-height:32px;flex-direction:row;align-items:center;padding-left:8px;padding-right:8px;border-bottom:1px solid #33363a;",
    },
    // productStyles.ts:92
    RnStyle {
        name: "parameterLabel",
        rn: "{width: 88, fontSize: 11, color: '#d5d6d3'}",
        css: "width:88px;font-size:11px;color:#d5d6d3;",
    },
    // productStyles.ts:93
    RnStyle {
        name: "parameterValue",
        rn: "{flex: 1, paddingHorizontal: 7, paddingVertical: 4, fontSize: 11, fontWeight: '600', textAlign: 'right', color: '#f2f2f0', borderWidth: 1, borderColor: '#575c61'}",
        css: "flex-grow:1;flex-shrink:1;flex-basis:0px;padding-left:7px;padding-right:7px;padding-top:4px;padding-bottom:4px;font-size:11px;font-weight:600;text-align:right;color:#f2f2f0;border:1px solid #575c61;",
    },
];

/// `AssetTaggingPanel.tsx` のファイルローカルな `StyleSheet.create` の出現順。
/// 共有表(`productStyles.ts`)には無い、この panel だけの表であることに注意。
pub const ASSET_TAGS_STYLES: &[RnStyle] = &[
    // AssetTaggingPanel.tsx:40
    RnStyle {
        name: "root",
        rn: "{padding: 12}",
        css: "padding:12px;",
    },
    // AssetTaggingPanel.tsx:41
    RnStyle {
        name: "heading",
        rn: "{fontSize: 14, fontWeight: '600', color: '#f0f2f6'}",
        css: "font-size:14px;font-weight:600;color:#f0f2f6;",
    },
    // AssetTaggingPanel.tsx:42
    RnStyle {
        name: "caption",
        rn: "{fontSize: 10, marginTop: 3, color: '#8e95a2'}",
        css: "font-size:10px;margin-top:3px;color:#8e95a2;",
    },
    // AssetTaggingPanel.tsx:43-52 (原文は複数行。空白を畳んだ形で持つ)
    RnStyle {
        name: "input",
        rn: "{ height: 32, marginTop: 14, paddingHorizontal: 8, color: '#f4f5f7', backgroundColor: '#15171c', borderWidth: 1, borderColor: '#484e59', borderRadius: 4, }",
        css: "height:32px;margin-top:14px;padding-left:8px;padding-right:8px;color:#f4f5f7;background:#15171c;border:1px solid #484e59;border-radius:4px;",
    },
    // AssetTaggingPanel.tsx:53
    RnStyle {
        name: "tags",
        rn: "{flexDirection: 'row', flexWrap: 'wrap', gap: 6, marginTop: 10}",
        css: "flex-direction:row;flex-wrap:wrap;gap:6px;margin-top:10px;",
    },
    // AssetTaggingPanel.tsx:54
    RnStyle {
        name: "tag",
        rn: "{paddingVertical: 4, paddingHorizontal: 8, backgroundColor: '#343b49', borderRadius: 10}",
        css: "padding-top:4px;padding-bottom:4px;padding-left:8px;padding-right:8px;background:#343b49;border-radius:10px;",
    },
    // AssetTaggingPanel.tsx:55
    RnStyle {
        name: "tagText",
        rn: "{fontSize: 10, color: '#dce0e7'}",
        css: "font-size:10px;color:#dce0e7;",
    },
];

/// `registry.tsx` の `ExportNotesPanel` 用のファイルローカルな `StyleSheet.create` の出現順。
pub const EXPORT_NOTES_STYLES: &[RnStyle] = &[
    // registry.tsx:21
    RnStyle {
        name: "root",
        rn: "{padding: 12}",
        css: "padding:12px;",
    },
    // registry.tsx:22
    RnStyle {
        name: "title",
        rn: "{fontSize: 14, fontWeight: '600', color: '#f0f2f6'}",
        css: "font-size:14px;font-weight:600;color:#f0f2f6;",
    },
    // registry.tsx:23
    RnStyle {
        name: "body",
        rn: "{fontSize: 11, marginTop: 8, color: '#9da3af'}",
        css: "font-size:11px;margin-top:8px;color:#9da3af;",
    },
];

/// `STYLES` から `name` を引く。`html.rs` は class 名しか書かないので、
/// 使っている class が表に無いことを test 側で落とすために使う。
pub fn has(name: &str) -> bool {
    STYLES.iter().any(|style| style.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 意味の出所である原文と突き合わせる。
    /// 写し間違いと、向こうが変わってこちらが取り残された事故の両方をここで落とす。
    const PRODUCT_STYLES: &str = include_str!("../../../../ui/motolii-rn/src/productStyles.ts");
    const ASSET_TAGGING_PANEL: &str =
        include_str!("../../../../ui/motolii-rn/src/panels/AssetTaggingPanel.tsx");
    const PANEL_REGISTRY: &str = include_str!("../../../../ui/motolii-rn/src/panels/registry.tsx");

    /// 連続する空白文字を1つのスペースに畳む。
    /// ローカル表には複数行で書かれた宣言(`AssetTaggingPanel.tsx` の `input`)があるので、
    /// 逐語一致の前に改行とインデントの差だけを消す。
    fn squeeze(source: &str) -> String {
        source.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// POSITIVE ORACLE: 各エントリが `productStyles.ts` に**逐語で**存在する。
    #[test]
    fn mirrors_product_styles() {
        for style in STYLES {
            let expected = format!("  {}: {},", style.name, style.rn);
            assert!(
                PRODUCT_STYLES.contains(&expected),
                "productStyles.ts に `{expected}` が無い"
            );
        }
    }

    /// POSITIVE ORACLE: 空白を畳んだ上で `AssetTaggingPanel.tsx` に逐語で存在する。
    #[test]
    fn mirrors_asset_tags_styles() {
        let source = squeeze(ASSET_TAGGING_PANEL);
        for style in ASSET_TAGS_STYLES {
            let expected = squeeze(&format!("{}: {},", style.name, style.rn));
            assert!(
                source.contains(&expected),
                "AssetTaggingPanel.tsx に `{expected}` が無い"
            );
        }
    }

    /// POSITIVE ORACLE: 空白を畳んだ上で `registry.tsx` に逐語で存在する。
    #[test]
    fn mirrors_export_notes_styles() {
        let source = squeeze(PANEL_REGISTRY);
        for style in EXPORT_NOTES_STYLES {
            let expected = squeeze(&format!("{}: {},", style.name, style.rn));
            assert!(
                source.contains(&expected),
                "registry.tsx に `{expected}` が無い"
            );
        }
    }

    /// 16進色を取り除いてから数値だけを拾う。`#3a3d41` の `3` を数値と誤認しないため。
    fn literals(source: &str) -> (Vec<String>, Vec<String>) {
        let chars: Vec<char> = source.chars().collect();
        let mut colors = Vec::new();
        let mut numbers = Vec::new();
        let mut index = 0;
        while index < chars.len() {
            if chars[index] == '#' {
                let mut end = index + 1;
                while end < chars.len() && chars[end].is_ascii_hexdigit() {
                    end += 1;
                }
                colors.push(chars[index..end].iter().collect::<String>());
                index = end;
                continue;
            }
            if chars[index].is_ascii_digit() {
                let mut end = index;
                while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
                    end += 1;
                }
                numbers.push(chars[index..end].iter().collect::<String>());
                index = end;
                continue;
            }
            index += 1;
        }
        (colors, numbers)
    }

    /// NEGATIVE ORACLE: CSS側に原文由来でないリテラルが1つも無い。
    /// `0` だけは箱モデルの写しで出る構造値(`flex-basis:0px`、`inset` の展開など)として許す。
    #[test]
    fn css_introduces_no_new_literals() {
        for table in [STYLES, ASSET_TAGS_STYLES, EXPORT_NOTES_STYLES] {
            for style in table {
                let (rn_colors, rn_numbers) = literals(style.rn);
                let (css_colors, css_numbers) = literals(style.css);
                for color in css_colors {
                    assert!(
                        rn_colors.contains(&color),
                        "{}: CSSの色 {color} が原文側に無い",
                        style.name
                    );
                }
                for number in css_numbers {
                    assert!(
                        number == "0" || rn_numbers.contains(&number),
                        "{}: CSSの数値 {number} が原文側に無い",
                        style.name
                    );
                }
            }
        }
    }

    /// 表の並びが原文の並びと同じであること。
    /// 後勝ちで効く上書き(`titleAction` → `titleActionDisabled` など)が原文と同じ順で出るのを保証する。
    #[test]
    fn keeps_the_source_order() {
        for (table, source, origin) in [
            (STYLES, PRODUCT_STYLES, "productStyles.ts"),
            (
                ASSET_TAGS_STYLES,
                ASSET_TAGGING_PANEL,
                "AssetTaggingPanel.tsx",
            ),
            (EXPORT_NOTES_STYLES, PANEL_REGISTRY, "registry.tsx"),
        ] {
            let mut last = 0usize;
            for style in table {
                let at = source
                    .find(&format!("  {}: ", style.name))
                    .unwrap_or_else(|| panic!("{origin} に {} が無い", style.name));
                assert!(
                    at > last,
                    "{} が {origin} の並びより前に出ている",
                    style.name
                );
                last = at;
            }
        }
    }
}
