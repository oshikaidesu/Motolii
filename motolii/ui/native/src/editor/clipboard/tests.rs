use crate::edit::{Animate, Document, Intent};
use super::*;
use crate::doc::store::{
    property, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

#[test]
fn copied_layers_survive_source_deletion_and_paste_as_one_history_step() {
    let layer = LayerId(1);
    let mut doc = Document::new().with_programs(crate::render::extensions::bundled());
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            value: Value::Vec2([42.0, 17.0]),
        },
    ])
    .unwrap();
    let clipboard = Clipboard::default();
    clipboard.copy_layers(&doc, &[layer]).unwrap();
    doc.apply(Intent::RemoveLayer(layer)).unwrap();
    let before = doc.history_depth().0;

    let PasteResult::Layers(pasted) = clipboard.paste(&mut doc, None, 0).unwrap() else {
        panic!("layer clipboard changed payload kind")
    };
    assert_eq!(pasted.len(), 1);
    assert_eq!(doc.history_depth().0, before + 1);
    assert_eq!(
        doc.view()
            .value_at(
                pasted[0],
                &PropertyId::new(property::POSITION).unwrap(),
                RationalTime::ZERO,
            )
            .unwrap(),
        Some(Value::Vec2([42.0, 17.0]))
    );
}

#[test]
fn copied_keys_keep_spacing_and_paste_at_the_requested_frame() {
    let layer = LayerId(1);
    let property = PropertyId::new(property::OPACITY).unwrap();
    let fps = crate::doc::store::Fps::try_new(30, 1).unwrap();
    let mut track = KeyframeTrack::new();
    for (frame, value) in [(3, 0.2), (9, 0.8)] {
        track.insert(Keyframe {
            t: RationalTime::try_from_frame(frame, fps).unwrap(),
            value: Value::F64(value),
            interp: crate::doc::store::Interp::Linear,
            spatial: None,
        });
    }
    let mut doc = Document::new().with_programs(crate::render::extensions::bundled());
    doc.apply_all([
        Intent::SetComposition(crate::doc::store::Composition {
            width: 640,
            height: 480,
            fps,
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }),
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: 0,
                timing: LayerTiming::place(0, None, 300),
            },
        },
        Intent::SetTrack {
            layer,
            property: property.clone(),
            track,
        },
    ])
    .unwrap();
    let clipboard = Clipboard::default();
    clipboard
        .copy_keys(
            &doc,
            &[
                KeySel {
                    layer,
                    property: Some(property.clone()),
                    at_sec: 3.0 / 30.0,
                },
                KeySel {
                    layer,
                    property: Some(property.clone()),
                    at_sec: 9.0 / 30.0,
                },
            ],
        )
        .unwrap();
    let PasteResult::Keys(keys) = clipboard.paste(&mut doc, Some(layer), 30).unwrap() else {
        panic!("key clipboard changed payload kind")
    };
    assert_eq!(keys.len(), 2);
    let frames: Vec<_> = doc
        .view()
        .track(layer, &property)
        .unwrap()
        .unwrap()
        .keys()
        .iter()
        .map(|key| key.t.try_to_frame_round(fps).unwrap())
        .collect();
    assert_eq!(frames, vec![3, 9, 30, 36]);
}

#[test]
fn key_paste_uses_the_selected_destination_and_undo_restores_its_track() {
    let mut doc = Document::new().with_programs(crate::render::extensions::bundled());
    let fps = crate::doc::store::Fps::try_new(30, 1).unwrap();
    let property = PropertyId::new(property::OPACITY).unwrap();
    let source = LayerId(1);
    let destination = LayerId(2);
    doc.apply(Intent::SetComposition(crate::doc::store::Composition {
        width: 640,
        height: 480,
        fps,
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    for (layer, frame, value) in [(source, 3, 0.2), (destination, 30, 0.8)] {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::try_from_frame(frame, fps).unwrap(),
            value: Value::F64(value),
            interp: crate::doc::store::Interp::Hold,
            spatial: None,
        });
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
            Intent::SetTrack {
                layer,
                property: property.clone(),
                track,
            },
        ])
        .unwrap();
    }
    let clipboard = Clipboard::default();
    clipboard
        .copy_keys(
            &doc,
            &[KeySel {
                layer: source,
                property: Some(property.clone()),
                at_sec: 0.1,
            }],
        )
        .unwrap();
    let source_before = doc.view().track(source, &property).unwrap().unwrap();
    let destination_before = doc.view().track(destination, &property).unwrap().unwrap();
    let history = doc.history_depth().0;
    let PasteResult::Keys(keys) = clipboard.paste(&mut doc, Some(destination), 30).unwrap()
    else {
        panic!("expected keys")
    };
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].layer, destination);
    assert_eq!(keys[0].at_sec, 1.0);
    assert_eq!(
        doc.view().track(source, &property).unwrap().unwrap(),
        source_before
    );
    let pasted = doc.view().track(destination, &property).unwrap().unwrap();
    assert_eq!(pasted.keys().len(), 1);
    assert_eq!(pasted.keys()[0].value, Value::F64(0.2));
    assert_eq!(pasted.keys()[0].interp, crate::doc::store::Interp::Hold);
    assert_eq!(doc.history_depth().0, history + 1);
    assert!(doc.undo());
    assert_eq!(
        doc.view().track(destination, &property).unwrap().unwrap(),
        destination_before
    );
    assert!(doc.redo());
    assert_eq!(
        doc.view().track(destination, &property).unwrap().unwrap(),
        pasted
    );

    let slot = crate::doc::store::SlotId("destination.opacity".into());
    doc.apply_all([
        Intent::SetSlots {
            slots: vec![crate::doc::store::Slot {
                id: slot.clone(),
                track: pasted,
            }],
        },
        Intent::SetPropertySlot {
            layer: destination,
            property: property.clone(),
            slot,
        },
    ])
    .unwrap();
    let history = doc.history_depth();
    assert!(clipboard.paste(&mut doc, Some(destination), 60).is_err());
    assert_eq!(doc.history_depth(), history);
}

