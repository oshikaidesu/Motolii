//! runtimeがcallerへ返す公開結果と失敗理由。

use std::sync::Arc;

use motolii_core::RationalTimeError;
use motolii_doc::{
    AddPositionKeyPrepareError, AddTransformParamKeyPrepareError, CommandError, Document,
    DocumentError, DocumentPluginError, DuplicateError, EffectId, JournalCommitReceipt, LayerId,
    LayerIdError, ProjectError, RemovePositionKeyPrepareError, UndoError,
};

use super::action::DocumentEditActionKind;

#[derive(Debug)]
pub(crate) struct PublishedDocument {
    pub(crate) kind: DocumentEditActionKind,
    pub(crate) revision: u64,
    /// writer の live snapshot。RN `read_snapshot` / dispatch はこれを投影する。
    pub(crate) snapshot: Arc<Document>,
    pub(crate) primary: Option<LayerId>,
    pub(crate) projection_generation: u64,
    pub(crate) created_effect_use: Option<EffectId>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DocumentEditRuntimeError {
    #[error("projection generation is exhausted at u64::MAX")]
    ProjectionGenerationExhausted,
    #[error("selection target does not exist in the live Document: {0:?}")]
    SelectionTargetNotFound(LayerId),
    #[error("user edit requires a primary selection")]
    NoPrimarySelection,
    #[error("prepared user edit was rejected")]
    PrepareRejected,
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
    #[error("media library file is unreadable")]
    LibraryFileUnreadable,
    #[error("prepared attach parameter `{param}` is not Const")]
    AttachDefaultNotConst { param: String },
    #[error("prepare_create_effect returned a non-CreateEffect command")]
    AttachPrepareCommandMismatch,
    #[error("prepare_set_position_key_value returned a non-value command")]
    PositionKeyPrepareMismatch,
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
    #[error(transparent)]
    PositionKeyPrepare(#[from] AddPositionKeyPrepareError),
    #[error(transparent)]
    TransformParamKeyPrepare(#[from] AddTransformParamKeyPrepareError),
    #[error(transparent)]
    RemovePositionKeyPrepare(#[from] RemovePositionKeyPrepareError),
    #[error("journal durable commit failed: {0}")]
    JournalCommit(#[source] Box<ProjectError>),
    #[error("journal commit receipt was not returned for an edit-only save")]
    MissingJournalCommitReceipt,
    #[error(
        "document writes are temporarily blocked while journal commit {receipt:?} is reconciled"
    )]
    DocumentWriteBlocked { receipt: JournalCommitReceipt },
    #[error(
        "journal commit receipt {receipt:?} was not observed after a successful commit result"
    )]
    CommitReceiptNotObserved { receipt: JournalCommitReceipt },
    #[error("recovered Document does not match prepared journal commit {receipt:?}")]
    ReconciledDocumentMismatch { receipt: JournalCommitReceipt },
    #[error("journal commit {receipt:?} could not yet be reconciled: {source}")]
    JournalReconcile {
        receipt: JournalCommitReceipt,
        #[source]
        source: Box<ProjectError>,
    },
    #[error(transparent)]
    Undo(#[from] UndoError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Duplicate(#[from] DuplicateError),
}
