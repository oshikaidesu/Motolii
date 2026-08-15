//! clip/toggle actionを既存writer prepareへ渡す。新しい編集経路は持たない。

use motolii_core::RationalTime;
use motolii_doc::LayerId;

use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;

use super::action::DocumentEditActionKind;
use super::error::{DocumentEditRuntimeError, PublishedDocument};
use super::runtime::{next_projection_generation, DocumentEditRuntime};

impl DocumentEditRuntime {
    pub(super) fn process_move_clip(
        &mut self,
        request: TimelineMoveRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(command) = self
            .writer
            .prepare_set_clip_start(request.layer, request.new_start)?
        else {
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

    pub(super) fn process_trim_clip(
        &mut self,
        request: TimelineTrimRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let command = match request {
            TimelineTrimRequest::In { layer, new_start } => {
                self.writer.prepare_trim_clip_in(layer, new_start)?
            }
            TimelineTrimRequest::Out { layer, new_end } => {
                self.writer.prepare_trim_clip_out(layer, new_end)?
            }
        };
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

    pub(super) fn process_split_clip(
        &mut self,
        layer: LayerId,
        at: RationalTime,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(command) = self.writer.prepare_split_clip(layer, at)? else {
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

    pub(super) fn process_reparent_clip(
        &mut self,
        layer: LayerId,
        dest_layer: LayerId,
        new_start: Option<RationalTime>,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let snapshot = self.writer.snapshot();
        let Some((new_parent, new_index, _)) =
            motolii_doc::find_item_location(snapshot.as_ref(), dest_layer)
        else {
            return Err(DocumentEditRuntimeError::SelectionTargetNotFound(
                dest_layer,
            ));
        };
        let Some(command) = self
            .writer
            .prepare_reparent_clip(layer, new_parent, new_index, new_start)?
        else {
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

    pub(super) fn process_duplicate_layer(
        &mut self,
        layer: LayerId,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let command = self.writer.prepare_duplicate_track_item(layer)?;
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

    pub(super) fn process_toggle_visible(
        &mut self,
        layer: LayerId,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(visible) = self.writer.find_envelope(layer).map(|env| env.visible) else {
            return Err(DocumentEditRuntimeError::SelectionTargetNotFound(layer));
        };
        let Some(command) = self.writer.prepare_set_item_visible(layer, !visible)? else {
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

    pub(super) fn process_toggle_solo(
        &mut self,
        layer: LayerId,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(solo) = self.writer.find_envelope(layer).map(|env| env.solo) else {
            return Err(DocumentEditRuntimeError::SelectionTargetNotFound(layer));
        };
        let Some(command) = self.writer.prepare_set_item_solo(layer, !solo)? else {
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
}
