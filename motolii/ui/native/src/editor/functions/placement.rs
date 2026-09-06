use super::{atom, compose, lens, read};
use crate::doc::store::{
    property, Document, Intent, LayerId, PropertyId, RationalTime, StoreError, StoreView, Value,
};

fn vec2(
    view: &StoreView<'_>,
    layer: LayerId,
    property: &PropertyId,
    at: RationalTime,
) -> Result<[f64; 2], StoreError> {
    match read::property_value(view, layer, property, at)? {
        Some(Value::Vec2(value)) if value.iter().all(|value| value.is_finite()) => Ok(value),
        _ => Err(StoreError::Property(format!(
            "{} needs a finite two-dimensional value",
            property.name()
        ))),
    }
}

fn position(
    view: &StoreView<'_>,
    layer: LayerId,
    property: &PropertyId,
    at: RationalTime,
) -> Result<[f64; 2], StoreError> {
    if view.property_source(layer, property)?.is_none()
        && (view
            .property_source(layer, &PropertyId::new(property::POSITION_X)?)?
            .is_some()
            || view
                .property_source(layer, &PropertyId::new(property::POSITION_Y)?)?
                .is_some())
    {
        return Err(StoreError::Property(
            "Edit the separate Position axes explicitly".into(),
        ));
    }
    vec2(view, layer, property, at)
}

pub(crate) fn nudge_plan(
    doc: &Document,
    targets: &[LayerId],
    by: [f64; 2],
    at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    for value in by {
        atom::bounded(value, None).map_err(|reason| StoreError::Property(reason.into()))?;
    }
    if by == [0.0, 0.0] {
        return Ok(Vec::new());
    }
    let property = PropertyId::new(property::POSITION)?;
    compose::independent_layers(doc, targets, |doc, layer| {
        let view = doc.view().without_transients();
        let original = position(&view, layer, &property, at)?;
        let value = atom::translate2(original, by);
        for axis in value {
            atom::bounded(axis, None).map_err(|reason| StoreError::Property(reason.into()))?;
        }
        Ok(lens::place(doc, layer, &property, Value::Vec2(value), at)?
            .into_iter()
            .collect())
    })
    .map(|(intents, _)| intents)
}

pub(crate) fn anchor_position_plan(
    doc: &Document,
    layer: LayerId,
    size: [f32; 2],
    at: RationalTime,
    fractions: [f64; 2],
) -> Result<Vec<Intent>, StoreError> {
    if size.iter().any(|value| !value.is_finite() || *value < 0.0) {
        return Err(StoreError::Property(
            "The layer size must be finite and nonnegative".into(),
        ));
    }
    for fraction in fractions {
        atom::bounded(fraction, None).map_err(|reason| StoreError::Property(reason.into()))?;
        if !(0.0..=1.0).contains(&fraction) {
            return Err(StoreError::Property(
                "Anchor fractions must lie inside the layer".into(),
            ));
        }
    }
    anchor_point_plan(doc, layer, at, atom::scale_about(fractions, [0.0, 0.0], size.map(f64::from)))
}

pub(crate) fn anchor_point_plan(
    doc: &Document, layer: LayerId, at: RationalTime, next: [f64; 2],
) -> Result<Vec<Intent>, StoreError> {
    for value in next { atom::bounded(value, None).map_err(|reason| StoreError::Property(reason.into()))?; }
    let view = doc.view().without_transients();
    let anchor_property = PropertyId::new(property::ANCHOR)?;
    let position_property = PropertyId::new(property::POSITION)?;
    let anchor = vec2(&view, layer, &anchor_property, at)?;
    if anchor == next {
        return Ok(Vec::new());
    }
    let original_position = position(&view, layer, &position_property, at)?;
    let delta = glam::Vec2::from_array(next.map(|value| value as f32))
        - glam::Vec2::from_array(anchor.map(|value| value as f32));
    let compensation = view.local_transform3d(layer, at)?.transform_vector3(delta.extend(0.0));
    if !compensation.is_finite() {
        return Err(StoreError::Property(
            "The anchor transform must be finite".into(),
        ));
    }
    let moved = atom::translate2(
        original_position,
        [f64::from(compensation.x), f64::from(compensation.y)],
    );
    let mut intents = Vec::new();
    if let Some(intent) = lens::place(doc, layer, &anchor_property, Value::Vec2(next), at)? {
        intents.push(intent);
    }
    if moved != original_position {
        if let Some(intent) = lens::place(doc, layer, &position_property, Value::Vec2(moved), at)? {
            intents.push(intent);
        }
    }
    if compensation.z != 0.0 {
        let depth_property = PropertyId::new(property::POSITION_Z)?;
        let depth = match view.value_at(layer, &depth_property, at)? {
            Some(Value::F64(value)) => value,
            None => 0.0,
            _ => return Err(StoreError::Property("Depth must be numeric".into())),
        };
        if let Some(intent) = lens::place(doc, layer, &depth_property, Value::F64(depth + f64::from(compensation.z)), at)? { intents.push(intent); }
    }
    Ok(intents)
}
