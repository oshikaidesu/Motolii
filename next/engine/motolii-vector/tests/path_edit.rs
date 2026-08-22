//! `edit.rs`(`values/bezier` — Lottie 公式スキーマの頂点編集)の受入条件。
//! 公開 API だけで書ける不変量・境界・退化ケース・store(`Value::Path`)との往復を
//! 見る。曲線の幾何そのもの(分割点が元の曲線上に乗るか)は `pub(crate)` 関数を
//! 直に使うため `src/edit.rs::internal_tests` にある。

use motolii_vector::edit::{
    close_path, contour_from_flat, contour_to_flat, insert_vertex, move_vertex, open_path,
    remove_vertex, set_handles, split_segment, with_contour, PathEditError,
};
use motolii_vector::{Contour, Point, Vertex};

fn corner(x: f64, y: f64) -> Vertex {
    Vertex::corner(Point { x, y })
}

fn triangle() -> Contour {
    Contour {
        vertices: vec![corner(0.0, 0.0), corner(10.0, 0.0), corner(5.0, 10.0)],
        closed: true,
    }
}

// ---------------------------------------------------------------------------
// 頂点数の不変量(`values/bezier` の v/i/o)
// ---------------------------------------------------------------------------

#[test]
fn insert_vertex_grows_by_one_and_places_at_index() {
    let c = triangle();
    let out = insert_vertex(&c, 1, corner(99.0, 99.0)).unwrap();
    assert_eq!(out.vertices.len(), c.vertices.len() + 1);
    assert_eq!(out.vertices[1].point.x, 99.0);
}

#[test]
fn insert_vertex_at_len_appends() {
    let c = triangle();
    let n = c.vertices.len();
    let out = insert_vertex(&c, n, corner(7.0, 7.0)).unwrap();
    assert_eq!(out.vertices.last().unwrap().point.x, 7.0);
}

#[test]
fn insert_vertex_past_len_errs() {
    let c = triangle();
    let n = c.vertices.len();
    let err = insert_vertex(&c, n + 1, corner(0.0, 0.0)).unwrap_err();
    assert_eq!(err, PathEditError::VertexOutOfRange { index: n + 1, len: n });
}

#[test]
fn remove_vertex_shrinks_by_one() {
    let c = triangle();
    let out = remove_vertex(&c, 1).unwrap();
    assert_eq!(out.vertices.len(), c.vertices.len() - 1);
    // 残った頂点は index 0 と旧 index 2。
    assert_eq!(out.vertices[0].point.x, 0.0);
    assert_eq!(out.vertices[1].point.x, 5.0);
}

#[test]
fn remove_vertex_out_of_range_errs() {
    let c = triangle();
    let err = remove_vertex(&c, 3).unwrap_err();
    assert_eq!(err, PathEditError::VertexOutOfRange { index: 3, len: 3 });
}

#[test]
fn move_vertex_preserves_count_and_only_changes_the_point() {
    let c = triangle();
    let out = move_vertex(&c, 0, Point { x: 42.0, y: 42.0 }).unwrap();
    assert_eq!(out.vertices.len(), c.vertices.len());
    assert_eq!(out.vertices[0].point, Point { x: 42.0, y: 42.0 });
    // タンジェント(i/o)は不変 — v[at] だけを触る。
    assert_eq!(out.vertices[0].in_tangent, c.vertices[0].in_tangent);
    assert_eq!(out.vertices[0].out_tangent, c.vertices[0].out_tangent);
    // 触っていない頂点は不変。
    assert_eq!(out.vertices[1].point, c.vertices[1].point);
}

#[test]
fn set_handles_preserves_point_and_count() {
    let c = triangle();
    let out = set_handles(&c, 0, Point { x: -1.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }).unwrap();
    assert_eq!(out.vertices.len(), c.vertices.len());
    assert_eq!(out.vertices[0].point, c.vertices[0].point);
    assert_eq!(out.vertices[0].in_tangent, Point { x: -1.0, y: 0.0 });
    assert_eq!(out.vertices[0].out_tangent, Point { x: 1.0, y: 0.0 });
}

// ---------------------------------------------------------------------------
// 閉じる/開く(`values/bezier` の c)
// ---------------------------------------------------------------------------

