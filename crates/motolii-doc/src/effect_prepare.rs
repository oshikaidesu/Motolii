//! D1l Stage B-3: Writer prepare API — counter clone 上で決定済み Command を構築する。

use std::collections::BTreeMap;

use motolii_core::RationalTime;
use motolii_eval::{DataTrackId, Interp};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::command::{
    find_envelope, find_use_location, guard_effect_lifecycle_document, introduced_ids_copy_local,
    introduced_ids_create, introduced_ids_link, validate_reservation_closure, Command,
    CommandError,
};
use crate::doc_keyframe::{
    validate_keyframe_times_and_interp, DocKeyframe, DocKeyframeError, DocKeyframeTrack,
};
use crate::doc_value::DocValue;
use crate::duplicate::{definition_semantic_body_eq, remint_effect_definition};
use crate::param::{DocParam, LookAtAxis};
use crate::schema::{EffectDefinition, EffectUse};
use crate::stable_id::{
    EffectDefinitionId, EffectId, KeyframeId, StableIdError, StableIdReservation, StableIdSeq,
};
use crate::validate::{self, DocumentError};
use crate::{Document, LayerId};

/// Create/Copy prepare 入力: identity・serde 無しの runtime-only キーフレーム。
#[derive(Debug, Clone, PartialEq)]
pub struct DraftKeyframe {
    pub t: RationalTime,
    pub value: DocValue,
    pub interp: Interp,
}

/// Create prepare 入力: 現行 `DocParam` 全 variant を保持する Draft。
#[derive(Debug, Clone, PartialEq)]
pub enum DraftDocParam {
    Const(DocValue),
    Keyframes(Vec<DraftKeyframe>),
    Data {
        track: DataTrackId,
        fallback: DocValue,
    },
    Vec2Axes {
        x: Box<DraftDocParam>,
        y: Box<DraftDocParam>,
    },
    LookAt {
        target: LayerId,
        axis: LookAtAxis,
    },
    Follow {
        target: LayerId,
        offset: [f64; 2],
    },
}

/// Create prepare 入力: 新 Effect recipe の Draft。
#[derive(Debug, Clone, PartialEq)]
pub struct EffectDefinitionDraft {
    pub plugin_id: String,
    pub effect_version: u32,
    pub enabled: bool,
    pub params: BTreeMap<String, DraftDocParam>,
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum PrepareError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Keyframe(#[from] DocKeyframeError),
    #[error(transparent)]
    StableId(#[from] StableIdError),
    #[error(transparent)]
    Validate(#[from] DocumentError),
}

fn validate_draft_param(
    doc: &Document,
    draft: &DraftDocParam,
    path: &str,
) -> Result<(), PrepareError> {
    match draft {
        DraftDocParam::Const(v) => {
            let param = DocParam::Const(v.clone());
            validate::validate_param_structure(doc, &param, path)?;
        }
        DraftDocParam::Keyframes(keys) => {
            validate_keyframe_times_and_interp(
                keys.iter().map(|k| k.t),
                keys.iter().map(|k| &k.interp),
            )?;
            validate::validate_keyframe_draft_values(
                doc,
                &keys.iter().map(|k| k.value.clone()).collect::<Vec<_>>(),
                path,
            )?;
        }
        DraftDocParam::Data { track, fallback } => {
            let param = DocParam::Data {
                track: track.clone(),
                fallback: fallback.clone(),
            };
            validate::validate_param_structure(doc, &param, path)?;
        }
        DraftDocParam::Vec2Axes { x, y } => {
            validate_draft_param(doc, x, &format!("{path}.x"))?;
            validate_draft_param(doc, y, &format!("{path}.y"))?;
        }
        DraftDocParam::LookAt { target, axis } => {
            let param = DocParam::LookAt {
                target: *target,
                axis: *axis,
            };
            validate::validate_param_structure(doc, &param, path)?;
        }
        DraftDocParam::Follow { target, offset } => {
            let param = DocParam::Follow {
                target: *target,
                offset: *offset,
            };
            validate::validate_param_structure(doc, &param, path)?;
        }
    }
    Ok(())
}

fn validate_draft_definition(
    doc: &Document,
    draft: &EffectDefinitionDraft,
) -> Result<(), PrepareError> {
    for (name, param) in &draft.params {
        validate_draft_param(doc, param, name)?;
    }
    Ok(())
}

fn materialize_draft_param(
    draft: &DraftDocParam,
    seq: &mut StableIdSeq,
) -> Result<DocParam, StableIdError> {
    match draft {
        DraftDocParam::Const(v) => Ok(DocParam::Const(v.clone())),
        DraftDocParam::Keyframes(keys) => {
            let mut track = DocKeyframeTrack::new();
            for key in keys {
                let id = KeyframeId::from_raw(seq.allocate()?);
                track.insert(DocKeyframe {
                    id,
                    t: key.t,
                    value: key.value.clone(),
                    interp: key.interp,
                });
            }
            Ok(DocParam::Keyframes(track))
        }
        DraftDocParam::Data { track, fallback } => Ok(DocParam::Data {
            track: track.clone(),
            fallback: fallback.clone(),
        }),
        DraftDocParam::Vec2Axes { x, y } => Ok(DocParam::Vec2Axes {
            x: Box::new(materialize_draft_param(x, seq)?),
            y: Box::new(materialize_draft_param(y, seq)?),
        }),
        DraftDocParam::LookAt { target, axis } => Ok(DocParam::LookAt {
            target: *target,
            axis: *axis,
        }),
        DraftDocParam::Follow { target, offset } => Ok(DocParam::Follow {
            target: *target,
            offset: *offset,
        }),
    }
}

fn verify_prepared_command(doc: &Document, command: &Command) -> Result<(), PrepareError> {
    let mut temp = doc.clone();
    command.apply(&mut temp).map_err(PrepareError::Command)?;
    Ok(())
}

pub(crate) fn prepare_create_effect(
    doc: &Document,
    target: LayerId,
    index: usize,
    draft: EffectDefinitionDraft,
) -> Result<Command, PrepareError> {
    guard_effect_lifecycle_document(doc)?;
    validate_draft_definition(doc, &draft)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        }
        .into());
    }

