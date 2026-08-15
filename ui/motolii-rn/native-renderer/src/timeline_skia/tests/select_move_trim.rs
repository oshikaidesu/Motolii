use super::*;

#[test]
fn clip_body_down_selects_without_moving_playhead() {
    let (mut sess, mut selected, mut playhead) = session();
    let x = lx_for_bar(8.0);
    let y = body_top() + f64::from(ROW) + 5.0; // band1 hero
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
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        x,
        y,
        0,
    );
    assert_eq!(selected, 3); // band0 has 3 clips
    assert!((playhead - 0.27).abs() < 1e-9);
}

#[test]
fn empty_bar_down_clears_selection() {
    let (mut sess, mut selected, mut playhead) = session();
    let y = body_top() + f64::from(ROW) * 4.5;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        lx_for_bar(10.0),
        y,
        0,
    );
    assert_eq!(selected, -1);
}

#[test]
fn clip_move_shifts_keys_and_clamps_to_neighbors() {
    let (mut sess, mut selected, mut playhead) = session();
    // band1 clip0 hero 4..22, next starts 26 → max a=8
    let press_x = lx_for_bar(10.0);
    let y = body_top() + f64::from(ROW) + 5.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        press_x,
        y,
        0,
    );
    let move_x = press_x + 10.0; // >3px, ~0.465 bars
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        move_x,
        y,
        0,
    );
    let clip = &sess.scene.bands[1].clips[0];
    let expected_dx =
        (10.0 / surface_width() * f64::from(sess.scene.view_b - sess.scene.view_a)) as f32;
    let snapped = test_snap_bar(&sess.scene, playhead, 1, Some(0), 4.0 + expected_dx, 0);
    assert!((clip.a - snapped).abs() < 1e-4);
    assert!((clip.b - (22.0 + (snapped - 4.0))).abs() < 1e-4);
    assert!((clip.keys[0].0 - (8.0 + (snapped - 4.0))).abs() < 1e-4);

    // 大きく右へ飛ばしてclamp
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        press_x + 500.0,
        y,
        0,
    );
    let clip = &sess.scene.bands[1].clips[0];
    assert!((clip.a - 8.0).abs() < 1e-4); // 26 - 18
    assert!((clip.b - 26.0).abs() < 1e-4);
    assert!((clip.keys[0].0 - 12.0).abs() < 1e-4);
}

#[test]
fn cancel_restores_full_scene_selection_playhead_and_key_state_after_clip_move() {
    let (mut sess, mut selected, mut playhead) = session();
    let key_x = f64::from(bx_default(8.0));
    let key_y = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        key_x,
        key_y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(9.0),
        key_y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar(9.0),
        key_y,
        0,
    );

    let snapshot_scene = sess.scene.clone();
    let snapshot_selected = selected;
    let snapshot_playhead = playhead;

    let press_x = lx_for_bar(10.0);
    let y = body_top() + f64::from(ROW) + 5.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        press_x,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        press_x - 200.0,
        y,
        0,
    );
    assert!(sess.scene.snap_guide.is_some());

    let cancel = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Cancel,
        press_x - 200.0,
        y,
        0,
    );
    assert!(cancel.edit_commit.is_none());
    assert_eq!(sess.scene, snapshot_scene);
    assert_eq!(selected, snapshot_selected);
    assert!((playhead - snapshot_playhead).abs() < 1e-9);
}

#[test]
fn trim_changes_edges_only_and_respects_min_length() {
    let (mut sess, mut selected, mut playhead) = session();
    // band1 clip0 hero 4..22。右端22にkeyは無い。
    let y = body_top() + f64::from(ROW) + 5.0;
    let end_x = f64::from(bx_default(22.0));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        end_x,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(10.0),
        y,
        0,
    );
    let clip = &sess.scene.bands[1].clips[0];
    assert!((clip.a - 4.0).abs() < 1e-4);
    assert!((clip.b - 10.0).abs() < 1e-4);

    // 最小長 1 frame へ押し込み
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(3.0),
        y,
        0,
    );
    let clip = &sess.scene.bands[1].clips[0];
    assert!((clip.b - (4.0 + 1.0 / 30.0)).abs() < 1e-4);

    // TrimStart: band1 clip1 26..40。左端にkeyは無い。keysは不変。
    // playhead既定0.27≈bar25.92がTRIM端に重なるため遠ざける(F1優先)。
    let (mut sess, mut selected, mut playhead) = session();
    playhead = 12.0 / 96.0;
    let keys_before = sess.scene.bands[1].clips[1].keys.clone();
    let start_x = f64::from(bx_default(26.0));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        start_x,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(28.0),
        y,
        0,
    );
    let clip = &sess.scene.bands[1].clips[1];
    assert!((clip.a - 28.0).abs() < 1e-4);
    assert!((clip.b - 40.0).abs() < 1e-4);
    assert_eq!(clip.keys, keys_before);
}

