use crate::doc::core::RationalTime;
use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Value};

use crate::doc::store::view::StoreView;
use crate::doc::store::{LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming, StoreError};

use super::{Document, Intent, LayerId, PropertyId};

impl Document {
    pub fn group_layers(&mut self, layers: &[LayerId]) -> Result<Option<LayerId>, StoreError> {
        let view = self.view();
        let roots = outermost_present(&view, layers)?;
        if roots.is_empty() {
            return Ok(None);
        }

        let group_id = LayerId(view.next_layer_id());
        let comp_duration = view
            .composition()?
            .map(|composition| composition.duration_frames)
            .unwrap_or(0);
        let order = view.meta(roots[0])?.map(|meta| meta.order).ok_or_else(|| {
            StoreError::Property(format!("layer {} has no placement", roots[0].0))
        })?;
        let common_parent = common_parent(&view, &roots)?;

        let mut intents = Vec::with_capacity(roots.len() + 3);
        intents.push(Intent::AddLayer(group_id));
        intents.push(Intent::SetMeta {
            layer: group_id,
            meta: LayerMeta {
                source: LayerSource::Group,
                order,
                timing: LayerTiming::place(0, None, comp_duration),
            },
        });
        intents.push(Intent::SetAttrs {
            layer: group_id,
            patch: LayerAttrsPatch {
                name: Some("Group".to_owned()),
                parent: Some(common_parent),
                ..Default::default()
            },
        });
        for &child in &roots {
            intents.push(Intent::SetAttrs {
                layer: child,
                patch: LayerAttrsPatch {
                    parent: Some(Some(group_id)),
                    ..Default::default()
                },
            });
        }

        self.apply_all(intents)?;
        Ok(Some(group_id))
    }

    pub fn ungroup_layers(&mut self, groups: &[LayerId]) -> Result<Vec<LayerId>, StoreError> {
        let t = RationalTime::ZERO;
        let view = self.view();
        let present = view.layers();
        let mut group_candidates = Vec::new();
        for &group in groups {
            if view
                .meta(group)?
                .is_some_and(|meta| meta.source == LayerSource::Group)
            {
                group_candidates.push(group);
            }
        }
        let groups = outermost_present(&view, &group_candidates)?;
        if groups.is_empty() {
            return Ok(Vec::new());
        }

        let mut intents = Vec::new();
        let mut released = Vec::new();

        for &group in &groups {
            if view.attrs(group)?.unwrap_or_default().frozen {
                return Err(StoreError::Property(format!(
                    "layer {} は凍結中(frozen)なので ungroup できない \
                     (先に unfreeze すること)",
                    group.0
                )));
            }
            reject_animated_transform(&view, group, "group")?;
            let new_parent = view.attrs(group)?.and_then(|attrs| attrs.parent);
            let group_local = (self.geometry.local)(&view, group, t)?;
            let identity = affine2_is_identity(group_local);

            for &child in &present {
                let Some(child_attrs) = view.attrs(child)? else {
                    continue;
                };
                if child_attrs.parent != Some(group) {
                    continue;
                }

                if !identity {
                    reject_animated_transform(&view, child, "child")?;
                }
                intents.push(Intent::SetAttrs {
                    layer: child,
                    patch: LayerAttrsPatch {
                        parent: Some(new_parent),
                        ..Default::default()
                    },
                });

                if !identity {
                    let anchor = read_vec2(
                        &view,
                        child,
                        crate::doc::store::property::ANCHOR,
                        [0.0, 0.0],
                        t,
                    )?;
                    let child_local = (self.geometry.local)(&view, child, t)?;
                    let baked = bake_child_local(group_local, child_local, anchor);
                    intents.extend(baked.into_intents(child)?);
                }

                released.push(child);
            }

            intents.push(Intent::RemoveLayer(group));
        }

        self.apply_all(intents)?;
        Ok(released)
    }
}

