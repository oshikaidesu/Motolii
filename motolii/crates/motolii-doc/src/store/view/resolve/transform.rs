
use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{property, LayerId, LayerPlacement, PropertyId, StoreError};

use super::super::StoreView;

pub(super) fn local_placement_transform(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<glam::Affine2, StoreError> {
    local_placement_transform_sampled(view, layer, t, None)
}

/// `sample` があれば、位置・大きさ・角度のうち印の付いた物だけをその時刻で取る(Motion Blur)。他は t。
pub(super) fn local_placement_transform_sampled(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    sample: Option<(RationalTime, [bool; 3])>,
) -> Result<glam::Affine2, StoreError> {
    let when = |which: usize| sample.filter(|(_, mask)| mask[which]).map_or(t, |(at, _)| at);
    let scalar = |name: &str, default: f32, t: RationalTime| -> Result<f32, StoreError> {
        let property = PropertyId::new(name)?;
        match view.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => Ok(v as f32),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に数値でない値が入っている: {other:?}"
            ))),
            None => Ok(default),
        }
    };
    let vec2 = |name: &str, default: [f32; 2], t: RationalTime| -> Result<[f32; 2], StoreError> {
        let property = PropertyId::new(name)?;
        match view.value_at(layer, &property, t)? {
            Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に2成分でない値が入っている: {other:?}"
            ))),
            None => Ok(default),
        }
    };
    // つなぐ線は道を親の空間で解いてある: 回さず伸ばさず、素材座標の原点のずれだけ戻して置く。
    if let Some(position) = view.connector_position(layer, t)? {
        return Ok(LayerPlacement::from_transform([0.0, 0.0], position, [1.0, 1.0], 0.0, 0.0, 0.0));
    }
    let slot = view.laid_out(layer, when(0))?;
    let (position, scale) = match slot {
        Some(slot) => (slot.position, slot.scale),
        None => {
            let (p, d) = (resolve_position(view, layer, when(0))?, view.nudge(layer, when(0))?);
            ([p[0] + d[0], p[1] + d[1]], vec2(property::SCALE, [1.0, 1.0], when(1))?)
        }
    };
    Ok(LayerPlacement::from_transform(
        match slot { Some(slot) => slot.anchor, None => view.free_anchor(layer, t)? },
        position,
        scale,
        scalar(property::ROTATION, 0.0, when(2))? + slot.map_or(0.0, |s| s.rotation[2]) + crate::doc::store::layout::path_offset_rotation(view, layer, when(2))?,
        scalar(property::SKEW, 0.0, t)?,
        scalar(property::SKEW_AXIS, 0.0, t)?,
    ))
}

pub fn local_transform(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
    local_placement_transform(view, layer, t)
}

