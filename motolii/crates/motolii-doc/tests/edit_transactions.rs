//! One action is one Undo; preview is the same edit projected through StoreView.
//! The key/source rules follow keys_only_when_asked.rs and the grouped-edit contract
//! in group_command.rs. No renderer, external tools, or generated golden is involved.

use motolii_doc as motolii;

use motolii::doc::store::{
    property, Composition, ContentKeyframe, ContentTrack, Document, FontRef, Fps, Intent, Interp, Keyframe, KeyframeTrack,
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
    let edit = doc.place_checked(layer, &position(), Value::Vec2([75.0, -5.0]), at(60)).unwrap().unwrap();
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
        assert!(doc.place_checked(layer, &position(), Value::Vec2([99.0, 99.0]), at(60)).is_err());
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
