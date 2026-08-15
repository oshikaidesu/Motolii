//! keyframe actionを既存writer prepareへ渡す。新しい編集経路は持たない。

use motolii_doc::{
    AddPositionKeyPreparation, AddTransformParamKeyPreparation, Command, LayerId, ScalarPropertyId,
};

use super::action::DocumentEditActionKind;
use super::error::{DocumentEditRuntimeError, PublishedDocument};
use super::requests::{
    AddPositionKeyRequest, AddTransformParamKeyRequest, RemovePositionKeyRequest,
    SetPositionKeyInterpRequest, SetPositionKeyTimeRequest, SetPositionKeyValueRequest,
};
use super::runtime::{next_projection_generation, DocumentEditRuntime};

impl DocumentEditRuntime {
    pub(super) fn process_add_position_key(
        &mut self,
        request: AddPositionKeyRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(primary) = current_primary else {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        };
        if primary != request.target {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        let preparation = self
            .writer
            .prepare_add_position_key(request.target, request.time)?;
        let AddPositionKeyPreparation::Prepared { command, .. } = preparation else {
            return Ok(None);
        };
        let projection_generation = next_projection_generation(current_projection_generation)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            None,
        )
    }

    pub(super) fn process_add_transform_param_key(
        &mut self,
        request: AddTransformParamKeyRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(primary) = current_primary else {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        };
        if primary != request.target {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        if !matches!(
            request.property,
            ScalarPropertyId::Scale | ScalarPropertyId::Rotation | ScalarPropertyId::Opacity
        ) {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        let preparation = self.writer.prepare_add_transform_param_key(
            request.target,
            request.property,
            request.time,
        )?;
        let AddTransformParamKeyPreparation::Prepared { command, .. } = preparation else {
            return Ok(None);
        };
        let projection_generation = next_projection_generation(current_projection_generation)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            None,
        )
    }

    pub(super) fn process_set_position_key_interp(
        &mut self,
        request: SetPositionKeyInterpRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(primary) = current_primary else {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        };
        if primary != request.target {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        let command = self.writer.prepare_set_position_key_interp(
            request.target,
            request.key,
            request.interp,
        )?;
        let Some(command) = command else {
            return Ok(None);
        };
        let projection_generation = next_projection_generation(current_projection_generation)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            None,
        )
    }

    pub(super) fn process_set_position_key_value(
        &mut self,
        request: SetPositionKeyValueRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(primary) = current_primary else {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        };
        if primary != request.target {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        let command =
            self.writer
                .prepare_set_position_key_value(request.target, request.key, request.new)?;
        let Some(command) = command else {
            return Ok(None);
        };
        let Command::SetPositionKeyValue { old, .. } = &command else {
            return Err(DocumentEditRuntimeError::PositionKeyPrepareMismatch);
        };
        if *old != request.old {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        let projection_generation = next_projection_generation(current_projection_generation)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            None,
        )
    }

    pub(super) fn process_set_position_key_time(
        &mut self,
        request: SetPositionKeyTimeRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let command =
            self.writer
                .prepare_set_position_key_time(request.target, request.key, request.new)?;
        let Some(command) = command else {
            return Ok(None);
        };
        let Command::SetPositionKeyTime { old, .. } = &command else {
            return Err(DocumentEditRuntimeError::PositionKeyPrepareMismatch);
        };
        if *old != request.old {
            return Err(DocumentEditRuntimeError::PrepareRejected);
        }
        let projection_generation = next_projection_generation(current_projection_generation)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            None,
        )
    }

    pub(super) fn process_remove_position_key(
        &mut self,
        request: RemovePositionKeyRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let command = self
            .writer
            .prepare_remove_position_key(request.target, request.key)?;
        let projection_generation = next_projection_generation(current_projection_generation)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            None,
        )
    }
}
