//! 写して貼る(複製・ゴースト・クリップボード)。
#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
use crate::doc::store::{
    LayerAttrs, LayerAttrsPatch, LayerId, LayerSource, PropertyBase, Slot, SlotId, StoreError,
    StoreView,
};

fn attrs_to_patch(a: &LayerAttrs) -> LayerAttrsPatch {
    LayerAttrsPatch {
        flatten: Some(a.flatten),
        environment: Some(a.environment),
        blocks_light: Some(a.blocks_light),
        ghost: Some(a.ghost),
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
pub(crate) fn duplicate_layer(doc: &mut Document, layer: LayerId) -> Option<LayerId> {
    duplicate_layers(doc, &[layer]).ok()?.first().copied()
}

/// ゴーストの既定の遅れ(フレーム)。0 だと元と重なって見えないので、帯が 1 つ分ずれて見える量。
pub(crate) const GHOST_DEFAULT_DELAY: i64 = 6;

/// ゴーストを持てる層か。ゴーストは「同じ層を d だけ遅れて見た姿」なので、
/// 姿を持たない層(カメラ・Null・環境(HDR)・音声)には旨みがない(裁定 2026-09-07 利用者)。
pub(crate) fn ghostable(view: &StoreView, layer: LayerId) -> bool {
    let Ok(Some(meta)) = view.meta(layer) else { return false };
    let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    if attrs.environment {
        return false;
    }
    match &meta.source {
        LayerSource::Camera | LayerSource::Stage | LayerSource::Null => false,
        LayerSource::File { path, .. } => {
            !crate::render::media::is_environment_image_path(path)
                && !crate::render::media::is_audio_path(path)
        }
        _ => true,
    }
}

pub(crate) fn duplicate_layers(
    doc: &mut Document,
    layers: &[LayerId],
) -> Result<Vec<LayerId>, StoreError> {
    let plan = copy_plan(&doc, layers)?;
    doc.apply_all(plan.intents)?;
    Ok(plan.roots.into_iter().map(|(_, copy)| copy).collect())
}

pub(super) struct CopyPlan {
    pub(super) intents: Vec<Intent>,
    pub(super) roots: Vec<(LayerId, LayerId)>,
    pub(super) ids: std::collections::HashMap<LayerId, LayerId>,
}

#[derive(Clone, Debug)]
pub(crate) struct LayerClipboard {
    pub(crate) intents: Vec<Intent>,
    pub(crate) roots: Vec<LayerId>,
    pub(crate) copies: Vec<LayerId>,
    pub(crate) slots: Vec<Slot>,
}

pub(crate) fn copy_layers(
    doc: &Document,
    layers: &[LayerId],
) -> Result<LayerClipboard, StoreError> {
    let existing_slots: std::collections::HashSet<_> = doc
        .view()
        .slots()?
        .into_iter()
        .map(|slot| slot.id)
        .collect();
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

pub(crate) fn paste_layers(
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
        let next = remap_clipboard_intent(intent, &ids, &slot_ids, &root_set, &mut next_order)?;
        remapped.push(next);
    }
    doc.apply_all(remapped)?;
    Ok(clipboard
        .roots
        .iter()
        .filter_map(|root| ids.get(root).copied())
        .collect())
}

pub(crate) fn remap_clipboard_intent(
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

pub(super) fn copy_plan(doc: &Document, layers: &[LayerId]) -> Result<CopyPlan, StoreError> {
    let view = doc.view().without_transients();
    let mut sources = Vec::new();
    for &layer in layers {
        if let Some(reason) = crate::editor::functions::lens::edit_rejection(&view, layer)? {
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
    let mut pending = Vec::new();
    for &root in &roots {
        if view
            .meta(root)?
            .is_some_and(|meta| meta.source == crate::doc::store::LayerSource::Group)
        {
            pending.push(root);
        }
    }
    let mut expanded = std::collections::HashSet::new();
    while let Some(parent) = pending.pop() {
        if !expanded.insert(parent) {
            continue;
        }
        for &child in &present {
            if view.attrs(child)?.and_then(|attrs| attrs.parent) == Some(parent) {
                if !sources.contains(&child) {
                    sources.push(child);
                }
                pending.push(child);
            }
        }
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
                if let Some(reason) = crate::editor::functions::lens::edit_rejection(&view, layer)? {
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
