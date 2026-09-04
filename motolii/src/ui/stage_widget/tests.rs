mod plane_tests {
    use crate::ui::stage_widget::*;

    // 隅だけ合っていても、**内側がずれていれば掴めない**。描く側と掴む側は
    // この1枚を共有しているので、任意の傾きの任意の点で往復できなければ、
    // 絵と当たり判定が別の場所に居る。
    //
    // 生成する四角形は角度を回り順にして凸に保つ。潰れた四角形では写像が
    // 定義できないので、面積に下限を置く。
    proptest::proptest! {
        #[test]
        fn any_point_on_any_tilted_plane_comes_back_where_it_started(
            cx in -500.0f64..500.0,
            cy in -500.0f64..500.0,
            radii in proptest::array::uniform4(30.0f64..400.0),
            wobble in proptest::array::uniform4(-0.6f64..0.6),
            probes in proptest::collection::vec((0.0f64..1.0, 0.0f64..1.0), 1..12),
        ) {
            let corner = |i: usize| {
                let a = std::f64::consts::FRAC_PI_4 + i as f64 * std::f64::consts::FRAC_PI_2 + wobble[i];
                glam::dvec2(cx + radii[i] * a.cos(), cy + radii[i] * a.sin())
            };
            let p = [corner(0), corner(1), corner(2), corner(3)];

            let area: f64 = (0..4)
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % 4]);
                    a.x * b.y - b.x * a.y
                })
                .sum::<f64>()
                .abs()
                * 0.5;
            proptest::prop_assume!(area > 2_000.0);

            let m = homography_from_unit_square(p);
            let inv = m.inverse();
            for (u, v) in probes {
                let (sx, sy) = apply_h(&m, u, v);
                proptest::prop_assume!(sx.is_finite() && sy.is_finite());
                let (bu, bv) = apply_h(&inv, sx, sy);
                proptest::prop_assert!(
                    (bu - u).abs() < 1e-6 && (bv - v).abs() < 1e-6,
                    "({u:.3},{v:.3}) が ({bu:.3},{bv:.3}) で戻ってきた。隅={p:?}"
                );
            }
        }
    }
}

mod gizmo_reach {
    use crate::ui::stage_widget::*;

    // **箱の真ん中はいつでも動かせなければならない。**
    //
    // 取っ手の許容は画面の画素で決まるので、画面上で小さい層では許容が箱より
    // 広くなる。実測: 1920 の枠を 198px で映すと 64px の層は画面上 6.6px、
    // 角の許容は 77 comp px —— 箱(64)より広く、掴んだ所は必ず角になる。
    // 試し手はこれで「動かず巨大化した」。
    proptest::proptest! {
        #[test]
        fn the_middle_of_the_box_is_always_a_move(
            // **画面上の大きさで振る。** comp の大きさで振ると、画面で小さく
            // 映る組み合わせが千に一つしか出ず、当たりの領域を素通りする。
            screen_w in 0.5f64..400.0,
            screen_h in 0.5f64..400.0,
            bx in -2000.0f64..2000.0,
            by in -2000.0f64..2000.0,
            scale in 0.01f64..8.0,
            rings in proptest::bool::ANY,
        ) {
            let (bw, bh) = (screen_w / scale, screen_h / scale);
            let mode = gizmo_mode_at(bx + bw * 0.5, by + bh * 0.5, (bx, by, bw, bh), scale, rings);
            proptest::prop_assert_eq!(
                mode,
                Some(GizmoMode::Move),
                "画面上 {:.1}x{:.1} px の箱の真ん中が掴めない(拡大率 {:.3})",
                screen_w, screen_h, scale
            );
        }

        /// 片側だけ守っても道具にならない。**画面で十分に大きい箱では、角は
        /// 角のまま**掴めなければ拡縮ができない。
        #[test]
        fn a_big_enough_box_still_has_grabbable_corners(
            screen_w in 80.0f64..2000.0,
            screen_h in 80.0f64..2000.0,
            bx in -2000.0f64..2000.0,
            by in -2000.0f64..2000.0,
            scale in 0.05f64..8.0,
        ) {
            let (bw, bh) = (screen_w / scale, screen_h / scale);
            let mode = gizmo_mode_at(bx, by, (bx, by, bw, bh), scale, false);
            proptest::prop_assert_eq!(
                mode,
                Some(GizmoMode::ScaleCorner { sx: false, sy: false }),
                "画面上 {:.0}x{:.0} px の箱の角が掴めない(拡大率 {:.3})",
                screen_w, screen_h, scale
            );
        }
    }
}

