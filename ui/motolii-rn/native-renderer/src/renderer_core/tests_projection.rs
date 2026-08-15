use super::host::{
    HostStageGeometryCommand, host_stage_geometry_command, stage_selection_commit,
    timeline_projection_selected_flat, timeline_scene_from_projection,
};
use super::types::StagePointerButton;
use crate::timeline_skia::{TimelineScene, TimelineSession};

#[test]
fn stage_pointer_buttons_accept_only_rerun_standard_buttons() {
    assert_eq!(
        StagePointerButton::from_raw(0),
        Some(StagePointerButton::Primary)
    );
    assert_eq!(
        StagePointerButton::from_raw(1),
        Some(StagePointerButton::Secondary)
    );
    assert_eq!(
        StagePointerButton::from_raw(2),
        Some(StagePointerButton::Middle)
    );
    assert_eq!(StagePointerButton::from_raw(3), None);
}

#[test]
fn rerun_entity_selection_remaps_to_existing_document_selection_intent() {
    assert_eq!(
        stage_selection_commit(Some("motolii/document/layers/42/fill")),
        crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
            layer_id: "42".into()
        }
    );
    assert_eq!(
        stage_selection_commit(Some("motolii/document/layers/42/path")),
        crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
            layer_id: "42".into()
        }
    );
    assert_eq!(
        stage_selection_commit(Some("motolii/document/frame")),
        crate::timeline_skia::TimelineSelectionCommit::ClearSelection
    );
    assert_eq!(
        stage_selection_commit(None),
        crate::timeline_skia::TimelineSelectionCommit::ClearSelection
    );
}

#[test]
fn timeline_view_is_preserved_when_revision_changes() {
    let mut scene = TimelineScene::from_snapshot(
        &[crate::timeline_skia::SnapshotLayerInput {
            layer_id: "L1".into(),
            display_name: "Layer 1".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![],
        }],
        Some("L1"),
    );
    scene.view_a = 1.0;
    scene.view_b = 4.0;
    let projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "r1".into(),
        projection_generation: "0".into(),
        primary_layer_id: Some("L2".into()),
        current_time: (0, 1),
        timeline_duration: Some((10, 1)),
        fps: None,
        bounds: vec![
            ("L1".into(), "Layer 1".into()),
            ("L2".into(), "Layer 2".into()),
        ],
        timeline_layers: None,
        stage_geometry: None,
    };
    let rebuilt = timeline_scene_from_projection(&scene, &projection);
    assert_eq!(rebuilt.view_a, 1.0);
    assert_eq!(rebuilt.view_b, 4.0);
    assert_eq!(rebuilt.selected_flat, 1);
    assert_eq!(timeline_projection_selected_flat(&projection), 1);
    let expected_song_bars = 10.0_f32 / crate::timeline_skia::SECONDS_PER_BAR as f32;
    assert!((rebuilt.song_bars - expected_song_bars).abs() < 1e-6);

    let mut missing = projection.clone();
    missing.primary_layer_id = Some("outside-truncated-projection".into());
    assert_eq!(timeline_projection_selected_flat(&missing), -1);
}

#[test]
fn timeline_projection_scales_song_bars_from_duration_10_and_40_seconds() {
    let existing = TimelineScene::default();

    for (duration_num, duration_den, expected_song_bars) in
        [(10_i64, 1_i64, 10.0_f32), (40_i64, 1_i64, 40.0_f32)]
    {
        let projection = crate::host_bridge::HostTimelineProjection {
            host_handle: None,
            revision: "r0".into(),
            projection_generation: "0".into(),
            primary_layer_id: Some("L1".into()),
            current_time: (0, 1),
            timeline_duration: Some((duration_num, duration_den)),
            fps: Some((30, 1)),
            bounds: vec![("L1".to_string(), "Layer 1".to_string())],
            timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
                layer_id: "L1".into(),
                display_name: "Layer 1".into(),
                start_secs: 0.0,
                duration_secs: duration_num as f64 / duration_den as f64,
                position_keys: vec![],
                param_keys: vec![],
                effects: vec![],
                effects_truncated: false,
                source_params: vec![],
                source_params_truncated: false,
                visible: true,
                solo: false,
            }]),
            stage_geometry: None,
        };

        let rebuilt = timeline_scene_from_projection(&existing, &projection);
        assert_eq!(rebuilt.selected_flat, 0);
        assert!((rebuilt.song_bars - expected_song_bars).abs() < 1e-6);
        assert!((rebuilt.view_a - 0.0).abs() < 1e-6);
        assert!((rebuilt.view_b - rebuilt.song_bars).abs() < 1e-6);
    }
}

#[test]
fn timeline_projection_short_duration_does_not_panic_and_sets_span_to_duration() {
    let existing = TimelineScene::default();
    let projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "r2".into(),
        projection_generation: "0".into(),
        primary_layer_id: Some("L1".into()),
        current_time: (0, 1),
        timeline_duration: Some((2, 1)),
        fps: Some((30, 1)),
        bounds: vec![("L1".into(), "Layer 1".into())],
        timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
            layer_id: "L1".into(),
            display_name: "Layer 1".into(),
            start_secs: 0.0,
            duration_secs: 2.0,
            position_keys: vec![],
            param_keys: vec![],
            effects: vec![],
            effects_truncated: false,
            source_params: vec![],
            source_params_truncated: false,
            visible: true,
            solo: false,
        }]),
        stage_geometry: None,
    };

    let rebuilt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        timeline_scene_from_projection(&existing, &projection)
    }));
    assert!(rebuilt.is_ok(), "short duration must not panic");
    let rebuilt = rebuilt.expect("short duration must not panic");
    assert!((rebuilt.song_bars - 2.0).abs() < 1e-6);
    assert_eq!(rebuilt.view_a, 0.0);
    assert_eq!(rebuilt.view_b, 2.0);
}

