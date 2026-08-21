//! 裁定174 G1(意図優先の原則 — グループ化動詞)。
//!
//! `Document::group_layers`/`Document::ungroup_layers` の oracle:
//! - ⌘G: N 層選択 → Group 層1 + parent N本が**1 undo**で入る・変換は不変
//!   (Group は単位変換で生まれる)
//! - ⌘⇧G: 子の world 位置が保存される(親の変換を子ローカルへ焼き込み、
//!   H1 `world_affine`/`local_transform` と同じ数字系)
//! - 縁: 空選択 no-op・単一選択可・入れ子グループ・ロック層 M13 拒否

use motolii_store::{
    property, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
    LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames,
        background: Composition::default_background(),
    }))
    .unwrap();
    doc
}

fn place(doc: &mut Document, layer: LayerId, parent: Option<LayerId>) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Null,
                order: layer.0 as i16,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
    if let Some(parent) = parent {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                parent: Some(Some(parent)),
                ..Default::default()
            },
        })
        .unwrap();
    }
}

fn set_vec2(doc: &mut Document, layer: LayerId, name: &str, value: [f64; 2]) {
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(name).unwrap(),
        track: still(Value::Vec2(value)),
    })
    .unwrap();
}

fn set_scalar(doc: &mut Document, layer: LayerId, name: &str, value: f64) {
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(name).unwrap(),
        track: still(Value::F64(value)),
    })
    .unwrap();
}

fn still(value: Value) -> motolii_store::KeyframeTrack {
    let mut track = motolii_store::KeyframeTrack::new();
    track.insert(motolii_store::Keyframe {
        t: t(0),
        value,
        interp: motolii_store::Interp::Hold,
        spatial: None,
    });
    track
}

fn approx(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-3, "{a} != {b}");
}

// ---------------------------------------------------------------------------
// ⌘G — 1 gesture = 1 undo・絵は不変
// ---------------------------------------------------------------------------

/// 3層選択 → Group 層1 + parent 3本が1 undo で入る。undo 1回で完全復元
/// (layer count が元へ戻る)。
#[test]
fn group_layers_creates_one_group_and_undoes_in_one_step() {
    let mut doc = doc_with_comp(300);
    let (a, b, c) = (LayerId(1), LayerId(2), LayerId(3));
    place(&mut doc, a, None);
    place(&mut doc, b, None);
    place(&mut doc, c, None);
    assert_eq!(doc.view().layers().len(), 3);

    let group = doc.group_layers(&[a, b, c]).unwrap().expect("グループが生まれる");
    assert_eq!(doc.view().layers().len(), 4, "Group 層が増えていない");
    assert_eq!(
        doc.view().meta(group).unwrap().unwrap().source,
        LayerSource::Group
    );
    for child in [a, b, c] {
        assert_eq!(
            doc.view().attrs(child).unwrap().unwrap().parent,
            Some(group),
            "子の parent が Group を向いていない"
        );
    }

    assert!(doc.undo(), "undo できない");
    assert_eq!(
        doc.view().layers().len(),
        3,
        "undo 1回で Group 化前へ完全に戻らない = 1操作が複数 undo になっている"
    );
    for child in [a, b, c] {
        assert_eq!(
            doc.view().attrs(child).unwrap().unwrap_or_default().parent,
            None,
            "undo 後も子の parent が残っている"
        );
    }
}

/// Group は単位変換で生まれる(anchor/position/scale/rotation/skew は既定値)
/// ので、子の world 位置はグループ化の前後で変わらない — 絵は不変。
#[test]
fn grouping_does_not_move_the_children_because_the_group_is_born_identity() {
    let mut doc = doc_with_comp(300);
    let (a, b) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, b, None);
    set_vec2(&mut doc, a, property::POSITION, [10.0, 20.0]);
    set_vec2(&mut doc, b, property::POSITION, [-5.0, 40.0]);

    let before_a = doc.view().resolve(a, t(0)).unwrap().unwrap().placement.transform;
    let before_b = doc.view().resolve(b, t(0)).unwrap().unwrap().placement.transform;

    doc.group_layers(&[a, b]).unwrap();

    let after_a = doc.view().resolve(a, t(0)).unwrap().unwrap().placement.transform;
    let after_b = doc.view().resolve(b, t(0)).unwrap().unwrap().placement.transform;
    assert_eq!(before_a, after_a, "グループ化で a の絵が動いた");
    assert_eq!(before_b, after_b, "グループ化で b の絵が動いた");
}