mod previews {
    use crate::ui::stage_widget::*;

    fn drag(mode: GizmoMode) -> GizmoDrag {
        GizmoDrag {
            owner: 0,
            layer: LayerId(1),
            mode,
            grab: (100.0, 100.0),
            orig_position: (100.0, 100.0),
            orig_rotation: 0.0,
            orig_rotation_xy: (0.0, 0.0),
            orig_z: 0.0,
            anchor: (0.5, 0.5),
            natural: (200.0, 100.0),
            orig_box: (0.0, 50.0, 200.0, 100.0),
            fit_z: 0.0,
            projection: LayerProjection::ThreeD,
            orig_placement: crate::doc::core::LayerPlacement::default(),
            last: None,
            preview: Vec::new(),
            original_values: Vec::new(),
            at: RationalTime::ZERO,
            others: vec![(
                LayerId(2),
                SelGeom {
                    projection: LayerProjection::ThreeD,
                    placement: crate::doc::core::LayerPlacement::default(),
                    z: 50.0,
                    rotation_x: 20.0,
                    rotation_y: 30.0,
                    position: (300.0, 300.0),
                    anchor: (0.5, 0.5),
                    rotation: 40.0,
                    natural: (100.0, 50.0),
                    box_: (250.0, 275.0, 100.0, 50.0),
                },
            )],
        }
    }

    /// Shift は支配軸に固定し、一緒に選んだ層も同じ差分で動く。
    #[test]
    fn shift_locks_the_dominant_axis_and_carries_companions() {
        let out = preview_values(&drag(GizmoMode::Move), (130.0, 108.0), true, false, 1.0);
        assert_eq!(
            out[0],
            (
                LayerId(1),
                PropertyId::new(property::POSITION).unwrap(),
                Value::Vec2([130.0, 100.0])
            )
        );
        assert_eq!(
            out[1],
            (
                LayerId(2),
                PropertyId::new(property::POSITION).unwrap(),
                Value::Vec2([330.0, 300.0])
            )
        );
    }

    /// 軌道は掴んだ輪の軸だけ書く。上へ引くと奥へ。
    #[test]
    fn orbit_touches_one_axis_and_depth_goes_up() {
        let out = preview_values(
            &drag(GizmoMode::Orbit { axis_x: true }),
            (100.0, 140.0),
            false,
            false,
            1.0,
        );
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].1, PropertyId::new(property::ROTATION_X).unwrap());
        let out = preview_values(&drag(GizmoMode::Depth), (100.0, 60.0), false, false, 1.0);
        assert_eq!(
            out[0],
            (
                LayerId(1),
                PropertyId::new(property::POSITION_Z).unwrap(),
                Value::F64(40.0)
            )
        );
    }

    #[test]
    fn every_transform_lifts_from_each_original_pose() {
        let orbit = preview_values(
            &drag(GizmoMode::Orbit { axis_x: true }),
            (100.0, 140.0),
            false,
            false,
            1.0,
        );
        assert_eq!(
            orbit[1],
            (
                LayerId(2),
                PropertyId::new(property::ROTATION_X).unwrap(),
                Value::F64(0.0)
            )
        );
        let depth = preview_values(&drag(GizmoMode::Depth), (100.0, 60.0), false, false, 1.0);
        assert_eq!(
            depth[1],
            (
                LayerId(2),
                PropertyId::new(property::POSITION_Z).unwrap(),
                Value::F64(90.0)
            )
        );
        let rotate = preview_values(&drag(GizmoMode::Rotate), (140.0, 100.0), false, false, 1.0);
        let Value::F64(primary) = rotate[0].2 else {
            panic!("rotation")
        };
        assert_eq!(
            rotate[1],
            (
                LayerId(2),
                PropertyId::new(property::ROTATION).unwrap(),
                Value::F64(40.0 + primary)
            )
        );
        let scale = preview_values(
            &drag(GizmoMode::ScaleCorner { sx: true, sy: true }),
            (400.0, 250.0),
            false,
            false,
            1.0,
        );
        assert_eq!(
            scale[2],
            (
                LayerId(2),
                PropertyId::new(property::SCALE).unwrap(),
                scale[0].2.clone()
            )
        );
    }

    /// 矢印は今の位置に差分を足す。
    #[test]
    fn nudge_adds_to_the_current_position() {
        let loaded = crate::ui::fixture::load_fixture();
        let mut doc = loaded.doc;
        let layer = doc.view().layers()[0];
        let prop = PropertyId::new(property::POSITION).unwrap();
        let before = match doc
            .view()
            .value_at(layer, &prop, RationalTime::ZERO)
            .unwrap()
        {
            Some(Value::Vec2(v)) => v,
            _ => [0.0, 0.0],
        };
        doc.apply_all(nudge_intents(&doc, &[layer], (10.0, -1.0), RationalTime::ZERO).unwrap())
            .unwrap();
        let after = doc
            .view()
            .value_at(layer, &prop, RationalTime::ZERO)
            .unwrap();
        assert_eq!(
            after,
            Some(Value::Vec2([before[0] + 10.0, before[1] - 1.0]))
        );
    }

    #[test]
    fn superseded_preview_still_releases_stage_gesture_ownership() {
        let doc = Arc::new(Mutex::new(Document::new()));
        let owner = doc.lock().unwrap().begin_preview();
        let _newer = doc.lock().unwrap().begin_preview();
        let gesture = GestureSurface::default();
        gesture.begin();

        assert!(!finish_preview_owner(&doc, &gesture, owner));
        assert!(!gesture.is_active());
    }
}

