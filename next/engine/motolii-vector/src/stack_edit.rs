//! `Shape.ops`(演算子スタック、`shapes/*` — trim-path / repeater / rounded-corners /
//! pucker-bloat / zig-zag / offset-path / twist)に対する編集。
//!
//! `edit.rs` が `values/bezier`(1輪郭の中身)を編集単位にしたのに対し、ここは
//! **スタック そのもの**(`Vec<ShapeOp>`)を編集単位にする。地図
//! (`reference/lottie-coverage.tsv` の `shapes/` グループ)を見ると、この波の対象
//! 7つの modifier(`trim-path`/`repeater`/`rounded-corners`/`pucker-bloat`/
//! `zig-zag`/`offset-path`/`twist`)は**全部 採用済**で、パラメータは既に
//! [`crate::OpKind`] の enum フィールドとして実在する(`merge` だけ不採用 — 裁定74、
//! fill-rule が中マドを賄うので path boolean を持たない)。つまり**新しい語彙は
//! 無い** — この波が足すのは「スタックへの編集操作」という単位だけである。
//!
//! # 「パラメータ変更」に専用関数を足さない理由
//!
//! [`crate::OpKind`] は plain data(`pub` フィールドの enum)で、`Fill`/`Stroke`
//! と同じく**この crate のどこにも専用 setter が無い**(呼び手が構造体ごと
//! 差し替えるか、パターンマッチでフィールドに触る)。`edit.rs` が
//! `move_vertex`/`set_handles` を分けたのは `v`/`i`/`o` が**同じ配列要素の中で
//! 独立に変わる**フィールド群だったからで、`OpKind` のバリアント内フィールド
//! (例えば `TrimPath.start`/`.end`)には同種の分割理由が無い —
//! 1つの意味段を丸ごと差し替える [`set_kind`] が `set_handles` と同じ粒度
//! (「配列要素の中の1フィールドを差し替える」)にあたる。
//!
//! # ここが足すもの — スタック(`Vec`)の構造編集
//!
//! `Vec::insert`/`remove` は範囲外 index で**panic** する。この crate の他の
//! エラー型([`crate::VectorError`]・[`crate::edit::PathEditError`])と同じ姿勢
//! (裁定37「壊れた入力を黙って丸めない」)を保つため、範囲外 index は
//! [`StackEditError`] を返す:
//!
//! | 関数 | Lottie 語彙 | 触るもの |
//! |---|---|---|
//! | [`insert_op`] | 段の**適用** — `shapes` 配列に1段挿す | 挿入位置を含め、**それより後ろの段の適用順が繰り下がる**(裁定73 のスタックは順序付き) |
//! | [`remove_op`] | 段を外す | 逆に繰り上がる |
//! | [`move_op`] | **順序**(`shapes` 配列の順序が適用順) | 位置を動かすだけ。段の中身(`kind`/`hidden`)は変えない |
//! | [`set_kind`] | 段の**パラメータ変更** | `kind` だけ差し替え。`hidden` は不変 |
//! | [`set_hidden`] | `graphic-element.hd` | `hidden` だけ差し替え。`kind` は不変 |
//!
//! 型そのものはすでに順序を表している——`Vec<ShapeOp>` は挿入順を保持し、
//! `resolve`(`lib.rs`)は `for op in &shape.ops` で先頭から順に畳む。
//! Lottie の `group.it`(兄弟順スコープ)は採らなかった(裁定66 と同型の理由、
//! `lib.rs` module doc 参照)ので、**この crate に「暗黙に効く隣接段」は無い** —
//! [`move_op`] で動かした段は、新しい位置から見て文字通りの適用順に従うだけである。

use crate::{OpKind, Shape, ShapeOp};

/// スタック編集の失敗。範囲外 index を**panic ではなく明示エラー**にする
/// (`edit.rs::PathEditError` と同じ姿勢・裁定37)。
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum StackEditError {
    /// [`insert_op`] に渡した挿入位置が `0..=len` の外。
    #[error("insert index {index} is out of range (stack has {len} entries)")]
    InsertOutOfRange { index: usize, len: usize },
    /// [`remove_op`]/[`move_op`]/[`set_kind`]/[`set_hidden`] に渡した index が
    /// `0..len` の外(空スタックなら常にこれ)。
    #[error("op index {index} is out of range (stack has {len} entries)")]
    OpOutOfRange { index: usize, len: usize },
}

