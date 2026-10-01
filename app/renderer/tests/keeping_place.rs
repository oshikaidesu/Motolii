//! 見た目を保つ編集の検査 — 束ねを解く・親を移す・札を変える。
//! どれも「どこに見えているか」を要るので、答える口を持つ絵の家で試す。

#[cfg(test)]
mod move_layer_tests {

    use motolii_edit::{blank_project, Animate, Document, Intent};
    /// この検査は「見た目を保つ」補正を見るので、答える口を自分で登録する
    /// (コアの既定は誰も答えない = 書いた値がそのまま残る)。
    fn placed() -> Document {
        use motolii_render::picture::resolve::{camera, transform};
        blank_project().with_geometry(motolii_doc::store::geometry::Geometry {
            local: transform::local_transform,
            local3d: transform::local_transform3d,
            world: transform::world_transform3d,
            worlds: transform::world_transforms3d,
            camera: camera::resolve_camera,
        })
    }
    use motolii_doc::store::*;
    use motolii_doc::store::{property};

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
        let mut doc = placed();
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
        let mut doc = placed();
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
        let mut doc = placed();
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
        let before = motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, at).unwrap();
        let history = doc.history_depth().0;
        doc.move_layers(&[child], Some(target), "inside", at)
            .unwrap();
        close(before, motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, at).unwrap());
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
        close(before, motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, at).unwrap());
    }

    #[test]
    fn identity_parent_accepts_animation_but_rotated_coordinate_system_does_not() {
        let mut doc = placed();
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
        let mut doc = placed();
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
        let mut doc = placed();
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
        let before = motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, at).unwrap();
        let history = doc.history_depth().0;
        doc.move_layers(&[child], Some(parent), "inside", at)
            .unwrap();
        close(before, motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, at).unwrap());
        assert_eq!(doc.history_depth().0, history + 1);
        assert!(doc
            .view()
            .track(child, &PropertyId::new(property::POSITION).unwrap())
            .unwrap()
            .is_none());
        let future = motolii_render::picture::resolve::transform::world_transform3d(&doc
            .view(), child, RationalTime::from_seconds(1))
            .unwrap()
            .transform_point3(glam::Vec3::ZERO);
        assert!((future.x - 250.0).abs() < 0.001);
        assert!((future.y - 70.0).abs() < 0.001);
        assert!(doc.undo());
        assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent, None);
        close(before, motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, at).unwrap());
    }

    #[test]
    fn translation_reparent_keeps_animated_spatial_keys_and_world_motion() {
        let mut doc = placed();
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
                spatial: Some(motolii_doc::store::SpatialTangent {
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
        let before = times.map(|t| motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, t).unwrap());
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
            close(before, motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, t).unwrap());
        }
        assert!(doc.undo());
        assert_eq!(doc.view().track(child, &property).unwrap().unwrap(), track);
        assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent, None);
    }

    #[test]
    fn translation_reparent_preserves_separate_xyz_tracks_and_constants() {
        let mut doc = placed();
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
        let before = times.map(|t| motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, t).unwrap());
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
            close(before, motolii_render::picture::resolve::transform::world_transform3d(&doc.view(), child, t).unwrap());
        }
        let moved = doc.view().track(child, &x).unwrap().unwrap();
        assert_eq!(moved.keys()[0].value, Value::F64(-49.0));
        assert_eq!(moved.keys()[1].value, Value::F64(-39.0));
        assert!(doc.undo());
        assert_eq!(doc.view().track(child, &x).unwrap().unwrap(), xt);
        assert_eq!(doc.view().track(child, &z).unwrap().unwrap(), zt);
    }
}

#[cfg(test)]
mod projection_switch_tests {
    use motolii_edit::{blank_project, Document, Intent};