// ---------------------------------------------------------------------------
// ⌘⇧G — 子の world 位置が保存される(焼き込み)
// ---------------------------------------------------------------------------

/// 数値証明(H1 と同じ数字系): Group(position(100,0)・rotation 90°) 下の子
/// (position(10,5))が、Ungroup 後も同じ world 位置になる。
///
/// 手計算(`transform_hierarchy.rs::rotation_and_scale_compose_in_the_documented_order`
/// と同型): 子の world(グループ化中)= R(90°)·(10,5) + (100,0) = (-5,10) +
/// (100,0) = (95,10)。
#[test]
fn ungroup_preserves_the_childs_world_position_with_rotation() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    set_vec2(&mut doc, a, property::POSITION, [10.0, 5.0]);

    let group = doc.group_layers(&[a]).unwrap().unwrap();
    set_vec2(&mut doc, group, property::POSITION, [100.0, 0.0]);
    set_scalar(&mut doc, group, property::ROTATION, 90.0);

    let before = doc.view().resolve(a, t(0)).unwrap().unwrap().placement.transform;
    approx(before.translation.x, 95.0);
    approx(before.translation.y, 10.0);

    let released = doc.ungroup_layers(&[group]).unwrap();
    assert_eq!(released, vec![a], "解放された子が期待と違う");
    assert!(!doc.view().has_layer(group), "Group が tombstone になっていない");
    assert_eq!(
        doc.view().attrs(a).unwrap().unwrap_or_default().parent,
        None,
        "子の parent が Group の親(トップレベル)へ付け替わっていない"
    );

    let after = doc.view().resolve(a, t(0)).unwrap().unwrap().placement.transform;
    approx(after.translation.x, before.translation.x);
    approx(after.translation.y, before.translation.y);
    approx(after.translation.x, 95.0);
    approx(after.translation.y, 10.0);
}

/// Group の親(grandparent)がいる場合でも世界位置は保存される — 焼き込みが
/// `group_local` だけを乗せればよい(grandparent の world は両辺で相殺される)
/// ことの直接証拠。
#[test]
fn ungroup_preserves_world_position_even_under_a_grandparent() {
    let mut doc = doc_with_comp(300);
    let (root, group_seed_a, child) = (LayerId(1), LayerId(2), LayerId(3));
    place(&mut doc, root, None);
    set_vec2(&mut doc, root, property::POSITION, [1000.0, 0.0]);
    place(&mut doc, group_seed_a, Some(root));
    place(&mut doc, child, None);
    set_vec2(&mut doc, child, property::POSITION, [10.0, 5.0]);

    // group_seed_a を子として Group 化する(Group 自身は root の子になる —
    // 呼び手が Group 化前に付けた parent が Group の親として引き継がれる)。
    let group = doc.group_layers(&[child]).unwrap().unwrap();
    doc.apply(Intent::SetAttrs {
        layer: group,
        patch: LayerAttrsPatch {
            parent: Some(Some(root)),
            ..Default::default()
        },
    })
    .unwrap();
    set_vec2(&mut doc, group, property::POSITION, [50.0, -20.0]);

    let before = doc.view().resolve(child, t(0)).unwrap().unwrap().placement.transform;

    doc.ungroup_layers(&[group]).unwrap();
    assert_eq!(
        doc.view().attrs(child).unwrap().unwrap().parent,
        Some(root),
        "子の parent が Group の親(root)へ付け替わっていない"
    );

    let after = doc.view().resolve(child, t(0)).unwrap().unwrap().placement.transform;
    approx(after.translation.x, before.translation.x);
    approx(after.translation.y, before.translation.y);
}

/// Ungroup 直後(Group が恒等のまま)は子の transform track を一切書き換えない
/// — `group_layers` で作った直後にそのまま `ungroup_layers` した往復は完全な
/// no-op(track の中身まで含めてバイト不変)。
#[test]
fn grouping_then_immediately_ungrouping_leaves_the_childs_track_untouched() {
    let mut doc = doc_with_comp(300);
    let (a, b) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, b, None);
    set_vec2(&mut doc, a, property::POSITION, [10.0, 5.0]);

    let before_track = doc
        .view()
        .track(a, &PropertyId::new(property::POSITION).unwrap())
        .unwrap();

    let group = doc.group_layers(&[a, b]).unwrap().unwrap();
    doc.ungroup_layers(&[group]).unwrap();

    let after_track = doc
        .view()
        .track(a, &PropertyId::new(property::POSITION).unwrap())
        .unwrap();
    assert_eq!(
        before_track, after_track,
        "恒等 Group の ungroup が子の position track を書き換えている(不要な副作用)"
    );
}