/// `at` に段を挿す(適用)。`at == ops.len()` は末尾への追加まで許す — 挿入位置
/// として自然な境界で、`edit.rs::insert_vertex` と同じ考え方([`Shape::new`] が
/// 作る空スタックへの最初の1段もこれで作れる)。
pub fn insert_op(ops: &[ShapeOp], at: usize, op: ShapeOp) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at > len {
        return Err(StackEditError::InsertOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out.insert(at, op);
    Ok(out)
}

/// `at` の段を外す。**0段まで許す** — 空スタックは「無加工のパス」を意味するだけ
/// で、`resolve`(`lib.rs`)は空 `ops` を前提に書かれている([`Shape::new`] の
/// 既定がまさにこれ)。
pub fn remove_op(ops: &[ShapeOp], at: usize) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at >= len {
        return Err(StackEditError::OpOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out.remove(at);
    Ok(out)
}

/// `from` の段を `to` の位置へ動かす(順序 — `shapes` 配列の順序が適用順)。
/// `kind`/`hidden` はどちらも変えない。`from == to` は恒等(冪等)。
///
/// `to` は「動かした後にその段が来てほしい最終位置」——`Vec::remove` してから
/// `insert` する2段操作の見た目上の位置ずれ(remove 後に後続が繰り上がる)を
/// 呼び手に見せない。`to` の範囲は `0..len`(移動先が既存の段の位置であって、
/// 末尾の**後ろ**を指す `len` は [`insert_op`] の仕事であって `move_op` の仕事
/// ではない — 段の**個数**を変えない操作だから)。
pub fn move_op(ops: &[ShapeOp], from: usize, to: usize) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if from >= len {
        return Err(StackEditError::OpOutOfRange { index: from, len });
    }
    if to >= len {
        return Err(StackEditError::OpOutOfRange { index: to, len });
    }
    if from == to {
        return Ok(ops.to_vec());
    }
    let mut out = ops.to_vec();
    let moved = out.remove(from);
    out.insert(to, moved);
    Ok(out)
}

/// `at` の段の `kind` だけを差し替える(パラメータ変更)。`hidden` は不変。
/// 新しい `kind` が古い `kind` と違う variant でも構わない(段を丸ごと別の
/// modifier へ差し替えるのも「パラメータ変更」の極端な1点に過ぎない——`OpKind`
/// は1つの閉じた enum で、`resolve` はどの variant も同じ `match` で畳む)。
pub fn set_kind(ops: &[ShapeOp], at: usize, kind: OpKind) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at >= len {
        return Err(StackEditError::OpOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out[at].kind = kind;
    Ok(out)
}

/// `at` の段の `hidden`(`graphic-element.hd`)だけを差し替える。`kind` は不変。
pub fn set_hidden(ops: &[ShapeOp], at: usize, hidden: bool) -> Result<Vec<ShapeOp>, StackEditError> {
    let len = ops.len();
    if at >= len {
        return Err(StackEditError::OpOutOfRange { index: at, len });
    }
    let mut out = ops.to_vec();
    out[at].hidden = hidden;
    Ok(out)
}

/// [`Shape::ops`] に `f` を適用した新しい [`Shape`] を返す橋渡し。呼び手が
/// `shape.ops = insert_op(&shape.ops, ...)?` と書く手間を1つにまとめるだけで、
/// 上の5関数以上のことはしない(`edit.rs::with_contour` と同じ形の糖衣)。
pub fn with_ops<F>(shape: &Shape, f: F) -> Result<Shape, StackEditError>
where
    F: FnOnce(&[ShapeOp]) -> Result<Vec<ShapeOp>, StackEditError>,
{
    let ops = f(&shape.ops)?;
    Ok(Shape {
        ops,
        ..shape.clone()
    })
}