    /// この検査は「見た目を保つ」補正を見るので、答える口を自分で登録する
    /// (コアの既定は誰も答えない = 書いた値がそのまま残る)。
    fn placed() -> Document {
        use motolii_render::picture::resolve::{camera, transform};
        blank_project().with_geometry(motolii_doc::store::geometry::Geometry {
            local: transform::local_transform,
            local3d: transform::local_transform3d,
            world: transform::world_transform3d,
            worlds: transform::world_transforms3d,
            camera: camera::resolve_camera,
        })
    }
    use motolii_doc::store::*;
    use motolii_doc::store::PropertyId;
    use motolii_doc::core::{projected_screen_corners, ResolvedCamera};
    use motolii_doc::eval::{Interp, Keyframe, KeyframeTrack};
    use motolii_doc::store::{LayerMeta, LayerSource, LayerTiming};

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
        motolii_render::picture::resolve::camera::resolve_camera(&doc.view(), RationalTime::ZERO).unwrap()
    }
    fn center() -> [f32; 3] {
        std::array::from_fn(|i| (MIN[i] + MAX[i]) * 0.5)
    }
    fn corners(doc: &Document, layer: LayerId) -> [glam::Vec2; 8] {
        let view = doc.view();
        let comp = view.composition().unwrap().unwrap().spec();
        let camera = motolii_render::picture::resolve::camera::resolve_camera(&view, RationalTime::ZERO).unwrap();
        let attrs = view.attrs(layer).unwrap().unwrap_or_default();
        let world = motolii_render::picture::resolve::transform::world_transform3d(&view, layer, RationalTime::ZERO).unwrap();
        projected_screen_corners(comp, camera, camera, attrs.projection, world, MIN, MAX)
    }
    /// 中心が画面のどこに映るか(法: 札を変えても中心は動かない。面の向きは札の意味に従う)。
    fn center_on_screen(doc: &Document, layer: LayerId) -> glam::Vec2 {
        let view = doc.view();
        let comp = view.composition().unwrap().unwrap().spec();
        let camera = motolii_render::picture::resolve::camera::resolve_camera(&view, RationalTime::ZERO).unwrap();
        let attrs = view.attrs(layer).unwrap().unwrap_or_default();
        let world = motolii_render::picture::resolve::transform::world_transform3d(&view, layer, RationalTime::ZERO).unwrap();
        projected_screen_corners(comp, camera, camera, attrs.projection, world, center(), center())[0]
    }
    fn switch(doc: &mut Document, layer: LayerId, to: LayerProjection) {
        let before = center_on_screen(doc, layer);
        doc.set_projection(&[(layer, center())], LayerAttrsPatch { projection: Some(to), ..Default::default() }, RationalTime::ZERO).unwrap();
        assert_eq!(doc.view().attrs(layer).unwrap().unwrap().projection, to);
        let after = center_on_screen(doc, layer);
        assert!(before.distance(after) < 0.05, "{to:?}: center moved {before:?} -> {after:?}");
    }
    fn scalar(doc: &Document, layer: LayerId, name: &str) -> f64 {
        match doc.view().value_at(layer, &PropertyId::new(name).unwrap(), RationalTime::ZERO).unwrap() {
            Some(Value::F64(v)) => v,
            None => 0.0,
            other => panic!("{name}: {other:?}"),
        }
    }

    #[test]
    fn tilted_child_under_a_moved_camera_keeps_its_center_through_every_switch() {
        let mut doc = placed();
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
        assert!((scalar(&doc, layer, property::ROTATION_X) - 30.0).abs() < 1e-6, "rotation is not rewritten by a projection change");
        let _ = corners(&doc, layer);
    }

    #[test]
    fn flat_layer_at_rest_switches_without_touching_values_and_undoes_in_one_step() {
        let mut doc = placed();
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
        let mut doc = placed();
        let layer = shape(&mut doc, 1, None);
        let mut track = KeyframeTrack::new();
        for (frame, x) in [(0, 100.0), (60, 700.0)] {
            track.insert(Keyframe { t: RationalTime::try_from_frame(frame, motolii_doc::core::Fps::try_new(30, 1).unwrap()).unwrap(), value: Value::Vec2([x, 300.0]), interp: Interp::Linear, spatial: None });
        }
        doc.apply(Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
        put(&mut doc, layer, property::POSITION_Z, Value::F64(250.0));
        doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } }).unwrap();
        switch(&mut doc, layer, LayerProjection::ThreeD);
        let keys = doc.view().track(layer, &PropertyId::new(property::POSITION).unwrap()).unwrap().unwrap().keys().len();
        assert_eq!(keys, 2);
        assert!(scalar(&doc, layer, property::POSITION_Z).abs() < 1e-3, "2D drew it on z=0, so 3D places it there");
        // 傾いていて位置が animate されていても切り替えられる(回転は触らず、中心だけ全 key をずらす)。
        put(&mut doc, layer, property::ROTATION_X, Value::F64(30.0));
        camera(&mut doc);
        switch(&mut doc, layer, LayerProjection::TwoD);
        let keys = doc.view().track(layer, &PropertyId::new(property::POSITION).unwrap()).unwrap().unwrap().keys().len();
        assert_eq!(keys, 2, "the animation survives the switch");
        assert!((scalar(&doc, layer, property::ROTATION_X) - 30.0).abs() < 1e-6);
    }
}

