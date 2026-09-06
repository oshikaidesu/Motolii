//! 層をいじる手(複製・分割)。timeline_widget.rs が 1737 行の天井を越えたので分けた。
use crate::doc::store::{
    Document, Intent, KeyframeTrack, LayerAttrs, LayerAttrsPatch, LayerId, LayerTiming,
    PropertyBase, PropertyId, RationalTime, Slot, SlotId, StoreError, StoreView,
};
use crate::ui::functions::atom;
use crate::ui::timeline_widget::document_fps;
use std::sync::{Arc, Mutex};

pub(super) fn attrs_to_patch(a: &LayerAttrs) -> LayerAttrsPatch {
    LayerAttrsPatch {
        flatten: Some(a.flatten),
        hidden: Some(a.hidden),
        parent: Some(a.parent),
        blend_mode: Some(a.blend_mode.clone()),
        matte: Some(a.matte.clone()),
        clip_to_below: Some(a.clip_to_below),
        name: Some(a.name.clone()),
        auto_orient: Some(a.auto_orient),
        projection: Some(a.projection),
        solo: Some(a.solo),
        locked: Some(a.locked),
        label_color: Some(a.label_color),
    }
}

#[cfg(test)]
pub(super) fn duplicate_layer(doc: &Arc<Mutex<Document>>, layer: LayerId) -> Option<LayerId> {
    duplicate_layers(doc, &[layer]).ok()?.first().copied()
}

pub(super) fn duplicate_layers(
    doc: &Arc<Mutex<Document>>,
    layers: &[LayerId],
) -> Result<Vec<LayerId>, StoreError> {
    let mut doc = doc.lock().unwrap();
    let plan = copy_plan(&doc, layers)?;
    doc.apply_all(plan.intents)?;
    Ok(plan.roots.into_iter().map(|(_, copy)| copy).collect())
}

struct CopyPlan {
    intents: Vec<Intent>,
    roots: Vec<(LayerId, LayerId)>,
    ids: std::collections::HashMap<LayerId, LayerId>,
}

#[derive(Clone, Debug)]
pub(super) struct LayerClipboard {
    intents: Vec<Intent>,
    roots: Vec<LayerId>,
    copies: Vec<LayerId>,
    slots: Vec<Slot>,
}

pub(super) fn copy_layers(
    doc: &Document,
    layers: &[LayerId],
) -> Result<LayerClipboard, StoreError> {
    let existing_slots: std::collections::HashSet<_> =
        doc.view().slots()?.into_iter().map(|slot| slot.id).collect();
    let plan = copy_plan(doc, layers)?;
    let copies: Vec<_> = plan.ids.values().copied().collect();
    let copy_set: std::collections::HashSet<_> = copies.iter().copied().collect();
    let mut slots = Vec::new();
    let mut intents = Vec::new();
    for intent in plan.intents {
        match intent {
            Intent::SetSlots { slots: all } => {
                slots.extend(
                    all.into_iter()
                        .filter(|slot| !existing_slots.contains(&slot.id)),
                );
            }
            Intent::SetOrder { layer, .. } if !copy_set.contains(&layer) => {}
            other => intents.push(other),
        }
    }
    Ok(LayerClipboard {
        intents,
        roots: plan.roots.into_iter().map(|(_, copy)| copy).collect(),
        copies,
        slots,
    })
}

