//! U0b-1: G0-2の代表状態を5層へ分類するfixture。

use motolii_doc::Document;
use motolii_ui::{UiStateLifetime, UiStateOwner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepresentativeState {
    Layer,
    ClipPlacement,
    Parameter,
    Connection,
    Camera,
    KeymapDelta,
    UiScale,
    Theme,
    ReduceMotion,
    ResourcePolicy,
    PanelOpen,
    PanelWidth,
    TimelineDensity,
    StageView,
    TimelineScrollZoom,
    SelectedPanel,
    Hover,
    Focus,
    DragPreview,
    ConnectionPicking,
    Popup,
    ImePreedit,
}

impl RepresentativeState {
    const ALL: [Self; 22] = [
        Self::Layer,
        Self::ClipPlacement,
        Self::Parameter,
        Self::Connection,
        Self::Camera,
        Self::KeymapDelta,
        Self::UiScale,
        Self::Theme,
        Self::ReduceMotion,
        Self::ResourcePolicy,
        Self::PanelOpen,
        Self::PanelWidth,
        Self::TimelineDensity,
        Self::StageView,
        Self::TimelineScrollZoom,
        Self::SelectedPanel,
        Self::Hover,
        Self::Focus,
        Self::DragPreview,
        Self::ConnectionPicking,
        Self::Popup,
        Self::ImePreedit,
    ];

    const fn owner(self) -> UiStateOwner {
        match self {
            Self::Layer
            | Self::ClipPlacement
            | Self::Parameter
            | Self::Connection
            | Self::Camera => UiStateOwner::Document,
            Self::KeymapDelta
            | Self::UiScale
            | Self::Theme
            | Self::ReduceMotion
            | Self::ResourcePolicy => UiStateOwner::UserSettings,
            Self::PanelOpen | Self::PanelWidth | Self::TimelineDensity => {
                UiStateOwner::WorkspaceProfile
            }
            Self::StageView | Self::TimelineScrollZoom | Self::SelectedPanel => {
                UiStateOwner::ProjectSession
            }
            Self::Hover
            | Self::Focus
            | Self::DragPreview
            | Self::ConnectionPicking
            | Self::Popup
            | Self::ImePreedit => UiStateOwner::Transient,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DocumentOwnedState;

#[derive(Debug, Default, PartialEq, Eq)]
struct NonDocumentMemory {
    user_settings: Vec<RepresentativeState>,
    workspace_profile: Vec<RepresentativeState>,
    project_session: Vec<RepresentativeState>,
    transient: Vec<RepresentativeState>,
}

#[derive(Debug)]
struct StateSeparationFixture {
    document: Document,
    memory: NonDocumentMemory,
}

impl StateSeparationFixture {
    fn new() -> Self {
        Self {
            document: Document::new_current(),
            memory: NonDocumentMemory::default(),
        }
    }

    fn apply_non_document(&mut self, state: RepresentativeState) -> Result<(), DocumentOwnedState> {
        match state.owner() {
            UiStateOwner::Document => return Err(DocumentOwnedState),
            UiStateOwner::UserSettings => self.memory.user_settings.push(state),
            UiStateOwner::WorkspaceProfile => self.memory.workspace_profile.push(state),
            UiStateOwner::ProjectSession => self.memory.project_session.push(state),
            UiStateOwner::Transient => self.memory.transient.push(state),
        }
        Ok(())
    }
}

#[test]
fn representative_states_cover_the_g0_2_five_layers() {
    let mut counts = [0_usize; 5];
    for state in RepresentativeState::ALL {
        let index = match state.owner() {
            UiStateOwner::Document => 0,
            UiStateOwner::UserSettings => 1,
            UiStateOwner::WorkspaceProfile => 2,
            UiStateOwner::ProjectSession => 3,
            UiStateOwner::Transient => 4,
        };
        counts[index] += 1;
    }

    assert_eq!(counts, [5, 5, 3, 3, 6]);
    assert_eq!(
        RepresentativeState::ClipPlacement.owner(),
        UiStateOwner::Document
    );
    assert_eq!(
        RepresentativeState::KeymapDelta.owner(),
        UiStateOwner::UserSettings
    );
    assert_eq!(
        RepresentativeState::PanelWidth.owner(),
        UiStateOwner::WorkspaceProfile
    );
    assert_eq!(
        RepresentativeState::TimelineScrollZoom.owner(),
        UiStateOwner::ProjectSession
    );
    assert_eq!(
        RepresentativeState::ImePreedit.owner(),
        UiStateOwner::Transient
    );
}

#[test]
fn every_owner_has_the_g0_2_lifetime() {
    let expected = [
        (UiStateOwner::Document, UiStateLifetime::ProjectDocument),
        (
            UiStateOwner::UserSettings,
            UiStateLifetime::UserAcrossProjects,
        ),
        (
            UiStateOwner::WorkspaceProfile,
            UiStateLifetime::UserWorkspaceProfile,
        ),
        (
            UiStateOwner::ProjectSession,
            UiStateLifetime::ProjectIdentityBestEffort,
        ),
        (UiStateOwner::Transient, UiStateLifetime::EventOrSession),
    ];

    for (owner, lifetime) in expected {
        assert_eq!(owner.lifetime(), lifetime);
    }
}

#[test]
fn workspace_profile_and_project_session_are_not_collapsed() {
    assert_ne!(
        RepresentativeState::PanelOpen.owner(),
        RepresentativeState::StageView.owner()
    );
    assert_ne!(
        UiStateOwner::WorkspaceProfile.lifetime(),
        UiStateOwner::ProjectSession.lifetime()
    );
}

#[test]
fn document_owned_state_is_rejected_by_the_non_document_boundary() {
    let mut fixture = StateSeparationFixture::new();
    let before =
        serde_json::to_vec(&fixture.document).expect("serialize current Document before rejection");

    assert_eq!(
        fixture.apply_non_document(RepresentativeState::ClipPlacement),
        Err(DocumentOwnedState)
    );
    assert_eq!(fixture.memory, NonDocumentMemory::default());
    assert_eq!(
        serde_json::to_vec(&fixture.document).expect("serialize Document after rejection"),
        before
    );
}

#[test]
fn non_document_updates_leave_the_owned_document_unchanged() {
    let mut fixture = StateSeparationFixture::new();
    let before =
        serde_json::to_vec(&fixture.document).expect("serialize current Document before updates");

    for state in RepresentativeState::ALL {
        if state.owner() == UiStateOwner::Document {
            continue;
        }
        fixture
            .apply_non_document(state)
            .expect("non-Document state must pass the separation boundary");
    }

    assert_eq!(fixture.memory.user_settings.len(), 5);
    assert_eq!(fixture.memory.workspace_profile.len(), 3);
    assert_eq!(fixture.memory.project_session.len(), 3);
    assert_eq!(fixture.memory.transient.len(), 6);
    assert_eq!(
        serde_json::to_vec(&fixture.document).expect("serialize Document after UI updates"),
        before
    );
}