pub fn local_transform3d(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<glam::Affine3A, StoreError> {
    let xy = local_placement_transform(view, layer, t)?;
    let slot = view.laid_out(layer, t)?;
    let connector = view.connector_position(layer, t)?;
    let position = match slot {
        _ if connector.is_some() => connector.unwrap_or_default(),
        Some(slot) => slot.position,
        None => {
            let (p, d) = (resolve_position(view, layer, t)?, view.nudge(layer, t)?);
            [p[0] + d[0], p[1] + d[1]]
        }
    };
    let scalar = |name| split_position_component(view, layer, name, t).map(|v| v.unwrap_or(0.0));
    Ok(LayerPlacement::spatial_from_transform(
        xy,
        position,
        scalar(property::POSITION_Z)? + match slot { Some(s) => s.z, None => view.nudge_z(layer, t)? },
        scalar(property::ROTATION_X)? + slot.map_or(0.0, |s| s.rotation[0]),
        scalar(property::ROTATION_Y)? + slot.map_or(0.0, |s| s.rotation[1]),
        split_position_component(view, layer, property::SCALE_Z, t)?.unwrap_or(1.0) * slot.map_or(1.0, |s| s.scale_z),
    ))
}

pub fn world_transform3d(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<glam::Affine3A, StoreError> {
    world_transforms3d(view, t)?.remove(&layer).ok_or_else(|| {
        StoreError::Property(format!("Layer {} is not present", layer.0))
    })
}

pub fn world_transforms3d(
    view: &StoreView<'_>,
    t: RationalTime,
) -> Result<HashMap<LayerId, glam::Affine3A>, StoreError> {
    let layers = view.layers();
    let present = layers.iter().copied().collect();
    let mut memo = HashMap::new();
    let mut visiting = HashSet::new();
    for layer in layers {
        inherited_transform(view, layer, &present, &mut memo, &mut visiting,
            |layer| local_transform3d(view, layer, t))?;
    }
    Ok(memo)
}

/// 1 つの層とその祖先だけの 3D world。全層を回さないので、配置の時刻ずらしで層ごとに呼べる。
pub(super) fn world_transform3d_chain(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    present: &HashSet<LayerId>,
) -> Result<HashMap<LayerId, glam::Affine3A>, StoreError> {
    let mut memo = HashMap::new();
    let mut visiting = HashSet::new();
    inherited_transform(view, layer, present, &mut memo, &mut visiting,
        |layer| local_transform3d(view, layer, t))?;
    Ok(memo)
}

pub(super) fn world_affine(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
    present: &HashSet<LayerId>,
    memo: &mut HashMap<LayerId, glam::Affine2>,
    visiting: &mut HashSet<LayerId>,
) -> Result<glam::Affine2, StoreError> {
    inherited_transform(view, layer, present, memo, visiting,
        |layer| local_placement_transform(view, layer, t))
}

pub(crate) fn inherited_transform<T: Copy + std::ops::Mul<Output = T>>(
    view: &StoreView<'_>,
    layer: LayerId,
    present: &HashSet<LayerId>,
    memo: &mut HashMap<LayerId, T>,
    visiting: &mut HashSet<LayerId>,
    local: impl Fn(LayerId) -> Result<T, StoreError> + Copy,
) -> Result<T, StoreError> {
    if let Some(world) = memo.get(&layer) { return Ok(*world); }
    let transform = local(layer)?;
    let parent = view.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
    let world = if let Some(parent) = parent {
        if !visiting.insert(layer) { return Ok(transform); }
        let parent_world = inherited_transform(view, parent, present, memo, visiting, local)?;
        visiting.remove(&layer);
        parent_world * transform
    } else { transform };
    memo.insert(layer, world);
    Ok(world)
}

pub(crate) fn resolve_position(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
    // 箱の輪郭を道にする物は、道の上の点が Position の代わり。
    if let Some((point, _)) = crate::doc::store::layout::path_on_offset_path(view, layer, t)? {
        return Ok(point);
    }
    let position = PropertyId::new(property::POSITION)?;
    match view.value_at(layer, &position, t)? {
        Some(Value::Vec2(v)) => return Ok([v[0] as f32, v[1] as f32]),
        Some(other) => {
            return Err(StoreError::Property(format!(
                "{} に2成分でない値が入っている: {other:?}",
                property::POSITION
            )))
        }
        None => {}
    }

    let x = split_position_component(view, layer, property::POSITION_X, t)?;
    let y = split_position_component(view, layer, property::POSITION_Y, t)?;
    Ok([x.unwrap_or(0.0), y.unwrap_or(0.0)])
}

pub(super) fn split_position_component(
    view: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    t: RationalTime,
) -> Result<Option<f32>, StoreError> {
    let property = PropertyId::new(name)?;
    match view.value_at(layer, &property, t)? {
        Some(Value::F64(v)) => Ok(Some(v as f32)),
        Some(other) => Err(StoreError::Property(format!(
            "{name} に数値でない値が入っている: {other:?}"
        ))),
        None => Ok(None),
    }
}

#[cfg(test)]
mod spatial_tests {
    use super::*;
    use crate::doc::store::{Document, Intent, LayerAttrsPatch};
    use glam::{Vec2, Vec3};

    fn set(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }

    fn pair() -> (Document, LayerId, LayerId) {
        let mut doc = Document::new();
        let parent = LayerId(1);
        let child = LayerId(2);
        doc.apply_all([Intent::AddLayer(parent), Intent::AddLayer(child), Intent::SetAttrs {
            layer: child,
            patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() },
        }]).unwrap();
        (doc, parent, child)
    }

    fn close(actual: Vec3, expected: Vec3) {
        assert!((actual - expected).length() < 0.0001, "actual {actual:?}, expected {expected:?}");
    }

    #[test]
    fn local_pose_matches_pinned_lottie_web_reference() {
        let oracle: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"), "/../../reference/lottie-transform-oracle.json"
        ))).unwrap();
        for case in oracle["cases"].as_array().unwrap() {
            let mut doc = Document::new();
            let layer = LayerId(1);
            doc.apply(Intent::AddLayer(layer)).unwrap();
            let number = |name: &str, index: usize| case[name][index].as_f64().unwrap();
            set(&mut doc, layer, property::ANCHOR, Value::Vec2([number("anchor", 0), number("anchor", 1)]));
            set(&mut doc, layer, property::POSITION, Value::Vec2([number("position", 0), number("position", 1)]));
            set(&mut doc, layer, property::POSITION_Z, Value::F64(number("position", 2)));
            set(&mut doc, layer, property::SCALE, Value::Vec2([number("scale", 0), number("scale", 1)]));
            for (property, value) in [
                (property::ROTATION_X, number("rotation", 0)),
                (property::ROTATION_Y, number("rotation", 1)),
                (property::ROTATION, number("rotation", 2)),
                (property::SKEW, case["skew"].as_f64().unwrap()),
                (property::SKEW_AXIS, case["axis"].as_f64().unwrap()),
            ] { set(&mut doc, layer, property, Value::F64(value)); }
            let transform = crate::doc::store::view::resolve::transform::local_transform3d(&doc.view(), layer, RationalTime::ZERO).unwrap();
            for sample in case["samples"].as_array().unwrap() {
                let point = |name: &str| Vec3::from_array(std::array::from_fn(|i| sample[name][i].as_f64().unwrap() as f32));
                let actual = transform.transform_point3(point("point"));
                assert!(actual.distance(point("expected")) < 0.0001,
                    "Lottie reference {}: actual {actual:?}, expected {:?}", case["name"], point("expected"));
            }
        }
    }

    #[test]
    fn empty_projection_and_missing_layer_queries_are_explicit() {
        let doc = Document::new();
        assert!(crate::doc::store::view::resolve::transform::world_transforms3d(&doc.view(), RationalTime::ZERO).unwrap().is_empty());
        assert!(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), LayerId(9), RationalTime::ZERO).is_err());
        let (doc, _, _) = pair();
        let revision = doc.revision();
        let head = doc.edit_head();
        let chunk_count = doc.view().db.storage_engine().store().iter_physical_chunks().count();
        assert_eq!(crate::doc::store::view::resolve::transform::world_transforms3d(&doc.view(), RationalTime::ZERO).unwrap().len(), 2);
        assert!(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), LayerId(9), RationalTime::ZERO).is_err());
        assert_eq!(doc.revision(), revision);
        assert_eq!(doc.edit_head(), head);
        assert_eq!(doc.view().db.storage_engine().store().iter_physical_chunks().count(), chunk_count);
    }

    #[test]
    fn resolved_layers_carry_the_same_inherited_world_pose() {
        use crate::doc::store::{Composition, Fps, LayerMeta, LayerSource, LayerTiming};
        let (mut doc, parent, child) = pair();
        doc.apply(Intent::SetComposition(Composition {
            width: 640, height: 480, fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30, background: Composition::default_background(),
        })).unwrap();
        for layer in [parent, child] {
            doc.apply(Intent::SetMeta { layer, meta: LayerMeta {
                source: if layer == parent { LayerSource::Group } else { LayerSource::Shape },
                order: layer.0 as i16, timing: LayerTiming::place(0, None, 30),
            }}).unwrap();
        }
        set(&mut doc, parent, property::POSITION_Z, Value::F64(25.0));
        set(&mut doc, parent, property::ROTATION_Y, Value::F64(90.0));
        set(&mut doc, child, property::POSITION, Value::Vec2([10.0, 0.0]));
        let view = doc.view();
        let poses = crate::doc::store::view::resolve::transform::world_transforms3d(&view, RationalTime::ZERO).unwrap();
        let layers = view.resolved_layers(RationalTime::ZERO).unwrap();
        let resolved = layers.iter().find(|layer| layer.id == child).unwrap();
        let world = resolved.placement.world_transform.unwrap();
        assert_eq!(world, poses[&child]);
        close(world.transform_point3(Vec3::ZERO), Vec3::new(0.0, 0.0, 15.0));
        assert_eq!(view.resolve(child, RationalTime::ZERO).unwrap().unwrap().placement.world_transform, Some(world));
    }

    #[test]
    fn parent_depth_and_rotation_orbit_the_child_around_the_authored_anchor() {
        let (mut doc, parent, child) = pair();
        set(&mut doc, parent, property::POSITION, Value::Vec2([100.0, 200.0]));
        set(&mut doc, parent, property::ANCHOR, Value::Vec2([10.0, 20.0]));
        set(&mut doc, parent, property::POSITION_Z, Value::F64(30.0));
        set(&mut doc, parent, property::ROTATION_Y, Value::F64(90.0));
        set(&mut doc, child, property::POSITION, Value::Vec2([15.0, 20.0]));
        set(&mut doc, child, property::POSITION_Z, Value::F64(4.0));
        let view = doc.view();
        close(crate::doc::store::view::resolve::transform::local_transform3d(&view, parent, RationalTime::ZERO).unwrap().transform_point3(Vec3::new(10.0, 20.0, 0.0)), Vec3::new(100.0, 200.0, 30.0));
        close(crate::doc::store::view::resolve::transform::world_transform3d(&view, child, RationalTime::ZERO).unwrap().transform_point3(Vec3::ZERO), Vec3::new(104.0, 200.0, 25.0));
    }

    #[test]
    fn untilted_transform_matches_xy_with_anchor_skew_and_scaled_parent() {
        let (mut doc, parent, child) = pair();
        for (layer, position, scale, rotation) in [
            (parent, [40.0, 70.0], [2.0, 0.5], 35.0),
            (child, [12.0, -8.0], [0.7, 1.3], -20.0),
        ] {
            set(&mut doc, layer, property::POSITION, Value::Vec2(position));
            set(&mut doc, layer, property::ANCHOR, Value::Vec2([3.0, 7.0]));
            set(&mut doc, layer, property::SCALE, Value::Vec2(scale));
            set(&mut doc, layer, property::ROTATION, Value::F64(rotation));
            set(&mut doc, layer, property::SKEW, Value::F64(12.0));
            set(&mut doc, layer, property::SKEW_AXIS, Value::F64(23.0));
        }
        let view = doc.view();
        let present = view.layers().into_iter().collect();
        let xy = crate::doc::store::view::resolve::transform::world_affine(&view, child, RationalTime::ZERO, &present, &mut HashMap::new(), &mut HashSet::new()).unwrap();
        let spatial = crate::doc::store::view::resolve::transform::world_transform3d(&view, child, RationalTime::ZERO).unwrap();
        for point in [Vec2::ZERO, Vec2::new(2.0, 5.0), Vec2::new(-4.0, 8.0)] {
            close(spatial.transform_point3(point.extend(0.0)), xy.transform_point2(point).extend(0.0));
            close(crate::doc::store::view::resolve::transform::local_transform3d(&view, child, RationalTime::ZERO).unwrap().transform_point3(point.extend(0.0)), crate::doc::store::view::resolve::transform::local_transform(&view, child, RationalTime::ZERO).unwrap().transform_point2(point).extend(0.0));
        }
    }

    #[test]
    fn parent_nonuniform_scale_precedes_child_tilt_and_preview_is_evaluated() {
        let (mut doc, parent, child) = pair();
        set(&mut doc, parent, property::SCALE, Value::Vec2([2.0, 3.0]));
        set(&mut doc, child, property::ROTATION_Y, Value::F64(90.0));
        close(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), child, RationalTime::ZERO).unwrap().transform_point3(Vec3::X), Vec3::new(0.0, 0.0, -1.0));
        doc.set_transient(parent, PropertyId::new(property::ROTATION_X).unwrap(), Value::F64(90.0));
        doc.set_transient(parent, PropertyId::new(property::POSITION_Z).unwrap(), Value::F64(7.0));
        close(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), child, RationalTime::ZERO).unwrap().transform_point3(Vec3::X), Vec3::new(0.0, 1.0, 7.0));
        close(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view().without_transients(), child, RationalTime::ZERO).unwrap().transform_point3(Vec3::X), Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn cycles_remain_rejected_and_parent_deletion_removes_descendants() {
        let (mut doc, parent, child) = pair();
        set(&mut doc, parent, property::POSITION_Z, Value::F64(7.0));
        set(&mut doc, child, property::POSITION_Z, Value::F64(3.0));
        assert!(doc.apply(Intent::SetAttrs {
            layer: parent,
            patch: LayerAttrsPatch { parent: Some(Some(child)), ..Default::default() },
        }).is_err());
        close(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), child, RationalTime::ZERO).unwrap().transform_point3(Vec3::ZERO), Vec3::new(0.0, 0.0, 10.0));
        doc.apply(Intent::RemoveLayer(parent)).unwrap();
        assert!(!doc.view().has_layer(child));
        assert!(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), child, RationalTime::ZERO).is_err());
        assert!(doc.undo());
        close(crate::doc::store::view::resolve::transform::world_transform3d(&doc.view(), child, RationalTime::ZERO).unwrap().transform_point3(Vec3::ZERO), Vec3::new(0.0, 0.0, 10.0));
    }
}
