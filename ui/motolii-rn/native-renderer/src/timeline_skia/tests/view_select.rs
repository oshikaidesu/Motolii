use super::*;

#[test]
fn wheel_pan_moves_view_and_clamps_to_song() {
    let (mut sess, _, _) = session();
    // 右へpan(負delta → view増加)。論理48px → span*48/sw bars
    let dirty = sess.scroll(1240, 400, -48.0, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);
    assert!(dirty);
    let expected = 48.0 / surface_width() as f32 * 48.0;
    assert!((sess.scene.view_a - expected).abs() < 1e-3);
    assert!((sess.scene.view_b - (48.0 + expected)).abs() < 1e-3);

    // 大きく右へ飛ばして右端clamp
    sess.scroll(1240, 400, -10_000.0, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);
    assert!((sess.scene.view_a - 48.0).abs() < 1e-3);
    assert!((sess.scene.view_b - 96.0).abs() < 1e-3);

    // 大きく左へ飛ばして左端clamp
    sess.scroll(1240, 400, 10_000.0, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);
    assert!((sess.scene.view_a - 0.0).abs() < 1e-3);
    assert!((sess.scene.view_b - 48.0).abs() < 1e-3);
}

#[test]
fn cmd_wheel_zoom_keeps_anchor_bar_fixed() {
    let (mut sess, _, _) = session();
    let anchor_bar = 24.0;
    let lx = lx_for_bar(anchor_bar);
    let before_bar = bar_at_lx(&sess.scene, lx);
    assert!((before_bar - anchor_bar).abs() < 1e-6);

    assert!(sess.scroll(1240, 400, 0.0, 12.0, 0.0, 1, lx, 100.0));
    assert!((bar_at_lx(&sess.scene, lx) - anchor_bar).abs() < 1e-3);
}

#[test]
fn wheel_vertical_delta_has_priority_for_horizontal_pan() {
    let (mut sess, _, _) = session();
    sess.scene.view_a = 24.0;
    sess.scene.view_b = 72.0;
    let before_a = sess.scene.view_a;
    let before_b = sess.scene.view_b;

    let dirty = sess.scroll(1240, 400, 1.0, -48.0, 0.0, 0, lx_for_bar(24.0), 100.0);
    assert!(dirty);
    let expected = before_a + (-(-48.0) / surface_width() * f64::from(before_b - before_a)) as f32;
    assert!((sess.scene.view_a - expected).abs() < 1e-3);
    assert!((sess.scene.view_b - (before_b - before_a + expected)).abs() < 1e-3);
}

#[test]
fn draw_timeline_inbox_pixels_stable_after_pan() {
    let w = 1240u32;
    let h = 620u32;
    let mut before = vec![0u8; (w * h * 4) as usize];
    draw_timeline(&TimelineScene::default(), &mut before, w, h, 0.22, 1);

    let mut sess = TimelineSession::default();
    let delta = -surface_width() / f64::from(SONG_BARS) * 20.0;
    sess.scroll(w, h, delta, 0.0, 0.0, 0, lx_for_bar(24.0), 100.0);

    let mut after = vec![0u8; (w * h * 4) as usize];
    draw_timeline(&sess.scene, &mut after, w, h, 0.22, 1);

    let x = 60usize;
    let y = body_top() as usize + 5;
    let idx = (y * w as usize + x) * 4;
    assert_eq!(&before[idx..idx + 4], &after[idx..idx + 4]);
}

#[test]
fn pinch_zoom_keeps_anchor_bar_and_clamps_span() {
    let (mut sess, _, _) = session();
    let anchor_bar = 24.0;
    let lx = lx_for_bar(anchor_bar);
    // magnification=0.5 → span *= 0.5 → 24
    assert!(sess.scroll(1240, 400, 0.0, 0.0, 0.5, 0, lx, 100.0));
    assert!((sess.scene.view_b - sess.scene.view_a - 24.0).abs() < 1e-3);
    assert!((bar_at_lx(&sess.scene, lx) - anchor_bar).abs() < 1e-3);

    // 大きく拡大してmin span 4
    assert!(sess.scroll(1240, 400, 0.0, 0.0, 0.9, 0, lx, 100.0));
    assert!((sess.scene.view_b - sess.scene.view_a - 4.0).abs() < 1e-3);

    // 大きく縮小してmax span 96
    sess.scene.view_a = 20.0;
    sess.scene.view_b = 40.0;
    let lx2 = lx_for_bar_in(&sess.scene, 30.0);
    assert!(sess.scroll(1240, 400, 0.0, 0.0, -10.0, 0, lx2, 100.0));
    assert!((sess.scene.view_a - 0.0).abs() < 1e-3);
    assert!((sess.scene.view_b - 96.0).abs() < 1e-3);
}

#[test]
fn overview_drag_centers_view_and_clamps() {
    let (mut sess, mut selected, mut playhead) = session();
    // overview上で曲中央(bar 48)へ
    let ox_48 = f64::from(SURF_X) + 48.0 / f64::from(SONG_BARS) * surface_width();
    let out = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        ox_48,
        5.0,
        0,
    );
    assert!(out.dirty);
    assert!(!out.feedback);
    assert!((sess.scene.view_a - 24.0).abs() < 1e-3);
    assert!((sess.scene.view_b - 72.0).abs() < 1e-3);

    // 右端へ → view 48..96
    let ox_96 = f64::from(SURF_X) + surface_width();
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        ox_96,
        5.0,
        0,
    );
    assert!((sess.scene.view_a - 48.0).abs() < 1e-3);
    assert!((sess.scene.view_b - 96.0).abs() < 1e-3);
}

