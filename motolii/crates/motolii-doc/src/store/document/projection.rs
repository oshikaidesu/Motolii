use crate::doc::core::{projection_switch_world, LayerPlacement, RationalTime};
use crate::doc::eval::Value;
use crate::doc::store::view::StoreView;
use crate::doc::store::{property, LayerAttrsPatch, LayerProjection, StoreError};

use super::group::{
    bake_child_local, move_static_transform, move_translation_values, read_vec2,
    refuse_split_position, write_transform_values,
};
use super::{Document, Intent, LayerId};

impl Document {
    /// Applies `patch` to each layer. A projection change keeps the picture seen at `at`
    /// under the document camera, like a parent change keeps the pose at drop time.
    /// `layers` pairs each layer with the center of its local bounds.
    pub fn set_projection(
        &mut self,
        layers: &[(LayerId, [f32; 3])],
        patch: LayerAttrsPatch,
        at: RationalTime,
    ) -> Result<(), StoreError> {
        let mut intents = Vec::new();
        {
            let view = self.view();
            for &(layer, local_center) in layers {
                if let Some(to) = patch.projection {
                    intents.extend(projection_compensation(&view, layer, local_center, to, at)?);
                }
                intents.push(Intent::SetAttrs { layer, patch: patch.clone() });
            }
        }
        self.apply_all(intents)
    }
}

fn projection_compensation(
    view: &StoreView<'_>,
    layer: LayerId,
    local_center: [f32; 3],
    to: LayerProjection,
    at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    let attrs = view.attrs(layer)?.unwrap_or_default();
    let from = attrs.projection;
    if from == to {
        return Ok(Vec::new());
    }
    let comp = view
        .composition()?
        .ok_or_else(|| StoreError::Property("No composition".into()))?
        .spec();
    let camera = view.resolve_camera(at)?;
    let worlds = view.world_transforms3d(at)?;
    let world = *worlds.get(&layer).ok_or_else(|| {
        StoreError::Property(format!("Layer {} is not present", layer.0))
    })?;
    let parent = attrs
        .parent
        .and_then(|id| worlds.get(&id).copied())
        .unwrap_or(glam::Affine3A::IDENTITY);
    let switched = projection_switch_world(comp, camera, from, to, world, local_center.into())
        .ok_or_else(|| StoreError::Property("Cannot keep this layer's picture across the projection change; it is not in front of the camera".into()))?;
    let local = view.local_transform3d(layer, at)?;
    let target = parent.inverse() * switched;
    let in_layer = local.inverse() * target;
    if !in_layer.is_finite() {
        return Err(StoreError::Property("Projection change needs a finite transform".into()));
    }
    // Only the face (local x and y) is drawn, so a change along local z alone is invisible.
    let face_unchanged = glam::Vec3::from(in_layer.matrix3.x_axis).distance(glam::Vec3::X) <= 1e-4
        && glam::Vec3::from(in_layer.matrix3.y_axis).distance(glam::Vec3::Y) <= 1e-4;
    if face_unchanged {
        let delta = local.transform_vector3(in_layer.translation.into());
        if delta.length() <= 1e-3 {
            return Ok(Vec::new());
        }
        return move_translation_values(view, layer, delta.to_array().map(f64::from));
    }
    move_static_transform(view, layer)?;
    refuse_split_position(view, layer)?;
    let anchor = read_vec2(view, layer, property::ANCHOR, [0.0, 0.0], at)?;
    let spatial = decompose_spatial(target, anchor).ok_or_else(|| {
        StoreError::Property("Projection change would fold the layer flat".into())
    })?;
    let rebuilt = LayerPlacement::spatial_from_transform(
        LayerPlacement::from_transform(
            anchor,
            spatial.planar.position.map(|v| v as f32),
            spatial.planar.scale.map(|v| v as f32),
            spatial.planar.rotation_degrees as f32,
            spatial.planar.skew_degrees as f32,
            spatial.planar.skew_axis_degrees as f32,
        ),
        spatial.planar.position.map(|v| v as f32),
        spatial.z,
        spatial.rotation_x_degrees,
        spatial.rotation_y_degrees,
    );
    let tolerance = 1e-3 * target.translation.length().max(1.0);
    let same_face = [
        (target.matrix3.x_axis, rebuilt.matrix3.x_axis),
        (target.matrix3.y_axis, rebuilt.matrix3.y_axis),
        (target.translation, rebuilt.translation),
    ]
    .into_iter()
    .all(|(a, b)| (glam::Vec3::from(a) - glam::Vec3::from(b)).length() <= tolerance);
    if !same_face {
        return Err(StoreError::Property(
            "Projection change cannot be preserved without changing the layer".into(),
        ));
    }
    write_transform_values(
        view,
        layer,
        at,
        [
            (property::POSITION, Value::Vec2(spatial.planar.position)),
            (property::POSITION_Z, Value::F64(f64::from(spatial.z))),
            (property::ROTATION_X, Value::F64(f64::from(spatial.rotation_x_degrees))),
            (property::ROTATION_Y, Value::F64(f64::from(spatial.rotation_y_degrees))),
            (property::SCALE, Value::Vec2(spatial.planar.scale)),
            (property::ROTATION, Value::F64(spatial.planar.rotation_degrees)),
            (property::SKEW, Value::F64(spatial.planar.skew_degrees)),
            (property::SKEW_AXIS, Value::F64(spatial.planar.skew_axis_degrees)),
        ],
    )
}

