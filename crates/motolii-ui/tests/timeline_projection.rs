//! U3a-1I: headless Timeline projection の決定的fixture審判。
//! packing/cull/座標overflowの順序と半開区間を、Document validate済みfixtureだけで固定する。

use motolii_core::RationalTime;
use motolii_doc::{
    AssetId, Clip, ClipSource, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document, Group,
    ItemEnvelope, KeyframeId, LayerId, LookAtAxis, Track, TrackItem,
};
use motolii_eval::{DataTrackId, Interp};
use motolii_ui::{
    project_timeline, TimelineBar, TimelineHit, TimelineMetrics, TimelineProjectionError,
    TimelineUnsupported, TimelineViewport,
};

fn metrics() -> TimelineMetrics {
    TimelineMetrics {
        band_height: 20.0,
        units_per_second: 100.0,
        key_half_extent: 8.0,
    }
}

fn viewport_wide() -> TimelineViewport {
    TimelineViewport {
        start: RationalTime::ZERO,
        end: RationalTime::from_seconds(60),
    }
}

struct DocFixture {
    doc: Document,
    asset: AssetId,
}

impl DocFixture {
    fn new() -> Self {
        let mut doc = Document::new_current();
        let track = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
        doc.tracks.push(Track {
            id: track,
            items: vec![],
        });
        Self { doc, asset }
    }

    fn push_clip(
        &mut self,
        layer_name: &str,
        start: RationalTime,
        duration: RationalTime,
        position: DocParam,
    ) -> LayerId {
        let layer = self.doc.layers.allocate(layer_name).unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = position;
        self.doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start,
            duration,
            time_map: Default::default(),
            source: ClipSource::asset_video_only(self.asset),
        }));
        layer
    }

    fn finish(self) -> Document {
        self.doc.validate().unwrap();
        self.doc
    }

    fn finish_without_validate(self) -> Document {
        self.doc
    }
}

fn sec(n: i64) -> RationalTime {
    RationalTime::try_new(n, 1).unwrap()
}

fn keyframe_vec2_at(f: &mut DocFixture, t: RationalTime) -> DocKeyframe {
    DocKeyframe {
        id: KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap()),
        t,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Hold,
    }
}

fn keyframe_f64_at(f: &mut DocFixture, t: RationalTime) -> DocKeyframe {
    DocKeyframe {
        id: KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap()),
        t,
        value: DocValue::F64(0.0),
        interp: Interp::Hold,
    }
}

#[test]
fn p1_two_non_overlapping_clips_share_band_zero() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    f.push_clip("b", sec(2), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert_eq!(proj.bars().len(), 2);
    assert!(proj.bars().iter().all(|b| b.band == 0));
}

#[test]
fn p2_overlapping_clips_use_separate_bands() {
    let mut f = DocFixture::new();
    let la = f.push_clip("a", sec(0), sec(3), DocParam::const_vec2([0.0, 0.0]));
    let lb = f.push_clip("b", sec(1), sec(2), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    let band_a = proj.bars().iter().find(|b| b.layer == la).unwrap().band;
    let band_b = proj.bars().iter().find(|b| b.layer == lb).unwrap().band;
    assert_eq!(band_a, 0);
    assert_eq!(band_b, 1);
}

#[test]
fn p3_touching_clips_share_band_zero() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    f.push_clip("b", sec(1), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert_eq!(proj.bars().len(), 2);
    assert!(proj.bars().iter().all(|b| b.band == 0));
}

#[test]
fn p4_third_clip_reuses_band_zero_without_creating_band_two() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(2), DocParam::const_vec2([0.0, 0.0]));
    f.push_clip("b", sec(1), sec(3), DocParam::const_vec2([0.0, 0.0]));
    let lc = f.push_clip("c", sec(4), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    let band_c = proj.bars().iter().find(|b| b.layer == lc).unwrap().band;
    assert_eq!(band_c, 0);
    assert!(proj.bars().iter().all(|b| b.band <= 1));
}

