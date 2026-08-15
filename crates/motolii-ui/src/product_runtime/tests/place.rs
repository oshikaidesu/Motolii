use super::*;
use crate::{AsciiKey, CommandId, KeymapDelta, Modifier, PlatformBindingConstraints};
use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_doc::{DocKeyframe, DocKeyframeTrack};
use motolii_transport::Transport;

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
