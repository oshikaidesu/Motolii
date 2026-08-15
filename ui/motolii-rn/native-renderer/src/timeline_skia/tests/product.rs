use super::*;

#[test]
fn hover_hit_maps_to_cursor_kinds() {
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::Trim, false),
        CursorKind::ResizeLeftRight
    );
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::PlayheadOrRuler, false),
        CursorKind::ResizeLeftRight
    );
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::Key, false),
        CursorKind::PointingHand
    );
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::Clip, false),
        CursorKind::OpenHand
    );
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::Clip, true),
        CursorKind::ClosedHand
    );
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::None, false),
        CursorKind::Arrow
    );
    // drag中はhitを外れてもclosedHand維持(gesture active優先)。
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::None, true),
        CursorKind::ClosedHand
    );
    assert_eq!(
        cursor_for_timeline_hover(TimelineHoverHit::Trim, true),
        CursorKind::ClosedHand
    );
    assert_eq!(cursor_for_stage_hover(true, false), CursorKind::OpenHand);
    assert_eq!(cursor_for_stage_hover(true, true), CursorKind::ClosedHand);
    assert_eq!(cursor_for_stage_hover(false, true), CursorKind::ClosedHand);
    assert_eq!(cursor_for_stage_hover(false, false), CursorKind::Arrow);

    let scene = TimelineScene::default();
    let playhead = 12.0 / 96.0;
    let px = f64::from(bx(&scene, 12.0));
    let body_y = body_top() + f64::from(ROW) + 5.0;
    assert_eq!(
        timeline_hover_hit(&scene, playhead, 1240, 400, px, body_y),
        TimelineHoverHit::PlayheadOrRuler
    );
    assert_eq!(
        timeline_hover_hit(&scene, playhead, 1240, 400, lx_for_bar(10.0), body_y),
        TimelineHoverHit::Clip
    );
    let ruler_y = f64::from(OVER_H + 1.0) + 4.0;
    assert_eq!(
        timeline_hover_hit(&scene, 0.0, 1240, 400, lx_for_bar(20.0), ruler_y),
        TimelineHoverHit::PlayheadOrRuler
    );
}

#[test]
fn first_absolute_tick_snaps_fractional_view_to_bar_multiples() {
    assert_eq!(first_absolute_tick(0.0, 4), 0);
    assert_eq!(first_absolute_tick(0.1, 4), 4);
    assert_eq!(first_absolute_tick(2.3, 4), 4);
    assert_eq!(first_absolute_tick(4.0, 4), 4);
    assert_eq!(first_absolute_tick(4.01, 4), 8);
    assert_eq!(first_absolute_tick(1.0, 8), 8);
}

#[test]
fn ruler_label_step_is_integer_frames() {
    let scene = TimelineScene::default();
    let step = ruler_label_step_secs(&scene, 1032.0);
    let frame = frame_duration_secs(&scene);
    let frames = step / frame;
    assert!(
        (frames - frames.round()).abs() < 1e-4,
        "step={step} frame={frame}"
    );
    assert_eq!(format_ruler_time(0.0, &scene, false), "0:00");
    assert_eq!(format_ruler_time(1.0, &scene, true), "0:01:00");
}

#[test]
fn snap_to_frame_at_30fps_lands_on_frame_grid() {
    let scene = TimelineScene::default();
    assert!((snap_to_frame(&scene, 1.0) - 1.0).abs() < 1e-5);
    assert!((snap_to_frame(&scene, 1.0 + 1.0 / 60.0) - 1.0).abs() < 1e-5);
    assert!((snap_to_frame(&scene, 1.0 + 1.0 / 30.0) - (1.0 + 1.0 / 30.0)).abs() < 1e-5);
}

#[test]
fn product_snap_bar_uses_fps_frames() {
    let scene = TimelineScene::empty_host().with_fps(30, 1);
    assert!(scene.real);
    let half = test_snap_bar(&scene, 0.0, 0, None, 1.0 + 1.0 / 60.0, 0);
    assert!((half - 1.0).abs() < 1e-5, "snap={half}");
    let on_frame = 1.0 + 1.0 / 30.0;
    let landed = test_snap_bar(&scene, 0.0, 0, None, on_frame, 0);
    assert!((landed - on_frame).abs() < 1e-5, "snap={landed}");
}

