use crate::doc::store::*;
pub(crate) fn document_fps(doc: &Document) -> Result<Fps, StoreError> {
    doc.view()
        .composition()?
        .map(|composition| composition.fps)
        .ok_or_else(|| StoreError::Property("Composition has no frame rate".to_owned()))
}
pub(crate) fn key_selection_move_intents(
    doc: &Document,
    keys: &[(LayerId, Option<crate::doc::store::PropertyId>, f64)],
    raw_delta: i64,
) -> Result<Vec<Intent>, StoreError> {
    if raw_delta == 0 {
        return Ok(Vec::new());
    }
    let fps = document_fps(doc)?.as_f64();
    let mut tracks: std::collections::BTreeMap<(LayerId, crate::doc::store::PropertyId), Vec<i64>> =
        std::collections::BTreeMap::new();
    let mut content: std::collections::BTreeMap<LayerId, Vec<i64>> =
        std::collections::BTreeMap::new();
    for (layer, only, at) in keys.iter().cloned() {
        let frame = (at * fps).round() as i64;
        let properties = crate::editor::timeline_edit::selected_properties(
            &doc.view().without_transients(),
            layer,
            only.as_ref(),
        );
        for property in properties {
            tracks.entry((layer, property)).or_default().push(frame);
        }
        if crate::editor::timeline_edit::selects_content(only.as_ref()) {
            content.entry(layer).or_default().push(frame);
        }
    }
    let mut intents = Vec::new();
    for ((layer, property), mut frames) in tracks {
        frames.sort();
        frames.dedup();
        crate::editor::functions::lens::require_local_source(
            &doc.view().without_transients(),
            layer,
            &property,
        )?;
        intents.extend(keyframe_move_intents(
            doc,
            layer,
            Some(&property),
            &frames,
            raw_delta,
        )?);
    }
    for (layer, mut frames) in content {
        frames.sort();
        frames.dedup();
        let fps = document_fps(doc)?;
        let shift = RationalTime::try_from_frame(raw_delta, fps)
            .map_err(|error| StoreError::Property(error.to_string()))?;
        if let Some(intent) = content_track_intent(
            &doc.view().without_transients(),
            layer,
            fps,
            &frames,
            |at| at.try_add(shift).ok(),
        )? {
            intents.push(intent);
        }
    }
    Ok(intents)
}
pub(crate) fn keyframe_move_intents(
    doc: &Document,
    layer: LayerId,
    only: Option<&crate::doc::store::PropertyId>,
    frames: &[i64],
    delta_frames: i64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view().without_transients();
    let fps = document_fps(doc)?;
    let shift = RationalTime::try_from_frame(delta_frames, fps)
        .map_err(|e| StoreError::Property(e.to_string()))?;
    let mut intents = Vec::new();
    for property in view.properties(layer) {
        if only.is_some_and(|p| *p != property) {
            continue;
        }
        let Some(track) = view.track(layer, &property)? else {
            continue;
        };
        let mut touched = false;
        let mut moved = KeyframeTrack::new();
        let mut selected = Vec::new();
        for key in track.keys() {
            let mut key = key.clone();
            let frame = key
                .t
                .try_to_frame_round(fps)
                .map_err(|e| StoreError::Property(e.to_string()))?;
            if frames.contains(&frame) {
                key.t = key
                    .t
                    .try_add(shift)
                    .map_err(|e| StoreError::Property(e.to_string()))?;
                touched = true;
                selected.push(key);
            } else {
                moved.insert(key);
            }
        }
        for key in selected {
            moved.insert(key);
        }
        if touched {
            intents.push(Intent::SetTrack {
                layer,
                property,
                track: moved,
            });
        }
    }
    if crate::editor::timeline_edit::selects_content(only) {
        if let Some(intent) =
            content_track_intent(&view, layer, fps, frames, |t| t.try_add(shift).ok())?
        {
            intents.push(intent);
        }
    }
    Ok(intents)
}
fn content_track_intent(
    view: &StoreView<'_>,
    layer: LayerId,
    fps: Fps,
    at_frames: &[i64],
    map: impl Fn(RationalTime) -> Option<RationalTime>,
) -> Result<Option<Intent>, StoreError> {
    let Some(mut text) = view.text_document(layer)? else {
        return Ok(None);
    };
    if text.content.keys().is_empty() {
        return Ok(None);
    }
    let mut next = crate::doc::store::ContentTrack::new();
    let mut selected = Vec::new();
    let mut touched = false;
    for key in text.content.keys() {
        let frame = key
            .t
            .try_to_frame_round(fps)
            .map_err(|e| StoreError::Property(e.to_string()))?;
        if at_frames.contains(&frame) {
            touched = true;
            match map(key.t) {
                Some(t) => selected.push(crate::doc::store::ContentKeyframe {
                    t,
                    content: key.content.clone(),
                }),
                None => continue,
            }
        } else {
            next.insert(key.clone());
        }
    }
    for key in selected {
        next.insert(key);
    }
    if !touched || next.keys().is_empty() {
        return Ok(None);
    }
    text.content = next;
    Ok(Some(Intent::SetTextDocument {
        layer,
        document: text,
    }))
}