    let before = doc.next_stable_id.peek_next();
    let mut seq = doc.next_stable_id;
    let use_id = EffectId::from_raw(seq.allocate()?);
    let def_id = EffectDefinitionId::from_raw(seq.allocate()?);
    let mut params = BTreeMap::new();
    for (name, draft_param) in &draft.params {
        params.insert(
            name.clone(),
            materialize_draft_param(draft_param, &mut seq)?,
        );
    }
    let definition = EffectDefinition::new(
        def_id,
        draft.plugin_id,
        draft.effect_version,
        draft.enabled,
        params,
        draft.extra,
    );
    let use_ = EffectUse {
        id: use_id,
        definition_id: def_id,
    };
    let after = seq.peek_next();
    let reservation = StableIdReservation::new(before, after);
    let introduced = introduced_ids_create(&use_, &definition);
    validate_reservation_closure(reservation, &introduced)?;

    let command = Command::CreateEffect {
        target,
        index,
        use_,
        definition,
        stable_id_reservation: reservation,
    };
    verify_prepared_command(doc, &command)?;
    Ok(command)
}

pub(crate) fn prepare_link_effect_use(
    doc: &Document,
    target: LayerId,
    index: usize,
    definition_id: EffectDefinitionId,
) -> Result<Command, PrepareError> {
    guard_effect_lifecycle_document(doc)?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    if index > env.effects.len() {
        return Err(CommandError::IndexOutOfRange {
            index,
            len: env.effects.len(),
        }
        .into());
    }
    if doc.effect_definition(definition_id).is_none() {
        return Err(CommandError::EffectDefinitionNotFound {
            id: definition_id.get(),
        }
        .into());
    }

    let before = doc.next_stable_id.peek_next();
    let mut seq = doc.next_stable_id;
    let use_id = EffectId::from_raw(seq.allocate()?);
    let after = seq.peek_next();
    let use_ = EffectUse {
        id: use_id,
        definition_id,
    };
    let reservation = StableIdReservation::new(before, after);
    let introduced = introduced_ids_link(&use_);
    validate_reservation_closure(reservation, &introduced)?;

    let command = Command::LinkEffectUse {
        target,
        index,
        use_,
        stable_id_reservation: reservation,
    };
    verify_prepared_command(doc, &command)?;
    Ok(command)
}

pub(crate) fn prepare_copy_local_effect(
    doc: &Document,
    use_id: EffectId,
) -> Result<Command, PrepareError> {
    guard_effect_lifecycle_document(doc)?;
    let (target, index) =
        find_use_location(doc, use_id).ok_or(CommandError::EffectUseNotFound {
            use_id: use_id.get(),
        })?;
    let env = find_envelope(doc, target).ok_or(CommandError::LayerNotFound(target.get()))?;
    let previous_definition_id = env.effects[index].definition_id;
    let source = doc.effect_definition(previous_definition_id).ok_or(
        CommandError::EffectDefinitionNotFound {
            id: previous_definition_id.get(),
        },
    )?;

    let before = doc.next_stable_id.peek_next();
    let mut seq = doc.next_stable_id;
    let new_definition_id = EffectDefinitionId::from_raw(seq.allocate()?);
    let mut new_definition = source.deep_copy(new_definition_id);
    remint_effect_definition(&mut new_definition, &mut seq)?;
    if !definition_semantic_body_eq(source, &new_definition) {
        return Err(CommandError::CopyLocalPayloadMismatch.into());
    }

    let after = seq.peek_next();
    let reservation = StableIdReservation::new(before, after);
    let introduced = introduced_ids_copy_local(&new_definition);
    validate_reservation_closure(reservation, &introduced)?;

    let command = Command::CopyLocalEffect {
        use_id,
        previous_definition_id,
        new_definition,
        stable_id_reservation: reservation,
    };
    verify_prepared_command(doc, &command)?;
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::RationalTime;

    #[test]
    fn draft_keyframe_duplicate_time_rejects_before_id_allocation() {
        let doc = Document::new_current();
        let t = RationalTime::ZERO;
        let draft = EffectDefinitionDraft {
            plugin_id: "core.filter.blur".into(),
            effect_version: 1,
            enabled: true,
            params: BTreeMap::from([(
                "amount".into(),
                DraftDocParam::Keyframes(vec![
                    DraftKeyframe {
                        t,
                        value: DocValue::F64(0.0),
                        interp: Interp::Hold,
                    },
                    DraftKeyframe {
                        t,
                        value: DocValue::F64(1.0),
                        interp: Interp::Hold,
                    },
                ]),
            )]),
            extra: Map::new(),
        };
        let before = doc.next_stable_id.peek_next();
        let err = prepare_create_effect(&doc, LayerId::from_raw(0), 0, draft).unwrap_err();
        assert!(matches!(
            err,
            PrepareError::Keyframe(DocKeyframeError::UnsortedOrDuplicateKeys)
        ));
        assert_eq!(doc.next_stable_id.peek_next(), before);
    }
}
