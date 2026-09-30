use crate::document::{Document, Intent};
use motolii_doc::core::RationalTime;
use motolii_doc::eval::{KeyframeTrack, Value};

use motolii_doc::store::view::StoreView;
use motolii_doc::store::{LayerAttrsPatch, LayerSource, StoreError};

use motolii_doc::store::{LayerId, PropertyId};

use super::{bake_child_local, outermost_present, read_vec2, TRANSFORM_PROPERTIES};

impl Document {
    pub fn move_layers(
        &mut self,
        layers: &[LayerId],
        target: Option<LayerId>,
        placement: &str,
        at: RationalTime,
    ) -> Result<Vec<LayerId>, StoreError> {
        let (intents, roots) = self.move_layer_intents(layers, target, placement, at)?;
        self.apply_all(intents)?;
        Ok(roots)
    }

    /// 並べ替えの intents を決めるだけで書かない。作る→並べるを 1 手にする側
    /// (`apply_then`)がこれを後段に使う。
    pub fn move_layer_intents(
        &self,
        layers: &[LayerId],
        target: Option<LayerId>,
        placement: &str,
        at: RationalTime,
    ) -> Result<(Vec<Intent>, Vec<LayerId>), StoreError> {
        use std::collections::{BTreeMap, HashSet};
        let view = self.view().without_transients();
        for &layer in layers {
            move_editable(&view, layer)?;
        }
        let mut roots = outermost_present(&view, layers)?;
        if roots.is_empty() {
            return Err(StoreError::Property("Select layers to move".into()));
        }
        if !matches!(placement, "before" | "after" | "inside" | "rootEnd") {
            return Err(StoreError::Property("Unknown layer drop placement".into()));
        }
        if placement == "rootEnd" && target.is_some() {
            return Err(StoreError::Property("Root end has no target layer".into()));
        }
        if placement != "rootEnd" {
            let target =
                target.ok_or_else(|| StoreError::Property("A drop target is required".into()))?;
            move_editable(&view, target)?;
            if placement == "inside"
                && view
                    .meta(target)?
                    .is_none_or(|m| m.source != LayerSource::Group)
            {
                return Err(StoreError::Property(
                    "Only a Group can contain a layer drop".into(),
                ));
            }
            if placement != "inside" && roots.contains(&target) {
                return Ok((Vec::new(), roots));
            }
        }
        let new_parent = match placement {
            "inside" => target,
            "before" | "after" => view.attrs(target.unwrap())?.and_then(|a| a.parent),
            _ => None,
        };
        if let Some(parent) = new_parent {
            move_editable(&view, parent)?;
        }
        let moving: HashSet<_> = roots.iter().copied().collect();
        let mut ancestor = new_parent;
        let mut seen = HashSet::new();
        while let Some(parent) = ancestor {
            if !seen.insert(parent) || moving.contains(&parent) {
                return Err(StoreError::Property(
                    "A layer cannot be moved into its own descendant".into(),
                ));
            }
            ancestor = view.attrs(parent)?.and_then(|a| a.parent);
        }

        let mut siblings: BTreeMap<Option<LayerId>, Vec<LayerId>> = BTreeMap::new();
        for layer in view.layers() {
            siblings
                .entry(view.attrs(layer)?.and_then(|a| a.parent))
                .or_default()
                .push(layer);
        }
        for list in siblings.values_mut() {
            list.sort_by_key(|id| {
                std::cmp::Reverse((view.meta(*id).ok().flatten().map_or(0, |m| m.order), *id))
            });
        }
        let mut display_order = BTreeMap::new();
        let mut pending: Vec<_> = siblings
            .get(&None)
            .into_iter()
            .flatten()
            .rev()
            .copied()
            .collect();
        while let Some(layer) = pending.pop() {
            if display_order.contains_key(&layer) {
                continue;
            }
            display_order.insert(layer, display_order.len());
            pending.extend(
                siblings
                    .get(&Some(layer))
                    .into_iter()
                    .flatten()
                    .rev()
                    .copied(),
            );
        }
        roots.sort_by_key(|id| display_order.get(id).copied().unwrap_or(usize::MAX));
        let original = siblings.clone();
        let mut scopes = std::collections::BTreeSet::from([new_parent]);
        let mut intents = Vec::new();
        for &layer in &roots {
            let old_parent = view.attrs(layer)?.and_then(|a| a.parent);
            if let Some(parent) = old_parent {
                move_editable(&view, parent)?;
            }
            scopes.insert(old_parent);
            if old_parent != new_parent {
                intents.extend(move_parent_compensation(
                    &view,
                    self.geometry(), layer, old_parent, new_parent, at,
                )?);
                intents.push(Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch {
                        parent: Some(new_parent),
                        ..Default::default()
                    },
                });
            }
        }
        for list in siblings.values_mut() {
            list.retain(|id| !moving.contains(id));
        }
        let destination = siblings.entry(new_parent).or_default();
        let index = match placement {
            "inside" => 0,
            "rootEnd" => destination.len(),
            _ => {
                let index = destination
                    .iter()
                    .position(|id| Some(*id) == target)
                    .ok_or_else(|| {
                        StoreError::Property("Drop target is not in the destination scope".into())
                    })?;
                index + usize::from(placement == "after")
            }
        };
        destination.splice(index..index, roots.iter().copied());
        for scope in scopes {
            let list = siblings.get(&scope).cloned().unwrap_or_default();
            if original.get(&scope).cloned().unwrap_or_default() == list {
                continue;
            }
            if list.len() > (u16::MAX as usize) + 1 {
                return Err(StoreError::Property(
                    "Too many sibling layers to reorder".into(),
                ));
            }
            let top = list.len().saturating_sub(1).min(i16::MAX as usize) as i32;
            for (index, layer) in list.into_iter().enumerate() {
                let order = (top - index as i32) as i16;
                if view.meta(layer)?.is_none_or(|m| m.order != order) {
                    move_editable(&view, layer)?;
                    intents.push(Intent::SetOrder { layer, order });
                }
            }
        }
        Ok((intents, roots))
    }
}