#[test]
fn aggregate_text_cut_paste_preserves_content_and_properties_as_one_undo_step() {
    use crate::doc::store::{ContentTrack, Fps, Interp, TextJustify};
    let fps = Fps::try_new(30, 1).unwrap();
    let source = LayerId(1);
    let destination = LayerId(2);
    let shape = LayerId(3);
    let property = PropertyId::new(property::OPACITY).unwrap();
    let time = |frame| RationalTime::try_from_frame(frame, fps).unwrap();
    let mut doc = Document::new().with_programs(crate::render::extensions::bundled());
    doc.apply(Intent::SetComposition(crate::doc::store::Composition {
        width: 640,
        height: 480,
        fps,
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    for layer in [source, destination, shape] {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: if layer == shape {
                        LayerSource::Shape
                    } else {
                        LayerSource::Text
                    },
                    order: layer.0 as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
    }
    for layer in [source, destination] {
        let mut content = ContentTrack::new();
        for (frame, word) in if layer == source {
            vec![(0, "before"), (3, "first"), (9, "second")]
        } else {
            vec![(0, "destination")]
        } {
            content.insert(ContentKeyframe {
                t: time(frame),
                content: word.into(),
            });
        }
        doc.apply(Intent::SetTextDocument {
            layer,
            document: TextDocument {
                content,
                justify: TextJustify::Center,
                wrap_size: Some([200.0, 100.0]),
                styles: Vec::new(),
                slot_id: None,
                ranges: Vec::new(),
                alignment: Default::default(),
                runs: Vec::new(),
            },
        })
        .unwrap();
    }
    let mut track = KeyframeTrack::new();
    for (frame, value) in [(3, 0.2), (9, 0.8)] {
        track.insert(Keyframe {
            t: time(frame),
            value: Value::F64(value),
            interp: Interp::Hold,
            spatial: None,
        });
    }
    doc.apply(Intent::SetTrack {
        layer: source,
        property: property.clone(),
        track,
    })
    .unwrap();
    let source_text = doc.view().text_document(source).unwrap().unwrap();
    let source_track = doc.view().track(source, &property).unwrap().unwrap();
    let destination_text = doc.view().text_document(destination).unwrap().unwrap();
    let clipboard = Clipboard::default();
    let selected = [
        KeySel {
            layer: source,
            property: None,
            at_sec: 0.1,
        },
        KeySel {
            layer: source,
            property: None,
            at_sec: 0.3,
        },
        KeySel {
            layer: source,
            property: Some(property.clone()),
            at_sec: 0.3,
        },
    ];
    assert_eq!(clipboard.copy_keys(&doc, &selected).unwrap(), 4);

    let history = doc.history_depth();
    assert!(clipboard.paste(&mut doc, Some(shape), 30).is_err());
    assert_eq!(doc.history_depth(), history);
    assert!(doc.view().track(shape, &property).unwrap().is_none());

    let selections: Vec<_> = selected
        .iter()
        .map(|key| (key.layer, key.property.clone(), key.at_sec))
        .collect();
    let cut =
        crate::editor::timeline_edit::delete_key_selection_intents(&doc, &selections, time(0))
            .unwrap();
    doc.apply_all(cut).unwrap();
    assert_eq!(
        doc.view()
            .text_document(source)
            .unwrap()
            .unwrap()
            .content
            .keys()
            .len(),
        1
    );
    let after_cut = doc.history_depth().0;
    let PasteResult::Keys(pasted) = clipboard.paste(&mut doc, Some(destination), 30).unwrap()
    else {
        panic!("expected keys")
    };
    assert_eq!(pasted.len(), 4);
    assert!(pasted.iter().all(|key| key.layer == destination));
    let result = doc.view().text_document(destination).unwrap().unwrap();
    assert_eq!(result.justify, destination_text.justify);
    assert_eq!(result.wrap_size, destination_text.wrap_size);
    assert_eq!(result.content.eval(time(0)), "destination");
    assert_eq!(result.content.eval(time(30)), "first");
    assert_eq!(result.content.eval(time(36)), "second");
    let result_track = doc.view().track(destination, &property).unwrap().unwrap();
    assert_eq!(
        result_track
            .keys()
            .iter()
            .map(|key| key.t)
            .collect::<Vec<_>>(),
        vec![time(30), time(36)]
    );
    assert_eq!(doc.history_depth().0, after_cut + 1);
    assert!(doc.undo());
    assert_eq!(
        doc.view().text_document(destination).unwrap().unwrap(),
        destination_text
    );
    assert!(doc.view().track(destination, &property).unwrap().is_none());
    assert!(doc.undo());
    assert_eq!(
        doc.view().text_document(source).unwrap().unwrap(),
        source_text
    );
    assert_eq!(
        doc.view().track(source, &property).unwrap().unwrap(),
        source_track
    );
    assert!(doc.redo());
    assert!(doc.redo());
    assert_eq!(
        doc.view().text_document(destination).unwrap().unwrap(),
        result
    );

    assert_eq!(
        clipboard
            .copy_keys(
                &doc,
                &[KeySel {
                    layer: source,
                    property: None,
                    at_sec: 0.0
                }]
            )
            .unwrap(),
        1
    );
    assert!(clipboard.paste(&mut doc, Some(destination), 90).is_ok());
    assert_eq!(
        doc.view()
            .text_document(destination)
            .unwrap()
            .unwrap()
            .content
            .eval(time(90)),
        "before"
    );
}
