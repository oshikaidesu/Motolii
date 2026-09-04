#![allow(clippy::all)]
mod follow {
    #[allow(unused_imports)]
    use crate::doc::store::*;
    #[allow(unused_imports)]
    use crate::ui::timeline_edit::*;
    use crate::ui::timeline_widget::*;

    proptest::proptest! {
        /// 止まっている間は、どこへ動かしても引き戻されない。
        /// **利用者が自分で動かした位置が真**で、再生位置ではない。
        #[test]
        fn a_paused_timeline_stays_where_you_put_it(
            scroll in 0.0f64..600.0,
            visible in 1.0f64..120.0,
            playhead in 0.0f64..600.0,
        ) {
            proptest::prop_assert_eq!(
                follow_playhead(scroll, visible, playhead, false, false),
                None
            );
        }

        /// 再生中に視界から出たら追いかける(こちらは効いていないと困る)。
        #[test]
        fn a_playing_timeline_catches_up_when_the_head_leaves(
            scroll in 10.0f64..600.0,
            visible in 1.0f64..120.0,
        ) {
            let behind = scroll - 1.0;
            proptest::prop_assert!(
                follow_playhead(scroll, visible, behind, true, false).is_some(),
                "再生中に置いていかれた"
            );
        }
    }
}

mod lyrics {
    #[allow(unused_imports)]
    use crate::doc::store::*;
    use crate::doc::store::{ContentKeyframe, ContentTrack, Intent, LayerSource};
    #[allow(unused_imports)]
    use crate::ui::timeline_edit::*;
    use crate::ui::timeline_widget::*;

    fn text_layer(doc: &Document) -> LayerId {
        let view = doc.view();
        view.layers()
            .into_iter()
            .find(|l| view.meta(*l).ok().flatten().map(|m| m.source) == Some(LayerSource::Text))
            .expect("a text layer in the fixture")
    }

    /// 歌詞の切替時刻は層と一緒に動く。
    #[test]
    fn moving_a_layer_carries_its_lyric_keys() {
        let loaded = crate::ui::fixture::load_fixture();
        let mut doc = loaded.doc;
        let layer = text_layer(&doc);
        let fps = document_fps(&doc).unwrap();
        let mut text = doc.view().text_document(layer).unwrap().unwrap();
        let mut track = ContentTrack::new();
        for (frame, word) in [(0, "a"), (24, "b")] {
            track.insert(ContentKeyframe {
                t: RationalTime::try_from_frame(frame, fps).unwrap(),
                content: word.into(),
            });
        }
        text.content = track;
        doc.apply(Intent::SetTextDocument {
            layer,
            document: text,
        })
        .unwrap();

        let intents = keyframe_shift_intents(&doc, layer, 10).unwrap();
        doc.apply_all(intents).unwrap();
        let keys = doc
            .view()
            .text_document(layer)
            .unwrap()
            .unwrap()
            .content
            .keys()
            .to_vec();
        let frames: Vec<i64> = keys
            .iter()
            .map(|k| k.t.try_to_frame_floor(fps).unwrap())
            .collect();
        assert_eq!(
            frames,
            vec![10, 34],
            "lyric keys did not move with the layer"
        );
    }

    /// 歌詞のキーが 1 つでも、層と一緒に動く(1 つの時だけ置き去りになっていた)。
    #[test]
    fn a_single_lyric_key_moves_with_the_layer_too() {
        let loaded = crate::ui::fixture::load_fixture();
        let mut doc = loaded.doc;
        let layer = text_layer(&doc);
        let fps = document_fps(&doc).unwrap();
        let mut text = doc.view().text_document(layer).unwrap().unwrap();
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe {
            t: RationalTime::try_from_frame(12, fps).unwrap(),
            content: "solo".into(),
        });
        text.content = track;
        doc.apply(Intent::SetTextDocument {
            layer,
            document: text,
        })
        .unwrap();

