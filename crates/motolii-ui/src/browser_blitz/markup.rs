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

pub(super) fn build_html(
    width: u32,
    height: u32,
    title: &str,
    items: &[BrowserItem],
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
        // blitz-net は file スキームを std::fs::read で処理する(probe P11)。
        let src = escape(&item.path.to_string_lossy());
        body.push_str(&format!(
            r#"<div class="{class}" style="left:{x}px;top:{y}px">
                 <img class="th" src="file://{src}" />
                 <div class="nm">{}</div>
               </div>"#,
            escape(&item.name)
        ));
    }

    let thumb_w = theme::CELL_W - 8.0;
    let thumb_h = theme::CELL_H - 24.0;
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

    format!(
        r#"<html><head><style>
  html,body {{ margin:0; padding:0; width:{width}px; height:{height}px;
              font-family:sans-serif; color:{ink}; }}
  .bg {{ position:absolute; left:0; top:0; width:{width}px; height:{height}px;
         background:{desktop}; }}
  .hdr {{ position:absolute; left:0; top:0; width:{width}px; height:{header_h}px;
          background:{surface}; color:{ruler}; font-size:11px; padding:7px 8px;
          border-bottom:1px solid {contrast}; overflow:hidden; white-space:nowrap; }}
  .empty {{ position:absolute; left:12px; top:48px; font-size:11px; color:{warn}; }}
  .card {{ position:absolute; width:{cell_w}px; height:{cell_h}px; background:{surface};
           border:1px solid {surface_lo}; }}
  .card:hover {{ background:{surface_hi}; border-color:{ruler}; }}
  .sel {{ border:1px solid {accent}; }}
  .drag {{ background:{surface_hi}; border:1px solid {ink}; }}
  .th {{ position:absolute; left:4px; top:4px; width:{thumb_w}px; height:{thumb_h}px;
         background:{surface_lo}; object-fit:contain; }}
  .nm {{ position:absolute; left:4px; top:{name_top}px; width:{thumb_w}px; font-size:9px;
         color:{ink}; overflow:hidden; white-space:nowrap; }}
</style></head><body>{body}</body></html>"#
    )
}
