use super::*;
use motolii_core::{ColorSpace, FrameDesc, PixelFormat};

fn test_layout(epoch: u64) -> NativeHostLayout {
    test_layout_with(epoch, crate::layout::PanelLayout::built_in())
}

fn test_layout_with(epoch: u64, authority: crate::layout::PanelLayout) -> NativeHostLayout {
    let frame =
        FrameDesc::try_packed(1920, 1080, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true).unwrap();
    NativeHostLayout::try_new(epoch, 1000, 800, 1.0, frame, &authority)
        .unwrap()
        .unwrap()
}

fn test_source() -> BrowserPlaceIntent {
    BrowserPlaceIntent {
        scope_ref: "builtin-stable".to_owned(),
        item_id: "rectangle".to_owned(),
    }
}

#[test]
fn stage_projection_accepts_only_the_latest_new_generation() {
    let one = RenderGeneration::new(1).unwrap();
    let two = RenderGeneration::new(2).unwrap();
    let mut projection = ProductStageProjection::default();

    assert!(!projection.accepts(one, Some(two)));
    assert!(projection.accepts(two, Some(two)));
    projection.commit(two);
    assert!(!projection.accepts(two, Some(two)));
    assert!(!projection.accepts(one, Some(one)));
}

#[test]
fn timeline_projection_uses_the_document_envelope_without_owned_range_state() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let bar = projection.projection.bars().first().unwrap();

    assert_eq!(projection.projection.bars().len(), 1);
    assert_eq!(bar.x_start, 0.0);
    assert_eq!(bar.x_end, 1.0);
    assert_eq!(projection.band_span, 1.0);
}

#[test]
fn timeline_time_surface_reuses_the_typed_projection_hit_and_excludes_chrome() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let expected_layer = projection.projection.bars()[0].layer;
    let layout = test_layout(9);
    let timeline = layout.timeline.unwrap();
    let time_surface = timeline_time_surface_logical_rect(layout).unwrap();
    let center = [
        time_surface.x + time_surface.width / 2.0,
        time_surface.y + time_surface.height / 2.0,
    ];

    assert_eq!(
        projection.hit_test(center, layout),
        Some(TimelineHit::Bar {
            layer: expected_layer
        })
    );
    assert_eq!(
        projection.hit_test([timeline.x + 100.0, time_surface.y + 10.0], layout),
        None
    );
    assert_eq!(
        projection.hit_test([timeline.x + 220.0, time_surface.y + 10.0], layout),
        None
    );
    assert_eq!(
        projection.hit_test([time_surface.x + 10.0, timeline.y + 15.0], layout),
        None
    );
    assert_eq!(
        projection.hit_test([time_surface.x + 10.0, timeline.y + 40.0], layout),
        None
    );
    assert_eq!(
        projection.hit_test([layout.stage.x, layout.stage.y], layout),
        None
    );
}

#[test]
fn timeline_ruler_maps_to_the_same_viewport_without_becoming_content_hit_input() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let layout = test_layout(10);
    let ruler = timeline_ruler_logical_rect(layout).unwrap();
    let midpoint = [ruler.x + ruler.width / 2.0, ruler.y + ruler.height / 2.0];

    assert_eq!(
        projection.ruler_time_at(midpoint, layout),
        document
            .composition
            .duration
            .try_mul(RationalTime::try_new(1, 2).unwrap())
            .ok()
    );
    assert_eq!(projection.hit_test(midpoint, layout), None);
    assert_eq!(
        projection.ruler_time_at([ruler.x - 1.0, midpoint[1]], layout),
        None
    );
}

#[test]
fn timeline_interval_press_distinguishes_in_move_and_out_on_the_existing_bar() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let layout = test_layout(9);
    let surface = timeline_time_surface_logical_rect(layout).unwrap();
    let y = surface.y + surface.height / 2.0;

    assert_eq!(
        projection
            .interval_press_target([surface.x + 1.0, y], layout)
            .map(|target| target.kind),
        Some(IntervalGestureKind::TrimIn)
    );
    assert_eq!(
        projection
            .interval_press_target([surface.x + surface.width / 2.0, y], layout)
            .map(|target| target.kind),
        Some(IntervalGestureKind::Move)
    );
    assert_eq!(
        projection
            .interval_press_target([surface.x + surface.width - 1.0, y], layout)
            .map(|target| target.kind),
        Some(IntervalGestureKind::TrimOut)
    );
}

