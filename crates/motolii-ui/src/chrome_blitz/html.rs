//! `ui/motolii-rn/src/chrome.tsx` と `ui/motolii-rn/src/panels/` の JSX の**写し**。
//!
//! 要素の並び・入れ子・`style={styles.X}` を、そのまま `<div class="X">` の入れ子へ移す。
//! RNの `View` → `div`、`Text` → `span`、`Pressable` → `div`(押せないので箱としてだけ)、
//! `TextInput` → `span`(値の表示だけ)。**意味は運ばない。**
//! 入力も dispatch も状態(`useRef` / `useState`)もここには無い。
//!
//! ## CSSに直書きした値の出所
//!
//! 色・寸法・間隔は**すべて** `theme.rs`(= `productStyles.ts` と各ローカル `StyleSheet` の写し)
//! から出る。それ以外にこのファイルが持つCSSは、RNの箱モデルの既定値をCSSへ言い直したものだけで、
//! 向こうに書かれていない「デザイン値」は1つも無い。
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
//! | `.parts-row` の `flex-direction:row;flex-grow:1;flex-shrink:1;flex-basis:0px` | — | `parts_html` 専用の**確認用の並べ台**。`Splitter` の `vertical` は幅8だけを持ち(`productStyles.ts:65`)高さは親の交差軸から来るので、席に伸びる行が要る。製品のレイアウトではない |
//! | `html,body` / `.shell` の `width`/`height` | 引数 | 席の寸法。`productStyles.ts:4` の `flex:1` に相当する place |
//!
//! `body` には背景色を置かない。置くとviewport全面が不透明になり Stage の上へ重ねられない
//! (`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。下地は `.shell` 自身が持つ
//! (`productStyles.ts:4` の `backgroundColor`)。
//!
//! ## パネルのローカル `StyleSheet` は名前空間で囲む
//!
//! `AssetTaggingPanel.tsx:39-56` と `registry.tsx:20-24` はそれぞれ自前の `StyleSheet` を持ち、
//! **両方が `root` という名前を持つ**。RNでは別モジュールなので衝突しないが、CSSのclass名は
//! 文書で1つの空間なので衝突する。そこで CSS は `.asset-tags .root { … }` /
//! `.export-notes .root { … }` と囲み、HTML側は `<div class="asset-tags"><div class="root">…`
//! とする。名前空間の文字列は `registry.tsx:16-17` の `id` をそのまま使う(新しい名前は作らない)。
//!
//! ## RN と CSS の差で気をつけたところ
//!
//! - **`flexDirection` の既定が違う。** RN は `column`、CSS は `row`。
//!   `div` の既定を `column` に倒し、原文が `flexDirection: 'row'` と書いているところ
//!   (`chromeModalHeader` `chromeModalActions` `panelHeader` `parameterRow`、
//!   `AssetTaggingPanel.tsx:53` の `tags`)だけが `flex-direction:row` を持つ。
//! - **RN の `StyleSheet` にカスケード/継承は無いが CSS にはある。** `color` と `font-size` は
//!   親から子へ漏れる。原文の `Text` はすべて自前の style を持つので実害は無いが、
//!   親の `div` 側には `color`/`font-size` を一切置かないことで漏れ元を作らない。

use super::sample::ExportSample;
use super::theme::{ASSET_TAGS_STYLES, EXPORT_NOTES_STYLES, STYLES};

/// `chrome.tsx:79-107` の `ChromeModal` の中へ `chrome.tsx:109-157` の `ExportScreen` を入れた1枚。
/// `sample` は固定値で、入力も編集意味もここには無い。
pub fn export_html(sample: &ExportSample, width: f64, height: f64) -> String {
    document(
        css(width, height, false),
        shell(
            width,
            height,
            // chrome.tsx:76 ChromeModalKind = 'export'。ChromeModal の title に渡る名前。
            &chrome_modal("Export", &export_screen(sample)),
        ),
    )
}

/// `chrome.tsx:79-107` の `ChromeModal` の中へ `chrome.tsx:159-172` の `SettingsScreen` を入れた1枚。
pub fn settings_html(width: f64, height: f64) -> String {
    document(
        css(width, height, false),
        shell(
            width,
            height,
            // chrome.tsx:76 ChromeModalKind = 'settings'。
            &chrome_modal("Settings", &settings_screen()),
        ),
    )
}

