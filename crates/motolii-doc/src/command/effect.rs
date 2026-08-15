use crate::duplicate::{definition_semantic_body_eq, remint_order_keyframe_ids};
use crate::schema::{EffectDefinition, EffectUse, TrackItem};
use crate::stable_id::{EffectDefinitionId, EffectId, StableIdReservation};
use crate::validate::{self, stable_id_in_use};
use crate::{Document, LayerId, WRITER_VERSION};

use super::locate::{envelope_of, find_envelope, find_envelope_mut};
use super::reservation::{
    apply_reservation_commit, swap_if_valid, validate_reservation_for_apply,
    validate_reservation_for_undo,
};
use super::CommandError;

pub(crate) fn guard_effect_lifecycle_document(doc: &Document) -> Result<(), CommandError> {
    let required = validate::MIN_READER_VERSION_FOR_COMP_CAMERA;
    if doc.version != WRITER_VERSION || doc.min_reader_version != required {
        return Err(CommandError::EffectLifecycleRequiresV4Document {
            version: doc.version,
            min_reader_version: doc.min_reader_version,
        });
    }
    doc.validate().map_err(CommandError::Validate)?;
    Ok(())
}

pub(crate) fn introduced_ids_create(use_: &EffectUse, definition: &EffectDefinition) -> Vec<u64> {
    let mut ids = vec![use_.id.get(), definition.id.get()];
    ids.extend(
        remint_order_keyframe_ids(definition)
            .into_iter()
            .map(|id| id.get()),
    );
    ids
}

pub(crate) fn introduced_ids_link(use_: &EffectUse) -> Vec<u64> {
    vec![use_.id.get()]
}

pub(crate) fn introduced_ids_copy_local(new_definition: &EffectDefinition) -> Vec<u64> {
    let mut ids = vec![new_definition.id.get()];
    ids.extend(
        remint_order_keyframe_ids(new_definition)
            .into_iter()
            .map(|id| id.get()),
    );
    ids
}

pub(crate) fn find_use_location(doc: &Document, use_id: EffectId) -> Option<(LayerId, usize)> {
    fn walk(items: &[TrackItem], use_id: EffectId) -> Option<(LayerId, usize)> {
        for item in items {
            let env = envelope_of(item);
            if let Some(index) = env.effects.iter().position(|u| u.id == use_id) {
                return Some((env.layer_id, index));
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, use_id) {
                    return Some(found);
                }
            }
        }
        None
    }
    doc.tracks
        .iter()
        .find_map(|track| walk(&track.items, use_id))
}

pub(super) fn apply_create_effect(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    if use_.definition_id != definition.id {
        return Err(CommandError::EffectDefinitionMismatch {
            id: definition.id.get(),
        });
    }
    let introduced = introduced_ids_create(&use_, &definition);
    let commit = validate_reservation_for_apply(doc, reservation, &introduced)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    if doc.effect_definition(definition.id).is_some() {
        return Err(CommandError::EffectDefinitionAlreadyExists {
            id: definition.id.get(),
        });
    }

    let mut next = doc.clone();
    {
        let env = find_envelope_mut(&mut next, target)?;
        env.effects.insert(index, use_);
    }
    next.effect_definitions.push(definition);
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

pub(super) fn apply_undo_create_effect(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_create(&use_, &definition);
    validate_reservation_for_undo(doc, reservation, &introduced)?;
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = &env.effects[index];
    if at_index.id != use_.id || at_index.definition_id != use_.definition_id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: use_.id.get(),
            found: at_index.id.get(),
        });
    }
    let ledger_def =
        doc.effect_definition(definition.id)
            .ok_or(CommandError::EffectDefinitionNotFound {
                id: definition.id.get(),
            })?;
    if ledger_def != &definition {
        return Err(CommandError::RemoveEffectDefinitionMismatch {
            id: definition.id.get(),
        });
    }
    let remaining = doc.effect_use_count(definition.id);
    if remaining != 1 {
        return Err(CommandError::DefinitionInUse {
            id: definition.id.get(),
            use_ids: doc
                .effect_use_ids(definition.id)
                .into_iter()
                .map(|id| id.get())
                .collect(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.effects.remove(index);
    next.effect_definitions.retain(|d| d.id != definition.id);
    swap_if_valid(doc, next)
}

pub(super) fn apply_link_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_link(&use_);
    let commit = validate_reservation_for_apply(doc, reservation, &introduced)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let existing = doc.effect_definition(use_.definition_id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        },
    )?;

    let mut next = doc.clone();
    let _ = existing;
    find_envelope_mut(&mut next, target)?
        .effects
        .insert(index, use_);
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

pub(super) fn apply_undo_link_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_link(&use_);
    validate_reservation_for_undo(doc, reservation, &introduced)?;
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = &env.effects[index];
    if at_index.id != use_.id || at_index.definition_id != use_.definition_id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: use_.id.get(),
            found: at_index.id.get(),
        });
    }
    if doc.effect_definition(use_.definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.effects.remove(index);
    swap_if_valid(doc, next)
}