fn move_editable(view: &StoreView<'_>, layer: LayerId) -> Result<(), StoreError> {
    if !view.has_layer(layer) {
        return Err(StoreError::Property(format!(
            "Layer {} no longer exists",
            layer.0
        )));
    }
    crate::document::validate::check_not_locked(view, layer)?;
    crate::document::validate::check_not_frozen(view, layer)?;
    if view.attrs(layer)?.unwrap_or_default().frozen {
        return Err(StoreError::Property(format!("Layer {} is frozen", layer.0)));
    }
    Ok(())
}

fn refuse_split_position(view: &StoreView<'_>, layer: LayerId) -> Result<(), StoreError> {
    for name in [
        motolii_doc::store::property::POSITION_X,
        motolii_doc::store::property::POSITION_Y,
    ] {
        if view
            .property_source(layer, &PropertyId::new(name)?)?
            .is_some()
        {
            return Err(StoreError::Property(
                "Cannot change coordinate systems with separate Position axes".into(),
            ));
        }
    }
    Ok(())
}

fn move_static_transform(view: &StoreView<'_>, layer: LayerId) -> Result<(), StoreError> {
    for name in TRANSFORM_PROPERTIES.iter().copied().chain([
        motolii_doc::store::property::POSITION_Z,
        motolii_doc::store::property::ROTATION_X,
        motolii_doc::store::property::ROTATION_Y,
        motolii_doc::store::property::SCALE_Z,
    ]) {
        let property = PropertyId::new(name)?;
        if let Some(source) = view.property_source(layer, &property)? {
            if !source.modulators.is_empty()
                || matches!(source.base, Some(motolii_doc::store::PropertyBase::Slot(_)))
            {
                return Err(StoreError::Property(format!(
                    "Cannot preserve driven transform {name} while changing parent"
                )));
            }
        }
        if view
            .track(layer, &property)?
            .is_some_and(|track| track.keys().len() > 1)
        {
            return Err(StoreError::Property(format!(
                "Cannot change coordinate systems for animated transform {name}; keep its parent"
            )));
        }
    }
    Ok(())
}