#[test]
fn close_and_open_toggle_only_the_flag() {
    let c = Contour {
        vertices: triangle().vertices,
        closed: false,
    };
    let closed = close_path(&c);
    assert!(closed.closed);
    assert_eq!(closed.vertices, c.vertices);
    let reopened = open_path(&closed);
    assert!(!reopened.closed);
    assert_eq!(reopened.vertices, c.vertices);
}

#[test]
fn close_path_is_idempotent() {
    let c = triangle(); // 既に closed
    let closed_again = close_path(&c);
    assert_eq!(closed_again, c);
}

// ---------------------------------------------------------------------------
// セグメント分割 — 境界
// ---------------------------------------------------------------------------

#[test]
fn split_segment_grows_vertex_count_by_one() {
    let c = triangle();
    let out = split_segment(&c, 0, 0.5).unwrap();
    assert_eq!(out.vertices.len(), c.vertices.len() + 1);
}

#[test]
fn split_segment_rejects_boundary_t() {
    let c = triangle();
    assert_eq!(
        split_segment(&c, 0, 0.0).unwrap_err(),
        PathEditError::SplitParameterOutOfRange(0.0)
    );
    assert_eq!(
        split_segment(&c, 0, 1.0).unwrap_err(),
        PathEditError::SplitParameterOutOfRange(1.0)
    );
}

#[test]
fn split_segment_rejects_edge_out_of_range() {
    let c = triangle(); // closed, 3 edges: 0,1,2
    let err = split_segment(&c, 3, 0.5).unwrap_err();
    assert_eq!(err, PathEditError::EdgeOutOfRange { index: 3, len: 3 });
}

#[test]
fn split_segment_open_contour_has_one_fewer_edge_than_vertices() {
    let c = Contour {
        vertices: triangle().vertices,
        closed: false,
    }; // 3頂点・開路 → 辺は2本(0,1)
    assert!(split_segment(&c, 1, 0.5).is_ok());
    let err = split_segment(&c, 2, 0.5).unwrap_err();
    assert_eq!(err, PathEditError::EdgeOutOfRange { index: 2, len: 2 });
}

// ---------------------------------------------------------------------------
// 退化ケース: 0頂点 / 1頂点
// ---------------------------------------------------------------------------

#[test]
fn degenerate_zero_vertex_contour() {
    let empty = Contour {
        vertices: vec![],
        closed: false,
    };
    // 挿入だけは成功する(at=0 は末尾=先頭で境界内)。
    let with_one = insert_vertex(&empty, 0, corner(0.0, 0.0)).unwrap();
    assert_eq!(with_one.vertices.len(), 1);
    // それ以外は全部 out-of-range で弾かれる(パニックしない)。
    assert!(matches!(
        remove_vertex(&empty, 0),
        Err(PathEditError::VertexOutOfRange { index: 0, len: 0 })
    ));
    assert!(matches!(
        move_vertex(&empty, 0, Point::ZERO),
        Err(PathEditError::VertexOutOfRange { index: 0, len: 0 })
    ));
    assert!(matches!(
        set_handles(&empty, 0, Point::ZERO, Point::ZERO),
        Err(PathEditError::VertexOutOfRange { index: 0, len: 0 })
    ));
    assert!(matches!(
        split_segment(&empty, 0, 0.5),
        Err(PathEditError::EdgeOutOfRange { index: 0, len: 0 })
    ));
    // 閉じる/開くはパニックしない(0頂点でも意味のある状態)。
    assert!(close_path(&empty).closed);
}

#[test]
fn degenerate_one_vertex_contour() {
    let one = Contour {
        vertices: vec![corner(3.0, 4.0)],
        closed: false,
    };
    // 頂点を消すと0頂点まで許す(パニックしない)。
    let zero = remove_vertex(&one, 0).unwrap();
    assert_eq!(zero.vertices.len(), 0);
    // 1頂点では辺が無いので split_segment は必ず弾かれる(開路・閉路とも)。
    assert!(matches!(
        split_segment(&one, 0, 0.5),
        Err(PathEditError::EdgeOutOfRange { index: 0, len: 0 })
    ));
    let one_closed = Contour {
        closed: true,
        ..one.clone()
    };
    assert!(matches!(
        split_segment(&one_closed, 0, 0.5),
        Err(PathEditError::EdgeOutOfRange { index: 0, len: 0 })
    ));
    // move/set_handles は唯一の頂点にでも効く。
    assert!(move_vertex(&one, 0, Point { x: 1.0, y: 1.0 }).is_ok());
    assert!(set_handles(&one, 0, Point::ZERO, Point::ZERO).is_ok());
}

