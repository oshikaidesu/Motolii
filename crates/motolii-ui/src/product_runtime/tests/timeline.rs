use super::*;
use crate::{AsciiKey, CommandId, KeymapDelta, Modifier, PlatformBindingConstraints};
use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_doc::{DocKeyframe, DocKeyframeTrack};
use motolii_transport::Transport;

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
fn editor_playhead_starts_at_zero_and_retains_release_value() {
    let mut playhead = EditorPlayhead::default();
    let interior = RationalTime::try_new(3, 2).unwrap();

    assert_eq!(playhead.current, RationalTime::ZERO);
    assert!(playhead.begin(9, interior));
    assert!(!playhead.finish(9));
    assert_eq!(playhead.current, interior);
    assert!(playhead.scrub.is_none());
}

#[test]
fn editor_playhead_cancel_and_layout_change_restore_press_value() {
    let mut playhead = EditorPlayhead::default();
    let press = RationalTime::try_new(1, 2).unwrap();
    let moved = RationalTime::try_new(3, 2).unwrap();

    assert!(playhead.begin(7, press));
    assert_eq!(playhead.update(7, moved), Some(true));
    assert!(playhead.cancel());
    assert_eq!(playhead.current, RationalTime::ZERO);
    assert!(playhead.begin(8, press));
    assert_eq!(playhead.update(9, moved), None);
    assert!(playhead.cancel());
    assert_eq!(playhead.current, RationalTime::ZERO);
}

#[test]
fn editor_playhead_publish_retirement_preserves_current_value() {
    let mut playhead = EditorPlayhead::default();
    let press = RationalTime::try_new(1, 2).unwrap();
    let current = RationalTime::try_new(3, 2).unwrap();

    assert!(playhead.begin(7, press));
    assert_eq!(playhead.update(7, current), Some(true));
    assert!(playhead.retire());
    assert_eq!(playhead.current, current);
    assert!(playhead.scrub.is_none());
}

#[test]
fn playhead_scrub_arms_existing_escape_and_safety_cancel_lifecycle() {
    let registry = builtin_command_registry().unwrap();
    let cancel = CommandId::try_new("motolii.gesture.cancel").unwrap();
    let mut router = InputRouter::new(registry);

    router
        .route(NormalizedInput::Phase(InputPhase::DragStart))
        .unwrap();
    assert!(matches!(
        router
            .route(NormalizedInput::Command {
                phase: InputPhase::Press,
                id: cancel.clone(),
            })
            .unwrap(),
        RouterOutput::Intent {
            intent: DomainIntent::CancelInFlightGesture,
            ..
        }
    ));
    router
        .route(NormalizedInput::Phase(InputPhase::DragStart))
        .unwrap();
    assert!(matches!(
        router
            .route(NormalizedInput::SafetyInterrupt(
                SafetyInterrupt::WindowFocusLost
            ))
            .unwrap(),
        RouterOutput::SafetyCancel {
            intent: DomainIntent::CancelInFlightGesture,
            ..
        }
    ));
}

