#![allow(deprecated)]

//! LookAt/Follow: 描画順非依存の world position 事前解決(F-3)。

use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    resolve_document_spaces, Affine2D, Clip, ClipSource, DocParam, Document, Group, ItemEnvelope,
    LayerId, LookAtAxis, ParamEvalError, Track, TrackItem, Transform2D, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;

fn approx(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} vs {b}");
}

fn approx2(a: [f64; 2], b: [f64; 2]) {
    approx(a[0], b[0]);
    approx(a[1], b[1]);
}

fn rotation_of(m: Affine2D) -> f64 {
    m.m[3].atan2(m.m[0])
}

fn rect_clip(layer: LayerId, xform: Transform2D) -> Clip {
    Clip {
        envelope: ItemEnvelope {
            transform: xform,
            ..ItemEnvelope::new(layer)
        },
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                ("size".into(), DocParam::const_vec2([0.1, 0.1])),
                ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 1.0])),
            ]),
            extra: Default::default(),
        },
    }
}

fn look_at_doc(order_target_first: bool) -> (Document, LayerId, LayerId) {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let target = doc.layers.allocate("target").unwrap();
    let looker = doc.layers.allocate("looker").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();

    let mut target_xf = Transform2D::identity();
    target_xf.position = DocParam::const_vec2([1.0, 1.0]);
    let mut looker_xf = Transform2D::identity();
    looker_xf.position = DocParam::const_vec2([0.0, 0.0]);
    looker_xf.rotation = DocParam::LookAt {
        target,
        axis: LookAtAxis::PlusX,
    };

    let target_item = TrackItem::Clip(rect_clip(target, target_xf));
    let looker_item = TrackItem::Clip(rect_clip(looker, looker_xf));
    let items = if order_target_first {
        vec![target_item, looker_item]
    } else {
        vec![looker_item, target_item]
    };
    doc.tracks.push(Track { id: tid, items });
    (doc, looker, target)
}

#[test]
fn look_at_document_order_independent_same_rotation() {
    let tracks = DataTracks::new();
    let (doc_front, looker, _) = look_at_doc(true);
    let (doc_back, looker2, _) = look_at_doc(false);
    assert_eq!(looker, looker2);

    let (_, worlds_front) =
        resolve_document_spaces(&doc_front, RationalTime::ZERO, &tracks).unwrap();
    let (_, worlds_back) = resolve_document_spaces(&doc_back, RationalTime::ZERO, &tracks).unwrap();

    let r_front = rotation_of(worlds_front[&looker.get()]);
    let r_back = rotation_of(worlds_back[&looker.get()]);
    approx(r_front, FRAC_PI_4);
    approx(r_back, FRAC_PI_4);
    approx(r_front, r_back);
}

#[test]
fn look_at_across_groups_uses_world_position() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let g_look = doc.layers.allocate("g_look").unwrap();
    let g_tgt = doc.layers.allocate("g_tgt").unwrap();
    let looker = doc.layers.allocate("looker").unwrap();
    let target = doc.layers.allocate("target").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();

    let mut g_look_xf = Transform2D::identity();
    g_look_xf.position = DocParam::const_vec2([10.0, 0.0]);
    let mut g_tgt_xf = Transform2D::identity();
    g_tgt_xf.position = DocParam::const_vec2([10.0, 10.0]);

    let mut looker_xf = Transform2D::identity();
    looker_xf.rotation = DocParam::LookAt {
        target,
        axis: LookAtAxis::PlusX,
    };
    let target_xf = Transform2D::identity();

    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Group(Group {
                envelope: ItemEnvelope {
                    transform: g_look_xf,
                    ..ItemEnvelope::new(g_look)
                },
                children: vec![TrackItem::Clip(rect_clip(looker, looker_xf))],
            }),
            TrackItem::Group(Group {
                envelope: ItemEnvelope {
                    transform: g_tgt_xf,
                    ..ItemEnvelope::new(g_tgt)
                },
                children: vec![TrackItem::Clip(rect_clip(target, target_xf))],
            }),
        ],
    });

    let (resolved, worlds) =
        resolve_document_spaces(&doc, RationalTime::ZERO, &DataTracks::new()).unwrap();

    // looker world (10,0), target world (10,10) → +Y → π/2
    approx2(resolved.position(looker).unwrap(), [10.0, 0.0]);
    approx2(resolved.position(target).unwrap(), [10.0, 10.0]);
    approx(rotation_of(worlds[&looker.get()]), FRAC_PI_2);
}

