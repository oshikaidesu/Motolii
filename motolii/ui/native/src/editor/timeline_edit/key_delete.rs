//! 選んだキーを消す(欄の選び方と、消す手)。
#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::doc::store::{KeyframeTrack, LayerId, PropertyId, RationalTime, StoreError, StoreView};
use crate::editor::keyframe_edit::document_fps;

pub(crate) fn selected_properties(
    view: &StoreView<'_>,
    layer: LayerId,
    only: Option<&PropertyId>,
) -> Vec<PropertyId> {
    match only {
        Some(property) if property.name() == "content" => Vec::new(),
        Some(property) => vec![property.clone()],
        None => view.properties(layer),
    }
}

pub(crate) fn selects_content(only: Option<&PropertyId>) -> bool {
    only.is_none_or(|property| property.name() == "content")
}

pub(crate) fn delete_key_selection_intents(
    doc: &Document,
    selections: &[(LayerId, Option<PropertyId>, f64)],
    fallback_at: RationalTime,
) -> Result<Vec<Intent>, StoreError> {
    use std::collections::{BTreeMap, BTreeSet};
    let view = doc.view().without_transients();
    let fps = document_fps(doc)?;
    let mut tracks: BTreeMap<(LayerId, PropertyId), BTreeSet<i64>> = BTreeMap::new();
    let mut contents: BTreeMap<LayerId, BTreeSet<i64>> = BTreeMap::new();
    for (layer, only, seconds) in selections {
        let frame = (seconds * fps.as_f64()).round();
        if !frame.is_finite() || frame < i64::MIN as f64 || frame >= -(i64::MIN as f64) {
            return Err(StoreError::Property(
                "Selected keyframe time is outside the frame domain".into(),
            ));
        }
        let frame = frame as i64;
        let properties = selected_properties(&view, *layer, only.as_ref());
        for property in properties {
            tracks.entry((*layer, property)).or_default().insert(frame);
        }
        if selects_content(only.as_ref()) {
            contents.entry(*layer).or_default().insert(frame);
        }
    }
    let mut intents = Vec::new();
    for ((layer, property), frames) in tracks {
        let Some(track) = view.track(layer, &property)? else {
            continue;
        };
        let mut kept = KeyframeTrack::new();
        let mut removed = false;
        for key in track.keys() {
            let frame = key
                .t
                .try_to_frame_round(fps)
                .map_err(|error| StoreError::Property(error.to_string()))?;
            if frames.contains(&frame) {
                removed = true;
            } else {
                kept.insert(key.clone());
            }
        }
        if !removed {
            continue;
        }
        crate::editor::functions::lens::require_local_source(&view, layer, &property)?;
        if kept.keys().is_empty() {
            intents.push(Intent::SetConstant {
                layer,
                property,
                value: track.eval(fallback_at),
            });
        } else {
            intents.push(Intent::SetTrack {
                layer,
                property,
                track: kept,
            });
        }
    }
    for (layer, frames) in contents {
        let Some(mut document) = view.text_document(layer)? else {
            continue;
        };
        let original = document.content.clone();
        let mut kept = crate::doc::store::ContentTrack::new();
        for key in original.keys() {
            let frame = key
                .t
                .try_to_frame_round(fps)
                .map_err(|error| StoreError::Property(error.to_string()))?;
            if !frames.contains(&frame) {
                kept.insert(key.clone());
            }
        }
        if kept.keys().is_empty() {
            if let Some(current) = original
                .keys()
                .iter()
                .rev()
                .find(|key| key.t <= fallback_at)
                .or_else(|| original.keys().first())
            {
                kept.insert(current.clone());
            }
        }
        if kept != original {
            document.content = kept;
            intents.push(Intent::SetTextDocument { layer, document });
        }
    }
    Ok(intents)
}
