//! 裁定173 H4 — シェイプ内階層(motolii-vector の `Shape` 入れ子)。
//!
//! 前提: H1(`docs/reviews/2026-08-22-transform-hierarchy-decision.md`、
//! `next/core/motolii-store/tests/transform_hierarchy.rs`)が「親が動くと子も動く・
//! キーフレームは各ノードローカルのまま・合成だけが再帰」を layer 粒度で固定した。
//! ここは同じ考え方・**同じ手計算**を shape 粒度で固定する — 数値証明2本は
//! `transform_hierarchy.rs` の `parent_translation_moves_the_childs_final_world_position`
//! / `rotation_and_scale_compose_in_the_documented_order` と**同型**(同じ数字)。
//!
//! 見るのは平坦化後の頂点座標(`flatten` は純関数・ラスタライズしない)。

use motolii_vector::{Contour, PathSource, Point, RepeaterTransform, Shape, ShapeGroup, ShapeNode};

// ---------------------------------------------------------------------------
// 器具
// ---------------------------------------------------------------------------

/// 頂点1個だけの開いた輪郭を持つ、最小のシェイプ。fill/stroke は無し —
/// ここで見るのは頂点座標だけなので、塗り方は要らない(`black_fill` 等を
/// 持ち込むと試験が実装の写しになる)。
fn point_shape(x: f64, y: f64) -> Shape {
    Shape::new(PathSource::Bezier(vec![Contour::open([Point { x, y }])]))
}

/// [`point_shape`] を `flatten` に通した結果から、その唯一の頂点を取り出す。
fn only_vertex(shape: &Shape) -> Point {
    match &shape.source {
        PathSource::Bezier(path) => path[0].vertices[0].point,
        other => panic!("flatten は PathSource::Bezier へ焼き込むはず: {other:?}"),
    }
}

fn approx(label: &str, a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{label}: {a} != {b}");
}

fn leaf(x: f64, y: f64) -> ShapeNode {
    ShapeNode::Leaf(point_shape(x, y))
}

// ---------------------------------------------------------------------------
// 数値証明1: 平行移動のみ(transform_hierarchy.rs の H1 具体例と同じ数字)
// ---------------------------------------------------------------------------

/// グループ position (100,0) + 子ローカル (10,5) → 子 world (110,5)。
#[test]
fn group_translation_moves_the_childs_final_vertex() {
    let group = ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform {
            position: Point { x: 100.0, y: 0.0 },
            ..RepeaterTransform::IDENTITY
        },
        children: vec![leaf(10.0, 5.0)],
    });

    let flat = motolii_vector::flatten(&[group]).expect("flatten は純関数、失敗しない入力");
    assert_eq!(flat.len(), 1);
    let p = only_vertex(&flat[0]);
    approx("x", p.x, 110.0);
    approx("y", p.y, 5.0);
}

// ---------------------------------------------------------------------------
// 数値証明2: 回転・スケールを含む合成順(H1 の
// rotation_and_scale_compose_in_the_documented_order と手計算まで同型)
// ---------------------------------------------------------------------------

/// グループ: position(100,0)・rotation 90°・scale(2,1)(非一様)。子ローカル:
/// (10,5)。
///
/// 手計算(H1 と同じ式・同じ数字): `S(2,1)·(10,5) = (20,5)` →
/// `R(90°)·(20,5) = (-5,20)`(cos90=0, sin90=1)→ `+ (100,0) = (95,20)`。
#[test]
fn group_rotation_and_scale_compose_in_the_documented_order() {
    let group = ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform {
            position: Point { x: 100.0, y: 0.0 },
            rotation: 90.0,
            scale: Point { x: 2.0, y: 1.0 },
            ..RepeaterTransform::IDENTITY
        },
        children: vec![leaf(10.0, 5.0)],
    });

    let flat = motolii_vector::flatten(&[group]).unwrap();
    let p = only_vertex(&flat[0]);
    approx("x", p.x, 95.0);
    approx("y", p.y, 20.0);
}

// ---------------------------------------------------------------------------
// 入れ子2段の flatten 決定論
// ---------------------------------------------------------------------------

