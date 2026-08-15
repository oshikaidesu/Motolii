//! dock を1枚の絵にするための配線。**dock の挙動には触らない。**
//!
//! ## tile tree の出所
//! `crates/motolii-ui/src/dock/tests.rs:64-70`(`oracle_split` の初期状態)の写し。
//! 同fileの5つの POSITIVE ORACLE はすべて同じ木から始まる:
//! pane `a` と pane `b` を `insert_horizontal_tile` で横に並べ、
//! `AREA`(`tests.rs:14-17` = 800x600)で `update` する。
//! 新しいレイアウトはここで発明しない。
//!
//! ## スタイルシートについて(判断が要った唯一の箇所)
//! `dock/css.rs` は **class名だけを出し色を一切持たない**(同file冒頭の宣言)。
//! よって「dockを絵にする」には、どこかで class → 色 の対応を書くしかない。
//! それを `dock/` へ置くと dock が色を持つことになるので、**ダンプ側に置く**。
//! 使う値は `timeline_blitz/theme.rs`(= `timeline_egui/theme.rs` の機械照合済みの写し)
//! にある定数だけで、新しい値は1つも作っていない。
//! この対応付けは**このダンプの見せ方であって、製品の決定ではない**。

use motolii_ui::dock::css::layout_html;
use motolii_ui::dock::geom::{Pos2, Rect};
use motolii_ui::dock::{Behavior, DockInput, Tiles, Tree};
use motolii_ui::timeline_blitz::theme;

/// tests.rs:14-17 の写し。
const AREA: Rect = Rect {
    min: Pos2 { x: 0.0, y: 0.0 },
    max: Pos2 { x: 800.0, y: 600.0 },
};

pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 600;

/// tests.rs:20-26 `TestBehavior` の写し。既定の挙動から何も変えない。
struct DumpBehavior;

impl Behavior<&'static str> for DumpBehavior {
    fn tab_title_for_pane(&mut self, pane: &&'static str) -> String {
        (*pane).to_owned()
    }
}

/// tests.rs の POSITIVE ORACLE が使う木を組み、1パスだけ回して HTML にする。
pub fn dock_html() -> String {
    let mut behavior = DumpBehavior;
    let mut tiles = Tiles::default();
    let a = tiles.insert_pane("a");
    let b = tiles.insert_pane("b");
    let root = tiles.insert_horizontal_tile(vec![a, b]);
    let mut tree = Tree::new("split", root, tiles);

    let frame = tree.update(&mut behavior, &DockInput::default(), AREA);
    let body = layout_html(
        &tree,
        &frame,
        |_id, pane| format!("<div class=\"dock-pane-body\">{pane}</div>"),
        |id| behavior.tab_title_for_tile(&tree.tiles, id),
    );

    format!(
        "<html><head><style>{}</style></head><body>{body}</body></html>",
        stylesheet()
    )
}

/// class → 色。値はすべて `timeline_blitz/theme.rs` の定数。
///
/// 背景は `body` へ置かない(罠1: `blitz-paint` が viewport 全面を塗る)。
/// タイル間の隙間は `dock/css.rs` の宣言どおり**透過のまま**にしてある。
fn stylesheet() -> String {
    let (surface, surface_hi, surface_lo) = (theme::SURFACE, theme::SURFACE_HI, theme::SURFACE_LO);
    let (contrast, accent, ink, ruler) = (theme::CONTRAST, theme::ACCENT, theme::INK, theme::RULER);
    format!(
        "
  html,body {{ margin:0; padding:0; width:{WIDTH}px; height:{HEIGHT}px;
              font-family:sans-serif; color:{ink}; }}
  .dock-pane {{ background:{surface}; }}
  .dock-pane-body {{ position:absolute; left:8px; top:6px; font-size:11px; color:{ruler}; }}
  .dock-splitter {{ background:{contrast}; }}
  .dock-splitter-hovering {{ background:{ruler}; }}
  .dock-splitter-dragging {{ background:{accent}; }}
  .dock-tab {{ background:{surface_lo}; color:{ruler}; font-size:11px; padding:0 8px;
               border:1px solid {contrast}; }}
  .dock-tab-active {{ background:{surface_hi}; color:{ink}; }}
  .dock-drop-parent {{ border:1px solid {ruler}; }}
  .dock-drop-preview {{ background:{surface_hi}; border:1px solid {accent}; }}
"
    )
}
