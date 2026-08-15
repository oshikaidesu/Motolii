// Part of the `egui_tiles` port. Upstream: https://github.com/rerun-io/egui_tiles
// (rerun.io / Emil Ernerfeldt), MIT OR Apache-2.0. This file has no upstream counterpart:
// upstream painted with `egui::Painter`, here the layout result is handed to Blitz as HTML/CSS.
//
// なぜ絶対配置か、なぜ flex/grid の入れ子ではないか:
// 1. ツリーは既に**正確な矩形**を出している。flex/grid に組み直すとCSS側がもう一度
//    レイアウトを解くことになり、share の分配と食い違う余地が生まれる。写すなら矩形を写す。
// 2. 共通 NEGATIVE ORACLE: `spikes/blitz-probe/` で使われていないCSSプロパティを使わない。
//    実測済みなのは position / left / top / width / height / display:flex / flex / gap /
//    align-items / overflow / z-index / pointer-events など。`flex-direction` や
//    `grid-template-*` は probe に無いため使わない(縦分割が書けない)。
//
// 色・寸法定数はここでは一切決めない(共通 NON-GOALS 1)。class 名だけを出し、
// 見た目はホスト側のスタイルシートが `timeline_egui/theme.rs` から与える。
//
// ホスト側スタイルシートへの拘束(2026-08-15 実測 P12 / `spikes/blitz-probe/src/bin/alpha_composite.rs`):
// **背景色を `body` へ置かないこと。** `html` が透明のとき `blitz-paint` は body の背景色で
// viewport 全面を塗る(`blitz-paint-0.3.0-beta.1/src/render.rs:127-160`)。
// ドッキングは各タイルが自分の背景を持つ形なので、背景は `.dock-pane` 等の**個々の要素**へ置く。
// タイル間の隙間を透過のままにすると Stage が透ける(透過合成は実測PASS、出力はプリマルチプライド済み)。
// ここが出す markup は色を一切持たないので、この規則を破れるのはホストのスタイルシートだけである。

use super::geom::Rect;
use super::tree::DockFrame;
use super::{Container, ResizeState, Tile, TileId, Tiles, Tree};

/// `left/top/width/height` for one rectangle, in points.
fn place(rect: Rect) -> String {
    format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px;",
        rect.left(),
        rect.top(),
        rect.width().max(0.0),
        rect.height().max(0.0)
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render the layout result as HTML for Blitz.
///
/// `pane_body` returns the host's own markup for a pane; it is placed inside the pane's box.
/// `tab_title` returns the text of one tab.
pub fn layout_html<Pane>(
    tree: &Tree<Pane>,
    frame: &DockFrame,
    mut pane_body: impl FnMut(TileId, &Pane) -> String,
    mut tab_title: impl FnMut(TileId) -> String,
) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"dock-root\" style=\"position: relative;\">");

    for tile_id in tree.active_tiles() {
        let Some(rect) = tree.tiles.rect(tile_id) else {
            continue;
        };
        match tree.tiles.get(tile_id) {
            Some(Tile::Pane(pane)) => {
                html.push_str(&format!(
                    "<div class=\"dock-pane\" style=\"{} overflow: hidden;\">{}</div>",
                    place(rect),
                    pane_body(tile_id, pane)
                ));
            }
            Some(Tile::Container(Container::Tabs(tabs))) => {
                push_tab_bar(&mut html, tabs, &mut tab_title);
            }
            Some(Tile::Container(_)) | None => {}
        }
    }

    for handle in &frame.handles {
        let class = match handle.state {
            ResizeState::Idle => "dock-splitter",
            ResizeState::Hovering => "dock-splitter dock-splitter-hovering",
            ResizeState::Dragging => "dock-splitter dock-splitter-dragging",
        };
        html.push_str(&format!(
            "<div class=\"{}\" style=\"{} z-index: 2;\"></div>",
            class,
            place(handle.rect)
        ));
    }

    if let Some(parent_rect) = frame.drag_parent_rect {
        html.push_str(&format!(
            "<div class=\"dock-drop-parent\" style=\"{} z-index: 3; pointer-events: none;\"></div>",
            place(parent_rect)
        ));
    }

    if let Some(preview_rect) = frame.drag_preview_rect {
        html.push_str(&format!(
            "<div class=\"dock-drop-preview\" style=\"{} z-index: 4; pointer-events: none;\"></div>",
            place(preview_rect)
        ));
    }

    html.push_str("</div>");
    html
}

fn push_tab_bar(
    html: &mut String,
    tabs: &super::Tabs,
    tab_title: &mut impl FnMut(TileId) -> String,
) {
    for &(child_id, tab_rect) in tabs.tab_rects() {
        let active = if tabs.is_active(child_id) {
            " dock-tab-active"
        } else {
            ""
        };
        html.push_str(&format!(
            "<div class=\"dock-tab{}\" style=\"{} z-index: 1; overflow: hidden; \
             white-space: nowrap; display: flex; align-items: center;\">{}</div>",
            active,
            place(tab_rect),
            escape(&tab_title(child_id))
        ));
    }
}

/// The pane boxes only, for hosts that draw their own chrome.
pub fn pane_rects<Pane>(tiles: &Tiles<Pane>, tile_ids: &[TileId]) -> Vec<(TileId, Rect)> {
    tile_ids
        .iter()
        .filter(|id| matches!(tiles.get(**id), Some(Tile::Pane(_))))
        .filter_map(|&id| tiles.rect(id).map(|rect| (id, rect)))
        .collect()
}
