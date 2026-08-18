use crate::schema::EffectDefinition;
use crate::validate::stable_id_in_use;
use crate::{Document, LayerId};

use super::locate::{find_envelope, find_envelope_mut};
use super::CommandError;
use crate::schema::EffectInstance;
use crate::stable_id::EffectId;

pub(super) fn apply_add_effect(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    effect: &EffectInstance,
    introduced_definition: bool,
) -> Result<(), CommandError> {
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let (use_, def) = effect.clone().into_use_and_definition();
    if stable_id_in_use(doc, use_.id.get()) {
        return Err(CommandError::StableIdCollision { id: use_.id.get() });
    }
    match doc.effect_definition(def.id) {
        Some(existing) if existing == &def => {
            if introduced_definition {
                return Err(CommandError::EffectDefinitionAlreadyExists {
                    id: def.id.get(),
                });
            }
        }
        Some(_) => {
            return Err(CommandError::EffectDefinitionMismatch { id: def.id.get() })
        }
        None => {
            if !introduced_definition {
                return Err(CommandError::EffectDefinitionNotFound {
                    id: def.id.get(),
                });
            }
            if stable_id_in_use(doc, def.id.get()) {
                return Err(CommandError::StableIdCollision { id: def.id.get() });
            }
            doc.effect_definitions.push(def);
        }
    }
    find_envelope_mut(doc, target)?.effects.insert(index, use_);
    Ok(())
}

pub(super) fn apply_remove_effect(
    doc: &mut Document,
    target: LayerId,
    index: usize,
    effect: &EffectInstance,
    introduced_definition: bool,
) -> Result<(), CommandError> {
    let layer = target.get();
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(layer))?;
    if index >= env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        });
    }
    let at_index = env.effects[index].clone();
    if at_index.id != effect.id {
        return Err(CommandError::RemoveEffectMismatch {
            expected: effect.id.get(),
            found: at_index.id.get(),
        });
    }
    if at_index.definition_id != effect.definition_id {
        return Err(CommandError::RemoveEffectDefinitionMismatch {
            id: effect.definition_id.get(),
        });
    }
    let (_, expected_def) = effect.clone().into_use_and_definition();
    let ledger_def = doc
        .effect_definition(effect.definition_id)
        .ok_or(CommandError::EffectDefinitionNotFound {
            id: effect.definition_id.get(),
        })?
        .clone();
    if ledger_def != expected_def {
        return Err(CommandError::RemoveEffectDefinitionMismatch {
            id: effect.definition_id.get(),
        });
    }
    if introduced_definition {
        let remaining = doc.effect_use_count(effect.definition_id);
        if remaining != 1 {
            return Err(CommandError::DefinitionInUse {
                id: effect.definition_id.get(),
                use_ids: doc
                    .effect_use_ids(effect.definition_id)
                    .into_iter()
                    .map(|id| id.get())
                    .collect(),
            });
        }
        if doc
            .effect_definitions
            .iter()
            .position(|d| d.id == effect.definition_id)
            .is_none()
        {
            return Err(CommandError::EffectDefinitionNotFound {
                id: effect.definition_id.get(),
            });
        }
    }
    find_envelope_mut(doc, target)?.effects.remove(index);
    if introduced_definition {
        let idx = doc
            .effect_definitions
            .iter()
            .position(|d| d.id == effect.definition_id)
            .ok_or(CommandError::EffectDefinitionNotFound {
                id: effect.definition_id.get(),
            })?;
        doc.effect_definitions.remove(idx);
    }
    Ok(())
}

/// 共有 recipe(`EffectDefinition`)の ON/OFF を準備する。
///
/// **相手が定義なのは文書模型がそうだから**(D1l)。`enabled` は
/// `EffectDefinition` に在り、評価(`graph::…`)はこの旗で effect を飛ばす。
/// `Command::SetEffectEnabled` は「どの layer のどの Use を通して触ったか」まで
/// 記録するので、ここで使い手を1つ名指す — **誰も使っていない定義は書けない**。
///
/// 同じ値なら `Ok(None)`(変化なし。失敗ではない)。
pub fn prepare_set_effect_enabled(
    doc: &Document,
    definition: crate::stable_id::EffectDefinitionId,
    new: bool,
) -> Result<Option<crate::Command>, CommandError> {
    let old = doc
        .effect_definition(definition)
        .ok_or(CommandError::EffectDefinitionNotFound {
            id: definition.get(),
        })?
        .enabled;
    if old == new {
        return Ok(None);
    }
    let effect = doc.effect_use_ids(definition).into_iter().next().ok_or(
        CommandError::EffectDefinitionUnused {
            id: definition.get(),
        },
    )?;
    let (target, _index) =
        super::effect::find_use_location(doc, effect).ok_or(CommandError::EffectUseNotFound {
            use_id: effect.get(),
        })?;
    Ok(Some(crate::Command::SetEffectEnabled {
        target,
        effect,
        old,
        new,
    }))
}

pub(super) fn apply_set_effect_enabled(
    doc: &mut Document,
    target: LayerId,
    effect: EffectId,
    new: bool,
) -> Result<(), CommandError> {
    let layer = target.get();
    let definition_id = find_envelope(doc, target)
        .ok_or(CommandError::LayerNotFound(layer))?
        .effects
        .iter()
        .find(|u| u.id == effect)
        .map(|u| u.definition_id)
        .ok_or(CommandError::EffectNotFound {
            effect: effect.get(),
            layer,
        })?;
    let def = doc.effect_definition_mut(definition_id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: definition_id.get(),
        },
    )?;
    def.enabled = new;
    Ok(())
}

pub(super) fn apply_delete_effect_definition(
    doc: &mut Document,
    definition: &EffectDefinition,
) -> Result<(), CommandError> {
    let use_ids: Vec<u64> = doc
        .effect_use_ids(definition.id)
        .into_iter()
        .map(|id| id.get())
        .collect();
    if !use_ids.is_empty() {
        return Err(CommandError::DefinitionInUse {
            id: definition.id.get(),
            use_ids,
        });
    }
    let existing = doc.effect_definition(definition.id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: definition.id.get(),
        },
    )?;
    if existing != definition {
        return Err(CommandError::EffectDefinitionMismatch {
            id: definition.id.get(),
        });
    }
    let idx = doc
        .effect_definitions
        .iter()
        .position(|d| d.id == definition.id)
        .ok_or(CommandError::EffectDefinitionNotFound {
            id: definition.id.get(),
        })?;
    doc.effect_definitions.remove(idx);
    Ok(())
}

pub(super) fn apply_add_effect_definition(
    doc: &mut Document,
    definition: &EffectDefinition,
) -> Result<(), CommandError> {
    if doc.effect_definition(definition.id).is_some() {
        return Err(CommandError::EffectDefinitionAlreadyExists {
            id: definition.id.get(),
        });
    }
    if stable_id_in_use(doc, definition.id.get()) {
        return Err(CommandError::StableIdCollision {
            id: definition.id.get(),
        });
    }
    doc.effect_definitions.push(definition.clone());
    Ok(())
}
