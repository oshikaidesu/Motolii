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
            stroke_color: None,
            stroke_width: 0.0,
            stroke_over_fill: false,
            axes: Vec::new(),
            features: Vec::new(),
        }],
        ranges: Vec::new(),
        alignment: Default::default(),
        runs: Vec::new(),
    }
}

fn projected(view: StoreView<'_>, layer: LayerId, frame: i64) -> (LayerTiming, Value, String, i64) {
    let timing = view.meta(layer).unwrap().unwrap().timing;
    let value = view.value_at(layer, &position(), at(frame)).unwrap().unwrap();
    let content = view.text_document(layer).unwrap().unwrap().content.eval(at(frame)).to_owned();
    let source_frame =
        view.resolved_layers(at(frame)).unwrap().into_iter().find(|resolved| resolved.id == layer).unwrap().source_frame;
    (timing, value, content, source_frame)
}

#[test]
fn moving_a_timed_text_layer_previews_and_commits_one_identical_composition() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Text);
    let authored_track = track([(30, [0.0, 0.0]), (60, [60.0, 30.0]), (90, [120.0, 60.0])]);
    let authored_text = text();
    doc.apply_all([
        Intent::SetTrack { layer, property: position(), track: authored_track.clone() },
        Intent::SetTextDocument { layer, document: authored_text.clone() },
    ])
    .unwrap();
    doc.mark_undo_floor();
    let fixed_frame = 60;
    let before = projected(doc.view(), layer, fixed_frame);
    assert_eq!((before.1.clone(), before.2.as_str(), before.3), (Value::Vec2([60.0, 30.0]), "Second line", 30));

    let shifted_track = KeyframeTrack::try_from_keys(
        authored_track.keys().iter().map(|key| Keyframe { t: key.t.try_add(at(15)).unwrap(), ..key.clone() }).collect(),
    )
    .unwrap();
    let mut shifted_text = authored_text.clone();
    shifted_text.content = ContentTrack::new();
    for key in authored_text.content.keys() {
        shifted_text.content.insert(ContentKeyframe { t: key.t.try_add(at(15)).unwrap(), content: key.content.clone() });
    }
    let edits = vec![
        Intent::SetTiming { layer, timing: LayerTiming { start: 45, ..before.0 } },
        Intent::SetTrack { layer, property: position(), track: shifted_track },
        Intent::SetTextDocument { layer, document: shifted_text },
    ];
    let (head, history) = (doc.edit_head(), doc.history_depth());
    let owner = doc.begin_preview();
    doc.preview_edits(owner, &edits).unwrap();
    let shown = projected(doc.view(), layer, fixed_frame);
    assert_eq!(shown.0.start, 45);
    assert_eq!((shown.1.clone(), shown.2.as_str(), shown.3), (Value::Vec2([30.0, 15.0]), "First line", 15));
    assert_eq!(projected(doc.view().without_transients(), layer, fixed_frame), before);
    assert_eq!((doc.edit_head(), doc.history_depth()), (head, history));
    assert!(doc.clear_preview_edits(owner));
    assert_eq!(projected(doc.view(), layer, fixed_frame), before);
    assert_eq!((doc.edit_head(), doc.history_depth()), (head, history));

    let owner = doc.begin_preview();
    doc.preview_edits(owner, &edits).unwrap();
    // Reading another playhead time must not retime the recorded operation.
    let _ = projected(doc.view(), layer, 90);
    doc.apply_all(edits).unwrap();
    assert!(!doc.preview_is_current(owner));
    assert_eq!(projected(doc.view(), layer, fixed_frame), shown);
    assert_eq!(doc.history_depth(), (1, 0));
    assert_eq!(doc.view().text_document(layer).unwrap().unwrap().styles, authored_text.styles);
    assert!(doc.undo());
    assert_eq!(projected(doc.view(), layer, fixed_frame), before);
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
fn a_placement_effect_multiplies_the_layer_and_delays_later_copies() {
    use motolii::doc::store::{placement, EffectId, EffectInstance};
    let mut doc = document();
    let paper = layer(&mut doc, 1, LayerSource::Shape);
    put(&mut doc, paper, [100.0, 50.0]);
    let repeat = EffectId(0);
    doc.apply_all([
        Intent::SetEffects {
            layer: paper,
            effects: vec![
                EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() },
                EffectInstance { id: EffectId(1), plugin_id: "motolii.blur".to_owned() },
            ],
        },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(3.0) },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([20.0, 0.0]) },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "opacity_each").unwrap(), value: Value::F64(-0.25) },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "delay_each").unwrap(), value: Value::F64(1.0) },
    ])
    .unwrap();

    // 層は 30f から居る。90f では全部の配置が出ていて、番号順に右へ 20 ずつずれる。
    let copies = doc.view().resolved_layers(at(90)).unwrap();
    assert_eq!(copies.iter().map(|c| (c.id, c.copy)).collect::<Vec<_>>(), [(paper, 0), (paper, 1), (paper, 2)]);
    let origins: Vec<[f32; 2]> = copies.iter().map(|c| c.placement.transform.translation.to_array()).collect();
    assert_eq!(origins, [[100.0, 50.0], [120.0, 50.0], [140.0, 50.0]]);
    assert_eq!(copies.iter().map(|c| c.placement.opacity).collect::<Vec<_>>(), [1.0, 0.75, 0.5]);
    // 配置効果より上には何も無く、下の blur は全体に掛かる側へ残る。
    assert!(copies.iter().all(|c| c.effects.is_empty()));
    assert!(copies.iter().all(|c| c.after_effects.iter().map(|e| e.plugin_id.as_str()).eq(["motolii.blur"])));

    // 45f では 2 番目(1 秒遅れ)はまだ 15f の姿 = 層の始まる前なので出ない。
    let early = doc.view().resolved_layers(at(45)).unwrap();
    assert_eq!(early.iter().map(|c| c.copy).collect::<Vec<_>>(), [0]);
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