#[test]
fn fixture_to_real_projection_resets_view_to_song_bars() {
    let existing = TimelineScene::default();
    assert!(!existing.real);
    let projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "r1".into(),
        projection_generation: "0".into(),
        primary_layer_id: Some("L1".into()),
        current_time: (0, 1),
        fps: Some((30, 1)),
        timeline_duration: Some((10, 1)),
        bounds: vec![("L1".into(), "Layer 1".into())],
        timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
            layer_id: "L1".into(),
            display_name: "Layer 1".into(),
            start_secs: 0.0,
            duration_secs: 10.0,
            position_keys: vec![],
            param_keys: vec![],
            effects: vec![],
            effects_truncated: false,
            source_params: vec![],
            source_params_truncated: false,
            visible: true,
            solo: false,
        }]),
        stage_geometry: None,
    };
    let rebuilt = timeline_scene_from_projection(&existing, &projection);
    assert!(rebuilt.real);
    assert!((rebuilt.view_a - 0.0).abs() < 1e-6);
    assert!((rebuilt.view_b - rebuilt.song_bars).abs() < 1e-6);
    assert!((rebuilt.song_bars - 10.0).abs() < 1e-6);
}

#[test]
fn product_timeline_session_starts_empty_host_not_fixture() {
    let session = TimelineSession::host_product();
    assert!(session.scene.real);
    assert_eq!(session.scene.band_count(), 0);
    let fixture = TimelineScene::default();
    assert!(!fixture.real);
    assert!(fixture.band_count() > 0);
}

#[test]
fn empty_host_projection_clears_fixture_bands() {
    let existing = TimelineScene::default();
    assert!(!existing.real);
    assert!(existing.band_count() > 0);
    let projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "0".into(),
        projection_generation: "0".into(),
        primary_layer_id: None,
        current_time: (0, 1),
        fps: Some((30, 1)),
        timeline_duration: Some((10, 1)),
        bounds: vec![],
        timeline_layers: Some(vec![]),
        stage_geometry: Some(crate::host_bridge::HostStageGeometry {
            layers: vec![],
            layers_truncated: false,
        }),
    };
    let rebuilt = timeline_scene_from_projection(&existing, &projection);
    assert!(rebuilt.real);
    assert_eq!(rebuilt.band_count(), 0);
    assert!(rebuilt.clip0_layer_id(0).is_none());
}

#[test]
fn host_projection_exposes_the_same_layer_id() {
    let existing = TimelineScene::empty_host();
    let projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "1".into(),
        projection_generation: "1".into(),
        primary_layer_id: Some("42".into()),
        current_time: (0, 1),
        fps: Some((30, 1)),
        timeline_duration: Some((10, 1)),
        bounds: vec![("42".into(), "Rectangle".into())],
        timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
            layer_id: "42".into(),
            display_name: "Rectangle".into(),
            start_secs: 0.0,
            duration_secs: 10.0,
            position_keys: vec![],
            param_keys: vec![],
            effects: vec![],
            effects_truncated: false,
            source_params: vec![],
            source_params_truncated: false,
            visible: true,
            solo: false,
        }]),
        stage_geometry: Some(crate::host_bridge::HostStageGeometry {
            layers: vec![crate::host_bridge::HostStageGeometryLayer {
                layer_id: "42".into(),
                corners: [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
                position: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            }],
            layers_truncated: false,
        }),
    };
    let rebuilt = timeline_scene_from_projection(&existing, &projection);
    assert!(rebuilt.real);
    assert_eq!(rebuilt.band_count(), 1);
    assert_eq!(rebuilt.clip0_layer_id(0), Some("42"));
    assert_eq!(rebuilt.selected_flat, 0);
    let apply = host_stage_geometry_command(None, Some(&projection));
    match apply {
        HostStageGeometryCommand::Apply(geometry) => {
            assert_eq!(geometry.layers.len(), 1);
            assert_eq!(geometry.layers[0].layer_id, "42");
        }
        other => panic!("expected Apply, got {other:?}"),
    }
}

#[test]
fn empty_host_stage_geometry_applies_instead_of_leaving_fixture() {
    let projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "0".into(),
        projection_generation: "0".into(),
        primary_layer_id: None,
        current_time: (0, 1),
        fps: None,
        timeline_duration: Some((10, 1)),
        bounds: vec![],
        timeline_layers: Some(vec![]),
        stage_geometry: Some(crate::host_bridge::HostStageGeometry {
            layers: vec![],
            layers_truncated: false,
        }),
    };
    let apply = host_stage_geometry_command(None, Some(&projection));
    match apply {
        HostStageGeometryCommand::Apply(geometry) => {
            assert!(geometry.layers.is_empty());
        }
        other => panic!("empty host must Apply empty geometry, got {other:?}"),
    }
}