        doc.apply_all(keyframe_shift_intents(&doc, layer, 10).unwrap())
            .unwrap();
        let keys = doc
            .view()
            .text_document(layer)
            .unwrap()
            .unwrap()
            .content
            .keys()
            .to_vec();
        assert_eq!(keys[0].t.try_to_frame_floor(fps).unwrap(), 22);
        // 最後の 1 つは消えない(削除の map が None を返しても 0 個にはしない)。
        let gone = content_track_intent(&doc.view(), layer, fps, &[22], |_| None).unwrap();
        assert!(gone.is_none(), "the only lyric line must survive a delete");
    }

    #[test]
    fn grouped_delete_of_all_lyric_keys_retains_only_the_current_line() {
        let mut doc = crate::ui::fixture::load_fixture().doc;
        let layer = text_layer(&doc);
        let fps = document_fps(&doc).unwrap();
        let mut document = doc.view().text_document(layer).unwrap().unwrap();
        let styles = document.styles.clone();
        let mut content = ContentTrack::new();
        for (frame, text) in [(0, "a"), (24, "b"), (48, "c")] {
            content.insert(ContentKeyframe {
                t: RationalTime::try_from_frame(frame, fps).unwrap(),
                content: text.into(),
            });
        }
        document.content = content;
        doc.apply(Intent::SetTextDocument { layer, document })
            .unwrap();
        let selections: Vec<_> = [0, 24, 48]
            .into_iter()
            .map(|frame| (layer, None, frame as f64 / fps.as_f64()))
            .collect();
        let intents = delete_key_selection_intents(
            &doc,
            &selections,
            RationalTime::try_from_frame(36, fps).unwrap(),
        )
        .unwrap();
        doc.apply_all(intents).unwrap();
        let after = doc.view().text_document(layer).unwrap().unwrap();
        assert_eq!(after.content.keys().len(), 1);
        assert_eq!(after.content.keys()[0].content, "b");
        assert_eq!(after.styles, styles);
        assert!(doc.undo());
        assert_eq!(
            doc.view()
                .text_document(layer)
                .unwrap()
                .unwrap()
                .content
                .keys()
                .len(),
            3
        );
    }

    /// 目盛の段は倍率に付いて行き、詰まらず空きすぎない。
    #[test]
    fn ruler_steps_follow_the_zoom() {
        assert_eq!(nice_step(8.0), 10.0);
        assert_eq!(nice_step(80.0), 1.0);
        assert_eq!(nice_step(600.0), 0.2);
        for pps in [8.0, 20.0, 80.0, 200.0, 600.0] {
            let px = nice_step(pps) * pps;
            assert!((80.0..=800.0).contains(&px), "{pps}: {px}px");
        }
    }

    /// 印は吸い付き先。
    #[test]
    fn markers_are_snap_targets() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut w = TimelineWidget::new(Vec::new(), Rc::new(rx));
        w.markers = vec![1.5, 7.25];
        let targets = w.snap_targets();
        assert!(
            targets.contains(&1.5) && targets.contains(&7.25),
            "{targets:?}"
        );
    }
}

mod keys {
    #[allow(unused_imports)]
    use crate::doc::store::*;
    use crate::doc::store::{
        property, Composition, Interp, Keyframe, LayerSource, PropertyId, Value,
    };
    #[allow(unused_imports)]
    use crate::ui::timeline_edit::*;
    use crate::ui::timeline_widget::*;

