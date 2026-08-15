//! 格子のhit判定と「掴む→動かす→離す」の状態。
//!
//! **離した先で何が起きるかはここで決めない。** 配置intentの意味はC6で未決であり、
//! `docs/blitz-port-order-capsules.md:116` によりRETURN事項。
//! よって release は「どの項目をどこで離したか」だけを返し、
//! `DomainIntent` / `Document` へは一切触れない。

use blitz_dom::BaseDocument;

use super::library_view::BrowserItem;
use super::markup::CARD_ID_PREFIX;

/// 座標(panel左上原点、**CSS px**)から項目indexを引く。
///
/// **格子の式は持たない。** レイアウトはCSSが決めるので、当たり判定もCSSが解決した
/// 結果から引く。以前はここに `spikes/blitz-probe/src/bin/browser_panel.rs:205-216`
/// の式を写していたが、それは `markup.rs` の `cell_origin` と同じ式の複製で、
/// **列数や間隔を変えた瞬間にずれる**。CSSを触るだけで当たり判定が追随するのが
/// HTML/CSSで組んでいることの利点なので、そちらへ寄せた。
///
/// 文書の `Viewport::hidpi_scale` は 1.0(`browser_blitz/mod.rs` の `build_document`)
/// なので、`hit` へ渡す座標も CSS px のままでよい。
///
/// **既知の穴**: blitz-dom の `hit` は z-index を見ない(`node/node.rs` に `TODO: z-index`)。
/// このパネルは card が重ならないので今は問題にならないが、重なる面へ同じ手を
/// 使うときは別の手当てが要る。
pub(super) fn index_at(document: &BaseDocument, x: f64, y: f64) -> Option<usize> {
    let hit = document.hit(x as f32, y as f32)?;
    // 当たったのが `<img>` や名前帯でも、card まで遡って id を拾う。
    let mut node_id = hit.node_id;
    loop {
        let node = document.get_node(node_id)?;
        if let Some(index) = node
            .attrs()
            .and_then(|attrs| {
                attrs
                    .iter()
                    .find(|attr| attr.name.local.as_ref() == "id")
                    .map(|attr| attr.value.as_str())
            })
            .and_then(|id| id.strip_prefix(CARD_ID_PREFIX))
            .and_then(|rest| rest.parse::<usize>().ok())
        {
            return Some(index);
        }
        node_id = node.parent?;
    }
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