#[test]
fn p5_keyframes_project_exact_rational_time() {
    let mut f = DocFixture::new();
    let t = RationalTime::try_new(3, 2).unwrap();
    let mut track = DocKeyframeTrack::new();
    let inserted = keyframe_vec2_at(&mut f, t);
    let expected_key_id = inserted.id;
    track.insert(inserted);
    let layer = f.push_clip("kf", sec(0), sec(5), DocParam::Keyframes(track));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert_eq!(proj.keys().len(), 1);
    let key = proj.keys().iter().find(|k| k.layer == layer).unwrap();
    assert_eq!(key.layer, layer);
    assert_eq!(key.key, expected_key_id);
    assert_eq!(key.t, t);
}

#[test]
fn p6_const_position_emits_no_keys_or_unsupported() {
    let mut f = DocFixture::new();
    f.push_clip("c", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert!(proj.keys().is_empty());
    assert!(proj.unsupported().is_empty());
}

#[test]
fn p7_cull_preserves_band_and_layer_for_visible_bars() {
    let mut f = DocFixture::new();
    let hidden = f.push_clip("hidden", sec(0), sec(5), DocParam::const_vec2([0.0, 0.0]));
    let visible = f.push_clip("visible", sec(4), sec(3), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let wide = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    let narrow_vp = TimelineViewport {
        start: sec(5),
        end: sec(20),
    };
    let narrow = project_timeline(&doc, &metrics(), &narrow_vp).unwrap();
    assert_eq!(narrow.bars().len(), 1);
    assert_eq!(narrow.bars()[0].layer, visible);
    assert!(narrow.bars().iter().all(|b| b.layer != hidden));
    let wide_bar = wide.bars().iter().find(|b| b.layer == visible).unwrap();
    let narrow_bar = narrow.bars().iter().find(|b| b.layer == visible).unwrap();
    assert_eq!(wide_bar.band, 1);
    assert_eq!(wide_bar.band, narrow_bar.band);
    assert_eq!(wide_bar.layer, narrow_bar.layer);
}

#[test]
fn p8_hit_test_prefers_key_over_bar() {
    let mut f = DocFixture::new();
    let t = sec(1);
    let mut track = DocKeyframeTrack::new();
    track.insert(keyframe_vec2_at(&mut f, t));
    let layer = f.push_clip("a", sec(0), sec(3), DocParam::Keyframes(track));
    let doc = f.finish();
    let m = metrics();
    let proj = project_timeline(&doc, &m, &viewport_wide()).unwrap();
    let key = proj.keys().iter().find(|k| k.layer == layer).unwrap();
    let hit = proj.hit_test(key.center_x, key.center_y);
    assert!(matches!(hit, TimelineHit::Key { layer: l, .. } if l == layer));
}

#[test]
fn p9_overlapping_key_diamonds_pick_min_layer_and_key() {
    let mut f = DocFixture::new();
    let t = sec(2);
    let mut track_a = DocKeyframeTrack::new();
    let id_a = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    track_a.insert(DocKeyframe {
        id: id_a,
        t,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Hold,
    });
    let mut track_b = DocKeyframeTrack::new();
    let id_b = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    track_b.insert(DocKeyframe {
        id: id_b,
        t,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Hold,
    });
    let la = f.push_clip("a", sec(0), sec(5), DocParam::Keyframes(track_a));
    let lb = f.push_clip("b", sec(1), sec(5), DocParam::Keyframes(track_b));
    let doc = f.finish();
    let m = TimelineMetrics {
        band_height: 20.0,
        units_per_second: 100.0,
        key_half_extent: 15.0,
    };
    let proj = project_timeline(&doc, &m, &viewport_wide()).unwrap();
    let ka = proj.keys().iter().find(|k| k.layer == la).unwrap();
    let kb = proj.keys().iter().find(|k| k.layer == lb).unwrap();
    let y_overlap = (ka.center_y + kb.center_y) / 2.0;
    let hit = proj.hit_test(ka.center_x, y_overlap);
    let expect_layer = if (la, id_a) < (lb, id_b) { la } else { lb };
    let expect_key = if (la, id_a) < (lb, id_b) { id_a } else { id_b };
    assert!(matches!(
        hit,
        TimelineHit::Key {
            layer: l,
            key: k
        } if l == expect_layer && k == expect_key
    ));
}

#[test]
fn p10_touch_boundary_hits_second_bar_only_once() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let lb = f.push_clip("b", sec(1), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    let bar_b = proj.bars().iter().find(|b| b.layer == lb).unwrap();
    let y = bar_b.y_top + 1.0;
    let x = bar_b.x_start;
    let hit = proj.hit_test(x, y);
    assert!(matches!(hit, TimelineHit::Bar { layer: l } if l == lb));
    let matching: Vec<_> = proj
        .bars()
        .iter()
        .filter(|b| x >= b.x_start && x < b.x_end && y >= b.y_top && y < b.y_bottom)
        .collect();
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].layer, lb);
}

