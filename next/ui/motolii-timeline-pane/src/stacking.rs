//! Stage 合成順(`meta.order`)の並べ替えの**意味関数**(純関数)。map B44 行
//! 184/292/293(Bring Layer Forward / Send Layer Backward / Send Layer to
//! Back)+ 正典 §8.1 ReorderLayerUp/Down(+ToEnd)の実体 — Timeline 第3切片
//! (B15+B52 発注)で追加した。
//!
//! ## この動詞が動かすのは Stage の重なりだけ(重要な限定)
//!
//! `projection::rows` は Timeline の縦位置を **LayerId 昇順**で組み、
//! `meta.order` を読まない(同 doc: 「bar の重ね順(`meta.order`)は Stage 側の
//! 合成順であって、Timeline の縦位置の所有者にしない」)。したがって
//! 「レーンバー行の drag 並べ替え」は現行の投影では**意味が store に無い**
//! (行位置は order の投影ではない) — 発注書の柵「意味が store に無い行は
//! 選ばない」により、この切片では行 drag は実装しない(RETURN の finding)。
//! ここにあるのは Stage の重なり(`StoreView::resolved_layers` が
//! `placement.order` **昇順**で合成する = order が大きいほど前面)を動かす
//! 動詞の計算だけ。
//!
//! ## 決定的な手順(呼び手は [`restacked`] の結果を `Intent::SetOrder` へ写す)
//!
//! 1. `(order, LayerId)` 昇順で背面→前面の1列に並べる(同 order は LayerId で
//!    決定的にタイブレーク — `resolved_layers` の `sort_by_key` は安定ソート
//!    なので、タイの実際の合成順とはずれ得るが、**この関数を通した後は全 order
//!    が一意になる**ので以後のタイは消える)
//! 2. 対象集合を1つの block(相対順は保持)として抜き出し、1歩/端へ挿し直す。
//!    非対象が間に挟まった散らばり選択は移動で1つに寄る(coalesce — AE の
//!    複数選択 reorder と同じ見え方)
//! 3. 列全体を 0..n で振り直す — 疎な order 値・i16 端の飽和・重複 order を
//!    まとめて消す。**列の並びが動かなかったら空を返す**(端で既に止まって
//!    いる no-op — 空 undo エントリを作らせない)

use std::collections::HashSet;

use motolii_store::LayerId;

/// 重なり移動の向き。`Forward`/`ToFront` = 前面(order 大)へ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    /// 1歩前面へ(map 184 Bring Layer Forward・§8.1 ReorderLayerUp)。
    Forward,
    /// 1歩背面へ(map 292 Send Layer Backward・§8.1 ReorderLayerDown)。
    Backward,
    /// 最前面へ(§8.1 ReorderLayer…ToEnd の前面側)。
    ToFront,
    /// 最背面へ(map 293 Send Layer to Back)。
    ToBack,
}