#[test]
fn product_ruler_format_is_timecode_not_bar() {
    let scene = TimelineScene::empty_host().with_fps(30, 1);
    assert!(scene.real);
    let twelve = format_ruler_time(12.0, &scene, false);
    assert_eq!(twelve, "0:12");
    assert!(!twelve.to_ascii_lowercase().contains("bar"));
    assert_eq!(format_ruler_time(12.0, &scene, true), "0:12:00");
    assert_eq!(format_ruler_time(72.0, &scene, false), "1:12");
}

#[test]
fn key_hit_radius_matches_outer_stroke_5_6() {
    assert!((KEY_HIT_PX - 5.6).abs() < f64::EPSILON);
}

#[test]
fn key_hit_boundary_is_5_6px() {
    let scene = TimelineScene::default();
    // band 0 clip 1のkey(bar 20.0)。playheadは遠くへ置き優先順位の干渉を避ける。
    let playhead = 0.9;
    let kx = f64::from(bx(&scene, 20.0));
    let row_cy = body_top() + 0.5 * f64::from(ROW) - 0.5;
    assert_eq!(
        timeline_hover_hit(&scene, playhead, 1240, 400, kx + 5.5, row_cy),
        TimelineHoverHit::Key
    );
    assert_eq!(
        timeline_hover_hit(&scene, playhead, 1240, 400, kx, row_cy + 5.5),
        TimelineHoverHit::Key
    );
    // 5.6pxの外はkeyではない(clip本体へ落ちる)。
    assert_ne!(
        timeline_hover_hit(&scene, playhead, 1240, 400, kx + 5.7, row_cy),
        TimelineHoverHit::Key
    );
    assert_ne!(
        timeline_hover_hit(&scene, playhead, 1240, 400, kx, row_cy + 5.7),
        TimelineHoverHit::Key
    );
}

#[test]
fn empty_real_scene_reserves_guide_row_and_filled_scene_does_not() {
    let mut scene = TimelineScene::default();
    scene.real = true;
    scene.bands.clear();
    assert!((empty_real_guide_rows(&scene) - ROW).abs() < f32::EPSILON);

    // layerが1件でも入れば消える。
    let filled = TimelineScene::default();
    if !filled.bands.is_empty() {
        let mut real_filled = filled;
        real_filled.real = true;
        assert_eq!(empty_real_guide_rows(&real_filled), 0.0);
    }

    // fixture(real=false)では空でも出さない(PNG sha不変の根拠)。
    let mut fixture = TimelineScene::default();
    fixture.real = false;
    fixture.bands.clear();
    assert_eq!(empty_real_guide_rows(&fixture), 0.0);
}

#[test]
fn host_empty_scene_has_no_fixture_clips() {
    let scene = TimelineScene::empty_host();
    assert!(scene.real);
    assert!(scene.bands.is_empty());
    assert!(scene.locators.is_empty());
    assert!(!scene.bands.iter().any(|band| {
        band.clips.iter().any(|clip| {
            clip.name == "sky_plate"
                || clip.name == "hero"
                || clip.name == "track_master.wav"
                || clip.name == "street_loop.mp4"
                || clip.layer_id.is_empty() && !clip.name.is_empty()
        })
    }));
    let product = TimelineSession::host_product();
    assert!(product.scene.real);
    assert!(product.scene.bands.is_empty());
    assert!(product.scene.locators.is_empty());
    assert_ne!(product.scene.real, TimelineScene::default().real);
}

#[test]
fn host_snapshot_layer_id_is_the_visible_clip_identity() {
    let scene = TimelineScene::from_snapshot(
        &[SnapshotLayerInput {
            layer_id: "42".into(),
            display_name: "Rectangle".into(),
            interval_secs: Some((0.0, 10.0)),
            keys: vec![],
        }],
        Some("42"),
    );
    assert!(scene.real);
    assert_eq!(scene.bands.len(), 1);
    assert_eq!(scene.bands[0].clips[0].layer_id, "42");
    assert_eq!(scene.bands[0].clips[0].name, "Rectangle");
    assert_eq!(scene.selected_flat, 0);
}
