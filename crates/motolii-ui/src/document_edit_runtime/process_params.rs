//! param/effect actionを既存prepare/commitへ渡す。新しい編集経路は持たない。

use motolii_core::RationalTime;
use motolii_doc::{Command, LayerId};

use super::action::DocumentEditActionKind;
use super::error::{DocumentEditRuntimeError, PublishedDocument};
use super::prepare_params::{
    prepare_attach_effect_command, prepare_set_opacity_command, prepare_set_position_const_command,
};
use super::requests::{
    AttachEffectRequest, SetEffectParamRequest, SetOpacityRequest, SetPositionConstRequest,
    SetSourceParamRequest,
};
use super::runtime::{next_projection_generation, DocumentEditRuntime};
use super::{prepare_set_effect_param_command, prepare_set_source_param_command};

impl DocumentEditRuntime {
    pub(super) fn process_attach_effect(
        &mut self,
        request: AttachEffectRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(target) = current_primary else {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        };
        let Some(index) = self
            .writer
            .find_envelope(target)
            .map(|envelope| envelope.effects.len())
        else {
            return Err(DocumentEditRuntimeError::SelectionTargetNotFound(target));
        };
        let projection_generation = next_projection_generation(current_projection_generation)?;
        let (command, created_effect_use) =
            prepare_attach_effect_command(&self.writer, &self.catalog, target, index, request)?;
        self.commit_command(
            command,
            kind,
            current_primary,
            current_primary,
            projection_generation,
            Some(created_effect_use),
        )
    }

    pub(super) fn process_set_effect_param(
        &mut self,
        request: SetEffectParamRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(command) =
            prepare_set_effect_param_command(self.writer.snapshot().as_ref(), &request)
        else {
            return Err(DocumentEditRuntimeError::PrepareRejected);
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

    pub(super) fn process_set_source_param(
        &mut self,
        request: SetSourceParamRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        if current_primary.is_none() {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        }
        let Some(command) =
            prepare_set_source_param_command(self.writer.snapshot().as_ref(), &request)
        else {
            return Err(DocumentEditRuntimeError::PrepareRejected);
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

    pub(super) fn process_set_position_const(
        &mut self,
        request: SetPositionConstRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(command) = prepare_set_position_const_command(&self.writer, request) else {
            return Err(DocumentEditRuntimeError::PrepareRejected);
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

    pub(super) fn process_set_opacity(
        &mut self,
        request: SetOpacityRequest,
        time: RationalTime,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        if current_primary.is_none() {
            return Err(DocumentEditRuntimeError::NoPrimarySelection);
        }
        let Some(command) = prepare_set_opacity_command(&self.writer, request, time) else {
            return Err(DocumentEditRuntimeError::PrepareRejected);
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

    pub(super) fn process_stage_transform(
        &mut self,
        command: Command,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
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
