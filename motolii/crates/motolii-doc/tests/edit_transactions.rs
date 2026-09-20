//! One action is one Undo; preview is the same edit projected through StoreView.
//! The key/source rules follow keys_only_when_asked.rs and the grouped-edit contract
//! in group_command.rs. No renderer, external tools, or generated golden is involved.

use motolii_doc as motolii;

use motolii::doc::store::{
    property, Animate, Composition, ContentKeyframe, ContentTrack, Document, EffectScope, FontRef, Fps, Intent, Interp, Keyframe, KeyframeTrack,
    LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, PropertyLink, RationalTime, Slot, SlotId, SpatialTangent,
    StoreView, TextDocument, TextDocumentStyle, TextJustify, TextStyleId, Value,
};

fn at(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn position() -> PropertyId {
    PropertyId::new(property::POSITION).unwrap()
}

fn document() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 480,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    doc
}

fn layer(doc: &mut Document, id: u64, source: LayerSource) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(30, Some(90), 300) } },
    ])
    .unwrap();
    layer
}

fn put(doc: &mut Document, layer: LayerId, value: [f64; 2]) {
    doc.apply(Intent::SetConstant { layer, property: position(), value: Value::Vec2(value) }).unwrap();
}

fn value(doc: &Document, layer: LayerId, frame: i64) -> Value {
    doc.view().value_at(layer, &position(), at(frame)).unwrap().unwrap()
}

fn track(keys: impl IntoIterator<Item = (i64, [f64; 2])>) -> KeyframeTrack {
    KeyframeTrack::try_from_keys(
        keys.into_iter()
            .map(|(frame, value)| Keyframe { t: at(frame), value: Value::Vec2(value), interp: Interp::Linear, spatial: None })
            .collect(),
    )
    .unwrap()
}

#[test]
fn a_failed_ordered_batch_after_undo_preserves_the_value_head_and_redo_branch() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    put(&mut doc, layer, [10.0, 20.0]);
    doc.mark_undo_floor();
    put(&mut doc, layer, [70.0, 80.0]);
    assert!(doc.undo());
    let head = doc.edit_head();
    let history = doc.history_depth();
    assert_eq!(history, (0, 1));
    assert_eq!(value(&doc, layer, 60), Value::Vec2([10.0, 20.0]));
    let existing_meta = doc.view().meta(layer).unwrap().unwrap();

    let rejected = doc.apply_all([
        Intent::SetConstant { layer, property: position(), value: Value::Vec2([999.0, 999.0]) },
        // SetMeta is creation-only: the preceding write must roll back when this fails.
        Intent::SetMeta { layer, meta: existing_meta },
    ]);
    assert!(rejected.is_err());
    assert_eq!(doc.edit_head(), head);
    assert_eq!(doc.history_depth(), history);
    assert_eq!(value(&doc, layer, 60), Value::Vec2([10.0, 20.0]));
    assert!(doc.redo(), "a refused action must not erase the previous future");
    assert_eq!(value(&doc, layer, 60), Value::Vec2([70.0, 80.0]));
    assert!(doc.undo());
    assert_eq!(value(&doc, layer, 60), Value::Vec2([10.0, 20.0]));
}