pub(super) fn paste_layers(
    doc: &mut Document,
    clipboard: &LayerClipboard,
) -> Result<Vec<LayerId>, StoreError> {
    if clipboard.copies.is_empty() {
        return Ok(Vec::new());
    }
    let view = doc.view().without_transients();
    let first = view.next_layer_id();
    let ids: std::collections::HashMap<_, _> = clipboard
        .copies
        .iter()
        .enumerate()
        .map(|(index, old)| {
            first
                .checked_add(index as u64)
                .map(|id| (*old, LayerId(id)))
                .ok_or_else(|| StoreError::Property("Layer identity space is full".into()))
        })
        .collect::<Result<_, _>>()?;
    let root_set: std::collections::HashSet<_> = clipboard.roots.iter().copied().collect();
    let mut next_order = view
        .layers()
        .into_iter()
        .filter_map(|layer| {
            if view
                .attrs(layer)
                .ok()
                .flatten()
                .and_then(|attrs| attrs.parent)
                .is_some()
            {
                return None;
            }
            view.meta(layer).ok().flatten().map(|meta| meta.order)
        })
        .max()
        .unwrap_or(-1)
        .saturating_add(1);

    let mut slot_ids = std::collections::HashMap::new();
    let mut slots = view.slots()?;
    for (index, slot) in clipboard.slots.iter().enumerate() {
        let id = SlotId(format!("clipboard.{first}.{index}"));
        slot_ids.insert(slot.id.clone(), id.clone());
        slots.push(Slot {
            id,
            track: slot.track.clone(),
        });
    }
    drop(view);

    let mut remapped = Vec::new();
    if !clipboard.slots.is_empty() {
        remapped.push(Intent::SetSlots { slots });
    }
    for intent in clipboard.intents.iter().cloned() {
        let next = remap_clipboard_intent(
            intent,
            &ids,
            &slot_ids,
            &root_set,
            &mut next_order,
        )?;
        remapped.push(next);
    }
    doc.apply_all(remapped)?;
    Ok(clipboard
        .roots
        .iter()
        .filter_map(|root| ids.get(root).copied())
        .collect())
}

fn remap_clipboard_intent(
    intent: Intent,
    layers: &std::collections::HashMap<LayerId, LayerId>,
    slots: &std::collections::HashMap<SlotId, SlotId>,
    roots: &std::collections::HashSet<LayerId>,
    next_order: &mut i16,
) -> Result<Intent, StoreError> {
    let layer = |old: LayerId| {
        layers
            .get(&old)
            .copied()
            .ok_or_else(|| StoreError::Property("Clipboard layer identity is incomplete".into()))
    };
    let links = |links: Vec<crate::doc::store::PropertyLink>| {
        links
            .into_iter()
            .filter_map(|mut link| {
                link.source_layer = layers.get(&link.source_layer).copied()?;
                Some(link)
            })
            .collect()
    };
    Ok(match intent {
        Intent::AddLayer(old) => Intent::AddLayer(layer(old)?),
        Intent::SetTrack {
            layer: old,
            property,
            track,
        } => Intent::SetTrack {
            layer: layer(old)?,
            property,
            track,
        },
        Intent::SetConstant {
            layer: old,
            property,
            value,
        } => Intent::SetConstant {
            layer: layer(old)?,
            property,
            value,
        },
        Intent::SetPropertySlot {
            layer: old,
            property,
            slot,
        } => Intent::SetPropertySlot {
            layer: layer(old)?,
            property,
            slot: slots.get(&slot).cloned().ok_or_else(|| {
                StoreError::Property("Clipboard shared value is incomplete".into())
            })?,
        },
        Intent::SetPropertyLink {
            layer: old,
            property,
            mut link,
        } => {
            link.source_layer = layer(link.source_layer)?;
            Intent::SetPropertyLink {
                layer: layer(old)?,
                property,
                link,
            }
        }
        Intent::SetPropertyModulators {
            layer: old,
            property,
            modulators,
        } => Intent::SetPropertyModulators {
            layer: layer(old)?,
            property,
            modulators: links(modulators),
        },
        Intent::SetMeta {
            layer: old,
            mut meta,
        } => {
            if roots.contains(&old) {
                meta.order = *next_order;
                *next_order = next_order.saturating_add(1);
            }
            Intent::SetMeta {
                layer: layer(old)?,
                meta,
            }
        }
        Intent::SetSource { layer: old, source } => Intent::SetSource {
            layer: layer(old)?,
            source,
        },
        Intent::SetOrder { layer: old, order } => Intent::SetOrder {
            layer: layer(old)?,
            order,
        },
        Intent::SetMasks { layer: old, masks } => Intent::SetMasks {
            layer: layer(old)?,
            masks,
        },
        Intent::AddMask {
            layer: old,
            mask,
            shape,
        } => Intent::AddMask {
            layer: layer(old)?,
            mask,
            shape,
        },
        Intent::SetTiming { layer: old, timing } => Intent::SetTiming {
            layer: layer(old)?,
            timing,
        },
        Intent::SetAttrs {
            layer: old,
            mut patch,
        } => {
            if let Some(parent) = patch.parent {
                patch.parent = Some(parent.and_then(|parent| layers.get(&parent).copied()));
            }
            if let Some(matte) = patch.matte {
                patch.matte = Some(matte.and_then(|mut matte| {
                    matte.layer = layers.get(&matte.layer).copied()?;
                    Some(matte)
                }));
            }
            Intent::SetAttrs {
                layer: layer(old)?,
                patch,
            }
        }
        Intent::SetEffects {
            layer: old,
            effects,
        } => Intent::SetEffects {
            layer: layer(old)?,
            effects,
        },
        Intent::SetShapes { layer: old, shapes } => Intent::SetShapes {
            layer: layer(old)?,
            shapes,
        },
        Intent::SetTextDocument {
            layer: old,
            document,
        } => Intent::SetTextDocument {
            layer: layer(old)?,
            document,
        },
        Intent::Freeze { group } => Intent::Freeze {
            group: layer(group)?,
        },
        Intent::Unfreeze { group } => Intent::Unfreeze {
            group: layer(group)?,
        },
        _ => {
            return Err(StoreError::Property(
                "Clipboard contains a non-layer operation".into(),
            ));
        }
    })
}