#[test]
fn p11_duplicate_projection_is_identical() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    f.push_clip("b", sec(2), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let m = metrics();
    let v = viewport_wide();
    let a = project_timeline(&doc, &m, &v).unwrap();
    let b = project_timeline(&doc, &m, &v).unwrap();
    assert_eq!(a.bars(), b.bars());
    assert_eq!(a.keys(), b.keys());
    assert_eq!(a.unsupported(), b.unsupported());
}

#[test]
fn p12_hundred_thousand_keys_cull_to_visible_identity() {
    let mut f = DocFixture::new();
    let mut track = DocKeyframeTrack::new();
    let mut visible_id = None;
    for i in 0..100_000_i64 {
        let key = keyframe_vec2_at(&mut f, sec(i));
        if i == 50_000 {
            visible_id = Some(key.id);
        }
        track.insert(key);
    }
    let layer = f.push_clip(
        "repeated-label",
        sec(0),
        sec(100_001),
        DocParam::Keyframes(track),
    );
    f.doc.composition.duration = sec(100_001);
    let doc = f.finish();
    let viewport = TimelineViewport {
        start: sec(50_000),
        end: sec(50_001),
    };
    let projection = project_timeline(&doc, &metrics(), &viewport).unwrap();
    assert_eq!(projection.keys().len(), 1);
    assert_eq!(projection.keys()[0].layer, layer);
    assert_eq!(projection.keys()[0].key, visible_id.unwrap());
}

#[test]
fn n1_zero_duration_is_invalid_duration_without_partial_output() {
    let mut f = DocFixture::new();
    f.push_clip(
        "a",
        sec(0),
        RationalTime::ZERO,
        DocParam::const_vec2([0.0, 0.0]),
    );
    let doc = f.finish_without_validate();
    let err = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap_err();
    assert!(matches!(
        err,
        TimelineProjectionError::InvalidDuration { .. }
    ));
}

#[test]
fn n2_negative_duration_is_invalid_duration() {
    let mut f = DocFixture::new();
    f.push_clip(
        "a",
        sec(0),
        RationalTime::try_new(-1, 1).unwrap(),
        DocParam::const_vec2([0.0, 0.0]),
    );
    let doc = f.finish_without_validate();
    let err = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap_err();
    assert!(matches!(
        err,
        TimelineProjectionError::InvalidDuration { .. }
    ));
}

#[test]
fn n3_time_overflow_on_start_plus_duration() {
    let mut f = DocFixture::new();
    let huge = RationalTime::try_new(i64::MAX, 1).unwrap();
    f.push_clip("a", huge, sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish_without_validate();
    let err = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap_err();
    assert!(matches!(err, TimelineProjectionError::TimeOverflow { .. }));
}

#[test]
fn n4_non_finite_metrics_rejected() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let nan = TimelineMetrics {
        band_height: f64::NAN,
        units_per_second: 100.0,
        key_half_extent: 5.0,
    };
    assert!(matches!(
        project_timeline(&doc, &nan, &viewport_wide()).unwrap_err(),
        TimelineProjectionError::NonFiniteMetric
    ));
    let inf = TimelineMetrics {
        band_height: 20.0,
        units_per_second: f64::INFINITY,
        key_half_extent: 5.0,
    };
    assert!(matches!(
        project_timeline(&doc, &inf, &viewport_wide()).unwrap_err(),
        TimelineProjectionError::NonFiniteMetric
    ));
}