#[test]
fn changing_an_existing_key_preserves_its_time_curve_tangents_and_neighbors() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    let mut authored = track([(30, [0.0, 0.0]), (60, [60.0, 30.0]), (90, [120.0, 60.0])]);
    authored.insert(Keyframe {
        t: at(60),
        value: Value::Vec2([60.0, 30.0]),
        interp: Interp::Bezier { x1: 0.2, y1: -0.3, x2: 0.8, y2: 1.4 },
        spatial: Some(SpatialTangent { out_tangent: [12.0, 5.0], in_tangent: [-8.0, 3.0] }),
    });
    doc.apply(Intent::SetTrack { layer, property: position(), track: authored.clone() }).unwrap();
    doc.mark_undo_floor();
    let edit = doc.place_checked(layer, &position(), Value::Vec2([75.0, -5.0]), at(60), Animate::now()).unwrap().unwrap();
    doc.apply(edit).unwrap();
    let changed = doc.view().track(layer, &position()).unwrap().unwrap();
    assert_eq!(changed.keys().len(), 3);
    assert_eq!(changed.keys()[0], authored.keys()[0]);
    assert_eq!(changed.keys()[2], authored.keys()[2]);
    assert_eq!(changed.keys()[1].t, authored.keys()[1].t);
    assert_eq!(changed.keys()[1].interp, authored.keys()[1].interp);
    assert_eq!(changed.keys()[1].spatial, authored.keys()[1].spatial);
    assert_eq!(changed.keys()[1].value, Value::Vec2([75.0, -5.0]));
    assert_eq!(doc.history_depth(), (1, 0));
    assert!(doc.undo());
    assert_eq!(doc.view().track(layer, &position()).unwrap().unwrap(), authored);
}

#[test]
fn direct_value_editing_cannot_silently_detach_a_slot_or_driver() {
    let mut doc = document();
    let shared = layer(&mut doc, 1, LayerSource::Shape);
    let driver = layer(&mut doc, 2, LayerSource::Shape);
    let driven = layer(&mut doc, 3, LayerSource::Shape);
    let slot = SlotId("shared-position".to_owned());
    doc.apply(Intent::SetSlots { slots: vec![Slot { id: slot.clone(), track: track([(0, [10.0, 20.0])]) }] }).unwrap();
    doc.apply(Intent::SetPropertySlot { layer: shared, property: position(), slot }).unwrap();
    put(&mut doc, driver, [30.0, 40.0]);
    put(&mut doc, driven, [1.0, 2.0]);
    doc.apply(Intent::SetPropertyModulators {
        layer: driven,
        property: position(),
        modulators: vec![PropertyLink {
            source_layer: driver,
            source_property: position(),
            time_offset: RationalTime::ZERO,
            plugin_id: "motolii.link.identity".to_owned(),
            params: Vec::new(),
        }],
    })
    .unwrap();
    let sources = [shared, driven].map(|layer| doc.view().property_source(layer, &position()).unwrap());
    let slots = doc.view().slots().unwrap();
    let visible = [shared, driven].map(|layer| value(&doc, layer, 60));
    let (head, history) = (doc.edit_head(), doc.history_depth());

    for layer in [shared, driven] {
        assert!(doc.place_checked(layer, &position(), Value::Vec2([99.0, 99.0]), at(60), Animate::Off).is_err());
    }
    assert_eq!([shared, driven].map(|layer| doc.view().property_source(layer, &position()).unwrap()), sources);
    assert_eq!(doc.view().slots().unwrap(), slots);
    assert_eq!([shared, driven].map(|layer| value(&doc, layer, 60)), visible);
    assert_eq!((doc.edit_head(), doc.history_depth()), (head, history));
}

#[test]
fn a_superseded_owner_cannot_cancel_or_overwrite_the_new_preview() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    put(&mut doc, layer, [0.0, 0.0]);
    let (head, history) = (doc.edit_head(), doc.history_depth());
    let first = doc.begin_preview();
    let first_edits = [Intent::SetConstant { layer, property: position(), value: Value::Vec2([10.0, 20.0]) }];
    doc.preview_edits(first, &first_edits).unwrap();
    let second = doc.begin_preview();
    assert_ne!(first, second);
    doc.preview_edits(second, &[Intent::SetConstant { layer, property: position(), value: Value::Vec2([30.0, 40.0]) }]).unwrap();

    assert!(!doc.clear_preview_edits(first));
    assert!(doc.preview_edits(first, &first_edits).is_err());
    assert!(doc.preview_is_current(second));
    assert_eq!(value(&doc, layer, 60), Value::Vec2([30.0, 40.0]));
    assert_eq!(doc.view().without_transients().value_at(layer, &position(), at(60)).unwrap(), Some(Value::Vec2([0.0, 0.0])));
    assert_eq!((doc.edit_head(), doc.history_depth()), (head, history));
    assert!(doc.clear_preview_edits(second));
    assert!(!doc.preview_is_current(second));
    assert_eq!(value(&doc, layer, 60), Value::Vec2([0.0, 0.0]));
    assert_eq!((doc.edit_head(), doc.history_depth()), (head, history));
}

