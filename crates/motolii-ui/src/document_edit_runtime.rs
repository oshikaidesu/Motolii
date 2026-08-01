//! 確定済みDocument編集をsingle writerへ直列配送するprivate runtime。

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use motolii_core::{RationalTime, RationalTimeError};
use motolii_doc::{
    Clip, ClipSource, Command, CommandError, Document, DocumentError, DocumentPluginError,
    DocumentWriter, DraftDocParam, EffectDefinitionDraft, EffectDefinitionId, EffectId,
    ItemEnvelope, JournalEdit, LayerId, LayerIdError, ParentLocator, PreparedPluginRecipe,
    ProjectError, ProjectSession, SaveProjectOptions, ScalarPropertyId, StandardShape, TrackItem,
    UndoError, VectorContent, VectorRecipe,
};
use motolii_plugin::{PluginCatalog, PluginKind};

use crate::{DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

#[derive(Debug)]
pub(crate) enum DocumentEditAction {
    Apply(DocumentCommandRequest),
    PlaceRectangle(PlaceRectangleRequest),
    AttachEffect(AttachEffectRequest),
    SetEffectParam(SetEffectParamRequest),
    MoveClip(MoveClipRequest),
    TrimClip(TrimClipRequest),
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
            Self::AttachEffect(_) => DocumentEditActionKind::AttachEffect,
            Self::SetEffectParam(_) => DocumentEditActionKind::SetEffectParam,
            Self::MoveClip(_) => DocumentEditActionKind::MoveClip,
            Self::TrimClip(_) => DocumentEditActionKind::TrimClip,
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
    AttachEffect,
    SetEffectParam,
    MoveClip,
    TrimClip,
    ReplacePrimary,
    ClearPrimary,
    Undo,
    Redo,
}

#[derive(Debug, Default)]
pub(crate) struct DocumentEditQueue {
    pending: VecDeque<DocumentEditAction>,
}

impl DocumentEditQueue {
    pub(crate) fn push_place_rectangle(&mut self, request: PlaceRectangleRequest) {
        self.pending
            .push_back(DocumentEditAction::PlaceRectangle(request));
    }

    pub(crate) fn push_attach_effect(&mut self, request: AttachEffectRequest) {
        self.pending
            .push_back(DocumentEditAction::AttachEffect(request));
    }

    pub(crate) fn push_set_effect_param(&mut self, request: SetEffectParamRequest) {
        self.pending
            .push_back(DocumentEditAction::SetEffectParam(request));
    }

    pub(crate) fn push_trim_clip(&mut self, request: TrimClipRequest) {
        self.pending
            .push_back(DocumentEditAction::TrimClip(request));
    }

    pub(crate) fn push_move_clip(&mut self, request: MoveClipRequest) {
        self.pending
            .push_back(DocumentEditAction::MoveClip(request));
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

    fn pop_front(&mut self) -> Option<DocumentEditAction> {
        self.pending.pop_front()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.pending.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlaceRectangleRequest {
    pub(crate) position: [f64; 2],
    pub(crate) playhead: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttachEffectRequest {
    pub(crate) plugin_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetEffectParamRequest {
    layer_id: LayerId,
    effect_use_id: EffectId,
    definition_id: EffectDefinitionId,
    plugin_id: String,
    effect_version: u32,
    param_id: String,
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrimClipEdge {
    In,
    Out,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MoveClipRequest {
    pub(crate) layer_id: LayerId,
    pub(crate) start: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrimClipRequest {
    pub(crate) layer_id: LayerId,
    pub(crate) edge: TrimClipEdge,
    pub(crate) time: RationalTime,
}

impl SetEffectParamRequest {
    pub(crate) fn new(
        layer_id: LayerId,
        effect_use_id: EffectId,
        definition_id: EffectDefinitionId,
        plugin_id: String,
        effect_version: u32,
        param_id: String,
        value: f64,
    ) -> Self {
        Self {
            layer_id,
            effect_use_id,
            definition_id,
            plugin_id,
            effect_version,
            param_id,
            value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeHealth {
    Healthy,
    Poisoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryDirection {
    Undo,
    Redo,
}

#[derive(Debug)]
struct PreparedHistoryAction {
    direction: HistoryDirection,
    durable_command: Command,
}

#[derive(Debug, Default)]
struct HistoryProjection {
    undo: Vec<Command>,
    redo: Vec<Command>,
}

impl HistoryProjection {
    fn record_forward(&mut self, command: Command) {
        self.undo.push(command);
        self.redo.clear();
    }

    fn prepare(
        &self,
        direction: HistoryDirection,
        writer: &DocumentWriter,
    ) -> Result<PreparedHistoryAction, DocumentEditRuntimeError> {
        if self.undo.len() != writer.undo_len() || self.redo.len() != writer.redo_len() {
            return Err(DocumentEditRuntimeError::HistoryProjectionMismatch);
        }
        let forward = match direction {
            HistoryDirection::Undo => self
                .undo
                .last()
                .ok_or(DocumentEditRuntimeError::NothingToUndo)?,
            HistoryDirection::Redo => self
                .redo
                .last()
                .ok_or(DocumentEditRuntimeError::NothingToRedo)?,
        };
        let durable_command = match direction {
            HistoryDirection::Undo => forward.inverse(),
            HistoryDirection::Redo => forward.clone(),
        };
        Ok(PreparedHistoryAction {
            direction,
            durable_command,
        })
    }

    fn accept(&mut self, direction: HistoryDirection) -> Result<(), DocumentEditRuntimeError> {
        match direction {
            HistoryDirection::Undo => {
                let command = self
                    .undo
                    .pop()
                    .ok_or(DocumentEditRuntimeError::HistoryProjectionMismatch)?;
                self.redo.push(command);
            }
            HistoryDirection::Redo => {
                let command = self
                    .redo
                    .pop()
                    .ok_or(DocumentEditRuntimeError::HistoryProjectionMismatch)?;
                self.undo.push(command);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeTestFailpoint {
    PostDurableLiveApply,
    Reconcile,
}

#[cfg(test)]
#[derive(Debug, Default)]
struct RuntimeTestHooks {
    failpoint: Option<RuntimeTestFailpoint>,
}

pub(crate) struct DocumentEditRuntime {
    session: ProjectSession,
    writer: DocumentWriter,
    catalog: Arc<PluginCatalog>,
    health: RuntimeHealth,
    history_projection: HistoryProjection,
    #[cfg(test)]
    test_hooks: RuntimeTestHooks,
}

impl DocumentEditRuntime {
    pub(crate) fn new(
        session: ProjectSession,
        document_writer: DocumentWriter,
        catalog: Arc<PluginCatalog>,
    ) -> Self {
        Self {
            session,
            writer: document_writer,
            catalog,
            health: RuntimeHealth::Healthy,
            history_projection: HistoryProjection::default(),
            #[cfg(test)]
            test_hooks: RuntimeTestHooks::default(),
        }
    }

    #[cfg(test)]
    fn set_test_failpoint(&mut self, failpoint: RuntimeTestFailpoint) {
        self.test_hooks.failpoint = Some(failpoint);
    }

    pub(crate) fn snapshot(&self) -> Arc<Document> {
        self.writer.snapshot()
    }

    fn poison(&mut self) {
        self.health = RuntimeHealth::Poisoned;
    }

    pub(crate) fn process_next(
        &mut self,
        queue: &mut DocumentEditQueue,
        current_primary: Option<LayerId>,
        current_projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        if self.health == RuntimeHealth::Poisoned {
            return Err(DocumentEditRuntimeError::SessionPoisoned);
        }

        let Some(action) = queue.pop_front() else {
            return Ok(None);
        };
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
            DocumentEditAction::PlaceRectangle(request) => {
                let next_projection_generation =
                    next_projection_generation(current_projection_generation)?;
                let (command, layer_id, expected_live_next) =
                    prepare_rectangle_command(&self.writer.snapshot(), current_primary, request)?;
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
            DocumentEditAction::AttachEffect(request) => {
                let Some(target) = current_primary else {
                    return Ok(None);
                };
                let Some(index) = self
                    .writer
                    .find_envelope(target)
                    .map(|envelope| envelope.effects.len())
                else {
                    return Ok(None);
                };
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                let (command, created_effect_use) = prepare_attach_effect_command(
                    &self.writer,
                    &self.catalog,
                    target,
                    index,
                    request,
                )?;
                self.commit_command(
                    command,
                    kind,
                    current_primary,
                    current_primary,
                    projection_generation,
                    Some(created_effect_use),
                )
            }
            DocumentEditAction::SetEffectParam(request) => {
                let Some(command) = prepare_set_effect_param_command(&self.writer, &request) else {
                    return Ok(None);
                };
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                self.commit_command(
                    command,
                    kind,
                    current_primary,
                    current_primary,
                    projection_generation,
                    None,
                )
            }
            DocumentEditAction::MoveClip(request) => {
                let Some(command) = self
                    .writer
                    .prepare_set_clip_start(request.layer_id, request.start)?
                else {
                    return Ok(None);
                };
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                self.commit_command(
                    command,
                    kind,
                    current_primary,
                    Some(request.layer_id),
                    projection_generation,
                    None,
                )
            }
            DocumentEditAction::TrimClip(request) => {
                let command = match request.edge {
                    TrimClipEdge::In => self
                        .writer
                        .prepare_trim_clip_in(request.layer_id, request.time)?,
                    TrimClipEdge::Out => self
                        .writer
                        .prepare_trim_clip_out(request.layer_id, request.time)?,
                };
                let Some(command) = command else {
                    return Ok(None);
                };
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
                self.commit_command(
                    command,
                    kind,
                    current_primary,
                    Some(request.layer_id),
                    projection_generation,
                    None,
                )
            }
            DocumentEditAction::ReplacePrimary(target) => {
                if self.writer.find_envelope(target).is_none() {
                    return Err(DocumentEditRuntimeError::SelectionTargetNotFound(target));
                }
                if current_primary == Some(target) {
                    return Ok(None);
                }
                let projection_generation =
                    next_projection_generation(current_projection_generation)?;
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

    fn commit_command(
        &mut self,
        command: Command,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        success_primary: Option<LayerId>,
        projection_generation: u64,
        created_effect_use: Option<EffectId>,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        self.commit_durable(&command)?;

        #[cfg(test)]
        if self.test_hooks.failpoint == Some(RuntimeTestFailpoint::PostDurableLiveApply) {
            self.poison();
            return Err(DocumentEditRuntimeError::PostDurableLiveApply(
                PostDurableLiveApplyError::InjectedInvariant,
            ));
        }

        if let Err(error) = self.writer.apply_macro(vec![command.clone()]) {
            self.poison();
            return Err(DocumentEditRuntimeError::PostDurableLiveApply(
                PostDurableLiveApplyError::Command(error),
            ));
        }
        self.history_projection.record_forward(command);

        #[cfg(test)]
        if self.test_hooks.failpoint == Some(RuntimeTestFailpoint::Reconcile) {
            self.poison();
            return Err(DocumentEditRuntimeError::ReconcileFailed);
        }

        let snapshot = self.writer.snapshot();
        let primary = success_primary
            .or(current_primary)
            .filter(|id| self.writer.find_envelope(*id).is_some());
        Ok(Some(PublishedDocument {
            kind,
            revision: self.writer.revision,
            snapshot,
            primary,
            projection_generation,
            created_effect_use,
        }))
    }

    fn commit_history_action(
        &mut self,
        action: PreparedHistoryAction,
        kind: DocumentEditActionKind,
        current_primary: Option<LayerId>,
        projection_generation: u64,
    ) -> Result<Option<PublishedDocument>, DocumentEditRuntimeError> {
        self.commit_durable(&action.durable_command)?;

        #[cfg(test)]
        if self.test_hooks.failpoint == Some(RuntimeTestFailpoint::PostDurableLiveApply) {
            self.poison();
            return Err(DocumentEditRuntimeError::PostDurableLiveApply(
                PostDurableLiveApplyError::InjectedInvariant,
            ));
        }

        let live_result = match action.direction {
            HistoryDirection::Undo => self.writer.undo(),
            HistoryDirection::Redo => self.writer.redo(),
        };
        if let Err(error) = live_result {
            self.poison();
            return Err(DocumentEditRuntimeError::PostDurableLiveApply(
                PostDurableLiveApplyError::History(error),
            ));
        }
        if self.history_projection.accept(action.direction).is_err() {
            self.poison();
            return Err(DocumentEditRuntimeError::PostDurableLiveApply(
                PostDurableLiveApplyError::HistoryProjection,
            ));
        }

        #[cfg(test)]
        if self.test_hooks.failpoint == Some(RuntimeTestFailpoint::Reconcile) {
            self.poison();
            return Err(DocumentEditRuntimeError::ReconcileFailed);
        }

        let snapshot = self.writer.snapshot();
        let primary = current_primary.filter(|id| self.writer.find_envelope(*id).is_some());
        let created_effect_use = if let HistoryDirection::Redo = action.direction {
            match &action.durable_command {
                Command::CreateEffect { target, use_, .. }
                    if current_primary.as_ref() == Some(target) =>
                {
                    Some(use_.id)
                }
                _ => None,
            }
        } else {
            None
        };
        Ok(Some(PublishedDocument {
            kind,
            revision: self.writer.revision,
            snapshot,
            primary,
            projection_generation,
            created_effect_use,
        }))
    }

    fn commit_durable(&mut self, command: &Command) -> Result<(), DocumentEditRuntimeError> {
        let mut candidate: Document = (*self.writer.snapshot()).clone();
        command.apply(&mut candidate)?;
        candidate.validate()?;
        candidate.prepare_plugins(&self.catalog)?;

        let options = SaveProjectOptions {
            limits: *self.session.limits(),
            journal_edit: Some(JournalEdit::new(command.clone())),
            checkpoint: false,
            ..SaveProjectOptions::default()
        };
        if let Err(error) = self.session.save_with_journal(&candidate, &options) {
            self.poison();
            return Err(DocumentEditRuntimeError::JournalCommit(error));
        }
        Ok(())
    }

    #[cfg(test)]
    fn history_lengths(&self) -> (usize, usize) {
        (self.writer.undo_len(), self.writer.redo_len())
    }

    #[cfg(test)]
    fn revision(&self) -> u64 {
        self.writer.revision
    }

    #[cfg(test)]
    fn is_poisoned(&self) -> bool {
        self.health == RuntimeHealth::Poisoned
    }
}

fn next_projection_generation(current: u64) -> Result<u64, DocumentEditRuntimeError> {
    current
        .checked_add(1)
        .ok_or(DocumentEditRuntimeError::ProjectionGenerationExhausted)
}

fn prepare_set_effect_param_command(
    writer: &DocumentWriter,
    request: &SetEffectParamRequest,
) -> Option<Command> {
    if request.plugin_id != "core.filter.opacity"
        || request.effect_version != 1
        || request.param_id != "amount"
        || !request.value.is_finite()
        || !(0.0..=1.0).contains(&request.value)
    {
        return None;
    }
    let snapshot = writer.snapshot();
    let effect_use = snapshot.find_effect_use(request.layer_id, request.effect_use_id)?;
    if effect_use.definition_id != request.definition_id {
        return None;
    }
    let definition = snapshot.effect_definition(effect_use.definition_id)?;
    if definition.plugin_id != request.plugin_id
        || definition.effect_version != request.effect_version
        || definition.params.len() != 1
    {
        return None;
    }
    let old_value = definition.params.get("amount")?;
    let motolii_doc::DocParam::Const(motolii_doc::DocValue::F64(old_f64)) = old_value else {
        return None;
    };
    if !old_f64.is_finite() || !(0.0..=1.0).contains(old_f64) {
        return None;
    }
    if *old_f64 == request.value {
        return None;
    }
    Some(Command::SetProperty {
        target: request.layer_id,
        property: ScalarPropertyId::EffectParam(request.effect_use_id, request.param_id.clone()),
        old_value: old_value.clone(),
        new_value: motolii_doc::DocParam::const_f64(request.value),
    })
}

fn prepare_attach_effect_command(
    writer: &DocumentWriter,
    catalog: &PluginCatalog,
    target: LayerId,
    index: usize,
    request: AttachEffectRequest,
) -> Result<(Command, EffectId), DocumentEditRuntimeError> {
    let current_version = catalog
        .get(&request.plugin_id)
        .ok_or_else(|| DocumentPluginError::ContractMissing {
            plugin_id: request.plugin_id.clone(),
        })?
        .node
        .version;
    let recipe = motolii_doc::prepare_plugin_recipe(
        &request.plugin_id,
        PluginKind::Filter,
        current_version,
        &BTreeMap::new(),
        catalog,
    )?;
    let draft = attach_effect_draft(recipe)?;
    let command = writer.prepare_create_effect(target, index, draft)?;
    let created_effect_use = match &command {
        Command::CreateEffect { use_, .. } => use_.id,
        _ => return Err(DocumentEditRuntimeError::AttachPrepareCommandMismatch),
    };
    Ok((command, created_effect_use))
}

fn attach_effect_draft(
    recipe: PreparedPluginRecipe,
) -> Result<EffectDefinitionDraft, DocumentEditRuntimeError> {
    let mut params = BTreeMap::new();
    for (name, param) in recipe.params {
        let motolii_doc::DocParam::Const(value) = param else {
            return Err(DocumentEditRuntimeError::AttachDefaultNotConst { param: name });
        };
        params.insert(name, DraftDocParam::Const(value));
    }
    Ok(EffectDefinitionDraft {
        plugin_id: recipe.plugin_id,
        effect_version: recipe.current_version,
        enabled: true,
        params,
        extra: Default::default(),
    })
}

fn prepare_rectangle_command(
    snapshot: &Document,
    current_primary: Option<LayerId>,
    request: PlaceRectangleRequest,
) -> Result<(Command, LayerId, u64), DocumentEditRuntimeError> {
    if !request.position[0].is_finite() || !request.position[1].is_finite() {
        return Err(DocumentEditRuntimeError::NonFiniteDropPosition);
    }
    if request.playhead < RationalTime::ZERO {
        return Err(DocumentEditRuntimeError::PlayheadOutsideComposition);
    }
    let duration = snapshot.composition.duration.try_sub(request.playhead)?;
    if duration < snapshot.composition.fps.frame_duration() {
        return Err(DocumentEditRuntimeError::RemainingDurationBelowOneFrame);
    }

    let mut layers = snapshot.layers.clone();
    let expected_live_next = layers.peek_next();
    let layer_id = layers.reserve()?;
    let (track_id, index) = rectangle_insertion(snapshot, current_primary)
        .ok_or(DocumentEditRuntimeError::NoTrackForRectangle)?;

    let mut envelope = ItemEnvelope::new(layer_id);
    envelope.transform.position = motolii_doc::DocParam::const_vec2(request.position);
    let item = TrackItem::Clip(Clip {
        envelope,
        start: request.playhead,
        duration,
        time_map: Default::default(),
        source: ClipSource::Vector {
            recipe: VectorRecipe {
                content: VectorContent::StandardShape {
                    shape: StandardShape::Rect {
                        width: motolii_doc::DocParam::const_f64(0.2),
                        height: motolii_doc::DocParam::const_f64(0.2),
                    },
                },
                modifiers: Vec::new(),
            },
        },
    });
    Ok((
        Command::AddTrackItem {
            parent: ParentLocator::Track(track_id),
            index,
            item,
            layer_names: BTreeMap::from([(layer_id, "Rectangle".to_owned())]),
        },
        layer_id,
        expected_live_next,
    ))
}

fn rectangle_insertion(
    snapshot: &Document,
    current_primary: Option<LayerId>,
) -> Option<(motolii_doc::TrackId, usize)> {
    if let Some(primary) = current_primary {
        for track in &snapshot.tracks {
            if let Some(index) = track
                .items
                .iter()
                .position(|item| item_layer_id(item) == primary)
            {
                if rectangle_selection_is_compatible(&track.items[index]) {
                    return Some((track.id, index + 1));
                }
                break;
            }
        }
    }
    snapshot
        .tracks
        .first()
        .map(|track| (track.id, track.items.len()))
}

fn item_layer_id(item: &TrackItem) -> LayerId {
    match item {
        TrackItem::Clip(clip) => clip.envelope.layer_id,
        TrackItem::Group(group) => group.envelope.layer_id,
    }
}

fn rectangle_selection_is_compatible(item: &TrackItem) -> bool {
    match item {
        TrackItem::Group(_) => true,
        TrackItem::Clip(clip) => match &clip.source {
            ClipSource::Asset { video, .. } => video.is_some(),
            ClipSource::Plugin { .. } | ClipSource::Vector { .. } => true,
        },
    }
}

#[derive(Debug)]
pub(crate) struct PublishedDocument {
    pub(crate) kind: DocumentEditActionKind,
    pub(crate) revision: u64,
    pub(crate) snapshot: Arc<Document>,
    pub(crate) primary: Option<LayerId>,
    pub(crate) projection_generation: u64,
    pub(crate) created_effect_use: Option<EffectId>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PostDurableLiveApplyError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    History(#[from] UndoError),
    #[error("private history projection diverged after durable commit")]
    HistoryProjection,
    #[cfg(test)]
    #[error("injected post-durable live apply invariant")]
    InjectedInvariant,
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

#[derive(Debug, thiserror::Error)]
pub(crate) enum DocumentEditRuntimeError {
    #[error("projection generation is exhausted at u64::MAX")]
    ProjectionGenerationExhausted,
    #[error("selection target does not exist in the live Document: {0:?}")]
    SelectionTargetNotFound(LayerId),
    #[error("document edit runtime session is poisoned")]
    SessionPoisoned,
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
    #[error("private prepared-action projection does not match live Undo/Redo history")]
    HistoryProjectionMismatch,
    #[error("document edit action must contain exactly one command")]
    MultiCommandActionRejected,
    #[error("Rectangle drop position must be finite canonical coordinates")]
    NonFiniteDropPosition,
    #[error("Rectangle playhead is outside the composition")]
    PlayheadOutsideComposition,
    #[error("Rectangle remaining duration is below one frame")]
    RemainingDurationBelowOneFrame,
    #[error("Rectangle placement requires an existing Track")]
    NoTrackForRectangle,
    #[error("Rectangle LayerId reservation changed before live apply")]
    LayerIdReservationChanged,
    #[error("prepared attach parameter `{param}` is not Const")]
    AttachDefaultNotConst { param: String },
    #[error("prepare_create_effect returned a non-CreateEffect command")]
    AttachPrepareCommandMismatch,
    #[error(transparent)]
    LayerId(#[from] LayerIdError),
    #[error(transparent)]
    RationalTime(#[from] RationalTimeError),
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error(transparent)]
    DocumentPlugin(#[from] DocumentPluginError),
    #[error(transparent)]
    EffectPrepare(#[from] motolii_doc::PrepareError),
    #[error("journal durable commit failed")]
    JournalCommit(#[source] Box<ProjectError>),
    #[error("live apply failed after durable journal commit")]
    PostDurableLiveApply(#[source] PostDurableLiveApplyError),
    #[error("transient primary reconcile failed after durable commit and live apply")]
    #[cfg(test)]
    ReconcileFailed,
    #[error(transparent)]
    Command(#[from] CommandError),
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::sync::Arc;

    use motolii_core::RationalTime;
    use motolii_doc::{
        journal_path_for_document, layer_names_for_item, motolii_dir_for_document, AssetId, Clip,
        ClipSource, Command, Document, DocumentError, DocumentPluginError, DocumentWriter,
        ItemEnvelope, LayerId, ParentLocator, ProjectSession, ResourceLimits, SaveProjectOptions,
        Track, TrackId, TrackItem,
    };
    use motolii_testkit::tmp_dir;

    use super::*;
    use crate::{
        adapt_document_command_request_error, builtin_command_registry, CommandId,
        DiagnosticReasonCode, DocumentCommandRequestError, InputRouter, InteractionState,
        InteractionStateMachine, NormalizedInput,
    };

    fn first_party_catalog() -> Arc<PluginCatalog> {
        Arc::new(motolii_plugins_firstparty::first_party_catalog().expect("first party catalog"))
    }

    use std::sync::atomic::{AtomicU64, Ordering};

    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_tmp(tag: &str) -> std::path::PathBuf {
        let id = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        tmp_dir(&format!("{tag}-{id}"))
    }

    fn persist_project(path: &std::path::Path, document: &Document) {
        let limits = ResourceLimits::production();
        let mut session = ProjectSession::acquire(path, &limits).expect("acquire");
        session
            .save_with_journal(
                document,
                &SaveProjectOptions {
                    limits,
                    checkpoint: true,
                    ..SaveProjectOptions::default()
                },
            )
            .expect("initial checkpoint");
    }

    fn open_runtime(document: Document) -> (std::path::PathBuf, DocumentEditRuntime) {
        let dir = unique_tmp("cu109-runtime-unit");
        let path = dir.join("proj.json");
        persist_project(&path, &document);
        let limits = ResourceLimits::production();
        let (session, opened) = ProjectSession::open(&path, &limits).expect("open");
        let catalog = first_party_catalog();
        let writer = DocumentWriter::new(opened.document, Arc::clone(&catalog)).expect("writer");
        let runtime = DocumentEditRuntime::new(session, writer, catalog);
        (path, runtime)
    }

    fn fixture() -> (Document, DocumentCommandRequest) {
        let mut document = Document::new_current();
        let layer = document.layers.allocate("fixture").unwrap();
        let track = document.track_ids.allocate("V1").unwrap();
        let asset = document
            .assets
            .allocate("media", "video/mp4", "hash")
            .unwrap();
        let item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        });
        document.tracks.push(Track {
            id: track,
            items: vec![item.clone()],
        });
        document.validate().unwrap();
        let request = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 0,
                layer_names: layer_names_for_item(&document, &item).unwrap(),
                item,
            }],
        )
        .unwrap();
        (document, request)
    }

    fn interval_sequence_fixture() -> (Document, Vec<LayerId>) {
        let mut document = Document::new_current();
        let track = document.track_ids.allocate("V1").unwrap();
        let asset = document
            .assets
            .allocate("media", "video/mp4", "hash")
            .unwrap();
        let mut layers = Vec::new();
        let mut items = Vec::new();
        for (index, start) in [0_i64, 2, 4].into_iter().enumerate() {
            let layer = document
                .layers
                .allocate(format!("fixture-{index}"))
                .unwrap();
            layers.push(layer);
            items.push(TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::try_new(start, 1).unwrap(),
                duration: RationalTime::try_new(1, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }));
        }
        document.tracks.push(Track { id: track, items });
        document.validate().unwrap();
        (document, layers)
    }

    fn clip_by_layer(document: &Document, layer: LayerId) -> &Clip {
        document
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .find_map(|item| match item {
                TrackItem::Clip(clip) if clip.envelope.layer_id == layer => Some(clip),
                _ => None,
            })
            .unwrap_or_else(|| panic!("fixture clip {layer:?} not found"))
    }

    struct TwoTrackFixture {
        document: Document,
        delete_request: DocumentCommandRequest,
        surviving: LayerId,
        deleted: LayerId,
        surviving_track: TrackId,
        asset: AssetId,
    }

    fn two_track_fixture() -> TwoTrackFixture {
        let mut document = Document::new_current();
        let surviving = document.layers.allocate("surviving").unwrap();
        let deleted = document.layers.allocate("deleted").unwrap();
        let surviving_track = document.track_ids.allocate("V1").unwrap();
        let deleted_track = document.track_ids.allocate("V2").unwrap();
        let asset = document
            .assets
            .allocate("media", "video/mp4", "hash")
            .unwrap();

        let surviving_item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(surviving),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        });
        document.tracks.push(Track {
            id: surviving_track,
            items: vec![surviving_item.clone()],
        });

        let deleted_item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(deleted),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        });
        document.tracks.push(Track {
            id: deleted_track,
            items: vec![deleted_item.clone()],
        });
        document.validate().unwrap();

        let delete_request = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(deleted_track),
                index: 0,
                layer_names: layer_names_for_item(&document, &deleted_item).unwrap(),
                item: deleted_item,
            }],
        )
        .unwrap();

        TwoTrackFixture {
            document,
            delete_request,
            surviving,
            deleted,
            surviving_track,
            asset,
        }
    }

    fn fixture_layer(document: &Document) -> LayerId {
        match &document.tracks[0].items[0] {
            TrackItem::Clip(clip) => clip.envelope.layer_id,
            _ => panic!("fixtureはclipを含む前提"),
        }
    }

    fn two_track_delete_request(document: &Document, deleted: LayerId) -> DocumentCommandRequest {
        let mut deleted_track = None;
        let mut deleted_item = None;

        'search: for track in &document.tracks {
            for item in &track.items {
                if let TrackItem::Clip(clip) = item {
                    if clip.envelope.layer_id == deleted {
                        deleted_track = Some(track.id);
                        deleted_item = Some(item.clone());
                        break 'search;
                    }
                }
            }
        }

        let deleted_track = deleted_track.unwrap_or_else(|| panic!("deleted track not found"));
        let deleted_item = deleted_item.unwrap_or_else(|| panic!("deleted item not found"));
        let layer_names = layer_names_for_item(document, &deleted_item).unwrap();

        DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(deleted_track),
                index: 0,
                layer_names,
                item: deleted_item,
            }],
        )
        .unwrap()
    }

    fn delete_output() -> RouterOutput {
        let mut router = InputRouter::new(builtin_command_registry().unwrap());
        router
            .route(NormalizedInput::Command {
                phase: InputPhase::Click,
                id: CommandId::try_new("motolii.edit.delete_targeted_items").unwrap(),
            })
            .unwrap()
    }

    fn history_output(intent: DomainIntent) -> RouterOutput {
        let id = match intent {
            DomainIntent::Undo => "motolii.edit.undo",
            DomainIntent::Redo => "motolii.edit.redo",
            _ => panic!("history output requires Undo or Redo"),
        };
        let mut router = InputRouter::new(builtin_command_registry().unwrap());
        router
            .route(NormalizedInput::Command {
                phase: InputPhase::Press,
                id: CommandId::try_new(id).unwrap(),
            })
            .unwrap()
    }

    fn assert_preflight_rejection_invariants(
        runtime: &DocumentEditRuntime,
        queue: &DocumentEditQueue,
        initial_json: &[u8],
        pre_revision: u64,
        pre_history: (usize, usize),
    ) {
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), pre_revision);
        assert_eq!(runtime.history_lengths(), pre_history);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        assert!(!runtime.is_poisoned());
    }

    fn dangling_parent_after_remove_fixture() -> (Document, DocumentCommandRequest) {
        let mut document = Document::new_current();
        let deleted_layer = document.layers.allocate("deleted-parent").unwrap();
        let survivor_layer = document.layers.allocate("survivor").unwrap();
        let track = document.track_ids.allocate("V1").unwrap();
        let asset = document
            .assets
            .allocate("media", "video/mp4", "hash")
            .unwrap();

        let deleted_item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(deleted_layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        });

        let mut survivor_envelope = ItemEnvelope::new(survivor_layer);
        survivor_envelope.transform.parent = Some(deleted_layer);
        let survivor_item = TrackItem::Clip(Clip {
            envelope: survivor_envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        });

        document.tracks.push(Track {
            id: track,
            items: vec![deleted_item.clone(), survivor_item],
        });
        document.validate().unwrap();

        let request = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 0,
                layer_names: layer_names_for_item(&document, &deleted_item).unwrap(),
                item: deleted_item,
            }],
        )
        .unwrap();

        (document, request)
    }

    fn open_runtime_with_catalog(
        document: Document,
        catalog: Arc<PluginCatalog>,
    ) -> (std::path::PathBuf, DocumentEditRuntime) {
        let dir = unique_tmp("cu109-runtime-unit");
        let path = dir.join("proj.json");
        persist_project(&path, &document);
        let limits = ResourceLimits::production();
        let (session, opened) = ProjectSession::open(&path, &limits).expect("open");
        let writer_catalog = first_party_catalog();
        let writer =
            DocumentWriter::new(opened.document, Arc::clone(&writer_catalog)).expect("writer");
        let runtime = DocumentEditRuntime::new(session, writer, catalog);
        (path, runtime)
    }

    #[test]
    fn apply_publishes_once_and_drains_queue() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let initial_snapshot = runtime.snapshot();
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let applied = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
        let applied_json = serde_json::to_vec(&*applied.snapshot).unwrap();
        assert_eq!(applied.kind, DocumentEditActionKind::Apply);
        assert_eq!(applied.revision, 1);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_ne!(applied_json, initial_json);
        assert_eq!(
            serde_json::to_vec(&*initial_snapshot).unwrap(),
            initial_json
        );
        assert_eq!(queue.len(), 0);
        assert_eq!(applied.projection_generation, 1);
    }

    #[test]
    fn undo_and_redo_commit_durably_and_publish_once_each() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();
        let applied = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
        let post_apply_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();

        queue.push_undo();
        let undone = runtime.process_next(&mut queue, None, 1).unwrap().unwrap();
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.revision, 2);
        assert_eq!(undone.projection_generation, 2);
        assert_eq!(serde_json::to_vec(&*undone.snapshot).unwrap(), initial_json);
        assert_eq!(runtime.history_lengths(), (0, 1));
        assert_eq!(queue.len(), 0);

        queue.push_redo();
        let redone = runtime.process_next(&mut queue, None, 2).unwrap().unwrap();
        assert_eq!(redone.kind, DocumentEditActionKind::Redo);
        assert_eq!(redone.revision, 3);
        assert_eq!(redone.projection_generation, 3);
        assert_eq!(
            serde_json::to_vec(&*redone.snapshot).unwrap(),
            post_apply_json
        );
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_eq!(applied.revision, 1);

        drop(runtime);
        let limits = ResourceLimits::production();
        let (_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
        assert_eq!(
            serde_json::to_vec(&reopened.document).unwrap(),
            post_apply_json
        );
    }

    #[test]
    fn attach_effect_commits_one_catalog_recipe_and_roundtrips_history() {
        let (document, _) = fixture();
        let primary = fixture_layer(&document);
        let initial_stable = document.next_stable_id.peek_next();
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        let journal_before = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
        let mut queue = DocumentEditQueue::default();
        queue.push_attach_effect(AttachEffectRequest {
            plugin_id: "core.filter.opacity".into(),
        });

        let attached = runtime
            .process_next(&mut queue, Some(primary), 4)
            .unwrap()
            .expect("validated attach must publish once");
        let created = attached
            .created_effect_use
            .expect("attach publish must identify the created Effect Use");
        assert_eq!(attached.kind, DocumentEditActionKind::AttachEffect);
        assert_eq!(attached.revision, 1);
        assert_eq!(attached.primary, Some(primary));
        assert_eq!(attached.projection_generation, 5);
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert!(fs::metadata(&journal).expect("journal").len() > journal_before);

        let envelope = runtime
            .writer
            .find_envelope(primary)
            .expect("primary envelope");
        assert_eq!(envelope.effects.len(), 1);
        assert_eq!(envelope.effects[0].id, created);
        let definition_id = envelope.effects[0].definition_id;
        let definition = attached
            .snapshot
            .effect_definition(definition_id)
            .expect("created definition");
        assert_eq!(definition.plugin_id, "core.filter.opacity");
        assert_eq!(definition.effect_version, 1);
        assert!(definition.enabled);
        assert_eq!(
            definition.params,
            BTreeMap::from([("amount".into(), motolii_doc::DocParam::const_f64(1.0))])
        );
        assert!(definition.extra.is_empty());
        assert_eq!(attached.snapshot.effect_definitions.len(), 1);
        assert_eq!(
            attached.snapshot.next_stable_id.peek_next(),
            initial_stable + 2
        );
        let attached_json = serde_json::to_vec(&*attached.snapshot).unwrap();

        queue.push_undo();
        let undone = runtime
            .process_next(&mut queue, Some(primary), 5)
            .unwrap()
            .expect("Undo must publish");
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.created_effect_use, None);
        assert_eq!(undone.primary, Some(primary));
        assert!(runtime
            .writer
            .find_envelope(primary)
            .expect("primary after Undo")
            .effects
            .is_empty());
        assert!(undone.snapshot.effect_definition(definition_id).is_none());
        assert_eq!(runtime.history_lengths(), (0, 1));

        queue.push_redo();
        let redone = runtime
            .process_next(&mut queue, Some(primary), 6)
            .unwrap()
            .expect("Redo must publish");
        assert_eq!(redone.kind, DocumentEditActionKind::Redo);
        assert_eq!(redone.created_effect_use, Some(created));
        assert_eq!(redone.primary, Some(primary));
        let redone_envelope = runtime
            .writer
            .find_envelope(primary)
            .expect("primary after Redo");
        assert_eq!(redone_envelope.effects.len(), 1);
        assert_eq!(redone_envelope.effects[0].id, created);
        assert_eq!(redone_envelope.effects[0].definition_id, definition_id);
        assert_eq!(
            serde_json::to_vec(&*redone.snapshot).unwrap(),
            attached_json
        );
        assert_eq!(runtime.history_lengths(), (1, 0));

        queue.push_undo();
        let undone_for_foreign_primary = runtime
            .process_next(&mut queue, Some(primary), 7)
            .unwrap()
            .expect("Undo must publish before foreign-primary Redo");
        assert_eq!(undone_for_foreign_primary.created_effect_use, None);

        queue.push_redo();
        let foreign_primary_redo = runtime
            .process_next(&mut queue, None, 8)
            .unwrap()
            .expect("foreign-primary Redo must still publish the Document");
        assert_eq!(foreign_primary_redo.created_effect_use, None);
        assert_eq!(foreign_primary_redo.primary, None);

        let request = SetEffectParamRequest::new(
            primary,
            created,
            definition_id,
            "core.filter.opacity".into(),
            1,
            "amount".into(),
            0.4,
        );
        queue.push_set_effect_param(request);
        let changed = runtime
            .process_next(&mut queue, Some(primary), 9)
            .unwrap()
            .expect("SetEffectParam must publish");
        assert_eq!(changed.kind, DocumentEditActionKind::SetEffectParam);
        assert_eq!(changed.created_effect_use, None);

        queue.push_undo();
        let undone_param = runtime
            .process_next(&mut queue, Some(primary), 10)
            .unwrap()
            .expect("undo param must publish");
        assert_eq!(undone_param.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone_param.created_effect_use, None);

        queue.push_redo();
        let redo_param = runtime
            .process_next(&mut queue, Some(primary), 11)
            .unwrap()
            .expect("redo param must publish");
        assert_eq!(redo_param.kind, DocumentEditActionKind::Redo);
        assert_eq!(redo_param.created_effect_use, None);

        queue.push_undo();
        let undo_param_restored = runtime
            .process_next(&mut queue, Some(primary), 12)
            .unwrap()
            .expect("undo param must publish");
        assert_eq!(undo_param_restored.kind, DocumentEditActionKind::Undo);
        assert_eq!(undo_param_restored.created_effect_use, None);

        drop(runtime);
        let limits = ResourceLimits::production();
        let (_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
        assert_eq!(
            serde_json::to_vec(&reopened.document).unwrap(),
            attached_json
        );
    }

    #[test]
    fn set_effect_param_commits_once_replay_is_noop_and_undo_restores_live_value() {
        let (document, _) = fixture();
        let primary = fixture_layer(&document);
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        let mut queue = DocumentEditQueue::default();
        queue.push_attach_effect(AttachEffectRequest {
            plugin_id: "core.filter.opacity".into(),
        });
        let attached = runtime
            .process_next(&mut queue, Some(primary), 0)
            .unwrap()
            .expect("opacity attach must publish");
        let effect_use_id = attached.created_effect_use.expect("effect use");
        let definition_id = attached
            .snapshot
            .find_effect_use(primary, effect_use_id)
            .expect("attached effect")
            .definition_id;
        let request = SetEffectParamRequest::new(
            primary,
            effect_use_id,
            definition_id,
            "core.filter.opacity".into(),
            1,
            "amount".into(),
            0.4,
        );
        queue.push_set_effect_param(request.clone());

        let changed = runtime
            .process_next(&mut queue, Some(primary), 1)
            .unwrap()
            .expect("changed release must publish once");
        assert_eq!(changed.kind, DocumentEditActionKind::SetEffectParam);
        assert_eq!(changed.revision, 2);
        assert_eq!(changed.projection_generation, 2);
        assert_eq!(runtime.history_lengths(), (2, 0));
        assert_eq!(
            changed
                .snapshot
                .effect_definition(definition_id)
                .unwrap()
                .params
                .get("amount"),
            Some(&motolii_doc::DocParam::const_f64(0.4))
        );

        let changed_json = serde_json::to_vec(&*changed.snapshot).unwrap();
        let journal_size = fs::metadata(&journal).unwrap().len();
        queue.push_set_effect_param(request);
        assert!(runtime
            .process_next(&mut queue, Some(primary), u64::MAX)
            .unwrap()
            .is_none());
        assert_eq!(runtime.revision(), 2);
        assert_eq!(runtime.history_lengths(), (2, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            changed_json
        );
        assert_eq!(fs::metadata(&journal).unwrap().len(), journal_size);

        queue.push_undo();
        let undone = runtime
            .process_next(&mut queue, Some(primary), 2)
            .unwrap()
            .expect("one Undo must restore the live old value");
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(runtime.history_lengths(), (1, 1));
        assert_eq!(
            undone
                .snapshot
                .effect_definition(definition_id)
                .unwrap()
                .params
                .get("amount"),
            Some(&motolii_doc::DocParam::const_f64(1.0))
        );
    }

    #[test]
    fn set_effect_param_stale_or_invalid_requests_write_nothing() {
        let (document, _) = fixture();
        let primary = fixture_layer(&document);
        let (path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_attach_effect(AttachEffectRequest {
            plugin_id: "core.filter.opacity".into(),
        });
        let attached = runtime
            .process_next(&mut queue, Some(primary), 0)
            .unwrap()
            .unwrap();
        let effect_use_id = attached.created_effect_use.unwrap();
        let definition_id = attached
            .snapshot
            .find_effect_use(primary, effect_use_id)
            .unwrap()
            .definition_id;
        let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
        let initial_history = runtime.history_lengths();
        let initial_revision = runtime.revision();
        let journal = journal_path_for_document(&path);
        let initial_journal_size = fs::metadata(&journal).unwrap().len();

        for request in [
            SetEffectParamRequest::new(
                LayerId::from_raw(u64::MAX),
                effect_use_id,
                definition_id,
                "core.filter.opacity".into(),
                1,
                "amount".into(),
                0.3,
            ),
            SetEffectParamRequest::new(
                primary,
                effect_use_id,
                definition_id,
                "core.filter.opacity".into(),
                2,
                "amount".into(),
                0.3,
            ),
            SetEffectParamRequest::new(
                primary,
                effect_use_id,
                definition_id,
                "core.filter.opacity".into(),
                1,
                "amount".into(),
                f64::NAN,
            ),
        ] {
            queue.push_set_effect_param(request);
            assert!(runtime
                .process_next(&mut queue, Some(primary), u64::MAX)
                .unwrap()
                .is_none());
            assert_eq!(runtime.revision(), initial_revision);
            assert_eq!(runtime.history_lengths(), initial_history);
            assert_eq!(
                serde_json::to_vec(&*runtime.snapshot()).unwrap(),
                initial_json
            );
            assert_eq!(fs::metadata(&journal).unwrap().len(), initial_journal_size);
        }
    }

    #[test]
    fn attach_effect_preflight_rejections_write_nothing() {
        fn assert_unchanged(
            runtime: &DocumentEditRuntime,
            queue: &DocumentEditQueue,
            initial_json: &[u8],
            initial_stable: u64,
            journal: &std::path::Path,
            journal_size: u64,
        ) {
            assert_preflight_rejection_invariants(runtime, queue, initial_json, 0, (0, 0));
            assert_eq!(
                runtime.snapshot().next_stable_id.peek_next(),
                initial_stable
            );
            assert_eq!(
                fs::metadata(journal).map(|meta| meta.len()).unwrap_or(0),
                journal_size
            );
        }

        for primary in [None, Some(LayerId::from_raw(u64::MAX))] {
            let (document, _) = fixture();
            let initial_json = serde_json::to_vec(&document).unwrap();
            let initial_stable = document.next_stable_id.peek_next();
            let (path, mut runtime) = open_runtime(document);
            let journal = journal_path_for_document(&path);
            let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
            let mut queue = DocumentEditQueue::default();
            queue.push_attach_effect(AttachEffectRequest {
                plugin_id: "core.filter.opacity".into(),
            });

            assert!(runtime
                .process_next(&mut queue, primary, u64::MAX)
                .unwrap()
                .is_none());
            assert_unchanged(
                &runtime,
                &queue,
                &initial_json,
                initial_stable,
                &journal,
                journal_size,
            );
        }

        for (plugin_id, expected) in [
            ("missing.filter", "missing"),
            ("core.layer_source.radial_repeater", "kind"),
        ] {
            let (document, _) = fixture();
            let primary = fixture_layer(&document);
            let initial_json = serde_json::to_vec(&document).unwrap();
            let initial_stable = document.next_stable_id.peek_next();
            let (path, mut runtime) = open_runtime(document);
            let journal = journal_path_for_document(&path);
            let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
            let mut queue = DocumentEditQueue::default();
            queue.push_attach_effect(AttachEffectRequest {
                plugin_id: plugin_id.into(),
            });

            let error = runtime
                .process_next(&mut queue, Some(primary), 0)
                .expect_err("invalid contract must fail before commit");
            match expected {
                "missing" => assert!(matches!(
                    error,
                    DocumentEditRuntimeError::DocumentPlugin(
                        DocumentPluginError::ContractMissing { .. }
                    )
                )),
                "kind" => assert!(matches!(
                    error,
                    DocumentEditRuntimeError::DocumentPlugin(
                        DocumentPluginError::KindMismatch { .. }
                    )
                )),
                _ => unreachable!(),
            }
            assert_unchanged(
                &runtime,
                &queue,
                &initial_json,
                initial_stable,
                &journal,
                journal_size,
            );
        }

        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let initial_stable = document.next_stable_id.peek_next();
        let (path, runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
        let non_const = PreparedPluginRecipe {
            plugin_id: "core.filter.opacity".into(),
            saved_version: 1,
            current_version: 1,
            params: BTreeMap::from([(
                "amount".into(),
                motolii_doc::DocParam::Keyframes(motolii_doc::DocKeyframeTrack::new()),
            )]),
        };
        assert!(matches!(
            attach_effect_draft(non_const),
            Err(DocumentEditRuntimeError::AttachDefaultNotConst { ref param })
                if param == "amount"
        ));
        assert_unchanged(
            &runtime,
            &DocumentEditQueue::default(),
            &initial_json,
            initial_stable,
            &journal,
            journal_size,
        );
    }

    #[test]
    fn routed_history_commands_reconcile_selection_without_restoring_it_on_redo() {
        let (document, _) = fixture();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_place_rectangle(PlaceRectangleRequest {
            position: [0.0, 0.0],
            playhead: RationalTime::ZERO,
        });
        let placed = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
        let selected = placed.primary.expect("Place selects the new Rectangle");

        queue
            .push_prepared(history_output(DomainIntent::Undo), None)
            .unwrap();
        let undone = runtime
            .process_next(&mut queue, Some(selected), 1)
            .unwrap()
            .unwrap();
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.primary, None);
        assert!(runtime.writer.find_envelope(selected).is_none());

        queue
            .push_prepared(history_output(DomainIntent::Redo), None)
            .unwrap();
        let redone = runtime
            .process_next(&mut queue, undone.primary, 2)
            .unwrap()
            .unwrap();
        assert_eq!(redone.kind, DocumentEditActionKind::Redo);
        assert_eq!(redone.primary, None);
        assert!(runtime.writer.find_envelope(selected).is_some());
        assert_eq!(runtime.history_lengths(), (1, 0));
    }

    #[test]
    fn missing_request_and_empty_history_publish_nothing() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();

        assert_eq!(
            queue.push_prepared(delete_output(), None),
            Err(DocumentEditDispatchError::MissingPreparedRequest)
        );
        assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());

        queue.push_undo();
        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::NothingToUndo)
        ));
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn failed_d2_action_is_consumed_without_snapshot_or_history_change() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let track = document.tracks[0].id;
        let item = document.tracks[0].items[0].clone();
        let request = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 1,
                layer_names: layer_names_for_item(&document, &item).unwrap(),
                item,
            }],
        )
        .unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let pre_revision = runtime.revision();
        let pre_history = runtime.history_lengths();
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::Command(_))
        ));
        assert_preflight_rejection_invariants(
            &runtime,
            &queue,
            &initial_json,
            pre_revision,
            pre_history,
        );
    }

    #[test]
    fn failed_validate_is_consumed_without_snapshot_or_history_change() {
        let (document, request) = dangling_parent_after_remove_fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let pre_revision = runtime.revision();
        let pre_history = runtime.history_lengths();
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let error = runtime
            .process_next(&mut queue, None, 0)
            .expect_err("dangling transform.parent must fail validate");
        assert!(matches!(
            error,
            DocumentEditRuntimeError::Document(DocumentError::UnknownLayerId { .. })
        ));
        assert_preflight_rejection_invariants(
            &runtime,
            &queue,
            &initial_json,
            pre_revision,
            pre_history,
        );
    }

    fn plugin_clip_survivor_fixture() -> (Document, DocumentCommandRequest) {
        let mut document = Document::new_current();
        let removable = document.layers.allocate("removable").unwrap();
        let plugin_layer = document.layers.allocate("plugin-survivor").unwrap();
        let track = document.track_ids.allocate("V1").unwrap();
        let asset = document
            .assets
            .allocate("media", "video/mp4", "hash")
            .unwrap();

        let removable_item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(removable),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        });

        let plugin_item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(plugin_layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: "core.layer_source.radial_repeater".into(),
                effect_version: 1,
                params: BTreeMap::from([("count".into(), motolii_doc::DocParam::const_f64(12.0))]),
                extra: Default::default(),
            },
        });

        document.tracks.push(Track {
            id: track,
            items: vec![removable_item.clone(), plugin_item],
        });
        document.validate().unwrap();

        let request = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 0,
                layer_names: layer_names_for_item(&document, &removable_item).unwrap(),
                item: removable_item,
            }],
        )
        .unwrap();

        (document, request)
    }

    #[test]
    fn failed_prepare_plugins_is_consumed_without_snapshot_or_history_change() {
        use motolii_plugin::PluginCatalogBuilder;

        let (document, request) = plugin_clip_survivor_fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        // CU-109 製品配線では writer と runtime が同一 catalog を共有するため到達不能。
        // prepare_plugins 前方ガードの単体試験としてだけ catalog 不一致を runtime に渡す。
        let mut mismatched_contract = first_party_catalog()
            .get("core.layer_source.radial_repeater")
            .expect("radial repeater contract")
            .clone();
        mismatched_contract
            .node
            .params
            .retain(|param| param.id != "count");
        let mut builder = PluginCatalogBuilder::new();
        builder
            .register(mismatched_contract)
            .expect("mismatched test contract");
        let mismatched_catalog = Arc::new(builder.build().expect("mismatched catalog"));
        let (_path, mut runtime) = open_runtime_with_catalog(document, mismatched_catalog);
        let pre_revision = runtime.revision();
        let pre_history = runtime.history_lengths();
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let error = runtime
            .process_next(&mut queue, None, 0)
            .expect_err("mismatched runtime catalog must fail prepare_plugins");
        assert!(matches!(
            error,
            DocumentEditRuntimeError::DocumentPlugin(
                DocumentPluginError::ContractViolation { ref param, .. }
            ) if param == "count"
        ));
        assert_preflight_rejection_invariants(
            &runtime,
            &queue,
            &initial_json,
            pre_revision,
            pre_history,
        );
    }

    #[test]
    fn empty_queue_publishes_nothing_and_does_not_advance_generation() {
        let (document, _) = fixture();
        let layer = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let pre_revision = runtime.revision();
        let mut queue = DocumentEditQueue::default();

        assert!(runtime
            .process_next(&mut queue, Some(layer), 5)
            .unwrap()
            .is_none());
        assert_eq!(runtime.revision(), pre_revision);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn diagnostic_adaptation_does_not_enqueue_or_mutate_document_runtime() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();

        let envelope =
            adapt_document_command_request_error(&DocumentCommandRequestError::EmptyCommands);

        assert_eq!(
            envelope.reason(),
            DiagnosticReasonCode::EmptyDocumentCommands
        );
        assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn target_and_preview_cancel_leave_edit_and_render_delivery_unchanged() {
        for with_preview in [false, true] {
            let (document, _) = fixture();
            let initial_json = serde_json::to_vec(&document).unwrap();
            let (_path, mut runtime) = open_runtime(document);
            let mut queue = DocumentEditQueue::default();
            let mut machine = InteractionStateMachine::new();
            machine.transition(InteractionState::Target).unwrap();
            if with_preview {
                machine.transition(InteractionState::Preview).unwrap();
            }

            machine.transition(InteractionState::Cancel).unwrap();
            machine.transition(InteractionState::Discover).unwrap();

            assert!(runtime.process_next(&mut queue, None, 0).unwrap().is_none());
            assert_eq!(queue.len(), 0);
            assert_eq!(runtime.revision(), 0);
            assert_eq!(runtime.history_lengths(), (0, 0));
            assert_eq!(
                serde_json::to_vec(&*runtime.snapshot()).unwrap(),
                initial_json
            );
        }
    }

    #[test]
    fn apply_success_retains_valid_primary_and_advances_generation_once() {
        let f = two_track_fixture();
        let surviving = f.surviving;
        let (_path, mut runtime) = open_runtime(f.document);
        let mut queue = DocumentEditQueue::default();
        queue
            .push_prepared(delete_output(), Some(f.delete_request))
            .unwrap();

        let applied = runtime
            .process_next(&mut queue, Some(surviving), 7)
            .unwrap()
            .unwrap();
        assert_eq!(applied.kind, DocumentEditActionKind::Apply);
        assert_eq!(applied.revision, 1);
        assert_eq!(applied.primary, Some(surviving));
        assert_eq!(applied.projection_generation, 8);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn apply_success_clears_primary_deleted_by_the_apply() {
        let (document, request) = fixture();
        let deleted = fixture_layer(&document);
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let applied = runtime
            .process_next(&mut queue, Some(deleted), 7)
            .unwrap()
            .unwrap();
        assert_eq!(applied.kind, DocumentEditActionKind::Apply);
        assert_eq!(applied.revision, 1);
        assert_eq!(applied.primary, None);
        assert_eq!(applied.projection_generation, 8);
        assert!(runtime.writer.find_envelope(deleted).is_none());
        assert_eq!(runtime.history_lengths(), (1, 0));
    }

    #[test]
    fn generation_at_max_minus_one_advances_to_max() {
        let (document, request) = fixture();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        let published = runtime
            .process_next(&mut queue, None, u64::MAX - 1)
            .unwrap()
            .unwrap();
        assert_eq!(published.kind, DocumentEditActionKind::Apply);
        assert_eq!(published.projection_generation, u64::MAX);
    }

    #[test]
    fn exhausted_generation_refuses_before_mutation_and_consumes_action() {
        let f = two_track_fixture();
        let (_path, mut runtime) = open_runtime(f.document.clone());
        let added_layer = runtime.writer.reserve_layer_id().unwrap();
        let item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(added_layer),
            start: RationalTime::try_new(1, 1).unwrap(),
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(f.asset),
        });
        let mut layer_names = BTreeMap::new();
        layer_names.insert(added_layer, "seeded".to_string());
        runtime
            .writer
            .apply_macro(vec![Command::AddTrackItem {
                parent: ParentLocator::Track(f.surviving_track),
                index: 1,
                item,
                layer_names,
            }])
            .unwrap();
        assert_eq!(runtime.revision(), 1);

        for action in ["apply", "undo"] {
            let mut queue = DocumentEditQueue::default();
            match action {
                "apply" => queue
                    .push_prepared(
                        delete_output(),
                        Some(two_track_delete_request(&f.document, f.deleted)),
                    )
                    .unwrap(),
                "undo" => queue.push_undo(),
                _ => unreachable!(),
            }

            let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
            let pre_revision = runtime.revision();
            let pre_history = runtime.history_lengths();

            match runtime.process_next(&mut queue, Some(added_layer), u64::MAX) {
                Err(DocumentEditRuntimeError::ProjectionGenerationExhausted) => {}
                result => panic!("unexpected result: {result:?}"),
            }
            assert_eq!(queue.len(), 0);
            assert_eq!(runtime.revision(), pre_revision);
            assert_eq!(runtime.history_lengths(), pre_history);
            assert_eq!(
                serde_json::to_vec(&*runtime.snapshot()).unwrap(),
                initial_json
            );
        }
    }

    #[test]
    fn remaining_queue_drains_after_exhaustion_refusal() {
        let f = two_track_fixture();
        let (_path, mut runtime) = open_runtime(f.document.clone());
        let added_layer = runtime.writer.reserve_layer_id().unwrap();
        let item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(added_layer),
            start: RationalTime::try_new(1, 1).unwrap(),
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(f.asset),
        });
        let mut layer_names = BTreeMap::new();
        layer_names.insert(added_layer, "seeded".to_string());
        runtime
            .writer
            .apply_macro(vec![Command::AddTrackItem {
                parent: ParentLocator::Track(f.surviving_track),
                index: 1,
                item,
                layer_names,
            }])
            .unwrap();

        let mut queue = DocumentEditQueue::default();
        queue
            .push_prepared(
                delete_output(),
                Some(two_track_delete_request(&f.document, f.deleted)),
            )
            .unwrap();
        queue.push_undo();

        assert!(matches!(
            runtime.process_next(&mut queue, None, u64::MAX),
            Err(DocumentEditRuntimeError::ProjectionGenerationExhausted)
        ));
        assert_eq!(queue.len(), 1);

        assert!(matches!(
            runtime.process_next(&mut queue, None, 4),
            Err(DocumentEditRuntimeError::HistoryProjectionMismatch)
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 1);
    }

    #[test]
    fn multi_command_apply_is_rejected_pre_mutation() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let track = document.tracks[0].id;
        let item = document.tracks[0].items[0].clone();
        let request = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![
                Command::RemoveTrackItem {
                    parent: ParentLocator::Track(track),
                    index: 0,
                    layer_names: layer_names_for_item(&document, &item).unwrap(),
                    item: item.clone(),
                },
                Command::RemoveTrackItem {
                    parent: ParentLocator::Track(track),
                    index: 0,
                    layer_names: layer_names_for_item(&document, &item).unwrap(),
                    item,
                },
            ],
        )
        .unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::MultiCommandActionRejected)
        ));
        assert_eq!(runtime.revision(), 0);
        assert!(!runtime.is_poisoned());
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn journal_commit_failure_poisons_without_live_apply() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        drop(fs::remove_file(&journal));
        fs::create_dir_all(&journal).unwrap();

        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::JournalCommit(_))
        ));
        assert!(runtime.is_poisoned());
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(queue.len(), 0);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn post_durable_live_apply_failure_poisons_and_leaves_journal_durable() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (path, mut runtime) = open_runtime(document);
        runtime.set_test_failpoint(RuntimeTestFailpoint::PostDurableLiveApply);

        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::PostDurableLiveApply(
                PostDurableLiveApplyError::InjectedInvariant
            ))
        ));
        assert!(runtime.is_poisoned());
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(queue.len(), 0);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );

        drop(runtime);
        let limits = ResourceLimits::production();
        let (_session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
        assert!(opened.document.tracks[0].items.is_empty());
    }

    #[test]
    fn reconcile_failure_poisons_without_publish() {
        let (document, request) = fixture();
        let (path, mut runtime) = open_runtime(document);
        runtime.set_test_failpoint(RuntimeTestFailpoint::Reconcile);

        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::ReconcileFailed)
        ));
        assert!(runtime.is_poisoned());
        assert_eq!(runtime.revision(), 1);
        assert_eq!(queue.len(), 0);
        let live_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();

        drop(runtime);
        let limits = ResourceLimits::production();
        let (_session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
        assert_eq!(serde_json::to_vec(&opened.document).unwrap(), live_json);
    }

    #[test]
    fn poisoned_runtime_does_not_consume_queue_or_mutate_writer() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        drop(fs::remove_file(&journal));
        fs::create_dir_all(&journal).unwrap();
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();
        let _ = runtime.process_next(&mut queue, None, 0);
        assert!(runtime.is_poisoned());
        assert_eq!(runtime.revision(), 0);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        let (_document2, request2) = fixture();
        queue
            .push_prepared(delete_output(), Some(request2))
            .unwrap();
        let initial_len = queue.len();
        let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
        let initial_revision = runtime.revision();
        let initial_history = runtime.history_lengths();
        let journal_entries_before = fs::read_dir(&journal).unwrap().count();

        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::SessionPoisoned)
        ));
        assert_eq!(queue.len(), initial_len);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        assert_eq!(runtime.revision(), initial_revision);
        assert_eq!(runtime.history_lengths(), initial_history);
        assert_eq!(
            fs::read_dir(&journal).unwrap().count(),
            journal_entries_before
        );
    }

    #[test]
    fn reopen_restores_healthy_runtime_after_poison() {
        let f = two_track_fixture();
        let (path, mut runtime) = open_runtime(f.document.clone());
        runtime.set_test_failpoint(RuntimeTestFailpoint::PostDurableLiveApply);
        let mut queue = DocumentEditQueue::default();
        queue
            .push_prepared(delete_output(), Some(f.delete_request))
            .unwrap();
        let _ = runtime.process_next(&mut queue, None, 0);
        assert!(runtime.is_poisoned());
        assert_eq!(runtime.revision(), 0);
        drop(runtime);

        let limits = ResourceLimits::production();
        let (session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
        let catalog = first_party_catalog();
        let writer = DocumentWriter::new(opened.document, Arc::clone(&catalog)).unwrap();
        let mut runtime = DocumentEditRuntime::new(session, writer, catalog);
        assert!(!runtime.is_poisoned());

        let mut queue = DocumentEditQueue::default();
        queue
            .push_prepared(
                delete_output(),
                Some(two_track_delete_request(
                    runtime.snapshot().as_ref(),
                    f.surviving,
                )),
            )
            .unwrap();
        let applied = runtime
            .process_next(&mut queue, None, 0)
            .unwrap()
            .expect("reopened runtime must accept Apply");
        assert_eq!(applied.kind, DocumentEditActionKind::Apply);
        assert_eq!(applied.revision, 1);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn duplicate_apply_after_success_is_rejected_at_preflight() {
        let (document, request) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let track = document.tracks[0].id;
        let item = document.tracks[0].items[0].clone();
        let layer_names = layer_names_for_item(&document, &item).unwrap();
        let (path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();
        let applied = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
        assert_eq!(applied.revision, 1);
        let post_apply_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
        assert_ne!(post_apply_json, initial_json);
        let journal = journal_path_for_document(&path);
        let journal_size = fs::metadata(&journal).unwrap().len();
        let request_again = DocumentCommandRequest::try_new(
            DomainIntent::DeleteTargetedItems,
            vec![Command::RemoveTrackItem {
                parent: ParentLocator::Track(track),
                index: 0,
                layer_names,
                item,
            }],
        )
        .unwrap();
        queue
            .push_prepared(delete_output(), Some(request_again))
            .unwrap();
        let pre_revision = runtime.revision();
        let pre_history = runtime.history_lengths();
        assert!(matches!(
            runtime.process_next(&mut queue, None, 1),
            Err(DocumentEditRuntimeError::Command(_))
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), pre_revision);
        assert_eq!(runtime.history_lengths(), pre_history);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            post_apply_json
        );
        assert_eq!(fs::metadata(&journal).unwrap().len(), journal_size);
        assert!(!runtime.is_poisoned());
    }

    #[test]
    fn journal_commit_precedes_live_apply_on_success() {
        let (document, request) = fixture();
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        let size_before = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);

        let mut queue = DocumentEditQueue::default();
        queue.push_prepared(delete_output(), Some(request)).unwrap();
        let published = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
        assert_eq!(published.revision, 1);
        let size_after = fs::metadata(&journal).expect("journal").len();
        assert!(size_after > size_before);
        assert_eq!(queue.len(), 0);
        assert_eq!(published.projection_generation, 1);
    }

    #[test]
    fn rectangle_place_uses_one_add_command_and_publishes_the_same_layer_id() {
        let (document, _) = fixture();
        let selected = fixture_layer(&document);
        let initial_next = document.layers.peek_next();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_place_rectangle(PlaceRectangleRequest {
            position: [0.25, -0.125],
            playhead: RationalTime::try_new(1, 1).unwrap(),
        });

        let published = runtime
            .process_next(&mut queue, Some(selected), 4)
            .unwrap()
            .expect("published Rectangle");
        let placed = published.primary.expect("placed selection receipt");
        assert_eq!(placed.get(), initial_next);
        assert_eq!(published.kind, DocumentEditActionKind::PlaceRectangle);
        assert_eq!(published.revision, 1);
        assert_eq!(published.projection_generation, 5);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_eq!(published.snapshot.tracks[0].items.len(), 2);
        let TrackItem::Clip(clip) = &published.snapshot.tracks[0].items[1] else {
            panic!("Rectangle must be a Clip");
        };
        assert_eq!(clip.envelope.layer_id, placed);
        assert_eq!(
            clip.envelope.transform.position,
            motolii_doc::DocParam::const_vec2([0.25, -0.125])
        );
        assert_eq!(clip.start, RationalTime::try_new(1, 1).unwrap());
        assert_eq!(
            clip.duration,
            published
                .snapshot
                .composition
                .duration
                .try_sub(clip.start)
                .unwrap()
        );
        assert!(matches!(
            clip.source,
            ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::StandardShape {
                        shape: StandardShape::Rect { .. }
                    },
                    ..
                }
            }
        ));
        assert_eq!(
            published.snapshot.layers.display_name(placed),
            Some("Rectangle")
        );
    }

    #[test]
    fn trim_clip_commits_through_the_shared_writer_and_roundtrips_history() {
        let (document, _) = fixture();
        let selected = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_trim_clip(TrimClipRequest {
            layer_id: selected,
            edge: TrimClipEdge::In,
            time: RationalTime::try_new(1, 4).unwrap(),
        });

        let trimmed = runtime
            .process_next(&mut queue, Some(selected), 4)
            .unwrap()
            .expect("changed trim must publish once");
        assert_eq!(trimmed.kind, DocumentEditActionKind::TrimClip);
        assert_eq!(trimmed.revision, 1);
        assert_eq!(trimmed.primary, Some(selected));
        assert_eq!(trimmed.projection_generation, 5);
        assert_eq!(runtime.history_lengths(), (1, 0));
        let TrackItem::Clip(clip) = &trimmed.snapshot.tracks[0].items[0] else {
            panic!("fixture must remain a Clip");
        };
        assert_eq!(clip.start, RationalTime::try_new(1, 4).unwrap());
        assert_eq!(clip.duration, RationalTime::try_new(3, 4).unwrap());

        queue.push_undo();
        let undone = runtime
            .process_next(&mut queue, Some(selected), 5)
            .unwrap()
            .expect("trim Undo must publish");
        assert_eq!(undone.kind, DocumentEditActionKind::Undo);
        assert_eq!(undone.revision, 2);
        assert_eq!(undone.primary, Some(selected));
        assert_eq!(undone.projection_generation, 6);
        assert_eq!(runtime.history_lengths(), (0, 1));
        assert_eq!(serde_json::to_vec(&*undone.snapshot).unwrap(), initial_json);

        queue.push_redo();
        let redone = runtime
            .process_next(&mut queue, Some(selected), 6)
            .unwrap()
            .expect("trim Redo must publish");
        assert_eq!(redone.kind, DocumentEditActionKind::Redo);
        assert_eq!(redone.revision, 3);
        assert_eq!(redone.primary, Some(selected));
        assert_eq!(redone.projection_generation, 7);
        assert_eq!(runtime.history_lengths(), (1, 0));
        assert_eq!(
            serde_json::to_vec(&*redone.snapshot).unwrap(),
            serde_json::to_vec(&*trimmed.snapshot).unwrap()
        );
    }

    #[test]
    fn unchanged_trim_is_a_noop_before_projection_or_history_changes() {
        let (document, _) = fixture();
        let selected = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_trim_clip(TrimClipRequest {
            layer_id: selected,
            edge: TrimClipEdge::In,
            time: RationalTime::ZERO,
        });

        assert!(runtime
            .process_next(&mut queue, Some(selected), u64::MAX)
            .unwrap()
            .is_none());
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn deterministic_move_trim_sequence_preserves_identity_and_full_undo_reopens_initial_bytes() {
        const STEPS: usize = 192;
        let (document, layers) = interval_sequence_fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let initial_ids = layers.iter().copied().collect::<BTreeSet<_>>();
        let (path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        let mut generation = 0_u64;
        let mut state = 0x4d59_5df4_d0f3_3173_u64;

        for step in 0..STEPS {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let layer = layers[(state as usize) % layers.len()];
            let before_snapshot = runtime.snapshot();
            let before = clip_by_layer(&before_snapshot, layer).clone();
            let delta = RationalTime::try_new(((state >> 16) % 5 + 1) as i64, 600).unwrap();

            match (state >> 8) % 3 {
                0 => {
                    let mut start =
                        RationalTime::try_new(((state >> 24) % 480) as i64, 60).unwrap();
                    if start == before.start {
                        start = start
                            .try_add(RationalTime::try_new(1, 60).unwrap())
                            .unwrap();
                    }
                    queue.push_move_clip(MoveClipRequest {
                        layer_id: layer,
                        start,
                    });
                }
                1 => queue.push_trim_clip(TrimClipRequest {
                    layer_id: layer,
                    edge: TrimClipEdge::In,
                    time: before.start.try_add(delta).unwrap(),
                }),
                _ => queue.push_trim_clip(TrimClipRequest {
                    layer_id: layer,
                    edge: TrimClipEdge::Out,
                    time: before
                        .start
                        .try_add(before.duration)
                        .unwrap()
                        .try_sub(delta)
                        .unwrap(),
                }),
            }

            let published = runtime
                .process_next(&mut queue, Some(layer), generation)
                .unwrap()
                .unwrap_or_else(|| panic!("step {step} must publish"));
            generation = published.projection_generation;
            let after = clip_by_layer(&published.snapshot, layer);
            let ids = published
                .snapshot
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .map(|item| match item {
                    TrackItem::Clip(clip) => clip.envelope.layer_id,
                    TrackItem::Group(group) => group.envelope.layer_id,
                })
                .collect::<BTreeSet<_>>();

            assert_eq!(ids, initial_ids, "step {step} changed Clip identity set");
            assert_eq!(published.primary, Some(layer));
            assert_eq!(queue.len(), 0);
            assert_eq!(runtime.history_lengths(), (step + 1, 0));
            if published.kind == DocumentEditActionKind::MoveClip {
                assert_eq!(after.duration, before.duration);
                assert_eq!(after.time_map, before.time_map);
                assert_eq!(after.source, before.source);
                assert_eq!(
                    after.start.try_add(after.duration).unwrap(),
                    before
                        .start
                        .try_add(before.duration)
                        .unwrap()
                        .try_add(after.start.try_sub(before.start).unwrap())
                        .unwrap(),
                    "step {step} did not translate both interval edges equally"
                );
            }
        }

        for remaining in (0..STEPS).rev() {
            queue.push_undo();
            let undone = runtime
                .process_next(&mut queue, None, generation)
                .unwrap()
                .unwrap_or_else(|| panic!("Undo {remaining} must publish"));
            generation = undone.projection_generation;
            assert_eq!(runtime.history_lengths(), (remaining, STEPS - remaining));
            assert_eq!(queue.len(), 0);
        }

        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        drop(runtime);
        let limits = ResourceLimits::production();
        let (_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
        assert_eq!(
            serde_json::to_vec(&reopened.document).unwrap(),
            initial_json
        );
    }

    #[test]
    fn rejected_rectangle_place_changes_no_document_counter_history_or_revision() {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let initial_next = document.layers.peek_next();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_place_rectangle(PlaceRectangleRequest {
            position: [f64::NAN, 0.0],
            playhead: RationalTime::ZERO,
        });
        assert!(matches!(
            runtime.process_next(&mut queue, None, 0),
            Err(DocumentEditRuntimeError::NonFiniteDropPosition)
        ));
        assert_eq!(runtime.snapshot().layers.peek_next(), initial_next);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn selection_actions_publish_only_accepted_primary_changes() {
        let (document, _) = fixture();
        let selected = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();

        queue.push_replace_primary(selected);
        let selected_publish = runtime
            .process_next(&mut queue, None, 7)
            .unwrap()
            .expect("live target selection publishes");
        assert_eq!(
            selected_publish.kind,
            DocumentEditActionKind::ReplacePrimary
        );
        assert_eq!(selected_publish.primary, Some(selected));
        assert_eq!(selected_publish.projection_generation, 8);
        assert_eq!(selected_publish.revision, 0);

        queue.push_replace_primary(selected);
        assert!(runtime
            .process_next(&mut queue, Some(selected), u64::MAX)
            .unwrap()
            .is_none());

        queue.push_clear_primary();
        let clear_publish = runtime
            .process_next(&mut queue, Some(selected), 8)
            .unwrap()
            .expect("non-empty primary clear publishes");
        assert_eq!(clear_publish.kind, DocumentEditActionKind::ClearPrimary);
        assert_eq!(clear_publish.primary, None);
        assert_eq!(clear_publish.projection_generation, 9);
        assert_eq!(clear_publish.revision, 0);

        queue.push_clear_primary();
        assert!(runtime
            .process_next(&mut queue, None, u64::MAX)
            .unwrap()
            .is_none());

        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn nonexistent_selection_rejects_before_same_id_or_generation_preflight() {
        let (mut document, _) = fixture();
        let table_only = document.layers.allocate("table-only").unwrap();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(table_only);

        assert!(matches!(
            runtime.process_next(&mut queue, Some(table_only), u64::MAX),
            Err(DocumentEditRuntimeError::SelectionTargetNotFound(target))
                if target == table_only
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn accepted_selection_change_rejects_generation_exhaustion_without_mutation() {
        let (document, _) = fixture();
        let selected = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let (_path, mut runtime) = open_runtime(document);
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(selected);

        assert!(matches!(
            runtime.process_next(&mut queue, None, u64::MAX),
            Err(DocumentEditRuntimeError::ProjectionGenerationExhausted)
        ));
        assert_eq!(queue.len(), 0);
        assert_eq!(runtime.revision(), 0);
        assert_eq!(runtime.history_lengths(), (0, 0));
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn sidecar_absent_before_initialized_project_open() {
        let dir = unique_tmp("cu109-no-sidecar");
        let path = dir.join("missing.json");
        let limits = ResourceLimits::production();
        assert!(ProjectSession::open(&path, &limits).is_err());
        assert!(!motolii_dir_for_document(&path).exists());
    }
}
