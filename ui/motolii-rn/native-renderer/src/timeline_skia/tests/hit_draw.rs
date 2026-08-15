use super::*;

#[test]
fn hit_test_returns_the_clip_under_the_pointer() {
    // 幅1240 → scale 1.0。band 0 の2本目 sky_plate は bar 14..27。
    let scene = TimelineScene::default();
    let scale = scale_for(1240);
    assert!((scale - 1.0).abs() < f32::EPSILON);
    let x = lx_for_bar(20.0);
    let y = body_top() + 5.0;
    assert_eq!(hit_test(&scene, 1240, 400, x, y).map(|hit| hit.0), Some(1));
}

#[test]
fn hit_test_rejects_the_header_columns_and_the_rulers() {
    let scene = TimelineScene::default();
    assert_eq!(hit_test(&scene, 1240, 400, 10.0, 100.0), None);
    assert_eq!(hit_test(&scene, 1240, 400, 600.0, 5.0), None);
}

#[test]
fn drag_move_left_of_first_clip_is_clamped_to_zero_left_bound() {
    let (mut sess, mut selected, mut playhead) = session();
    let press_x = lx_for_bar(10.0);
    let y = body_top() + 5.0; // band0 の clip0
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
        lx_for_bar(-20.0),
        y,
        0,
    );
    let clip = &sess.scene.bands[0].clips[0];
    assert!((clip.a - 0.0).abs() < 1e-4);
    assert!((clip.b - 14.0).abs() < 1e-4);
}

#[test]
fn drag_move_of_band0_clip1_is_clamped_to_prev_neighbor_start() {
    let (mut sess, mut selected, mut playhead) = session();
    let press_x = lx_for_bar(20.0);
    let y = body_top() + 5.0; // band0 の clip1
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
        lx_for_bar(1.0),
        y,
        0,
    );
    let clip = &sess.scene.bands[0].clips[1];
    assert!((clip.a - 14.0).abs() < 1e-4);
    assert!((clip.b - 27.0).abs() < 1e-4);
}

#[test]
fn trim_cannot_cross_neighbor_clip_edges() {
    let (mut sess, mut selected, mut playhead) = session();
    let y = body_top() + 5.0; // band0

    let clip0_end_x = f64::from(bx_default(14.0)) + 1.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        clip0_end_x,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(20.0),
        y,
        0,
    );
    let clip0 = &sess.scene.bands[0].clips[0];
    assert!((clip0.a - 0.0).abs() < 1e-4);
    assert!((clip0.b - 14.0).abs() < 1e-4);

    let clip1_start_x = f64::from(bx_default(14.0)) + 1.0;
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        clip1_start_x,
        y,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        lx_for_bar(1.0),
        y,
        0,
    );
    let clip1 = &sess.scene.bands[0].clips[1];
    assert!((clip1.a - 14.0).abs() < 1e-4);
    assert!((clip1.b - 27.0).abs() < 1e-4);
}

#[test]
fn scrub_does_not_fire_on_inbox_column_even_with_ruler_y() {
    let (mut sess, mut selected, mut playhead) = session();
    let out = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        50.0,
        f64::from(OVER_H + 1.0) + 4.0,
        0,
    );
    assert!(!out.feedback);
    assert_eq!(selected, 1);
    assert!((playhead - 0.27).abs() < 1e-9);
}

#[test]
fn scrub_follows_time_ruler() {
    let (mut sess, mut selected, mut playhead) = session();
    let y = body_bottom(&TimelineScene::default()) + 4.0;
    let out = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        lx_for_bar(33.0),
        y,
        0,
    );
    assert!(out.feedback);
    assert!(out.scrub_playhead.is_some());
    assert!(!out.scrub_release);
    assert!((playhead - 33.0 / 96.0).abs() < 1e-9);
    let up = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        lx_for_bar(33.0),
        y,
        0,
    );
    assert!(up.scrub_release);
    assert!((up.scrub_playhead.unwrap() - 33.0 / 96.0).abs() < 1e-9);
}

