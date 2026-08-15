use super::*;

fn key_drag_moves_time_only_and_selects_single_key() {
    let (mut sess, mut selected, mut playhead) = session();
    // band1 hero key at 8.0
    let kx = f64::from(bx_default(8.0));
    let ky = body_top() + f64::from(ROW) + f64::from(ROW - 1.0) / 2.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        kx,
        ky,
        0,
    );
    assert_eq!(selected, 3);
    assert!(sess.scene.bands[1].clips[0].keys[0].2);
    assert!(!sess.scene.bands[0].clips[1].keys[1].2); // previous true cleared

    // 水平移動 + 縦ノイズ
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(11.0),
        ky + 40.0,
        0,
    );
    let key = &sess.scene.bands[1].clips[0].keys[0];
    assert!((key.0 - 11.0).abs() < 1e-4);
    assert!((key.1 - 0.71).abs() < 1e-4);

    // clamp to clip.b
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(40.0),
        ky,
        0,
    );
    assert!((sess.scene.bands[1].clips[0].keys[0].0 - 22.0).abs() < 1e-4);
}

#[test]
fn real_projection_key_drag_clamps_to_clip_start() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "12".into(),
            display_name: "keyed".into(),
            interval_secs: Some((4.0, 6.0)),
            keys: vec![SnapshotKeyInput {
                key_id: 11,
                time_secs: 8.0,
            }],
        }],
        Some("12"),
    );
    assert!(sess.scene.real);
    let mut selected = 0;
    let mut playhead = 0.27;
    let clip = sess.scene.bands[0].clips[0].clone();
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let x = f64::from(bx(&sess.scene, clip.keys[0].0));
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
    let left_x = lx_for_bar_in(&sess.scene, f64::from(clip.a) - 10.0);
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        left_x,
        y,
        0,
    );
    assert!((sess.scene.bands[0].clips[0].keys[0].0 - 4.0).abs() < 1e-4);
}

#[test]
fn real_projection_key_drag_commits_once_on_release_with_clamped_time() {
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
    assert!(sess.scene.real);
    let mut selected = 0;
    let mut playhead = 0.27;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let x = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].keys[0].0));
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
    assert!(down.edit_commit.is_none());
    let moved = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar_in(&sess.scene, 40.0),
        y,
        0,
    );
    assert!(moved.edit_commit.is_none());
    // clip span 0..10s。clamp to clip.b=10
    assert!((sess.scene.bands[0].clips[0].keys[0].0 - 10.0).abs() < 1e-3);

    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, 40.0),
        y,
        0,
    );
    assert_eq!(
        up.edit_commit,
        Some(TimelineEditCommit::SetPositionKeyTime {
            layer_id: "11".into(),
            key_id: 7,
            bar: sess.scene.bands[0].clips[0].keys[0].0,
        })
    );
    // 二度目のUpではgesture無し → commitなし(二重dispatch防止)
    let up2 = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar_in(&sess.scene, 40.0),
        y,
        0,
    );
    assert!(up2.edit_commit.is_none());
}

#[test]
fn real_projection_move_cancel_restores_full_state_without_commit() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "3".into(),
            display_name: "a".into(),
            interval_secs: Some((2.0, 4.0)), // bars 1..3、移動余地あり
            keys: vec![],
        }],
        Some("3"),
    );
    let mut selected = 0;
    let mut playhead = 0.1;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let before = sess.scene.clone();
    let before_selected = selected;
    let before_playhead = playhead;
    let origin_a = before.bands[0].clips[0].a;
    let origin_b = before.bands[0].clips[0].b;

    let mid = f64::from(bx(&sess.scene, (origin_a + origin_b) * 0.5));
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
        mid + 24.0,
        y,
        0,
    );
    assert_ne!(sess.scene.bands[0].clips[0].a, before.bands[0].clips[0].a);

    let cancel = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Cancel,
        mid + 24.0,
        y,
        0,
    );
    assert!(cancel.edit_commit.is_none());
    assert_eq!(sess.scene, before);
    assert_eq!(selected, before_selected);
    assert!((playhead - before_playhead).abs() < 1e-9);
}

#[test]
fn real_projection_move_trim_commit_on_release_cancel_restores_without_commit() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[
            SnapshotLayerInput {
                layer_id: "3".into(),
                display_name: "a".into(),
                interval_secs: Some((2.0, 4.0)), // bars 1..3
                keys: vec![],
            },
            SnapshotLayerInput {
                layer_id: "4".into(),
                display_name: "b".into(),
                interval_secs: Some((0.0, 4.0)), // bars 0..2
                keys: vec![],
            },
        ],
        Some("3"),
    );
    let mut selected = 0;
    let mut playhead = 40.0 / f64::from(SONG_BARS);
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let origin_a = sess.scene.bands[0].clips[0].a;
    let origin_b = sess.scene.bands[0].clips[0].b;

    // move: mid-clip drag
    let mid = f64::from(bx(&sess.scene, (origin_a + origin_b) * 0.5));
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
    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        mid + 40.0,
        y,
        0,
    );
    assert!(matches!(
        up.edit_commit,
        Some(TimelineEditCommit::SetClipStart { .. })
    ));

    // trim left then cancel restores, no commit
    let left = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].a));
    let before = sess.scene.clone();
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        left,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        left + 24.0,
        y,
        0,
    );
    let cancel = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Cancel,
        left + 24.0,
        y,
        0,
    );
    assert!(cancel.edit_commit.is_none());
    assert_eq!(sess.scene.bands[0].clips[0].a, before.bands[0].clips[0].a);
    assert_eq!(sess.scene.bands[0].clips[0].b, before.bands[0].clips[0].b);

    // trim right release commits
    let right = f64::from(bx(&sess.scene, sess.scene.bands[0].clips[0].b));
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        right,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        right - 20.0,
        y,
        0,
    );
    let trim_up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        right - 20.0,
        y,
        0,
    );
    assert!(
        matches!(
            trim_up.edit_commit,
            Some(TimelineEditCommit::TrimClipOut { .. })
        ),
        "got {:?}",
        trim_up.edit_commit
    );
}