fn copy_plan(doc: &Document, layers: &[LayerId]) -> Result<CopyPlan, StoreError> {
    let view = doc.view().without_transients();
    let mut sources = Vec::new();
    for &layer in layers {
        if let Some(reason) = crate::ui::functions::lens::edit_rejection(&view, layer)? {
            return Err(StoreError::Property(format!(
                "Cannot copy layer {}: {reason}",
                layer.0
            )));
        } else if !sources.contains(&layer) {
            sources.push(layer);
        }
    }
    let selected = sources.clone();
    sources.retain(|layer| {
        let mut parent = view
            .attrs(*layer)
            .ok()
            .flatten()
            .and_then(|attrs| attrs.parent);
        let mut seen = std::collections::HashSet::new();
        while let Some(ancestor) = parent {
            if !seen.insert(ancestor) {
                break;
            }
            if selected.contains(&ancestor)
                && view
                    .meta(ancestor)
                    .ok()
                    .flatten()
                    .is_some_and(|meta| meta.source == crate::doc::store::LayerSource::Group)
            {
                return false;
            }
            parent = view
                .attrs(ancestor)
                .ok()
                .flatten()
                .and_then(|attrs| attrs.parent);
        }
        true
    });
    let roots = sources.clone();
    let present = view.layers();
    let mut at = 0;
    while at < sources.len() {
        let parent = sources[at];
        if view
            .meta(parent)?
            .is_some_and(|meta| meta.source == crate::doc::store::LayerSource::Group)
        {
            for &child in &present {
                if view.attrs(child)?.and_then(|attrs| attrs.parent) == Some(parent)
                    && !sources.contains(&child)
                {
                    sources.push(child);
                }
            }
        }
        at += 1;
    }
    let first_id = view.next_layer_id();
    let ids: std::collections::HashMap<_, _> = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            first_id
                .checked_add(index as u64)
                .map(|id| (*source, LayerId(id)))
                .ok_or_else(|| StoreError::Property("Layer identity space is full".into()))
        })
        .collect::<Result<_, _>>()?;
    let mut intents = Vec::new();
    let mut copy_orders = std::collections::HashMap::new();
    let mut scopes = Vec::new();
    for &root in &roots {
        let parent = view.attrs(root)?.and_then(|attrs| attrs.parent);
        if !scopes.contains(&parent) {
            scopes.push(parent);
        }
    }
    for parent in scopes {
        let mut siblings = Vec::new();
        for &layer in &present {
            if view.attrs(layer)?.and_then(|attrs| attrs.parent) == parent {
                if let Some(meta) = view.meta(layer)? {
                    siblings.push((meta.order, layer));
                }
            }
        }
        siblings.sort();
        let mut next = i32::from(i16::MIN);
        for (old, layer) in siblings {
            let order = i32::from(old).max(next);
            let order = i16::try_from(order)
                .map_err(|_| StoreError::Property("Layer order is full".into()))?;
            if old != order {
                if let Some(reason) = crate::ui::functions::lens::edit_rejection(&view, layer)? {
                    return Err(StoreError::Property(format!(
                        "Cannot insert a copy beside layer {}: {reason}",
                        layer.0
                    )));
                }
                intents.push(Intent::SetOrder { layer, order });
            }
            next = i32::from(order) + 1;
            if roots.contains(&layer) {
                let order = i16::try_from(next)
                    .map_err(|_| StoreError::Property("Layer order is full".into()))?;
                copy_orders.insert(layer, order);
                next += 1;
            }
        }
    }
    let mut slots = view.slots()?;
    let mut slot_copies = std::collections::HashMap::new();
    let mut terminal = Vec::new();
    for &source in &sources {
        let copy = ids[&source];
        let Some(mut meta) = view.meta(source)? else {
            continue;
        };
        if let Some(order) = copy_orders.get(&source) {
            meta.order = *order;
        }
        intents.push(Intent::AddLayer(copy));
        intents.push(Intent::SetMeta { layer: copy, meta });
        let mut attrs = view.attrs(source)?.unwrap_or_default();
        let locked = attrs.locked;
        attrs.locked = false;
        attrs.parent = attrs
            .parent
            .map(|layer| ids.get(&layer).copied().unwrap_or(layer));
        if let Some(matte) = &mut attrs.matte {
            matte.layer = ids.get(&matte.layer).copied().unwrap_or(matte.layer);
        }
        intents.push(Intent::SetAttrs {
            layer: copy,
            patch: attrs_to_patch(&attrs),
        });
        intents.extend(copy_content_intents(
            &view,
            source,
            copy,
            &ids,
            &mut slots,
            &mut slot_copies,
            true,
        )?);
        terminal.push((copy, attrs.frozen, locked));
    }
    if !slot_copies.is_empty() {
        intents.insert(0, Intent::SetSlots { slots });
    }
    for (copy, frozen, locked) in terminal.into_iter().rev() {
        if frozen {
            intents.push(Intent::Freeze { group: copy });
        }
        if locked {
            intents.push(Intent::SetAttrs {
                layer: copy,
                patch: LayerAttrsPatch {
                    locked: Some(true),
                    ..Default::default()
                },
            });
        }
    }
    Ok(CopyPlan {
        intents,
        roots: roots
            .into_iter()
            .map(|source| (source, ids[&source]))
            .collect(),
        ids,
    })
}