#[test]
fn hit_test_reports_no_clip_inside_the_empty_band() {
    // 5番目の帯は空。当たらないので -1。
    let scene = TimelineScene::default();
    let y = body_top() + f64::from(ROW) * 4.5;
    assert_eq!(
        hit_test(&scene, 1240, 400, 600.0, y).map(|hit| hit.0),
        Some(-1)
    );
}

#[test]
fn fixture_trim_down_keeps_selection_playhead_and_feedback() {
    let (mut sess, mut selected, mut playhead) = session();
    sess.scene.bands[0].clips[0].a = 8.0;
    sess.scene.bands[0].clips[0].b = 16.0;
    let orig_selected = selected;
    let orig_playhead = playhead;
    let y = body_top() + 5.0;
    let down_cases = [
        f64::from(bx(&sess.scene, 8.0)) + 1.0,
        f64::from(bx(&sess.scene, 16.0)) - 1.0,
    ];
    for x in down_cases {
        let out = sess.pointer(
            &mut selected,
            &mut playhead,
            1240,
            400,
            TimelinePointerPhase::Down,
            x,
            y,
            0,
        );
        assert!(!out.feedback);
        assert!(!out.dirty);
        assert_eq!(selected, orig_selected);
        assert!((playhead - orig_playhead).abs() < 1e-9);
        assert_eq!(sess.scene.bands[0].clips[0].a, 8.0);
        assert_eq!(sess.scene.bands[0].clips[0].b, 16.0);
    }
}

/// 決定的疑似乱数のgesture嵐。panicと不変条件破壊を狩る(seed固定で再現可能)。
#[test]
fn deterministic_gesture_storm_holds_invariants() {
    // LCG(固定seed)。Date/外部乱数へ依存しない。
    let mut state: u64 = 0x00C0FFEE_5EED_1234;
    let mut next = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as u32
    };
    let mut rf = {
        let mut n = next;
        move |lo: f64, hi: f64| lo + (n() as f64 / u32::MAX as f64) * (hi - lo)
    };

    let scenes: Vec<TimelineScene> = vec![
        TimelineScene::default(),
        TimelineScene::from_snapshot_with_song_bars(
            &[
                SnapshotLayerInput {
                    layer_id: "1".into(),
                    display_name: "a".into(),
                    interval_secs: Some((0.0, 10.0)),
                    keys: vec![
                        SnapshotKeyInput {
                            key_id: 10,
                            time_secs: 2.0,
                        },
                        SnapshotKeyInput {
                            key_id: 11,
                            time_secs: 7.0,
                        },
                    ],
                },
                SnapshotLayerInput {
                    layer_id: "2".into(),
                    display_name: "b".into(),
                    interval_secs: Some((1.0, 4.0)),
                    keys: vec![],
                },
            ],
            Some("1"),
            10.0,
        ),
    ];

    for (w, h) in [(1240u32, 400u32), (2480, 620), (620, 200)] {
        let mut session = TimelineSession::default();
        let mut selected = 1i32;
        let mut playhead = 0.5f64;
        for step in 0..120_000u32 {
            let roll = next() % 100;
            if roll < 80 {
                let phase = match next() % 4 {
                    0 => TimelinePointerPhase::Down,
                    1 => TimelinePointerPhase::Move,
                    2 => TimelinePointerPhase::Up,
                    _ => TimelinePointerPhase::Cancel,
                };
                let x = rf(-200.0, w as f64 + 200.0);
                let y = rf(-200.0, h as f64 + 200.0);
                let m = next() % 2;
                let _ = session.pointer(&mut selected, &mut playhead, w, h, phase, x, y, m);
            } else if roll < 95 {
                let _ = session.scroll(
                    w,
                    h,
                    rf(-500.0, 500.0),
                    rf(-500.0, 500.0),
                    if next() % 4 == 0 { rf(-0.5, 0.5) } else { 0.0 },
                    next() % 2,
                    rf(0.0, w as f64),
                    rf(0.0, h as f64),
                );
            } else {
                // gesture中も含む任意タイミングのscene差し替え(P0-2経路)。
                session.scene = scenes[(next() % scenes.len() as u32) as usize].clone();
            }

            // 不変条件: viewとplayheadとclip幾何が常に健全。
            let sb = session.scene.song_bars;
            assert!(
                playhead.is_finite() && (0.0..=1.0).contains(&playhead),
                "step {step}"
            );
            assert!(session.scene.view_a.is_finite() && session.scene.view_b.is_finite());
            assert!(session.scene.view_a >= -0.001 && session.scene.view_b <= sb + 0.001);
            assert!(session.scene.view_b > session.scene.view_a);
            for band in &session.scene.bands {
                for clip in &band.clips {
                    assert!(clip.a.is_finite() && clip.b.is_finite());
                    assert!(clip.b >= clip.a, "clip inverted at step {step}");
                    for key in &clip.keys {
                        assert!(key.0.is_finite());
                    }
                }
            }
        }
    }
}

