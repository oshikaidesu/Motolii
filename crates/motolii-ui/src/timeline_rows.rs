//! Timeline の行モデル。**Document の階層を、畳んだ結果の平坦な列にする。**
//!
//! 2026-08-16 の[実行時基盤の再選定](../../../docs/reviews/2026-08-16-timeline-runtime-reselection-to-egui.md)で
//! 決めた **1 Layer = 1行**(Alight Motion 型)をここで実装する。
//!
//! ## なぜ `timeline_projection` を使わないのか
//!
//! `timeline_projection::project_timeline` は今日の決定より前の設計で書かれており、
//! (1) `TrackItem::Group` を `TimelineUnsupported::GroupItem` へ捨て、
//! (2) `assign_bands` で first-fit packing して1レーンに複数 clip を詰め、
//! (3) キーは `transform.position` しか見ない。
//! **どれも 1 Layer = 1行 と再帰 Group Layer に反する。**
//! 行の構造はここが持ち、投影には時間→座標の変換だけを残す。
//!
//! ## 木を持たない
//!
//! 描画も hit も「行の index」で引ければ済むので、木のまま持たない。
//! `rows()` が毎フレーム平坦な列を作る。Document は既に再帰構造
//! (`Group { envelope, children }`)を持っているので、**足りないのは畳み方だけ**である。
//!
//! ## 開閉は2軸独立・持ち主は Project session
//!
//! グループの子を出す軸(左端の矢印)と、キーのパラメータ行を出す軸(`◇`/`◆`)は
//! 互いに影響しない。両方出す・片方だけ出す・どちらも出さない、が全部作れる。
//! **Document schema には入れない** — 開閉は Undo の対象ではなく、
//! Timeline の scroll/zoom と同じ棚(Project session)に置く。

use std::collections::HashSet;

use motolii_doc::{Document, LayerId, TrackItem};

/// キーを持ちうるパラメータ。**effect のパラメータはこの版では扱わない**
/// (必要になったら `RETURN`。enum に足すのは設計判断である)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ParamRef {
    Position,
    Anchor,
    Scale,
    Rotation,
    Opacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowKind {
    /// object の行。clip bar を持ち、キーがあれば要約を出す
    Object,
    /// あるパラメータの key 行。親 object の直下に並ぶ
    Property(ParamRef),
}

/// 画面に出る1行。**縦の index がそのままレーンである**(band を持たない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineRow {
    pub(crate) layer: LayerId,
    pub(crate) kind: RowKind,
    /// 0 = 最上位。Group の子は +1
    pub(crate) depth: u16,
    /// Group かどうかではなく「子を持つか」。空の Group は矢印を出さない
    pub(crate) has_children: bool,
    pub(crate) children_open: bool,
    pub(crate) params_open: bool,
}

/// 開閉状態。Project session が持つ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TimelineFoldState {
    children_open: HashSet<LayerId>,
    params_open: HashSet<LayerId>,
}

impl TimelineFoldState {
    pub(crate) fn open_children(&mut self, layer: LayerId) {
        self.children_open.insert(layer);
    }

    pub(crate) fn close_children(&mut self, layer: LayerId) {
        self.children_open.remove(&layer);
    }

    pub(crate) fn open_params(&mut self, layer: LayerId) {
        self.params_open.insert(layer);
    }

    pub(crate) fn close_params(&mut self, layer: LayerId) {
        self.params_open.remove(&layer);
    }

    pub(crate) fn children_are_open(&self, layer: LayerId) -> bool {
        self.children_open.contains(&layer)
    }

    pub(crate) fn params_are_open(&self, layer: LayerId) -> bool {
        self.params_open.contains(&layer)
    }
}

