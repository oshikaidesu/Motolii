//! 確定済み編集のactionと、single writerへ直列するqueue。

use std::collections::VecDeque;

use motolii_core::RationalTime;
use motolii_doc::{Command, LayerId, ScalarPropertyId};

use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;
use crate::{DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

use super::requests::{
    AddPositionKeyRequest, AddTransformParamKeyRequest, AttachEffectRequest, PlaceEllipseRequest,
    PlaceMediaRequest, PlaceRectangleRequest, PlaceVismRequest, RemovePositionKeyRequest,
    SetEffectParamRequest, SetOpacityRequest, SetPositionConstRequest, SetPositionKeyInterpRequest,
    SetPositionKeyTimeRequest, SetPositionKeyValueRequest, SetSourceParamRequest,
};

#[derive(Debug)]
pub(crate) enum DocumentEditAction {
    Apply(DocumentCommandRequest),
    PlaceRectangle(PlaceRectangleRequest),
    PlaceEllipse(PlaceEllipseRequest),
    PlaceVism(PlaceVismRequest),
    PlaceMedia(PlaceMediaRequest),
    AttachEffect(AttachEffectRequest),
    SetEffectParam(SetEffectParamRequest),
    SetSourceParam(SetSourceParamRequest),
    SetPositionConst(SetPositionConstRequest),
    SetOpacity {
        request: SetOpacityRequest,
        time: RationalTime,
    },
    StageTransform(Command),
    AddPositionKey(AddPositionKeyRequest),
    AddTransformParamKey(AddTransformParamKeyRequest),
    SetPositionKeyInterp(SetPositionKeyInterpRequest),
    SetPositionKeyValue(SetPositionKeyValueRequest),
    SetPositionKeyTime(SetPositionKeyTimeRequest),
    RemovePositionKey(RemovePositionKeyRequest),
    MoveClip(TimelineMoveRequest),
    TrimClip(TimelineTrimRequest),
    SplitClip {
        layer: LayerId,
        at: RationalTime,
    },
    ReparentClip {
        layer: LayerId,
        dest_layer: LayerId,
        new_start: Option<RationalTime>,
    },
    DuplicateLayer {
        layer: LayerId,
    },
    ToggleVisible {
        layer: LayerId,
    },
    ToggleSolo {
        layer: LayerId,
    },
    ReplacePrimary(LayerId),
    ClearPrimary,
    Undo,
    Redo,
}

impl DocumentEditAction {
    pub(crate) const fn kind(&self) -> DocumentEditActionKind {
        match self {
            Self::Apply(_) => DocumentEditActionKind::Apply,
            Self::PlaceRectangle(_) => DocumentEditActionKind::PlaceRectangle,
            Self::PlaceEllipse(_) => DocumentEditActionKind::PlaceEllipse,
            Self::PlaceVism(_) => DocumentEditActionKind::PlaceVism,
            Self::PlaceMedia(_) => DocumentEditActionKind::PlaceMedia,
            Self::AttachEffect(_) => DocumentEditActionKind::AttachEffect,
            Self::SetEffectParam(_) => DocumentEditActionKind::SetEffectParam,
            Self::SetSourceParam(_) => DocumentEditActionKind::SetSourceParam,
            Self::SetPositionConst(_) => DocumentEditActionKind::SetPositionConst,
            Self::SetOpacity { .. } => DocumentEditActionKind::SetOpacity,
            Self::StageTransform(_) => DocumentEditActionKind::StageTransform,
            Self::AddPositionKey(_) => DocumentEditActionKind::AddPositionKey,
            Self::AddTransformParamKey(_) => DocumentEditActionKind::AddTransformParamKey,
            Self::SetPositionKeyInterp(_) => DocumentEditActionKind::SetPositionKeyInterp,
            Self::SetPositionKeyValue(_) => DocumentEditActionKind::SetPositionKeyValue,
            Self::SetPositionKeyTime(_) => DocumentEditActionKind::SetPositionKeyTime,
            Self::RemovePositionKey(_) => DocumentEditActionKind::RemovePositionKey,
            Self::MoveClip(_) => DocumentEditActionKind::MoveClip,
            Self::TrimClip(_) => DocumentEditActionKind::TrimClip,
            Self::SplitClip { .. } => DocumentEditActionKind::SplitClip,
            Self::ReparentClip { .. } => DocumentEditActionKind::ReparentClip,
            Self::DuplicateLayer { .. } => DocumentEditActionKind::DuplicateLayer,
            Self::ToggleVisible { .. } => DocumentEditActionKind::ToggleVisible,
            Self::ToggleSolo { .. } => DocumentEditActionKind::ToggleSolo,
            Self::ReplacePrimary(_) => DocumentEditActionKind::ReplacePrimary,
            Self::ClearPrimary => DocumentEditActionKind::ClearPrimary,
            Self::Undo => DocumentEditActionKind::Undo,
            Self::Redo => DocumentEditActionKind::Redo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocumentEditActionKind {
    Apply,
    PlaceRectangle,
    PlaceEllipse,
    PlaceVism,
    PlaceMedia,
    AttachEffect,
    SetEffectParam,
    SetSourceParam,
    SetPositionConst,
    SetOpacity,
    StageTransform,
    AddPositionKey,
    AddTransformParamKey,
    SetPositionKeyInterp,
    SetPositionKeyValue,
    SetPositionKeyTime,
    RemovePositionKey,
    MoveClip,
    TrimClip,
    SplitClip,
    ReparentClip,
    DuplicateLayer,
    ToggleVisible,
    ToggleSolo,
    ReplacePrimary,
    ClearPrimary,
    Undo,
    Redo,
}

#[derive(Debug, Default)]
pub(crate) struct DocumentEditQueue {
    pub(super) pending: VecDeque<DocumentEditAction>,
}

impl DocumentEditQueue {
    pub(crate) fn push_place_rectangle(&mut self, request: PlaceRectangleRequest) {
        self.pending
            .push_back(DocumentEditAction::PlaceRectangle(request));
    }

    pub(crate) fn push_place_ellipse(&mut self, request: PlaceEllipseRequest) {
        self.pending
            .push_back(DocumentEditAction::PlaceEllipse(request));
    }

    pub(crate) fn push_place_vism(&mut self, request: PlaceVismRequest) {
        self.pending
            .push_back(DocumentEditAction::PlaceVism(request));
    }

    pub(crate) fn push_place_media(&mut self, request: PlaceMediaRequest) {
        self.pending
            .push_back(DocumentEditAction::PlaceMedia(request));
    }

    pub(crate) fn push_attach_effect(&mut self, request: AttachEffectRequest) {
        self.pending
            .push_back(DocumentEditAction::AttachEffect(request));
    }

    pub(crate) fn push_set_effect_param(&mut self, request: SetEffectParamRequest) {
        self.pending
            .push_back(DocumentEditAction::SetEffectParam(request));
    }

    pub(crate) fn push_set_source_param(&mut self, request: SetSourceParamRequest) {
        self.pending
            .push_back(DocumentEditAction::SetSourceParam(request));
    }

    pub(crate) fn push_set_position_const(&mut self, request: SetPositionConstRequest) {
        self.pending
            .push_back(DocumentEditAction::SetPositionConst(request));
    }

    pub(crate) fn push_set_opacity(&mut self, request: SetOpacityRequest) {
        self.push_set_opacity_at(request, RationalTime::ZERO);
    }

    pub(crate) fn push_set_opacity_at(&mut self, request: SetOpacityRequest, time: RationalTime) {
        self.pending
            .push_back(DocumentEditAction::SetOpacity { request, time });
    }

    pub(crate) fn push_stage_transform(&mut self, command: Command) -> bool {
        let accepted = matches!(
            &command,
            Command::SetProperty {
                property: ScalarPropertyId::Position
                    | ScalarPropertyId::Scale
                    | ScalarPropertyId::Rotation,
                ..
            } | Command::SetPositionKeyValue { .. }
        );
        if accepted {
            self.pending
                .push_back(DocumentEditAction::StageTransform(command));
        }
        accepted
    }

    pub(crate) fn push_add_position_key(&mut self, request: AddPositionKeyRequest) {
        self.pending
            .push_back(DocumentEditAction::AddPositionKey(request));
    }

    pub(crate) fn push_add_transform_param_key(&mut self, request: AddTransformParamKeyRequest) {
        self.pending
            .push_back(DocumentEditAction::AddTransformParamKey(request));
    }

    pub(crate) fn push_set_position_key_interp(&mut self, request: SetPositionKeyInterpRequest) {
        self.pending
            .push_back(DocumentEditAction::SetPositionKeyInterp(request));
    }

    pub(crate) fn push_set_position_key_value(&mut self, request: SetPositionKeyValueRequest) {
        self.pending
            .push_back(DocumentEditAction::SetPositionKeyValue(request));
    }

    pub(crate) fn push_set_position_key_time(&mut self, request: SetPositionKeyTimeRequest) {
        self.pending
            .push_back(DocumentEditAction::SetPositionKeyTime(request));
    }

    pub(crate) fn push_remove_position_key(&mut self, request: RemovePositionKeyRequest) {
        self.pending
            .push_back(DocumentEditAction::RemovePositionKey(request));
    }

    pub(crate) fn push_move_clip(&mut self, request: TimelineMoveRequest) {
        self.pending
            .push_back(DocumentEditAction::MoveClip(request));
    }

    pub(crate) fn push_trim_clip(&mut self, request: TimelineTrimRequest) {
        self.pending
            .push_back(DocumentEditAction::TrimClip(request));
    }

    pub(crate) fn push_split_clip(&mut self, layer: LayerId, at: RationalTime) {
        self.pending
            .push_back(DocumentEditAction::SplitClip { layer, at });
    }

    pub(crate) fn push_reparent_clip(
        &mut self,
        layer: LayerId,
        dest_layer: LayerId,
        new_start: Option<RationalTime>,
    ) {
        self.pending.push_back(DocumentEditAction::ReparentClip {
            layer,
            dest_layer,
            new_start,
        });
    }

    pub(crate) fn push_duplicate_layer(&mut self, layer: LayerId) {
        self.pending
            .push_back(DocumentEditAction::DuplicateLayer { layer });
    }

    pub(crate) fn push_toggle_visible(&mut self, layer: LayerId) {
        self.pending
            .push_back(DocumentEditAction::ToggleVisible { layer });
    }

    pub(crate) fn push_toggle_solo(&mut self, layer: LayerId) {
        self.pending
            .push_back(DocumentEditAction::ToggleSolo { layer });
    }

    pub(crate) fn push_replace_primary(&mut self, target: LayerId) {
        self.pending
            .push_back(DocumentEditAction::ReplacePrimary(target));
    }

    pub(crate) fn push_clear_primary(&mut self) {
        self.pending.push_back(DocumentEditAction::ClearPrimary);
    }

    pub(crate) fn push_prepared(
        &mut self,
        output: RouterOutput,
        request: Option<DocumentCommandRequest>,
    ) -> Result<(), DocumentEditDispatchError> {
        let RouterOutput::Intent { phase, intent, .. } = output else {
            return Err(DocumentEditDispatchError::NotCommitIntent);
        };
        match (phase, intent) {
            (InputPhase::Click, DomainIntent::DeleteTargetedItems) => {
                let request = request.ok_or(DocumentEditDispatchError::MissingPreparedRequest)?;
                if request.intent() != intent {
                    return Err(DocumentEditDispatchError::IntentMismatch {
                        routed: intent,
                        request: request.intent(),
                    });
                }
                self.pending.push_back(DocumentEditAction::Apply(request));
            }
            (InputPhase::Press, DomainIntent::Undo) if request.is_none() => {
                self.pending.push_back(DocumentEditAction::Undo);
            }
            (InputPhase::Press, DomainIntent::Redo) if request.is_none() => {
                self.pending.push_back(DocumentEditAction::Redo);
            }
            (InputPhase::Press, DomainIntent::Undo | DomainIntent::Redo) => {
                return Err(DocumentEditDispatchError::UnexpectedPreparedRequest);
            }
            _ => return Err(DocumentEditDispatchError::NotCommitIntent),
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push_undo(&mut self) {
        self.pending.push_back(DocumentEditAction::Undo);
    }

    #[cfg(test)]
    pub(crate) fn push_redo(&mut self) {
        self.pending.push_back(DocumentEditAction::Redo);
    }

    pub(super) fn pop_front(&mut self) -> Option<DocumentEditAction> {
        self.pending.pop_front()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum DocumentEditDispatchError {
    #[error("router output is not a supported committed Document intent")]
    NotCommitIntent,
    #[error("committed delete intent has no prepared Document request")]
    MissingPreparedRequest,
    #[error("routed intent {routed:?} does not match request intent {request:?}")]
    IntentMismatch {
        routed: DomainIntent,
        request: DomainIntent,
    },
    #[error("Undo/Redo command must not carry a prepared Document request")]
    UnexpectedPreparedRequest,
}
