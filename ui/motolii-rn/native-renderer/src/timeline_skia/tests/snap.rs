use super::*;

fn real_projection_move_snaps_to_other_clip_edge_and_commits_snapped_start() {
    let mut sess = TimelineSession::default();
    let mut scene = TimelineScene::default();
    scene.real = true;
    scene.bands[0].clips[0].a = 0.0;
    scene.bands[0].clips[0].b = 7.3;
    scene.bands[0].clips[0].layer_id = "move-a".into();
    scene.bands[0].clips[1].a = 9.0;
    scene.bands[0].clips[1].b = 15.0;
    scene.bands[0].clips[1].layer_id = "move-b".into();
    scene.bands[0].clips[2].a = 18.0;
    scene.bands[0].clips[2].b = 25.0;
    scene.bands[0].clips[2].layer_id = "move-c".into();
    sess.scene = scene;

    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        lx_for_bar_in(&sess.scene, 10.0),
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
        lx_for_bar_in(&sess.scene, 7.5),
        y,
        0,
    );
    assert!(moved.edit_commit.is_none());
    assert!(
        (sess.scene.bands[0].clips[1].a - 7.3).abs() < 1e-3,
        "got {}",
        sess.scene.bands[0].clips[1].a
    );
    assert_eq!(sess.scene.snap_guide, Some(7.3));
    let mut without_guide = sess.scene.clone();
    without_guide.snap_guide = None;
    let mut baseline = vec![0u8; 1240 * 400 * 4];
    let mut preview = vec![0u8; 1240 * 400 * 4];
    draw_timeline(&without_guide, &mut baseline, 1240, 400, playhead, selected);
    draw_timeline(&sess.scene, &mut preview, 1240, 400, playhead, selected);
    assert_ne!(preview, baseline, "snap guide must be visible during drag");

    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, 7.5),
        y,
        0,
    );
    assert_eq!(
        up.edit_commit,
        Some(TimelineEditCommit::SetClipStart {
            layer_id: "move-b".into(),
            bar: 7.3,
        })
    );
    assert!(sess.scene.snap_guide.is_none());
}

#[test]
fn real_projection_move_snap_ignores_cmd_key_modifier() {
    let mut sess = TimelineSession::default();
    let mut scene = TimelineScene::default();
    scene.real = true;
    scene.bands[0].clips[0].a = 0.0;
    scene.bands[0].clips[0].b = 7.3;
    scene.bands[0].clips[0].layer_id = "move-a".into();
    scene.bands[0].clips[1].a = 9.0;
    scene.bands[0].clips[1].b = 15.0;
    scene.bands[0].clips[1].layer_id = "move-b".into();
    sess.scene = scene;

    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let down_bar = 10.0;
    let move_bar = 10.6;
    let down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        lx_for_bar_in(&sess.scene, down_bar),
        y,
        0,
    );
    assert!(down.edit_commit.is_none());
    assert_eq!(selected, 1);
    let moved = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar_in(&sess.scene, move_bar),
        y,
        1,
    );
    assert!(moved.edit_commit.is_none());
    assert!(sess.scene.snap_guide.is_none());
    let expected_bar = 9.6_f32;
    assert!(
        (sess.scene.bands[0].clips[1].a - expected_bar).abs() < 1e-3,
        "got {}",
        sess.scene.bands[0].clips[1].a
    );

    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, move_bar),
        y,
        1,
    );
    assert_eq!(
        up.edit_commit,
        Some(TimelineEditCommit::SetClipStart {
            layer_id: "move-b".into(),
            bar: expected_bar,
        })
    );
}