/// Document と開閉状態から、画面に出る順に行を作る。
///
/// 規則:
/// - 順序は Document の `tracks[*].items` の順。Group の子は Group 行の直後
/// - Group を畳んでいるとき、**Group 行自身は残り、子孫だけが消える**
/// - `params_open` の行は、その object の直後に、キーを持つパラメータだけを並べる
/// - **キーを持たないパラメータの行は出さない**(モックの
///   "Only parameters with keys appear" と同じ)
/// - 2つの軸は独立。畳んだ子の開閉状態は捨てず、開き直したら復元される
///   (`TimelineFoldState` が LayerId で持つので自然にそうなる)
/// - Document に存在しない LayerId が開閉状態に残っていても無視する
pub(crate) fn rows(document: &Document, fold: &TimelineFoldState) -> Vec<TimelineRow> {
    let _ = (document, fold);
    todo!("行モデルの実装。テストを通すこと。他の module を触らないこと")
}

/// その object が、キーを持つパラメータを並び順で返す。
///
/// 並び順は `ParamRef` の宣言順(Position → Anchor → Scale → Rotation → Opacity)で固定する。
/// **Document の内部順序に依存させない** — 行の並びが保存順で揺れると、
/// 同じ画面が2回違う形で出る。
pub(crate) fn keyed_params(item: &TrackItem) -> Vec<ParamRef> {
    let _ = item;
    todo!("キーを持つパラメータの列挙。テストを通すこと")
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::RationalTime;
    use motolii_doc::{
        Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Group, ItemEnvelope,
        KeyframeId, Track, Transform2D,
    };
    use motolii_eval::Interp;

    fn time(seconds: i64) -> RationalTime {
        RationalTime::try_new(seconds, 1).expect("fixture time")
    }

    fn position_keys(document: &mut Document) -> DocParam {
        let key_id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id"));
        let mut keys = DocKeyframeTrack::new();
        keys.insert(DocKeyframe {
            id: key_id,
            t: time(1),
            value: DocValue::Vec2([0.0, 0.0]),
            interp: Interp::Hold,
        });
        DocParam::Keyframes(keys)
    }

    fn clip(document: &mut Document, name: &str, keyed: bool) -> (TrackItem, LayerId) {
        let asset = document
            .assets
            .allocate(name, "video/mp4", &format!("{name}-hash"))
            .expect("asset id");
        let layer = document.layers.allocate(name).expect("layer id");
        let mut envelope = ItemEnvelope::new(layer);
        if keyed {
            envelope.transform = Transform2D {
                position: position_keys(document),
                ..Transform2D::identity()
            };
        }
        (
            TrackItem::Clip(Clip {
                envelope,
                start: time(0),
                duration: time(3),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }),
            layer,
        )
    }

    /// Group(キーあり) > [子A(キーあり), 子B(キーなし)]、および兄弟の平clip
    fn fixture() -> (Document, LayerId, LayerId, LayerId, LayerId) {
        let mut document = Document::new_current();
        let track = document.track_ids.allocate("V1").expect("track id");

        let (child_a, layer_a) = clip(&mut document, "Shared left", true);
        let (child_b, layer_b) = clip(&mut document, "Reference text", false);

        let group_layer = document.layers.allocate("Title scene").expect("layer id");
        let mut group_envelope = ItemEnvelope::new(group_layer);
        group_envelope.transform = Transform2D {
            position: position_keys(&mut document),
            ..Transform2D::identity()
        };
        let group = TrackItem::Group(Group {
            envelope: group_envelope,
            children: vec![child_a, child_b],
        });

        let (sibling, layer_sibling) = clip(&mut document, "Background", false);

        document.tracks.push(Track {
            id: track,
            items: vec![group, sibling],
        });
        document.validate().expect("valid fixture");
        (document, group_layer, layer_a, layer_b, layer_sibling)
    }

    #[test]
    fn group_closed_hides_children_but_keeps_the_group_row() {
        let (document, group, child_a, child_b, sibling) = fixture();
        let out = rows(&document, &TimelineFoldState::default());

        let layers: Vec<LayerId> = out.iter().map(|r| r.layer).collect();
        assert_eq!(layers, vec![group, sibling], "畳んだ子は出ない。Group行は残る");
        assert!(!layers.contains(&child_a));
        assert!(!layers.contains(&child_b));
        assert!(out[0].has_children, "空でないGroupは矢印を出す");
        assert!(!out[0].children_open);
        assert!(!out[1].has_children, "平のclipは矢印を出さない");
    }

    #[test]
    fn group_open_puts_children_directly_after_at_depth_plus_one() {
        let (document, group, child_a, child_b, sibling) = fixture();
        let mut fold = TimelineFoldState::default();
        fold.open_children(group);
        let out = rows(&document, &fold);

        let layers: Vec<LayerId> = out.iter().map(|r| r.layer).collect();
        assert_eq!(layers, vec![group, child_a, child_b, sibling], "子はGroupの直後");
        assert_eq!(out[0].depth, 0);
        assert_eq!(out[1].depth, 1);
        assert_eq!(out[2].depth, 1);
        assert_eq!(out[3].depth, 0, "兄弟は最上位へ戻る");
    }

    #[test]
    fn param_rows_and_child_rows_open_independently() {
        let (document, group, child_a, _child_b, _sibling) = fixture();

        // パラメータだけ開く
        let mut only_params = TimelineFoldState::default();
        only_params.open_params(group);
        let out = rows(&document, &only_params);
        assert_eq!(out[0].layer, group);
        assert_eq!(
            out[1].kind,
            RowKind::Property(ParamRef::Position),
            "Group自身のキー行がGroup行の直下へ出る"
        );
        assert!(
            !out.iter().any(|r| r.layer == child_a),
            "子は出ない。2つの軸は独立"
        );

        // 子だけ開く
        let mut only_children = TimelineFoldState::default();
        only_children.open_children(group);
        let out = rows(&document, &only_children);
        assert!(out.iter().any(|r| r.layer == child_a));
        assert!(
            !out.iter().any(|r| matches!(r.kind, RowKind::Property(_))),
            "パラメータ行は出ない"
        );
    }

    #[test]
    fn reopening_a_group_restores_each_child_fold_state() {
        let (document, group, child_a, _child_b, _sibling) = fixture();
        let mut fold = TimelineFoldState::default();
        fold.open_children(group);
        fold.open_params(child_a);

        let before = rows(&document, &fold);
        assert!(before
            .iter()
            .any(|r| r.layer == child_a && r.kind == RowKind::Property(ParamRef::Position)));

        // Group を畳んで開き直す。子の開閉には触らない
        fold.close_children(group);
        fold.open_children(group);
        let after = rows(&document, &fold);

        assert_eq!(before, after, "畳んで戻すと同じ画面へ復元される");
    }

    #[test]
    fn parameters_without_keys_get_no_row() {
        let (document, group, _child_a, child_b, _sibling) = fixture();
        let mut fold = TimelineFoldState::default();
        fold.open_children(group);
        fold.open_params(child_b); // child_b はキーを1つも持たない

        let out = rows(&document, &fold);
        assert!(
            !out.iter().any(|r| r.layer == child_b && matches!(r.kind, RowKind::Property(_))),
            "キーを持たないパラメータの行は出さない"
        );
        assert!(
            out.iter().any(|r| r.layer == child_b && r.kind == RowKind::Object),
            "object行そのものは出る"
        );
    }

    #[test]
    fn fold_state_for_a_missing_layer_is_ignored() {
        let (document, group, _child_a, _child_b, _sibling) = fixture();
        let mut fold = TimelineFoldState::default();
        fold.open_children(group);

        let ghost = LayerId::from_raw(999_999);
        fold.open_children(ghost);
        fold.open_params(ghost);

        let out = rows(&document, &fold);
        assert!(
            !out.iter().any(|r| r.layer == ghost),
            "消えたLayerIdの開閉状態が残っていても壊れない"
        );
    }

    #[test]
    fn keyed_params_are_listed_in_declaration_order_not_document_order() {
        let mut document = Document::new_current();
        let (item, _layer) = clip(&mut document, "keyed", true);
        assert_eq!(
            keyed_params(&item),
            vec![ParamRef::Position],
            "キーを持つのは position だけ"
        );

        let (bare, _layer) = clip(&mut document, "bare", false);
        assert!(keyed_params(&bare).is_empty());
    }
}