struct SpatialValues {
    planar: super::group::BakedChildTransform,
    z: f32,
    rotation_x_degrees: f32,
    rotation_y_degrees: f32,
}

/// Splits a local 3D transform into the layer's properties. Exact for the layer's face
/// (its x and y axes); the face normal is rebuilt from them, so a volume's depth axis
/// keeps only its direction.
fn decompose_spatial(local: glam::Affine3A, anchor: [f32; 2]) -> Option<SpatialValues> {
    use glam::{Affine2, Quat, Vec3};
    let bx = Vec3::from(local.matrix3.x_axis);
    let by = Vec3::from(local.matrix3.y_axis);
    let normal = bx.cross(by).try_normalize()?;
    let rotation_y = normal.x.clamp(-1.0, 1.0).asin();
    let rotation_x = (-normal.y).atan2(normal.z);
    let tilt = Quat::from_rotation_x(rotation_x) * Quat::from_rotation_y(rotation_y);
    let untilted = glam::Mat3::from_quat(tilt.inverse()) * glam::Mat3::from(local.matrix3);
    let matrix2 = glam::Mat2::from_cols(untilted.x_axis.truncate(), untilted.y_axis.truncate());
    let position = local.transform_point3(Vec3::new(anchor[0], anchor[1], 0.0));
    let xy = Affine2::from_mat2_translation(
        matrix2,
        position.truncate() - matrix2 * glam::Vec2::from(anchor),
    );
    let planar = bake_child_local(Affine2::IDENTITY, xy, anchor);
    (position.is_finite() && planar.position.iter().all(|v| v.is_finite())).then_some(SpatialValues {
        planar,
        z: position.z,
        rotation_x_degrees: rotation_x.to_degrees(),
        rotation_y_degrees: rotation_y.to_degrees(),
    })
}

#[cfg(test)]
mod projection_switch_tests {
    use super::*;
    use crate::doc::store::PropertyId;
    use crate::doc::core::{projected_screen_corners, ResolvedCamera};
    use crate::doc::eval::{Interp, Keyframe, KeyframeTrack};
    use crate::doc::store::{blank_project, LayerMeta, LayerSource, LayerTiming};

    const MIN: [f32; 3] = [-12.0, -20.0, 0.0];
    const MAX: [f32; 3] = [212.0, 140.0, 0.0];