fn copy_content_intents(
    view: &StoreView<'_>,
    source: LayerId,
    copy: LayerId,
    ids: &std::collections::HashMap<LayerId, LayerId>,
    slots: &mut Vec<Slot>,
    slot_copies: &mut std::collections::HashMap<SlotId, SlotId>,
    detach_slots: bool,
) -> Result<Vec<Intent>, StoreError> {
    let mut intents = Vec::new();
    let effects = view.effects(source)?;
    if !effects.is_empty() {
        intents.push(Intent::SetEffects {
            layer: copy,
            effects,
        });
    }
    let shapes = view.shapes(source)?;
    if !shapes.is_empty() {
        intents.push(Intent::SetShapes {
            layer: copy,
            shapes,
        });
    }
    if let Some(document) = view.text_document(source)? {
        intents.push(Intent::SetTextDocument {
            layer: copy,
            document,
        });
    }
    for property in view.properties(source) {
        let Some(mut value) = view.property_source(source, &property)? else {
            continue;
        };
        match value.base {
            Some(PropertyBase::Track(track)) => intents.push(Intent::SetTrack {
                layer: copy,
                property: property.clone(),
                track,
            }),
            Some(PropertyBase::Constant(value)) => intents.push(Intent::SetConstant {
                layer: copy,
                property: property.clone(),
                value,
            }),
            Some(PropertyBase::Slot(slot)) => {
                let new_slot = if !detach_slots {
                    slot.clone()
                } else if let Some(id) = slot_copies.get(&slot) {
                    id.clone()
                } else {
                    let track = slots
                        .iter()
                        .find(|entry| entry.id == slot)
                        .map(|entry| entry.track.clone())
                        .ok_or_else(|| {
                            StoreError::Property(format!("Missing shared value {slot}"))
                        })?;
                    let mut suffix = 0;
                    let id = loop {
                        let id = SlotId(format!("{}.copy.{}.{}", slot.0, copy.0, suffix));
                        if !slots.iter().any(|entry| entry.id == id) {
                            break id;
                        }
                        suffix += 1;
                    };
                    slots.push(Slot {
                        id: id.clone(),
                        track,
                    });
                    slot_copies.insert(slot, id.clone());
                    id
                };
                intents.push(Intent::SetPropertySlot {
                    layer: copy,
                    property: property.clone(),
                    slot: new_slot,
                });
            }
            None => {}
        }
        for link in &mut value.modulators {
            link.source_layer = ids
                .get(&link.source_layer)
                .copied()
                .unwrap_or(link.source_layer);
        }
        if !value.modulators.is_empty() {
            intents.push(Intent::SetPropertyModulators {
                layer: copy,
                property,
                modulators: value.modulators,
            });
        }
    }
    let masks = view.masks(source)?;
    if !masks.is_empty() {
        intents.push(Intent::SetMasks { layer: copy, masks });
    }
    Ok(intents)
}