/// 現在の `(layer, order)` の集合と対象集合から、並べ替え後に **order が変わる
/// layer だけ**の新しい割当を返す。空 = no-op(対象が居ない・端で既に止まって
/// いる)。モジュール doc の手順1〜3そのもの。
pub fn restacked(
    stack: &[(LayerId, i16)],
    targets: &[LayerId],
    direction: StackDirection,
) -> Vec<(LayerId, i16)> {
    let mut ordered: Vec<(LayerId, i16)> = stack.to_vec();
    ordered.sort_by_key(|&(id, order)| (order, id));

    let target_set: HashSet<LayerId> = targets.iter().copied().collect();
    let block: Vec<LayerId> = ordered
        .iter()
        .map(|&(id, _)| id)
        .filter(|id| target_set.contains(id))
        .collect();
    if block.is_empty() {
        return Vec::new(); // 対象が stack に居ない(消えた layer 等)— no-op。
    }
    let rest: Vec<LayerId> = ordered
        .iter()
        .map(|&(id, _)| id)
        .filter(|id| !target_set.contains(id))
        .collect();

    // 「block の下に居る非対象の枚数」= 挿し込み先。手順2の1歩は、
    // Forward = 最上位対象の上に居る非対象を1枚跨ぐ / Backward = 最下位対象の
    // 下に居る非対象を1枚跨ぐ、と定義する(散らばり選択の coalesce 込み)。
    let first_target_pos = ordered
        .iter()
        .position(|(id, _)| target_set.contains(id))
        .expect("block 非空なら必ず見つかる");
    let last_target_pos = ordered
        .iter()
        .rposition(|(id, _)| target_set.contains(id))
        .expect("block 非空なら必ず見つかる");
    let below = ordered[..first_target_pos]
        .iter()
        .filter(|(id, _)| !target_set.contains(id))
        .count();
    let above = ordered[last_target_pos + 1..]
        .iter()
        .filter(|(id, _)| !target_set.contains(id))
        .count();

    let insert_at = match direction {
        StackDirection::Forward => (rest.len() - above).saturating_add(1).min(rest.len()),
        StackDirection::Backward => below.saturating_sub(1),
        StackDirection::ToFront => rest.len(),
        StackDirection::ToBack => 0,
    };

    let mut new_sequence: Vec<LayerId> = Vec::with_capacity(ordered.len());
    new_sequence.extend_from_slice(&rest[..insert_at]);
    new_sequence.extend_from_slice(&block);
    new_sequence.extend_from_slice(&rest[insert_at..]);

    let old_sequence: Vec<LayerId> = ordered.iter().map(|&(id, _)| id).collect();
    if new_sequence == old_sequence {
        return Vec::new(); // 並びが動かない = no-op(空 undo を作らない)。
    }

    // 手順3: 0..n で振り直し、値が実際に変わる layer だけ返す。
    let current: std::collections::HashMap<LayerId, i16> = stack.iter().copied().collect();
    new_sequence
        .into_iter()
        .enumerate()
        .filter_map(|(index, id)| {
            let new_order = index as i16;
            (current.get(&id) != Some(&new_order)).then_some((id, new_order))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> LayerId {
        LayerId(n)
    }

    /// 変更集合を適用した後の背面→前面の並びを組む(テスト側の独立検算)。
    fn applied_sequence(stack: &[(LayerId, i16)], changes: &[(LayerId, i16)]) -> Vec<LayerId> {
        let mut merged: Vec<(LayerId, i16)> = stack
            .iter()
            .map(|&(layer, order)| {
                let order = changes
                    .iter()
                    .find(|(changed, _)| *changed == layer)
                    .map(|&(_, new_order)| new_order)
                    .unwrap_or(order);
                (layer, order)
            })
            .collect();
        merged.sort_by_key(|&(layer, order)| (order, layer));
        merged.into_iter().map(|(layer, _)| layer).collect()
    }

    /// **オラクル(赤→緑)**: Forward は直上の非対象を1枚だけ跨ぐ(map 184)。
    #[test]
    fn forward_swaps_with_the_layer_just_in_front() {
        let stack = [(id(1), 0), (id(2), 1), (id(3), 2)];
        let changes = restacked(&stack, &[id(1)], StackDirection::Forward);
        assert_eq!(applied_sequence(&stack, &changes), vec![id(2), id(1), id(3)]);
    }

    /// **オラクル(赤→緑)**: Backward は直下の非対象を1枚だけ跨ぐ(map 292)。
    #[test]
    fn backward_swaps_with_the_layer_just_behind() {
        let stack = [(id(1), 0), (id(2), 1), (id(3), 2)];
        let changes = restacked(&stack, &[id(3)], StackDirection::Backward);
        assert_eq!(applied_sequence(&stack, &changes), vec![id(1), id(3), id(2)]);
    }

    /// **オラクル**: ToFront / ToBack は端まで動く(map 293 は ToBack)。
    #[test]
    fn to_front_and_to_back_move_to_the_extremes() {
        let stack = [(id(1), 0), (id(2), 1), (id(3), 2)];
        let front = restacked(&stack, &[id(1)], StackDirection::ToFront);
        assert_eq!(applied_sequence(&stack, &front), vec![id(2), id(3), id(1)]);
        let back = restacked(&stack, &[id(3)], StackDirection::ToBack);
        assert_eq!(applied_sequence(&stack, &back), vec![id(3), id(1), id(2)]);
    }

    /// **オラクル(no-op 柵)**: 端で既に止まっている移動は空を返す —
    /// 空 undo エントリを作らせない(`write.rs` 側はこの空で `apply_all` を
    /// 呼ばない)。
    #[test]
    fn a_move_that_cannot_go_further_returns_no_changes() {
        let stack = [(id(1), 0), (id(2), 1)];
        assert!(restacked(&stack, &[id(2)], StackDirection::Forward).is_empty());
        assert!(restacked(&stack, &[id(1)], StackDirection::Backward).is_empty());
        assert!(restacked(&stack, &[id(2)], StackDirection::ToFront).is_empty());
        assert!(restacked(&stack, &[id(1)], StackDirection::ToBack).is_empty());
    }

    /// **オラクル**: 複数選択は相対順を保った block として動く。間に挟まった
    /// 非対象は移動で跨がれ、block は1つに寄る(モジュール doc 手順2)。
    #[test]
    fn a_scattered_selection_moves_as_one_block_preserving_relative_order() {
        let stack = [(id(1), 0), (id(2), 1), (id(3), 2), (id(4), 3)];
        // 1 と 3 を選んで ToFront — [2, 4, 1, 3](block 内の 1<3 は保たれる)。
        let changes = restacked(&stack, &[id(1), id(3)], StackDirection::ToFront);
        assert_eq!(
            applied_sequence(&stack, &changes),
            vec![id(2), id(4), id(1), id(3)],
            "block の相対順(1が3の背面)が保たれていない"
        );
    }

    /// 疎な order 値(fixture の `layer.0 as i16` 等)でも並びの意味だけで
    /// 動き、結果は 0..n へ正規化される。変わらない layer は返さない。
    #[test]
    fn sparse_orders_are_renumbered_and_unchanged_assignments_are_omitted() {
        let stack = [(id(1), -5), (id(2), 10), (id(3), 100)];
        let changes = restacked(&stack, &[id(1)], StackDirection::Forward);
        assert_eq!(applied_sequence(&stack, &changes), vec![id(2), id(1), id(3)]);
        for &(_, order) in &changes {
            assert!((0..3).contains(&(order as i64)), "0..n へ正規化されていない: {order}");
        }
    }

    /// 対象が stack に存在しない(消えた layer への操作)は no-op。
    #[test]
    fn a_missing_target_is_a_noop() {
        let stack = [(id(1), 0), (id(2), 1)];
        assert!(restacked(&stack, &[id(999)], StackDirection::ToFront).is_empty());
    }

    /// 同 order のタイは LayerId でタイブレークされ、結果は決定的
    /// (`resolved_layers` の安定ソートと同じ列を見る)。
    #[test]
    fn equal_orders_are_tie_broken_by_layer_id_deterministically() {
        let stack = [(id(2), 0), (id(1), 0), (id(3), 0)];
        let changes = restacked(&stack, &[id(1)], StackDirection::ToFront);
        // タイブレーク後の列は [1,2,3] → 1 を前面へ = [2,3,1]。
        assert_eq!(applied_sequence(&stack, &changes), vec![id(2), id(3), id(1)]);
    }
}
