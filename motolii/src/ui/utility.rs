use crate::doc::store::{Document, LayerId, RationalTime, StoreError};
use std::sync::{Arc, Mutex};

pub(super) fn move_anchor_in_bounds(
    doc: &Arc<Mutex<Document>>, layer: LayerId,
    bounds: crate::render::media::SpatialBounds, at: RationalTime, fx: f64, fy: f64,
) -> Result<(), StoreError> {
    let size = bounds.size_xy();
    let point = [bounds.min[0] as f64 + fx * size[0] as f64, bounds.min[1] as f64 + fy * size[1] as f64];
    let mut doc = doc.lock().unwrap();
    let intents = crate::ui::functions::placement::anchor_point_plan(&doc, layer, at, point)?;
    doc.apply_all(intents)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{property, Intent, PropertyId, Value};

    #[test]
    fn offset_bounds_anchor_keeps_a_tilted_layer_in_place_with_one_undo() {
        let mut document = Document::new();
        let layer = LayerId(1);
        document.apply(Intent::AddLayer(layer)).unwrap();
        for (name, value) in [
            (property::ANCHOR, Value::Vec2([0.0, 0.0])),
            (property::POSITION, Value::Vec2([40.0, 50.0])),
            (property::POSITION_Z, Value::F64(12.0)),
            (property::SCALE, Value::Vec2([2.0, 0.5])),
            (property::ROTATION_X, Value::F64(35.0)),
            (property::ROTATION_Y, Value::F64(20.0)),
        ] {
            document.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
        }
        let before = document.view().local_transform3d(layer, RationalTime::ZERO).unwrap();
        let history = document.history_depth().0;
        let doc = Arc::new(Mutex::new(document));
        move_anchor_in_bounds(&doc, layer, crate::render::media::SpatialBounds {
            min: [100.0, 200.0, -5.0], max: [300.0, 400.0, 5.0],
        }, RationalTime::ZERO, 0.5, 0.5).unwrap();
        let mut document = doc.lock().unwrap();
        let after = document.view().local_transform3d(layer, RationalTime::ZERO).unwrap();
        for point in [glam::Vec3::ZERO, glam::vec3(100.0, 200.0, 5.0), glam::vec3(300.0, 400.0, -5.0)] {
            assert!(before.transform_point3(point).distance(after.transform_point3(point)) < 0.0001);
        }
        assert_eq!(document.view().value_at(layer, &PropertyId::new(property::ANCHOR).unwrap(), RationalTime::ZERO).unwrap(), Some(Value::Vec2([200.0, 300.0])));
        assert_eq!(document.history_depth().0, history + 1);
        assert!(document.undo());
        assert_eq!(document.view().local_transform3d(layer, RationalTime::ZERO).unwrap(), before);
    }
}