#[test]
fn interval_move_preview_snaps_both_edges_with_no_document_write() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let bar = projection.projection.bars()[0];
    let active = ActiveIntervalGesture {
        generation: 1,
        target: IntervalPressTarget {
            layer: bar.layer,
            kind: IntervalGestureKind::Move,
            start: bar.start,
            end: bar.end,
        },
        grab_time: RationalTime::from_seconds(5),
        layout_epoch: 9,
        projection_generation: 0,
    };
    let preview = interval_preview_candidate(
        active,
        RationalTime::try_new(501, 100).unwrap(),
        test_layout(9),
        &projection,
        &document,
    )
    .unwrap();

    assert_eq!(preview.layer, bar.layer);
    assert_eq!(preview.start, bar.start);
    assert_eq!(preview.end, bar.end);
}

#[test]
fn interval_cancel_clears_only_transient_state_and_enqueues_no_document_work() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let bar = projection.projection.bars()[0];
    let mut active = Some(ActiveIntervalGesture {
        generation: 1,
        target: IntervalPressTarget {
            layer: bar.layer,
            kind: IntervalGestureKind::Move,
            start: bar.start,
            end: bar.end,
        },
        grab_time: bar.start,
        layout_epoch: 9,
        projection_generation: 0,
    });
    let mut preview = Some(TimelineIntervalPreview {
        layer: bar.layer,
        start: bar.start,
        end: bar.end,
    });
    let queue = DocumentEditQueue::default();

    clear_interval_gesture(&mut active, &mut preview);

    assert!(active.is_none());
    assert!(preview.is_none());
    assert_eq!(queue.len(), 0);
}

#[test]
fn equal_distance_item_edge_beats_frame_before_stable_identity_ties() {
    let delta = RationalTime::try_new(1, 30).unwrap();
    let frame = SnapCandidate {
        delta,
        target_kind: SnapTargetKind::Frame,
        target_time: RationalTime::from_seconds(1),
        other_layer: None,
        target_edge: IntervalEdge::In,
        moving_edge: IntervalEdge::In,
    };
    let item = SnapCandidate {
        delta,
        target_kind: SnapTargetKind::OtherClip,
        target_time: RationalTime::from_seconds(1),
        other_layer: Some(LayerId::from_raw(9)),
        target_edge: IntervalEdge::Out,
        moving_edge: IntervalEdge::Out,
    };
    let lower_identity = SnapCandidate {
        other_layer: Some(LayerId::from_raw(4)),
        ..item
    };

    assert!(snap_candidate_is_better(item, frame));
    assert!(snap_candidate_is_better(lower_identity, item));
}

#[test]
fn hidden_timeline_has_no_selection_hit() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let mut authority = crate::layout::PanelLayout::built_in();
    authority
        .apply(
            crate::layout::LayoutAction::Hide(crate::layout::PanelRole::Timeline),
            crate::layout::LayoutConstraints {
                viewport_width: 1_000.0,
                stage_min_width: 320.0,
            },
        )
        .unwrap();
    let layout = test_layout_with(10, authority);

    assert_eq!(projection.hit_test([500.0, 700.0], layout), None);
}

#[test]
fn product_history_shortcuts_resolve_to_stable_command_ids() {
    let registry = builtin_command_registry().unwrap();
    let keymap = product_command_keymap(&registry).unwrap();
    let z = KeyToken::Ascii(AsciiKey::try_new('z').unwrap());
    let undo = EffectiveTrigger::Keyboard {
        key: z,
        modifiers: Modifiers::try_new([Modifier::Meta]).unwrap(),
        phase: InputPhase::Press,
    };
    let redo = EffectiveTrigger::Keyboard {
        key: z,
        modifiers: Modifiers::try_new([Modifier::Meta, Modifier::Shift]).unwrap(),
        phase: InputPhase::Press,
    };

    assert_eq!(
        keymap.get(&undo).map(CommandId::as_str),
        Some("motolii.edit.undo")
    );
    assert_eq!(
        keymap.get(&redo).map(CommandId::as_str),
        Some("motolii.edit.redo")
    );
    assert!(keymap.diagnostics().is_empty());
}

#[test]
fn active_effect_candidate_prefers_attach_and_clears_on_primary_change() {
    let primary = motolii_doc::LayerId::from_raw(7);
    let other_primary = motolii_doc::LayerId::from_raw(8);
    let previous = EffectId::from_raw(10);
    let attached = EffectId::from_raw(11);

    assert_eq!(
        active_effect_candidate(
            Some(primary),
            Some(previous),
            Some(primary),
            Some(attached),
            |candidate_primary, candidate| {
                candidate_primary == primary && candidate == attached
            },
        ),
        Some(attached)
    );
    assert_eq!(
        active_effect_candidate(
            Some(primary),
            Some(previous),
            Some(other_primary),
            Some(attached),
            |_, _| true,
        ),
        None
    );
}

