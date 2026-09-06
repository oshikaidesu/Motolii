use std::sync::{Arc, Mutex};

use crate::doc::store::{
    ContentKeyframe, Document, Intent, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime,
    StoreError, TextDocument,
};
use crate::editor::session::KeySel;
use crate::editor::timeline_edit::{copy_layers, paste_layers, LayerClipboard};

/// Typed, process-local editor clipboard. The payload owns a layer snapshot, so
/// Cut followed by Paste and pasting after the source project changes are valid.
#[derive(Clone)]
struct CopiedKey {
    layer: LayerId,
    offset_frames: i64,
    value: CopiedKeyValue,
}

#[derive(Clone, PartialEq)]
enum CopiedKeyValue {
    Property(PropertyId, Keyframe),
    Content(ContentKeyframe),
}

#[derive(Clone)]
enum Payload {
    Layers(LayerClipboard),
    Keys {
        source_document: String,
        keys: Vec<CopiedKey>,
    },
}

pub(crate) enum PasteResult {
    Layers(Vec<LayerId>),
    Keys(Vec<KeySel>),
}

#[derive(Clone, Default)]
pub(crate) struct Clipboard(Arc<Mutex<Option<Payload>>>);

impl Clipboard {
    pub(crate) fn copy_layers(
        &self,
        doc: &Document,
        layers: &[LayerId],
    ) -> Result<usize, StoreError> {
        let payload = copy_layers(doc, layers)?;
        let count = layers.len();
        *self.0.lock().unwrap() = Some(Payload::Layers(payload));
        Ok(count)
    }

    pub(crate) fn copy_keys(
        &self,
        doc: &Document,
        selected: &[KeySel],
    ) -> Result<usize, StoreError> {
        let fps = crate::editor::keyframe_edit::document_fps(doc)?;
        let view = doc.view().without_transients();
        let mut copied = Vec::new();
        for selected in selected {
            if !view.has_layer(selected.layer) {
                return Err(StoreError::Property(
                    "Selected keyframe layer no longer exists".into(),
                ));
            }
            let properties = crate::editor::timeline_edit::selected_properties(
                &view,
                selected.layer,
                selected.property.as_ref(),
            );
            let frame = (selected.at_sec * fps.as_f64()).round() as i64;
            for property in properties {
                let Some(track) = view.track(selected.layer, &property)? else {
                    continue;
                };
                let Some(key) = track
                    .keys()
                    .iter()
                    .find(|key| key.t.try_to_frame_round(fps).ok() == Some(frame))
                else {
                    continue;
                };
                let value = CopiedKeyValue::Property(property, key.clone());
                if copied
                    .iter()
                    .any(|known: &CopiedKey| known.layer == selected.layer && known.value == value)
                {
                    continue;
                }
                copied.push(CopiedKey {
                    layer: selected.layer,
                    offset_frames: frame,
                    value,
                });
            }
            if crate::editor::timeline_edit::selects_content(selected.property.as_ref()) {
                if let Some(document) = view.text_document(selected.layer)? {
                    for key in document
                        .content
                        .keys()
                        .iter()
                        .filter(|key| key.t.try_to_frame_round(fps).ok() == Some(frame))
                    {
                        let value = CopiedKeyValue::Content(key.clone());
                        if !copied
                            .iter()
                            .any(|known| known.layer == selected.layer && known.value == value)
                        {
                            copied.push(CopiedKey {
                                layer: selected.layer,
                                offset_frames: frame,
                                value,
                            });
                        }
                    }
                }
            }
        }
        let Some(first) = copied.iter().map(|key| key.offset_frames).min() else {
            return Err(StoreError::Property("Select keyframes to copy".into()));
        };
        for key in &mut copied {
            key.offset_frames -= first;
        }
        let count = copied.len();
        *self.0.lock().unwrap() = Some(Payload::Keys {
            source_document: doc.identity(),
            keys: copied,
        });
        Ok(count)
    }

    pub(crate) fn paste(
        &self,
        doc: &mut Document,
        fallback_layer: Option<LayerId>,
        at_frame: i64,
    ) -> Result<PasteResult, StoreError> {
        let payload = self
            .0
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| StoreError::Property("The editor clipboard is empty".into()))?;
        match payload {
            Payload::Layers(payload) => paste_layers(doc, &payload).map(PasteResult::Layers),
            Payload::Keys {
                source_document,
                keys,
            } => {
                let same_document = source_document == doc.identity();
                paste_keys(doc, &keys, same_document, fallback_layer, at_frame)
                    .map(PasteResult::Keys)
            }
        }
    }

    pub(crate) fn has_payload(&self) -> bool {
        self.0.lock().unwrap().is_some()
    }
}

