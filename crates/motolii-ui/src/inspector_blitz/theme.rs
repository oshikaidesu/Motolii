//! `ui/motolii-rn/src/productStyles.ts` の**写し**。
//!
//! Inspector が使う StyleSheet エントリだけを、**原文の並びのまま**持つ。
//! ここで値を決めない。色も寸法も間隔も `productStyles.ts` にあるものだけ。
//!
//! 各エントリは3つ組で持つ:
//! - `name` … `styles.<name>` の名前。そのままCSSのclass名になる(camelCaseはclass名として合法)。
//!   `Inspector.tsx` の `style={styles.parameterRow}` と `class="parameterRow"` が1対1で読める。
//! - `rn` … `productStyles.ts` の**原文そのまま**の右辺。
//! - `css` … `rn` をRNの箱モデルからCSSへ機械的に写したもの。**新しい値は入れない**。
//!
//! `tests::mirrors_product_styles` が `include_str!` した `productStyles.ts` の原文に対して
//! `"  {name}: {rn},"` の**逐語一致**を要求する。エントリのどのフィールドが向こうで変わっても落ちる。
//! `tests::css_introduces_no_new_literals` が `css` 側のリテラル(16進色・数値)が
//! すべて `rn` 側にもあることを要求する。写し間違いはここで落ちる。
//!
//! ## RN → CSS の写し規則(値ではなく箱モデルの対応)
//!
//! | RN | CSS |
//! |---|---|
//! | `paddingHorizontal: N` | `padding-left:Npx;padding-right:Npx` |
//! | `paddingVertical: N` | `padding-top:Npx;padding-bottom:Npx` |
//! | `borderWidth: N, borderColor: C` | `border:Npx solid C` |
//! | `borderBottomWidth: N, borderBottomColor: C` | `border-bottom:Npx solid C` |
//! | `borderTopWidth: N, borderColor: C` | `border-top:Npx solid C` (RNのborderColorは全辺だが幅が上だけ) |
//! | `flex: 1` | `flex-grow:1;flex-shrink:1;flex-basis:0px` |
//! | `backgroundColor: C` | `background:C` |
//! | `fontWeight: 'N'` | `font-weight:N` |
//! | `letterSpacing: N` / `lineHeight: N` | `letter-spacing:Npx` / `line-height:Npx` |
//! | 数値の長さ | `px` を付ける (RNの長さ単位はDIP) |

pub struct RnStyle {
    /// `styles.<name>`。CSSのclass名も同じ。
    pub name: &'static str,
    /// `productStyles.ts` の原文の右辺。
    pub rn: &'static str,
    /// 上をCSSへ写したもの。
    pub css: &'static str,
}

