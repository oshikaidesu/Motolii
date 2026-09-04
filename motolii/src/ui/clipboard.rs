use std::sync::{Arc, Mutex};

use crate::doc::store::{
    Document, Intent, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime, StoreError,
};
use crate::ui::session::KeySel;
use crate::ui::timeline_edit::{copy_layers, paste_layers, LayerClipboard};

/// Typed, process-local editor clipboard. The payload owns a layer snapshot, so
/// Cut followed by Paste and pasting after the source project changes are valid.
#[derive(Clone)]
struct CopiedKey {
    layer: LayerId,
    property: PropertyId,
    offset_frames: i64,
    key: Keyframe,
}

#[derive(Clone)]
enum Payload {
    Layers(LayerClipboard),
    Keys(Vec<CopiedKey>),
}

pub(super) enum PasteResult {
    Layers(Vec<LayerId>),
    Keys(Vec<KeySel>),
}

#[derive(Clone, Default)]
pub(super) struct Clipboard(Arc<Mutex<Option<Payload>>>);

impl Clipboard {
    pub(super) fn copy_layers(
        &self,
        doc: &Document,
        layers: &[LayerId],
    ) -> Result<usize, StoreError> {
        let payload = copy_layers(doc, layers)?;
        let count = layers.len();
        *self.0.lock().unwrap() = Some(Payload::Layers(payload));
        Ok(count)
    }

    pub(super) fn copy_keys(
        &self,
        doc: &Document,
        selected: &[KeySel],
    ) -> Result<usize, StoreError> {
        let fps = crate::ui::timeline_widget::document_fps(doc)?;
        let view = doc.view().without_transients();
        let mut copied = Vec::new();
        for selected in selected {
            let properties: Vec<_> = match &selected.property {
                Some(property) => vec![property.clone()],
                None => view.properties(selected.layer),
            };
            let frame = (selected.at_sec * fps.as_f64()).round() as i64;
            for property in properties {
                let Some(track) = view.track(selected.layer, &property)? else {
                    continue;
                };
                let Some(key) = track.keys().iter().find(|key| {
                    key.t.try_to_frame_round(fps).ok() == Some(frame)
                }) else {
                    continue;
                };
                if copied.iter().any(|known: &CopiedKey| {
                    known.layer == selected.layer
                        && known.property == property
                        && known.key.t == key.t
                }) {
                    continue;
                }
                copied.push(CopiedKey {
                    layer: selected.layer,
                    property,
                    offset_frames: frame,
                    key: key.clone(),
                });
            }
        }
        let Some(first) = copied.iter().map(|key| key.offset_frames).min() else {
            return Err(StoreError::Property("Select keyframes to copy".into()));
        };
        for key in &mut copied {
            key.offset_frames -= first;
        }
        let count = copied.len();
        *self.0.lock().unwrap() = Some(Payload::Keys(copied));
        Ok(count)
    }

    pub(super) fn paste(
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
            Payload::Keys(keys) => paste_keys(doc, &keys, fallback_layer, at_frame)
                .map(PasteResult::Keys),
        }
    }

    pub(super) fn has_payload(&self) -> bool {
        self.0.lock().unwrap().is_some()
    }
}

fn paste_keys(
    doc: &mut Document,
    copied: &[CopiedKey],
    fallback_layer: Option<LayerId>,
    at_frame: i64,
) -> Result<Vec<KeySel>, StoreError> {
    let fps = crate::ui::timeline_widget::document_fps(doc)?;
    let view = doc.view().without_transients();
    let live = view.layers();
    let mut tracks: Vec<(LayerId, PropertyId, KeyframeTrack)> = Vec::new();
    let mut selected = Vec::new();
    for copied in copied {
        let layer = if live.contains(&copied.layer) {
            copied.layer
        } else {
            fallback_layer.ok_or_else(|| {
                StoreError::Property("Select a destination layer for these keyframes".into())
            })?
        };
        let index = tracks
            .iter()
            .position(|(known_layer, property, _)| {
                *known_layer == layer && *property == copied.property
            });
        let index = if let Some(index) = index {
            index
        } else {
            tracks.push((
                layer,
                copied.property.clone(),
                view.track(layer, &copied.property)?.unwrap_or_default(),
            ));
            tracks.len() - 1
        };
        let at = RationalTime::try_from_frame(at_frame + copied.offset_frames, fps)
            .map_err(|error| StoreError::Property(error.to_string()))?;
        let mut key = copied.key.clone();
        key.t = at;
        tracks[index].2.insert(key);
        selected.push(KeySel {
            layer,
            property: Some(copied.property.clone()),
            at_sec: at.as_seconds_f64(),
        });
    }
    drop(view);
    doc.apply_all(tracks.into_iter().map(|(layer, property, track)| {
        Intent::SetTrack {
            layer,
            property,
            track,
        }
    }))?;
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
}