    fn shape(doc: &mut Document, id: u64, parent: Option<LayerId>) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta { source: LayerSource::Shape, order: id as i16, timing: LayerTiming::place(0, None, 300) },
            },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), ..Default::default() } },
        ])
        .unwrap();
        layer
    }
    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }
    fn camera(doc: &mut Document) -> ResolvedCamera {
        for (name, value) in [
            (property::CAMERA_CENTER, Value::Vec2([210.0, -130.0])),
            (property::CAMERA_ZOOM, Value::F64(1.7)),
            (property::CAMERA_ROLL, Value::F64(47.0)),
        ] {
            doc.apply(Intent::SetCameraConstant { property: PropertyId::camera(name).unwrap(), value }).unwrap();
        }
        doc.view().resolve_camera(RationalTime::ZERO).unwrap()
    }
    fn center() -> [f32; 3] {
        std::array::from_fn(|i| (MIN[i] + MAX[i]) * 0.5)
    }
    fn corners(doc: &Document, layer: LayerId) -> [glam::Vec2; 8] {
        let view = doc.view();
        let comp = view.composition().unwrap().unwrap().spec();
        let camera = view.resolve_camera(RationalTime::ZERO).unwrap();
        let attrs = view.attrs(layer).unwrap().unwrap_or_default();
        let world = view.world_transform3d(layer, RationalTime::ZERO).unwrap();
        projected_screen_corners(comp, camera, camera, attrs.projection, world, MIN, MAX)
    }
    fn switch(doc: &mut Document, layer: LayerId, to: LayerProjection) {
        let before = corners(doc, layer);
        doc.set_projection(&[(layer, center())], LayerAttrsPatch { projection: Some(to), ..Default::default() }, RationalTime::ZERO).unwrap();
        assert_eq!(doc.view().attrs(layer).unwrap().unwrap().projection, to);
        let after = corners(doc, layer);
        for (a, b) in before.iter().zip(after) {
            assert!(a.distance(b) < 0.05, "{to:?}: {a:?} != {b:?}");
        }
    }
    fn scalar(doc: &Document, layer: LayerId, name: &str) -> f64 {
        match doc.view().value_at(layer, &PropertyId::new(name).unwrap(), RationalTime::ZERO).unwrap() {
            Some(Value::F64(v)) => v,
            None => 0.0,
            other => panic!("{name}: {other:?}"),
        }
    }

    #[test]
    fn tilted_child_under_a_moved_camera_keeps_its_picture_through_every_switch() {
        let mut doc = blank_project();
        camera(&mut doc);
        let group = shape(&mut doc, 1, None);
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, group, property::ROTATION, Value::F64(20.0));
        put(&mut doc, group, property::ROTATION_Y, Value::F64(-25.0));
        let layer = shape(&mut doc, 2, Some(group));
        put(&mut doc, layer, property::POSITION, Value::Vec2([1200.0, 300.0]));
        put(&mut doc, layer, property::POSITION_Z, Value::F64(260.0));
        put(&mut doc, layer, property::ROTATION_X, Value::F64(30.0));
        put(&mut doc, layer, property::ROTATION_Y, Value::F64(-50.0));
        put(&mut doc, layer, property::ROTATION, Value::F64(15.0));
        put(&mut doc, layer, property::SCALE, Value::Vec2([1.4, 0.8]));
        put(&mut doc, layer, property::ANCHOR, Value::Vec2([40.0, 20.0]));
        for to in [LayerProjection::ThreeD, LayerProjection::TwoD, LayerProjection::TwoPointFiveD, LayerProjection::TwoD, LayerProjection::ThreeD] {
            switch(&mut doc, layer, to);
        }
        assert!(scalar(&doc, layer, property::ROTATION_X).abs() > 1.0, "the tilt survives as a value");
    }

    #[test]
    fn flat_layer_at_rest_switches_without_touching_values_and_undoes_in_one_step() {
        let mut doc = blank_project();
        let layer = shape(&mut doc, 1, None);
        put(&mut doc, layer, property::POSITION, Value::Vec2([400.0, 300.0]));
        put(&mut doc, layer, property::ROTATION, Value::F64(15.0));
        let initial = doc.view().attrs(layer).unwrap().unwrap().projection;
        switch(&mut doc, layer, LayerProjection::TwoD);
        assert_eq!(scalar(&doc, layer, property::ROTATION), 15.0);
        assert_eq!(scalar(&doc, layer, property::POSITION_Z), 0.0);
        assert!(doc.undo());
        assert_eq!(doc.view().attrs(layer).unwrap().unwrap().projection, initial);
        assert_eq!(scalar(&doc, layer, property::ROTATION), 15.0);
    }

    #[test]
    fn animated_flat_layer_with_depth_keeps_its_keys_when_leaving_two_d() {
        let mut doc = blank_project();
        let layer = shape(&mut doc, 1, None);
        let mut track = KeyframeTrack::new();
        for (frame, x) in [(0, 100.0), (60, 700.0)] {
            track.insert(Keyframe { t: RationalTime::try_from_frame(frame, crate::doc::core::Fps::try_new(30, 1).unwrap()).unwrap(), value: Value::Vec2([x, 300.0]), interp: Interp::Linear, spatial: None });
        }
        doc.apply(Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        put(&mut doc, layer, property::POSITION_Z, Value::F64(250.0));
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } }).unwrap();
        switch(&mut doc, layer, LayerProjection::ThreeD);
        let keys = doc.view().track(layer, &PropertyId::new(property::POSITION).unwrap()).unwrap().unwrap().keys().len();
        assert_eq!(keys, 2);
        assert!(scalar(&doc, layer, property::POSITION_Z).abs() < 1e-3, "2D drew it on z=0, so 3D places it there");
        put(&mut doc, layer, property::ROTATION_X, Value::F64(30.0));
        let err = doc.set_projection(&[(layer, center())], LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() }, RationalTime::ZERO).unwrap_err();
        assert!(err.to_string().contains("animated"), "{err}");
    }
}