mod marquee_contract {
    use crate::doc::store::LayerId;
    use crate::ui::session::Selection;
    use crate::ui::stage_widget::{apply_marquee_selection, marquee_hits};
    use dioxus_native::prelude::*;

    #[test]
    fn marquee_intersection_is_independent_of_drag_direction() {
        let layer = [(20.0, 20.0), (60.0, 20.0), (60.0, 60.0), (20.0, 60.0)];
        assert!(marquee_hits((0.0, 0.0), (40.0, 40.0), layer));
        assert!(marquee_hits((40.0, 0.0), (0.0, 40.0), layer));
        assert!(marquee_hits((0.0, 0.0), (80.0, 80.0), layer));
        assert!(!marquee_hits((0.0, 0.0), (10.0, 10.0), layer));
    }

    #[test]
    fn additive_marquee_preserves_selection_and_plain_empty_click_clears_it() {
        let selection = Selection::default();
        selection.set(Some(LayerId(1)));
        apply_marquee_selection(&selection, true, Some(vec![LayerId(2)]));
        assert_eq!(selection.all(), vec![LayerId(1), LayerId(2)]);

        apply_marquee_selection(&selection, true, None);
        assert_eq!(selection.all(), vec![LayerId(1), LayerId(2)]);
        apply_marquee_selection(&selection, false, Some(Vec::new()));
        assert!(
            selection.all().is_empty(),
            "an empty completed marquee must clear"
        );
        selection.set(Some(LayerId(1)));
        apply_marquee_selection(&selection, false, None);
        assert!(
            selection.all().is_empty(),
            "a sub-slop plain click must clear"
        );
    }

