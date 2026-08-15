//! 格子のhit判定と「掴む→動かす→離す」の状態。
//!
//! **離した先で何が起きるかはここで決めない。** 配置intentの意味はC6で未決であり、
//! `docs/blitz-port-order-capsules.md:116` によりRETURN事項。
//! よって release は「どの項目をどこで離したか」だけを返し、
//! `DomainIntent` / `Document` へは一切触れない。

use super::library_view::BrowserItem;
use super::theme;

/// 格子座標(panel左上原点、px)から項目indexを引く。
/// `spikes/blitz-probe/src/bin/browser_panel.rs:205-216` の式の写し。
pub(super) fn index_at(x: f64, y: f64, item_count: usize) -> Option<usize> {
    if y < theme::TOP {
        return None;
    }
    let col = ((x - theme::PAD) / (theme::CELL_W + theme::PAD)).floor();
    let row = ((y - theme::TOP) / (theme::CELL_H + theme::PAD)).floor();
    if col < 0.0 || row < 0.0 || col as usize >= theme::COLS {
        return None;
    }
    let index = row as usize * theme::COLS + col as usize;
    (index < item_count).then_some(index)
}

/// 掴んでいる最中の状態。掴んだ項目と現在のpointer位置だけを持つ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrowserDrag {
    pub index: usize,
    pub pointer: (f64, f64),
}

/// 離した瞬間の観測。**これは配置の確定ではない。**
/// 呼び手がこれをDocumentへどう写すかは未決であり、本moduleは決めない。
///
/// `item` は `media_library` が解決した実在fileであり、
/// `browser_host.rs:23` の `BrowserPlaceIntent` は現状 built-in rectangle source
/// しか受け付けない(`browser_host.rs:120-124` の source 照合)。
/// つまり任意メディアの配置意味は**まだ存在しない**。ここでは作らない。
#[derive(Debug, Clone, PartialEq)]
pub struct BrowserDragRelease {
    pub item: BrowserItem,
    /// panel左上原点のpanel内座標。panel外で離した場合も値はそのまま返す。
    pub pointer: (f64, f64),
    /// panelの表示領域内で離したか。領域外の意味づけも呼び手に委ねる。
    pub inside_panel: bool,
}
