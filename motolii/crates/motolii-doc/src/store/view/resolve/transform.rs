
use std::collections::{HashMap, HashSet};

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;

use crate::doc::store::{property, LayerId, LayerPlacement, PropertyId, StoreError};

use super::super::StoreView;

impl<'a> StoreView<'a> {
    pub(super) fn local_placement_transform(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<glam::Affine2, StoreError> {
        let scalar = |name: &str, default: f32| -> Result<f32, StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => Ok(v as f32),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に数値でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };
        let vec2 = |name: &str, default: [f32; 2]| -> Result<[f32; 2], StoreError> {
            let property = PropertyId::new(name)?;
            match self.value_at(layer, &property, t)? {
                Some(Value::Vec2(v)) => Ok([v[0] as f32, v[1] as f32]),
                Some(other) => Err(StoreError::Property(format!(
                    "{name} に2成分でない値が入っている: {other:?}"
                ))),
                None => Ok(default),
            }
        };
        Ok(LayerPlacement::from_transform(
            vec2(property::ANCHOR, [0.0, 0.0])?,
            self.resolve_position(layer, t)?,
            vec2(property::SCALE, [1.0, 1.0])?,
            scalar(property::ROTATION, 0.0)?,
            scalar(property::SKEW, 0.0)?,
            scalar(property::SKEW_AXIS, 0.0)?,
        ))
    }

    pub fn local_transform(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        self.local_placement_transform(layer, t)
    }

    pub fn local_transform3d(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<glam::Affine3A, StoreError> {
        let xy = self.local_placement_transform(layer, t)?;
        let position = self.resolve_position(layer, t)?;
        let scalar = |name| self.split_position_component(layer, name, t).map(|v| v.unwrap_or(0.0));
        Ok(LayerPlacement::spatial_from_transform(
            xy,
            position,
            scalar(property::POSITION_Z)?,
            scalar(property::ROTATION_X)?,
            scalar(property::ROTATION_Y)?,
            self.split_position_component(layer, property::SCALE_Z, t)?.unwrap_or(1.0),
        ))
    }

    pub fn world_transform3d(
        &self,
        layer: LayerId,
        t: RationalTime,
    ) -> Result<glam::Affine3A, StoreError> {
        self.world_transforms3d(t)?.remove(&layer).ok_or_else(|| {
            StoreError::Property(format!("Layer {} is not present", layer.0))
        })
    }

    pub fn world_transforms3d(
        &self,
        t: RationalTime,
    ) -> Result<HashMap<LayerId, glam::Affine3A>, StoreError> {
        let layers = self.layers();
        let present = layers.iter().copied().collect();
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        for layer in layers {
            self.inherited_transform(layer, &present, &mut memo, &mut visiting,
                |layer| self.local_transform3d(layer, t))?;
        }
        Ok(memo)
    }

    /// 1 つの層とその祖先だけの 3D world。全層を回さないので、配置の時刻ずらしで層ごとに呼べる。
    pub(super) fn world_transform3d_chain(
        &self,
        layer: LayerId,
        t: RationalTime,
        present: &HashSet<LayerId>,
    ) -> Result<HashMap<LayerId, glam::Affine3A>, StoreError> {
        let mut memo = HashMap::new();
        let mut visiting = HashSet::new();
        self.inherited_transform(layer, present, &mut memo, &mut visiting,
            |layer| self.local_transform3d(layer, t))?;
        Ok(memo)
    }

    pub(super) fn world_affine(
        &self,
        layer: LayerId,
        t: RationalTime,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, glam::Affine2>,
        visiting: &mut HashSet<LayerId>,
    ) -> Result<glam::Affine2, StoreError> {
        self.inherited_transform(layer, present, memo, visiting,
            |layer| self.local_placement_transform(layer, t))
    }

    fn inherited_transform<T: Copy + std::ops::Mul<Output = T>>(
        &self,
        layer: LayerId,
        present: &HashSet<LayerId>,
        memo: &mut HashMap<LayerId, T>,
        visiting: &mut HashSet<LayerId>,
        local: impl Fn(LayerId) -> Result<T, StoreError> + Copy,
    ) -> Result<T, StoreError> {
        if let Some(world) = memo.get(&layer) { return Ok(*world); }
        let transform = local(layer)?;
        let parent = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| present.contains(p));
        let world = if let Some(parent) = parent {
            if !visiting.insert(layer) { return Ok(transform); }
            let parent_world = self.inherited_transform(parent, present, memo, visiting, local)?;
            visiting.remove(&layer);
            parent_world * transform
        } else { transform };
        memo.insert(layer, world);
        Ok(world)
    }