/// `registry.tsx:15-18` の `panelRegistry` の2パネルを縦に並べた1枚。
/// 選択(どちらを出すか)は意味なので写さない。**両方を同時に並べて見る**。
pub fn panels_html(width: f64, height: f64) -> String {
    let mut inner = String::new();
    // registry.tsx:16  {id: 'asset-tags', title: 'Tags', Component: AssetTaggingPanel}
    inner.push_str(&asset_tagging_panel());
    // registry.tsx:17  {id: 'export-notes', title: 'Notes', Component: ExportNotesPanel}
    inner.push_str(&export_notes_panel());
    document(css(width, height, true), shell(width, height, &inner))
}

/// `chrome.tsx:26-65` の `Splitter`(`vertical` と `horizontal` の両方)と
/// `chrome.tsx:67-74` の `PanelHeader` を、見るためだけに並べた1枚。
///
/// **これは製品のレイアウトではなく確認用の並びである。** 製品では `Splitter` は
/// 隣り合う席の境目に、`PanelHeader` は各パネルの先頭に置かれる。ここでは
/// 「その3つが Blitz で正しい寸法・色で出るか」だけを見たいので、席の中へ順に置いてある。
pub fn parts_html(width: f64, height: f64) -> String {
    let mut inner = String::new();
    // chrome.tsx:67-74 PanelHeader。title/detail は呼び出し側の値なので
    // 実在の呼び出し(Inspector.tsx:135-138)の文字列をそのまま借りる。
    inner.push_str(&panel_header("Inspector", Some("sample.vism")));
    // chrome.tsx:62  orientation === 'horizontal' → styles.hSplitter
    inner.push_str(r#"<div class="hSplitter"></div>"#);
    // chrome.tsx:62  orientation === 'vertical' → styles.vSplitter
    // 幅しか持たない(productStyles.ts:65)ので、伸びる行の中へ置いて高さを与える。
    inner.push_str(r#"<div class="parts-row"><div class="vSplitter"></div></div>"#);
    document(css(width, height, false), shell(width, height, &inner))
}

fn document(css: String, body: String) -> String {
    format!("<html><head><style>{css}</style></head><body>{body}</body></html>")
}

/// `locals` が真のときだけ、パネルのローカル `StyleSheet` を名前空間つきで足す。
fn css(width: f64, height: f64, locals: bool) -> String {
    let mut out = format!(
        "
  html,body {{ margin:0; padding:0; width:{width}px; height:{height}px;
               font-family:sans-serif; }}
  div {{ display:flex; flex-direction:column; align-items:stretch;
         flex-shrink:0; position:relative; box-sizing:border-box; }}
  span {{ display:block; flex-shrink:0; position:relative; box-sizing:border-box; }}
  .parts-row {{ flex-direction:row; flex-grow:1; flex-shrink:1; flex-basis:0px; }}
"
    );
    // productStyles.ts の並びのまま出す。後勝ちの上書き関係を原文と同じに保つ。
    for style in STYLES {
        out.push_str(&format!("  .{} {{ {} }}\n", style.name, style.css));
    }
    if locals {
        // registry.tsx:16-17 の id を名前空間にする。`root` が両方にあるので必須。
        for style in ASSET_TAGS_STYLES {
            out.push_str(&format!(
                "  .asset-tags .{} {{ {} }}\n",
                style.name, style.css
            ));
        }
        for style in EXPORT_NOTES_STYLES {
            out.push_str(&format!(
                "  .export-notes .{} {{ {} }}\n",
                style.name, style.css
            ));
        }
    }
    out
}

/// 下地。`productStyles.ts:4` の `shell`。席の寸法だけを引数から受ける。
fn shell(width: f64, height: f64, inner: &str) -> String {
    format!(r#"<div class="shell" style="width:{width}px;height:{height}px">{inner}</div>"#)
}

/// `chrome.tsx:67-74` の `PanelHeader` の写し。
/// `detail` が無いときは `panelDetail` を出さない(`chrome.tsx:71` の三項)。
fn panel_header(title: &str, detail: Option<&str>) -> String {
    let mut out = String::from(r#"<div class="panelHeader">"#);
    // chrome.tsx:70
    out.push_str(&format!(
        r#"<span class="panelTitle">{}</span>"#,
        escape(title)
    ));
    // chrome.tsx:71
    if let Some(detail) = detail {
        out.push_str(&format!(
            r#"<span class="panelDetail">{}</span>"#,
            escape(detail)
        ));
    }
    out.push_str("</div>");
    out
}

/// `chrome.tsx:79-107` の `ChromeModal` の写し。
/// `children`(`chrome.tsx:103`)は `chromeModalCard` の直下、ヘッダの次に入る。
fn chrome_modal(title: &str, children: &str) -> String {
    let mut out = String::new();
    // chrome.tsx:91
    out.push_str(r#"<div class="chromeModalScrim">"#);
    // chrome.tsx:92
    out.push_str(r#"<div class="chromeModalCard">"#);
    // chrome.tsx:93
    out.push_str(r#"<div class="chromeModalHeader">"#);
    // chrome.tsx:94
    out.push_str(&format!(
        r#"<span class="panelTitle">{}</span>"#,
        escape(title)
    ));
    // chrome.tsx:95-101  Pressable(押せない箱として div)。文字は chrome.tsx:100 の "Close"。
    out.push_str(
        r#"<div class="titleAction"><span class="titleActionText">Close</span></div></div>"#,
    );
    out.push_str(children);
    out.push_str("</div></div>");
    out
}

/// `chrome.tsx:109-157` の `ExportScreen` の写し。
fn export_screen(sample: &ExportSample) -> String {
    // chrome.tsx:125  <View testID="export-screen">。style を持たないので素の div。
    let mut out = String::from("<div>");

    // chrome.tsx:126-137  Output 行。TextInput は値の表示だけの span に写す。
    out.push_str(r#"<div class="parameterRow">"#);
    // chrome.tsx:127
    out.push_str(r#"<span class="parameterLabel">Output</span>"#);
    // chrome.tsx:131,135  value が空のときだけ placeholder の文字が見える。
    let shown = if sample.output_path.is_empty() {
        "/path/to/output.mp4"
    } else {
        sample.output_path
    };
    out.push_str(&format!(
        r#"<span class="parameterValue">{}</span></div>"#,
        escape(shown)
    ));

    // chrome.tsx:138-141  Format 行。
    out.push_str(
        r#"<div class="parameterRow"><span class="parameterLabel">Format</span><span class="parameterValue">MP4 (H.264)</span></div>"#,
    );

    // chrome.tsx:142-151  chromeModalActions > Pressable。
    // chrome.tsx:147  style={[styles.titleAction, !canRun && styles.titleActionDisabled]}
    out.push_str(r#"<div class="chromeModalActions">"#);
    out.push_str(&format!(
        r#"<div class="titleAction{}"><span class="titleActionText">Export</span></div></div>"#,
        if sample.can_run() {
            ""
        } else {
            " titleActionDisabled"
        }
    ));

    // chrome.tsx:152-154
    out.push_str(&format!(
        r#"<span class="chromeModalStatus">{}</span>"#,
        escape(sample.status_text)
    ));

    out.push_str("</div>");
    out
}

/// `chrome.tsx:159-172` の `SettingsScreen` の写し。props も状態も無く、文言は原文の直書き。
fn settings_screen() -> String {
    let mut out = String::from("<div>");
    // chrome.tsx:162-165
    out.push_str(
        r#"<div class="parameterRow"><span class="parameterLabel">Color theme</span><span class="muted">Default</span></div>"#,
    );
    // chrome.tsx:166-169
    out.push_str(
        r#"<div class="parameterRow"><span class="parameterLabel">Keyboard</span><span class="muted">Ableton Live</span></div>"#,
    );
    out.push_str("</div>");
    out
}

/// `AssetTaggingPanel.tsx:8-35` の写し。名前空間は `registry.tsx:16` の `id`。
/// `useState` の初期値(`AssetTaggingPanel.tsx:5`)がそのまま出る値になる。
fn asset_tagging_panel() -> String {
    let mut out = String::from(r#"<div class="asset-tags">"#);
    // AssetTaggingPanel.tsx:9
    out.push_str(r#"<div class="root">"#);
    // AssetTaggingPanel.tsx:10
    out.push_str(r#"<span class="heading">Asset Tagging Extension</span>"#);
    // AssetTaggingPanel.tsx:11-13
    out.push_str(r#"<span class="caption">Registered from a separate TypeScript module</span>"#);
    // AssetTaggingPanel.tsx:14-27  TextInput。draft は '' なので placeholder の文字が見える。
    out.push_str(r#"<span class="input">Add tag and press Return</span>"#);
    // AssetTaggingPanel.tsx:28-34
    out.push_str(r#"<div class="tags">"#);
    // AssetTaggingPanel.tsx:5  useState(['Interview', 'B-roll'])
    for tag in ["Interview", "B-roll"] {
        out.push_str(&format!(
            r#"<div class="tag"><span class="tagText">{}</span></div>"#,
            escape(tag)
        ));
    }
    out.push_str("</div></div></div>");
    out
}

/// `registry.tsx:6-13` の `ExportNotesPanel` の写し。名前空間は `registry.tsx:17` の `id`。
fn export_notes_panel() -> String {
    let mut out = String::from(r#"<div class="export-notes">"#);
    // registry.tsx:8
    out.push_str(r#"<div class="root">"#);
    // registry.tsx:9
    out.push_str(r#"<span class="title">Export Notes Extension</span>"#);
    // registry.tsx:10
    out.push_str(r#"<span class="body">A second panel proves registry-driven selection.</span>"#);
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
    use super::super::sample::{ExportPhase, EXPORT_SAMPLE, EXPORT_SAMPLE_BUSY};
    use super::super::theme;
    use super::*;

    fn all_html() -> Vec<(&'static str, String)> {
        vec![
            ("export", export_html(&EXPORT_SAMPLE, 980.0, 650.0)),
            ("settings", settings_html(980.0, 650.0)),
            ("panels", panels_html(980.0, 650.0)),
            ("parts", parts_html(980.0, 650.0)),
        ]
    }

    /// 出力に現れる class の全部が、`productStyles.ts` 由来か、パネルのローカル
    /// `StyleSheet` 由来か、このファイルが明示している構造用の名前だけ。
    /// **新しい見た目の名前を作らない。**
    fn known(name: &str) -> bool {
        // 構造用。`parts-row` は確認用の並べ台、あとの2つは registry.tsx:16-17 の id。
        if name == "parts-row" || name == "asset-tags" || name == "export-notes" {
            return true;
        }
        theme::has(name)
            || theme::ASSET_TAGS_STYLES.iter().any(|s| s.name == name)
            || theme::EXPORT_NOTES_STYLES.iter().any(|s| s.name == name)
    }

    /// NEGATIVE ORACLE 1: 4枚とも、使っている class 名が全部どこかの表にある。
    #[test]
    fn uses_only_known_class_names() {
        for (label, html) in all_html() {
            for chunk in html.split(r#"class=""#).skip(1) {
                let value = chunk.split('"').next().expect("class 属性の終端");
                for name in value.split_whitespace() {
                    assert!(known(name), "{label}: {name} がどの表にも無い");
                }
            }
        }
    }

    /// NEGATIVE ORACLE 2: 16進色は1つも直書きしない。色は `theme.rs` 経由でしか出ない。
    #[test]
    fn declares_no_color_outside_the_style_tables() {
        let table: String = theme::STYLES
            .iter()
            .chain(theme::ASSET_TAGS_STYLES)
            .chain(theme::EXPORT_NOTES_STYLES)
            .map(|style| style.css)
            .collect();
        for (label, html) in all_html() {
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
                    "{label}: #{hex} が theme.rs の表に無い"
                );
            }
        }
    }

    /// `body` に背景色を置くとviewport全面が不透明になる
    /// (`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。下地は `.shell` が持つ。
    #[test]
    fn keeps_the_shell_background_off_of_body() {
        for (label, html) in all_html() {
            let head_end = html.find("div {").expect("div rule");
            assert!(
                !html[..head_end].contains("background"),
                "{label}: html/body に背景色が残っている"
            );
            assert!(html.contains(r#"class="shell""#), "{label}: 下地が無い");
        }
    }

    /// POSITIVE ORACLE: `chrome.tsx` の literal がそのまま出ている。
    #[test]
    fn emits_the_chrome_literals() {
        let export = export_html(&EXPORT_SAMPLE, 980.0, 650.0);
        for text in [
            "Export",
            "Close",
            "Output",
            "Format",
            "MP4 (H.264)",
            "Ready",
        ] {
            assert!(export.contains(text), "export: {text} が出ていない");
        }
        // chrome.tsx:131 の placeholder は value が空のときだけ見える。
        let empty = ExportSample {
            output_path: "",
            phase: ExportPhase::Idle,
            status_text: "Ready",
        };
        assert!(export_html(&empty, 980.0, 650.0).contains("/path/to/output.mp4"));

        let settings = settings_html(980.0, 650.0);
        for text in [
            "Settings",
            "Color theme",
            "Default",
            "Keyboard",
            "Ableton Live",
        ] {
            assert!(settings.contains(text), "settings: {text} が出ていない");
        }
    }

    /// `chrome.tsx:122-123` の `canRun`。`exporting` のときだけ `titleActionDisabled` が付く。
    #[test]
    fn disables_the_export_action_when_it_cannot_run() {
        // `titleActionDisabled` はCSSの表にも出るので、class属性の側で見る。
        let enabled = r#"<div class="titleAction"><span class="titleActionText">Export</span>"#;
        let disabled = r#"<div class="titleAction titleActionDisabled"><span class="titleActionText">Export</span>"#;
        let idle = export_html(&EXPORT_SAMPLE, 980.0, 650.0);
        assert!(idle.contains(enabled) && !idle.contains(disabled));
        let busy = export_html(&EXPORT_SAMPLE_BUSY, 980.0, 650.0);
        assert!(busy.contains(disabled));
    }

    /// `registry.tsx` と `AssetTaggingPanel.tsx` はどちらも `root` を持つ。
    /// 名前空間で囲んでいないとCSSが衝突して片方が消える。
    #[test]
    fn namespaces_the_two_panel_style_sheets() {
        let html = panels_html(980.0, 650.0);
        assert!(html.contains(".asset-tags .root {"));
        assert!(html.contains(".export-notes .root {"));
        assert!(html.contains(r#"<div class="asset-tags"><div class="root">"#));
        assert!(html.contains(r#"<div class="export-notes"><div class="root">"#));
        for text in [
            "Asset Tagging Extension",
            "Registered from a separate TypeScript module",
            "Add tag and press Return",
            "Interview",
            "B-roll",
            "Export Notes Extension",
            "A second panel proves registry-driven selection.",
        ] {
            assert!(html.contains(text), "panels: {text} が出ていない");
        }
    }

    /// `parts_html` は `Splitter` の両向きと `PanelHeader` を出す。
    #[test]
    fn emits_both_splitters_and_the_panel_header() {
        let html = parts_html(980.0, 650.0);
        for fragment in [
            r#"class="panelHeader""#,
            r#"class="panelTitle""#,
            r#"class="panelDetail""#,
            r#"class="hSplitter""#,
            r#"class="vSplitter""#,
        ] {
            assert!(html.contains(fragment), "parts: {fragment} が出ていない");
        }
    }

    /// RN の `View` 既定は `column`。CSS の `div` 既定は `block`(子は `row` に並ばない)ので、
    /// 明示していないと silent に崩れる。写し規則がCSSに出ていること。
    #[test]
    fn pins_the_react_native_flex_direction_default() {
        let html = panels_html(980.0, 650.0);
        assert!(html.contains("div { display:flex; flex-direction:column;"));
        // row を持つのは原文が flexDirection:'row' と書いたぶんだけ。
        let row_rules = theme::STYLES
            .iter()
            .filter(|style| style.css.contains("flex-direction:row"))
            .map(|style| style.name)
            .collect::<Vec<_>>();
        assert_eq!(
            row_rules,
            [
                "chromeModalHeader",
                "chromeModalActions",
                "panelHeader",
                "parameterRow"
            ]
        );
        let local_row_rules = theme::ASSET_TAGS_STYLES
            .iter()
            .filter(|style| style.css.contains("flex-direction:row"))
            .map(|style| style.name)
            .collect::<Vec<_>>();
        // AssetTaggingPanel.tsx:53 の tags だけ。
        assert_eq!(local_row_rules, ["tags"]);
    }
}
