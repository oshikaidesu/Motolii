#![allow(clippy::all)]
mod follow {
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
    use crate::ui::timeline_widget::*;
    use crate::doc::store::{ContentKeyframe, ContentTrack, Intent, LayerSource};

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
            track.insert(ContentKeyframe { t: RationalTime::try_from_frame(frame, fps).unwrap(), content: word.into() });
        }
        text.content = track;
        doc.apply(Intent::SetTextDocument { layer, document: text }).unwrap();

        let intents = keyframe_shift_intents(&doc, layer, 10).unwrap();
        doc.apply_all(intents).unwrap();
        let keys = doc.view().text_document(layer).unwrap().unwrap().content.keys().to_vec();
        let frames: Vec<i64> = keys.iter().map(|k| k.t.try_to_frame_floor(fps).unwrap()).collect();
        assert_eq!(frames, vec![10, 34], "lyric keys did not move with the layer");
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
        assert!(targets.contains(&1.5) && targets.contains(&7.25), "{targets:?}");
    }
}

mod keys {
    use crate::ui::timeline_widget::*;
    use crate::doc::store::{
        property, Composition, Interp, Keyframe, LayerSource, PropertyId, Value,
    };

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
                meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 240) },
            },
            Intent::SetTrack { layer, property: property.clone(), track },
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

    /// Delete はその時刻のキーだけ外す。最後の 1 つを外すと値は定数で残り、絵は飛ばない。
    #[test]
    fn deleting_a_key_leaves_the_others_and_then_a_constant() {
        let (mut doc, layer, property, fps) = two_key_doc();
        doc.apply_all(keyframe_delete_intents(&doc, layer, Some(&property), 0.5).unwrap()).unwrap();
        assert_eq!(frames(&doc, layer, &property, fps), vec![36]);
        doc.apply_all(keyframe_delete_intents(&doc, layer, Some(&property), 1.5).unwrap()).unwrap();
        assert!(doc.view().track(layer, &property).unwrap().is_none(), "track should be gone");
        let at = RationalTime::try_from_frame(36, fps).unwrap();
        assert_eq!(doc.view().value_at(layer, &property, at).unwrap(), Some(Value::Vec2([3.0, 0.0])));
    }

    /// 切ると跨いだ区間のイージングが両側へ分かれ、切り口の値は両方に立つ。
    #[test]
    fn splitting_a_track_keeps_the_curve_on_both_sides() {
        let (doc, layer, property, fps) = two_key_doc();
        let mut eased = doc.view().track(layer, &property).unwrap().unwrap();
        let mut first = eased.keys()[0].clone();
        first.interp = Interp::Bezier { x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0 };
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
        track.keys().iter().map(|k| k.t.try_to_frame_round(fps).unwrap()).collect()
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
                    meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 240) },
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
    use crate::ui::timeline_widget::*;
    use crate::doc::store::{
        property, Composition, Interp, Keyframe, LayerSource, PropertyId, Value,
    };

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