    fn signal<T: 'static>(value: T) -> dioxus_native::prelude::Signal<T> {
        dioxus_native::prelude::Signal::leak_with_caller(value, std::panic::Location::caller())
    }

    fn pointer(
        id: blitz_traits::events::BlitzPointerId,
    ) -> blitz_traits::events::BlitzPointerEvent {
        blitz_traits::events::BlitzPointerEvent {
            id,
            is_primary: true,
            coords: blitz_traits::events::PointerCoords {
                page_x: 100.0,
                page_y: 100.0,
                screen_x: 100.0,
                screen_y: 100.0,
                client_x: 100.0,
                client_y: 100.0,
            },
            button: blitz_traits::events::MouseEventButton::Main,
            buttons: blitz_traits::events::MouseEventButtons::Primary,
            mods: Default::default(),
            details: Default::default(),
            element: blitz_traits::events::Point { x: 100.0, y: 100.0 },
            active_pointers: Default::default(),
        }
    }

    #[test]
    fn matching_pointer_cancel_rolls_back_marquee_and_foreign_cancel_is_ignored() {
        use crate::doc::store::{Composition, Document, Fps, Intent};
        use crate::ui::mount::SurfaceState;
        use crate::ui::session::{GestureSurface, SurfaceCapture};
        use crate::ui::stage_widget::{StageBindings, StageMount, StageState};
        use std::sync::{Arc, Mutex};

        let runtime = VirtualDom::new(|| rsx! {});
        runtime.in_scope(dioxus_native::prelude::dioxus_core::ScopeId::ROOT, || {
            let mut doc = Document::new();
            doc.apply(Intent::SetComposition(Composition {
                width: 640,
                height: 480,
                fps: Fps::try_new(30, 1).unwrap(),
                duration_frames: 300,
                background: [0.0, 0.0, 0.0, 1.0],
            }))
            .unwrap();
            let clock = Arc::new(crate::ui::playback::Clock::from_document(&doc, 10.0));
            let doc = Arc::new(Mutex::new(doc));
            let selection = Selection::default();
            selection.set(Some(LayerId(99)));
            let gesture = GestureSurface::default();
            let mut state = StageState::new(
                clock,
                doc,
                selection.clone(),
                Arc::new(Mutex::new(None)),
                Arc::new(Mutex::new(Default::default())),
                Arc::new(std::sync::atomic::AtomicBool::new(true)),
                Arc::new(std::sync::atomic::AtomicU32::new(75)),
                gesture.clone(),
                SurfaceCapture::default(),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
                Arc::new(Mutex::new(None)),
            );
            let bindings = StageBindings {
                selected: signal(Some(LayerId(99))),
                revision: signal(0),
                view_pct: signal(100),
                context_menu: signal(None),
            };
            let mut mount = StageMount::default();
            let first = pointer(blitz_traits::events::BlitzPointerId::Finger(1));
            <StageState as SurfaceState>::handle_event(
                &mut state,
                &mut mount,
                &bindings,
                &blitz_traits::events::UiEvent::PointerDown(first),
            );
            assert!(gesture.is_active());
            assert!(state.marquee.is_some());

            let mut foreign = pointer(blitz_traits::events::BlitzPointerId::Finger(2));
            foreign.buttons = blitz_traits::events::MouseEventButtons::None;
            <StageState as SurfaceState>::handle_event(
                &mut state,
                &mut mount,
                &bindings,
                &blitz_traits::events::UiEvent::PointerCancel(foreign),
            );
            assert!(gesture.is_active() && state.marquee.is_some());

            let mut matching = pointer(blitz_traits::events::BlitzPointerId::Finger(1));
            matching.buttons = blitz_traits::events::MouseEventButtons::None;
            <StageState as SurfaceState>::handle_event(
                &mut state,
                &mut mount,
                &bindings,
                &blitz_traits::events::UiEvent::PointerCancel(matching),
            );
            assert!(!gesture.is_active());
            assert!(state.marquee.is_none());
            assert_eq!(selection.all(), vec![LayerId(99)]);
        });
    }
}

mod checked_placement_commands {
    use crate::doc::store::*;
    use crate::ui::{stage_widget::nudge_intents, utility};
    use std::sync::{Arc, Mutex};

