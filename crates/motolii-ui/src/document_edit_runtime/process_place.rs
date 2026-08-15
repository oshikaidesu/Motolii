//! 配置actionを既存prepare/commitへ渡す。queueは増やさない。

use motolii_doc::LayerId;

use super::action::DocumentEditActionKind;
use super::error::{DocumentEditRuntimeError, PublishedDocument};
use super::prepare_place::{prepare_media_commands, prepare_vism_command, VectorShapeKind};
use super::requests::{PlaceMediaRequest, PlaceRectangleRequest, PlaceVismRequest};
use super::runtime::{next_projection_generation, DocumentEditRuntime};

impl DocumentEditRuntime {
    pub(super) fn process_place_rectangle(
        &mut self,
        request: PlaceRectangleRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        self.commit_vector_shape(
            request,
            VectorShapeKind::Rectangle,
            kind,
            current_primary,
            current_projection_generation,
        )
    }

    pub(super) fn process_place_ellipse(
        &mut self,
        request: PlaceRectangleRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        self.commit_vector_shape(
            request,
            VectorShapeKind::Ellipse,
            kind,
            current_primary,
            current_projection_generation,
        )
    }

    pub(super) fn process_place_vism(
        &mut self,
        request: PlaceVismRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let next_projection_generation = next_projection_generation(current_projection_generation)?;
        let (command, layer_id, expected_live_next) = prepare_vism_command(
            &self.writer.snapshot(),
            &self.catalog,
            current_primary,
            request,
        )?;
        if self.writer.snapshot().layers.peek_next() != expected_live_next {
            return Err(DocumentEditRuntimeError::LayerIdReservationChanged);
        }
        self.commit_command(
            command,
            kind,
            current_primary,
            Some(layer_id),
            next_projection_generation,
            None,
        )
    }

    pub(super) fn process_place_media(
        &mut self,
        request: PlaceMediaRequest,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        let next_projection_generation = next_projection_generation(current_projection_generation)?;
        let (commands, layer_id, expected_live_next) =
            prepare_media_commands(&self.writer.snapshot(), current_primary, request)?;
        if self.writer.snapshot().layers.peek_next() != expected_live_next {
            return Err(DocumentEditRuntimeError::LayerIdReservationChanged);
        }
        // ponytail: AdmitAsset と AddTrackItem は既存 1 command commit を直列する。
        // 1 undo で clip が消え、未使用 Asset は次の undo。macro 化は journal 面が要る。
        let last = commands.len().saturating_sub(1);
        let mut published = None;
        for (index, command) in commands.into_iter().enumerate() {
            let success_primary = if index == last {
                Some(layer_id)
            } else {
                current_primary
            };
            published = self.commit_command(
                command,
                kind,
                current_primary,
                success_primary,
                next_projection_generation,
                None,
            )?;
        }
        Ok(published)
    }
}