fn move_planar(matrix: glam::Affine3A) -> Result<glam::Affine2, StoreError> {
    let x = glam::Vec3::from(matrix.matrix3.x_axis);
    let y = glam::Vec3::from(matrix.matrix3.y_axis);
    let z = glam::Vec3::from(matrix.matrix3.z_axis);
    let p = glam::Vec3::from(matrix.translation);
    if !matrix.is_finite()
        || x.z.abs() > 1e-6
        || y.z.abs() > 1e-6
        || (z - glam::Vec3::Z).length() > 1e-6
        || p.z.abs() > 1e-6
    {
        return Err(StoreError::Property("Cannot preserve this spatial transform while changing parent; move within its current group".into()));
    }
    Ok(glam::Affine2::from_mat2_translation(
        glam::Mat2::from_cols(x.truncate(), y.truncate()),
        p.truncate(),
    ))
}

pub(in crate::document) fn move_translation_values(
    view: &StoreView<'_>,
    layer: LayerId,
    delta: [f64; 3],
) -> Result<Vec<Intent>, StoreError> {
    use motolii_doc::store::property;
    let position = PropertyId::new(property::POSITION)?;
    let split = if view.property_source(layer, &position)?.is_none() {
        view.property_source(layer, &PropertyId::new(property::POSITION_X)?)?
            .is_some()
            || view
                .property_source(layer, &PropertyId::new(property::POSITION_Y)?)?
                .is_some()
    } else {
        false
    };
    let mut offsets = Vec::new();
    if split {
        for (name, delta) in [
            (property::POSITION_X, delta[0]),
            (property::POSITION_Y, delta[1]),
        ] {
            if delta != 0.0 {
                offsets.push((PropertyId::new(name)?, Value::F64(delta)));
            }
        }
    } else if delta[0] != 0.0 || delta[1] != 0.0 {
        offsets.push((position, Value::Vec2([delta[0], delta[1]])));
    }
    if delta[2] != 0.0 {
        offsets.push((PropertyId::new(property::POSITION_Z)?, Value::F64(delta[2])));
    }
    let mut intents = Vec::new();
    for (property, offset) in offsets {
        if let Some(reason) = crate::document::edit::property_write_rejection(&view, layer, &property)? {
            return Err(StoreError::Property(reason.into()));
        }
        let add = |value: Value| -> Result<Value, StoreError> {
            match (value, &offset) {
                (Value::F64(v), Value::F64(d)) if (v + d).is_finite() => Ok(Value::F64(v + d)),
                (Value::Vec2(v), Value::Vec2(d))
                    if (v[0] + d[0]).is_finite() && (v[1] + d[1]).is_finite() =>
                {
                    Ok(Value::Vec2([v[0] + d[0], v[1] + d[1]]))
                }
                _ => Err(StoreError::Property(
                    "Position offset needs finite matching values".into(),
                )),
            }
        };
        let intent = if let Some(track) = view.track(layer, &property)? {
            let mut keys = track.keys().to_vec();
            for key in &mut keys {
                key.value = add(key.value.clone())?;
            }
            if keys.is_empty() {
                Intent::SetConstant {
                    layer,
                    property,
                    value: offset,
                }
            } else {
                Intent::SetTrack {
                    layer,
                    property,
                    track: KeyframeTrack::try_from_keys(keys)
                        .map_err(|e| StoreError::Property(e.to_string()))?,
                }
            }
        } else {
            let zero = match offset {
                Value::Vec2(_) => Value::Vec2([0.0, 0.0]),
                _ => Value::F64(0.0),
            };
            let value = add(view
                .value_at(layer, &property, RationalTime::ZERO)?
                .unwrap_or(zero))?;
            Intent::SetConstant {
                layer,
                property,
                value,
            }
        };
        intents.push(intent);
    }
    Ok(intents)
}

