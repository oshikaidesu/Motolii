//! `values/bezier`(Lottie 公式スキーマ、`reference/lottie-coverage.tsv` 参照)に
//! 対する頂点編集。
//!
//! **意味の正本は Lottie の地図であって UI 動詞の束ではない**。`values/bezier` は
//! 4フィールドだけの閉じた語彙 — `v`(Vertices)/`i`(In Tangents)/`o`(Out Tangents)/
//! `c`(Closed)、全部**採用済**で、この crate では [`Contour`]
//! (`vertices: Vec<Vertex>, closed: bool` — `Vertex` が `v`/`i`/`o` を1つに束ねる)
//! として既に実装済み(`geom.rs` module doc「Lottie の `bezier` の `v`/`i`/`o` と
//! 同型」)。ここが足すのは**新しい語彙ではない** — 4フィールドへの読み書きを
//! 「頂点1個の追加/削除/移動」「ハンドルの設定」「閉路の開閉」という**単位**で
//! まとめた純関数と、それらの合成である「セグメント分割」(De Casteljau — 既存の
//! `v`/`i`/`o` だけから導出できる、新しい入力を要らない操作)だけである。
//!
//! # 各関数と Lottie 語彙の対応
//!
//! | 関数 | 触る `values/bezier` フィールド | 備考 |
//! |---|---|---|
//! | [`insert_vertex`] | `v`+`i`+`o` に1組追加 | `Vertex` が3つを束ねるので同時に増える(型で同期を縛る、地図の note のとおり) |
//! | [`remove_vertex`] | `v`+`i`+`o` から1組削除 | 同上 |
//! | [`move_vertex`] | `v[at]` のみ | `i`/`o` は頂点相対なので不変 |
//! | [`set_handles`] | `i[at]`/`o[at]` のみ | `v[at]` は不変 |
//! | [`close_path`] | `c = true` | |
//! | [`open_path`] | `c = false` | |
//! | [`split_segment`] | `v`/`i`/`o` から1組を導出して挿入 | 新しい入力ではなく既存2頂点(`v`/`i`/`o` 込み)から De Casteljau で導く。地図に無い概念を持ち込んでいない |
//! | [`contour_to_flat`]/[`contour_from_flat`] | `v`/`i`/`o`/`c` 全体 | `motolii_eval::Path`(store の `Value::Path` 正本)との往復 |
//!
//! # 単位は `Contour`(輪郭)であって `Path`(輪郭の列)ではない
//!
//! `motolii_eval::Path`(store の `Value::Path` 正本、`next/core/motolii-eval/
//! src/value.rs`)が最初から単一輪郭(`vertices: Vec<PathVertex>, closed: bool`)
//! なので、この crate の `Contour` 1個を編集単位にするほうが `values/bezier` との
//! 対応が1:1になる。`Shape.source` 側の複数輪郭([`PathSource::Bezier`])を
//! 束ねる話は [`with_contour`] が担う——「選んだ1輪郭に編集を適用する」以上の
//! ことはしない橋渡し。
//!
//! # 依存(引かない)
//!
//! `next/core/*`(`motolii-eval`/`motolii-store`)は**引かない**——`group.rs` が
//! 明言している「この crate は元々 `next/core/*` を1つも引かない(裁定 crate
//! 独立性)」を dev-dependency も含めて破らない。`motolii_eval::Path`/`PathVertex`
//! との往復は具体型ではなく `([f64;2],[f64;2],[f64;2])` の平坦タプルで運ぶ
//! ([`contour_to_flat`]/[`contour_from_flat`]) — `next/engine/motolii-engine/
//! src/mask.rs::eval_path_to_vector_path` が既にこの構造的往復を1方向だけ
//! 手書きしているのと同じ形で、ここは両方向を crate 内の共通関数にしただけ
//! (mask.rs 自体は EXACT TARGET 外なので書き換えない)。
//!
//! # UI 側への口(次波が呼ぶ場所、まだ配線しない)
//!
//! ツール側は `Shape.source` が `PathSource::Bezier(path)` の時、編集中の輪郭
//! index を持ち、[`with_contour`] 経由でここの関数を呼んで新しい `Path` を作り、
//! `Shape` へ書き戻す想定(このモジュールはどれも**入力を書き換えず新しい値を
//! 返す純関数**)。どのジェスチャがどの関数を呼ぶかは UI 発注(次波)の仕事であって、
//! この波では決め打たない。

