use super::RendererCore;
use super::host::{
    HostStageGeometryCommand, HostTerminalLatch, host_stage_geometry_command,
    timeline_scene_from_projection,
};
use super::scrub::{ScrubPointerPhase, ScrubTimePump};
use super::types::NativeHostTerminalEvent;
use crate::host_bridge::frame_from_scrub_bar;
use crate::timeline_skia::{TimelinePointerPhase, TimelineScene, TimelineSession};

#[test]
fn discard_gesture_on_scene_rebuild_leaves_no_active_gesture() {
    let mut session = TimelineSession::default();
    session.scene = TimelineScene::from_snapshot(
        &[crate::timeline_skia::SnapshotLayerInput {
            layer_id: "clip-real".into(),
            display_name: "clip".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![],
        }],
        None,
    );
    let mut selected = -1;
    // clip中心(bar1)からplayheadを離す — F1でplayhead優先になると選択がscrubになる。
    let mut playhead = 0.0;
    let x = 202.0 + (1.0f64 / 5.0) * (1240.0 - 202.0 - 6.0);
    let y = 66.5;
    let down = session.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    assert!(down.selection_commit.is_some() || selected >= 0);
    assert!(session.discard_active_gesture());
    // 差し替え後にUpしてもdispatchなし(gesture無し)。
    let up = session.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        x,
        y,
        0,
    );
    assert!(up.edit_commit.is_none());
}

#[test]
fn scrub_time_pump_throttle_moves_and_always_dispatches_release() {
    let mut pump = ScrubTimePump::new();
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Down, 4.0, 0, 30, 1),
        Some(frame_from_scrub_bar(4.0, 30, 1))
    );
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Move, 8.0, 16, 30, 1),
        None
    );
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Move, 8.0, 31, 30, 1),
        None
    );
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Move, 8.0, 32, 30, 1),
        Some(frame_from_scrub_bar(8.0, 30, 1))
    );
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Up, 24.0, 40, 30, 1),
        Some(frame_from_scrub_bar(24.0, 30, 1))
    );
}

#[test]
fn scrub_time_pump_restores_down_frame_only_after_dispatch() {
    let mut pump = ScrubTimePump::new();
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Down, 11.0, 0, 30, 1),
        Some(frame_from_scrub_bar(11.0, 30, 1))
    );
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 10, 30, 1),
        Some(frame_from_scrub_bar(11.0, 30, 1))
    );
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 20, 30, 1),
        None
    );
    let mut pump = ScrubTimePump::new();
    assert_eq!(
        pump.next_frame(ScrubPointerPhase::Cancel, 24.0, 20, 30, 1),
        None
    );
}

#[test]
fn real_clip_down_dispatches_selection_once_via_renderer_path() {
    let mut real_session = TimelineSession::default();
    real_session.scene = TimelineScene::from_snapshot(
        &[crate::timeline_skia::SnapshotLayerInput {
            layer_id: "clip-real".into(),
            display_name: "clip".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![],
        }],
        None,
    );
    let mut selected = -1;
    let mut playhead = 0.27;
    let x = 202.0 + (3.0f64 / 5.0) * (1240.0 - 202.0 - 6.0);
    let y = 66.5;
    crate::host_bridge::test_reset_timeline_selection_dispatch_count();
    crate::host_bridge::test_clear_host_slot();
    let down = real_session.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    assert!(down.selection_commit.is_some());
    if let Some(commit) = down.selection_commit {
        assert!(crate::host_bridge::try_dispatch_timeline_selection(&commit).is_none());
    }
    assert_eq!(
        crate::host_bridge::test_timeline_selection_dispatch_count(),
        1
    );
}

#[test]
fn fixture_down_or_trim_down_does_not_dispatch_selection() {
    let mut session = TimelineSession::default();
    let down_x = 202.0 + (3.0f64 / 48.0) * (1240.0 - 202.0 - 6.0);
    let y = 66.5;
    crate::host_bridge::test_reset_timeline_selection_dispatch_count();
    crate::host_bridge::test_clear_host_slot();
    let mut selected = -1;
    let mut playhead = 0.27;
    let clip_down = session.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        down_x,
        y,
        0,
    );
    assert!(clip_down.selection_commit.is_none());

    let trim_down = session.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        202.0 + (14.0f64 / 48.0) * (1240.0 - 202.0 - 6.0),
        y,
        0,
    );
    assert!(trim_down.selection_commit.is_none());

    if let Some(commit) = clip_down.selection_commit {
        assert!(crate::host_bridge::try_dispatch_timeline_selection(&commit).is_none());
    }
    if let Some(commit) = trim_down.selection_commit {
        assert!(crate::host_bridge::try_dispatch_timeline_selection(&commit).is_none());
    }
    assert_eq!(
        crate::host_bridge::test_timeline_selection_dispatch_count(),
        0
    );
}