/// `productStyles.ts` の出現順。CSSもこの順で出すので、
/// `tabActive` が `tab` を、`keyButtonCurrent` が `keyButton` を後勝ちで上書きする関係が
/// 原文の並びのまま保たれる。
pub const STYLES: &[RnStyle] = &[
    // productStyles.ts:27
    RnStyle {
        name: "inspector",
        rn: "{backgroundColor: '#1b1d20'}",
        css: "background:#1b1d20;",
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
    // productStyles.ts:31
    RnStyle {
        name: "tabRow",
        rn: "{height: 31, flexDirection: 'row', borderBottomWidth: 1, borderBottomColor: '#3a3d41'}",
        css: "height:31px;flex-direction:row;border-bottom:1px solid #3a3d41;",
    },
    // productStyles.ts:32
    RnStyle {
        name: "tab",
        rn: "{flex: 1, alignItems: 'center', justifyContent: 'center'}",
        css: "flex-grow:1;flex-shrink:1;flex-basis:0px;align-items:center;justify-content:center;",
    },
    // productStyles.ts:33
    RnStyle {
        name: "tabActive",
        rn: "{borderBottomWidth: 2, borderBottomColor: '#b4a66a', backgroundColor: '#191b1e'}",
        css: "border-bottom:2px solid #b4a66a;background:#191b1e;",
    },
    // productStyles.ts:34
    RnStyle {
        name: "tabText",
        rn: "{fontSize: 9, color: '#c6c7c5'}",
        css: "font-size:9px;color:#c6c7c5;",
    },
    // productStyles.ts:90
    RnStyle {
        name: "inspectorTitle",
        rn: "{paddingHorizontal: 10, paddingTop: 8, fontSize: 13, fontWeight: '700', color: '#f0f0ed'}",
        css: "padding-left:10px;padding-right:10px;padding-top:8px;font-size:13px;font-weight:700;color:#f0f0ed;",
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
    // productStyles.ts:94
    RnStyle {
        name: "stepButton",
        rn: "{width: 25, height: 24, alignItems: 'center', justifyContent: 'center'}",
        css: "width:25px;height:24px;align-items:center;justify-content:center;",
    },
    // productStyles.ts:95
    RnStyle {
        name: "stepText",
        rn: "{fontSize: 11, color: '#c8bc7b'}",
        css: "font-size:11px;color:#c8bc7b;",
    },
    // productStyles.ts:96
    RnStyle {
        name: "keyButton",
        rn: "{width: 25, height: 24, alignItems: 'center', justifyContent: 'center', borderWidth: 1, borderColor: 'transparent'}",
        css: "width:25px;height:24px;align-items:center;justify-content:center;border:1px solid transparent;",
    },
    // productStyles.ts:97
    RnStyle {
        name: "keyButtonAnimated",
        rn: "{borderColor: '#45494d'}",
        css: "border-color:#45494d;",
    },
    // productStyles.ts:98
    RnStyle {
        name: "keyButtonCurrent",
        rn: "{borderColor: '#c8bc7b', backgroundColor: '#343228'}",
        css: "border-color:#c8bc7b;background:#343228;",
    },
    // productStyles.ts:99
    RnStyle {
        name: "keyGlyph",
        rn: "{fontSize: 11, color: '#b8bab8'}",
        css: "font-size:11px;color:#b8bab8;",
    },
    // productStyles.ts:100
    RnStyle {
        name: "keyGlyphUnkeyed",
        rn: "{color: '#777c80'}",
        css: "color:#777c80;",
    },
    // productStyles.ts:101
    RnStyle {
        name: "dial",
        rn: "{flex: 1, height: 24, position: 'relative', overflow: 'hidden', borderWidth: 1, borderColor: '#45494d', backgroundColor: '#111315'}",
        css: "flex-grow:1;flex-shrink:1;flex-basis:0px;height:24px;position:relative;overflow:hidden;border:1px solid #45494d;background:#111315;",
    },
    // productStyles.ts:103
    RnStyle {
        name: "dialTicks",
        rn: "{position: 'absolute', left: 4, right: 44, top: 3, bottom: 3, overflow: 'hidden'}",
        css: "position:absolute;left:4px;right:44px;top:3px;bottom:3px;overflow:hidden;",
    },
    // productStyles.ts:104
    RnStyle {
        name: "dialTickStrip",
        rn: "{position: 'absolute', left: '50%', width: 810, height: '100%', marginLeft: -405, flexDirection: 'row', alignItems: 'flex-end'}",
        css: "position:absolute;left:50%;width:810px;height:100%;margin-left:-405px;flex-direction:row;align-items:flex-end;",
    },
    // productStyles.ts:105
    RnStyle {
        name: "dialTickCell",
        rn: "{width: 10, height: '100%', alignItems: 'center', justifyContent: 'flex-end'}",
        css: "width:10px;height:100%;align-items:center;justify-content:flex-end;",
    },
    // productStyles.ts:106
    RnStyle {
        name: "dialTick",
        rn: "{width: 1, height: 7, backgroundColor: '#686d72'}",
        css: "width:1px;height:7px;background:#686d72;",
    },
    // productStyles.ts:107
    RnStyle {
        name: "dialTickMajor",
        rn: "{height: 13, backgroundColor: '#c8bc7b'}",
        css: "height:13px;background:#c8bc7b;",
    },
    // productStyles.ts:108
    RnStyle {
        name: "dialPointer",
        rn: "{position: 'absolute', top: 0, bottom: 0, left: '50%', width: 1, backgroundColor: '#f0f0ed'}",
        css: "position:absolute;top:0px;bottom:0px;left:50%;width:1px;background:#f0f0ed;",
    },
    // productStyles.ts:109
    RnStyle {
        name: "dialValue",
        rn: "{position: 'absolute', right: 4, top: 0, minWidth: 35, height: 22, paddingHorizontal: 2, paddingVertical: 0, fontSize: 10, textAlign: 'center', color: '#e7e7e4', backgroundColor: '#111315', zIndex: 1}",
        css: "position:absolute;right:4px;top:0px;min-width:35px;height:22px;padding-left:2px;padding-right:2px;padding-top:0px;padding-bottom:0px;font-size:10px;text-align:center;color:#e7e7e4;background:#111315;z-index:1;",
    },
    // productStyles.ts:110
    RnStyle {
        name: "dialValueWithUnit",
        rn: "{right: 13}",
        css: "right:13px;",
    },
    // productStyles.ts:111
    RnStyle {
        name: "dialUnit",
        rn: "{position: 'absolute', right: 4, top: 5, fontSize: 8, color: '#c8bc7b', zIndex: 2}",
        css: "position:absolute;right:4px;top:5px;font-size:8px;color:#c8bc7b;z-index:2;",
    },
    // productStyles.ts:112
    RnStyle {
        name: "pathOperationSection",
        rn: "{borderTopWidth: 1, borderColor: '#383c40', paddingVertical: 4}",
        css: "border-top:1px solid #383c40;padding-top:4px;padding-bottom:4px;",
    },
    // productStyles.ts:113
    RnStyle {
        name: "transformSection",
        rn: "{borderTopWidth: 1, borderColor: '#33454c', paddingVertical: 4}",
        css: "border-top:1px solid #33454c;padding-top:4px;padding-bottom:4px;",
    },
    // productStyles.ts:114
    RnStyle {
        name: "pathOperationTitle",
        rn: "{paddingHorizontal: 10, paddingTop: 4, fontSize: 11, fontWeight: '700', color: '#f2f2f0'}",
        css: "padding-left:10px;padding-right:10px;padding-top:4px;font-size:11px;font-weight:700;color:#f2f2f0;",
    },
    // productStyles.ts:115
    RnStyle {
        name: "pathOperationDescription",
        rn: "{paddingHorizontal: 10, paddingTop: 2, fontSize: 11, lineHeight: 15, color: '#d5d6d3'}",
        css: "padding-left:10px;padding-right:10px;padding-top:2px;font-size:11px;line-height:15px;color:#d5d6d3;",
    },
    // productStyles.ts:116
    RnStyle {
        name: "pathOperationGrid",
        rn: "{flexDirection: 'row', flexWrap: 'wrap', gap: 5, paddingHorizontal: 10, paddingVertical: 6}",
        css: "flex-direction:row;flex-wrap:wrap;gap:5px;padding-left:10px;padding-right:10px;padding-top:6px;padding-bottom:6px;",
    },
    // productStyles.ts:117
    RnStyle {
        name: "pathOperationButton",
        rn: "{borderWidth: 1, borderColor: '#45494d', paddingHorizontal: 6, paddingVertical: 5}",
        css: "border:1px solid #45494d;padding-left:6px;padding-right:6px;padding-top:5px;padding-bottom:5px;",
    },
    // productStyles.ts:118
    RnStyle {
        name: "pathOperationButtonText",
        rn: "{fontSize: 10, color: '#e7e7e4'}",
        css: "font-size:10px;color:#e7e7e4;",
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

    /// 意味の出所である `productStyles.ts` の原文と突き合わせる。
    /// 写し間違いと、向こうが変わってこちらが取り残された事故の両方をここで落とす。
    const PRODUCT_STYLES: &str = include_str!("../../../../ui/motolii-rn/src/productStyles.ts");

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

    /// NEGATIVE ORACLE: CSS側に `productStyles.ts` 由来でないリテラルが1つも無い。
    /// `0` だけは箱モデルの写しで出る構造値(`flex-basis:0px` など)として許す。
    #[test]
    fn css_introduces_no_new_literals() {
        for style in STYLES {
            let (rn_colors, rn_numbers) = literals(style.rn);
            let (css_colors, css_numbers) = literals(style.css);
            for color in css_colors {
                assert!(
                    rn_colors.contains(&color),
                    "{}: CSSの色 {color} が productStyles.ts 側に無い",
                    style.name
                );
            }
            for number in css_numbers {
                assert!(
                    number == "0" || rn_numbers.contains(&number),
                    "{}: CSSの数値 {number} が productStyles.ts 側に無い",
                    style.name
                );
            }
        }
    }

    /// 表の並びが `productStyles.ts` の並びと同じであること。
    /// 後勝ちで効く上書き(`tab` → `tabActive` など)が原文と同じ順で出るのを保証する。
    #[test]
    fn keeps_the_source_order() {
        let mut last = 0usize;
        for style in STYLES {
            let at = PRODUCT_STYLES
                .find(&format!("  {}: ", style.name))
                .unwrap_or_else(|| panic!("productStyles.ts に {} が無い", style.name));
            assert!(
                at > last,
                "{} が productStyles.ts の並びより前に出ている",
                style.name
            );
            last = at;
        }
    }
}
