//! サムネイル格子のHTML/CSSを組む。
//!
//! **Blitzはブラウザではない。** JSエンジンは無く、ブラウザで効くCSSが silent に
//! 効かない事例が実測で出ている(`docs/reviews/2026-08-15-blitz-ui-runtime-probe.md`)。
//! よってここで使うCSSプロパティは `spikes/blitz-probe/src/bin/browser_panel.rs:102-118`
//! で実際に効いたものだけに限る:
//!   margin / padding / width / height / background / font-family / color /
//!   position / left / top / border / border-bottom / border-color /
//!   overflow / white-space / font-size / object-fit
//! セレクタも同ファイルで実証済みの要素・class・`:hover` だけ。

use std::path::PathBuf;

use super::library_view::BrowserItem;
use super::theme;

/// 表示中の選択・ドラッグ状態。意味は「どのcardを強調するか」だけで、
/// Documentへの影響は持たない。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct GridHighlight {
    pub(super) selected: Option<usize>,
    pub(super) dragging: Option<usize>,
}

/// テキストノード/属性値へ入れる前の最小escape。
/// probe は escape していないが、走査対象は利用者のフォルダなので
/// `&<>"` を含むfile名でmarkupが壊れないようにする。
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

/// 格子1セルの左上座標。hit判定(`interaction.rs`)と同じ式を使う。
pub(super) fn cell_origin(index: usize) -> (f64, f64) {
    let col = index % theme::COLS;
    let row = index / theme::COLS;
    (
        theme::PAD + col as f64 * (theme::CELL_W + theme::PAD),
        theme::TOP + row as f64 * (theme::CELL_H + theme::PAD),
    )
}

/// `thumbs` は `items` と**同じ長さ・同じ並び**の縮小実体path(`thumbnail.rs`)。
/// `None` の項目は画像なしで描く(元画像へは戻さない — 戻すと重さの原因が残る)。
pub(super) fn build_html(
    width: u32,
    height: u32,
    title: &str,
    items: &[BrowserItem],
    thumbs: &[Option<PathBuf>],
    highlight: GridHighlight,
) -> String {
    let mut body = String::new();
    // 罠(b): `body` に背景色を置くと viewport 全面が不透明で塗り潰される
    // (`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。
    // パネルの地色は body ではなく通常の要素として敷く。
    body.push_str(r#"<div class="bg"></div>"#);
    body.push_str(&format!(
        r#"<div class="hdr">{} ({}件)</div>"#,
        escape(title),
        items.len()
    ));
    if items.is_empty() {
        body.push_str(r#"<div class="empty">画像が見つかりません。</div>"#);
    }
    for (index, item) in items.iter().enumerate() {
        let (x, y) = cell_origin(index);
        let class = if Some(index) == highlight.dragging {
            "card drag"
        } else if Some(index) == highlight.selected {
            "card sel"
        } else {
            "card"
        };
        // **元画像ではなく縮小実体を指す。** 元寸を出すとatlasを元解像度で食う
        // (`thumbnail.rs` / `library_view.rs` の実測)。
        // blitz-net は file スキームを std::fs::read で処理する(probe P11)。
        let img = match thumbs.get(index).and_then(|thumb| thumb.as_ref()) {
            Some(thumb) => format!(
                r#"<img class="th" src="file://{}" />"#,
                escape(&thumb.to_string_lossy())
            ),
            // 作れなかった項目。画像なしのcardとして描く。
            None => String::new(),
        };
        body.push_str(&format!(
            r#"<div class="{class}" style="left:{x}px;top:{y}px">
                 {img}
                 <div class="nm">{}</div>
               </div>"#,
            escape(&item.name)
        ));
    }

    let thumb_w = theme::THUMB_W;
    let thumb_h = theme::THUMB_H;
    let name_top = theme::CELL_H - 18.0;
    let header_h = theme::HEADER_H;
    let (desktop, surface, surface_hi, surface_lo) = (
        theme::DESKTOP,
        theme::SURFACE,
        theme::SURFACE_HI,
        theme::SURFACE_LO,
    );
    let (contrast, ruler, accent, ink, warn) = (
        theme::CONTRAST,
        theme::RULER,
        theme::ACCENT,
        theme::INK,
        theme::PALETTE_4,
    );
    let (cell_w, cell_h) = (theme::CELL_W, theme::CELL_H);

    let style = fill(
        &css_template(),
        &[
            ("width", &width.to_string()),
            ("height", &height.to_string()),
            ("header_h", &header_h.to_string()),
            ("cell_w", &cell_w.to_string()),
            ("cell_h", &cell_h.to_string()),
            ("thumb_w", &thumb_w.to_string()),
            ("thumb_h", &thumb_h.to_string()),
            ("name_top", &name_top.to_string()),
            ("desktop", desktop),
            ("surface_hi", surface_hi),
            ("surface_lo", surface_lo),
            ("surface", surface),
            ("contrast", contrast),
            ("ruler", ruler),
            ("accent", accent),
            ("ink", ink),
            ("warn", warn),
        ],
    );

    format!("<html><head><style>{style}</style></head><body>{body}</body></html>")
}

/// CSSの実体。**既定は埋め込み、`MOTOLII_BLITZ_CSS_DIR` があれば実行時に読み直す。**
///
/// CSSがRustの文字列リテラルに埋まっていると、色を1つ変えるのに `motolii-ui` の
/// 再ビルドが要る(実測 23〜30秒)。外に出すと、その往復から**ビルドが消える** —
/// ファイルを保存して dump を実行し直すだけになる(実測 0.70秒)。
///
/// 製品のバイナリは `include_str!` の方を使うので、実行時にファイルを要求しない。
fn css_template() -> std::borrow::Cow<'static, str> {
    const EMBEDDED: &str = include_str!("browser.css");
    if let Some(dir) = std::env::var_os("MOTOLII_BLITZ_CSS_DIR") {
        let path = std::path::Path::new(&dir).join("browser.css");
        match std::fs::read_to_string(&path) {
            Ok(text) => return std::borrow::Cow::Owned(text),
            Err(error) => {
                eprintln!(
                    "browser_blitz: {} を読めないので埋め込みのCSSで描く: {error}",
                    path.display()
                );
            }
        }
    }
    std::borrow::Cow::Borrowed(EMBEDDED)
}

/// `{name}` を差し替える。**長い名前から先に**渡すこと
/// (`surface_hi` を `surface` より先に置換しないと `{surface_hi}` が壊れる)。
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in values {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}