// ---------------------------------------------------------------------------
// 縁
// ---------------------------------------------------------------------------

#[test]
fn grouping_an_empty_selection_is_a_no_op() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);
    let head_before = doc.edit_head();

    let result = doc.group_layers(&[]).unwrap();
    assert_eq!(result, None, "空選択なのに Group が生まれている");
    assert_eq!(doc.view().layers().len(), 1, "空選択なのに layer 数が変わった");
    assert_eq!(
        doc.edit_head(),
        head_before,
        "空選択なのに undo 刻みが積まれている(edit_head が動いた)"
    );
}

#[test]
fn ungrouping_an_empty_selection_is_a_no_op() {
    let mut doc = doc_with_comp(300);
    let head_before = doc.edit_head();

    let result = doc.ungroup_layers(&[]).unwrap();
    assert_eq!(result, Vec::<LayerId>::new());
    assert_eq!(
        doc.edit_head(),
        head_before,
        "空選択なのに undo 刻みが積まれている(edit_head が動いた)"
    );
}

/// 単一選択も可(その1層だけを子に持つ Group が生まれる)。
#[test]
fn a_single_selected_layer_can_be_grouped() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);

    let group = doc.group_layers(&[a]).unwrap().expect("単一選択でも Group が生まれる");
    assert_eq!(doc.view().attrs(a).unwrap().unwrap().parent, Some(group));

    let released = doc.ungroup_layers(&[group]).unwrap();
    assert_eq!(released, vec![a]);
}

/// 入れ子グループ: Group を含む選択を ⌘G すると、その Group ごと外側の
/// Group の子になる(普通の layer と同じ経路 — 特別扱いは要らない)。
#[test]
fn a_selection_containing_an_existing_group_can_be_grouped_again() {
    let mut doc = doc_with_comp(300);
    let (a, b) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, b, None);
    let inner = doc.group_layers(&[a]).unwrap().unwrap();

    let outer = doc.group_layers(&[inner, b]).unwrap().unwrap();
    assert_eq!(doc.view().attrs(inner).unwrap().unwrap().parent, Some(outer));
    assert_eq!(doc.view().attrs(b).unwrap().unwrap().parent, Some(outer));
    assert_eq!(
        doc.view().attrs(a).unwrap().unwrap().parent,
        Some(inner),
        "内側 Group の子は内側 Group を親にしたまま"
    );
}

/// locked な layer が選択に混じっていると、バッチ全体が拒否される(M13) —
/// Group 自体も作られない(`apply_all` の原子性)。
#[test]
fn grouping_a_selection_with_a_locked_layer_is_rejected_atomically() {
    let mut doc = doc_with_comp(300);
    let (a, locked) = (LayerId(1), LayerId(2));
    place(&mut doc, a, None);
    place(&mut doc, locked, None);
    doc.apply(Intent::SetAttrs {
        layer: locked,
        patch: LayerAttrsPatch {
            locked: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let before_count = doc.view().layers().len();
    let result = doc.group_layers(&[a, locked]);
    assert!(result.is_err(), "locked 層混じりのグループ化が通ってしまった");
    assert_eq!(
        doc.view().layers().len(),
        before_count,
        "拒否されたのに Group 層が残っている(原子性が崩れている)"
    );
    assert_eq!(
        doc.view().attrs(a).unwrap().unwrap_or_default().parent,
        None,
        "拒否されたのに a の parent が変わっている"
    );
}

/// Group でない id を `ungroup_layers` に渡しても無視される(黙って飛ばす)。
#[test]
fn ungrouping_a_non_group_layer_is_ignored() {
    let mut doc = doc_with_comp(300);
    let a = LayerId(1);
    place(&mut doc, a, None);

    let released = doc.ungroup_layers(&[a]).unwrap();
    assert_eq!(released, Vec::<LayerId>::new());
    assert!(doc.view().has_layer(a), "Group でない layer が消されている");
}
