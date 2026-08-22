//! Timeline レイヤー行の操作束(B52 第2切片、発注「Timeline レイヤー行の
//! 操作束」)の**外部からの柵**。`src/rows.rs` の単体試験(`#[cfg(test)] mod
//! tests`)が実装の中身を検算するのに対し、こちらは `motolii_timeline_pane`
//! を普通の利用者(supervisor 統合後の `write.rs`/`Shell::update`)として
//! 外から呼び、公開面(`pub mod rows;`)が実際に crate 境界を跨いで届くこと
//! 自体を確認する(`tests/transport_fence.rs`/`tests/work_area_shuttle_fence.rs`
//! と同じ「fence」の役割)。
//!
//! **未実行明記**: この発注の検収線は `cargo check --tests` まで
//! (`cargo test` は実行しない) — ここに書く assert は「型が合ってコンパイル
//! が通る」ことまでを見るオラクルであって、実行結果そのものはこの回では
//! 検分していない。
//!
//! オラクル対応(発注書「範囲選択の境界・一括トグルの1 undo・fold の再帰・
//! 空選択の扱い」の4本柱を外側からもなぞる):
//! - (a) 範囲選択の境界: Shift 範囲が anchor から `order` 上の閉区間になる
//! - (b) 一括トグルの1 undo: 戻り値が単一の `bool`(選択集合全員へ同じ値を
//!   1回で書ける形)
//! - (c) fold の再帰: `subtree_ids` が入れ子の子孫まで、兄弟の手前で止まる
//! - (d) 空選択の扱い: 上記どれも空入力で panic しない

use motolii_store::LayerId;
use motolii_timeline_pane::rows::{
    bulk_toggle_target, isolate_others, resolve_layer_selection, subtree_ids, LayerSelectionOp,
};
use motolii_timeline_pane::RowProjection;

fn row(id: u64, depth: u16) -> RowProjection {
    RowProjection {
        id: LayerId(id),
        name: String::new(),
        hidden: false,
        solo: false,
        locked: false,
        label_color: None,
        start: 0,
        duration: 0,
        selected: false,
        dragging: false,
        depth,
        has_children: false,
        children_open: true,
    }
}

// ---------------------------------------------------------------------------
// (a) 範囲選択の境界
// ---------------------------------------------------------------------------

#[test]
fn shift_range_spans_the_closed_interval_between_anchor_and_click() {
    let order = [LayerId(10), LayerId(20), LayerId(30), LayerId(40)];
    let (selection, anchor) = resolve_layer_selection(
        &order,
        Some(LayerId(10)),
        &[],
        LayerSelectionOp::Range(LayerId(30)),
    );
    assert_eq!(selection, vec![LayerId(10), LayerId(20), LayerId(30)]);
    assert_eq!(anchor, Some(LayerId(10)), "Shift 連打の基点(anchor)が動いている");
}

/// 境界: anchor が現在の `order`(見えている行)から消えていたら単独選択へ
/// 安全側で倒れる(fold/削除で行が消えた直後の Shift クリックを想定)。
#[test]
fn shift_range_with_a_vanished_anchor_falls_back_safely() {
    let order = [LayerId(1), LayerId(2)];
    let (selection, _) = resolve_layer_selection(
        &order,
        Some(LayerId(999)),
        &[],
        LayerSelectionOp::Range(LayerId(1)),
    );
    assert_eq!(selection, vec![LayerId(1)]);
}

// ---------------------------------------------------------------------------
// (b) 一括トグルの1 undo(戻り値が単一 bool であること自体が柵)
// ---------------------------------------------------------------------------

#[test]
fn bulk_toggle_target_is_one_bool_selected_layers_can_share_in_a_single_write() {
    // 3層のうち1層だけ既に true(混在) → 全員 true へ揃える1値。
    let target = bulk_toggle_target(&[true, false, true]);
    assert!(target, "混在状態から一括ONへ揃わない");

    // 全員 true → 一括OFFへ折り返す1値(2回目クリックで元に戻る動線)。
    let target_all_on = bulk_toggle_target(&[true, true, true]);
    assert!(!target_all_on, "全員ON状態から一括OFFへ折り返らない");
}

// ---------------------------------------------------------------------------
// (c) fold の再帰
// ---------------------------------------------------------------------------

#[test]
fn subtree_ids_recurses_through_grandchildren_and_stops_before_the_next_sibling() {
    // 1(depth0) -> 2(depth1) -> 3(depth2); 4(depth0) は無関係の兄弟根。
    let rows = vec![row(1, 0), row(2, 1), row(3, 2), row(4, 0)];
    let ids = subtree_ids(&rows, LayerId(1));
    assert_eq!(ids, vec![LayerId(1), LayerId(2), LayerId(3)]);
}

// ---------------------------------------------------------------------------
// (d) 空選択の扱い(3関数とも panic しない)
// ---------------------------------------------------------------------------

#[test]
fn empty_inputs_never_panic_across_the_bundle() {
    assert!(!bulk_toggle_target(&[]));
    assert!(isolate_others(&[], LayerId(1)).is_empty());
    assert!(subtree_ids(&[], LayerId(1)).is_empty());
    let (selection, anchor) =
        resolve_layer_selection(&[], None, &[], LayerSelectionOp::Single(LayerId(1)));
    assert_eq!(selection, vec![LayerId(1)]);
    assert_eq!(anchor, Some(LayerId(1)));
}
