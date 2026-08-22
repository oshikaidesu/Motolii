//! `stack_edit.rs`(`Shape.ops` — `shapes/*` modifier スタックの編集)の受入条件。
//!
//! 構造(`Vec` の境界・退化)は公開 API だけで見る。「編集の効果」(適用/パラメータ
//! 変更/順序が画にどう効くか)は resolve 済みの画素を見る — `shape_1.rs`/`shape_2.rs`
//! と同じ oracle(内部状態ではなく出た画素だけを見る)。

use motolii_vector::stack_edit::{
    insert_op, move_op, remove_op, set_hidden, set_kind, with_ops, StackEditError,
};
use motolii_vector::{
    render, Brush, Canvas, Fill, OpKind, PathSource, Point, Rgb, Shape, ShapeOp, Stroke,
    TrimMultiple,
};

fn canvas() -> Canvas {
    Canvas::centered(200, 200)
}

fn square() -> PathSource {
    PathSource::Rectangle {
        size: Point { x: 100.0, y: 100.0 },
    }
}

fn black_fill() -> Fill {
    Fill {
        brush: Brush::Solid(Rgb::BLACK),
        ..Fill::default()
    }
}

fn black_stroke(width: f64) -> Stroke {
    Stroke {
        brush: Brush::Solid(Rgb::BLACK),
        width,
        ..Stroke::default()
    }
}

fn round(radius: f64) -> ShapeOp {
    ShapeOp::new(OpKind::RoundedCorners { radius })
}

fn trim(start: f64, end: f64) -> ShapeOp {
    ShapeOp::new(OpKind::TrimPath {
        start,
        end,
        offset: 0.0,
        multiple: TrimMultiple::Simultaneously,
    })
}

// ---------------------------------------------------------------------------
// 構造(Vec の境界・退化) — insert/remove/move/set_kind/set_hidden
// ---------------------------------------------------------------------------

#[test]
fn insert_op_grows_the_stack_and_places_at_index() {
    let ops = vec![round(10.0)];
    let out = insert_op(&ops, 0, trim(0.0, 0.5)).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].kind, trim(0.0, 0.5).kind);
    assert_eq!(out[1].kind, round(10.0).kind);
}

#[test]
fn insert_op_at_len_appends_to_an_empty_stack() {
    let out = insert_op(&[], 0, round(5.0)).unwrap();
    assert_eq!(out.len(), 1);
}

#[test]
fn insert_op_past_len_errs() {
    let ops = vec![round(5.0)];
    let err = insert_op(&ops, 2, trim(0.0, 1.0)).unwrap_err();
    assert_eq!(err, StackEditError::InsertOutOfRange { index: 2, len: 1 });
}

#[test]
fn remove_op_shrinks_to_empty() {
    let ops = vec![round(5.0)];
    let out = remove_op(&ops, 0).unwrap();
    assert!(out.is_empty(), "0段まで許すはず");
}

#[test]
fn remove_op_out_of_range_errs_on_empty_stack() {
    let err = remove_op(&[], 0).unwrap_err();
    assert_eq!(err, StackEditError::OpOutOfRange { index: 0, len: 0 });
}

#[test]
fn move_op_reorders_without_changing_contents() {
    let ops = vec![round(10.0), trim(0.0, 0.5)];
    let out = move_op(&ops, 0, 1).unwrap();
    assert_eq!(out[0].kind, trim(0.0, 0.5).kind);
    assert_eq!(out[1].kind, round(10.0).kind);
}

#[test]
fn move_op_same_index_is_identity() {
    let ops = vec![round(10.0), trim(0.0, 0.5)];
    let out = move_op(&ops, 1, 1).unwrap();
    assert_eq!(out, ops);
}

#[test]
fn move_op_out_of_range_errs() {
    let ops = vec![round(10.0)];
    assert_eq!(
        move_op(&ops, 1, 0).unwrap_err(),
        StackEditError::OpOutOfRange { index: 1, len: 1 }
    );
    assert_eq!(
        move_op(&ops, 0, 1).unwrap_err(),
        StackEditError::OpOutOfRange { index: 1, len: 1 }
    );
}

#[test]
fn set_kind_replaces_only_the_kind() {
    let ops = vec![ShapeOp {
        hidden: true,
        kind: round(5.0).kind,
    }];
    let out = set_kind(&ops, 0, trim(0.0, 1.0).kind).unwrap();
    assert_eq!(out[0].kind, trim(0.0, 1.0).kind);
    assert!(out[0].hidden, "hidden は set_kind で変わらないはず");
}

#[test]
fn set_hidden_replaces_only_hidden() {
    let ops = vec![round(5.0)];
    let out = set_hidden(&ops, 0, true).unwrap();
    assert!(out[0].hidden);
    assert_eq!(out[0].kind, round(5.0).kind);
}

