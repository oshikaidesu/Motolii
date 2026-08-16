//! `ui/motolii-rn/src/Browser.tsx` の MEDIA / GRID をそのまま HTML の木へ写す。
//!
//! `View` → `div`、`Text` → `span`、`TextInput` / `Pressable` → 見た目だけの `div`。
//! input、tab切替、選択、drag、Document intent はこの文書に持たない。メディアの
//! 走査・path解決は既存 `media_library`、縮小画像は `thumbnail.rs` が owner のまま。

use std::path::PathBuf;

use super::library_view::BrowserItem;
use super::theme;

fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
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

/// DOM id は表示の identity だけ。Browser hit-test の入口にはしない。
pub(super) const CARD_ID_PREFIX: &str = "c";

fn directory_label(items: &[BrowserItem]) -> String {
    items
        .first()
        .and_then(|item| item.path.parent())
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Media".to_owned())
}

/// Browser.tsx:378-427 の immutable MEDIA / GRID snapshot。
pub(super) fn build_html(
    _title: &str,
    items: &[BrowserItem],
    thumbs: &[Option<PathBuf>],
) -> String {
    let mut body = String::from(r#"<div class="browser">"#);

    // Browser.tsx:378-379 → chrome.tsx:67-74 PanelHeader。
    body.push_str(
        r#"<div class="panelHeader"><span class="panelTitle">Browser</span><span class="panelDetail">MEDIA / CREATE / EFFECTS</span></div>"#,
    );
    // Browser.tsx:380-392。表示中の immutable snapshot は MEDIA。
    body.push_str(
        r#"<div class="tabRow"><div class="tab tabActive"><span class="tabText">Media</span></div><div class="tab"><span class="tabText">Effects</span></div><div class="tab"><span class="tabText">Create</span></div></div>"#,
    );
    // Browser.tsx:393-424。Blitz の既定フォントに無い icon は T/G/L fallback で表す。
    body.push_str(
        r#"<div class="searchRow"><div class="search"><span>Search media</span></div><div class="iconButton"><span class="iconText">T</span></div><div class="iconButton iconButtonActive"><span class="iconText">G</span></div><div class="iconButton"><span class="iconText">L</span></div></div>"#,
    );

    // BrowserResults.tsx:186-266。
    body.push_str(r#"<div class="discoveryBody"><div class="sourceRail"><span class="railItem effectSelected">All media</span><span class="railHeading">DIRECTORIES</span>"#);
    body.push_str(&format!(
        r#"<span class="railItem">{}</span>"#,
        escape(&directory_label(items))
    ));
    body.push_str(&format!(
        r#"<span class="railHeading">TYPE</span><span class="railItem">Image</span></div><div class="results"><div class="resultsHeader"><span class="resultTitle">Results</span><span class="panelDetail">{}</span></div>"#,
        items.len()
    ));

    if items.is_empty() {
        body.push_str(
            r#"<div class="emptyPanel"><span class="muted">画像が見つかりません。</span></div>"#,
        );
    } else {
        body.push_str(r#"<div class="resultGrid">"#);
        for (index, item) in items.iter().enumerate() {
            let image = match thumbs.get(index).and_then(|thumb| thumb.as_ref()) {
                Some(thumb) => format!(
                    r#"<img class="thumb" style="background:{}" src="file://{}" />"#,
                    theme::MEDIA_COLORS[index % theme::MEDIA_COLORS.len()],
                    escape(&thumb.to_string_lossy()),
                ),
                None => format!(
                    r#"<div class="thumb" style="background:{}"></div>"#,
                    theme::MEDIA_COLORS[index % theme::MEDIA_COLORS.len()],
                ),
            };
            // BrowserResults.tsx:215-251。image は既存縮小実体、色面/name/detail は RN source。
            body.push_str(&format!(
                r#"<div id="{CARD_ID_PREFIX}{index}" class="browserCard"><div class="browserThumb">{image}</div><span class="effectName">{}</span><span class="effectTags">{}</span></div>"#,
                escape(&item.name),
                escape(&item.kind),
            ));
        }
        body.push_str("</div>");
    }
    body.push_str("</div></div></div>");

    let style = crate::blitz_css::fill(
        &crate::blitz_css::template("browser_blitz/browser.css", include_str!("browser.css")),
        &[
            ("background", theme::BACKGROUND),
            ("border", theme::BORDER),
            ("active", theme::ACTIVE),
            ("active_background", theme::ACTIVE_BACKGROUND),
            ("input_background", theme::INPUT_BACKGROUND),
            ("control_border", theme::CONTROL_BORDER),
            ("mode_border", theme::MODE_BORDER),
            ("mode_background", theme::MODE_BACKGROUND),
            ("rail_background", theme::RAIL_BACKGROUND),
            ("rail_text", theme::RAIL_TEXT),
            ("muted", theme::MUTED),
            ("title", theme::TITLE),
            ("thumb_border", theme::THUMB_BORDER),
            ("item_text", theme::ITEM_TEXT),
            ("item_detail", theme::ITEM_DETAIL),
        ],
    );
    format!("<html><head><style>{style}</style></head><body>{body}</body></html>")
}
