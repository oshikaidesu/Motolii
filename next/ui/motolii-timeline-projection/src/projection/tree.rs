//! `rows()`(H2 の parent 木の flatten)・`mark_reachable`/`push_layer`・
//! `row_selected` の選択判定(SP-2 分割、`projection.rs` 51-207行・336-345行・
//! 982-1232行を移設)。**中身は無改変**。

use super::*;

/// `store`/`session` から Timeline の行を組み立てる。**読むだけ**。
///
/// `store.layers()` は「present な layer」しか返さない(削除は墓標なので既に除外
/// 済み — `view.rs`)。bar の重ね順(`meta.order`)は Stage 側の合成順であって、
/// Timeline の縦位置の所有者にしない(`ui-score-model.md` 4層構成: 縦位置は
/// packing 結果にすぎない)。
///
/// **裁定173 H2**(旧世界 `crates/motolii-ui/src/timeline_rows.rs::rows` の
/// fold 軸独立 flatten-per-frame アルゴリズムの概念移植): `attrs.parent` を
/// 表示の木の辺として読み、親→子の順に深さ優先で平坦化する。**H1(変換合成)
/// はこの関数の範囲外** — ここは表示上の親子だけを見る、`meta.timing`/
/// `attrs.*` の読み方自体は導入前と無改造。**木を持たない**: 呼ばれるたびに
/// `store.layers()` から作り直す使い捨ての `Vec`(旧世界と同じ思想、mod doc
/// 「木を持たない」節参照)。
///
/// 兄弟の並びは常に `LayerId` 昇順(`store.layers()` の既存順をそのまま踏襲 —
/// 導入前の並びを変えない、oracle「既存 suite 全緑」の根拠)。fold が閉じている
/// layer の子孫は出ない(行自身は残る、旧世界と同じ規則)。**既定 = 全展開**
/// (`TimelineFoldState` doc 参照)なので、parent が1つも設定されていない
/// Document(導入前の全ドキュメント)は今までと完全に同じ並び・同じ行数を返す。
///
/// **循環 parent は投影段でも安全**(H-survey §2.2 の推奨どおり、書き込み時
/// ガード[`validate_no_parent_cycle`]とは独立な第二の柵): 壊れた Document
/// (バグ・旧ファイルの直接編集)を読んでも無限再帰せず、どの layer もちょうど
/// 1回だけ行になる — 根( parent=None)へ辿り着けない循環メンバーは、深さ0の
/// 孤立行として出す(消えない)。
pub fn rows(store: &StoreView<'_>, session: &Session) -> Vec<RowProjection> {
    let layers = store.layers();
    let present: HashSet<LayerId> = layers.iter().copied().collect();

    // 親 → 子(`store.layers()` の昇順のまま集める)のマップ。parent が
    // present set の外(削除済み/存在しない LayerId)を指していたら根として
    // 扱う — 壊れた参照で行を失わない(§2.2 と同じ防衛の精神)。
    let mut children_of: HashMap<Option<LayerId>, Vec<LayerId>> = HashMap::new();
    for &id in &layers {
        let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
        let parent = attrs.parent.filter(|p| present.contains(p));
        children_of.entry(parent).or_default().push(id);
    }

    // **fold を無視した構造的到達性**を先に求める(1巡目)。ここで「`None` から
    // 辿り着けない」と判定された layer だけが、本当に壊れた(循環)参照 —
    // fold で畳まれているだけの子は、fold を見ないこの巡回では普通に辿り
    // 着けるので、ここでは「壊れていない」と正しく判定される(旧設計の
    // バグ: 畳んだ子を「未出力」のまま防衛2巡目に回してしまい、fold を
    // 無視して孤立行として出してしまっていた — 到達性判定と fold 適用を
    // 同じ巡回に混ぜていたのが原因。ここで2つの巡回に分離して切り離す)。
    let mut reachable: HashSet<LayerId> = HashSet::new();
    {
        let mut visiting = HashSet::new();
        if let Some(roots) = children_of.get(&None) {
            for &root in roots {
                mark_reachable(root, &children_of, &mut visiting, &mut reachable);
            }
        }
    }

    // 根の集合 = 本来の根(`parent == None`)+ `None` から辿り着けない孤立
    // (循環)メンバー。後者は本来ならここに来ない(書き込み時ガード
    // `validate_no_parent_cycle` が防ぐ)が、壊れた Document を読んだ時の
    // 第二の柵として、深さ0の孤立行にする(oracle「循環 parent は投影段でも
    // 安全」— 消さない・panic しない)。
    let mut roots: Vec<LayerId> = children_of.get(&None).cloned().unwrap_or_default();
    for &id in &layers {
        if !reachable.contains(&id) {
            roots.push(id);
        }
    }

    // 2巡目(実際の行組み立て)。ここで初めて fold を見る。
    let mut out = Vec::with_capacity(layers.len());
    let mut emitted: HashSet<LayerId> = HashSet::new();
    let mut visiting: HashSet<LayerId> = HashSet::new();
    for root in roots {
        push_layer(store, session, root, 0, &children_of, &mut visiting, &mut emitted, &mut out);
    }
    out
}