#[test]
fn ruler_mapping_is_closed_clamped_and_excludes_non_ruler_inputs() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let layout = test_layout(9);
    let ruler = timeline_ruler_logical_rect(layout).unwrap();
    let duration = document.composition.duration;
    let y = ruler.y + ruler.height / 2.0;

    assert_eq!(
        projection.ruler_time_at([ruler.x, y], layout, true),
        Some(RationalTime::ZERO)
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x + ruler.width / 2.0, y], layout, true),
        Some(
            duration
                .try_mul(RationalTime::try_new(1, 2).unwrap())
                .unwrap()
        )
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x + ruler.width, y], layout, true),
        Some(duration)
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x - 1.0, y], layout, true),
        None
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x + 1.0, ruler.y - 1.0], layout, true),
        None
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x + 1.0, ruler.y + ruler.height + 1.0], layout, true),
        None
    );
    assert_eq!(projection.ruler_time_at([f64::NAN, y], layout, true), None);
    let mut missing_timeline = layout;
    missing_timeline.timeline = None;
    assert_eq!(
        projection.ruler_time_at([ruler.x, y], missing_timeline, true),
        None
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x - 1.0, y], layout, false),
        Some(RationalTime::ZERO)
    );
    assert_eq!(
        projection.ruler_time_at([ruler.x + ruler.width + 1.0, y], layout, false),
        Some(duration)
    );
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
        Some(ProductTimelineHit::Body {
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
fn timeline_private_hit_refines_only_admitted_bar_edges() {
    let document = crate::static_preview::bootstrap_document().unwrap();
    let projection = ProductTimelineProjection::from_document(&document).unwrap();
    let layer = projection.projection.bars()[0].layer;
    let layout = test_layout(9);
    let surface = timeline_time_surface_logical_rect(layout).unwrap();
    let y = surface.y + surface.height / 2.0;

    assert_eq!(
        projection.hit_test([surface.x + 15.0, y], layout),
        Some(ProductTimelineHit::Left { layer })
    );
    assert_eq!(
        projection.hit_test([surface.x + 15.001, y], layout),
        Some(ProductTimelineHit::Body { layer })
    );
    assert_eq!(
        projection.hit_test([surface.x + surface.width - 15.0, y], layout),
        Some(ProductTimelineHit::Right { layer })
    );

    let timeline = layout.timeline.unwrap();
    let horizontal_chrome = timeline.width - surface.width;
    let mut narrow = layout;
    narrow.timeline.as_mut().unwrap().width = horizontal_chrome + 24.999;
    let narrow_surface = timeline_time_surface_logical_rect(narrow).unwrap();
    assert_eq!(
        projection.hit_test(
            [
                narrow_surface.x + 1.0,
                narrow_surface.y + narrow_surface.height / 2.0
            ],
            narrow,
        ),
        Some(ProductTimelineHit::Body { layer })
    );
    narrow.timeline.as_mut().unwrap().width = horizontal_chrome + 25.0;
    let cutoff_surface = timeline_time_surface_logical_rect(narrow).unwrap();
    assert_eq!(
        projection.hit_test(
            [
                cutoff_surface.x + 1.0,
                cutoff_surface.y + cutoff_surface.height / 2.0
            ],
            narrow,
        ),
        Some(ProductTimelineHit::Left { layer })
    );

    let vertical_chrome = timeline.height - surface.height;
    let mut short = layout;
    short.timeline.as_mut().unwrap().height = vertical_chrome + 15.999;
    let short_surface = timeline_time_surface_logical_rect(short).unwrap();
    assert_eq!(
        projection.hit_test(
            [
                short_surface.x + 1.0,
                short_surface.y + short_surface.height / 2.0
            ],
            short,
        ),
        Some(ProductTimelineHit::Body { layer })
    );
    short.timeline.as_mut().unwrap().height = vertical_chrome + 16.0;
    let height_cutoff_surface = timeline_time_surface_logical_rect(short).unwrap();
    assert_eq!(
        projection.hit_test(
            [
                height_cutoff_surface.x + 1.0,
                height_cutoff_surface.y + height_cutoff_surface.height / 2.0,
            ],
            short,
        ),
        Some(ProductTimelineHit::Left { layer })
    );
}

#[test]
fn timeline_trim_rejects_stale_generation_changed_interval_and_target_loss() {
    let interval = (
        RationalTime::try_new(2, 10).unwrap(),
        RationalTime::try_new(8, 10).unwrap(),
    );
    let gesture = TimelineTrimGesture::begin(
        LayerId::from_raw(7),
        TimelineTrimEdge::Left,
        RationalTime::try_new(3, 10).unwrap(),
        interval.0,
        interval.1,
        4,
    );

    assert!(timeline_trim_gesture_is_current(
        &gesture,
        4,
        Some(interval)
    ));
    assert!(!timeline_trim_gesture_is_current(
        &gesture,
        5,
        Some(interval)
    ));
    assert!(!timeline_trim_gesture_is_current(
        &gesture,
        4,
        Some((interval.0, RationalTime::try_new(9, 10).unwrap())),
    ));
    assert!(!timeline_trim_gesture_is_current(&gesture, 4, None));
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
fn product_shortcuts_resolve_to_stable_command_ids() {
    let registry = builtin_command_registry().unwrap();
    let base = product_builtin_keymap();
    let keymap = resolve_keymap(
        &base,
        &KeymapDelta::default(),
        &PlatformBindingConstraints::new(PlatformCommandModifier::Meta, Vec::new()),
        &registry,
    );
    let z = KeyToken::Ascii(AsciiKey::try_new('z').unwrap());
    let none = Modifiers::default();
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
    let cancel = EffectiveTrigger::Keyboard {
        key: KeyToken::Escape,
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let modified_cancel = EffectiveTrigger::Keyboard {
        key: KeyToken::Escape,
        modifiers: Modifiers::try_new([Modifier::Shift]).unwrap(),
        phase: InputPhase::Press,
    };
    let delete = EffectiveTrigger::Keyboard {
        key: KeyToken::Delete,
        modifiers: none.clone(),
        phase: InputPhase::Press,
    };
    let backspace = EffectiveTrigger::Keyboard {
        key: KeyToken::Backspace,
        modifiers: none,
        phase: InputPhase::Press,
    };

    assert_eq!(base.version, crate::PRODUCT_BUILTIN_KEYMAP_VERSION);
    assert_eq!(
        keymap.get(&undo).map(CommandId::as_str),
        Some("motolii.edit.undo")
    );
    assert_eq!(
        keymap.get(&redo).map(CommandId::as_str),
        Some("motolii.edit.redo")
    );
    assert_eq!(
        keymap.get(&cancel).map(CommandId::as_str),
        Some("motolii.gesture.cancel")
    );
    assert_eq!(
        keymap.get(&delete).map(CommandId::as_str),
        Some("motolii.edit.delete_targeted_items")
    );
    assert_eq!(
        keymap.get(&backspace).map(CommandId::as_str),
        Some("motolii.edit.delete_targeted_items")
    );
    assert_eq!(keymap.get(&modified_cancel), None);
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
