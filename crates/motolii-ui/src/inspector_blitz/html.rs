//! `ui/motolii-rn/src/Inspector.tsx` の JSX の**写し**。
//!
//! `Inspector.tsx` の要素の並び・入れ子・`style={styles.X}` を、そのまま
//! `<div class="X">` の入れ子へ移す。RNの `View` → `div`、`Text` → `span`、
//! `Pressable` → `div`(押せないので箱としてだけ)、`TextInput` → `span`(値の表示だけ)。
//! **意味は運ばない。**入力も dispatch もここには無い。
//!
//! ## CSSに直書きした値の出所
//!
//! 色・寸法・間隔は**すべて** `theme.rs`(= `productStyles.ts` の写し)から出る。
//! それ以外にこのファイルが持つCSSは、RNの箱モデルの既定値をCSSへ言い直したものだけで、
//! `productStyles.ts` にも `Inspector.tsx` にも書かれていない「デザイン値」は1つも無い。
//!
//! | CSS | 値 | 出所 |
//! |---|---|---|
//! | `div` の `display:flex;flex-direction:column` | — | RN `View` の既定。**CSSのdivは `row` ではなく `block`** なので明示が要る |
//! | `div` の `align-items:stretch` | — | RN `View` の既定 |
//! | `div`/`span` の `flex-shrink:0` | — | RN の既定。CSSの既定は `1` |
//! | `div`/`span` の `position:relative` | — | RN の既定。CSSの既定は `static` |
//! | `div`/`span` の `box-sizing:border-box` | — | RN は borderWidth が箱の内側 |
//! | `span` の `display:block` | — | RN `Text` を葉として置くため |
//! | `html,body` の `margin:0;padding:0` | — | ブラウザ既定の打ち消し |
//! | `html,body` の `font-family:sans-serif` | — | `timeline_blitz/html.rs:79` と同じ扱い |
//! | `.scroll` の `flex-grow:1;flex-shrink:1;flex-basis:0px;overflow:hidden` | — | RN `ScrollView`(`Inspector.tsx:147`)の既定 |
//! | `html,body` / `.inspector` の `width`/`height` | viewport | 席の寸法。`Inspector.tsx:134` の `{width}` に相当 |
//!
//! `body` には背景色を置かない。置くとviewport全面が不透明になり Stage の上へ重ねられない
//! (`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。下地は `.inspector` 自身が持つ
//! (`productStyles.ts:27` の `backgroundColor`)。
//!
//! ## RN と CSS の差で気をつけたところ
//!
//! - **`flexDirection` の既定が違う。** RN は `column`、CSS は `row`。
//!   この写しでは `div` を flex container にして既定を `column` に倒し、
//!   `productStyles.ts` が `flexDirection: 'row'` と
//!   書いているところ(`panelHeader` `tabRow` `parameterRow` `pathOperationGrid`
//!   `dialTickStrip`)だけが `flex-direction:row` を持つ。
//!   これは RN View の写し規則であり、Blitz panel 全体を Flex に限定する規則ではない。
//!   Grid/Flex の採否と固定crate版dumpによる確認は
//!   `docs/reviews/2026-08-16-blitz-html-css-authoring-and-validation-decision.md` を正とする。
//! - **RN の `StyleSheet` にカスケード/継承は無いが CSS にはある。** `color` と `font-size` は
//!   親から子へ漏れる。`Inspector.tsx` の `Text` はすべて自前の style を持つので実害は無いが、
//!   親の `div` 側には `color`/`font-size` を一切置かない(= `productStyles.ts` に書いてある
//!   ぶんだけが出る)ことで漏れ元を作らない。

use super::sample::{DialSample, EffectSample, InspectorSample, KeyState, ParamSample};
use super::theme::STYLES;

/// `Inspector.tsx` の INSPECTOR パネル(既定タブ)を1枚のHTML文書にする。
/// 入力も編集意味もここには無い。`sample` は固定値。
pub fn inspector_html(sample: &InspectorSample) -> String {
    format!(
        "<html><head><style>{}</style></head><body>{}</body></html>",
        css(),
        body(sample)
    )
}

fn css() -> String {
    let mut out = String::from(
        "
  html,body { margin:0; padding:0; width:100%; height:100%;
               font-family:sans-serif; }
  div { display:flex; flex-direction:column; align-items:stretch;
        flex-shrink:0; position:relative; box-sizing:border-box; }
  span { display:block; flex-shrink:0; position:relative; box-sizing:border-box; }
  .scroll { flex-grow:1; flex-shrink:1; flex-basis:0px; overflow:hidden; }
",
    );
    // productStyles.ts の並びのまま出す。後勝ちの上書き関係を原文と同じに保つ。
    for style in STYLES {
        out.push_str(&format!("  .{} {{ {} }}\n", style.name, style.css));
    }
    out
}