#[test]
fn set_kind_and_set_hidden_out_of_range_err_on_empty_stack() {
    assert_eq!(
        set_kind(&[], 0, round(1.0).kind).unwrap_err(),
        StackEditError::OpOutOfRange { index: 0, len: 0 }
    );
    assert_eq!(
        set_hidden(&[], 0, true).unwrap_err(),
        StackEditError::OpOutOfRange { index: 0, len: 0 }
    );
}

#[test]
fn with_ops_rewrites_the_shape_ops_only() {
    let shape = Shape {
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let out = with_ops(&shape, |ops| insert_op(ops, 0, round(20.0))).unwrap();
    assert_eq!(out.ops.len(), 1);
    assert_eq!(out.fill, shape.fill, "ops 以外は変わらないはず");
}

// ---------------------------------------------------------------------------
// 効果 — 適用/パラメータ変更/順序が画に効く(shape_1.rs/shape_2.rs と同じ pixel oracle)
// ---------------------------------------------------------------------------

#[test]
fn applying_a_modifier_via_insert_op_changes_the_picture() {
    let plain = Shape {
        stroke: Some(black_stroke(4.0)),
        ..Shape::new(square())
    };
    let with_round = with_ops(&plain, |ops| insert_op(ops, 0, round(30.0))).unwrap();
    let c = canvas();
    assert_ne!(
        render(&plain, &c).unwrap().premultiplied_rgba8,
        render(&with_round, &c).unwrap().premultiplied_rgba8,
        "insert_op で足した rounded-corners が効いていない"
    );
}

#[test]
fn changing_a_parameter_via_set_kind_changes_the_picture() {
    let base = Shape {
        ops: vec![round(5.0)],
        stroke: Some(black_stroke(4.0)),
        ..Shape::new(square())
    };
    let bigger = with_ops(&base, |ops| set_kind(ops, 0, round(40.0).kind)).unwrap();
    let c = canvas();
    assert_ne!(
        render(&base, &c).unwrap().premultiplied_rgba8,
        render(&bigger, &c).unwrap().premultiplied_rgba8,
        "set_kind で radius を変えても効いていない"
    );
}

/// `shape_1.rs::the_operator_stack_is_ordered` と同じ非可換性を、直接 `Vec` を
/// 組み直すのではなく `move_op` 越しに確かめる。
#[test]
fn reordering_via_move_op_changes_the_picture() {
    let round_then_cut = Shape {
        ops: vec![round(25.0), trim(0.0, 0.5)],
        stroke: Some(black_stroke(4.0)),
        ..Shape::new(square())
    };
    let cut_then_round = with_ops(&round_then_cut, |ops| move_op(ops, 0, 1)).unwrap();
    let c = canvas();
    assert_ne!(
        render(&round_then_cut, &c).unwrap().premultiplied_rgba8,
        render(&cut_then_round, &c).unwrap().premultiplied_rgba8,
        "move_op で並べ替えても同じ画 = 順序が効いていない"
    );
}

/// `graphic-element.hd` の意味 — 消すのではなく黙らせる。`set_hidden` で黙らせた
/// 段は、`remove_op` で外した段と**同じ画**になる。
#[test]
fn hiding_via_set_hidden_matches_removing_the_op() {
    let with_round = Shape {
        ops: vec![round(30.0)],
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let hidden = with_ops(&with_round, |ops| set_hidden(ops, 0, true)).unwrap();
    let removed = with_ops(&with_round, |ops| remove_op(ops, 0)).unwrap();
    let c = canvas();
    assert_eq!(
        render(&hidden, &c).unwrap().premultiplied_rgba8,
        render(&removed, &c).unwrap().premultiplied_rgba8,
        "hidden にした段と外した段は同じ画になるはず"
    );
}

// ---------------------------------------------------------------------------
// 退化ケース
// ---------------------------------------------------------------------------

#[test]
fn removing_the_last_op_returns_to_the_unmodified_shape() {
    let with_round = Shape {
        ops: vec![round(30.0)],
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let plain = Shape {
        fill: Some(black_fill()),
        ..Shape::new(square())
    };
    let out = with_ops(&with_round, |ops| remove_op(ops, 0)).unwrap();
    let c = canvas();
    assert_eq!(
        render(&out, &c).unwrap().premultiplied_rgba8,
        render(&plain, &c).unwrap().premultiplied_rgba8,
    );
}

#[test]
fn empty_stack_move_and_remove_always_err_but_insert_at_zero_succeeds() {
    assert!(move_op(&[], 0, 0).is_err());
    assert!(remove_op(&[], 0).is_err());
    assert!(insert_op(&[], 0, round(1.0)).is_ok());
}
