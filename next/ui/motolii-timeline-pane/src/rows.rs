//! Timeline レイヤー行の操作束(B52 第2切片、発注「Timeline レイヤー行の
//! 操作束」)。`clip_gesture.rs`/`key_gesture.rs` と同じ形の**純関数だけの
//! 置き場** — `motolii_store::Document`/`StoreView`/`crate::state::Session`
//! を一切持たない。ここにある関数はどれも「今の状態(引数)」から「次に
//! 適用すべき値」を計算するだけで、書き込み自体は呼び手
//! (`Shell::update`/`write.rs`、supervisor 統合)の仕事(`clip_gesture.rs`
//! module doc の役割分担と同じ)。
//!
//! ## この切片で足す4つの束(RETURN の消化/見送り台帳と対応)
//!
//! - [`LayerSelectionOp`]/[`resolve_layer_selection`] — レイヤー行の複数選択
//!   (単独/Cmd トグル/Shift 範囲)。`crate::state::KeySelectionOp` と同じ
//!   「単独/Toggle/Range」の3形だが、こちらは `Session`/`KeySelector` に
//!   依存しない自己完結版(`LayerId` と `Vec<LayerId>` の順序だけで完結する)。
//!   **map 由来の逸脱**: 発注書の想定候補にあったが、`normal-map.tsv` を
//!   実測すると該当行(id 841/843/966/452「レイヤー選択の拡張/番号選択
//!   トグル/全解除」)は **B31** 所属であって B52/B32 ではない
//!   (EXACT TARGET の許可範囲は B52+B32 のみ)。B32 の一括 M/S/L
//!   ([`bulk_toggle_target`])が実際に効くには複数選択の集合が要るので、
//!   隣接の前提として実装だけはしておく — 消化行 id には数えていない
//!   (RETURN の逸脱節を参照、supervisor/発注元の裁可待ち)。
//! - [`bulk_toggle_target`] — 選択レイヤー一括の M(mute=hidden)/S(solo)/
//!   L(locked)トグル。`normal-map.tsv` 実測: id 40・41・875・970・971
//!   (audio/misc「Lock/Unlock All (Video) Tracks」「Unlock All Layers」)・
//!   1325「Clear Mutes」・1326「Clear Solo」・1345「Mute Tracks」・1355
//!   「Solo Tracks」— どれも「track」という語だが、この map 自体が音声
//!   トラック系の理由欄で「GOALS標準『行の…lock』の音声版」と明記しており、
//!   本ドキュメントの track = Timeline の行(layer)に他ならない(`motolii_store`
//!   に独立した track 型は無い — `LayerAttrs.hidden`/`solo`/`locked` が実体)。
//!   **1関数で3属性を賄う**(hidden/solo/locked どれも「bool の配列→次の
//!   共通値」という同じ形だから、属性ごとに関数を割らない)。
//! - [`isolate_others`] — 「これだけ残して他を消す」束。`normal-map.tsv`
//!   実測: id 219「Hide Other Video」(hide-others)・338「Turn off all
//!   other solo switches」(solo 排他)。M(mute)にも S(solo)にも同じ形で
//!   使える(`bulk_toggle_target` が「揃える」束なら、これは「1つだけ残す」束)。
//! - [`subtree_ids`] — レイヤーの畳み(fold)の**再帰版**。裁定173 H2(単一
//!   layer の `TimelineFoldState::toggle`、`motolii-shell-state` 側に実装
//!   済み)の延長 — 発注書の言う「既存 H2 ツリー行の延長」。**map 由来の
//!   逸脱**: `normal-map.tsv` に「Collapse All」/「Expand All」に相当する
//!   B52/B32 行は無い(H2 自体が map 外の裁定173 由来)。ここでは
//!   「1layerを畳む/開くと、その子孫も一緒に畳む/開く」ための対象 id 集合を
//!   `RowProjection`(深さ優先前順、[`super::projection::rows`] module doc
//!   参照)だけから求める — `children_of` マップを作り直さず、既に確定した
//!   `depth` の並びから「次に自分以下の深さの行が現れるまで」を切り出すだけ
//!   (`StoreView` を持たない自己完結を保つため)。**畳まれて非表示の孫**は
//!   この関数の入力(`rows()` の出力)自体に現れないので対象外
//!   (RETURN の finding 参照)。
//!
//! ## 見送り(track 概念そのもの等、`normal-map.tsv` 実測で意味が実在しない)
//!
//! RETURN 本文の台帳を参照(README/map は不触の指示のためコード内には
//! 全件を転記しない)。要旨: トラック高の増減(id 29/30/35/36/193/220/
//! 1328/1335 — `Dimensions` は静的トークンで Session 側に増減モードの
//! 状態が無い)・トラックの追加/削除/スナップショット/マット(id 173/195/
//! 714/1349/1354/1361 等 — track/snapshot/matte-track は `motolii_store` に
//! 型が無い)・リンク(id 187/229/230 — `LayerAttrs` に link フィールドが
//! 無い)・shy(id 304 — hidden と別概念だが store に無い)・音声レイヤー内
//! 編集(id 33/38/58 — 1 clip 内の複数音声レイヤーは別機構)。