#[test]
fn a_repeater_on_a_group_hands_out_the_children_instead_of_the_group() {
    use motolii::doc::store::{placement, EffectId, EffectInstance, LayerAttrsPatch};
    let mut doc = document();
    let group = layer(&mut doc, 1, LayerSource::Group);
    let circle = layer(&mut doc, 2, LayerSource::Shape);
    let square = layer(&mut doc, 3, LayerSource::Shape);
    for child in [circle, square] {
        doc.apply(Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } }).unwrap();
    }
    put(&mut doc, group, [100.0, 100.0]);
    put(&mut doc, circle, [10.0, 0.0]);
    put(&mut doc, square, [0.0, 10.0]);
    let repeat = EffectId(0);
    doc.apply_all([
        Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }] },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(4.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "pick").unwrap(), value: Value::F64(1.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([50.0, 0.0]) },
    ])
    .unwrap();
    // Iterate: 4 placements alternate circle, square, circle, square. The children are not drawn on their own.
    let out = doc.view().resolved_layers(at(60)).unwrap();
    let mut seen: Vec<(LayerId, u32, [f32; 2])> = out.iter().map(|c| (c.id, c.copy, c.placement.transform.translation.to_array())).collect();
    seen.sort_by_key(|(id, copy, _)| (id.0, *copy));
    assert_eq!(seen, [
        (circle, 0, [110.0, 100.0]), (circle, 2, [210.0, 100.0]),
        (square, 1, [150.0, 110.0]), (square, 3, [250.0, 110.0]),
    ]);
    assert!(out.iter().all(|c| c.id != group));
    // Random with all the weight on the square: every placement is a square.
    doc.apply_all([
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "pick").unwrap(), value: Value::F64(0.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "share.2").unwrap(), value: Value::F64(0.0) },
    ])
    .unwrap();
    let out = doc.view().resolved_layers(at(60)).unwrap();
    assert_eq!(out.len(), 4);
    assert!(out.iter().all(|c| c.id == square));
    // Whole group: every placement carries both children.
    doc.apply(Intent::SetConstant { layer: group, property: PropertyId::effect_scope(repeat), value: Value::Enum(EffectScope::Whole.enum_value()) }).unwrap();
    let out = doc.view().resolved_layers(at(60)).unwrap();
    assert_eq!(out.len(), 8);
    assert_eq!(out.iter().filter(|c| c.id == circle).count(), 4);
    assert_eq!(out.iter().filter(|c| c.id == square && c.copy == 3).map(|c| c.placement.transform.translation.to_array()).next(), Some([250.0, 110.0]));
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
