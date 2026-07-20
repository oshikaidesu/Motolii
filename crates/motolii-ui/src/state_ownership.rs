//! G0-2で決定済みのUI状態所有と寿命を表す。

/// UI状態の所有層。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiStateOwner {
    Document,
    UserSettings,
    WorkspaceProfile,
    ProjectSession,
    Transient,
}

/// 所有層に対応する状態寿命。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiStateLifetime {
    ProjectDocument,
    UserAcrossProjects,
    UserWorkspaceProfile,
    ProjectIdentityBestEffort,
    EventOrSession,
}

impl UiStateOwner {
    /// G0-2で所有層と同時に固定した寿命を返す。
    pub const fn lifetime(self) -> UiStateLifetime {
        match self {
            Self::Document => UiStateLifetime::ProjectDocument,
            Self::UserSettings => UiStateLifetime::UserAcrossProjects,
            Self::WorkspaceProfile => UiStateLifetime::UserWorkspaceProfile,
            Self::ProjectSession => UiStateLifetime::ProjectIdentityBestEffort,
            Self::Transient => UiStateLifetime::EventOrSession,
        }
    }
}