use super::projection::RowProjection;
use motolii_store::LayerId;

// ---------------------------------------------------------------------------
// 複数選択(単独/Cmd トグル/Shift 範囲) — module doc「map 由来の逸脱」参照
// ---------------------------------------------------------------------------

/// レイヤー行クリックの操作種別。`crate::state::KeySelectionOp` と同じ3形
/// (正典 §3・§4 と同じ文法をレイヤー行へ延長したもの)だが、こちらは
/// `KeySelector`(layer+property+frame の3つ組)を要らない — レイヤー行の
/// 識別子は `LayerId` 単体で足りるため、独立した型にしてある。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerSelectionOp {
    /// クリック=単独選択。
    Single(LayerId),
    /// Cmd=トグル(集合への足し引き)。
    Toggle(LayerId),
    /// Shift=`anchor` から `order` 上の範囲。基点が無い/`order` に居ない時は
    /// 単独選択へ安全側で倒す(`crate::write::apply_key_selection` の
    /// `KeySelectionOp::Range` 分岐と同じ安全側の形)。
    Range(LayerId),
}

/// [`LayerSelectionOp`] を解決する。**`Session` を書き換えない** — 呼び手
/// (supervisor 統合後の `write.rs`)が戻り値 `(新しい選択集合, 新しい anchor)`
/// をそのまま `Session::selected_layers`/新設 anchor フィールドへ詰め直す
/// (`crate::write::apply_key_selection` と同じ「解決はここ、確定は呼び手」
/// の役割分担 — 違いは `Session` 型に触れずに済むこと)。
///
/// - `order`: 範囲の基準になる並び順(rail に見えている行の並び — 通常は
///   [`super::projection::rows`] の出力から `.id` を集めたもの。畳まれて
///   非表示の行は対象に入らない、キー行の `key_order` と同じ「見えている
///   ものだけ」の姿勢)。
/// - `anchor`: 直前に単独/Cmd で選んだ行(`Session::key_anchor` と同じ役)。
/// - `current`: 選択前の集合(Toggle 分岐でだけ読む)。
pub fn resolve_layer_selection(
    order: &[LayerId],
    anchor: Option<LayerId>,
    current: &[LayerId],
    op: LayerSelectionOp,
) -> (Vec<LayerId>, Option<LayerId>) {
    match op {
        LayerSelectionOp::Single(id) => (vec![id], Some(id)),
        LayerSelectionOp::Toggle(id) => {
            let mut next = current.to_vec();
            if let Some(pos) = next.iter().position(|&existing| existing == id) {
                next.remove(pos);
            } else {
                next.push(id);
            }
            (next, Some(id))
        }
        LayerSelectionOp::Range(id) => {
            let Some(anchor_id) = anchor else {
                // 基点が無ければ単独選択と同じ扱い(`apply_key_selection` の
                // `KeySelectionOp::Range` フォールバックと同じ安全側)。
                return (vec![id], Some(id));
            };
            let anchor_pos = order.iter().position(|&existing| existing == anchor_id);
            let clicked_pos = order.iter().position(|&existing| existing == id);
            match (anchor_pos, clicked_pos) {
                (Some(a), Some(c)) => {
                    let (lo, hi) = if a <= c { (a, c) } else { (c, a) };
                    // anchor は不変(同じ基点から Shift 連打で範囲を伸縮できる、
                    // `apply_key_selection` の Range 分岐末尾コメントと同じ)。
                    (order[lo..=hi].to_vec(), Some(anchor_id))
                }
                _ => (vec![id], Some(id)),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 選択レイヤー一括 M/S/L トグル
// ---------------------------------------------------------------------------

/// 選択集合の bool 属性(hidden/solo/locked のどれか)を一括で揃える先の
/// **単一の目標値**を返す(`normal-map.tsv` id 40・41・875・970・971・1325・
/// 1326・1345・1355)。**全員が既に true の時だけ false へ**、それ以外
/// (1人でも false が混ざっている/全員 false)は true へ揃える —
/// 「まず全員を同じ状態に集める」を1クリックで達成する一般的な一括トグルの
/// 形(個別に反転すると混在状態のまま何も揃わない)。
///
/// **戻り値が1つの `bool` である**こと自体が「1操作 = 1 undo」の土台
/// (`write.rs` module doc の作法): 呼び手は選択集合の全 `LayerId` へ同じ
/// この1値を書く1本の `Intent`/`apply_all` にまとめられる — 属性ごとに
/// 別々の目標値を返す設計だと、書き込みも人数分に分かれてしまい undo が
/// 割れる。
///
/// 空選択(`states` が空)は呼び手が「何も選ばれていないので no-op」として
/// 弾く前提(`crate::write::nudge_keyframe` の「選択が空なら何もしない」と
/// 同じ姿勢)だが、この関数自体は panic せず `false` を返す(M16、安全側)。
pub fn bulk_toggle_target(states: &[bool]) -> bool {
    if states.is_empty() {
        return false;
    }
    !states.iter().all(|&state| state)
}

// ---------------------------------------------------------------------------
// 排他(これだけ残して他を消す) — Hide Other Video(219)/Solo 排他(338)
// ---------------------------------------------------------------------------

/// `keep` 以外の全 id を返す(「他を消す」束の対象集合)。呼び手はこの戻り値
/// へ属性(hidden=true で「他を隠す」、solo=false で「他のソロを解除」)を
/// 書く。`keep` が `ids` に含まれない場合(選択外の行をクリックした等の
/// 呼び手の契約違反)も panic しない — 単に全 `ids` がそのまま返る。
pub fn isolate_others(ids: &[LayerId], keep: LayerId) -> Vec<LayerId> {
    ids.iter().copied().filter(|&id| id != keep).collect()
}

// ---------------------------------------------------------------------------
// fold の再帰(H2 ツリー行の延長) — module doc「map 由来の逸脱」参照
// ---------------------------------------------------------------------------

/// `root` とその子孫(`rows` の中で**現に見えているもの**だけ)の id 一覧を
/// 深さ優先前順のまま返す(先頭が `root` 自身)。呼び手はこの一覧の各 id へ
/// `TimelineFoldState::fold`/`unfold` を呼べば、1回のジェスチャで
/// 「このレイヤーと全部の子孫」を畳む/開く一括操作になる。
///
/// **前提**: `rows` は [`super::projection::rows`] の出力(親→子の順に深さ
/// 優先で平坦化済み、`depth` は木の深さ)。この前提が崩れている(呼び手が
/// 勝手に並べ替えた)配列を渡すと結果は未定義に近くなる — が、この関数
/// 自体は境界外アクセスも panic もしない(`depth` の大小比較だけで判定する
/// ため)。
///
/// `root` が `rows` に無ければ空 `Vec` を返す(存在しない/既に削除された
/// layer への操作は何もしない、`crate::write::Message::ToggleFold` が
/// 「存在しない LayerId でも panic しない」のと同じ安全側 — module doc の
/// `ghost` 試験参照)。
///
/// **既に畳まれて非表示の孫**は `rows` の入力自体に出てこないので、この
/// 関数の対象にも入らない(recursive-unfold で「畳まれたまま隠れている
/// 孫」まで一度に開く用途には使えない — この関数は「今見えている範囲の
/// 再帰」に限定したスコープ、RETURN の finding 参照)。
pub fn subtree_ids(rows: &[RowProjection], root: LayerId) -> Vec<LayerId> {
    let Some(start) = rows.iter().position(|row| row.id == root) else {
        return Vec::new();
    };
    let root_depth = rows[start].depth;
    let mut out = vec![root];
    for row in &rows[start + 1..] {
        if row.depth <= root_depth {
            break;
        }
        out.push(row.id);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // -----------------------------------------------------------------------
    // resolve_layer_selection: 範囲選択の境界
    // -----------------------------------------------------------------------

    #[test]
    fn single_click_replaces_the_selection_and_sets_the_anchor() {
        let (selection, anchor) =
            resolve_layer_selection(&[], None, &[LayerId(9)], LayerSelectionOp::Single(LayerId(1)));
        assert_eq!(selection, vec![LayerId(1)]);
        assert_eq!(anchor, Some(LayerId(1)));
    }

    #[test]
    fn toggle_adds_when_absent_and_removes_when_present() {
        let current = vec![LayerId(1), LayerId(2)];
        let (added, _) =
            resolve_layer_selection(&[], None, &current, LayerSelectionOp::Toggle(LayerId(3)));
        assert_eq!(added, vec![LayerId(1), LayerId(2), LayerId(3)]);

        let (removed, _) =
            resolve_layer_selection(&[], None, &current, LayerSelectionOp::Toggle(LayerId(1)));
        assert_eq!(removed, vec![LayerId(2)]);
    }

    /// 境界: anchor が無い時は Range も単独選択に安全側で倒れる。
    #[test]
    fn range_without_an_anchor_falls_back_to_a_single_selection() {
        let order = [LayerId(1), LayerId(2), LayerId(3)];
        let (selection, anchor) =
            resolve_layer_selection(&order, None, &[], LayerSelectionOp::Range(LayerId(2)));
        assert_eq!(selection, vec![LayerId(2)]);
        assert_eq!(anchor, Some(LayerId(2)));
    }

    /// 境界: anchor==clicked(1行だけの範囲)。
    #[test]
    fn range_where_anchor_equals_clicked_selects_just_that_one_row() {
        let order = [LayerId(1), LayerId(2), LayerId(3)];
        let (selection, anchor) = resolve_layer_selection(
            &order,
            Some(LayerId(2)),
            &[],
            LayerSelectionOp::Range(LayerId(2)),
        );
        assert_eq!(selection, vec![LayerId(2)]);
        assert_eq!(anchor, Some(LayerId(2)));
    }

    /// 境界: clicked が anchor より前(逆順ドラッグ)でも順序を並べ直して
    /// 範囲を作る。
    #[test]
    fn range_normalizes_when_the_clicked_row_is_before_the_anchor() {
        let order = [LayerId(1), LayerId(2), LayerId(3), LayerId(4)];
        let (selection, anchor) = resolve_layer_selection(
            &order,
            Some(LayerId(4)),
            &[],
            LayerSelectionOp::Range(LayerId(2)),
        );
        assert_eq!(selection, vec![LayerId(2), LayerId(3), LayerId(4)]);
        // anchor は不変(同じ基点から Shift 連打で伸縮できる)。
        assert_eq!(anchor, Some(LayerId(4)));
    }

    /// 境界: anchor/clicked のどちらかが今の `order` に居ない(fold や削除で
    /// 消えた)場合は単独選択へ安全側で倒す。
    #[test]
    fn range_falls_back_to_single_when_the_anchor_is_no_longer_in_order() {
        let order = [LayerId(1), LayerId(2)];
        let (selection, anchor) = resolve_layer_selection(
            &order,
            Some(LayerId(999)),
            &[],
            LayerSelectionOp::Range(LayerId(1)),
        );
        assert_eq!(selection, vec![LayerId(1)]);
        assert_eq!(anchor, Some(LayerId(1)));
    }

    /// 空選択の扱い: `order`/`current` がどちらも空でも panic しない。
    #[test]
    fn resolve_layer_selection_never_panics_on_empty_inputs() {
        let (selection, anchor) =
            resolve_layer_selection(&[], None, &[], LayerSelectionOp::Toggle(LayerId(1)));
        assert_eq!(selection, vec![LayerId(1)]);
        assert_eq!(anchor, Some(LayerId(1)));
    }

    // -----------------------------------------------------------------------
    // bulk_toggle_target: 一括トグルの1 undo(単一値であることそのものが柵)
    // -----------------------------------------------------------------------

    #[test]
    fn bulk_toggle_turns_everyone_on_when_any_one_is_off() {
        assert!(bulk_toggle_target(&[true, false, true]));
        assert!(bulk_toggle_target(&[false, false]));
    }

    #[test]
    fn bulk_toggle_turns_everyone_off_only_when_all_are_already_on() {
        assert!(!bulk_toggle_target(&[true, true, true]));
    }

    /// **1操作=1undo の柵**: 戻り値は選択人数によらず常に1つの `bool` —
    /// 呼び手は選択集合の全員へこの同じ値を1回の書き込みで適用できる
    /// (属性ごとに別々の目標を返す設計だと、書き込みが人数分に割れて undo
    /// が1回で済まなくなる)。ここでは「関数のシグネチャ自体が単一値を
    /// 返す」ことを型で検算する — 3人分呼んでも4人分呼んでも同じ形の答えが
    /// 1つ返ることを確認する。
    #[test]
    fn bulk_toggle_target_returns_a_single_value_regardless_of_selection_size() {
        let three = bulk_toggle_target(&[true, false, true]);
        let four = bulk_toggle_target(&[true, false, true, false]);
        // どちらも「1つの bool」— 呼び手はこの1値だけを選択集合全員へ書けば
        // 1回の apply_all(1 undo)で足りる、という契約自体を確認する。
        assert!(three);
        assert!(four);
    }

    /// 空選択の扱い: panic せず `false`(呼び手側で no-op にする前提の安全側)。
    #[test]
    fn bulk_toggle_target_on_empty_selection_is_false_and_does_not_panic() {
        assert!(!bulk_toggle_target(&[]));
    }

    // -----------------------------------------------------------------------
    // isolate_others
    // -----------------------------------------------------------------------

    #[test]
    fn isolate_others_returns_every_id_except_the_kept_one() {
        let ids = [LayerId(1), LayerId(2), LayerId(3)];
        let others = isolate_others(&ids, LayerId(2));
        assert_eq!(others, vec![LayerId(1), LayerId(3)]);
    }

    /// 空選択の扱い: 空配列を渡しても panic せず空を返す。
    #[test]
    fn isolate_others_on_empty_ids_is_empty() {
        let others: Vec<LayerId> = isolate_others(&[], LayerId(1));
        assert!(others.is_empty());
    }

    #[test]
    fn isolate_others_when_keep_is_not_present_returns_everyone() {
        let ids = [LayerId(1), LayerId(2)];
        let others = isolate_others(&ids, LayerId(999));
        assert_eq!(others, vec![LayerId(1), LayerId(2)]);
    }

    // -----------------------------------------------------------------------
    // subtree_ids: fold の再帰
    // -----------------------------------------------------------------------

    /// 木:
    /// 1(depth0)
    /// ├─ 2(depth1)
    /// │  └─ 3(depth2)
    /// └─ 4(depth1)
    /// 5(depth0、無関係の兄弟根)
    #[test]
    fn subtree_ids_collects_nested_descendants_and_stops_at_the_sibling_boundary() {
        let rows = vec![row(1, 0), row(2, 1), row(3, 2), row(4, 1), row(5, 0)];
        let ids = subtree_ids(&rows, LayerId(1));
        assert_eq!(ids, vec![LayerId(1), LayerId(2), LayerId(3), LayerId(4)], "兄弟根(5)まで巻き込んでいる、または孫(3)を取りこぼしている");
    }

    /// 再帰の途中(depth1 の 2)から呼んでも自分以下だけを返す(祖先(1)や
    /// 兄弟(4)は含めない)。
    #[test]
    fn subtree_ids_from_a_middle_node_only_includes_its_own_descendants() {
        let rows = vec![row(1, 0), row(2, 1), row(3, 2), row(4, 1)];
        let ids = subtree_ids(&rows, LayerId(2));
        assert_eq!(ids, vec![LayerId(2), LayerId(3)]);
    }

    /// 葉(子を持たない)なら自分自身だけ。
    #[test]
    fn subtree_ids_of_a_leaf_is_just_itself() {
        let rows = vec![row(1, 0), row(2, 1)];
        let ids = subtree_ids(&rows, LayerId(2));
        assert_eq!(ids, vec![LayerId(2)]);
    }

    /// 空選択の扱い: 空 `rows` でも、存在しない id でも panic せず空を返す
    /// (`Message::ToggleFold` の ghost 試験と同じ安全側)。
    #[test]
    fn subtree_ids_of_a_missing_root_is_empty() {
        assert!(subtree_ids(&[], LayerId(1)).is_empty());
        let rows = vec![row(1, 0)];
        assert!(subtree_ids(&rows, LayerId(999)).is_empty());
    }
}