pub(super) fn split_layers(
    doc: &Arc<Mutex<Document>>,
    layers: &[LayerId],
    comp_frame: i64,
) -> Result<Vec<LayerId>, StoreError> {
    let mut doc = doc.lock().unwrap();
    let mut eligible = Vec::new();
    for &layer in layers {
        let view = doc.view().without_transients();
        if let Some(reason) = crate::ui::functions::lens::edit_rejection(&view, layer)? {
            println!(
                "PROBE room=write verdict=split-skip layer={} reason={reason}",
                layer.0
            );
            continue;
        }
        let covers_cut = match view.meta(layer)? {
            Some(meta) => {
                let span = atom::frame_span(meta.timing.start, meta.timing.duration)
                    .map_err(|reason| StoreError::Property(reason.into()))?;
                span.start < comp_frame && span.contains(&comp_frame)
            }
            None => false,
        };
        if covers_cut {
            if !eligible.contains(&layer) {
                eligible.push(layer);
            }
        } else {
            println!(
                "PROBE room=write verdict=split-skip layer={} reason=playhead-outside-layer",
                layer.0
            );
        }
    }
    let mut plan = copy_plan(&doc, &eligible)?;
    for &(source, tail) in &plan.roots {
        plan.intents
            .extend(split_intents(&doc, source, tail, comp_frame, &plan.ids)?);
    }
    doc.apply_all(plan.intents)?;
    Ok(plan.roots.into_iter().map(|(_, tail)| tail).collect())
}