fn paste_keys(
    doc: &mut Document,
    copied: &[CopiedKey],
    same_document: bool,
    fallback_layer: Option<LayerId>,
    at_frame: i64,
) -> Result<Vec<KeySel>, StoreError> {
    let fps = crate::editor::keyframe_edit::document_fps(doc)?;
    let view = doc.view().without_transients();
    let live = view.layers();
    let single_source = copied
        .first()
        .is_some_and(|first| copied.iter().all(|key| key.layer == first.layer));
    if !same_document && !single_source {
        return Err(StoreError::Property(
            "Pasting multi-layer keyframes into another document requires a destination for each source layer".into(),
        ));
    }
    let mut tracks: Vec<(LayerId, PropertyId, KeyframeTrack)> = Vec::new();
    let mut texts: Vec<(LayerId, TextDocument)> = Vec::new();
    let mut selected = Vec::new();
    for copied in copied {
        let layer = if let Some(destination) = fallback_layer.filter(|_| single_source) {
            destination
        } else if same_document && live.contains(&copied.layer) {
            copied.layer
        } else {
            if !single_source {
                return Err(StoreError::Property(
                    "The source layers for this multi-layer keyframe selection no longer exist"
                        .into(),
                ));
            }
            fallback_layer.ok_or_else(|| {
                StoreError::Property("Select a destination layer for these keyframes".into())
            })?
        };
        if !live.contains(&layer) {
            return Err(StoreError::Property(
                "The destination layer no longer exists".into(),
            ));
        }
        let frame = at_frame.checked_add(copied.offset_frames).ok_or_else(|| {
            StoreError::Property("Pasted keyframe time is outside the frame domain".into())
        })?;
        let at = RationalTime::try_from_frame(frame, fps)
            .map_err(|error| StoreError::Property(error.to_string()))?;
        let property = match &copied.value {
            CopiedKeyValue::Property(property, key) => {
                let index = tracks.iter().position(|(known_layer, known_property, _)| {
                    *known_layer == layer && known_property == property
                });
                let index = if let Some(index) = index {
                    index
                } else {
                    crate::editor::functions::lens::require_local_source(&view, layer, property)?;
                    tracks.push((
                        layer,
                        property.clone(),
                        view.track(layer, property)?.unwrap_or_default(),
                    ));
                    tracks.len() - 1
                };
                let mut key = key.clone();
                key.t = at;
                tracks[index].2.insert(key);
                Some(property.clone())
            }
            CopiedKeyValue::Content(key) => {
                let index = if let Some(index) = texts.iter().position(|(known, _)| *known == layer)
                {
                    index
                } else {
                    let document = view.text_document(layer)?.ok_or_else(|| {
                        StoreError::Property("Select a text layer for text keyframes".into())
                    })?;
                    texts.push((layer, document));
                    texts.len() - 1
                };
                let mut key = key.clone();
                key.t = at;
                texts[index].1.content.insert(key);
                Some(PropertyId::new("content")?)
            }
        };
        selected.push(KeySel {
            layer,
            property,
            at_sec: at.as_seconds_f64(),
        });
    }
    drop(view);
    doc.apply_all(
        tracks
            .into_iter()
            .map(|(layer, property, track)| Intent::SetTrack {
                layer,
                property,
                track,
            })
            .chain(
                texts
                    .into_iter()
                    .map(|(layer, document)| Intent::SetTextDocument { layer, document }),
            ),
    )?;
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{
        property, Intent, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
    };

    #[test]
    fn copied_layers_survive_source_deletion_and_paste_as_one_history_step() {
        let layer = LayerId(1);
        let mut doc = Document::new();
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
        let mut doc = Document::new();
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
        let mut doc = Document::new();
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
        let mut doc = Document::new();
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
}

#[cfg(test)]
mod key_ownership_regressions {
    use super::*;
    use crate::doc::store::*;

    #[test]
    fn content_only_copy_and_paste_keep_explicit_content_identity() {
        let mut doc = crate::doc::fixture::build().doc;
        let layer = doc
            .view()
            .layers()
            .into_iter()
            .find(|id| doc.view().text_document(*id).unwrap().is_some())
            .unwrap();
        let fps = crate::editor::keyframe_edit::document_fps(&doc).unwrap();
        let mut document = doc.view().text_document(layer).unwrap().unwrap();
        document.content = ContentTrack::new();
        document.content.insert(ContentKeyframe {
            t: RationalTime::ZERO,
            content: "a".into(),
        });
        let property = PropertyId::new(property::POSITION).unwrap();
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::Vec2([1.0, 2.0]),
            interp: Interp::Linear,
            spatial: None,
        });
        doc.apply_all([
            Intent::SetTextDocument { layer, document },
            Intent::SetTrack {
                layer,
                property: property.clone(),
                track: track.clone(),
            },
        ])
        .unwrap();
        let clipboard = Clipboard::default();
        assert_eq!(
            clipboard
                .copy_keys(
                    &doc,
                    &[KeySel {
                        layer,
                        property: Some(PropertyId::new("content").unwrap()),
                        at_sec: 0.0
                    }]
                )
                .unwrap(),
            1
        );
        let PasteResult::Keys(selected) = clipboard.paste(&mut doc, Some(layer), 20).unwrap()
        else {
            panic!("expected keys")
        };
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].property.as_ref().unwrap().name(), "content");
        assert_eq!(doc.view().track(layer, &property).unwrap().unwrap(), track);
        assert_eq!(
            doc.view()
                .text_document(layer)
                .unwrap()
                .unwrap()
                .content
                .keys()[1]
                .t
                .try_to_frame_round(fps)
                .unwrap(),
            20
        );
    }

    #[test]
    fn missing_multi_layer_sources_cannot_merge_onto_one_fallback() {
        let mut doc = blank_project();
        for id in [1, 2, 3] {
            doc.apply_all([
                Intent::AddLayer(LayerId(id)),
                Intent::SetMeta {
                    layer: LayerId(id),
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order: id as i16,
                        timing: LayerTiming::place(0, None, 100),
                    },
                },
            ])
            .unwrap();
        }
        let property = PropertyId::new(property::OPACITY).unwrap();
        let copied: Vec<_> = [1, 2]
            .into_iter()
            .map(|id| CopiedKey {
                layer: LayerId(id),
                offset_frames: 0,
                value: CopiedKeyValue::Property(
                    property.clone(),
                    Keyframe {
                        t: RationalTime::ZERO,
                        value: Value::F64(id as f64 * 0.1),
                        interp: Interp::Linear,
                        spatial: None,
                    },
                ),
            })
            .collect();
        doc.apply(Intent::RemoveLayer(LayerId(1))).unwrap();
        let history = doc.history_depth();
        assert!(paste_keys(&mut doc, &copied, true, Some(LayerId(3)), 10).is_err());
        assert_eq!(doc.history_depth(), history);
        assert!(doc.view().track(LayerId(3), &property).unwrap().is_none());
    }
}

