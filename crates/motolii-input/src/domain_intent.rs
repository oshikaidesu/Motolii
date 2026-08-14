//! UI入口をtoolkit非依存の目的へ畳む最小境界。

use crate::UiStateOwner;

/// 入力方法に依存しない操作目的。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainIntent {
    DeleteTargetedItems,
    EnableReduceMotion,
    ResetWorkspaceProfile,
    FitStageView,
    CancelInFlightGesture,
    Undo,
    Redo,
}

impl DomainIntent {
    pub const ALL: [Self; 7] = [
        Self::DeleteTargetedItems,
        Self::EnableReduceMotion,
        Self::ResetWorkspaceProfile,
        Self::FitStageView,
        Self::CancelInFlightGesture,
        Self::Undo,
        Self::Redo,
    ];

    /// UI adapter内だけで使う一時kindを既知の目的へ変換する。
    ///
    /// `kind`は保存・再割当・安定識別に使わない。安定Command IDは後続の入力境界が所有する。
    pub const fn try_from_adapter_kind(kind: u16) -> Result<Self, DomainIntentError> {
        match kind {
            0 => Ok(Self::DeleteTargetedItems),
            1 => Ok(Self::EnableReduceMotion),
            2 => Ok(Self::ResetWorkspaceProfile),
            3 => Ok(Self::FitStageView),
            4 => Ok(Self::CancelInFlightGesture),
            5 => Ok(Self::Undo),
            6 => Ok(Self::Redo),
            got => Err(DomainIntentError::UnknownAdapterKind { got }),
        }
    }

    /// この目的が変更または制御する状態の所有層を返す。
    pub const fn owner(self) -> UiStateOwner {
        match self {
            Self::DeleteTargetedItems => UiStateOwner::Document,
            Self::EnableReduceMotion => UiStateOwner::UserSettings,
            Self::ResetWorkspaceProfile => UiStateOwner::WorkspaceProfile,
            Self::FitStageView => UiStateOwner::ProjectSession,
            Self::CancelInFlightGesture => UiStateOwner::Transient,
            Self::Undo | Self::Redo => UiStateOwner::Document,
        }
    }
}

/// UI操作を既知の目的へ変換できなかった理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DomainIntentError {
    #[error("adapter intent kind {got} is unknown")]
    UnknownAdapterKind { got: u16 },
}