    fn two_key_doc() -> (Document, LayerId, PropertyId, Fps) {
        let fps = Fps::try_new(24, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 480,
            fps,
            duration_frames: 240,
            background: Composition::default_background(),
        }))
        .unwrap();
        let layer = LayerId(1);
        let property = PropertyId::new(property::POSITION).unwrap();
        let mut track = KeyframeTrack::new();
        for (frame, x) in [(12, 1.0), (36, 3.0)] {
            track.insert(Keyframe {
                t: RationalTime::try_from_frame(frame, fps).unwrap(),
                value: Value::Vec2([x, 0.0]),
                interp: Interp::Linear,
                spatial: None,
            });
        }
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: 0,
                    timing: LayerTiming::place(0, None, 240),
                },
            },
            Intent::SetTrack {
                layer,
                property: property.clone(),
                track,
            },
        ])
        .unwrap();
        (doc, layer, property, fps)
    }

    fn frames(doc: &Document, layer: LayerId, property: &PropertyId, fps: Fps) -> Vec<i64> {
        doc.view()
            .track(layer, property)
            .unwrap()
            .unwrap()
            .keys()
            .iter()
            .map(|k| k.t.try_to_frame_round(fps).unwrap())
            .collect()
    }

    /// 同じ帯の 2 つを掴めば 2 つとも動く。別々の SetTrack だと最後の 1 本が勝っていた。
    #[test]
    fn two_keys_on_one_track_move_together() {
        let (mut doc, layer, property, fps) = two_key_doc();
        let intents = keyframe_move_intents(&doc, layer, Some(&property), &[12, 36], 5).unwrap();
        assert_eq!(intents.len(), 1, "one track, one SetTrack");
        doc.apply_all(intents).unwrap();
        assert_eq!(frames(&doc, layer, &property, fps), vec![17, 41]);
    }

    #[test]
    fn grouped_delete_unions_aggregate_and_property_keys_and_keeps_the_displayed_value() {
        let (mut doc, layer, property, fps) = two_key_doc();
        let at = RationalTime::try_from_frame(24, fps).unwrap();
        let before = doc.view().value_at(layer, &property, at).unwrap();
        let head = doc.edit_head();
        let intents = delete_key_selection_intents(
            &doc,
            &[
                (layer, None, 0.5),
                (layer, Some(property.clone()), 0.5),
                (layer, Some(property.clone()), 1.5),
            ],
            at,
        )
        .unwrap();
        assert_eq!(
            intents.len(),
            1,
            "overlapping selection addresses share one persistent track"
        );
        doc.apply_all(intents).unwrap();
        assert_eq!(doc.edit_head(), head + 1);
        assert!(
            doc.view().track(layer, &property).unwrap().is_none(),
            "both selected keys must be removed"
        );
        assert_eq!(
            doc.view().value_at(layer, &property, at).unwrap(),
            before,
            "removing every key keeps the value at the fixed command time"
        );
        assert!(doc.undo());
        assert_eq!(frames(&doc, layer, &property, fps), vec![12, 36]);
        assert!(doc.redo());
        assert!(doc.view().track(layer, &property).unwrap().is_none());
    }

    #[test]
    fn grouped_delete_preserves_unselected_key_metadata_and_undo_restores_all_selected_keys() {
        let (mut doc, layer, property, fps) = two_key_doc();
        let mut track = doc.view().track(layer, &property).unwrap().unwrap();
        let survivor = Keyframe {
            t: RationalTime::try_from_frame(60, fps).unwrap(),
            value: Value::Vec2([9.0, 2.0]),
            interp: Interp::Hold,
            spatial: None,
        };
        track.insert(survivor.clone());
        doc.apply(Intent::SetTrack {
            layer,
            property: property.clone(),
            track,
        })
        .unwrap();
        let head = doc.edit_head();
        let intents = delete_key_selection_intents(
            &doc,
            &[
                (layer, Some(property.clone()), 1.5),
                (layer, Some(property.clone()), 0.5),
            ],
            RationalTime::try_from_frame(24, fps).unwrap(),
        )
        .unwrap();
        doc.apply_all(intents).unwrap();
        assert_eq!(doc.edit_head(), head + 1);
        assert_eq!(
            doc.view().track(layer, &property).unwrap().unwrap().keys(),
            &[survivor]
        );
        assert!(doc.undo());
        assert_eq!(frames(&doc, layer, &property, fps), vec![12, 36, 60]);
    }

    /// Delete はその時刻のキーだけ外す。最後の 1 つを外すと値は定数で残り、絵は飛ばない。
    #[test]
    fn deleting_a_key_leaves_the_others_and_then_a_constant() {
        let (mut doc, layer, property, fps) = two_key_doc();
        doc.apply_all(keyframe_delete_intents(&doc, layer, Some(&property), 0.5).unwrap())
            .unwrap();
        assert_eq!(frames(&doc, layer, &property, fps), vec![36]);
        doc.apply_all(keyframe_delete_intents(&doc, layer, Some(&property), 1.5).unwrap())
            .unwrap();
        assert!(
            doc.view().track(layer, &property).unwrap().is_none(),
            "track should be gone"
        );
        let at = RationalTime::try_from_frame(36, fps).unwrap();
        assert_eq!(
            doc.view().value_at(layer, &property, at).unwrap(),
            Some(Value::Vec2([3.0, 0.0]))
        );
    }

    /// 切ると跨いだ区間のイージングが両側へ分かれ、切り口の値は両方に立つ。
    #[test]
    fn splitting_a_track_keeps_the_curve_on_both_sides() {
        let (doc, layer, property, fps) = two_key_doc();
        let mut eased = doc.view().track(layer, &property).unwrap().unwrap();
        let mut first = eased.keys()[0].clone();
        first.interp = Interp::Bezier {
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        };
        eased.insert(first);
        let cut = RationalTime::try_from_frame(24, fps).unwrap();
        let (head, tail) = split_track_at(&eased, cut);
        let head = head.expect("the cut straddles the only segment");
        assert_eq!(frames_of(&head, fps), vec![12, 24]);
        assert_eq!(frames_of(&tail, fps), vec![24, 36]);
        assert_ne!(head.keys()[0].interp, Interp::Linear);
        assert_ne!(tail.keys()[0].interp, Interp::Linear);
        assert_eq!(head.keys()[1].value, eased.eval(cut));
    }

    fn frames_of(track: &KeyframeTrack, fps: Fps) -> Vec<i64> {
        track
            .keys()
            .iter()
            .map(|k| k.t.try_to_frame_round(fps).unwrap())
            .collect()
    }

    /// 再生位置の擦りは印と帯の端へ吸う。
    #[test]
    fn scrubbing_snaps_to_markers_and_edges() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut w = TimelineWidget::new(Vec::new(), Rc::new(rx));
        w.markers = vec![2.0];
        assert_eq!(w.snapped_time(2.0 + SNAP_PX * 0.5 / w.pps), 2.0);
        assert_eq!(w.snapped_time(5.0), 5.0);
    }

    /// 複製は元のすぐ上に割り込み、上に居た層は退く。
    #[test]
    fn duplicate_slips_in_above_the_original() {
        let (doc, layer, _, _) = two_key_doc();
        let doc = Arc::new(Mutex::new(doc));
        {
            let mut d = doc.lock().unwrap();
            let upper = LayerId(2);
            d.apply_all([
                Intent::AddLayer(upper),
                Intent::SetMeta {
                    layer: upper,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order: 1,
                        timing: LayerTiming::place(0, None, 240),
                    },
                },
            ])
            .unwrap();
        }
        let copy = duplicate_layer(&doc, layer).unwrap();
        let d = doc.lock().unwrap();
        let order = |l: LayerId| d.view().meta(l).unwrap().unwrap().order;
        assert_eq!((order(layer), order(copy), order(LayerId(2))), (0, 1, 2));
    }
}

mod timebase {
    #[allow(unused_imports)]
    use crate::doc::store::*;
    use crate::doc::store::{
        property, Composition, Interp, Keyframe, LayerSource, PropertyId, Value,
    };
    #[allow(unused_imports)]
    use crate::ui::timeline_edit::*;
    use crate::ui::timeline_widget::*;