#[test]
fn look_at_across_transform_parent_uses_world_position() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let parent = doc.layers.allocate("parent").unwrap();
    let looker = doc.layers.allocate("looker").unwrap();
    let target = doc.layers.allocate("target").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();

    let mut parent_xf = Transform2D::identity();
    parent_xf.position = DocParam::const_vec2([5.0, 0.0]);
    let mut looker_xf = Transform2D::identity();
    looker_xf.position = DocParam::const_vec2([0.0, 0.0]);
    looker_xf.parent = Some(parent);
    looker_xf.rotation = DocParam::LookAt {
        target,
        axis: LookAtAxis::PlusX,
    };
    let mut target_xf = Transform2D::identity();
    target_xf.position = DocParam::const_vec2([5.0, 5.0]);

    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Clip(rect_clip(looker, looker_xf)),
            TrackItem::Clip(rect_clip(parent, parent_xf)),
            TrackItem::Clip(rect_clip(target, target_xf)),
        ],
    });

    let (resolved, worlds) =
        resolve_document_spaces(&doc, RationalTime::ZERO, &DataTracks::new()).unwrap();

    approx2(resolved.position(looker).unwrap(), [5.0, 0.0]);
    approx2(resolved.position(target).unwrap(), [5.0, 5.0]);
    approx(rotation_of(worlds[&looker.get()]), FRAC_PI_2);
}

/// 親が回転しているとき、world 方向を placement 逆で local へ戻さないと向きがずれる。
#[test]
fn look_at_rotated_parent_maps_world_direction_to_local() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let parent = doc.layers.allocate("parent").unwrap();
    let looker = doc.layers.allocate("looker").unwrap();
    let target = doc.layers.allocate("target").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();

    // 親 +90°。looker local (1,0) → world (0,1)。target world (1,1) → 期待 world 角 0。
    let mut parent_xf = Transform2D::identity();
    parent_xf.rotation = DocParam::const_f64(FRAC_PI_2);
    let mut looker_xf = Transform2D::identity();
    looker_xf.position = DocParam::const_vec2([1.0, 0.0]);
    looker_xf.parent = Some(parent);
    looker_xf.rotation = DocParam::LookAt {
        target,
        axis: LookAtAxis::PlusX,
    };
    let mut target_xf = Transform2D::identity();
    target_xf.position = DocParam::const_vec2([1.0, 1.0]);

    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Clip(rect_clip(parent, parent_xf)),
            TrackItem::Clip(rect_clip(looker, looker_xf)),
            TrackItem::Clip(rect_clip(target, target_xf)),
        ],
    });

    let (resolved, worlds) =
        resolve_document_spaces(&doc, RationalTime::ZERO, &DataTracks::new()).unwrap();
    approx2(resolved.position(looker).unwrap(), [0.0, 1.0]);
    approx2(resolved.position(target).unwrap(), [1.0, 1.0]);
    approx(rotation_of(worlds[&looker.get()]), 0.0);
}

#[test]
fn look_at_parent_cycle_is_typed_spatial_link_cycle() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let a = doc.layers.allocate("a").unwrap();
    let b = doc.layers.allocate("b").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();

    // A LookAt B、B の parent が A → world_pos(B) が A の回転に依存し循環。
    let mut a_xf = Transform2D::identity();
    a_xf.rotation = DocParam::LookAt {
        target: b,
        axis: LookAtAxis::PlusX,
    };
    let mut b_xf = Transform2D::identity();
    b_xf.position = DocParam::const_vec2([1.0, 0.0]);
    b_xf.parent = Some(a);

    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Clip(rect_clip(a, a_xf)),
            TrackItem::Clip(rect_clip(b, b_xf)),
        ],
    });

    let err = resolve_document_spaces(&doc, RationalTime::ZERO, &DataTracks::new()).unwrap_err();
    assert!(
        matches!(err, ParamEvalError::SpatialLinkCycle { .. }),
        "got {err:?}"
    );
}

#[test]
fn mutual_look_at_without_position_deps_ok() {
    let mut doc = Document::new_v1();
    doc.composition.duration = RationalTime::try_new(10, 1).unwrap();
    let a = doc.layers.allocate("a").unwrap();
    let b = doc.layers.allocate("b").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();

    let mut a_xf = Transform2D::identity();
    a_xf.position = DocParam::const_vec2([0.0, 0.0]);
    a_xf.rotation = DocParam::LookAt {
        target: b,
        axis: LookAtAxis::PlusX,
    };
    let mut b_xf = Transform2D::identity();
    b_xf.position = DocParam::const_vec2([1.0, 0.0]);
    b_xf.rotation = DocParam::LookAt {
        target: a,
        axis: LookAtAxis::PlusX,
    };

    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Clip(rect_clip(a, a_xf)),
            TrackItem::Clip(rect_clip(b, b_xf)),
        ],
    });

    let (_, worlds) =
        resolve_document_spaces(&doc, RationalTime::ZERO, &DataTracks::new()).unwrap();
    approx(rotation_of(worlds[&a.get()]), 0.0);
    approx(rotation_of(worlds[&b.get()]).abs(), std::f64::consts::PI);
}