    fn doc() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 480,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: Composition::default_background(),
        }))
        .unwrap();
        for layer in [LayerId(1), LayerId(2)] {
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order: layer.0 as i16,
                        timing: LayerTiming::place(0, None, 300),
                    },
                },
            ])
            .unwrap();
        }
        doc
    }

    #[test]
    fn nudge_keeps_key_interpolation_and_zero_delta_creates_no_key() {
        let mut doc = doc();
        let layer = LayerId(1);
        let property = PropertyId::new(property::POSITION).unwrap();
        let original = Keyframe {
            t: RationalTime::ZERO,
            value: Value::Vec2([10.0, 20.0]),
            interp: Interp::Hold,
            spatial: None,
        };
        let mut track = KeyframeTrack::new();
        track.insert(original.clone());
        doc.apply(Intent::SetTrack {
            layer,
            property: property.clone(),
            track,
        })
        .unwrap();
        let head = doc.edit_head();
        doc.apply_all(nudge_intents(&doc, &[layer], (10.0, -1.0), RationalTime::ZERO).unwrap())
            .unwrap();
        let changed = doc.view().track(layer, &property).unwrap().unwrap().keys()[0].clone();
        assert_eq!(changed.value, Value::Vec2([20.0, 19.0]));
        assert_eq!(changed.interp, original.interp);
        assert_eq!(doc.edit_head(), head + 1);
        assert!(
            nudge_intents(&doc, &[layer], (0.0, 0.0), RationalTime::from_seconds(1))
                .unwrap()
                .is_empty()
        );
        assert!(doc.undo());
        assert_eq!(
            doc.view().track(layer, &property).unwrap().unwrap().keys(),
            &[original]
        );
    }

    #[test]
    fn rejected_shared_or_driven_position_never_partially_moves_anchor_or_other_layers() {
        for driven in [false, true] {
            let mut doc = doc();
            let layer = LayerId(1);
            let property = PropertyId::new(property::POSITION).unwrap();
            if driven {
                doc.apply_all([
                    Intent::SetConstant {
                        layer: LayerId(2),
                        property: property.clone(),
                        value: Value::Vec2([10.0, 20.0]),
                    },
                    Intent::SetPropertyLink {
                        layer,
                        property: property.clone(),
                        link: PropertyLink {
                            source_layer: LayerId(2),
                            source_property: property.clone(),
                            time_offset: RationalTime::ZERO,
                            plugin_id: "motolii.link.identity".into(),
                            params: Vec::new(),
                        },
                    },
                ])
                .unwrap();
            } else {
                let mut track = KeyframeTrack::new();
                track.insert(Keyframe {
                    t: RationalTime::ZERO,
                    value: Value::Vec2([10.0, 20.0]),
                    interp: Interp::Hold,
                    spatial: None,
                });
                let slot = SlotId("position".into());
                doc.apply_all([
                    Intent::SetSlots {
                        slots: vec![Slot {
                            id: slot.clone(),
                            track,
                        }],
                    },
                    Intent::SetPropertySlot {
                        layer,
                        property: property.clone(),
                        slot,
                    },
                ])
                .unwrap();
            }
            let source = doc.view().property_source(layer, &property).unwrap();
            let head = doc.edit_head();
            assert!(
                nudge_intents(&doc, &[LayerId(2), layer], (10.0, 0.0), RationalTime::ZERO).is_err()
            );
            let doc = Arc::new(Mutex::new(doc));
            assert!(utility::move_anchor(
                &doc,
                layer,
                [200.0, 100.0],
                RationalTime::ZERO,
                0.5,
                0.5
            )
            .is_err());
            let doc = doc.lock().unwrap();
            assert_eq!(doc.edit_head(), head);
            assert_eq!(
                doc.view().property_source(layer, &property).unwrap(),
                source
            );
            assert!(doc
                .view()
                .property_source(layer, &PropertyId::new(property::ANCHOR).unwrap())
                .unwrap()
                .is_none());
        }
    }

    #[test]
    fn anchor_position_composition_preserves_rotated_skewed_geometry_with_one_undo() {
        let mut doc = doc();
        let layer = LayerId(1);
        let position = PropertyId::new(property::POSITION).unwrap();
        let anchor = PropertyId::new(property::ANCHOR).unwrap();
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::Vec2([100.0, 120.0]),
            interp: Interp::Hold,
            spatial: None,
        });
        doc.apply_all([
            Intent::SetTrack {
                layer,
                property: position.clone(),
                track,
            },
            Intent::SetConstant {
                layer,
                property: anchor.clone(),
                value: Value::Vec2([10.0, 20.0]),
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::SCALE).unwrap(),
                value: Value::Vec2([2.0, 3.0]),
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::ROTATION).unwrap(),
                value: Value::F64(30.0),
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::SKEW).unwrap(),
                value: Value::F64(15.0),
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::SKEW_AXIS).unwrap(),
                value: Value::F64(20.0),
            },
        ])
        .unwrap();
        let before = doc
            .view()
            .local_transform(layer, RationalTime::ZERO)
            .unwrap();
        let head = doc.edit_head();
        let doc = Arc::new(Mutex::new(doc));
        utility::move_anchor(&doc, layer, [200.0, 100.0], RationalTime::ZERO, 0.5, 0.5).unwrap();
        {
            let doc = doc.lock().unwrap();
            let after = doc
                .view()
                .local_transform(layer, RationalTime::ZERO)
                .unwrap();
            for point in [
                glam::Vec2::ZERO,
                glam::vec2(100.0, 0.0),
                glam::vec2(0.0, 80.0),
                glam::vec2(20.0, 40.0),
            ] {
                assert!(
                    (before.transform_point2(point) - after.transform_point2(point)).length()
                        < 0.001
                );
            }
            assert_eq!(doc.edit_head(), head + 1);
            assert_eq!(
                doc.view().track(layer, &position).unwrap().unwrap().keys()[0].interp,
                Interp::Hold
            );
        }
        utility::move_anchor(&doc, layer, [200.0, 100.0], RationalTime::ZERO, 0.5, 0.5).unwrap();
        let mut doc = doc.lock().unwrap();
        assert_eq!(doc.edit_head(), head + 1, "the same anchor is a no-op");
        assert!(doc.undo());
        assert_eq!(
            doc.view()
                .value_at(layer, &anchor, RationalTime::ZERO)
                .unwrap(),
            Some(Value::Vec2([10.0, 20.0]))
        );
        assert_eq!(
            doc.view()
                .value_at(layer, &position, RationalTime::ZERO)
                .unwrap(),
            Some(Value::Vec2([100.0, 120.0]))
        );
    }
}