#[test]
fn host_stage_geometry_command_transitions_apply_and_clear() {
    let geometry_a = crate::host_bridge::HostStageGeometry {
        layers: vec![crate::host_bridge::HostStageGeometryLayer {
            layer_id: "L1".into(),
            corners: [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }],
        layers_truncated: false,
    };
    let geometry_b = crate::host_bridge::HostStageGeometry {
        layers: vec![crate::host_bridge::HostStageGeometryLayer {
            layer_id: "L1".into(),
            corners: [[-0.4, -0.4], [0.4, -0.4], [0.4, 0.4], [-0.4, 0.4]],
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }],
        layers_truncated: false,
    };
    let projection_a = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "r".into(),
        projection_generation: "0".into(),
        primary_layer_id: None,
        current_time: (0, 1),
        fps: None,
        timeline_duration: Some((10, 1)),
        bounds: vec![],
        timeline_layers: None,
        stage_geometry: Some(geometry_a.clone()),
    };
    let projection_b = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "r".into(),
        projection_generation: "0".into(),
        primary_layer_id: None,
        current_time: (0, 1),
        fps: None,
        timeline_duration: Some((10, 1)),
        bounds: vec![],
        timeline_layers: None,
        stage_geometry: Some(geometry_b.clone()),
    };

    let mut cached = None;
    let apply = host_stage_geometry_command(cached.as_ref(), Some(&projection_a));
    assert_eq!(apply, HostStageGeometryCommand::Apply(geometry_a.clone()));
    if let HostStageGeometryCommand::Apply(next) = apply {
        cached = Some(next);
    }
    assert_eq!(cached.as_ref(), Some(&geometry_a));

    let noop = host_stage_geometry_command(cached.as_ref(), Some(&projection_a));
    assert_eq!(noop, HostStageGeometryCommand::Noop);

    let apply = host_stage_geometry_command(cached.as_ref(), Some(&projection_b));
    assert_eq!(apply, HostStageGeometryCommand::Apply(geometry_b.clone()));
    if let HostStageGeometryCommand::Apply(next) = apply {
        cached = Some(next);
    }
    assert_eq!(cached.as_ref(), Some(&geometry_b));

    let clear = host_stage_geometry_command(cached.as_ref(), None);
    assert_eq!(clear, HostStageGeometryCommand::Clear);
}

#[test]
fn timeline_keymap_delete_branches_on_selected_real_key() {
    crate::host_bridge::test_reset_keymap_dispatch_counts();
    let mut scene = TimelineScene::from_snapshot(
        &[crate::timeline_skia::SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![crate::timeline_skia::SnapshotKeyInput {
                key_id: 7,
                time_secs: 4.0,
            }],
        }],
        None,
    );
    let _ = crate::host_bridge::try_timeline_keymap_delete(&scene);
    assert_eq!(
        crate::host_bridge::test_keymap_remove_position_key_count(),
        0
    );
    assert_eq!(crate::host_bridge::test_keymap_delete_layer_count(), 1);

    crate::timeline_skia::test_select_first_real_key(&mut scene);
    let _ = crate::host_bridge::try_timeline_keymap_delete(&scene);
    assert_eq!(
        crate::host_bridge::test_keymap_remove_position_key_count(),
        1
    );
    assert_eq!(crate::host_bridge::test_keymap_delete_layer_count(), 1);
}