pub(super) fn apply_unlink_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = &env.effects[index];
    if at_index.id != use_.id || at_index.definition_id != use_.definition_id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: use_.id.get(),
            found: at_index.id.get(),
        });
    }
    if doc.effect_definition(use_.definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?.effects.remove(index);
    swap_if_valid(doc, next)
}

pub(super) fn apply_restore_effect_use(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    use_: EffectUse,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    if stable_id_in_use(doc, use_.id.get()) {
        return Err(CommandError::StableIdCollision { id: use_.id.get() });
    }
    if doc.effect_definition(use_.definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: use_.definition_id.get(),
        });
    }

    let mut next = doc.clone();
    find_envelope_mut(&mut next, target)?
        .effects
        .insert(index, use_);
    swap_if_valid(doc, next)
}

pub(super) fn apply_copy_local_effect(
    doc: &mut Document,
    use_id: EffectId,
    previous_definition_id: EffectDefinitionId,
    new_definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_copy_local(&new_definition);
    let commit = validate_reservation_for_apply(doc, reservation, &introduced)?;
    let (target, index) =
        find_use_location(doc, use_id).ok_or(CommandError::EffectUseNotFound {
            use_id: use_id.get(),
        })?;
    {
        let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
        let use_ = &env.effects[index];
        if use_.id != use_id {
            return Err(CommandError::EffectUseNotFound {
                use_id: use_id.get(),
            });
        }
        if use_.definition_id != previous_definition_id {
            return Err(CommandError::CopyLocalDefinitionMismatch {
                expected: previous_definition_id.get(),
                found: use_.definition_id.get(),
            });
        }
    }
    let source = doc.effect_definition(previous_definition_id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: previous_definition_id.get(),
        },
    )?;
    if !definition_semantic_body_eq(source, &new_definition) {
        return Err(CommandError::CopyLocalPayloadMismatch);
    }
    if doc.effect_definition(new_definition.id).is_some() {
        return Err(CommandError::EffectDefinitionAlreadyExists {
            id: new_definition.id.get(),
        });
    }

    let mut next = doc.clone();
    {
        let env = find_envelope_mut(&mut next, target)?;
        env.effects[index].definition_id = new_definition.id;
    }
    next.effect_definitions.push(new_definition);
    apply_reservation_commit(&mut next, commit);
    swap_if_valid(doc, next)
}

pub(super) fn apply_undo_copy_local_effect(
    doc: &mut Document,
    use_id: EffectId,
    previous_definition_id: EffectDefinitionId,
    new_definition: EffectDefinition,
    reservation: StableIdReservation,
) -> Result<(), CommandError> {
    guard_effect_lifecycle_document(doc)?;
    let introduced = introduced_ids_copy_local(&new_definition);
    validate_reservation_for_undo(doc, reservation, &introduced)?;
    let (target, index) =
        find_use_location(doc, use_id).ok_or(CommandError::EffectUseNotFound {
            use_id: use_id.get(),
        })?;
    {
        let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
        let use_ = &env.effects[index];
        if use_.definition_id != new_definition.id {
            return Err(CommandError::CopyLocalDefinitionMismatch {
                expected: new_definition.id.get(),
                found: use_.definition_id.get(),
            });
        }
    }
    if doc.effect_definition(previous_definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: previous_definition_id.get(),
        });
    }
    let ledger =
        doc.effect_definition(new_definition.id)
            .ok_or(CommandError::EffectDefinitionNotFound {
                id: new_definition.id.get(),
            })?;
    if ledger != &new_definition {
        return Err(CommandError::EffectDefinitionMismatch {
            id: new_definition.id.get(),
        });
    }
    let shared_use_ids: Vec<u64> = doc
        .effect_use_ids(new_definition.id)
        .into_iter()
        .map(|id| id.get())
        .filter(|id| *id != use_id.get())
        .collect();
    if !shared_use_ids.is_empty() {
        return Err(CommandError::UndoCopyLocalDefinitionInUse {
            id: new_definition.id.get(),
            use_ids: shared_use_ids,
        });
    }

    let mut next = doc.clone();
    {
        let env = find_envelope_mut(&mut next, target)?;
        env.effects[index].definition_id = previous_definition_id;
    }
    next.effect_definitions
        .retain(|d| d.id != new_definition.id);
    swap_if_valid(doc, next)
}