#[test]
fn preview_projection_changes_identity_and_survives_rejected_edits() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    put(&mut doc, layer, [0.0, 0.0]);
    let history = doc.history_depth();
    let owner = doc.begin_preview();
    let edit = |x| Intent::SetConstant { layer, property: position(), value: Value::Vec2([x, 0.0]) };
    doc.preview_edits(owner, &[edit(10.0)]).unwrap();
    let first = doc.view().revision_key();
    doc.preview_edits(owner, &[edit(20.0)]).unwrap();
    let second = doc.view().revision_key();
    assert_ne!(first, second, "equal-sized previews still change the displayed content");
    assert_eq!(value(&doc, layer, 60), Value::Vec2([20.0, 0.0]));
    assert!(doc.preview_edits(owner, &[edit(30.0), Intent::RemoveLayer(layer)]).is_err());
    assert_eq!(doc.view().revision_key(), second);
    assert_eq!(value(&doc, layer, 60), Value::Vec2([20.0, 0.0]));
    let existing_meta = doc.view().meta(layer).unwrap().unwrap();
    assert!(doc.apply_all([edit(40.0), Intent::SetMeta { layer, meta: existing_meta }]).is_err());
    assert_eq!(value(&doc, layer, 60), Value::Vec2([20.0, 0.0]));
    assert_eq!(doc.history_depth(), history);
    doc.clear_preview_edits(owner);
    assert_eq!(value(&doc, layer, 60), Value::Vec2([0.0, 0.0]));
}

#[test]
fn preview_projection_matches_ordered_writes_and_does_not_own_history() {
    use motolii::doc::store::{LayerAttrsPatch, rect_shape};
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    let edits = vec![
        Intent::SetTrack { layer, property: position(), track: track([(0, [10.0, 0.0]), (60, [20.0, 0.0])]) },
        Intent::SetConstant { layer, property: position(), value: Value::Vec2([33.0, 44.0]) },
        Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("First".into()), hidden: Some(true), ..Default::default() } },
        Intent::SetAttrs { layer, patch: LayerAttrsPatch { name: Some("Last".into()), ..Default::default() } },
        Intent::SetTiming { layer, timing: LayerTiming::place(10, Some(60), 300) },
        Intent::SetTiming { layer, timing: LayerTiming::place(20, Some(80), 300) },
        Intent::SetShapes { layer, shapes: vec![rect_shape([255, 0, 0, 255], [10.0, 20.0])] },
        Intent::SetShapes { layer, shapes: vec![rect_shape([0, 255, 0, 255], [30.0, 40.0])] },
    ];
    let owner = doc.begin_preview();
    let history = doc.history_depth();
    doc.preview_edits(owner, &edits).unwrap();
    let shown = (value(&doc, layer, 60), doc.view().attrs(layer).unwrap(), doc.view().meta(layer).unwrap(), doc.view().shapes(layer).unwrap());
    assert_eq!(doc.history_depth(), history);
    assert_eq!(shown.0, Value::Vec2([33.0, 44.0]));
    assert_eq!(shown.1.as_ref().unwrap().name, "Last");
    assert!(shown.1.as_ref().unwrap().hidden);
    assert!(doc.view().without_transients().shapes(layer).unwrap().is_empty());
    doc.apply_all(edits).unwrap();
    assert_eq!((value(&doc, layer, 60), doc.view().attrs(layer).unwrap(), doc.view().meta(layer).unwrap(), doc.view().shapes(layer).unwrap()), shown);
}