#[test]
fn active_effect_candidate_does_not_resurrect_after_disappearance() {
    let primary = motolii_doc::LayerId::from_raw(7);
    let effect = EffectId::from_raw(10);

    let after_removal =
        active_effect_candidate(Some(primary), Some(effect), Some(primary), None, |_, _| {
            false
        });
    assert_eq!(after_removal, None);
    assert_eq!(
        active_effect_candidate(
            Some(primary),
            after_removal,
            Some(primary),
            None,
            |candidate_primary, candidate| { candidate_primary == primary && candidate == effect },
        ),
        None
    );
}

#[test]
fn moved_progress_creates_a_nonterminal_preview_phase() {
    let layout = test_layout(9);
    let center = [
        layout.stage.x + layout.stage.width / 2.0,
        layout.stage.y + layout.stage.height / 2.0,
    ];
    let source = test_source();
    let mut phase = PlacePreviewPhase::default();

    phase.deliver(&source, 4, center, layout);

    assert_eq!(
        phase.latest,
        Some(PlacePreviewProgress {
            source,
            generation: 4,
            layout_epoch: 9,
            stage_ndc: Some([0.0, 0.0]),
        })
    );
}

#[test]
fn outside_progress_updates_preview_without_becoming_a_terminal() {
    let layout = test_layout(9);
    let source = test_source();
    let mut phase = PlacePreviewPhase::default();
    phase.deliver(&source, 4, [10.0, 10.0], layout);

    assert_eq!(
        phase.latest,
        Some(PlacePreviewProgress {
            source,
            generation: 4,
            layout_epoch: 9,
            stage_ndc: None,
        })
    );
    phase.clear();
    assert_eq!(phase.latest, None);
}

#[test]
fn release_inside_stage_has_no_noncommit_cause() {
    let layout = test_layout(9);
    let position = [
        layout.stage.x + layout.stage.width / 2.0,
        layout.stage.y + layout.stage.height / 2.0,
    ];

    assert_eq!(
        ClassifiedPlaceTerminal::released(test_source(), 4, position, layout),
        ClassifiedPlaceTerminal {
            source: test_source(),
            generation: 4,
            cause: PlaceTerminalCause::NoNonCommitCause,
            layout_epoch: Some(9),
            stage_ndc: Some([0.0, 0.0]),
        }
    );
}

#[test]
fn release_outside_stage_has_outside_cause() {
    let layout = test_layout(9);

    assert_eq!(
        ClassifiedPlaceTerminal::released(test_source(), 4, [10.0, 10.0], layout),
        ClassifiedPlaceTerminal {
            source: test_source(),
            generation: 4,
            cause: PlaceTerminalCause::OutsideStage,
            layout_epoch: Some(9),
            stage_ndc: None,
        }
    );
}

#[test]
fn cancellation_reason_maps_exhaustively_to_noncommit_cause() {
    for (reason, cause) in [
        (HostPointerCancel::Escape, PlaceTerminalCause::Escape),
        (
            HostPointerCancel::CaptureLost,
            PlaceTerminalCause::CaptureLoss,
        ),
    ] {
        assert_eq!(
            ClassifiedPlaceTerminal::cancelled(test_source(), 4, reason),
            ClassifiedPlaceTerminal {
                source: test_source(),
                generation: 4,
                cause,
                layout_epoch: None,
                stage_ndc: None,
            }
        );
    }
}

#[test]
fn admission_accepts_at_most_one_matching_commit_candidate() {
    let layout = test_layout(9);
    let position = [
        layout.stage.x + layout.stage.width / 2.0,
        layout.stage.y + layout.stage.height / 2.0,
    ];
    let terminal = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
    let mut admission = PlaceTerminalAdmission::default();

    assert!(admission.begin(4));
    assert!(admission.admit(&terminal));
    assert!(!admission.admit(&terminal));
    assert!(!admission.begin(4));
}

#[test]
fn noncommit_terminal_retires_generation_without_admission() {
    let terminal =
        ClassifiedPlaceTerminal::cancelled(test_source(), 4, HostPointerCancel::CaptureLost);
    let mut admission = PlaceTerminalAdmission::default();

    assert!(admission.begin(4));
    assert!(!admission.admit(&terminal));
    assert!(!admission.admit(&terminal));
    assert!(!admission.begin(4));
}

