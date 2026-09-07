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
            let group_local = view.local_transform(group, t)?;
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
                    let child_local = view.local_transform(child, t)?;
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
                    &view, layer, old_parent, new_parent, at,
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
    layer: LayerId,
    old_parent: Option<LayerId>,
    new_parent: Option<LayerId>,
    at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    let world = |parent: Option<LayerId>| match parent {
        Some(parent) => view.world_transform3d(parent, at),
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
    let local = move_planar(view.local_transform3d(layer, at)?)?;
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

#[cfg(test)]
mod move_layer_tests {
    use super::*;
    use crate::doc::store::{blank_project, property};

    fn add(doc: &mut Document, id: u64, group: bool, parent: Option<LayerId>) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: if group {
                        LayerSource::Group
                    } else {
                        LayerSource::Shape
                    },
                    order: id as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch {
                    parent: Some(parent),
                    ..Default::default()
                },
            },
        ])
        .unwrap();
        layer
    }
    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant {
            layer,
            property: PropertyId::new(name).unwrap(),
            value,
        })
        .unwrap();
    }
    fn siblings(doc: &Document, parent: Option<LayerId>) -> Vec<LayerId> {
        let v = doc.view();
        let mut ids: Vec<_> = v
            .layers()
            .into_iter()
            .filter(|id| v.attrs(*id).unwrap().unwrap_or_default().parent == parent)
            .collect();
        ids.sort_by_key(|id| std::cmp::Reverse((v.meta(*id).unwrap().unwrap().order, *id)));
        ids
    }
    fn close(a: glam::Affine3A, b: glam::Affine3A) {
        for point in [
            glam::Vec3::ZERO,
            glam::Vec3::X,
            glam::Vec3::Y,
            glam::vec3(37.0, -12.0, 0.0),
        ] {
            assert!(
                (a.transform_point3(point) - b.transform_point3(point)).length() < 0.003,
                "{a:?} != {b:?}"
            );
        }
    }
    fn animated(doc: &mut Document, layer: LayerId) -> KeyframeTrack {
        let mut t = KeyframeTrack::new();
        for (sec, x) in [(0, 0.0), (1, 100.0)] {
            t.insert(Keyframe {
                t: RationalTime::from_seconds(sec),
                value: Value::Vec2([x, 0.0]),
                interp: Interp::Linear,
                spatial: None,
            });
        }
        doc.apply(Intent::SetTrack {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            track: t.clone(),
        })
        .unwrap();
        t
    }

    #[test]
    fn sibling_drop_preserves_selected_order_and_is_one_undo() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, false, None);
        let b = add(&mut doc, 2, false, None);
        let c = add(&mut doc, 3, false, None);
        let before = doc.history_depth().0;
        let moved = doc
            .move_layers(&[a, c, a], Some(b), "after", RationalTime::ZERO)
            .unwrap();
        assert_eq!(moved, vec![c, a]);
        assert_eq!(siblings(&doc, None), vec![b, c, a]);
        assert_eq!(doc.history_depth().0, before + 1);
        assert!(doc.undo());
        assert_eq!(siblings(&doc, None), vec![c, b, a]);
        assert!(doc.redo());
        assert_eq!(siblings(&doc, None), vec![b, c, a]);
        let head = doc.history_depth();
        doc.move_layers(&[a], None, "rootEnd", RationalTime::ZERO)
            .unwrap();
        assert_eq!(doc.history_depth(), head);
    }

    #[test]
    fn dropping_selected_parent_and_child_moves_subtree_once() {
        let mut doc = blank_project();
        let parent = add(&mut doc, 1, true, None);
        let child = add(&mut doc, 2, false, Some(parent));
        let target = add(&mut doc, 3, true, None);
        assert_eq!(
            doc.move_layers(&[child, parent], Some(target), "inside", RationalTime::ZERO)
                .unwrap(),
            vec![parent]
        );
        assert_eq!(
            doc.view().attrs(parent).unwrap().unwrap().parent,
            Some(target)
        );
        assert_eq!(
            doc.view().attrs(child).unwrap().unwrap().parent,
            Some(parent)
        );
        doc.move_layers(&[parent], None, "rootEnd", RationalTime::ZERO)
            .unwrap();
        assert_eq!(doc.view().attrs(parent).unwrap().unwrap().parent, None);
        assert_eq!(
            doc.view().attrs(child).unwrap().unwrap().parent,
            Some(parent)
        );
    }

    #[test]
    fn static_reparent_preserves_world_pose_and_existing_key_time() {
        let mut doc = blank_project();
        let target = add(&mut doc, 1, true, None);
        let child = add(&mut doc, 2, false, None);
        put(
            &mut doc,
            target,
            property::POSITION,
            Value::Vec2([100.0, 200.0]),
        );
        put(&mut doc, target, property::ROTATION, Value::F64(30.0));
        put(&mut doc, target, property::SCALE, Value::Vec2([2.0, 3.0]));
        put(&mut doc, child, property::ANCHOR, Value::Vec2([5.0, 7.0]));
        put(&mut doc, child, property::ROTATION, Value::F64(10.0));
        put(&mut doc, child, property::SCALE, Value::Vec2([1.2, 0.8]));
        let key_time = RationalTime::try_new(9, 30).unwrap();
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: key_time,
            value: Value::Vec2([450.0, 220.0]),
            interp: Interp::Hold,
            spatial: None,
        });
        doc.apply(Intent::SetTrack {
            layer: child,
            property: PropertyId::new(property::POSITION).unwrap(),
            track,
        })
        .unwrap();
        let at = RationalTime::try_new(15, 30).unwrap();
        let before = doc.view().world_transform3d(child, at).unwrap();
        let history = doc.history_depth().0;
        doc.move_layers(&[child], Some(target), "inside", at)
            .unwrap();
        close(before, doc.view().world_transform3d(child, at).unwrap());
        assert_eq!(doc.history_depth().0, history + 1);
        let keys = doc
            .view()
            .track(child, &PropertyId::new(property::POSITION).unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(keys.keys().len(), 1);
        assert_eq!(keys.keys()[0].t, key_time);
        assert!(doc
            .view()
            .track(child, &PropertyId::new(property::SCALE).unwrap())
            .unwrap()
            .is_none());
        assert!(doc.undo());
        assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent, None);
        close(before, doc.view().world_transform3d(child, at).unwrap());
    }

    #[test]
    fn identity_parent_accepts_animation_but_rotated_coordinate_system_does_not() {
        let mut doc = blank_project();
        let identity = add(&mut doc, 1, true, None);
        let translated = add(&mut doc, 2, true, None);
        let child = add(&mut doc, 3, false, None);
        put(
            &mut doc,
            translated,
            property::POSITION,
            Value::Vec2([50.0, 0.0]),
        );
        put(&mut doc, translated, property::ROTATION, Value::F64(30.0));
        let track = animated(&mut doc, child);
        doc.move_layers(&[child], Some(identity), "inside", RationalTime::ZERO)
            .unwrap();
        assert_eq!(
            doc.view()
                .track(child, &PropertyId::new(property::POSITION).unwrap())
                .unwrap()
                .unwrap(),
            track
        );
        let history = doc.history_depth();
        assert!(doc
            .move_layers(&[child], Some(translated), "inside", RationalTime::ZERO)
            .is_err());
        assert_eq!(doc.history_depth(), history);
        assert_eq!(
            doc.view().attrs(child).unwrap().unwrap().parent,
            Some(identity)
        );
        let moving_parent = add(&mut doc, 4, true, None);
        animated(&mut doc, moving_parent);
        let history = doc.history_depth();
        doc.move_layers(&[child], Some(moving_parent), "inside", RationalTime::ZERO)
            .unwrap();
        assert_eq!(doc.history_depth().0, history.0 + 1);
        assert_eq!(
            doc.view()
                .track(child, &PropertyId::new(property::POSITION).unwrap())
                .unwrap()
                .unwrap(),
            track
        );
    }

    #[test]
    fn invalid_cycle_locked_and_frozen_drops_are_atomic() {
        let mut doc = blank_project();
        let parent = add(&mut doc, 1, true, None);
        let nested = add(&mut doc, 2, true, Some(parent));
        let plain = add(&mut doc, 3, false, None);
        for (layers, target, placement) in [
            (vec![parent], Some(nested), "inside"),
            (vec![parent], Some(plain), "inside"),
            (vec![LayerId(99)], Some(parent), "before"),
        ] {
            let history = doc.history_depth();
            assert!(doc
                .move_layers(&layers, target, placement, RationalTime::ZERO)
                .is_err());
            assert_eq!(doc.history_depth(), history);
        }
        doc.apply(Intent::SetAttrs {
            layer: plain,
            patch: LayerAttrsPatch {
                locked: Some(true),
                ..Default::default()
            },
        })
        .unwrap();
        let history = doc.history_depth();
        assert!(doc
            .move_layers(&[plain], Some(parent), "before", RationalTime::ZERO)
            .is_err());
        assert_eq!(doc.history_depth(), history);
        doc.apply(Intent::Freeze { group: nested }).unwrap();
        let history = doc.history_depth();
        assert!(doc
            .move_layers(&[nested], None, "rootEnd", RationalTime::ZERO)
            .is_err());
        assert_eq!(doc.history_depth(), history);
    }

    #[test]
    fn animated_parent_accepts_static_child_at_current_pose_then_drives_it() {
        let mut doc = blank_project();
        let parent = add(&mut doc, 1, true, None);
        let child = add(&mut doc, 2, false, None);
        animated(&mut doc, parent);
        put(
            &mut doc,
            child,
            property::POSITION,
            Value::Vec2([200.0, 70.0]),
        );
        let at = RationalTime::try_new(1, 2).unwrap();
        let before = doc.view().world_transform3d(child, at).unwrap();
        let history = doc.history_depth().0;
        doc.move_layers(&[child], Some(parent), "inside", at)
            .unwrap();
        close(before, doc.view().world_transform3d(child, at).unwrap());
        assert_eq!(doc.history_depth().0, history + 1);
        assert!(doc
            .view()
            .track(child, &PropertyId::new(property::POSITION).unwrap())
            .unwrap()
            .is_none());
        let future = doc
            .view()
            .world_transform3d(child, RationalTime::from_seconds(1))
            .unwrap()
            .transform_point3(glam::Vec3::ZERO);
        assert!((future.x - 250.0).abs() < 0.001);
        assert!((future.y - 70.0).abs() < 0.001);
        assert!(doc.undo());
        assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent, None);
        close(before, doc.view().world_transform3d(child, at).unwrap());
    }

    #[test]
    fn translation_reparent_keeps_animated_spatial_keys_and_world_motion() {
        let mut doc = blank_project();
        let parent = add(&mut doc, 1, true, None);
        let child = add(&mut doc, 2, false, None);
        put(
            &mut doc,
            parent,
            property::POSITION,
            Value::Vec2([100.0, 200.0]),
        );
        put(&mut doc, parent, property::POSITION_Z, Value::F64(30.0));
        put(&mut doc, child, property::ROTATION_X, Value::F64(45.0));
        put(&mut doc, child, property::POSITION_Z, Value::F64(8.0));
        let mut track = KeyframeTrack::new();
        for (sec, position) in [(0, [10.0, 20.0]), (1, [60.0, 90.0])] {
            track.insert(Keyframe {
                t: RationalTime::from_seconds(sec),
                value: Value::Vec2(position),
                interp: Interp::Bezier {
                    x1: 0.25,
                    y1: 0.1,
                    x2: 0.25,
                    y2: 1.0,
                },
                spatial: Some(crate::doc::store::SpatialTangent {
                    out_tangent: [3.0, 5.0],
                    in_tangent: [-4.0, -2.0],
                }),
            });
        }
        let property = PropertyId::new(property::POSITION).unwrap();
        doc.apply(Intent::SetTrack {
            layer: child,
            property: property.clone(),
            track: track.clone(),
        })
        .unwrap();
        let times = [
            RationalTime::ZERO,
            RationalTime::try_new(3, 10).unwrap(),
            RationalTime::try_new(7, 10).unwrap(),
            RationalTime::from_seconds(1),
        ];
        let before = times.map(|t| doc.view().world_transform3d(child, t).unwrap());
        let history = doc.history_depth().0;
        doc.move_layers(
            &[child],
            Some(parent),
            "inside",
            RationalTime::try_new(1, 2).unwrap(),
        )
        .unwrap();
        assert_eq!(doc.history_depth().0, history + 1);
        let moved = doc.view().track(child, &property).unwrap().unwrap();
        for (old, new) in track.keys().iter().zip(moved.keys()) {
            assert_eq!(old.t, new.t);
            assert_eq!(old.interp, new.interp);
            assert_eq!(old.spatial, new.spatial);
            let (Value::Vec2(old), Value::Vec2(new)) = (&old.value, &new.value) else {
                panic!("position type")
            };
            assert_eq!(*new, [old[0] - 100.0, old[1] - 200.0]);
        }
        for (t, before) in times.into_iter().zip(before) {
            close(before, doc.view().world_transform3d(child, t).unwrap());
        }
        assert!(doc.undo());
        assert_eq!(doc.view().track(child, &property).unwrap().unwrap(), track);
        assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent, None);
    }

    #[test]
    fn translation_reparent_preserves_separate_xyz_tracks_and_constants() {
        let mut doc = blank_project();
        let parent = add(&mut doc, 1, true, None);
        let child = add(&mut doc, 2, false, None);
        put(
            &mut doc,
            parent,
            property::POSITION,
            Value::Vec2([50.0, 80.0]),
        );
        put(&mut doc, parent, property::POSITION_Z, Value::F64(20.0));
        let scalar = |a, b| {
            let mut track = KeyframeTrack::new();
            for (sec, value) in [(0, a), (1, b)] {
                track.insert(Keyframe {
                    t: RationalTime::from_seconds(sec),
                    value: Value::F64(value),
                    interp: Interp::Linear,
                    spatial: None,
                });
            }
            track
        };
        let x = PropertyId::new(property::POSITION_X).unwrap();
        let z = PropertyId::new(property::POSITION_Z).unwrap();
        let xt = scalar(1.0, 11.0);
        let zt = scalar(5.0, 15.0);
        doc.apply_all([
            Intent::SetTrack {
                layer: child,
                property: x.clone(),
                track: xt.clone(),
            },
            Intent::SetTrack {
                layer: child,
                property: z.clone(),
                track: zt.clone(),
            },
        ])
        .unwrap();
        put(&mut doc, child, property::POSITION_Y, Value::F64(4.0));
        let times = [
            RationalTime::ZERO,
            RationalTime::try_new(1, 2).unwrap(),
            RationalTime::from_seconds(1),
        ];
        let before = times.map(|t| doc.view().world_transform3d(child, t).unwrap());
        doc.move_layers(&[child], Some(parent), "inside", times[1])
            .unwrap();
        assert!(doc
            .view()
            .property_source(child, &PropertyId::new(property::POSITION).unwrap())
            .unwrap()
            .is_none());
        assert!(doc
            .view()
            .track(child, &PropertyId::new(property::POSITION_Y).unwrap())
            .unwrap()
            .is_none());
        for (t, before) in times.into_iter().zip(before) {
            close(before, doc.view().world_transform3d(child, t).unwrap());
        }
        let moved = doc.view().track(child, &x).unwrap().unwrap();
        assert_eq!(moved.keys()[0].value, Value::F64(-49.0));
        assert_eq!(moved.keys()[1].value, Value::F64(-39.0));
        assert!(doc.undo());
        assert_eq!(doc.view().track(child, &x).unwrap().unwrap(), xt);
        assert_eq!(doc.view().track(child, &z).unwrap().unwrap(), zt);
    }
}
