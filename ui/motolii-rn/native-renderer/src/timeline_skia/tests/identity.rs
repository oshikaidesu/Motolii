use super::*;

#[test]
fn selected_real_key_tracks_sel_flag() {
    let mut scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![SnapshotKeyInput {
                key_id: 7,
                time_secs: 4.0,
            }],
        }],
        None,
    );
    assert!(selected_real_key(&scene).is_none());
    assert!(remove_position_key_commit(&scene).is_none());
    scene.bands[0].clips[0].keys[0].2 = true;
    assert_eq!(selected_real_key(&scene), Some(("11".into(), 7)));
    assert_eq!(
        remove_position_key_commit(&scene),
        Some(TimelineEditCommit::RemovePositionKey {
            layer_id: "11".into(),
            key_id: 7,
        })
    );
}

#[test]
fn key_click_without_drag_does_not_commit_remove() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![SnapshotKeyInput {
                key_id: 7,
                time_secs: 4.0,
            }],
        }],
        Some("11"),
    );
    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let x = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].keys[0].0));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    let up = sess.pointer(
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
    assert_eq!(selected_real_key(&sess.scene), Some(("11".into(), 7)));
    assert_eq!(
        remove_position_key_commit(&sess.scene),
        Some(TimelineEditCommit::RemovePositionKey {
            layer_id: "11".into(),
            key_id: 7,
        })
    );
}

#[test]
fn real_clip_body_click_clears_key_selection() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![SnapshotKeyInput {
                key_id: 7,
                time_secs: 4.0,
            }],
        }],
        Some("11"),
    );
    test_select_first_real_key(&mut sess.scene);
    assert!(selected_real_key(&sess.scene).is_some());
    let mut selected = 0;
    let mut playhead = 0.1;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    // keyでないclip本体中央。
    let x = f64::from(bx(&sess.scene, 3.0));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    assert!(selected_real_key(&sess.scene).is_none());
}

#[test]
fn clip_move_commit_follows_layer_id_after_band_index_shift() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[
            SnapshotLayerInput {
                layer_id: "1".into(),
                display_name: "a".into(),
                interval_secs: Some((0.0, 4.0)),
                keys: vec![],
            },
            SnapshotLayerInput {
                layer_id: "2".into(),
                display_name: "b".into(),
                interval_secs: Some((0.0, 4.0)),
                keys: vec![],
            },
        ],
        Some("2"),
    );
    let mut selected = 1;
    let mut playhead = 0.1;
    // 0..4s clipの中央。adopt済みedge幅との境界をbody pressに使わない。
    let x = lx_for_bar_in(&sess.scene, 2.0);
    let y = body_top() + f64::from(ROW) + 5.0;
    let down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    assert_eq!(
        down.selection_commit,
        Some(TimelineSelectionCommit::SelectLayer {
            layer_id: "2".into()
        })
    );
    assert_eq!(sess.scene.bands[1].clips[0].layer_id, "2");

    let dummy = sess.scene.bands[0].clone();
    sess.scene.bands.insert(0, dummy);
    assert_eq!(sess.scene.bands[1].clips[0].layer_id, "1");
    assert_eq!(sess.scene.bands[2].clips[0].layer_id, "2");
    let origin_a = sess.scene.bands[2].clips[0].a;

    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar_in(&sess.scene, 3.0),
        y,
        0,
    );
    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, 3.0),
        y,
        0,
    );
    assert_eq!(sess.scene.bands[2].clips[0].layer_id, "2");
    assert!((sess.scene.bands[1].clips[0].a - 0.0).abs() < 1e-3);
    match up.edit_commit {
        Some(TimelineEditCommit::SetClipStart { layer_id, bar }) => {
            assert_eq!(layer_id, "2");
            assert!((bar - sess.scene.bands[2].clips[0].a).abs() < 1e-3);
            assert!((sess.scene.bands[2].clips[0].a - origin_a).abs() > 1e-3);
        }
        other => panic!("expected SetClipStart for layer 2, got {other:?}"),
    }
}