fn outermost_present(view: &StoreView<'_>, layers: &[LayerId]) -> Result<Vec<LayerId>, StoreError> {
    let present: std::collections::HashSet<_> = view.layers().into_iter().collect();
    let mut unique = Vec::new();
    let mut selected = std::collections::HashSet::new();
    for &layer in layers {
        if present.contains(&layer) && selected.insert(layer) {
            unique.push(layer);
        }
    }

    let mut roots = Vec::new();
    for layer in unique {
        let mut parent = view.attrs(layer)?.and_then(|attrs| attrs.parent);
        let mut seen = std::collections::HashSet::new();
        let mut has_selected_ancestor = false;
        while let Some(candidate) = parent {
            if !seen.insert(candidate) {
                break;
            }
            if selected.contains(&candidate) {
                has_selected_ancestor = true;
                break;
            }
            parent = view.attrs(candidate)?.and_then(|attrs| attrs.parent);
        }
        if !has_selected_ancestor {
            roots.push(layer);
        }
    }
    Ok(roots)
}

fn common_parent(view: &StoreView<'_>, layers: &[LayerId]) -> Result<Option<LayerId>, StoreError> {
    let Some((&first, rest)) = layers.split_first() else {
        return Ok(None);
    };
    let parent = view.attrs(first)?.and_then(|attrs| attrs.parent);
    for &layer in rest {
        if view.attrs(layer)?.and_then(|attrs| attrs.parent) != parent {
            return Err(StoreError::Property(
                "cannot group layers from different parents without changing their world transforms"
                    .to_owned(),
            ));
        }
    }
    Ok(parent)
}

const TRANSFORM_PROPERTIES: &[&str] = &[
    crate::doc::store::property::ANCHOR,
    crate::doc::store::property::POSITION,
    crate::doc::store::property::POSITION_X,
    crate::doc::store::property::POSITION_Y,
    crate::doc::store::property::SCALE,
    crate::doc::store::property::ROTATION,
    crate::doc::store::property::SKEW,
    crate::doc::store::property::SKEW_AXIS,
];

fn reject_animated_transform(
    view: &StoreView<'_>,
    layer: LayerId,
    role: &str,
) -> Result<(), StoreError> {
    for &name in TRANSFORM_PROPERTIES {
        let property = PropertyId::new(name)?;
        if view
            .track(layer, &property)?
            .is_some_and(|track| track.keys().len() > 1)
        {
            return Err(StoreError::Property(format!(
                "cannot ungroup: {role} layer {} has animated `{name}`; keep the group or remove the animation first",
                layer.0
            )));
        }
    }
    Ok(())
}

pub(super) struct BakedChildTransform {
    pub(super) position: [f64; 2],
    pub(super) rotation_degrees: f64,
    pub(super) scale: [f64; 2],
    pub(super) skew_degrees: f64,
    pub(super) skew_axis_degrees: f64,
}

impl BakedChildTransform {
    fn into_intents(self, layer: LayerId) -> Result<Vec<Intent>, StoreError> {
        Ok(vec![
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::POSITION)?,
                track: still(Value::Vec2(self.position)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::ROTATION)?,
                track: still(Value::F64(self.rotation_degrees)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::SCALE)?,
                track: still(Value::Vec2(self.scale)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::SKEW)?,
                track: still(Value::F64(self.skew_degrees)),
            },
            Intent::SetTrack {
                layer,
                property: PropertyId::new(crate::doc::store::property::SKEW_AXIS)?,
                track: still(Value::F64(self.skew_axis_degrees)),
            },
        ])
    }
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