    #[test]
    fn moving_a_key_uses_the_composition_frame_rate() {
        let fps = Fps::try_new(24, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 480,
            fps,
            duration_frames: 240,
            background: Composition::default_background(),
        }))
        .unwrap();
        let layer = LayerId(1);
        let property = PropertyId::new(property::POSITION).unwrap();
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::try_from_frame(12, fps).unwrap(),
            value: Value::Vec2([1.0, 2.0]),
            interp: Interp::Linear,
            spatial: None,
        });
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: 0,
                    timing: LayerTiming::place(0, None, 240),
                },
            },
            Intent::SetTrack {
                layer,
                property: property.clone(),
                track,
            },
        ])
        .unwrap();

        doc.apply_all(keyframe_shift_intents(&doc, layer, 1).unwrap())
            .unwrap();
        let moved = doc.view().track(layer, &property).unwrap().unwrap();
        assert_eq!(moved.keys()[0].t.try_to_frame_round(fps).unwrap(), 13);
    }
}

mod markers_move {
    #[allow(unused_imports)]
    use crate::doc::store::*;
    use crate::doc::store::{Intent, Marker};
    #[allow(unused_imports)]
    use crate::ui::timeline_edit::*;
    use crate::ui::timeline_widget::*;

    /// 印は掴んで動かせ、Document の印が書き直される。名前は残る。
    #[test]
    fn a_dragged_marker_is_written_back_with_its_name() {
        let loaded = crate::ui::fixture::load_fixture();
        let mut doc = loaded.doc;
        let fps = document_fps(&doc).unwrap();
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        doc.apply(Intent::SetMarkers {
            markers: vec![
                Marker {
                    name: "verse".into(),
                    time: at(24),
                    duration: RationalTime::ZERO,
                    body: String::new(),
                },
                Marker {
                    name: "chorus".into(),
                    time: at(96),
                    duration: RationalTime::ZERO,
                    body: String::new(),
                },
            ],
        })
        .unwrap();
        let doc = Arc::new(Mutex::new(doc));
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut w =
            TimelineWidget::new(Vec::new(), Rc::new(rx)).with_document(doc.clone(), |_| Vec::new());
        w.fps = fps.as_f64();
        let second = 96.0 / fps.as_f64();
        w.markers = vec![1.0, second];
        w.marker_drag = Some((0, 1.0, 1.0, 1.5));
        assert_eq!(
            w.snapped_time_excluding(second + 0.01, 0),
            second,
            "other markers are snap targets"
        );
        assert_eq!(
            w.snapped_time_excluding(1.01, 0),
            1.01,
            "the dragged marker does not snap to itself"
        );
        w.finish_drag(&mut TimelineBindings::default());
        let markers = doc.lock().unwrap().view().markers().unwrap();
        assert_eq!(markers[0].name, "verse");
        assert_eq!(markers[0].time, at((2.5 * fps.as_f64()).round() as i64));
        assert_eq!(w.markers, vec![2.5, second]);
    }
}

mod selection_identity {
    #[allow(unused_imports)]
    use crate::doc::store::*;
    #[allow(unused_imports)]
    use crate::ui::timeline_edit::*;
    use crate::ui::timeline_widget::*;

    fn row(layer: u64, keys: Vec<f64>) -> CanvasRow {
        CanvasRow {
            is_group: false,
            keys,
            span: Some((0.0, 10.0)),
            agg: Vec::new(),
            layer: Some(LayerId(layer)),
            prop: None,
            color: [0, 0, 0],
        }
    }

    /// 行が組み替わっても、選んだキーは同じキーのまま。
    #[test]
    fn a_selected_key_follows_its_layer_when_rows_are_rebuilt() {
        let (_tx, rx) = std::sync::mpsc::channel();
        let mut w =
            TimelineWidget::new(vec![row(1, vec![1.0, 2.0]), row(2, vec![3.0])], Rc::new(rx));
        w.selected = vec![(1, 0)];
        w.replace_rows(vec![
            row(3, vec![0.5]),
            row(2, vec![3.0]),
            row(1, vec![1.0, 2.0]),
        ]);
        assert_eq!(
            w.selected,
            vec![(1, 0)],
            "the key on layer 2 at 3.0s must stay selected"
        );
        w.replace_rows(vec![row(1, vec![1.0, 2.0])]);
        assert!(w.selected.is_empty(), "a vanished row drops its selection");
    }
}

mod composed_edit_contracts {
    use crate::doc::store::*;
    use crate::ui::functions::{compose, control, verb};
    use crate::ui::property_edit;
    use crate::ui::session::Selection;
    use crate::ui::timeline_edit::{duplicate_layers, split_layers};
    use crate::ui::timeline_widget::*;