#[test]
fn camera_preview_projects_track_and_constant_without_editing_the_record() {
    let mut doc = document();
    let camera = PropertyId::camera(property::CAMERA_CENTER).unwrap();
    doc.apply(Intent::SetCameraConstant { property: camera.clone(), value: Value::Vec2([1.0, 2.0]) }).unwrap();
    let owner = doc.begin_preview();
    let history = doc.history_depth();
    doc.preview_edits(owner, &[Intent::SetCameraTrack { property: camera.clone(), track: track([(0, [0.0, 0.0]), (60, [60.0, 30.0])]) }]).unwrap();
    assert_eq!(doc.view().camera_value_at(&camera, at(30)).unwrap(), Some(Value::Vec2([30.0, 15.0])));
    doc.preview_edits(owner, &[
        Intent::SetCameraTrack { property: camera.clone(), track: track([(0, [0.0, 0.0]), (60, [60.0, 30.0])]) },
        Intent::SetCameraConstant { property: camera.clone(), value: Value::Vec2([7.0, 8.0]) },
    ]).unwrap();
    assert_eq!(doc.view().camera_value_at(&camera, at(30)).unwrap(), Some(Value::Vec2([7.0, 8.0])));
    assert_eq!(doc.view().without_transients().camera_value_at(&camera, at(30)).unwrap(), Some(Value::Vec2([1.0, 2.0])));
    assert_eq!(doc.history_depth(), history);
    doc.clear_preview_edits(owner);
    assert_eq!(doc.view().camera_value_at(&camera, at(30)).unwrap(), Some(Value::Vec2([1.0, 2.0])));
}

#[test]
fn undo_and_redo_retire_the_preview_that_started_on_the_old_history_head() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    put(&mut doc, layer, [10.0, 20.0]);
    doc.mark_undo_floor();
    put(&mut doc, layer, [70.0, 80.0]);
    let owner = doc.begin_preview();
    doc.preview_edits(owner, &[Intent::SetConstant { layer, property: position(), value: Value::Vec2([99.0, 99.0]) }]).unwrap();
    assert!(doc.undo());
    assert!(!doc.preview_is_current(owner));
    assert!(!doc.clear_preview_edits(owner));
    assert_eq!(value(&doc, layer, 60), Value::Vec2([10.0, 20.0]));
    let owner = doc.begin_preview();
    doc.preview_edits(owner, &[Intent::SetConstant { layer, property: position(), value: Value::Vec2([-1.0, -2.0]) }]).unwrap();
    assert!(doc.redo());
    assert!(!doc.preview_is_current(owner));
    assert_eq!(value(&doc, layer, 60), Value::Vec2([70.0, 80.0]));
    assert_eq!(doc.history_depth(), (1, 0));
}

fn text() -> TextDocument {
    let mut content = ContentTrack::new();
    for (frame, line) in [(30, "First line"), (60, "Second line")] {
        content.insert(ContentKeyframe { t: at(frame), content: line.to_owned() });
    }
    TextDocument {
        content,
        justify: TextJustify::Left,
        wrap_size: None,
        slot_id: None,
        styles: vec![TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef::default(),
            size: 32.0,
            fill: [1.0; 4],
            line_height: None,
            tracking: 0.0,
            axes: Vec::new(),
            features: Vec::new(),
        }],
        ranges: Vec::new(),
        alignment: Default::default(),
        runs: Vec::new(),
    }
}



#[test]
fn a_text_edit_refused_by_the_document_cannot_be_shown_as_a_valid_preview() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Text);
    let original = text();
    doc.apply(Intent::SetTextDocument { layer, document: original.clone() }).unwrap();
    let mut invalid = original.clone();
    invalid.runs = vec![motolii::doc::store::TextRun { len: 0, style: TextStyleId(0) }];
    let invalid = Intent::SetTextDocument { layer, document: invalid };
    let (head, history) = (doc.edit_head(), doc.history_depth());
    assert!(doc.apply(invalid.clone()).is_err());
    let owner = doc.begin_preview();
    assert!(doc.preview_edits(owner, &[invalid]).is_err());
    assert_eq!(doc.view().text_document(layer).unwrap().unwrap(), original);
    assert_eq!((doc.edit_head(), doc.history_depth()), (head, history));
}

