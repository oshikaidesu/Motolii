use super::{clear_slot, dispatch_kind, install_slot, temp_project, test_lock};
use super::super::keymap::{resolve_mac_key_action, MOD_META, MOD_SHIFT};
use super::super::{
    test_reset_snapshot_read_count, test_snapshot_read_count, try_read_projection_stamp,
    try_read_timeline_projection,
};
use crate::renderer_core::RendererCore;

/// F9: stamp不変tickではsnapshot読みFFI(=try_read_timeline_projection)を呼ばない。
#[test]
fn unchanged_projection_stamp_skips_snapshot_json_read() {
    let _lock = test_lock();
    let path = temp_project("stamp-skip-read");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    test_reset_snapshot_read_count();

    let stamp = try_read_projection_stamp().expect("stamp");
    assert_eq!(stamp, (0, 0));
    let before = test_snapshot_read_count();

    // stamp一致ならgateはfull読み不要 → カウンタ不変
    assert!(!RendererCore::host_snapshot_read_needed(
        Some(stamp),
        Some(stamp),
        false
    ));
    assert_eq!(test_snapshot_read_count(), before);

    // 変化時だけ読む
    dispatch_kind(host, "set_time", r#","frame":12"#);
    let next = try_read_projection_stamp().expect("stamp after set_time");
    assert_ne!(next, stamp);
    assert!(RendererCore::host_snapshot_read_needed(
        Some(stamp),
        Some(next),
        false
    ));
    let _ = try_read_timeline_projection().expect("projection");
    assert_eq!(test_snapshot_read_count(), before + 1);

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn mac_key_table_maps_space_delete_undo_to_existing_kinds() {
    let space = resolve_mac_key_action(49, 0, " ");
    assert_eq!(
        space
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("toggle_playback")
    );
    let delete = resolve_mac_key_action(117, 0, "");
    assert_eq!(
        delete
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("delete_layer")
    );
    let backspace = resolve_mac_key_action(51, 0, "");
    assert_eq!(
        backspace
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("delete_layer")
    );
    let undo = resolve_mac_key_action(6, MOD_META, "z");
    assert_eq!(
        undo.as_ref().and_then(motolii_ui::product_action_host_kind),
        Some("undo")
    );
    let redo = resolve_mac_key_action(6, MOD_META | MOD_SHIFT, "z");
    assert_eq!(
        redo.as_ref().and_then(motolii_ui::product_action_host_kind),
        Some("redo")
    );
    let duplicate = resolve_mac_key_action(2, MOD_META, "d");
    assert_eq!(
        duplicate
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("duplicate")
    );
    let shuttle_forward = resolve_mac_key_action(37, 0, "l");
    assert_eq!(
        shuttle_forward
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("shuttle_forward")
    );
    let shuttle_reverse = resolve_mac_key_action(38, 0, "j");
    assert_eq!(
        shuttle_reverse
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("shuttle_reverse")
    );
    let shuttle_stop = resolve_mac_key_action(40, 0, "k");
    assert_eq!(
        shuttle_stop
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("shuttle_stop")
    );
    let split = resolve_mac_key_action(40, MOD_META, "k");
    assert_eq!(
        split
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("split")
    );
    let mark_in = resolve_mac_key_action(34, 0, "i");
    assert_eq!(
        mark_in
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("trim_clip_in")
    );
    let mark_out = resolve_mac_key_action(31, 0, "o");
    assert_eq!(
        mark_out
            .as_ref()
            .and_then(motolii_ui::product_action_host_kind),
        Some("trim_clip_out")
    );
    let mute = resolve_mac_key_action(46, 0, "m");
    assert_eq!(
        mute.as_ref().and_then(motolii_ui::product_action_host_kind),
        Some(motolii_ui::PRODUCT_HOST_KIND_MUTE)
    );
    let solo = resolve_mac_key_action(1, 0, "s");
    assert_eq!(
        solo.as_ref().and_then(motolii_ui::product_action_host_kind),
        Some(motolii_ui::PRODUCT_HOST_KIND_SOLO)
    );
}