fn split_intents(
    doc: &Document,
    layer: LayerId,
    tail: LayerId,
    comp_frame: i64,
    ids: &std::collections::HashMap<LayerId, LayerId>,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view().without_transients();
    let Some(meta) = view.meta(layer)? else {
        return Ok(Vec::new());
    };
    let span = atom::frame_span(meta.timing.start, meta.timing.duration)
        .map_err(|reason| StoreError::Property(reason.into()))?;
    if !span.contains(&comp_frame) || comp_frame == span.start {
        return Ok(Vec::new());
    }
    let (head, tail_span) =
        atom::split_span(span, comp_frame).map_err(|reason| StoreError::Property(reason.into()))?;
    let head_timing = LayerTiming {
        duration: head.end - head.start,
        ..meta.timing
    };
    let tail_timing = LayerTiming {
        start: tail_span.start,
        duration: tail_span.end - tail_span.start,
        source_in: meta
            .timing
            .source_frame(comp_frame)
            .ok_or_else(|| StoreError::Property("Split is outside the source".into()))?,
        ..meta.timing
    };

    let text = view.text_document(layer)?;
    let fps = document_fps(doc)?;
    let cut = RationalTime::try_from_frame(comp_frame, fps)
        .map_err(|error| StoreError::Property(error.to_string()))?;
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();

    let mut intents = vec![
        Intent::SetTiming {
            layer,
            timing: head_timing,
        },
        Intent::SetTiming {
            layer: tail,
            timing: tail_timing,
        },
    ];
    if let Some(document) = text {
        // 歌詞の切替も切り口で分ける。頭は切り口より前、尻は切り口の行から(両方に全部を配らない)。
        let keys = document.content.keys().to_vec();
        if keys.len() > 1 {
            let at_cut = keys
                .iter()
                .filter(|k| k.t <= cut)
                .last()
                .or(keys.first())
                .cloned();
            let mut head_track = crate::doc::store::ContentTrack::new();
            let mut tail_track = crate::doc::store::ContentTrack::new();
            for k in &keys {
                if k.t < cut {
                    head_track.insert(k.clone());
                } else {
                    tail_track.insert(k.clone());
                }
            }
            if let Some(k) = at_cut {
                if head_track.keys().is_empty() {
                    head_track.insert(k.clone());
                }
                tail_track.insert(crate::doc::store::ContentKeyframe {
                    t: cut,
                    content: k.content.clone(),
                });
            }
            let mut head_doc = document.clone();
            head_doc.content = head_track;
            intents.push(Intent::SetTextDocument {
                layer,
                document: head_doc,
            });
            let mut tail_doc = document.clone();
            tail_doc.content = tail_track;
            intents.push(Intent::SetTextDocument {
                layer: tail,
                document: tail_doc,
            });
        } else {
            intents.push(Intent::SetTextDocument {
                layer: tail,
                document,
            });
        }
    }
    for (property, track) in tracks {
        let (head, tail_track) = split_track_at(&track, cut);
        if let Some(head) = head {
            intents.push(Intent::SetTrack {
                layer,
                property: property.clone(),
                track: head,
            });
        }
        intents.push(Intent::SetTrack {
            layer: tail,
            property: property.clone(),
            track: tail_track,
        });
        if let Some(source) = view.property_source(layer, &property)? {
            if !source.modulators.is_empty() {
                intents.push(Intent::SetPropertyModulators {
                    layer,
                    property: property.clone(),
                    modulators: source.modulators.clone(),
                });
                let modulators = source
                    .modulators
                    .into_iter()
                    .map(|mut link| {
                        link.source_layer = ids
                            .get(&link.source_layer)
                            .copied()
                            .unwrap_or(link.source_layer);
                        link
                    })
                    .collect();
                intents.push(Intent::SetPropertyModulators {
                    layer: tail,
                    property,
                    modulators,
                });
            }
        }
    }

    Ok(intents)
}

/// 切り口を跨ぐ区間のイージングを両側へ分ける(Premiere・Resolve は切っても見た目が変わらない)。
/// 跨ぐ区間が無ければ頭はそのまま(None)、尻は丸ごと写す。
pub(super) fn split_track_at(
    track: &KeyframeTrack,
    cut: RationalTime,
) -> (Option<KeyframeTrack>, KeyframeTrack) {
    let keys = track.keys();
    let straddle = keys
        .windows(2)
        .position(|pair| pair[0].t < cut && cut < pair[1].t);
    let Some(i) = straddle else {
        return (None, track.clone());
    };
    let (a, b) = (&keys[i], &keys[i + 1]);
    let progress = (cut.as_seconds_f64() - a.t.as_seconds_f64())
        / (b.t.as_seconds_f64() - a.t.as_seconds_f64()).max(f64::EPSILON);
    let Ok((first, second)) = a.interp.split_at(progress) else {
        return (None, track.clone());
    };
    let at_cut = crate::doc::store::Keyframe {
        t: cut,
        value: track.eval(cut),
        interp: second,
        spatial: None,
    };
    let mut head = KeyframeTrack::new();
    let mut tail = KeyframeTrack::new();
    for (k, key) in keys.iter().enumerate() {
        let mut key = key.clone();
        if k == i {
            key.interp = first;
        }
        if k <= i {
            head.insert(key.clone());
        }
        if k > i {
            tail.insert(key);
        }
    }
    head.insert(at_cut.clone());
    tail.insert(at_cut);
    (Some(head), tail)
}

pub(super) fn delete_key_selection_intents(
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
        let properties = only
            .clone()
            .map(|property| vec![property])
            .unwrap_or_else(|| view.properties(*layer));
        for property in properties {
            tracks.entry((*layer, property)).or_default().insert(frame);
        }
        if only.is_none() {
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
        crate::ui::functions::lens::require_local_source(&view, layer, &property)?;
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