#[test]
fn stale_terminal_does_not_retire_the_current_drag() {
    let layout = test_layout(9);
    let position = [
        layout.stage.x + layout.stage.width / 2.0,
        layout.stage.y + layout.stage.height / 2.0,
    ];
    let stale = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
    let current = ClassifiedPlaceTerminal::released(test_source(), 5, position, layout);
    let mut admission = PlaceTerminalAdmission::default();

    assert!(admission.begin(4));
    assert!(admission.admit(&stale));
    assert!(admission.begin(5));
    assert!(!admission.admit(&stale));
    assert!(admission.admit(&current));
}

#[test]
fn retained_high_water_rejects_replay_after_detail_eviction() {
    let layout = test_layout(9);
    let position = [
        layout.stage.x + layout.stage.width / 2.0,
        layout.stage.y + layout.stage.height / 2.0,
    ];
    let terminal = ClassifiedPlaceTerminal::released(test_source(), 8, position, layout);
    let mut admission = PlaceTerminalAdmission::default();

    assert!(admission.begin(8));
    assert!(admission.admit(&terminal));
    assert!(!admission.begin(7));
    assert!(!admission.begin(8));
    assert!(admission.begin(9));
}

#[test]
fn admitted_terminal_delivers_once_to_the_single_pending_boundary() {
    let layout = test_layout(9);
    let position = [
        layout.stage.x + layout.stage.width / 2.0,
        layout.stage.y + layout.stage.height / 2.0,
    ];
    let terminal = ClassifiedPlaceTerminal::released(test_source(), 4, position, layout);
    let mut delivery = PlaceTerminalDelivery::default();

    let delivered = delivery.deliver(&terminal).unwrap();
    assert_eq!(delivered.generation, 4);
    assert_eq!(delivered.layout_epoch, 9);
    assert_eq!(delivered.ndc, [0.0, 0.0]);
    assert!(delivery.deliver(&terminal).is_none());
}

#[test]
fn unadmitted_causes_cannot_enter_the_delivery_boundary() {
    let mut delivery = PlaceTerminalDelivery::default();
    for reason in [HostPointerCancel::Escape, HostPointerCancel::CaptureLost] {
        let terminal = ClassifiedPlaceTerminal::cancelled(test_source(), 4, reason);
        assert!(delivery.deliver(&terminal).is_none());
    }
}

#[test]
fn lifecycle_replaces_with_new_epochs_and_ignores_old_callbacks() {
    let mut lifecycle = BrowserLifecycleCoordinator::new(7).unwrap();

    assert_eq!(
        lifecycle
            .observe(
                7,
                BrowserLifecycleEvent::ReloadStarted { instance_epoch: 7 }
            )
            .unwrap(),
        BrowserRecoveryDecision::Replace { instance_epoch: 8 }
    );
    assert_eq!(
        lifecycle
            .observe(
                8,
                BrowserLifecycleEvent::ReloadStarted { instance_epoch: 7 }
            )
            .unwrap(),
        BrowserRecoveryDecision::Ignore
    );
    assert_eq!(
        lifecycle
            .observe(
                8,
                BrowserLifecycleEvent::ReloadStarted { instance_epoch: 8 }
            )
            .unwrap(),
        BrowserRecoveryDecision::Replace { instance_epoch: 9 }
    );
}

#[test]
fn automatic_process_recovery_is_bounded_to_one_replacement() {
    let mut lifecycle = BrowserLifecycleCoordinator::new(10).unwrap();

    assert_eq!(
        lifecycle
            .observe(
                10,
                BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 10 }
            )
            .unwrap(),
        BrowserRecoveryDecision::Replace { instance_epoch: 11 }
    );
    assert_eq!(
        lifecycle
            .observe(
                11,
                BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 11 }
            )
            .unwrap(),
        BrowserRecoveryDecision::Degrade
    );
    assert_eq!(
        lifecycle
            .observe(
                11,
                BrowserLifecycleEvent::ReloadStarted { instance_epoch: 11 }
            )
            .unwrap(),
        BrowserRecoveryDecision::Ignore
    );
}

#[test]
fn lifecycle_epoch_exhaustion_is_typed() {
    assert!(matches!(
        BrowserLifecycleCoordinator::new(u64::MAX),
        Err(ProductRuntimeError::BrowserInstanceEpochExhausted)
    ));
}