#[test]
fn playhead_is_song_normalized_and_survives_view_change() {
    let (mut sess, mut selected, mut playhead) = session();
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        lx_for_bar(24.0),
        f64::from(OVER_H + 1.0) + 4.0,
        0,
    );
    assert!((playhead - 24.0 / 96.0).abs() < 1e-9);
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar(24.0),
        f64::from(OVER_H + 1.0) + 4.0,
        0,
    );

    // viewを後半へ移しても同じpointer barでscrubが正しい
    sess.scene.view_a = 48.0;
    sess.scene.view_b = 96.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        lx_for_bar_in(&sess.scene, 72.0),
        f64::from(OVER_H + 1.0) + 4.0,
        0,
    );
    assert!((playhead - 72.0 / 96.0).abs() < 1e-9);
}

#[test]
fn clip_hit_works_after_view_moves_to_second_half() {
    let (mut sess, mut selected, mut playhead) = session();
    sess.scene.view_a = 48.0;
    sess.scene.view_b = 96.0;
    sess.scene.bands[5].clips[0].a = 60.0;
    sess.scene.bands[5].clips[0].b = 80.0;
    let x = lx_for_bar_in(&sess.scene, 70.0);
    let y = body_top() + f64::from(ROW) * 5.5;
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
    // band0..4 flat: 3+2+3+2+0 = 10
    assert_eq!(selected, 10);
}

#[test]
fn overview_cancel_restores_view() {
    let (mut sess, mut selected, mut playhead) = session();
    let before = (sess.scene.view_a, sess.scene.view_b);
    let ox_48 = f64::from(SURF_X) + 48.0 / f64::from(SONG_BARS) * surface_width();
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        ox_48,
        5.0,
        0,
    );
    assert!((sess.scene.view_a - 24.0).abs() < 1e-3);
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Cancel,
        ox_48,
        5.0,
        0,
    );
    assert!((sess.scene.view_a - before.0).abs() < 1e-6);
    assert!((sess.scene.view_b - before.1).abs() < 1e-6);
}

#[test]
fn real_clip_down_emits_select_layer_once_and_empty_bar_clears() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "42".into(),
            display_name: "clip".into(),
            interval_secs: Some((2.0, 8.0)),
            keys: vec![],
        }],
        None,
    );
    assert!(sess.scene.real);
    let mut selected = -1;
    let mut playhead = 0.1;
    let y = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let x = f64::from(bx(&sess.scene, 3.0));
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
            layer_id: "42".into(),
        })
    );
    assert_eq!(selected, 0);

    let empty_x = f64::from(bx(&sess.scene, 20.0));
    let clear = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        empty_x,
        y,
        0,
    );
    assert_eq!(
        clear.selection_commit,
        Some(TimelineSelectionCommit::ClearSelection)
    );
    assert_eq!(selected, -1);
}

#[test]
fn fixture_clip_down_does_not_emit_selection_commit() {
    let (mut sess, mut selected, mut playhead) = session();
    assert!(!sess.scene.real);
    let y = body_top() + 5.0;
    let x = lx_for_bar(2.0);
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
    assert!(down.selection_commit.is_none());
    assert!(selected >= 0);
}

#[test]
fn lane_drag_commits_reparent_to_destination_band() {
    let mut sess = TimelineSession::default();
    sess.scene = TimelineScene::from_snapshot(
        &[
            SnapshotLayerInput {
                layer_id: "src".into(),
                display_name: "a".into(),
                interval_secs: Some((0.0, 4.0)),
                keys: vec![],
            },
            SnapshotLayerInput {
                layer_id: "dst".into(),
                display_name: "b".into(),
                interval_secs: Some((0.0, 4.0)),
                keys: vec![],
            },
        ],
        Some("src"),
    );
    let mut selected = 0;
    let mut playhead = 40.0 / f64::from(SONG_BARS);
    let x = f64::from(bx(&sess.scene, 2.0));
    let y0 = body_top() + f64::from(ROW) * 0.5 - 0.5;
    let y1 = body_top() + f64::from(ROW) * 1.5 - 0.5;
    let down = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x,
        y0,
        0,
    );
    assert!(down.edit_commit.is_none());
    let moved = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        x,
        y1,
        0,
    );
    assert!(moved.edit_commit.is_none());
    assert_eq!(sess.scene.lane_preview_band, Some(1));
    assert!(sess.scene.snap_guide.is_none());
    let mut without_preview = sess.scene.clone();
    without_preview.lane_preview_band = None;
    let mut baseline = vec![0u8; 1240 * 400 * 4];
    let mut preview = vec![0u8; 1240 * 400 * 4];
    draw_timeline(
        &without_preview,
        &mut baseline,
        1240,
        400,
        playhead,
        selected,
    );
    draw_timeline(&sess.scene, &mut preview, 1240, 400, playhead, selected);
    assert_ne!(
        preview, baseline,
        "destination lane feedback must be visible"
    );
    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        x,
        y1,
        0,
    );
    match up.edit_commit {
        Some(TimelineEditCommit::ReparentClip {
            layer_id,
            dest_layer_id,
            bar,
        }) => {
            assert_eq!(layer_id, "src");
            assert_eq!(dest_layer_id, "dst");
            assert!((bar - 0.0).abs() < 1e-3);
        }
        other => panic!("expected ReparentClip, got {other:?}"),
    }
    assert!(sess.scene.lane_preview_band.is_none());
    assert!(sess.scene.snap_guide.is_none());
    let second_up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        x,
        y1,
        0,
    );
    assert!(second_up.edit_commit.is_none());
}

#[test]
fn real_key_down_emits_select_layer_for_owner_clip() {
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
        None,
    );
    let mut selected = -1;
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
    assert_eq!(
        down.selection_commit,
        Some(TimelineSelectionCommit::SelectLayer {
            layer_id: "11".into(),
        })
    );
}