#[test]
fn trim_hit_uses_inside_handle_width_and_keeps_key_priority() {
    let mut scene = TimelineScene::default();
    let band = 1usize;
    let clip = 0usize;
    let y = body_top() + f64::from(ROW) * 1.5 - 0.5;
    let ax = f64::from(bx(&scene, scene.bands[band].clips[clip].a));

    assert_eq!(
        timeline_hover_hit(&scene, 0.0, 1240, 400, ax + 14.9, y),
        TimelineHoverHit::Trim
    );
    assert_eq!(
        timeline_hover_hit(&scene, 0.0, 1240, 400, ax + 15.1, y),
        TimelineHoverHit::Clip
    );
    assert_eq!(
        timeline_hover_hit(&scene, 0.0, 1240, 400, ax - 0.1, y),
        TimelineHoverHit::None
    );

    let key_time = scene.bands[band].clips[clip].a;
    scene.bands[band].clips[clip]
        .keys
        .push((key_time, 0.42, false, 99));
    assert_eq!(
        timeline_hover_hit(&scene, 0.0, 1240, 400, ax, y),
        TimelineHoverHit::Key
    );
}

#[test]
fn trim_edge_width_obeys_clip_width_and_height_cutoffs() {
    assert_eq!(trim_edge_width(24.9, 19.0), None);
    assert_eq!(trim_edge_width(25.0, 15.9), None);
    assert_eq!(trim_edge_width(25.0, 16.0), Some(6.25));
    assert_eq!(trim_edge_width(59.0, 19.0), Some(14.75));
    assert_eq!(trim_edge_width(60.0, 19.0), Some(15.0));
    assert_eq!(trim_edge_width(120.0, 19.0), Some(15.0));
}

#[test]
fn real_trim_hit_inside_fifteen_px_releases_existing_commit_once() {
    for (left, move_bar) in [(true, 3.0), (false, 7.0)] {
        let mut sess = TimelineSession::default();
        sess.scene = TimelineScene::from_snapshot(
            &[SnapshotLayerInput {
                layer_id: "trim".into(),
                display_name: "clip".into(),
                interval_secs: Some((2.0, 6.0)),
                keys: vec![],
            }],
            Some("trim"),
        );
        let mut selected = 0;
        let mut playhead = 0.0;
        let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
        let (a, b, _) = sess.scene.clip0_span_and_keys(0).unwrap();
        let edge_x = if left {
            f64::from(bx(&sess.scene, a)) + 10.0
        } else {
            f64::from(bx(&sess.scene, b)) - 10.0
        };
        let move_x = lx_for_bar_in(&sess.scene, move_bar);

        let down = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            edge_x,
            y,
            0,
        );
        assert!(down.edit_commit.is_none());
        let moved = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Move,
            move_x,
            y,
            0,
        );
        assert!(moved.edit_commit.is_none());
        let up = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            move_x,
            y,
            0,
        );
        let expected = if left {
            TimelineEditCommit::TrimClipIn {
                layer_id: "trim".into(),
                bar: move_bar as f32,
            }
        } else {
            TimelineEditCommit::TrimClipOut {
                layer_id: "trim".into(),
                bar: move_bar as f32,
            }
        };
        assert_eq!(up.edit_commit, Some(expected));

        let duplicate = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Up,
            move_x,
            y,
            0,
        );
        assert!(duplicate.edit_commit.is_none());
    }
}

#[test]
fn narrow_real_clip_edge_stays_body_move() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "narrow".into(),
            display_name: "clip".into(),
            interval_secs: Some((4.0, 1.0)),
            keys: vec![],
        }],
        Some("narrow"),
    );
    let mut selected = 0;
    let mut playhead = 0.0;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let (a, b, _) = sess.scene.clip0_span_and_keys(0).unwrap();
    let ax = f64::from(bx(&sess.scene, a));
    let bx_ = f64::from(bx(&sess.scene, b));
    assert!(bx_ - ax < TRIM_EDGE_MIN_CLIP_W);

    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        ax + 1.0,
        y,
        1,
    );
    let move_x = ax + 11.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        move_x,
        y,
        1,
    );
    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        move_x,
        y,
        1,
    );
    assert!(matches!(
        up.edit_commit,
        Some(TimelineEditCommit::SetClipStart { ref layer_id, .. }) if layer_id == "narrow"
    ));
}