    fn document() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 480,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        doc
    }

    fn add(
        doc: &mut Document,
        id: u64,
        start: i64,
        order: i16,
        source: LayerSource,
        parent: Option<LayerId>,
    ) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source,
                    order,
                    timing: LayerTiming {
                        start,
                        duration: 60,
                        ..Default::default()
                    },
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

    #[test]
    fn relative_values_preview_then_commit_once_and_superseded_owner_cannot_write() {
        let mut doc = document();
        let a = add(&mut doc, 1, 10, 0, LayerSource::Shape, None);
        let b = add(&mut doc, 2, 30, 1, LayerSource::Shape, None);
        let property = PropertyId::new(property::POSITION).unwrap();
        let initial = [(a, Value::Vec2([2.0, 3.0])), (b, Value::Vec2([11.0, 5.0]))];
        doc.apply_all(initial.iter().map(|(layer, value)| Intent::SetConstant {
            layer: *layer,
            property: property.clone(),
            value: value.clone(),
        }))
        .unwrap();
        let head = doc.edit_head();
        let values: Vec<_> = initial
            .iter()
            .map(|(layer, value)| {
                (
                    *layer,
                    property.clone(),
                    control::axis_delta(value, 0, 7.0, None).unwrap(),
                )
            })
            .collect();
        let owner = doc.begin_preview();
        property_edit::preview_owned(&mut doc, owner, &values).unwrap();
        for (layer, property, value) in &values {
            assert_eq!(
                doc.view()
                    .value_at(*layer, property, RationalTime::ZERO)
                    .unwrap(),
                Some(value.clone())
            );
        }
        assert_eq!(doc.edit_head(), head);
        property_edit::commit_owned(&mut doc, owner, RationalTime::ZERO, &values).unwrap();
        assert_eq!(doc.edit_head(), head + 1);
        assert!(doc.undo());
        for (layer, value) in initial {
            assert_eq!(
                doc.view()
                    .value_at(layer, &property, RationalTime::ZERO)
                    .unwrap(),
                Some(value)
            );
        }
        let older = doc.begin_preview();
        let newer = doc.begin_preview();
        property_edit::preview_owned(&mut doc, newer, &values).unwrap();
        assert!(property_edit::commit_owned(&mut doc, older, RationalTime::ZERO, &values).is_err());
        assert!(!property_edit::cancel_owned(&mut doc, older));
        assert!(doc.preview_is_current(newer));
    }

    #[test]
    fn duplicate_preserves_group_content_and_remaps_internal_references_and_shared_values() {
        let mut doc = document();
        let group = add(&mut doc, 1, 0, 0, LayerSource::Group, None);
        let child = add(&mut doc, 2, 0, 0, LayerSource::Shape, Some(group));
        let matte = add(&mut doc, 3, 0, 10, LayerSource::Shape, None);
        let locked = add(&mut doc, 4, 0, 100, LayerSource::Shape, None);
        let property = PropertyId::new(property::OPACITY).unwrap();
        let slot = SlotId("shared.opacity".into());
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.5),
            interp: Interp::Hold,
            spatial: None,
        });
        doc.apply_all([
            Intent::SetSlots {
                slots: vec![Slot {
                    id: slot.clone(),
                    track,
                }],
            },
            Intent::SetPropertySlot {
                layer: child,
                property: property.clone(),
                slot: slot.clone(),
            },
            Intent::SetAttrs {
                layer: child,
                patch: LayerAttrsPatch {
                    matte: Some(Some(Matte {
                        layer: matte,
                        mode: MatteMode::Alpha,
                    })),
                    ..Default::default()
                },
            },
            Intent::SetConstant {
                layer: child,
                property: PropertyId::new(property::POSITION).unwrap(),
                value: Value::Vec2([7.0, 4.0]),
            },
            Intent::SetAttrs {
                layer: locked,
                patch: LayerAttrsPatch {
                    locked: Some(true),
                    ..Default::default()
                },
            },
        ])
        .unwrap();
        let head = doc.edit_head();
        let doc = Arc::new(Mutex::new(doc));
        let copies = duplicate_layers(&doc, &[group, child, matte]).unwrap();
        assert_eq!(
            copies.len(),
            2,
            "a child inside a selected group is copied once"
        );
        let mut doc = doc.lock().unwrap();
        assert_eq!(doc.edit_head(), head + 1);
        let copied_child = doc
            .view()
            .layers()
            .into_iter()
            .find(|layer| {
                doc.view()
                    .attrs(*layer)
                    .unwrap()
                    .and_then(|attrs| attrs.parent)
                    == Some(copies[0])
            })
            .unwrap();
        assert_eq!(
            doc.view()
                .attrs(copied_child)
                .unwrap()
                .unwrap()
                .matte
                .unwrap()
                .layer,
            copies[1]
        );
        assert_eq!(
            doc.view()
                .value_at(
                    copied_child,
                    &PropertyId::new(property::POSITION).unwrap(),
                    RationalTime::ZERO
                )
                .unwrap(),
            Some(Value::Vec2([7.0, 4.0]))
        );
        let copied_source = doc
            .view()
            .property_source(copied_child, &property)
            .unwrap()
            .unwrap();
        let Some(PropertyBase::Slot(copy_slot)) = copied_source.base else {
            panic!("slot identity is preserved")
        };
        assert_ne!(
            copy_slot, slot,
            "ordinary duplicate detaches the shared value"
        );
        assert_eq!(
            doc.view().meta(locked).unwrap().unwrap().order,
            100,
            "a distant locked sibling need not move"
        );
        assert!(doc.undo());
        assert!(!doc.view().has_layer(copies[0]));
        assert!(doc.view().has_layer(child));
    }

    #[test]
    fn split_reuses_structural_copy_so_a_group_tail_is_not_empty() {
        let mut doc = document();
        let group = add(&mut doc, 1, 0, 0, LayerSource::Group, None);
        let child = add(&mut doc, 2, 0, 0, LayerSource::Shape, Some(group));
        let head = doc.edit_head();
        let doc = Arc::new(Mutex::new(doc));
        let tails = split_layers(&doc, &[group], 15).unwrap();
        let mut doc = doc.lock().unwrap();
        assert_eq!(doc.edit_head(), head + 1);
        assert_eq!(doc.view().meta(group).unwrap().unwrap().timing.duration, 15);
        assert_eq!(doc.view().meta(tails[0]).unwrap().unwrap().timing.start, 15);
        assert!(doc.view().layers().into_iter().any(|layer| layer != child
            && doc
                .view()
                .attrs(layer)
                .unwrap()
                .and_then(|attrs| attrs.parent)
                == Some(tails[0])));
        assert!(doc.undo());
        assert_eq!(doc.view().layers().len(), 2);
        assert_eq!(doc.view().meta(group).unwrap().unwrap().timing.duration, 60);
    }

    #[test]
    fn timeline_targets_are_fixed_and_document_preview_equals_committed_timing() {
        let mut doc = document();
        let a = add(&mut doc, 1, 10, 0, LayerSource::Shape, None);
        let b = add(&mut doc, 2, 30, 1, LayerSource::Shape, None);
        let original = doc.view().meta(a).unwrap().unwrap().timing;
        let head = doc.edit_head();
        let doc = Arc::new(Mutex::new(doc));
        let selection = Selection::default();
        selection.replace([a, b]);
        let rows = vec![CanvasRow {
            is_group: false,
            keys: vec![],
            span: Some((10.0 / 30.0, 70.0 / 30.0)),
            agg: vec![],
            layer: Some(a),
            prop: None,
            color: [0; 3],
        }];
        let (_, rx) = std::sync::mpsc::channel();
        let mut state = TimelineState::new(rows, Rc::new(rx))
            .with_selection(selection.clone())
            .with_document(doc.clone(), crate::ui::fixture::canvas_rows_from_doc);
        state.drag = state.begin_edit(a, DragMode::Move, 0, original, 10.0 / 30.0);
        selection.set(Some(a));
        state.drag.as_mut().unwrap().delta_sec = -100.0 / 30.0;
        state.preview_edit();
        let before_commit: Vec<_> = [a, b]
            .into_iter()
            .map(|layer| {
                doc.lock()
                    .unwrap()
                    .view()
                    .meta(layer)
                    .unwrap()
                    .unwrap()
                    .timing
            })
            .collect();
        assert_eq!(before_commit[0].start, 0);
        assert_eq!(before_commit[1].start, 20);
        assert_eq!(doc.lock().unwrap().edit_head(), head);
        state.finish_drag(&mut TimelineBindings::default());
        let mut doc = doc.lock().unwrap();
        for (layer, preview) in [a, b].into_iter().zip(before_commit) {
            assert_eq!(doc.view().meta(layer).unwrap().unwrap().timing, preview);
        }
        assert_eq!(doc.edit_head(), head + 1);
        assert!(doc.undo());
        assert_eq!(doc.view().meta(a).unwrap().unwrap().timing.start, 10);
        assert_eq!(doc.view().meta(b).unwrap().unwrap().timing.start, 30);
    }

    #[test]
    fn held_out_distribution_uses_existing_move_blocks_without_new_gesture_or_shell_cases() {
        let mut doc = document();
        let a = add(&mut doc, 1, 10, 0, LayerSource::Shape, None);
        let b = add(&mut doc, 2, 95, 1, LayerSource::Shape, None);
        let c = add(&mut doc, 3, 41, 2, LayerSource::Shape, None);
        let targets = [a, b, c];
        let head = doc.edit_head();
        let (intents, _) = compose::independent_layers(&doc, &targets, |doc, layer| {
            let rank = targets.iter().position(|target| *target == layer).unwrap() as i64;
            let original = doc.view().meta(layer)?.unwrap().timing;
            let changed =
                verb::timing_delta(original, TimingMode::Move, 10 + rank * 20 - original.start)?;
            verb::retime_layer(doc, layer, original, changed, true)
        })
        .unwrap();
        let owner = doc.begin_preview();
        doc.preview_edits(owner, &intents).unwrap();
        for (layer, expected) in targets.into_iter().zip([10, 30, 50]) {
            assert_eq!(
                doc.view().meta(layer).unwrap().unwrap().timing.start,
                expected
            );
        }
        doc.clear_preview_edits(owner);
        doc.apply_all(intents).unwrap();
        assert_eq!(doc.edit_head(), head + 1);
        assert!(doc.undo());
        assert_eq!(doc.view().meta(b).unwrap().unwrap().timing.start, 95);
    }
}