#[test]
fn real_projection_trim_snaps_to_frame_and_commits() {
    let mut sess = TimelineSession::default();
    let mut scene = TimelineScene::default();
    scene.real = true;
    scene.bands[0].clips[0].a = 0.0;
    scene.bands[0].clips[0].b = 7.3;
    scene.bands[0].clips[0].layer_id = "trim-a".into();
    scene.bands[0].clips[1].a = 9.0;
    scene.bands[0].clips[1].b = 16.0;
    scene.bands[0].clips[1].layer_id = "trim-b".into();
    scene.bands[0].clips[2].a = 18.0;
    scene.bands[0].clips[2].b = 22.0;
    scene.bands[0].clips[2].layer_id = "trim-c".into();
    sess.scene = scene;

    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        f64::from(bx(&sess.scene, 16.0)),
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
        lx_for_bar_in(&sess.scene, 17.9),
        y,
        0,
    );
    assert!(moved.edit_commit.is_none());
    assert!(
        (sess.scene.bands[0].clips[1].b - 17.9).abs() < 1e-3,
        "got {}",
        sess.scene.bands[0].clips[1].b
    );

    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, 17.9),
        y,
        0,
    );
    match up.edit_commit {
        Some(TimelineEditCommit::TrimClipOut { layer_id, bar }) => {
            assert_eq!(layer_id, "trim-b");
            assert!((bar - 17.9).abs() < 1e-3);
        }
        other => panic!("expected TrimClipOut at 17.9s, got {other:?}"),
    }
}

#[test]
fn real_projection_key_drag_snaps_to_playhead_and_commits() {
    let mut sess = TimelineSession::default();
    let mut scene = TimelineScene::default();
    scene.real = true;
    scene.bands[1].clips[0].layer_id = "11".into();
    sess.scene = scene;

    let mut selected = 0;
    let mut playhead = 4.7 / f64::from(SONG_BARS);
    let y = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
    let key_id = sess.scene.bands[1].clips[0].keys[0].3;
    let down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        f64::from(bx(&sess.scene, sess.scene.bands[1].clips[0].keys[0].0)),
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
        lx_for_bar_in(&sess.scene, 4.6),
        y,
        0,
    );
    assert!(moved.edit_commit.is_none());
    assert!((sess.scene.bands[1].clips[0].keys[0].0 - 4.6).abs() < 1e-3);

    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, 4.6),
        y,
        0,
    );
    let edit_bar = match up.edit_commit {
        Some(TimelineEditCommit::SetPositionKeyTime {
            layer_id,
            key_id: up_key_id,
            bar,
        }) if layer_id.as_str() == "11" && up_key_id == key_id => bar,
        _ => {
            panic!(
                "expected key drag commit for layer 11 key {key_id}, got {:?}",
                up.edit_commit
            )
        }
    };
    assert!((edit_bar - 4.6).abs() < 1e-3);
}

#[test]
fn snap_threshold_tracks_6px_distance_under_zoom_changes_bar_delta() {
    let base = TimelineScene::default().with_fps(1, 1);
    let band = 4usize;
    let integer_bar = 10.0f32;
    let near6_base = 6.0 / surface_width() as f32 * (base.view_b - base.view_a);
    let far7_base = 7.0 / surface_width() as f32 * (base.view_b - base.view_a);
    assert!(
        (test_snap_bar(&base, 0.0, band, None, integer_bar + near6_base * 0.9, 0,) - integer_bar)
            .abs()
            < 2e-3
    );
    assert!(
        (test_snap_bar(&base, 0.0, band, None, integer_bar + far7_base, 0)
            - (integer_bar + far7_base))
            .abs()
            < 2e-3
    );

    let mut sess = TimelineSession::default();
    sess.scene = sess.scene.clone().with_fps(1, 1);
    sess.scroll(1240, 400, 0.0, 0.0, 0.5, 0, lx_for_bar(24.0), 100.0);
    let half = &sess.scene;
    let near6_half = 6.0 / surface_width() as f32 * (half.view_b - half.view_a);
    let far7_half = 7.0 / surface_width() as f32 * (half.view_b - half.view_a);
    assert!(near6_base > near6_half);
    assert!(
        (test_snap_bar(half, 0.0, band, None, integer_bar + near6_half * 0.9, 0) - integer_bar)
            .abs()
            < 2e-3
    );
    assert!(
        (test_snap_bar(half, 0.0, band, None, integer_bar + far7_half, 0)
            - (integer_bar + far7_half))
            .abs()
            < 2e-3
    );
}