#[test]
fn key_selection_survives_revision_reproject_so_delete_removes_key() {
    crate::host_bridge::test_reset_keymap_dispatch_counts();
    let mut scene = TimelineScene::from_snapshot(
        &[crate::timeline_skia::SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![
                crate::timeline_skia::SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                },
                crate::timeline_skia::SnapshotKeyInput {
                    key_id: 8,
                    time_secs: 6.0,
                },
            ],
        }],
        Some("11"),
    );
    crate::timeline_skia::test_select_first_real_key(&mut scene);
    assert_eq!(
        crate::timeline_skia::selected_real_key(&scene),
        Some(("11".into(), 7))
    );

    let mut projection = crate::host_bridge::HostTimelineProjection {
        host_handle: None,
        revision: "r2".into(),
        projection_generation: "1".into(),
        primary_layer_id: Some("11".into()),
        current_time: (0, 1),
        timeline_duration: Some((10, 1)),
        fps: None,
        bounds: vec![("11".into(), "keyed".into())],
        timeline_layers: Some(vec![crate::host_bridge::HostTimelineLayer {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            start_secs: 0.0,
            duration_secs: 10.0,
            position_keys: vec![
                crate::host_bridge::HostTimelineKey {
                    key_id: 7,
                    time_secs: 4.0,
                    value: None,
                },
                crate::host_bridge::HostTimelineKey {
                    key_id: 8,
                    time_secs: 6.0,
                    value: None,
                },
            ],
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
    let rebuilt = timeline_scene_from_projection(&scene, &projection);
    assert_eq!(
        crate::timeline_skia::selected_real_key(&rebuilt),
        Some(("11".into(), 7))
    );
    let _ = crate::host_bridge::try_timeline_keymap_delete(&rebuilt);
    assert_eq!(
        crate::host_bridge::test_keymap_remove_position_key_count(),
        1
    );
    assert_eq!(crate::host_bridge::test_keymap_delete_layer_count(), 0);

    projection.primary_layer_id = None;
    let primary_cleared = timeline_scene_from_projection(&scene, &projection);
    assert_eq!(
        crate::timeline_skia::selected_real_key(&primary_cleared),
        None
    );
}

#[test]
fn move_preview_geometry_translates_only_target_layer() {
    let geometry = crate::host_bridge::HostStageGeometry {
        layers: vec![
            crate::host_bridge::HostStageGeometryLayer {
                layer_id: "A".into(),
                corners: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
                position: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            },
            crate::host_bridge::HostStageGeometryLayer {
                layer_id: "B".into(),
                corners: [[2.0, 2.0], [3.0, 2.0], [3.0, 3.0], [2.0, 3.0]],
                position: [0.0, 0.0],
                rotation: 0.0,
                scale: [1.0, 1.0],
            },
        ],
        layers_truncated: false,
    };
    let preview = crate::rerun_stage::apply_move_preview_to_geometry(
        &geometry,
        Some(&("A".into(), [0.5, -0.25])),
    );
    assert_eq!(
        preview.layers[0].corners,
        [[0.5, -0.25], [1.5, -0.25], [1.5, 0.75], [0.5, 0.75]]
    );
    assert_eq!(preview.layers[1].corners, geometry.layers[1].corners);
    let restored = crate::rerun_stage::apply_move_preview_to_geometry(&geometry, None);
    assert_eq!(restored, geometry);
}

#[test]
fn host_snapshot_read_needed_force_and_first_read_and_missing_stamp() {
    // force / 初回(None)は読む。host不在でstamp取得失敗もfull読みへ落とす。
    assert!(RendererCore::host_snapshot_read_needed(
        None,
        Some((1, 2)),
        false
    ));
    assert!(RendererCore::host_snapshot_read_needed(
        Some((1, 2)),
        Some((1, 2)),
        true
    ));
    assert!(RendererCore::host_snapshot_read_needed(
        Some((1, 2)),
        Some((0, 3)),
        false
    ));
    assert!(!RendererCore::host_snapshot_read_needed(
        Some((1, 2)),
        Some((1, 2)),
        false
    ));
    assert!(RendererCore::host_snapshot_read_needed(
        Some((1, 2)),
        None,
        false
    ));
}

#[test]
fn host_terminal_latch_returns_latest_terminal_once() {
    let mut latch = HostTerminalLatch::default();
    latch.record(&crate::host_bridge::HostTerminalResult {
        accepted: true,
        diagnostics: vec![],
        message: None,
        projection: None,
    });
    latch.record(&crate::host_bridge::HostTerminalResult {
        accepted: false,
        diagnostics: vec![crate::host_bridge::HostTerminalDiagnostic {
            reason: "stale_projection_generation".into(),
            host_handle: Some("7".into()),
            stage_handle: None,
            timeline_handle: Some("8".into()),
            expected_projection_generation: Some("5".into()),
            actual_projection_generation: Some("4".into()),
        }],
        message: None,
        projection: None,
    });

    assert_eq!(
        latch.take(),
        Some(NativeHostTerminalEvent {
            accepted: false,
            message: "stale_projection_generation".into(),
        })
    );
    assert_eq!(latch.take(), None);
}

#[test]
fn host_terminal_latch_ignores_late_old_host_and_lower_generation() {
    let projection = |host: &str, generation: &str| crate::host_bridge::HostTimelineProjection {
        host_handle: Some(host.into()),
        revision: "9".into(),
        projection_generation: generation.into(),
        primary_layer_id: None,
        current_time: (0, 1),
        timeline_duration: None,
        fps: None,
        bounds: vec![],
        timeline_layers: None,
        stage_geometry: None,
    };
    let terminal = |projection| crate::host_bridge::HostTerminalResult {
        accepted: true,
        diagnostics: vec![],
        message: None,
        projection: Some(projection),
    };
    let mut latch = HostTerminalLatch::default();

    assert!(!latch.record_if_current(Some("7"), Some("5"), &terminal(projection("6", "99")),));
    assert!(!latch.record_if_current(Some("7"), Some("5"), &terminal(projection("7", "4")),));
    assert_eq!(latch.take(), None);

    assert!(latch.record_if_current(Some("7"), Some("5"), &terminal(projection("8", "0")),));
    assert_eq!(
        latch.take(),
        Some(NativeHostTerminalEvent {
            accepted: true,
            message: String::new(),
        })
    );
}
