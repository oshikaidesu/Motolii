//! MASK の膨張量 component。
//!
//! `Mask` の mode/inverted と分け、動く値である `mask.{id}.expansion` の投影だけを
//! 持つ。書き込みは既存の `TransformField` → `SetTrack` 経路を再利用する。

use motolii_core::{Fps, RationalTime};
use motolii_shell_state::Session;
use motolii_store::{LayerId, MaskId, PropertyId, StoreError, StoreView, Value};

use crate::projection::{ComponentSlot, KeyCellProjection, RowValue, TransformRowProjection};
use crate::transform::{field_decimals, has_real_keys, key_cell_state, KeyRow, TransformField};

/* motolii-component
id = "inspector.mask_expansion"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["project_mask_expansion", "commit_inspector_field"]
meaning = ["MaskExpansionInput"]
evaluation = ["project_mask_expansion", "commit_inspector_field"]
render = ["transform_row"]
observable = ["mask_expansion_changes_coverage"]
*/

/// Inspector の入力種別。実際の編集文法は既存の `TransformField` を共有する。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskExpansionInput {
    Value,
}

/// mask の expansion 行。既存の値セル・Key 列の文法へ接続するため、独自の行型や
/// 書き込み口を作らず `TransformRowProjection` を返す。
pub(crate) fn project_mask_expansion(
    store: &StoreView<'_>,
    layer: LayerId,
    mask: MaskId,
    t: RationalTime,
    session: &Session,
    fps: Fps,
) -> Result<TransformRowProjection, StoreError> {
    let property = PropertyId::mask_expansion(mask);
    let track = store.track(layer, &property)?;
    let value = match store.value_at(layer, &property, t)? {
        Some(Value::F64(value)) => value,
        _ => 0.0,
    };
    Ok(TransformRowProjection {
        label: "Expansion",
        value: RowValue::Scalar(ComponentSlot {
            axis: "Expansion",
            present: true,
            value,
            editable: true,
            keyed: has_real_keys(track.as_ref()),
            field: Some(TransformField::MaskExpansion(mask)),
        }),
        decimals: field_decimals(TransformField::MaskExpansion(mask)),
        key: KeyCellProjection {
            row: KeyRow::MaskExpansion(mask),
            state: key_cell_state(track.as_ref(), session.playhead, fps),
        },
    })
}