fn body(sample: &InspectorSample) -> String {
    let mut b = String::new();
    // Inspector.tsx:134。席の幅高はDocument viewportから受ける。
    b.push_str(r#"<div class="inspector" style="width:100%;height:100%">"#);

    // Inspector.tsx:135-138 → chrome.tsx:67-74 PanelHeader
    b.push_str(&format!(
        r#"<div class="panelHeader"><span class="panelTitle">Inspector</span><span class="panelDetail">{}</span></div>"#,
        escape(sample.detail)
    ));

    // Inspector.tsx:139-145  tabRow。panel === 'INSPECTOR' が既定(Inspector.tsx:70)なので
    // 左のタブに tabActive が付く。ラベルは Inspector.tsx:142 の写し。
    b.push_str(
        r#"<div class="tabRow">
             <div class="tab tabActive"><span class="tabText">Effect</span></div>
             <div class="tab"><span class="tabText">Custom</span></div>
           </div>"#,
    );

    // Inspector.tsx:147  <ScrollView disableScrollViewPanResponder>
    b.push_str(r#"<div class="scroll">"#);

    // ---- Inspector.tsx:150-278  layer section ----
    b.push_str(r#"<div class="pathOperationSection">"#);
    b.push_str(&format!(
        r#"<span class="inspectorTitle">{}</span>"#,
        escape(sample.display_name)
    ));
    b.push_str(&format!(
        r#"<span class="pathOperationDescription">position keys: {}</span>"#,
        sample.position_key_count
    ));
    // Inspector.tsx:157-197 の <ParameterRow label="Opacity" .../>
    b.push_str(&parameter_row(
        "Opacity",
        sample.opacity,
        Some(sample.opacity_key),
        true,
    ));
    // Inspector.tsx:219-277  pathOperationGrid
    b.push_str(r#"<div class="pathOperationGrid">"#);
    for action in sample.clip_actions {
        b.push_str(&format!(
            r#"<div class="pathOperationButton"><span class="pathOperationButtonText">{}</span></div>"#,
            escape(action)
        ));
    }
    b.push_str("</div></div>");

    // ---- Inspector.tsx:279-300  source section ----
    if !sample.source_params.is_empty() {
        b.push_str(
            r#"<div class="pathOperationSection"><span class="pathOperationTitle">Source</span>"#,
        );
        for param in sample.source_params {
            b.push_str(&source_param_row(param));
        }
        b.push_str("</div>");
    }

    // ---- Inspector.tsx:301-329  effects section ----
    if !sample.effects.is_empty() {
        b.push_str(
            r#"<div class="pathOperationSection"><span class="pathOperationTitle">Effects</span>"#,
        );
        for effect in sample.effects {
            b.push_str(&effect_block(effect));
        }
        b.push_str("</div>");
    }

    // ---- Inspector.tsx:330-592  transform section ----
    b.push_str(
        r#"<div class="transformSection"><span class="pathOperationTitle">Transform</span>"#,
    );
    for dial in sample.dials {
        b.push_str(&dial_parameter(dial));
    }
    b.push_str("</div>");

    b.push_str("</div></div>");
    b
}

/// `Inspector.tsx:620-635` の `ParameterRow` の写し。
/// `−`(U+2212) / `＋`(U+FF0B) は Inspector.tsx:631,633 のリテラル。
fn parameter_row(label: &str, value: &str, key: Option<KeyState>, steppers: bool) -> String {
    let mut row = String::from(r#"<div class="parameterRow">"#);
    row.push_str(&format!(
        r#"<span class="parameterLabel">{}</span>"#,
        escape(label)
    ));
    if let Some(state) = key {
        row.push_str(&key_affordance(state));
    }
    if steppers {
        row.push_str(r#"<div class="stepButton"><span class="stepText">−</span></div>"#);
    }
    row.push_str(&format!(
        r#"<span class="parameterValue">{}</span>"#,
        escape(value)
    ));
    if steppers {
        row.push_str(r#"<div class="stepButton"><span class="stepText">＋</span></div>"#);
    }
    row.push_str("</div>");
    row
}

/// color を持つ param は `ColorRgbaEditor`(`Inspector.tsx:708-748`)、
/// f64 の param は `SourceParamEditor`(`Inspector.tsx:871-903`) /
/// `EffectParamEditor`(`Inspector.tsx:958-991`)。どちらの枝も
/// `parameterRow` > `parameterLabel` + `parameterValue` の行でできている。
fn source_param_row(param: &ParamSample) -> String {
    let Some(channels) = param.color else {
        return parameter_row(param.label, param.value, None, false);
    };
    // Inspector.tsx:709-747: ラベル行のあとに R/G/B/A の4行。
    let mut out = format!(
        r#"<div><span class="pathOperationDescription">{}</span>"#,
        escape(param.label)
    );
    // Inspector.tsx:638 COLOR_CHANNELS / 713 channel.toUpperCase()
    for (channel, value) in ["R", "G", "B", "A"].iter().zip(channels) {
        out.push_str(&parameter_row(channel, value, None, false));
    }
    out.push_str("</div>");
    out
}

/// `Inspector.tsx:304-327`。effect ごとに名前の行があり、その下に param 行が並ぶ。
fn effect_block(effect: &EffectSample) -> String {
    let mut out = format!(
        r#"<div><span class="pathOperationDescription">{}</span>"#,
        escape(effect.name)
    );
    for param in effect.params {
        out.push_str(&source_param_row(param));
    }
    out.push_str("</div>");
    out
}

/// `Inspector.tsx:48-66` の `KeyAffordance` の写し。
/// グリフ `◇`/`◆` は Inspector.tsx:63。
fn key_affordance(state: KeyState) -> String {
    let (button, glyph, mark) = match state {
        KeyState::Unkeyed => ("keyButton", "keyGlyph keyGlyphUnkeyed", "◇"),
        KeyState::Animated => ("keyButton keyButtonAnimated", "keyGlyph", "◆"),
        KeyState::Current => ("keyButton keyButtonCurrent", "keyGlyph", "◆"),
    };
    format!(r#"<div class="{button}"><span class="{glyph}">{mark}</span></div>"#)
}

/// `Inspector.tsx:1517-1593` の `DialParameter` の写し。
fn dial_parameter(dial: &DialSample) -> String {
    let mut out = String::from(r#"<div class="parameterRow">"#);
    out.push_str(&format!(
        r#"<span class="parameterLabel">{}</span>"#,
        escape(dial.label)
    ));
    if let Some(state) = dial.key {
        out.push_str(&key_affordance(state));
    }
    // Inspector.tsx:1528 DialSurface(= Pressable) style={styles.dial}
    out.push_str(r#"<div class="dial"><div class="dialTicks"><div class="dialTickStrip">"#);
    // Inspector.tsx:1568-1572  Array.from({length: 81}) / index % 5 === 0 が major
    for index in 0..81 {
        out.push_str(&format!(
            r#"<div class="dialTickCell"><div class="dialTick{}"></div></div>"#,
            if index % 5 == 0 { " dialTickMajor" } else { "" }
        ));
    }
    // Inspector.tsx:1574  dialPointer は dialTickStrip の兄弟(dialTicks の中)
    out.push_str(r#"</div><div class="dialPointer"></div></div>"#);
    // Inspector.tsx:1576-1589  TextInput style={[styles.dialValue, unit ? dialValueWithUnit : null]}
    out.push_str(&format!(
        r#"<span class="dialValue{}">{}</span>"#,
        if dial.unit.is_empty() {
            ""
        } else {
            " dialValueWithUnit"
        },
        escape(dial.value)
    ));
    // Inspector.tsx:1590
    if !dial.unit.is_empty() {
        out.push_str(&format!(
            r#"<span class="dialUnit">{}</span>"#,
            escape(dial.unit)
        ));
    }
    out.push_str("</div></div>");
    out
}

/// 表示文字列はサンプル由来でもDOMを壊せる。表示としてだけ通す。
/// `timeline_blitz/html.rs:264-276` の写し。
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::sample::SAMPLE;
    use super::super::theme;
    use super::*;

    fn sample_html() -> String {
        inspector_html(&SAMPLE)
    }

    /// POSITIVE ORACLE: `Inspector.tsx` の各セクションが同じ構造で出ている。
    #[test]
    fn emits_every_inspector_section() {
        let html = sample_html();
        for fragment in [
            // Inspector.tsx:135 PanelHeader
            r#"class="panelHeader""#,
            // Inspector.tsx:139 tabRow / 141 tabActive
            r#"class="tabRow""#,
            r#"class="tab tabActive""#,
            // Inspector.tsx:150 layer section
            r#"class="pathOperationSection""#,
            r#"class="inspectorTitle""#,
            // Inspector.tsx:219 clip 操作
            r#"class="pathOperationGrid""#,
            r#"class="pathOperationButton""#,
            // Inspector.tsx:330 transform
            r#"class="transformSection""#,
            r#"class="dial""#,
            r#"class="dialTickStrip""#,
            r#"class="dialPointer""#,
        ] {
            assert!(html.contains(fragment), "{fragment} が出ていない");
        }
        for text in [
            "Inspector",
            "Effect",
            "Custom",
            "position keys: 3",
            "Opacity",
            "Delete",
            "Duplicate",
            "Split",
            "Mute",
            "Solo",
            "Source",
            "Effects",
            "Transform",
            "Position X",
            "Position Y",
            "Rotation Z",
            "Scale X",
            "Scale Y",
        ] {
            assert!(html.contains(text), "{text} が出ていない");
        }
    }

    /// key の3状態が3つとも出る(`Inspector.tsx:56-64`)。
    #[test]
    fn emits_all_three_key_states() {
        let html = sample_html();
        assert!(html.contains(r#"class="keyButton keyButtonCurrent""#));
        assert!(html.contains(r#"class="keyButton keyButtonAnimated""#));
        assert!(html.contains(r#"class="keyGlyph keyGlyphUnkeyed""#));
        assert!(html.contains('◇') && html.contains('◆'));
    }

    /// `Inspector.tsx:1568` の 81 本。5本ごとが major(`Inspector.tsx:1570`)。
    #[test]
    fn emits_eighty_one_ticks_per_dial() {
        let dial = dial_parameter(&SAMPLE.dials[0]);
        assert_eq!(dial.matches(r#"class="dialTickCell""#).count(), 81);
        assert_eq!(dial.matches("dialTickMajor").count(), 17);
    }

    /// NEGATIVE ORACLE 1: 出力に現れる class は全部 `productStyles.ts` 由来か、
    /// このファイルが明示している構造用の `scroll` だけ。新しい見た目の名前を作らない。
    #[test]
    fn uses_only_product_style_class_names() {
        let html = sample_html();
        for chunk in html.split(r#"class=""#).skip(1) {
            let value = chunk.split('"').next().expect("class 属性の終端");
            for name in value.split_whitespace() {
                assert!(
                    name == "scroll" || theme::has(name),
                    "{name} が productStyles.ts に無い"
                );
            }
        }
    }

    /// NEGATIVE ORACLE 2: 16進色は1つも直書きしない。色は `theme.rs` 経由でしか出ない。
    #[test]
    fn declares_no_color_outside_the_style_table() {
        let html = sample_html();
        let table: String = theme::STYLES.iter().map(|style| style.css).collect();
        for chunk in html.split('#').skip(1) {
            let hex: String = chunk
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            if hex.len() != 6 && hex.len() != 3 {
                continue;
            }
            assert!(
                table.contains(&format!("#{hex}")),
                "#{hex} が theme.rs の表に無い"
            );
        }
    }

    /// `body` に背景色を置くとviewport全面が不透明になる
    /// (`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。下地は `.inspector` が持つ。
    #[test]
    fn keeps_the_panel_background_off_of_body() {
        let html = sample_html();
        let head_end = html.find("div {").expect("div rule");
        assert!(
            !html[..head_end].contains("background"),
            "html/body に背景色が残っている"
        );
        assert!(html.contains(r#"class="inspector""#));
    }

    /// RN の `View` 既定は `column`。CSS の `div` 既定は `block`(子は `row` に並ばない)ので、
    /// 明示していないと silent に崩れる。写し規則がCSSに出ていること。
    #[test]
    fn pins_the_react_native_flex_direction_default() {
        let html = sample_html();
        assert!(html.contains("div { display:flex; flex-direction:column;"));
        // row を持つのは productStyles.ts が flexDirection:'row' と書いた5つだけ。
        let row_rules = theme::STYLES
            .iter()
            .filter(|style| style.css.contains("flex-direction:row"))
            .map(|style| style.name)
            .collect::<Vec<_>>();
        assert_eq!(
            row_rules,
            [
                "panelHeader",
                "tabRow",
                "parameterRow",
                "dialTickStrip",
                "pathOperationGrid"
            ]
        );
    }
}
