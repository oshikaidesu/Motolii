//! G0-2 §2.2 に基づく UI 状態の所有区分。
//!
//! 永続 codec ではなく、toolkit 非依存の分類と代表 fixture 表のみを提供する。

/// UI 状態の所有区分。Workspace profile と Project session cache は潰さない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiStateOwner {
    Document,
    UserSettings,
    WorkspaceSession(WorkspaceSessionKind),
    Transient,
}

/// G0-2: Workspace-session 候補の下位区分。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceSessionKind {
    /// panel 開閉・幅、Timeline density 等。user 単位。
    WorkspaceProfile,
    /// Stage pan/zoom/fit、Timeline scroll/zoom、選択中 panel 等。project identity 単位 cache。
    ProjectSessionCache,
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_doc::Document;

    struct UiStateFixtureId(&'static str);

    /// G0-2 表から選んだ代表 UI 状態。全エントリは `owner()` で所有者を返す。
    struct UiStateFixture {
        id: UiStateFixtureId,
        description: &'static str,
        owner: UiStateOwner,
    }

    impl UiStateFixture {
        fn id_str(&self) -> &'static str {
            self.id.0
        }

        fn owner(&self) -> UiStateOwner {
            self.owner
        }
    }

    const REPRESENTATIVE_FIXTURES: &[UiStateFixture] = &[
        UiStateFixture {
            id: UiStateFixtureId("document.layer"),
            description: "layer tree",
            owner: UiStateOwner::Document,
        },
        UiStateFixture {
            id: UiStateFixtureId("document.clip"),
            description: "clip placement",
            owner: UiStateOwner::Document,
        },
        UiStateFixture {
            id: UiStateFixtureId("document.parameter"),
            description: "parameter value",
            owner: UiStateOwner::Document,
        },
        UiStateFixture {
            id: UiStateFixtureId("user.keymap_delta"),
            description: "keymap delta",
            owner: UiStateOwner::UserSettings,
        },
        UiStateFixture {
            id: UiStateFixtureId("user.ui_scale"),
            description: "UI scale",
            owner: UiStateOwner::UserSettings,
        },
        UiStateFixture {
            id: UiStateFixtureId("user.theme"),
            description: "theme",
            owner: UiStateOwner::UserSettings,
        },
        UiStateFixture {
            id: UiStateFixtureId("user.reduce_motion"),
            description: "reduce motion",
            owner: UiStateOwner::UserSettings,
        },
        UiStateFixture {
            id: UiStateFixtureId("user.resource_policy"),
            description: "resource policy",
            owner: UiStateOwner::UserSettings,
        },
        UiStateFixture {
            id: UiStateFixtureId("workspace.panel_open"),
            description: "panel open/close",
            owner: UiStateOwner::WorkspaceSession(WorkspaceSessionKind::WorkspaceProfile),
        },
        UiStateFixture {
            id: UiStateFixtureId("workspace.timeline_density"),
            description: "Timeline density",
            owner: UiStateOwner::WorkspaceSession(WorkspaceSessionKind::WorkspaceProfile),
        },
        UiStateFixture {
            id: UiStateFixtureId("project.stage_view"),
            description: "Stage View pan/zoom/fit",
            owner: UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache),
        },
        UiStateFixture {
            id: UiStateFixtureId("project.timeline_scroll_zoom"),
            description: "Timeline scroll/zoom",
            owner: UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache),
        },
        UiStateFixture {
            id: UiStateFixtureId("project.selected_panel"),
            description: "selected panel",
            owner: UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache),
        },
        UiStateFixture {
            id: UiStateFixtureId("transient.hover"),
            description: "hover",
            owner: UiStateOwner::Transient,
        },
        UiStateFixture {
            id: UiStateFixtureId("transient.focus"),
            description: "focus",
            owner: UiStateOwner::Transient,
        },
        UiStateFixture {
            id: UiStateFixtureId("transient.drag_preview"),
            description: "drag preview",
            owner: UiStateOwner::Transient,
        },
        UiStateFixture {
            id: UiStateFixtureId("transient.connection_picking"),
            description: "connection picking",
            owner: UiStateOwner::Transient,
        },
        UiStateFixture {
            id: UiStateFixtureId("transient.popup"),
            description: "popup",
            owner: UiStateOwner::Transient,
        },
        UiStateFixture {
            id: UiStateFixtureId("transient.ime_preedit"),
            description: "IME preedit",
            owner: UiStateOwner::Transient,
        },
    ];

    fn representative_fixtures() -> &'static [UiStateFixture] {
        REPRESENTATIVE_FIXTURES
    }

    fn fixture_by_id(id: &str) -> &'static UiStateFixture {
        representative_fixtures()
            .iter()
            .find(|fixture| fixture.id_str() == id)
            .unwrap_or_else(|| panic!("missing representative fixture: {id}"))
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DocumentMutationRejected;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct UserSettingsMemory {
        keymap_delta: bool,
        ui_scale: bool,
        theme: bool,
        reduce_motion: bool,
        resource_policy: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct WorkspaceProfileMemory {
        panel_open: bool,
        timeline_density: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct ProjectSessionCacheMemory {
        stage_view: bool,
        timeline_scroll_zoom: bool,
        selected_panel: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct TransientMemory {
        hover: bool,
        focus: bool,
        drag_preview: bool,
        connection_picking: bool,
        popup: bool,
        ime_preedit: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct TestUiMemory {
        user_settings: UserSettingsMemory,
        workspace_profile: WorkspaceProfileMemory,
        project_session_cache: ProjectSessionCacheMemory,
        transient: TransientMemory,
    }

    impl TestUiMemory {
        fn apply_fixture(
            &mut self,
            fixture: &UiStateFixture,
        ) -> Result<(), DocumentMutationRejected> {
            match fixture.owner() {
                UiStateOwner::Document => Err(DocumentMutationRejected),
                UiStateOwner::UserSettings => {
                    match fixture.id_str() {
                        "user.keymap_delta" => self.user_settings.keymap_delta = true,
                        "user.ui_scale" => self.user_settings.ui_scale = true,
                        "user.theme" => self.user_settings.theme = true,
                        "user.reduce_motion" => self.user_settings.reduce_motion = true,
                        "user.resource_policy" => self.user_settings.resource_policy = true,
                        other => panic!("unexpected UserSettings fixture: {other}"),
                    }
                    Ok(())
                }
                UiStateOwner::WorkspaceSession(WorkspaceSessionKind::WorkspaceProfile) => {
                    match fixture.id_str() {
                        "workspace.panel_open" => self.workspace_profile.panel_open = true,
                        "workspace.timeline_density" => {
                            self.workspace_profile.timeline_density = true
                        }
                        other => panic!("unexpected WorkspaceProfile fixture: {other}"),
                    }
                    Ok(())
                }
                UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache) => {
                    match fixture.id_str() {
                        "project.stage_view" => self.project_session_cache.stage_view = true,
                        "project.timeline_scroll_zoom" => {
                            self.project_session_cache.timeline_scroll_zoom = true
                        }
                        "project.selected_panel" => {
                            self.project_session_cache.selected_panel = true
                        }
                        other => panic!("unexpected ProjectSessionCache fixture: {other}"),
                    }
                    Ok(())
                }
                UiStateOwner::Transient => {
                    match fixture.id_str() {
                        "transient.hover" => self.transient.hover = true,
                        "transient.focus" => self.transient.focus = true,
                        "transient.drag_preview" => self.transient.drag_preview = true,
                        "transient.connection_picking" => self.transient.connection_picking = true,
                        "transient.popup" => self.transient.popup = true,
                        "transient.ime_preedit" => self.transient.ime_preedit = true,
                        other => panic!("unexpected Transient fixture: {other}"),
                    }
                    Ok(())
                }
            }
        }
    }

    #[test]
    fn exact_g0_2_case_to_owner_mapping() {
        let assert_owner = |id: &str, expected: UiStateOwner| {
            assert_eq!(fixture_by_id(id).owner(), expected, "{id}");
        };

        assert_owner("document.layer", UiStateOwner::Document);
        assert_owner("document.clip", UiStateOwner::Document);
        assert_owner("document.parameter", UiStateOwner::Document);

        assert_owner("user.keymap_delta", UiStateOwner::UserSettings);
        assert_owner("user.ui_scale", UiStateOwner::UserSettings);
        assert_owner("user.theme", UiStateOwner::UserSettings);
        assert_owner("user.reduce_motion", UiStateOwner::UserSettings);
        assert_owner("user.resource_policy", UiStateOwner::UserSettings);

        assert_owner(
            "workspace.panel_open",
            UiStateOwner::WorkspaceSession(WorkspaceSessionKind::WorkspaceProfile),
        );
        assert_owner(
            "workspace.timeline_density",
            UiStateOwner::WorkspaceSession(WorkspaceSessionKind::WorkspaceProfile),
        );

        assert_owner(
            "project.stage_view",
            UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache),
        );
        assert_owner(
            "project.timeline_scroll_zoom",
            UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache),
        );
        assert_owner(
            "project.selected_panel",
            UiStateOwner::WorkspaceSession(WorkspaceSessionKind::ProjectSessionCache),
        );

        assert_owner("transient.hover", UiStateOwner::Transient);
        assert_owner("transient.focus", UiStateOwner::Transient);
        assert_owner("transient.drag_preview", UiStateOwner::Transient);
        assert_owner("transient.connection_picking", UiStateOwner::Transient);
        assert_owner("transient.popup", UiStateOwner::Transient);
        assert_owner("transient.ime_preedit", UiStateOwner::Transient);

        assert_eq!(representative_fixtures().len(), 19);
    }

    #[test]
    fn workspace_profile_fixtures_are_never_user_settings() {
        for fixture in representative_fixtures() {
            if matches!(
                fixture.owner(),
                UiStateOwner::WorkspaceSession(WorkspaceSessionKind::WorkspaceProfile)
            ) {
                assert_ne!(
                    fixture.owner(),
                    UiStateOwner::UserSettings,
                    "{}",
                    fixture.id_str()
                );
            }
        }

        assert_ne!(
            fixture_by_id("workspace.panel_open").owner(),
            UiStateOwner::UserSettings
        );
        assert_ne!(
            fixture_by_id("workspace.timeline_density").owner(),
            UiStateOwner::UserSettings
        );
    }

    #[test]
    fn document_clip_fixture_describes_clip_placement_not_selection() {
        let fixture = fixture_by_id("document.clip");
        assert_eq!(fixture.owner(), UiStateOwner::Document);
        assert_eq!(fixture.description, "clip placement");
        assert!(!fixture.description.contains("selection"));
    }

    #[test]
    fn document_owned_fixtures_are_rejected_by_classification_boundary() {
        for fixture in representative_fixtures() {
            if fixture.owner() == UiStateOwner::Document {
                let mut memory = TestUiMemory::default();
                assert_eq!(
                    memory.apply_fixture(fixture),
                    Err(DocumentMutationRejected),
                    "{}",
                    fixture.id_str()
                );
            }
        }
    }

    #[test]
    fn non_document_fixtures_update_test_only_memory_via_classification() {
        let mut memory = TestUiMemory::default();

        for fixture in representative_fixtures() {
            if fixture.owner() == UiStateOwner::Document {
                continue;
            }

            assert_eq!(
                memory.apply_fixture(fixture),
                Ok(()),
                "{}",
                fixture.id_str()
            );

            match fixture.id_str() {
                "user.keymap_delta" => assert!(memory.user_settings.keymap_delta),
                "user.ui_scale" => assert!(memory.user_settings.ui_scale),
                "user.theme" => assert!(memory.user_settings.theme),
                "user.reduce_motion" => assert!(memory.user_settings.reduce_motion),
                "user.resource_policy" => assert!(memory.user_settings.resource_policy),
                "workspace.panel_open" => assert!(memory.workspace_profile.panel_open),
                "workspace.timeline_density" => assert!(memory.workspace_profile.timeline_density),
                "project.stage_view" => assert!(memory.project_session_cache.stage_view),
                "project.timeline_scroll_zoom" => {
                    assert!(memory.project_session_cache.timeline_scroll_zoom)
                }
                "project.selected_panel" => assert!(memory.project_session_cache.selected_panel),
                "transient.hover" => assert!(memory.transient.hover),
                "transient.focus" => assert!(memory.transient.focus),
                "transient.drag_preview" => assert!(memory.transient.drag_preview),
                "transient.connection_picking" => assert!(memory.transient.connection_picking),
                "transient.popup" => assert!(memory.transient.popup),
                "transient.ime_preedit" => assert!(memory.transient.ime_preedit),
                other => panic!("unexpected non-Document fixture: {other}"),
            }
        }
    }

    #[test]
    fn mutating_non_document_ui_state_leaves_document_serialization_unchanged() {
        let doc = Document::new_current();
        let before = serde_json::to_vec(&doc).unwrap();
        let mut memory = TestUiMemory::default();

        for fixture in representative_fixtures() {
            if fixture.owner() == UiStateOwner::Document {
                continue;
            }
            memory
                .apply_fixture(fixture)
                .expect("non-Document fixture must be accepted");
            assert_eq!(
                serde_json::to_vec(&doc).unwrap(),
                before,
                "Document bytes changed after {}",
                fixture.id_str()
            );
        }
    }
}