pub(super) fn bake_child_local(
    group_local: glam::Affine2,
    child_local: glam::Affine2,
    anchor: [f32; 2],
) -> BakedChildTransform {
    use glam::{Affine2, Mat2, Vec2};

    let x = child_local * Affine2::from_translation(Vec2::new(anchor[0], anchor[1]));
    let x_prime = group_local * x;

    let position = x_prime.translation;
    let linear = x_prime.matrix2;

    let col0 = linear.x_axis;
    let sx = col0.length();
    let theta = if sx > 1e-6 { col0.y.atan2(col0.x) } else { 0.0 };

    let rest = Mat2::from_angle(-theta) * linear;
    let sy = rest.y_axis.y;
    let skew_tan = if sy.abs() > 1e-6 {
        rest.y_axis.x / sy
    } else {
        0.0
    };

    BakedChildTransform {
        position: [position.x as f64, position.y as f64],
        rotation_degrees: theta.to_degrees() as f64,
        scale: [sx as f64, sy as f64],
        skew_degrees: -skew_tan.atan().to_degrees() as f64,
        skew_axis_degrees: 0.0,
    }
}

fn affine2_is_identity(m: glam::Affine2) -> bool {
    const EPS: f32 = 1e-4;
    m.translation.length() < EPS
        && (m.matrix2.x_axis - glam::Vec2::X).length() < EPS
        && (m.matrix2.y_axis - glam::Vec2::Y).length() < EPS
}

pub(super) fn read_vec2(
    view: &StoreView,
    layer: LayerId,
    name: &str,
    default: [f32; 2],
    t: RationalTime,
) -> Result<[f32; 2], StoreError> {
    let property = PropertyId::new(name)?;
    match view.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
        Some(other) => Err(StoreError::Property(format!(
            "{name} に2成分でない値が入っている: {other:?}"
        ))),
        None => Ok(default),
    }
}

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
    super::validate::check_not_locked(view, layer)?;
    super::validate::check_not_frozen(view, layer)?;
    if view.attrs(layer)?.unwrap_or_default().frozen {
        return Err(StoreError::Property(format!("Layer {} is frozen", layer.0)));
    }
    Ok(())
}

pub(super) fn refuse_split_position(view: &StoreView<'_>, layer: LayerId) -> Result<(), StoreError> {
    for name in [
        crate::doc::store::property::POSITION_X,
        crate::doc::store::property::POSITION_Y,
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

pub(super) fn move_static_transform(view: &StoreView<'_>, layer: LayerId) -> Result<(), StoreError> {
    for name in TRANSFORM_PROPERTIES.iter().copied().chain([
        crate::doc::store::property::POSITION_Z,
        crate::doc::store::property::ROTATION_X,
        crate::doc::store::property::ROTATION_Y,
        crate::doc::store::property::SCALE_Z,
    ]) {
        let property = PropertyId::new(name)?;
        if let Some(source) = view.property_source(layer, &property)? {
            if !source.modulators.is_empty()
                || matches!(source.base, Some(crate::doc::store::PropertyBase::Slot(_)))
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

pub(super) fn move_translation_values(
    view: &StoreView<'_>,
    layer: LayerId,
    delta: [f64; 3],
) -> Result<Vec<Intent>, StoreError> {
    use crate::doc::store::property;
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
        if let Some(reason) = view.property_write_rejection(layer, &property)? {
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
    geometry: crate::doc::store::geometry::Geometry,
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
        crate::doc::store::property::ANCHOR,
        [0.0, 0.0],
        at,
    )?;
    let baked = bake_child_local(correction, local, anchor);
    let rebuilt = crate::doc::core::LayerPlacement::from_transform(
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
            (crate::doc::store::property::POSITION, Value::Vec2(baked.position)),
            (crate::doc::store::property::SCALE, Value::Vec2(baked.scale)),
            (crate::doc::store::property::ROTATION, Value::F64(baked.rotation_degrees)),
            (crate::doc::store::property::SKEW, Value::F64(baked.skew_degrees)),
            (crate::doc::store::property::SKEW_AXIS, Value::F64(baked.skew_axis_degrees)),
        ],
    )
}

/// Writes still transform values, keeping each property's key layout so Undo and the
/// Timeline see the same rows as before.
pub(super) fn write_transform_values(
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
            .or(view.default_value(layer, &property)?);
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