#[test]
fn n5_non_positive_metrics_rejected() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let zero_extent = TimelineMetrics {
        band_height: 20.0,
        units_per_second: 100.0,
        key_half_extent: 0.0,
    };
    assert!(matches!(
        project_timeline(&doc, &zero_extent, &viewport_wide()).unwrap_err(),
        TimelineProjectionError::NonPositiveMetric
    ));
    let neg_height = TimelineMetrics {
        band_height: -1.0,
        units_per_second: 100.0,
        key_half_extent: 5.0,
    };
    assert!(matches!(
        project_timeline(&doc, &neg_height, &viewport_wide()).unwrap_err(),
        TimelineProjectionError::NonPositiveMetric
    ));
}

#[test]
fn n6_invalid_viewport_rejected() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let equal = TimelineViewport {
        start: sec(0),
        end: sec(0),
    };
    assert!(matches!(
        project_timeline(&doc, &metrics(), &equal).unwrap_err(),
        TimelineProjectionError::InvalidViewport
    ));
    let reversed = TimelineViewport {
        start: sec(5),
        end: sec(1),
    };
    assert!(matches!(
        project_timeline(&doc, &metrics(), &reversed).unwrap_err(),
        TimelineProjectionError::InvalidViewport
    ));
}

#[test]
fn n7_read_only_accessors_and_two_arg_hit_test() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let projection = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    let _: &[TimelineBar] = projection.bars();
    let _ = projection.hit_test(0.0, 0.0);
}

#[test]
fn n8_group_item_is_unsupported_without_child_bars() {
    let mut f = DocFixture::new();
    let group_layer = f.doc.layers.allocate("grp").unwrap();
    let child_layer = f.doc.layers.allocate("child").unwrap();
    let child = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(child_layer),
        start: sec(0),
        duration: sec(1),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(f.asset),
    });
    f.doc.tracks[0].items.push(TrackItem::Group(Group {
        envelope: ItemEnvelope::new(group_layer),
        children: vec![child],
    }));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert_eq!(proj.unsupported().len(), 1);
    assert!(matches!(
        proj.unsupported()[0],
        TimelineUnsupported::GroupItem { layer } if layer == group_layer
    ));
    assert!(proj.bars().is_empty());
    assert!(proj.keys().is_empty());
}

#[test]
fn n9_unsupported_position_variants_emit_no_keys() {
    let mut f = DocFixture::new();
    let data_layer = {
        let layer = f.doc.layers.allocate("data").unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = DocParam::Data {
            track: DataTrackId::from("dt"),
            fallback: DocValue::Vec2([0.0, 0.0]),
        };
        f.doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: sec(0),
            duration: sec(1),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(f.asset),
        }));
        layer
    };
    let mut inner = DocKeyframeTrack::new();
    inner.insert(keyframe_f64_at(&mut f, sec(1)));
    let vec_layer = {
        let layer = f.doc.layers.allocate("vec").unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = DocParam::Vec2Axes {
            x: Box::new(DocParam::Keyframes(inner)),
            y: Box::new(DocParam::const_f64(0.0)),
        };
        f.doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: sec(0),
            duration: sec(1),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(f.asset),
        }));
        layer
    };
    let look_layer = {
        let layer = f.doc.layers.allocate("look").unwrap();
        let target = f.doc.layers.allocate("tgt").unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = DocParam::LookAt {
            target,
            axis: LookAtAxis::PlusY,
        };
        f.doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: sec(0),
            duration: sec(1),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(f.asset),
        }));
        layer
    };
    let follow_layer = {
        let layer = f.doc.layers.allocate("fol").unwrap();
        let target = f.doc.layers.allocate("tgt2").unwrap();
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = DocParam::Follow {
            target,
            offset: [0.0, 0.0],
        };
        f.doc.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: sec(0),
            duration: sec(1),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(f.asset),
        }));
        layer
    };
    let doc = f.finish_without_validate();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert!(proj
        .unsupported()
        .iter()
        .any(|u| matches!(u, TimelineUnsupported::DataParam { layer } if *layer == data_layer)));
    assert!(proj.unsupported().iter().any(|u| matches!(
        u,
        TimelineUnsupported::Vec2AxesParam { layer } if *layer == vec_layer
    )));
    assert!(proj.unsupported().iter().any(|u| matches!(
        u,
        TimelineUnsupported::LookAtParam { layer } if *layer == look_layer
    )));
    assert!(proj.unsupported().iter().any(|u| matches!(
        u,
        TimelineUnsupported::FollowParam { layer } if *layer == follow_layer
    )));
    assert!(proj.keys().is_empty());
}