    pub(super) fn resolve_position(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        let position = PropertyId::new(property::POSITION)?;
        match self.value_at(layer, &position, t)? {
            Some(Value::Vec2(v)) => return Ok([v[0] as f32, v[1] as f32]),
            Some(other) => {
                return Err(StoreError::Property(format!(
                    "{} に2成分でない値が入っている: {other:?}",
                    property::POSITION
                )))
            }
            None => {}
        }

        let x = self.split_position_component(layer, property::POSITION_X, t)?;
        let y = self.split_position_component(layer, property::POSITION_Y, t)?;
        Ok([x.unwrap_or(0.0), y.unwrap_or(0.0)])
    }

    pub(super) fn split_position_component(
        &self,
        layer: LayerId,
        name: &str,
        t: RationalTime,
    ) -> Result<Option<f32>, StoreError> {
        let property = PropertyId::new(name)?;
        match self.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => Ok(Some(v as f32)),
            Some(other) => Err(StoreError::Property(format!(
                "{name} に数値でない値が入っている: {other:?}"
            ))),
            None => Ok(None),
        }
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
            let transform = doc.view().local_transform3d(layer, RationalTime::ZERO).unwrap();
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
        assert!(doc.view().world_transforms3d(RationalTime::ZERO).unwrap().is_empty());
        assert!(doc.view().world_transform3d(LayerId(9), RationalTime::ZERO).is_err());
        let (doc, _, _) = pair();
        let revision = doc.revision();
        let head = doc.edit_head();
        let chunk_count = doc.view().db.storage_engine().store().iter_physical_chunks().count();
        assert_eq!(doc.view().world_transforms3d(RationalTime::ZERO).unwrap().len(), 2);
        assert!(doc.view().world_transform3d(LayerId(9), RationalTime::ZERO).is_err());
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
        let poses = view.world_transforms3d(RationalTime::ZERO).unwrap();
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
        close(view.local_transform3d(parent, RationalTime::ZERO).unwrap().transform_point3(Vec3::new(10.0, 20.0, 0.0)), Vec3::new(100.0, 200.0, 30.0));
        close(view.world_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(Vec3::ZERO), Vec3::new(104.0, 200.0, 25.0));
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
        let xy = view.world_affine(child, RationalTime::ZERO, &present, &mut HashMap::new(), &mut HashSet::new()).unwrap();
        let spatial = view.world_transform3d(child, RationalTime::ZERO).unwrap();
        for point in [Vec2::ZERO, Vec2::new(2.0, 5.0), Vec2::new(-4.0, 8.0)] {
            close(spatial.transform_point3(point.extend(0.0)), xy.transform_point2(point).extend(0.0));
            close(view.local_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(point.extend(0.0)), view.local_transform(child, RationalTime::ZERO).unwrap().transform_point2(point).extend(0.0));
        }
    }

    #[test]
    fn parent_nonuniform_scale_precedes_child_tilt_and_preview_is_evaluated() {
        let (mut doc, parent, child) = pair();
        set(&mut doc, parent, property::SCALE, Value::Vec2([2.0, 3.0]));
        set(&mut doc, child, property::ROTATION_Y, Value::F64(90.0));
        close(doc.view().world_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(Vec3::X), Vec3::new(0.0, 0.0, -1.0));
        doc.set_transient(parent, PropertyId::new(property::ROTATION_X).unwrap(), Value::F64(90.0));
        doc.set_transient(parent, PropertyId::new(property::POSITION_Z).unwrap(), Value::F64(7.0));
        close(doc.view().world_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(Vec3::X), Vec3::new(0.0, 1.0, 7.0));
        close(doc.view().without_transients().world_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(Vec3::X), Vec3::new(0.0, 0.0, -1.0));
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
        close(doc.view().world_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(Vec3::ZERO), Vec3::new(0.0, 0.0, 10.0));
        doc.apply(Intent::RemoveLayer(parent)).unwrap();
        assert!(!doc.view().has_layer(child));
        assert!(doc.view().world_transform3d(child, RationalTime::ZERO).is_err());
        assert!(doc.undo());
        close(doc.view().world_transform3d(child, RationalTime::ZERO).unwrap().transform_point3(Vec3::ZERO), Vec3::new(0.0, 0.0, 10.0));
    }
}
