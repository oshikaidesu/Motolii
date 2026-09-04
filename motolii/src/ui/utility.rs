use crate::doc::store::{Document, LayerId, RationalTime, StoreError};
use std::sync::{Arc, Mutex};

/// This instantaneous command fixes its time and baseline while preparing both writes.
pub(super) fn move_anchor(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    size: [f32; 2],
    at: RationalTime,
    fx: f64,
    fy: f64,
) -> Result<(), StoreError> {
    let mut doc = doc.lock().unwrap();
    let intents =
        crate::ui::functions::placement::anchor_position_plan(&doc, layer, size, at, [fx, fy])?;
    doc.apply_all(intents)
}