#[test]
fn removing_parent_removes_descendants_and_undo_restores_their_keys() {
    use motolii::doc::store::LayerAttrsPatch;
    let mut doc = document();
    let group = layer(&mut doc, 1, LayerSource::Group);
    let child = layer(&mut doc, 2, LayerSource::Shape);
    let grandchild = layer(&mut doc, 3, LayerSource::Shape);
    let sibling = layer(&mut doc, 4, LayerSource::Shape);
    doc.apply_all([
        Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } },
        Intent::SetAttrs { layer: grandchild, patch: LayerAttrsPatch { parent: Some(Some(child)), ..Default::default() } },
        Intent::SetTrack { layer: grandchild, property: position(), track: track([(30,[1.0,2.0]),(60,[3.0,4.0])]) },
    ]).unwrap();
    doc.mark_undo_floor();
    doc.apply_all([Intent::RemoveLayer(group), Intent::RemoveLayer(child)]).unwrap();
    assert_eq!(doc.view().layers(), vec![sibling]);
    assert_eq!(doc.history_depth(), (1,0));
    assert!(doc.undo());
    assert_eq!(doc.view().layers(), vec![group,child,grandchild,sibling]);
    assert_eq!(doc.view().attrs(grandchild).unwrap().unwrap().parent,Some(child));
    assert_eq!(doc.view().track(grandchild,&position()).unwrap().unwrap().keys().len(),2);
    assert!(doc.redo());
    assert_eq!(doc.view().layers(),vec![sibling]);
}

#[test]
fn deleting_a_locked_descendant_rejects_the_whole_subtree() {
    use motolii::doc::store::LayerAttrsPatch;
    let mut doc=document();
    let group=layer(&mut doc,1,LayerSource::Group);
    let child=layer(&mut doc,2,LayerSource::Shape);
    doc.apply(Intent::SetAttrs{layer:child,patch:LayerAttrsPatch{parent:Some(Some(group)),locked:Some(true),..Default::default()}}).unwrap();
    doc.mark_undo_floor();
    assert!(doc.apply(Intent::RemoveLayer(group)).is_err());
    assert_eq!(doc.view().layers(),vec![group,child]);
    assert_eq!(doc.history_depth(),(0,0));
}

#[test]
fn ungroup_reparents_children_before_removing_the_container() {
    use motolii::doc::store::LayerAttrsPatch;
    let mut doc=document();
    let child=layer(&mut doc,1,LayerSource::Shape);
    put(&mut doc,child,[12.0,24.0]);
    let group=doc.group_layers(&[child]).unwrap().unwrap();
    doc.mark_undo_floor();
    assert_eq!(doc.ungroup_layers(&[group]).unwrap(),vec![child]);
    assert_eq!(doc.view().layers(),vec![child]);
    assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent,None);
    assert!(doc.undo());
    assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent,Some(group));
    assert!(doc.apply(Intent::SetAttrs{layer:child,patch:LayerAttrsPatch{parent:Some(Some(LayerId(999))),..Default::default()}}).is_err());
}



#[test]
fn animate_decides_whether_a_touch_becomes_a_key_or_moves_the_whole_motion() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    // Animate off, no keys: the value changes and no key appears.
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([10.0, 10.0]), at(30), Animate::Off).unwrap().unwrap()).unwrap();
    assert!(doc.view().track(layer, &position()).unwrap().is_none());
    // Animate on, no keys: the first key is born at the playhead.
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([20.0, 10.0]), at(30), Animate::now()).unwrap().unwrap()).unwrap();
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([40.0, 10.0]), at(60), Animate::now()).unwrap().unwrap()).unwrap();
    let keys = doc.view().track(layer, &position()).unwrap().unwrap().keys().len();
    assert_eq!(keys, 2);
    // Animate off, keyed: every key moves by the same difference, no key is added.
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([25.0, 15.0]), at(45), Animate::Off).unwrap().unwrap()).unwrap();
    let track = doc.view().track(layer, &position()).unwrap().unwrap();
    assert_eq!(track.keys().len(), 2);
    assert_eq!(track.keys()[0].value, Value::Vec2([15.0, 15.0]));
    assert_eq!(track.keys()[1].value, Value::Vec2([35.0, 15.0]));
    assert_eq!(value(&doc, layer, 45), Value::Vec2([25.0, 15.0]));
}