// ---------------------------------------------------------------------------
// Path(輪郭の列)橋渡し
// ---------------------------------------------------------------------------

#[test]
fn with_contour_applies_the_edit_to_the_selected_contour_only() {
    let path = vec![triangle(), triangle()];
    let out = with_contour(&path, 1, |c| move_vertex(c, 0, Point { x: 999.0, y: 999.0 })).unwrap();
    assert_eq!(out[0], path[0]); // 触っていない輪郭は不変
    assert_eq!(out[1].vertices[0].point, Point { x: 999.0, y: 999.0 });
}

#[test]
fn with_contour_rejects_out_of_range_contour_index() {
    let path = vec![triangle()];
    let err = with_contour(&path, 1, |c| Ok(close_path(c))).unwrap_err();
    assert_eq!(err, PathEditError::ContourOutOfRange { index: 1, len: 1 });
}

#[test]
fn with_contour_propagates_inner_error() {
    let path = vec![triangle()];
    let err = with_contour(&path, 0, |c| move_vertex(c, 99, Point::ZERO)).unwrap_err();
    assert_eq!(err, PathEditError::VertexOutOfRange { index: 99, len: 3 });
}

// ---------------------------------------------------------------------------
// store(`Value::Path` — `values/bezier` の Rust 側正本)との往復
// ---------------------------------------------------------------------------
//
// この crate は `next/core/*`(`motolii-eval`/`motolii-store`)を引かない
// (`edit.rs` module doc「依存」節、`group.rs` の crate 独立性と同じ理由)ので、
// `motolii_eval::PathVertex`/`Path` そのものを import してはいない。代わりに
// `next/core/motolii-eval/src/value.rs` の `PathVertex { point: [f64;2],
// in_tangent: [f64;2], out_tangent: [f64;2] }` / `Path { vertices, closed }` と
// **フィールド名・型を1対1で揃えた**ローカルのミラー型を使う — 実体の型ではなく
// 「同じ形にラウンドトリップできるか」を見る試験なので、これで往復の主張は
// 検証できる(型そのものの一致は `motolii-store`/`motolii-engine` 側の既存
// テストが見る領分)。
struct MirrorEvalVertex {
    point: [f64; 2],
    in_tangent: [f64; 2],
    out_tangent: [f64; 2],
}

struct MirrorEvalPath {
    vertices: Vec<MirrorEvalVertex>,
    closed: bool,
}

#[test]
fn contour_roundtrips_through_the_stores_flat_path_shape() {
    let c = Contour {
        vertices: vec![
            Vertex {
                point: Point { x: 1.0, y: 2.0 },
                in_tangent: Point { x: -3.0, y: 0.5 },
                out_tangent: Point { x: 3.0, y: -0.5 },
            },
            Vertex {
                point: Point { x: 10.0, y: -4.0 },
                in_tangent: Point::ZERO,
                out_tangent: Point::ZERO,
            },
        ],
        closed: true,
    };

    let (flat, closed) = contour_to_flat(&c);

    // motolii_eval::Path の形へ(呼び手がやる詰め替え。ここでは mirror 型で代用)。
    let eval_path = MirrorEvalPath {
        vertices: flat
            .iter()
            .map(|&(point, in_tangent, out_tangent)| MirrorEvalVertex {
                point,
                in_tangent,
                out_tangent,
            })
            .collect(),
        closed,
    };

    // 「store 側で往復した」を模す — 一度 mirror 型へ落として、また平坦タプルへ戻す。
    let flat_back: Vec<_> = eval_path
        .vertices
        .iter()
        .map(|v| (v.point, v.in_tangent, v.out_tangent))
        .collect();
    let c_back = contour_from_flat(&flat_back, eval_path.closed);

    assert_eq!(c, c_back);
}

#[test]
fn empty_contour_roundtrips() {
    let c = Contour {
        vertices: vec![],
        closed: false,
    };
    let (flat, closed) = contour_to_flat(&c);
    assert!(flat.is_empty());
    let back = contour_from_flat(&flat, closed);
    assert_eq!(c, back);
}