use crate::geom::{Contour, Path, Point, Vertex};
use crate::ops;
#[cfg(test)]
use crate::geom::bezier_point;

/// 頂点編集の失敗。**壊れた入力を黙って丸めない**(裁定37 と同じ考え方 — この
/// crate の他のエラー型 [`crate::VectorError`] と同じ姿勢)。UI 側の bug(存在しない
/// index を渡した)を「何も起きなかった」に化けさせない。
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum PathEditError {
    /// [`Path`](輪郭の列)に存在しない輪郭 index を指した。
    #[error("contour index {index} is out of range (path has {len} contours)")]
    ContourOutOfRange { index: usize, len: usize },
    /// [`Contour`] に存在しない頂点 index を指した。
    #[error("vertex index {index} is out of range (contour has {len} vertices)")]
    VertexOutOfRange { index: usize, len: usize },
    /// [`split_segment`] に存在しない辺 index を指した(頂点が2未満で辺が0本の
    /// 場合も含む)。
    #[error("edge index {index} is out of range (contour has {len} edges)")]
    EdgeOutOfRange { index: usize, len: usize },
    /// [`split_segment`] の `t` は `(0, 1)` の開区間でなければならない —
    /// 端は既存頂点と重複するため「分割」にならない。
    #[error("split parameter t={0} must be in the open interval (0, 1)")]
    SplitParameterOutOfRange(f64),
}

// ---------------------------------------------------------------------------
// Path(輪郭の列)⇔ 1輪郭 の橋渡し
// ---------------------------------------------------------------------------

/// `path[contour]` に `f` を適用した新しい [`Path`] を返す。**このモジュールの他の
/// 関数は全部 [`Contour`] 単位**(module doc「単位は Contour」参照)なので、
/// 複数輪郭を持つ `Shape.source` 側から呼ぶときの唯一の橋渡しがここになる。
pub fn with_contour<F>(path: &Path, contour: usize, f: F) -> Result<Path, PathEditError>
where
    F: FnOnce(&Contour) -> Result<Contour, PathEditError>,
{
    let c = path.get(contour).ok_or(PathEditError::ContourOutOfRange {
        index: contour,
        len: path.len(),
    })?;
    let updated = f(c)?;
    let mut out = path.clone();
    out[contour] = updated;
    Ok(out)
}

// ---------------------------------------------------------------------------
// 頂点の追加・削除・移動・ハンドル(`values/bezier` の v/i/o)
// ---------------------------------------------------------------------------

/// `at` の位置へ頂点を挿入する(`v`/`i`/`o` に1組追加)。`at == vertices.len()` は
/// 末尾追加まで許す(挿入位置として自然な境界 — 空輪郭への最初の1点もこれで作れる)。
pub fn insert_vertex(contour: &Contour, at: usize, vertex: Vertex) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at > len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices.insert(at, vertex);
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

/// `at` の頂点を消す(`v`/`i`/`o` から1組削除)。**0頂点まで許す** — 空輪郭は
/// 「何も描かれない」だけで、この crate の他の演算子(`round_corners`/
/// `pucker_bloat`/`zigzag` 等)は既に 0/1頂点をパニックせず素通しする前提で
/// 書かれている(`ops.rs` の早期 return 群)。
pub fn remove_vertex(contour: &Contour, at: usize) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at >= len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices.remove(at);
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

/// アンカー(`v[at]`)だけを動かす。タンジェントは頂点相対
/// ([`Vertex::in_tangent`]/[`Vertex::out_tangent`] は `geom.rs` の doc のとおり
/// `values/bezier` の `i`/`o` と同型で頂点からの相対ベクトル)なので、**ここでは
/// `i`/`o` に触らない** — 相対値は点が動いても意味を保つ。ハンドルごと動かしたい
/// 場合は呼び手が [`set_handles`] を別途呼ぶ(2つの仕事を1関数に混ぜない)。
pub fn move_vertex(contour: &Contour, at: usize, point: Point) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at >= len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices[at].point = point;
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