fn move_parent_compensation(
    view: &StoreView<'_>,
    geometry: motolii_doc::store::geometry::Geometry,
    layer: LayerId,
    old_parent: Option<LayerId>,
    new_parent: Option<LayerId>,
    at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    let world = |parent: Option<LayerId>| match parent {
        Some(parent) => (geometry.world)(view, parent, at),
        None => Ok(glam::Affine3A::IDENTITY),
    };
    let old_world = world(old_parent)?;
    let new_world = world(new_parent)?;
    if old_world
        .to_cols_array()
        .into_iter()
        .zip(new_world.to_cols_array())
        .all(|(a, b)| (a - b).abs() <= 1e-6)
    {
        return Ok(Vec::new());
    }
    if !new_world.is_finite() || new_world.matrix3.determinant().abs() < 1e-8 {
        return Err(StoreError::Property(
            "Destination parent transform is singular".into(),
        ));
    }
    let correction = new_world.inverse() * old_world;
    if correction.is_finite()
        && correction
            .matrix3
            .to_cols_array()
            .into_iter()
            .zip(glam::Mat3A::IDENTITY.to_cols_array())
            .all(|(a, b)| (a - b).abs() <= 1e-6)
    {
        return move_translation_values(
            view,
            layer,
            glam::Vec3::from(correction.translation)
                .to_array()
                .map(f64::from),
        );
    }
    move_static_transform(view, layer)?;
    refuse_split_position(view, layer)?;
    let correction = move_planar(correction)?;
    let local = move_planar((geometry.local3d)(view, layer, at)?)?;
    let anchor = read_vec2(
        view,
        layer,
        motolii_doc::store::property::ANCHOR,
        [0.0, 0.0],
        at,
    )?;
    let baked = bake_child_local(correction, local, anchor);
    let rebuilt = motolii_doc::core::LayerPlacement::from_transform(
        anchor,
        baked.position.map(|v| v as f32),
        baked.scale.map(|v| v as f32),
        baked.rotation_degrees as f32,
        baked.skew_degrees as f32,
        baked.skew_axis_degrees as f32,
    );
    if !(correction * local)
        .to_cols_array()
        .into_iter()
        .zip(rebuilt.to_cols_array())
        .all(|(a, b)| (a - b).abs() <= 1e-3)
    {
        return Err(StoreError::Property(
            "Parent transform cannot be preserved without changing the layer".into(),
        ));
    }
    write_transform_values(
        view,
        layer,
        at,
        [
            (motolii_doc::store::property::POSITION, Value::Vec2(baked.position)),
            (motolii_doc::store::property::SCALE, Value::Vec2(baked.scale)),
            (motolii_doc::store::property::ROTATION, Value::F64(baked.rotation_degrees)),
            (motolii_doc::store::property::SKEW, Value::F64(baked.skew_degrees)),
            (motolii_doc::store::property::SKEW_AXIS, Value::F64(baked.skew_axis_degrees)),
        ],
    )
}

/// Writes still transform values, keeping each property's key layout so Undo and the
/// Timeline see the same rows as before.
fn write_transform_values(
    view: &StoreView<'_>,
    layer: LayerId,
    at: RationalTime,
    values: impl IntoIterator<Item = (&'static str, Value)>,
) -> Result<Vec<Intent>, StoreError> {
    let mut intents = Vec::new();
    for (name, value) in values {
        let property = PropertyId::new(name)?;
        let current = view
            .value_at(layer, &property, at)?
            .or(crate::document::edit::default_value(&view, layer, &property)?);
        if current.as_ref() == Some(&value) {
            continue;
        }
        let intent = if let Some(track) = view.track(layer, &property)? {
            let mut keys = track.keys().to_vec();
            for key in &mut keys {
                key.value = value.clone();
            }
            if keys.is_empty() {
                Intent::SetConstant {
                    layer,
                    property,
                    value,
                }
            } else {
                Intent::SetTrack {
                    layer,
                    property,
                    track: KeyframeTrack::try_from_keys(keys)
                        .map_err(|e| StoreError::Property(e.to_string()))?,
                }
            }
        } else {
            Intent::SetConstant {
                layer,
                property,
                value,
            }
        };
        intents.push(intent);
    }
    Ok(intents)
}
