//! `Composition.camera` の store 側の口(裁定113/115、裁定116 実装)。
//!
//! ここで固定するのは:
//! - camera の center/zoom/roll は `SetCameraTrack` + `PropertyId::camera` で書け、
//!   既存の `KeyframeTrack`/`PropertyId` の平坦な流儀にそのまま乗る(新機構ゼロ)
//! - track が無ければ既定(パン無し・zoom=1・roll=0、裁定20 と同じ扱い)
//! - `resolve()` が `position.z` を読んで `LayerPlacement.z` へ運ぶ(既定 0、裁定113)
//! - `LayerAttrs.pinned` が `ResolvedLayer.pinned` まで届く(既定 false)
//! - camera の track も `flattened()`(裁定57 の「store に聞く」経路)でそのまま保存される
//!
//! カメラの投影数学そのもの(view/projection 行列・視差)は `motolii-core::camera` の
//! 純粋な単体試験と `motolii-compositor` の GPU 試験が持つ — ここは「Document に
//! 正しく出入りするか」だけを縛る。

use motolii_store::{
    property, Composition, Document, Fps, Interp, Intent, Keyframe, KeyframeTrack, LayerAttrsPatch,
    LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, ResolvedCamera, Value,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn still(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn camera_prop(name: &str) -> PropertyId {
    PropertyId::camera(name).expect("camera property name")
}

// ---------------------------------------------------------------------------
// camera.center / camera.zoom / camera.roll
// ---------------------------------------------------------------------------

/// track を一切打たなければ既定(パン無し・zoom=1・roll=0)。
#[test]
fn no_camera_track_at_all_defaults_to_identity_camera() {
    let doc = doc_with_comp();
    let camera = doc.view().resolve_camera(t(0)).unwrap();
    assert_eq!(camera, ResolvedCamera::default());
}

/// center/zoom/roll は既存の `SetTrack` と同じ形(`SetCameraTrack`)で書け、
/// `resolve_camera` が組み立てて返す。
#[test]
fn camera_properties_round_trip_through_set_camera_track() {
    let mut doc = doc_with_comp();
    doc.apply_all([
        Intent::SetCameraTrack {
            property: camera_prop(property::CAMERA_CENTER),
            track: still(Value::Vec2([12.0, -7.0])),
        },
        Intent::SetCameraTrack {
            property: camera_prop(property::CAMERA_ZOOM),
            track: still(Value::F64(1.5)),
        },
        Intent::SetCameraTrack {
            property: camera_prop(property::CAMERA_ROLL),
            track: still(Value::F64(45.0)),
        },
    ])
    .unwrap();

    let camera = doc.view().resolve_camera(t(0)).unwrap();
    assert_eq!(camera.center, [12.0, -7.0]);
    assert!((camera.zoom - 1.5).abs() < 1e-6);
    assert!((camera.roll_degrees - 45.0).abs() < 1e-6);
}

/// camera の track はキーフレーム可能(裁定115「全てキーフレーム可能」)。
#[test]
fn camera_zoom_can_be_keyframed_over_time() {
    let mut doc = doc_with_comp();
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t(0),
        value: Value::F64(1.0),
        interp: Interp::Linear,
        spatial: None,
    });
    track.insert(Keyframe {
        t: t(30),
        value: Value::F64(2.0),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_ZOOM),
        track,
    })
    .unwrap();

    let start = doc.view().resolve_camera(t(0)).unwrap();
    let mid = doc.view().resolve_camera(t(15)).unwrap();
    let end = doc.view().resolve_camera(t(30)).unwrap();
    assert!((start.zoom - 1.0).abs() < 1e-6);
    assert!((mid.zoom - 1.5).abs() < 1e-6, "中間で補間されていない: {mid:?}");
    assert!((end.zoom - 2.0).abs() < 1e-6);
}

/// undo で戻る — camera も layer の property と同じ `edit` timeline に乗っている。
#[test]
fn camera_edits_are_undoable_like_any_other_edit() {
    let mut doc = doc_with_comp();
    doc.mark_undo_floor();
    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_ZOOM),
        track: still(Value::F64(3.0)),
    })
    .unwrap();
    assert!((doc.view().resolve_camera(t(0)).unwrap().zoom - 3.0).abs() < 1e-6);

    assert!(doc.undo());
    assert_eq!(doc.view().resolve_camera(t(0)).unwrap(), ResolvedCamera::default());
}

/// camera の track は他の component と同じく `flattened()`(store に聞く経路、裁定57)で
/// 運ばれる — camera 専用のコピー処理を書き足していないことの柵。
#[test]
fn camera_tracks_survive_flattened() {
    let mut doc = doc_with_comp();
    doc.apply(Intent::SetCameraTrack {
        property: camera_prop(property::CAMERA_ROLL),
        track: still(Value::F64(30.0)),
    })
    .unwrap();

    let flat = doc.flattened().unwrap();
    let camera = flat.view().resolve_camera(t(0)).unwrap();
    assert!((camera.roll_degrees - 30.0).abs() < 1e-6);
}

// ---------------------------------------------------------------------------
// position.z(裁定113: 全員 z=0 既定)
// ---------------------------------------------------------------------------

fn solid() -> LayerSource {
    LayerSource::Solid {
        rgba: [255, 0, 0, 255],
        width: 64,
        height: 64,
    }
}

fn place(doc: &mut Document, layer: LayerId) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
    ])
    .unwrap();
}

/// `position.z` を一切打たなければ 0(裁定113「全員 z=0 既定」)。
#[test]
fn a_layer_without_a_z_track_defaults_to_z0() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer);

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.placement.z, 0.0);
}

/// `position.z` は `position.x`/`position.y` と同じ流儀(平坦な `PropertyId`)で効く。
#[test]
fn position_z_track_moves_the_layer_along_z() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer);

    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::POSITION_Z).unwrap(),
        track: still(Value::F64(120.0)),
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.placement.z, 120.0);
}

// ---------------------------------------------------------------------------
// pinned(裁定113: カメラに張り付く層は明示属性)
// ---------------------------------------------------------------------------

/// 属性を一度も書いていない layer は既定 `pinned = false`。
#[test]
fn a_layer_without_attrs_is_not_pinned() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer);

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert!(!resolved.pinned);
}

/// `SetAttrs { pinned: true, .. }` が `ResolvedLayer.pinned` まで届く。
#[test]
fn pinned_attr_reaches_the_resolved_layer() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    place(&mut doc, layer);

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            pinned: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert!(resolved.pinned);
}

/// pinned を立てても `meta`(素材・重ね順・配置)は一切変わらない — `hidden`/`parent` と
/// 同じ component(`Layer:attrs`)に同居しているので、裁定108(c) の構造修正の恩恵を
/// そのまま受ける。
#[test]
fn pinning_a_layer_never_touches_timing_or_source() {
    let mut doc = doc_with_comp();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 5,
                timing: LayerTiming {
                    start: 42,
                    duration: 17,
                    source_in: 3,
                    ..Default::default()
                },
            },
        },
    ])
    .unwrap();

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            pinned: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let meta = doc.view().meta(layer).unwrap().expect("meta が無い");
    assert_eq!(meta.order, 5);
    assert_eq!(meta.timing.start, 42);
    assert_eq!(meta.timing.duration, 17);
    assert_eq!(meta.timing.source_in, 3);
}