/// `MOTOLII_WRITE_PREVIEW=1 cargo test -- --nocapture` でpanel実寸のPNGを書く。
/// 見た目の確認用で、判定はしない(probeなのでgoldenを持たない)。
#[test]
fn writes_a_panel_sized_preview_when_asked() {
    if std::env::var_os("MOTOLII_WRITE_PREVIEW").is_none() {
        return;
    }
    let (w, h) = (2480u32, 620u32);
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    draw_timeline(&TimelineScene::default(), &mut bytes, w, h, 0.22, 1);
    let info = ImageInfo::new(
        (w as i32, h as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface = surfaces::wrap_pixels(&info, &mut bytes, Some(w as usize * 4), None).unwrap();
    let png = surface
        .image_snapshot()
        .encode(None, skia_safe::EncodedImageFormat::PNG, 100)
        .unwrap();
    std::fs::write("timeline-rn-probe-preview.png", png.as_bytes()).unwrap();
    println!("timeline-rn-probe-preview.png {w}x{h}");
}

#[test]
fn draw_timeline_fills_the_requested_buffer() {
    let (w, h) = (620u32, 200u32);
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    draw_timeline(&TimelineScene::default(), &mut bytes, w, h, 0.5, 1);
    assert!(bytes.iter().any(|byte| *byte != 0));
}

#[test]
fn scrub_follows_ruler_and_cancel_restores_press_playhead() {
    let (mut sess, mut selected, mut playhead) = session();
    let (x0, y0) = phys(lx_for_bar(12.0), f64::from(OVER_H + 1.0) + 4.0);
    let out = sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x0,
        y0,
        0,
    );
    assert!(out.feedback);
    let pressed = playhead;
    assert!((pressed - 12.0 / 96.0).abs() < 1e-9);

    let (x1, y1) = phys(lx_for_bar(24.0), f64::from(OVER_H + 1.0) + 4.0);
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        x1,
        y1,
        0,
    );
    assert!((playhead - 24.0 / 96.0).abs() < 1e-9);

    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Up,
        x1,
        y1,
        0,
    );
    assert!((playhead - 24.0 / 96.0).abs() < 1e-9);

    // 新しいscrubをcancelで戻す
    let (mut sess, mut selected, mut playhead) = session();
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Down,
        x0,
        y0,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Move,
        x1,
        y1,
        0,
    );
    sess.pointer(
        &mut selected,
        &mut playhead,
        1240,
        400,
        TimelinePointerPhase::Cancel,
        x1,
        y1,
        0,
    );
    assert!((playhead - 0.27).abs() < 1e-9);
    assert_eq!(selected, 1);
}