#[test]
fn animate_from_keys_the_untouched_value_where_animate_was_turned_on() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Shape);
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([10.0, 10.0]), at(30), Animate::Off).unwrap().unwrap()).unwrap();
    // Animate turned on at 30, the touch lands at 60: both ends appear, the origin keeps the old value.
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([40.0, 10.0]), at(60), Animate::from(at(30))).unwrap().unwrap()).unwrap();
    let track = doc.view().track(layer, &position()).unwrap().unwrap();
    assert_eq!(track.keys().len(), 2);
    assert_eq!((track.keys()[0].t, track.keys()[0].value.clone()), (at(30), Value::Vec2([10.0, 10.0])));
    assert_eq!((track.keys()[1].t, track.keys()[1].value.clone()), (at(60), Value::Vec2([40.0, 10.0])));
    // Already keyed: the origin is not written again, only the playhead.
    doc.apply(doc.place_checked(layer, &position(), Value::Vec2([80.0, 10.0]), at(90), Animate::from(at(30))).unwrap().unwrap()).unwrap();
    assert_eq!(doc.view().track(layer, &position()).unwrap().unwrap().keys().len(), 3);
    // A touch at the origin itself is one key, as with Animate::Now.
    let scale = PropertyId::new(property::SCALE).unwrap();
    doc.apply(doc.place_checked(layer, &scale, Value::Vec2([2.0, 1.0]), at(30), Animate::from(at(30))).unwrap().unwrap()).unwrap();
    assert_eq!(doc.view().track(layer, &scale).unwrap().unwrap().keys().len(), 1);
    // Newborn keys take the shape Animate carries; keys already there keep theirs.
    let eased = Interp::Bezier { x1: 0.42, y1: 0.0, x2: 0.58, y2: 1.0 };
    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(doc.place_checked(layer, &opacity, Value::F64(0.5), at(60), Animate::From { origin: at(30), interp: eased }).unwrap().unwrap()).unwrap();
    let keys = doc.view().track(layer, &opacity).unwrap().unwrap().keys().to_vec();
    assert!(keys.iter().all(|k| k.interp == eased), "{keys:?}");
    doc.apply(doc.place_checked(layer, &opacity, Value::F64(0.2), at(90), Animate::now()).unwrap().unwrap()).unwrap();
    let keys = doc.view().track(layer, &opacity).unwrap().unwrap().keys().to_vec();
    assert_eq!(keys.iter().map(|k| k.interp).collect::<Vec<_>>(), vec![eased, eased, Interp::Linear]);
}


/// 作ってから並べる 2 段が同じ履歴の段に乗る。Undo 一発で作った層ごと消える。
#[test]
fn apply_then_lands_both_stages_in_one_undo_step() {
    let mut doc = document();
    let a = layer(&mut doc, 1, LayerSource::Shape);
    let b = LayerId(2);
    doc.apply_then(
        [
            Intent::AddLayer(b),
            Intent::SetMeta {
                layer: b,
                meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(30, Some(90), 300) },
            },
        ],
        |doc| doc.move_layer_intents(&[b], Some(a), "after", at(0)).map(|(intents, _)| intents),
    )
    .unwrap();
    let order = |doc: &Document, layer: LayerId| doc.view().meta(layer).unwrap().unwrap().order;
    assert!(order(&doc, b) < order(&doc, a), "after = 目標の下");
    assert!(doc.undo());
    assert!(!doc.view().layers().contains(&b));
    assert_eq!(order(&doc, a), 1);
}