/// `rows()` 1巡目: fold を一切見ず、`None`(根)から `children_of` を辿って
/// 構造的に到達できる layer 集合を求める。`push_layer` と同じ
/// `visiting`(同一枝内の循環ガード)+ `reachable`(二重挿入防止、木なので
/// 通常は1回しか呼ばれないが循環時の保険)の形。
fn mark_reachable(
    id: LayerId,
    children_of: &HashMap<Option<LayerId>, Vec<LayerId>>,
    visiting: &mut HashSet<LayerId>,
    reachable: &mut HashSet<LayerId>,
) {
    if !reachable.insert(id) {
        return;
    }
    if !visiting.insert(id) {
        return;
    }
    if let Some(children) = children_of.get(&Some(id)) {
        for &child in children {
            mark_reachable(child, children_of, visiting, reachable);
        }
    }
    visiting.remove(&id);
}

/// 1 layer を、開いていればその子孫まで再帰的に `out` へ積む(旧世界
/// `push_item` の概念移植)。`visiting` は同一枝内の再訪(循環)を止める防御的
/// ガード、`emitted` は「もう出力したか」を覚える(枝をまたいだ二重計上・
/// `rows()` の防衛2巡目での再計上を防ぐ)。
fn push_layer(
    store: &StoreView<'_>,
    session: &Session,
    id: LayerId,
    depth: u16,
    children_of: &HashMap<Option<LayerId>, Vec<LayerId>>,
    visiting: &mut HashSet<LayerId>,
    emitted: &mut HashSet<LayerId>,
    out: &mut Vec<RowProjection>,
) {
    if !emitted.insert(id) {
        return; // 既に出力済み(通常は起きない — 循環防衛の保険)。
    }
    if !visiting.insert(id) {
        return; // 同一枝内で自分自身に戻ってきた(循環) — ここで止める。
    }

    let Ok(Some(meta)) = store.meta(id) else {
        visiting.remove(&id);
        return; // present だが meta が引けない(起こらないはずだが安全側、旧来と同じ)。
    };
    let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
    let children = children_of.get(&Some(id)).cloned().unwrap_or_default();
    let has_children = !children.is_empty();
    let children_open = !session.timeline_fold.is_folded(id);

    out.push(RowProjection {
        id,
        name: attrs.name,
        hidden: attrs.hidden,
        solo: attrs.solo,
        locked: attrs.locked,
        label_color: attrs.label_color,
        start: meta.timing.start,
        duration: meta.timing.duration,
        selected: row_selected(session, id),
        dragging: false,
        depth,
        has_children,
        children_open,
    });

    if children_open {
        for child in children {
            push_layer(store, session, child, depth + 1, children_of, visiting, emitted, out);
        }
    }
    visiting.remove(&id);
}