#[cfg(test)]
mod document_identity_regressions {
    use super::*;
    use crate::doc::store::*;

    fn document() -> Document {
        let mut doc = blank_project();
        for id in [1, 2] {
            let mut track = KeyframeTrack::new();
            track.insert(Keyframe {
                t: RationalTime::ZERO,
                value: Value::F64(id as f64 * 0.1),
                interp: Interp::Linear,
                spatial: None,
            });
            doc.apply_all([
                Intent::AddLayer(LayerId(id)),
                Intent::SetMeta {
                    layer: LayerId(id),
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order: id as i16,
                        timing: LayerTiming::place(0, None, 100),
                    },
                },
                Intent::SetTrack {
                    layer: LayerId(id),
                    property: PropertyId::new(property::OPACITY).unwrap(),
                    track,
                },
            ])
            .unwrap();
        }
        doc
    }

    fn key(layer: u64) -> KeySel {
        KeySel {
            layer: LayerId(layer),
            property: Some(PropertyId::new(property::OPACITY).unwrap()),
            at_sec: 0.0,
        }
    }

    #[test]
    fn matching_layer_numbers_in_another_document_require_explicit_destination() {
        let source = document();
        let mut destination = document();
        assert_ne!(source.identity(), destination.identity());
        let clipboard = Clipboard::default();
        clipboard.copy_keys(&source, &[key(1)]).unwrap();
        let history = destination.history_depth();
        assert!(clipboard.paste(&mut destination, None, 30).is_err());
        assert_eq!(destination.history_depth(), history);
        let PasteResult::Keys(keys) = clipboard
            .paste(&mut destination, Some(LayerId(2)), 30)
            .unwrap()
        else {
            panic!("expected keys")
        };
        assert_eq!(keys[0].layer, LayerId(2));
        let property = PropertyId::new(property::OPACITY).unwrap();
        assert_eq!(
            destination
                .view()
                .track(LayerId(1), &property)
                .unwrap()
                .unwrap()
                .keys()
                .len(),
            1
        );
        assert_eq!(
            destination
                .view()
                .track(LayerId(2), &property)
                .unwrap()
                .unwrap()
                .keys()
                .len(),
            2
        );
        assert!(destination.undo());
        assert_eq!(
            destination
                .view()
                .track(LayerId(2), &property)
                .unwrap()
                .unwrap()
                .keys()
                .len(),
            1
        );
    }

    #[test]
    fn matching_multi_layer_numbers_in_another_document_are_not_a_mapping() {
        let source = document();
        let mut destination = document();
        let clipboard = Clipboard::default();
        clipboard.copy_keys(&source, &[key(1), key(2)]).unwrap();
        let history = destination.history_depth();
        assert!(clipboard
            .paste(&mut destination, Some(LayerId(2)), 30)
            .is_err());
        assert_eq!(destination.history_depth(), history);
        let property = PropertyId::new(property::OPACITY).unwrap();
        for id in [1, 2] {
            assert_eq!(
                destination
                    .view()
                    .track(LayerId(id), &property)
                    .unwrap()
                    .unwrap()
                    .keys()
                    .len(),
                1
            );
        }
    }
}