/// 外側 group(+0,+10)の中に内側 group(+10,+0)、その中に子ローカル (1,0)。
/// world = (1,0) → 内側で (11,0) → 外側で (11,10)。**同じ入力を2回 flatten して
/// 一致する**ことも同時に縛る(乱数・時刻を読まない決定論)。
#[test]
fn nested_two_levels_compose_and_flatten_is_deterministic() {
    let inner = ShapeGroup {
        transform: RepeaterTransform {
            position: Point { x: 10.0, y: 0.0 },
            ..RepeaterTransform::IDENTITY
        },
        children: vec![leaf(1.0, 0.0)],
    };
    let outer = ShapeGroup {
        transform: RepeaterTransform {
            position: Point { x: 0.0, y: 10.0 },
            ..RepeaterTransform::IDENTITY
        },
        children: vec![ShapeNode::Group(inner)],
    };
    let nodes = vec![ShapeNode::Group(outer)];

    let a = motolii_vector::flatten(&nodes).unwrap();
    let b = motolii_vector::flatten(&nodes).unwrap();
    assert_eq!(a, b, "同じ入力は同じ出力(決定論) — 合成は純粋な行列積のみ");

    let p = only_vertex(&a[0]);
    approx("x", p.x, 11.0);
    approx("y", p.y, 10.0);
}

// ---------------------------------------------------------------------------
// 空グループの単位元
// ---------------------------------------------------------------------------

/// 空グループ(children が空)は transform がどんな値でも何も出さない —
/// 「単位元」を特別扱いのコードではなく「空なら何もしない」だけで表現している
/// ことの oracle。IDENTITY transform で子を包んでも頂点は動かない。
#[test]
fn empty_group_contributes_nothing_and_identity_transform_changes_nothing() {
    let empty_group = ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform {
            position: Point {
                x: 999.0,
                y: 999.0,
            },
            ..RepeaterTransform::IDENTITY
        },
        children: vec![],
    });
    let identity_wrap = ShapeNode::Group(ShapeGroup {
        transform: RepeaterTransform::IDENTITY,
        children: vec![leaf(3.0, 4.0)],
    });

    let nodes = vec![leaf(1.0, 2.0), empty_group, identity_wrap];
    let flat = motolii_vector::flatten(&nodes).unwrap();

    assert_eq!(flat.len(), 2, "空グループ自身は1つも出さない(単位元)");
    let p0 = only_vertex(&flat[0]);
    approx("sibling.x", p0.x, 1.0);
    approx("sibling.y", p0.y, 2.0);
    let p1 = only_vertex(&flat[1]);
    approx("identity.x", p1.x, 3.0);
    approx("identity.y", p1.y, 4.0);
}

// ---------------------------------------------------------------------------
// serde: 既存 flat shape の JSON 不変・Group 混在の往復
// ---------------------------------------------------------------------------

/// `ShapeNode::Leaf` は `#[serde(untagged)]` なので、包んでいない `Shape` と
/// **文字通り同じ JSON** になる。旧 `Vec<Shape>` の JSON も無改造で
/// `Vec<ShapeNode>` として読める。
#[test]
fn existing_flat_shape_json_is_unchanged() {
    let shape = Shape::new(PathSource::Rectangle {
        size: Point { x: 10.0, y: 10.0 },
    });

    let as_shape_json = serde_json::to_string(&shape).unwrap();
    let as_leaf_json = serde_json::to_string(&ShapeNode::Leaf(shape.clone())).unwrap();
    assert_eq!(
        as_shape_json, as_leaf_json,
        "Leaf(untagged) は Shape と同一の JSON でなければならない"
    );

    // 旧 Document(`Vec<Shape>`)がそのまま保存していた形そのもの。
    let old_document_json = serde_json::to_string(&vec![shape.clone()]).unwrap();
    let parsed: Vec<ShapeNode> = serde_json::from_str(&old_document_json)
        .expect("旧 flat shape JSON は無改造で Vec<ShapeNode> として読めるはず");
    assert_eq!(parsed, vec![ShapeNode::Leaf(shape)]);
}

/// Group と Leaf が混在した木も、そのまま往復する(shape schema への Group 追加
/// が既存 flat 表現を壊していないことの oracle と対になる、新語彙側の oracle)。
#[test]
fn group_mixed_with_leaves_round_trips() {
    let nodes = vec![
        leaf(1.0, 2.0),
        ShapeNode::Group(ShapeGroup {
            transform: RepeaterTransform {
                position: Point { x: 5.0, y: 6.0 },
                ..RepeaterTransform::IDENTITY
            },
            children: vec![
                leaf(7.0, 8.0),
                // 空グループも混在の対象(中身が空でも構造としては保存される)。
                ShapeNode::Group(ShapeGroup {
                    transform: RepeaterTransform::IDENTITY,
                    children: vec![],
                }),
            ],
        }),
    ];

    let json = serde_json::to_string(&nodes).unwrap();
    let back: Vec<ShapeNode> = serde_json::from_str(&json).unwrap();
    assert_eq!(nodes, back);
}