/// ベジエハンドル(`i[at]`/`o[at]`)を差し替える。`v[at]` には触らない。
pub fn set_handles(
    contour: &Contour,
    at: usize,
    in_tangent: Point,
    out_tangent: Point,
) -> Result<Contour, PathEditError> {
    let len = contour.vertices.len();
    if at >= len {
        return Err(PathEditError::VertexOutOfRange { index: at, len });
    }
    let mut vertices = contour.vertices.clone();
    vertices[at].in_tangent = in_tangent;
    vertices[at].out_tangent = out_tangent;
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

// ---------------------------------------------------------------------------
// 閉じる/開く(`values/bezier` の c)
// ---------------------------------------------------------------------------

/// 輪郭を閉じる(`c = true`)。`v`/`i`/`o` には触らない — 既に閉じていても
/// 冪等(同じ結果をもう一度返すだけ)。
pub fn close_path(contour: &Contour) -> Contour {
    Contour {
        vertices: contour.vertices.clone(),
        closed: true,
    }
}

/// 輪郭を開く(`c = false`)。[`close_path`] と対称。
pub fn open_path(contour: &Contour) -> Contour {
    Contour {
        vertices: contour.vertices.clone(),
        closed: false,
    }
}

// ---------------------------------------------------------------------------
// セグメント分割(既存の v/i/o だけから導出、新しい語彙を持ち込まない)
// ---------------------------------------------------------------------------

/// `edge` 番目の辺(`vertices[edge]` → 次の頂点)を弧長ではなく**ベジエパラメータ**
/// `t`(De Casteljau、`ops::split_bezier` — trim-path と同じ分割式)で2つに割り、
/// 新しい頂点を1つ挿入する。**新しい入力語彙ではない** — 挿入する頂点の `v`/`i`/`o`
/// は全部、既存2頂点の `v`/`i`/`o` から計算で決まる(呼び手は座標を渡さない)。
/// 形は変わらない — 既存の曲線上に点を足すだけの操作であって、
/// [`insert_vertex`](形が変わってよい・任意の頂点を差し込む)とは別の関数にした。
///
/// `t` は開区間 `(0,1)`。閉路の最終辺(`edge == n-1`)は次の頂点が `vertices[0]`
/// へ巻き戻る — 挿入後の並びが `..., 更新済み edge, 新頂点(末尾に追加),
/// 更新済み vertices[0], ...` になり、`vertices[0]` から見ればラップアラウンドの
/// 手前に新頂点が挟まる形で、輪郭の巡回順は保たれる。
pub fn split_segment(contour: &Contour, edge: usize, t: f64) -> Result<Contour, PathEditError> {
    if !(t > 0.0 && t < 1.0) {
        return Err(PathEditError::SplitParameterOutOfRange(t));
    }
    let n = contour.vertices.len();
    let edge_count = if n < 2 {
        0
    } else if contour.closed {
        n
    } else {
        n - 1
    };
    if edge >= edge_count {
        return Err(PathEditError::EdgeOutOfRange { index: edge, len: edge_count });
    }
    let next = (edge + 1) % n;
    let v0 = contour.vertices[edge];
    let v1 = contour.vertices[next];
    let (head, mid, tail) = ops::split_bezier(&v0, &v1, t);

    let mut vertices = contour.vertices.clone();
    vertices[edge] = head;
    vertices[next] = tail;
    vertices.insert(edge + 1, mid);
    Ok(Contour {
        vertices,
        closed: contour.closed,
    })
}

// ---------------------------------------------------------------------------
// store(`Value::Path`)との往復 — 具体型は引かない(module doc「依存」参照)
// ---------------------------------------------------------------------------

/// `(point, in_tangent, out_tangent)` の平坦表現。`motolii_eval::PathVertex`
/// (`next/core/motolii-eval/src/value.rs`、`values/bezier` の `v`/`i`/`o` の
/// Rust 側の正本)とフィールド順・意味とも同型 — 呼び手はタプルを分解して
/// `PathVertex { point, in_tangent, out_tangent }` へ詰め替えるだけで済む。
/// この crate は `next/core/*` を引かないので、具体型ではなくこの形で運ぶ
/// (module doc 参照)。
pub type FlatVertex = ([f64; 2], [f64; 2], [f64; 2]);

/// [`Contour`] → 平坦表現(`values/bezier` の `v`/`i`/`o`/`c` 全部)。
/// `motolii_eval::Path` 化するのは呼び手の仕事。
pub fn contour_to_flat(contour: &Contour) -> (Vec<FlatVertex>, bool) {
    let vertices = contour
        .vertices
        .iter()
        .map(|v| {
            (
                [v.point.x, v.point.y],
                [v.in_tangent.x, v.in_tangent.y],
                [v.out_tangent.x, v.out_tangent.y],
            )
        })
        .collect();
    (vertices, contour.closed)
}

/// 平坦表現 → [`Contour`]。[`contour_to_flat`] の逆。
pub fn contour_from_flat(vertices: &[FlatVertex], closed: bool) -> Contour {
    let vertices = vertices
        .iter()
        .map(|&(point, in_tangent, out_tangent)| Vertex {
            point: Point {
                x: point[0],
                y: point[1],
            },
            in_tangent: Point {
                x: in_tangent[0],
                y: in_tangent[1],
            },
            out_tangent: Point {
                x: out_tangent[0],
                y: out_tangent[1],
            },
        })
        .collect();
    Contour { vertices, closed }
}

// ---------------------------------------------------------------------------
// 内部不変量の試験。`ops::split_bezier`/`geom::bezier_point` は `pub(crate)` なので
// crate 内の単体試験でしか直接触れない(`lib.rs::geometry_tests` と同じ理由)。
// 境界・エラー経路・往復は公開 API だけで書けるので `tests/path_edit.rs` に置く。
// ---------------------------------------------------------------------------

#[cfg(test)]
mod internal_tests {
    use super::*;

    fn corner(x: f64, y: f64) -> Vertex {
        Vertex::corner(Point { x, y })
    }

    /// 曲がった辺(両端にハンドルがある)を分割すると、新しい中間頂点の座標は
    /// 元のベジエを `t` で評価した点と一致する(形が変わっていない証拠)。
    #[test]
    fn split_segment_new_vertex_lies_on_the_original_curve() {
        let v0 = Vertex {
            point: Point { x: 0.0, y: 0.0 },
            in_tangent: Point::ZERO,
            out_tangent: Point { x: 10.0, y: 0.0 },
        };
        let v1 = Vertex {
            point: Point { x: 100.0, y: 0.0 },
            in_tangent: Point { x: -10.0, y: 40.0 },
            out_tangent: Point::ZERO,
        };
        let contour = Contour {
            vertices: vec![v0, v1],
            closed: false,
        };
        let expected = bezier_point(&v0, &v1, 0.37);
        let out = split_segment(&contour, 0, 0.37).expect("有効な分割");
        assert_eq!(out.vertices.len(), 3);
        let mid = out.vertices[1].point;
        assert!(
            (mid.x - expected.x).abs() < 1e-9 && (mid.y - expected.y).abs() < 1e-9,
            "分割点が元の曲線上に無い: got {mid:?}, expected {expected:?}"
        );
    }

    /// 閉路の最終辺(`vertices[0]` へ巻き戻る辺)を分割しても巡回順が保たれる —
    /// 新頂点は末尾に追加され、`vertices[0]` は更新後のタンジェントを保つ。
    #[test]
    fn split_segment_wraps_around_closed_contour_last_edge() {
        let v0 = corner(10.0, 0.0);
        let v1 = corner(0.0, 10.0);
        let v2 = corner(-10.0, 0.0); // edge 2: v2 -> v0(ラップ)
        let contour = Contour {
            vertices: vec![v0, v1, v2],
            closed: true,
        };
        let out = split_segment(&contour, 2, 0.5).expect("有効な分割");
        assert_eq!(out.vertices.len(), 4);
        // v0(直線区間なので中点は単純な中間点)が末尾のひとつ前に挟まっている。
        let mid = out.vertices[3].point;
        assert!((mid.x - 0.0).abs() < 1e-9 && (mid.y - 0.0).abs() < 1e-9, "{mid:?}");
        // v0 自身(ラップ先)は index 0 のまま残っている。
        assert_eq!(out.vertices[0].point.x, 10.0);
        assert_eq!(out.vertices[0].point.y, 0.0);
    }
}
