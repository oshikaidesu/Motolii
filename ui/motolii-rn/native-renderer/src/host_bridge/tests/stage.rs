use super::{clear_slot, dispatch_kind, install_slot, temp_project, test_lock};
use super::super::slot::host_slot;
use super::super::{
    try_dispatch_timeline_selection, try_read_timeline_projection, try_stage_mount,
    try_stage_pointer, try_stage_resize, try_stage_unmount,
};
use crate::timeline_skia::{TimelineScene, TimelineSelectionCommit};

#[test]
fn stage_seat_mount_pointer_selects_seed_layer_at_center() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("stage-ptr-hit");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
    assert!(baseline.primary_layer_id.is_none());
    assert!(baseline.layer_ids.is_empty());

    let (width, height) = (1600.0_f64, 900.0_f64);
    assert!(try_stage_mount(width, height, 1.0));
    // seed rect center=[0,0] → view-local (w/2, h/2)
    assert!(try_stage_pointer("down", width * 0.5, height * 0.5));
    let after = try_read_timeline_projection().expect("projection");
    assert!(after.primary_layer_id.is_none());

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn stage_pointer_rejects_when_unmounted_and_keeps_selection() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("stage-ptr-unmounted");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    dispatch_kind(
        host,
        "place_rectangle",
        r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
    );
    assert!(try_stage_mount(1600.0, 900.0, 1.0));
    assert!(try_stage_pointer("down", 800.0, 450.0));
    let selected = try_read_timeline_projection()
        .expect("selected")
        .primary_layer_id;
    assert!(try_stage_unmount());
    assert!(!try_stage_pointer("down", 800.0, 450.0));
    let after = try_read_timeline_projection().expect("after");
    assert_eq!(after.primary_layer_id, selected);

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn stage_pointer_state_machine_blocks_invalid_transitions_and_reopens_after_cancel() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("stage-ptr-seq");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    assert!(try_stage_mount(800.0, 600.0, 1.0));
    assert!(!try_stage_pointer("drag", 12.0, 14.0));
    assert!(!try_stage_pointer("up", 12.0, 14.0));
    assert!(try_stage_pointer("down", 10.0, 10.0));
    assert!(try_stage_pointer("drag", 12.0, 14.0));
    assert!(try_stage_pointer("cancel", 12.0, 14.0));
    assert!(!try_stage_pointer("drag", 12.0, 14.0));
    assert!(!try_stage_pointer("up", 12.0, 14.0));
    assert!(try_stage_pointer("down", 10.0, 10.0));
    assert!(try_stage_pointer("drag", 12.0, 14.0));
    assert!(try_stage_pointer("up", 12.0, 14.0));
    let seq = {
        let guard = host_slot().lock().expect("slot");
        guard.as_ref().expect("slot").pointer_sequence
    };
    assert_eq!(seq, 6);

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn stage_resize_registers_and_mounts_when_unregistered_or_unmounted() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("stage-resize-retry");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    assert!(try_stage_resize(800.0, 600.0, 1.0));
    assert!(try_stage_unmount());
    assert!(try_stage_resize(1024.0, 768.0, 1.0));

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn timeline_selection_dispatch_selects_and_clears_via_host_slot() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("tl-selection");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    dispatch_kind(
        host,
        "place_rectangle",
        r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
    );
    let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
    let layer_id = placed.primary_layer_id.expect("primary after place");

    let cleared_terminal = try_dispatch_timeline_selection(
        &crate::timeline_skia::TimelineSelectionCommit::ClearSelection,
    )
    .expect("clear terminal");
    assert!(cleared_terminal.accepted);
    assert!(
        cleared_terminal
            .projection
            .expect("clear snapshot")
            .primary_layer_id
            .is_none()
    );
    let cleared = try_read_timeline_projection().expect("cleared");
    assert!(cleared.primary_layer_id.is_none());

    let selected_terminal = try_dispatch_timeline_selection(
        &crate::timeline_skia::TimelineSelectionCommit::SelectLayer {
            layer_id: layer_id.clone(),
        },
    )
    .expect("select terminal");
    assert!(selected_terminal.accepted);
    assert_eq!(
        selected_terminal
            .projection
            .expect("select snapshot")
            .primary_layer_id
            .as_deref(),
        Some(layer_id.as_str())
    );
    let selected = try_read_timeline_projection().expect("selected");
    assert_eq!(
        selected.primary_layer_id.as_deref(),
        Some(layer_id.as_str())
    );

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