#[test]
fn restore_key_selection_matches_by_key_id() {
    let mut scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![
                SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                },
                SnapshotKeyInput {
                    key_id: 9,
                    time_secs: 6.0,
                },
            ],
        }],
        Some("11"),
    );
    assert!(restore_key_selection(&mut scene, "11", 9));
    assert_eq!(selected_real_key(&scene), Some(("11".into(), 9)));
    assert!(!scene.bands[0].clips[0].keys[0].2);
    assert!(scene.bands[0].clips[0].keys[1].2);
}

#[test]
fn restore_key_selection_requires_layer_match() {
    let mut scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![
                SnapshotKeyInput {
                    key_id: 7,
                    time_secs: 4.0,
                },
                SnapshotKeyInput {
                    key_id: 9,
                    time_secs: 6.0,
                },
            ],
        }],
        Some("11"),
    );
    assert!(!restore_key_selection(&mut scene, "12", 9));
    assert!(selected_real_key(&scene).is_none());
}

#[test]
fn real_projection_key_drag_discarded_when_scene_layer_id_changes() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "11".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![SnapshotKeyInput {
                key_id: 7,
                time_secs: 4.0,
            }],
        }],
        Some("11"),
    );
    sess.scene.real = true;
    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let start_key = sess.scene.bands[0].clips[0].keys[0];
    let x = f64::from(bx(&sess.scene, start_key.0));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    let replaced = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "12".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![SnapshotKeyInput {
                key_id: 7,
                time_secs: 4.0,
            }],
        }],
        Some("12"),
    );
    sess.scene = replaced;
    sess.scene.real = true;

    let moved = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        x + 12.0,
        y,
        0,
    );
    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        x + 12.0,
        y,
        0,
    );
    assert!(moved.edit_commit.is_none());
    assert!(up.edit_commit.is_none());
    assert!(!sess.has_active_gesture());
    assert_eq!(sess.scene.bands[0].clips[0].keys[0].0, start_key.0);
}

#[test]
fn real_clip_move_preview_does_not_shift_keys() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "k-move".into(),
            display_name: "keyed".into(),
            interval_secs: Some((0.0, 8.0)), // bars 0..4
            keys: vec![SnapshotKeyInput {
                key_id: 3,
                time_secs: 2.0, // bar 1.0
            }],
        }],
        Some("k-move"),
    );
    let key_before = sess.scene.bands[0].clips[0].keys[0].0;
    let origin_a = sess.scene.bands[0].clips[0].a;
    let mut selected = 0;
    let mut playhead = 0.1;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let mid = f64::from(bx(
        &sess.scene,
        (origin_a + sess.scene.bands[0].clips[0].b) * 0.5,
    ));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        mid,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        mid + 40.0,
        y,
        0,
    );
    assert!((sess.scene.bands[0].clips[0].a - origin_a).abs() > f32::EPSILON);
    assert!((sess.scene.bands[0].clips[0].keys[0].0 - key_before).abs() < 1e-4);
}

#[test]
fn discard_active_gesture_drops_without_restore() {
    let mut sess = TimelineSession::default();
    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + 5.0;
    let x = lx_for_bar(2.0);
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y,
        0,
    );
    let before = sess.scene.clone();
    assert!(sess.discard_active_gesture());
    assert_eq!(sess.scene, before);
    assert!(!sess.discard_active_gesture());
}

#[test]
fn playhead_near_down_starts_scrub_distant_clip_does_not() {
    let (mut sess, mut selected, mut playhead) = session();
    playhead = 12.0 / 96.0;
    let px = f64::from(bx(&sess.scene, 12.0));
    let body_y = body_top() + f64::from(ROW) + 5.0;
    let near = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        px + 2.0,
        body_y,
        0,
    );
    assert!(near.scrub_playhead.is_some());
    assert!(matches!(sess.gesture, Some(ActiveGesture::Scrub { .. })));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        px + 2.0,
        body_y,
        0,
    );

    let (mut sess, mut selected, mut playhead) = session();
    playhead = 12.0 / 96.0;
    // band1 hero本体。key(8/13/18)とplayhead(12)から離す。
    let clip_x = lx_for_bar(10.0);
    let clip_down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        clip_x,
        body_y,
        0,
    );
    assert!(clip_down.scrub_playhead.is_none());
    assert!(matches!(
        sess.gesture,
        Some(ActiveGesture::SelectOrMove { .. })
    ));
    assert!((playhead - 12.0 / 96.0).abs() < 1e-9);
}
