//! 仮の編集の姿は、解いた層まで含めて元に戻る。解いた層を見るので絵の家で試す。

use motolii_doc::store::*;
use motolii_render::picture::resolve::resolved_layers;

fn track(keys: impl IntoIterator<Item = (i64, [f64; 2])>) -> KeyframeTrack {
    KeyframeTrack::try_from_keys(
        keys.into_iter()
            .map(|(frame, value)| Keyframe { t: at(frame), value: Value::Vec2(value), interp: Interp::Linear, spatial: None })
            .collect(),
    )
    .unwrap()
}

fn text_document() -> TextDocument {
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

fn at(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn position() -> PropertyId {
    PropertyId::new(property::POSITION).unwrap()
}

fn document() -> Document {
    let mut doc = Document::new().with_programs(motolii_render::extensions::bundled());
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


fn projected(view: StoreView<'_>, layer: LayerId, frame: i64) -> (LayerTiming, Value, String, i64) {
    let timing = view.meta(layer).unwrap().unwrap().timing;
    let value = view.value_at(layer, &position(), at(frame)).unwrap().unwrap();
    let content = view.text_document(layer).unwrap().unwrap().content.eval(at(frame)).to_owned();
    let source_frame =
        resolved_layers(&view, at(frame)).unwrap().into_iter().find(|resolved| resolved.id == layer).unwrap().source_frame;
    (timing, value, content, source_frame)
}

#[test]
fn moving_a_timed_text_layer_previews_and_commits_one_identical_composition() {
    let mut doc = document();
    let layer = layer(&mut doc, 1, LayerSource::Text);
    let authored_track = track([(30, [0.0, 0.0]), (60, [60.0, 30.0]), (90, [120.0, 60.0])]);
    let authored_text = text_document();
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