mod interaction_terminals {
    use crate::doc::store::*;
    use crate::ui::session::{GestureSurface, Selection, SurfaceCapture};
    use crate::ui::timeline_widget::*;
    use blitz_traits::events::{
        BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point,
        PointerCoords, UiEvent,
    };
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

    fn extract(_: &Document) -> Vec<CanvasRow> {
        Vec::new()
    }

    fn widget(
        locked: bool,
    ) -> (
        TimelineWidget,
        Arc<Mutex<Document>>,
        GestureSurface,
        Arc<Mutex<Vec<crate::ui::session::KeySel>>>,
        LayerId,
    ) {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 480,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: 0,
                    timing: LayerTiming {
                        duration: 300,
                        ..Default::default()
                    },
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch {
                    locked: Some(locked),
                    ..Default::default()
                },
            },
        ])
        .unwrap();
        let doc = Arc::new(Mutex::new(doc));
        let row = CanvasRow {
            is_group: false,
            keys: vec![1.0],
            span: Some((0.0, 10.0)),
            agg: Vec::new(),
            layer: Some(layer),
            prop: None,
            color: [0, 0, 0],
        };
        let (_tx, rx) = std::sync::mpsc::channel();
        let gesture = GestureSurface::default();
        let keys = Arc::new(Mutex::new(Vec::new()));
        let selection = Selection::default();
        selection.set(Some(layer));
        let widget = TimelineWidget::new(vec![row], Rc::new(rx))
            .with_document(doc.clone(), extract)
            .with_selection(selection)
            .with_gesture(gesture.clone())
            .with_capture(SurfaceCapture::default())
            .with_key_mirror(keys.clone());
        (widget, doc, gesture, keys, layer)
    }

    fn pointer(
        id: BlitzPointerId,
        x: f32,
        y: f32,
        button: MouseEventButton,
        buttons: MouseEventButtons,
        mods: keyboard_types::Modifiers,
    ) -> BlitzPointerEvent {
        BlitzPointerEvent {
            id,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button,
            buttons,
            mods,
            details: Default::default(),
            element: Point { x, y },
            active_pointers: Default::default(),
        }
    }

    #[test]
    fn locked_press_and_modifier_toggle_do_not_leave_an_armed_drag() {
        let (mut locked, _, locked_gesture, _, _) = widget(true);
        let down = pointer(
            BlitzPointerId::Mouse,
            120.0,
            (RULER_H + ROW_H * 0.5) as f32,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
            Default::default(),
        );
        locked.event(
            &UiEvent::PointerDown(down.clone()),
            &mut TimelineBindings::default(),
        );
        assert!(!locked_gesture.is_active());
        assert!(locked.drag.is_none());

        let (mut widget, _, gesture, _, _) = widget(false);
        widget.selected = vec![(0, 0)];
        let mut toggle = down;
        toggle.mods = keyboard_types::Modifiers::SHIFT;
        widget.event(
            &UiEvent::PointerDown(toggle),
            &mut TimelineBindings::default(),
        );
        assert!(widget.selected.is_empty());
        assert!(widget.drag.is_none());
        assert!(!gesture.is_active());
    }

    #[test]
    fn foreign_up_is_ignored_and_matching_cancel_rolls_back_without_history() {
        let (mut widget, doc, gesture, _, _) = widget(false);
        let y = (RULER_H + ROW_H * 0.5) as f32;
        let down = pointer(
            BlitzPointerId::Finger(1),
            300.0,
            y,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
            Default::default(),
        );
        let history = doc.lock().unwrap().history_depth();
        widget.event(
            &UiEvent::PointerDown(down),
            &mut TimelineBindings::default(),
        );
        assert!(gesture.is_active() && widget.drag.is_some());

        let foreign = pointer(
            BlitzPointerId::Finger(2),
            330.0,
            y,
            MouseEventButton::Main,
            MouseEventButtons::None,
            Default::default(),
        );
        widget.event(
            &UiEvent::PointerUp(foreign),
            &mut TimelineBindings::default(),
        );
        assert!(gesture.is_active() && widget.drag.is_some());

        let cancel = pointer(
            BlitzPointerId::Finger(1),
            330.0,
            y,
            MouseEventButton::Main,
            MouseEventButtons::None,
            Default::default(),
        );
        widget.event(
            &UiEvent::PointerCancel(cancel),
            &mut TimelineBindings::default(),
        );
        assert!(!gesture.is_active());
        assert!(widget.drag.is_none());
        assert_eq!(doc.lock().unwrap().history_depth(), history);
    }

    #[test]
    fn right_clicking_an_unselected_key_replaces_the_key_selection() {
        let (mut widget, _, _, keys, layer) = widget(false);
        let right = pointer(
            BlitzPointerId::Mouse,
            PX_PER_SEC as f32,
            (RULER_H + ROW_H * 0.5) as f32,
            MouseEventButton::Secondary,
            MouseEventButtons::Secondary,
            Default::default(),
        );
        widget.event(
            &UiEvent::PointerDown(right),
            &mut TimelineBindings::default(),
        );
        assert_eq!(widget.selected, vec![(0, 0)]);
        let selected = keys.lock().unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].layer, layer);
        assert!((selected[0].at_sec - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_plain_empty_marquee_clears_layer_and_key_domains() {
        let (mut widget, _, gesture, keys, layer) = widget(false);
        widget.selected = vec![(0, 0)];
        widget.publish_keys();
        let selection = widget.selection.clone().unwrap();
        assert_eq!(selection.get(), Some(layer));
        assert_eq!(keys.lock().unwrap().len(), 1);

        let y = (RULER_H + ROW_H * 0.5) as f32;
        let down = pointer(
            BlitzPointerId::Mouse,
            2_000.0,
            y,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
            Default::default(),
        );
        widget.event(
            &UiEvent::PointerDown(down),
            &mut TimelineBindings::default(),
        );
        assert!(selection.all().is_empty());
        assert!(keys.lock().unwrap().is_empty());
        assert!(gesture.is_active() && widget.marquee.is_some());

        let up = pointer(
            BlitzPointerId::Mouse,
            2_010.0,
            y,
            MouseEventButton::Main,
            MouseEventButtons::None,
            Default::default(),
        );
        widget.event(&UiEvent::PointerUp(up), &mut TimelineBindings::default());
        assert!(!gesture.is_active());
        assert!(selection.all().is_empty());
        assert!(keys.lock().unwrap().is_empty());
    }

    #[test]
    fn line_and_pixel_wheels_share_one_distance() {
        assert_eq!(
            wheel_pixels(1.0, -2.0, true),
            wheel_pixels(20.0, -40.0, false)
        );
    }
}

mod borrowed_atom_laws {
    use crate::doc::store::LayerTiming;
    use crate::ui::functions::{
        atom,
        verb::{self, TimingMode},
    };
    use peniko::kurbo::{Affine, Point};
    use proptest::prelude::*;

    fn close(actual: [f64; 2], expected: [f64; 2]) -> bool {
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() < 1e-8)
    }

    proptest! {
        #[test]
        fn about_obeys_kurbo_anchor_rotation_and_inverse_laws(
            x in -10000.0f64..10000.0,
            y in -10000.0f64..10000.0,
            ax in -10000.0f64..10000.0,
            ay in -10000.0f64..10000.0,
            angle in -std::f64::consts::PI..std::f64::consts::PI,
            sx in 0.1f64..4.0,
            sy in 0.1f64..4.0,
        ) {
            let point = [x, y];
            let anchor = [ax, ay];
            prop_assert!(close(atom::scale_about(anchor, anchor, [sx, sy]), anchor));
            prop_assert!(close(atom::rotate_about(anchor, anchor, angle), anchor));
            prop_assert_eq!(atom::scale_about(point, anchor, [1.0, 1.0]), point);
            let rotated = atom::rotate_about(point, anchor, angle);
            let reference = Affine::rotate_about(angle, Point::new(ax, ay)) * Point::new(x, y);
            prop_assert!(close(rotated, [reference.x, reference.y]));
            prop_assert!(close(atom::rotate_about(rotated, anchor, -angle), point));
            let scaled = atom::scale_about(point, anchor, [sx, sy]);
            prop_assert!(close(atom::scale_about(scaled, anchor, [1.0 / sx, 1.0 / sy]), point));
        }

        #[test]
        fn frame_split_and_shift_partition_the_same_half_open_coverage(
            start in -1000i64..1000,
            duration in 1i64..1000,
            cut_offset in 0i64..1001,
            shift in -1000i64..1000,
            probe_offset in -1i64..1002,
        ) {
            let span = atom::frame_span(start, duration).unwrap();
            let cut = start + cut_offset % (duration + 1);
            let (left, right) = atom::split_span(span.clone(), cut).unwrap();
            let timing = LayerTiming { start, duration, ..Default::default() };
            let probe = start + probe_offset;
            prop_assert_eq!(span.contains(&probe), timing.covers(probe));
            prop_assert_eq!(span.contains(&probe), left.contains(&probe) || right.contains(&probe));
            prop_assert!(!(left.contains(&probe) && right.contains(&probe)));
            prop_assert_eq!(left.start, span.start);
            prop_assert_eq!(left.end, right.start);
            prop_assert_eq!(right.end, span.end);
            let moved = atom::shift_span(span.clone(), shift).unwrap();
            prop_assert_eq!(atom::shift_span(moved.clone(), -shift).unwrap(), span.clone());
            prop_assert_eq!(atom::split_span(moved, cut + shift).unwrap(),
                (atom::shift_span(left, shift).unwrap(), atom::shift_span(right, shift).unwrap()));
            let clamped = atom::clamp_to_span(probe, span.clone()).unwrap();
            prop_assert!(span.contains(&clamped));
            prop_assert_eq!(atom::clamp_to_span(clamped, span).unwrap(), clamped);
        }
    }

    #[test]
    fn frame_boundaries_reject_overflow_and_semantic_trim_keeps_one_frame() {
        assert!(atom::frame_span(i64::MAX, 1).is_err());
        assert!(atom::frame_span(0, -1).is_err());
        assert!(atom::shift_span(i64::MIN..(i64::MIN + 1), -1).is_err());
        assert!(atom::clamp_to_span(0, 2..2).is_err());
        assert_eq!(atom::split_span(4..8, 8).unwrap(), (4..8, 8..8));
        let orig = LayerTiming {
            start: 10,
            duration: 20,
            source_in: 5,
            ..Default::default()
        };
        let head = verb::timing_delta(orig, TimingMode::TrimStart, i64::MAX).unwrap();
        assert_eq!(head.duration, 1);
        assert_eq!(head.start + head.duration, orig.start + orig.duration);
        assert_eq!(Some(head.source_in), orig.source_frame(head.start));
        let extended = verb::timing_delta(orig, TimingMode::TrimStart, i64::MIN).unwrap();
        assert_eq!(extended.source_in, 0);
        assert_eq!(
            extended.start + extended.duration,
            orig.start + orig.duration
        );
        assert_eq!(
            verb::timing_delta(orig, TimingMode::TrimEnd, i64::MIN)
                .unwrap()
                .duration,
            1
        );
        assert_eq!(
            atom::scale_about([13.0, 24.0], [10.0, 20.0], [-2.0, 0.5]),
            [4.0, 22.0]
        );
    }
}
