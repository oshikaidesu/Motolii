//! queue先頭を既存prepare/commitへ振り分ける。新しい編集経路は持たない。

use motolii_doc::LayerId;

use super::action::{DocumentEditAction, DocumentEditQueue};
use super::error::{DocumentEditRuntimeError, PublishedDocument};
use super::runtime::{
    next_projection_generation, DocumentEditRuntime, HistoryDirection, RuntimeHealth,
};

impl DocumentEditRuntime {
    pub(crate) fn process_next(
        &mut self,
        queue: &mut DocumentEditQueue,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let Some(next_action) = queue.pending.front() else {
            return Ok(None);
        };
        let selection_only = matches!(
            next_action,
            DocumentEditAction::ReplacePrimary(_) | DocumentEditAction::ClearPrimary
        );
        if !selection_only {
            if let Some(receipt) = self.blocked_commit_receipt() {
                return Err(DocumentEditRuntimeError::DocumentWriteBlocked { receipt });
            }
        }

        let action = queue.pop_front().expect("front was checked above");
        let kind = action.kind();
        match action {
            DocumentEditAction::Undo => {
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                let action = self
                    .history_projection
                    .prepare(HistoryDirection::Undo, &self.writer)?;
                self.commit_history_action(action, kind, current_primary, projection_generation)
            }
            DocumentEditAction::Redo => {
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                let action = self
                    .history_projection
                    .prepare(HistoryDirection::Redo, &self.writer)?;
                self.commit_history_action(action, kind, current_primary, projection_generation)
            }
            DocumentEditAction::Apply(request) => {
                let next_projection_generation =
                    next_projection_generation(current_projection_generation)?;
                let mut commands = request.into_commands();
                if commands.len() != 1 {
                    return Err(DocumentEditRuntimeError::MultiCommandActionRejected);
                }
                let command = commands.remove(0);
                self.commit_command(
                    command,
                    kind,
                    current_primary,
                    current_primary,
                    next_projection_generation,
                    None,
                )
            }
            DocumentEditAction::PlaceRectangle(request) => self.process_place_rectangle(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::PlaceEllipse(request) => self.process_place_ellipse(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::PlaceVism(request) => self.process_place_vism(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::PlaceMedia(request) => self.process_place_media(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::AttachEffect(request) => self.process_attach_effect(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::SetEffectParam(request) => self.process_set_effect_param(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::SetSourceParam(request) => self.process_set_source_param(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::SetPositionConst(request) => self.process_set_position_const(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::SetOpacity { request, time } => self.process_set_opacity(
                request,
                time,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::StageTransform(command) => self.process_stage_transform(
                command,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::AddPositionKey(request) => self.process_add_position_key(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::AddTransformParamKey(request) => self
                .process_add_transform_param_key(
                    request,
                    kind,
                    current_primary,
                    current_projection_generation,
                ),
            DocumentEditAction::SetPositionKeyInterp(request) => self
                .process_set_position_key_interp(
                    request,
                    kind,
                    current_primary,
                    current_projection_generation,
                ),
            DocumentEditAction::SetPositionKeyValue(request) => self
                .process_set_position_key_value(
                    request,
                    kind,
                    current_primary,
                    current_projection_generation,
                ),
            DocumentEditAction::SetPositionKeyTime(request) => self.process_set_position_key_time(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::RemovePositionKey(request) => self.process_remove_position_key(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::MoveClip(request) => self.process_move_clip(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::TrimClip(request) => self.process_trim_clip(
                request,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::SplitClip { layer, at } => self.process_split_clip(
                layer,
                at,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::ReparentClip {
                layer,
                dest_layer,
                new_start,
            } => self.process_reparent_clip(
                layer,
                dest_layer,
                new_start,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::DuplicateLayer { layer } => self.process_duplicate_layer(
                layer,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::ToggleVisible { layer } => self.process_toggle_visible(
                layer,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::ToggleSolo { layer } => self.process_toggle_solo(
                layer,
                kind,
                current_primary,
                current_projection_generation,
            ),
            DocumentEditAction::ReplacePrimary(target) => {
                if self.writer.find_envelope(target).is_none() {
                    return Err(DocumentEditRuntimeError::SelectionTargetNotFound(target));
                }
                if current_primary == Some(target) {
                    return Ok(None);
                }
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                self.sync_pending_selection(Some(target), projection_generation);
                Ok(Some(PublishedDocument {
                    kind,
                    revision: self.writer.revision,
                    snapshot: self.writer.snapshot(),
                    primary: Some(target),
                    projection_generation,
                    created_effect_use: None,
                }))
            }
            DocumentEditAction::ClearPrimary => {
                if current_primary.is_none() {
                    return Ok(None);
                }
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                self.sync_pending_selection(None, projection_generation);
                Ok(Some(PublishedDocument {
                    kind,
                    revision: self.writer.revision,
                    snapshot: self.writer.snapshot(),
                    primary: None,
                    projection_generation,
                    created_effect_use: None,
                }))
            }
        }
    }

    fn sync_pending_selection(&mut self, primary: Option<LayerId>, projection_generation: u64) {
        let RuntimeHealth::WriteBlocked(pending) = &mut self.health else {
            return;
        };
        pending.prepared.primary =
            primary.filter(|layer| pending.prepared.writer.find_envelope(*layer).is_some());
        pending.prepared.projection_generation = projection_generation;
    }
}