/// 行ハイライトの選択判定。`selection`(単一 focus)と `selected_layers`
/// (U1 の複数選択集合)は身分が別(`Session` の doc 参照)だが、**行の見た目は
/// どちらも同じ選択**(AE 同型: 複数選択の各 layer 行は同一ハイライト。primary の
/// 区別は property 行の展開(`selected_row_index` = `selection` のみ)が担う)。
pub(crate) fn row_selected(session: &Session, id: LayerId) -> bool {
    session.selection == Some(id) || session.selected_layers.contains(&id)
}

#[cfg(test)]
mod tree_tests {
    use super::*;
    use motolii_store::{
        Composition, Document, Intent, LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming,
    };

    fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).expect("30/1 は正の既約 fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("comp 設定");
        doc
    }

    fn solid() -> LayerSource {
        LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 }
    }

    /// `AddLayer`+`SetMeta` を1回で済ませる fixture ヘルパー(`layer_meta.rs`
    /// の `place` と同じ形)。
    fn place(doc: &mut Document, layer: LayerId, start: i64, duration: i64) {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: solid(),
                    order: layer.0 as i16,
                    timing: LayerTiming { start, duration, source_in: 0, ..Default::default() },
                },
            },
        ])
        .expect("layer 配置");
    }

    fn set_parent(doc: &mut Document, layer: LayerId, parent: LayerId) {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() },
        })
        .expect("parent 設定");
    }

    /// 親(`Title scene`) > [子A(`Shared left`), 子B(`Reference text`)]、
    /// および parent を持たない兄弟(`Background`)。旧世界の fixture
    /// (`Group(キーあり) > [子A, 子B]`+兄弟の平clip)と同型 — Group は今回の
    /// スコープ外(H1)なので、代わりに「親を持つ普通の layer」で同じ形を作る。
    fn fixture() -> (Document, LayerId, LayerId, LayerId, LayerId) {
        let mut doc = doc_with_comp();
        let parent = LayerId(1);
        let child_a = LayerId(2);
        let child_b = LayerId(3);
        let sibling = LayerId(4);
        place(&mut doc, parent, 0, 100);
        place(&mut doc, child_a, 0, 100);
        place(&mut doc, child_b, 0, 100);
        place(&mut doc, sibling, 0, 100);
        set_parent(&mut doc, child_a, parent);
        set_parent(&mut doc, child_b, parent);
        (doc, parent, child_a, child_b, sibling)
    }

    /// **オラクル(赤→緑)**: fold 既定 = 全展開。parent を持つ子は、親を畳んで
    /// いなければ最初から見える(旧世界は既定畳みだったが、H2 は「導入前の
    /// 見た目を壊さない」ため既定を反転している — `TimelineFoldState` doc 参照)。
    #[test]
    fn default_fold_state_shows_the_whole_tree_expanded() {
        let (doc, parent, child_a, child_b, sibling) = fixture();
        let session = Session::default();
        let out = rows(&doc.view(), &session);

        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(
            ids,
            vec![parent, child_a, child_b, sibling],
            "既定(fold なし)で子が最初から展開されていない"
        );
        assert_eq!(out[0].depth, 0);
        assert_eq!(out[1].depth, 1, "子の深さが+1になっていない");
        assert_eq!(out[2].depth, 1);
        assert_eq!(out[3].depth, 0, "parentを持たない兄弟は最上位のまま");
        assert!(out[0].has_children, "子を持つ行が矢印を出さない");
        assert!(out[0].children_open);
        assert!(!out[3].has_children, "子を持たない行が矢印を出している");
    }

    /// **オラクル(赤→緑)**: 親を畳むと、子は消えるが親行自身は残る
    /// (旧世界 `group_closed_hides_children_but_keeps_the_group_row` の移植)。
    #[test]
    fn folding_the_parent_hides_children_but_keeps_the_parent_row() {
        let (doc, parent, child_a, child_b, sibling) = fixture();
        let mut session = Session::default();
        session.timeline_fold.fold(parent);
        let out = rows(&doc.view(), &session);

        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![parent, sibling], "畳んだ子が出てしまっている。親行は残るべき");
        assert!(!ids.contains(&child_a));
        assert!(!ids.contains(&child_b));
        assert!(out[0].has_children, "畳んでいても子を持つことは変わらない");
        assert!(!out[0].children_open);
    }

    /// **オラクル(赤→緑)**: 畳んで開き直すと同じ画面へ復元される(旧世界
    /// `reopening_a_group_restores_each_child_fold_state` の移植 — H2 は
    /// children_open の1軸のみ移植、旧世界の params_open 軸は別玉)。
    #[test]
    fn refolding_and_reopening_restores_the_same_screen() {
        let (doc, parent, ..) = fixture();
        let mut session = Session::default();
        let before = rows(&doc.view(), &session);

        session.timeline_fold.fold(parent);
        session.timeline_fold.unfold(parent);
        let after = rows(&doc.view(), &session);

        assert_eq!(before, after, "畳んで戻すと同じ画面へ復元されない");
    }

    /// **オラクル**: 兄弟の並びは常に `LayerId` 昇順(Document の書き込み順や
    /// parent 設定順に依存しない — `store.layers()` の既存順を踏襲)。
    #[test]
    fn sibling_order_follows_layer_id_ascending_not_write_order() {
        let mut doc = doc_with_comp();
        let parent = LayerId(10);
        let child_hi = LayerId(30);
        let child_lo = LayerId(20);
        place(&mut doc, parent, 0, 100);
        // 書き込み順は hi → lo(昇順の逆)。
        place(&mut doc, child_hi, 0, 100);
        place(&mut doc, child_lo, 0, 100);
        set_parent(&mut doc, child_hi, parent);
        set_parent(&mut doc, child_lo, parent);

        let out = rows(&doc.view(), &Session::default());
        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![parent, child_lo, child_hi], "兄弟の並びがLayerId昇順になっていない");
    }

    /// **オラクル(既存の柵の回帰確認)**: `validate_no_parent_cycle`(書き込み
    /// 時)は今も循環を拒む — H2 はこの既存ガードには触れていない。
    #[test]
    fn writing_a_cyclic_parent_is_still_rejected_at_write_time() {
        let mut doc = doc_with_comp();
        let a = LayerId(1);
        let b = LayerId(2);
        place(&mut doc, a, 0, 100);
        place(&mut doc, b, 0, 100);
        set_parent(&mut doc, a, b);

        let cyclic = doc.apply(Intent::SetAttrs {
            layer: b,
            patch: LayerAttrsPatch { parent: Some(Some(a)), ..Default::default() },
        });
        assert!(cyclic.is_err(), "書き込み時ガードが循環を通してしまっている");
    }

    /// **オラクル(H-survey §2.2 防衛)**: `validate_no_parent_cycle` は書き込み
    /// 時にしか働かない — 壊れた Document(バグ・旧ファイルの直接編集)を
    /// 読んだ時のために、投影段(`push_layer`)にも独立な第二の柵がある。
    /// ここでは書き口を経由せず `children_of` マップへ直接 A↔B の循環を
    /// 仕込み、`push_layer` を直接駆動して無限ループにならず・行を1つも
    /// 失わないことを確かめる(`rows()` は書き口の柵がある限り本物の循環
    /// Document を作れないので、内部関数を直接叩くのがこの柵を見る唯一の道)。
    #[test]
    fn push_layer_survives_a_cyclic_children_map_without_hanging_or_dropping_rows() {
        let mut doc = doc_with_comp();
        let a = LayerId(1);
        let b = LayerId(2);
        place(&mut doc, a, 0, 100); // parent は設定しない(attrs.parent = None のまま)。
        place(&mut doc, b, 0, 100);

        let mut children_of: HashMap<Option<LayerId>, Vec<LayerId>> = HashMap::new();
        children_of.insert(Some(a), vec![b]); // 人為的な循環: a の子が b、
        children_of.insert(Some(b), vec![a]); // b の子が a。

        let mut out = Vec::new();
        let mut visiting = HashSet::new();
        let mut emitted = HashSet::new();
        push_layer(&doc.view(), &Session::default(), a, 0, &children_of, &mut visiting, &mut emitted, &mut out);

        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![a, b], "循環マップで行が欠けた、または多重計上された");
        assert_eq!(out[0].depth, 0);
        assert_eq!(out[1].depth, 1, "循環の2番目のノードが子として1段深くならない");
        assert!(visiting.is_empty(), "呼び出し後にvisitingガードが掃除されていない(将来の再帰が誤爆する)");
    }

    /// Document に存在しない LayerId が fold 状態に残っていても壊れない
    /// (旧世界 `fold_state_for_a_missing_layer_is_ignored` の移植)。
    #[test]
    fn fold_state_for_a_missing_layer_is_ignored() {
        let (doc, parent, ..) = fixture();
        let mut session = Session::default();
        session.timeline_fold.fold(parent);

        let ghost = LayerId(999_999);
        session.timeline_fold.fold(ghost);
        session.timeline_fold.unfold(ghost);

        let out = rows(&doc.view(), &session);
        assert!(!out.iter().any(|r| r.id == ghost), "消えたLayerIdの行が出てしまっている");
    }

    /// 裁定174 G1(グループ化動詞)の実証: `fixture()` は「Group は今回のスコープ
    /// 外(H1)」と当時は注記していたが、G1 着地により `LayerSource::Group` を
    /// 実際の親として使える。`Document::group_layers`(既存 Intent の合成)で
    /// 組んだ Group が、H2 のツリー行へ**改造なしで**そのまま乗ることを見る
    /// — projection.rs 自身は G1 のために1行も変わっていない(裁定174 doc の
    /// 「H2 のツリー行が自動反映するはず」を直接確かめる)。
    #[test]
    fn a_group_layers_group_shows_up_as_a_tree_parent_with_no_projection_changes() {
        let mut doc = doc_with_comp();
        let (a, b, sibling) = (LayerId(1), LayerId(2), LayerId(3));
        place(&mut doc, a, 0, 100);
        place(&mut doc, b, 0, 100);
        place(&mut doc, sibling, 0, 100);

        let group = doc
            .group_layers(&[a, b])
            .expect("グループ化できる")
            .expect("非空選択なので Group が必ず生まれる");
        assert_eq!(doc.view().meta(group).unwrap().unwrap().source, LayerSource::Group);

        let session = Session::default();
        let out = rows(&doc.view(), &session);
        let ids: Vec<LayerId> = out.iter().map(|r| r.id).collect();
        // 兄弟は常に LayerId 昇順(`rows()` の doc 参照)。sibling(3) は
        // group(4)より id が若いので Group より先に並ぶ — Group は最後に
        // 生まれた layer なので `LayerId` が一番大きい(`next_layer_id` は
        // 常に最大+1)。
        assert_eq!(
            ids,
            vec![sibling, group, a, b],
            "Group とその子がツリー行として順どおりに並んでいない"
        );
        let group_row = &out[1];
        assert_eq!(group_row.id, group);
        assert_eq!(group_row.depth, 0);
        assert!(group_row.has_children, "Group 行が子持ち矢印を出していない");
        assert_eq!(out[0].depth, 0, "Group に属さない兄弟が最上位のまま");
        assert_eq!(out[2].depth, 1, "Group の子が深さ+1になっていない");
        assert_eq!(out[3].depth, 1);
    }
}