#[test]
fn n10_viewport_cull_excludes_touching_bar_end_and_out_of_viewport_keys() {
    let mut f = DocFixture::new();
    f.push_clip("touch", sec(0), sec(2), DocParam::const_vec2([0.0, 0.0]));
    let mut track = DocKeyframeTrack::new();
    track.insert(keyframe_vec2_at(&mut f, sec(2)));
    track.insert(keyframe_vec2_at(&mut f, sec(6)));
    let layer = f.push_clip("bar", sec(5), sec(5), DocParam::Keyframes(track));
    let doc = f.finish();
    let culled_touch = TimelineViewport {
        start: sec(2),
        end: sec(10),
    };
    let proj_touch = project_timeline(&doc, &metrics(), &culled_touch).unwrap();
    assert_eq!(proj_touch.bars().len(), 1);
    assert_eq!(proj_touch.bars()[0].layer, layer);
    let mut f_touch = DocFixture::new();
    f_touch.push_clip("only", sec(0), sec(2), DocParam::const_vec2([0.0, 0.0]));
    let touch_doc = f_touch.finish();
    let touch_only = project_timeline(&touch_doc, &metrics(), &culled_touch).unwrap();
    assert!(touch_only.bars().is_empty());
    let narrow = TimelineViewport {
        start: sec(5),
        end: sec(11),
    };
    let proj = project_timeline(&doc, &metrics(), &narrow).unwrap();
    assert_eq!(proj.bars().len(), 1);
    assert_eq!(proj.bars()[0].layer, layer);
    assert_eq!(proj.keys().len(), 1);
    assert_eq!(proj.keys()[0].t, sec(6));
    let v2 = TimelineViewport {
        start: sec(0),
        end: sec(2),
    };
    let proj2 = project_timeline(&doc, &metrics(), &v2).unwrap();
    assert_eq!(proj2.bars().len(), 1);
    assert!(proj2.keys().is_empty());
}

#[test]
fn n11_hit_test_none_outside_targets() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert!(matches!(proj.hit_test(-1000.0, -1000.0), TimelineHit::None));
}

#[test]
fn n13_finite_metrics_coordinate_product_overflow_is_time_overflow() {
    let mut f = DocFixture::new();
    let layer = f.push_clip("a", sec(0), sec(10), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let huge_scale = TimelineMetrics {
        band_height: 20.0,
        units_per_second: f64::MAX,
        key_half_extent: 8.0,
    };
    let err = project_timeline(&doc, &huge_scale, &viewport_wide()).unwrap_err();
    assert!(matches!(
        err,
        TimelineProjectionError::TimeOverflow { layer: l } if l == layer
    ));
}

#[test]
fn n12_nan_coordinates_yield_none_without_panic() {
    let mut f = DocFixture::new();
    f.push_clip("a", sec(0), sec(1), DocParam::const_vec2([0.0, 0.0]));
    let doc = f.finish();
    let proj = project_timeline(&doc, &metrics(), &viewport_wide()).unwrap();
    assert!(matches!(proj.hit_test(f64::NAN, 0.0), TimelineHit::None));
    assert!(matches!(proj.hit_test(0.0, f64::NAN), TimelineHit::None));
}
