#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use std::sync::{Arc, Mutex};

use crate::doc::store::{
    ContentKeyframe, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime,
    StoreError, TextDocument,
};
use crate::viewer::KeySel;
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
mod tests;

#[cfg(test)]
mod key_ownership_regressions;

#[cfg(test)]
mod document_identity_regressions;
