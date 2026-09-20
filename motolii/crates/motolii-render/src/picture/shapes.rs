//! 時刻 t の形 — 書類の形に `shape.*` の値を重ね、つなぐ線なら 2 つの箱から解いた道に差し替える。
//! 生の形は書類の値(`view.shapes`)、重ねた姿は絵。

use crate::doc::core::RationalTime;
use crate::doc::store::{LayerId, PropertyId, StoreError, StoreView};
use crate::doc::vector::ShapeNode;

/// 時刻 t の形: 書類の形に `shape.*` の property を重ねた姿。描画・枠・出力はこれを読む。
pub fn shapes_at(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<Vec<ShapeNode>, StoreError> {
    let shapes = view.shapes(layer)?;
    if shapes.is_empty() { return Ok(shapes); }
    let get = |name: &str| PropertyId::new(name).ok().and_then(|p| view.value_at(layer, &p, t).ok().flatten());
    // つなぐ線は輪郭を 2 つの箱から解いた道に差し替える。
    crate::picture::connect::connect_shapes(view, layer, t, crate::picture::shape_props::apply(&shapes, &get))
}
