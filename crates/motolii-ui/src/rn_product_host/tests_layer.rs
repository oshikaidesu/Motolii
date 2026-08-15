//! delete / duplicate / mute / solo の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn remove_position_key_clears_timeline_projection_and_undo_restores() {
    let _lock = test_lock();
    let host = create_host("remove-position-key");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let target = LayerId::from_raw(layer_id.parse::<u64>().expect("layer id"));
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(target);
        let published = product
            .runtime
            .process_next(&mut queue, product.primary, product.projection_generation)
            .expect("process")
            .expect("published");
        product.primary = published.primary;
        product.projection_generation = published.projection_generation;
        Ok(())
    })
    .expect("seed primary");

    let add = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"add_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","time":{{"num":1,"den":1}}}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(add.accepted);
    let key_id = add
        .snapshot
        .expect("keyed")
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .position_keys[0]
        .key_id
        .clone();

    let removed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"remove_position_key","#,
                r#""host_handle":"{host}","target":"{layer}","key_id":"{key}"}}"#
            ),
            host = host,
            layer = layer_id,
            key = key_id,
        ),
    );
    assert!(removed.accepted);
    let after_remove = removed.snapshot.expect("removed");
    let after_layer = after_remove
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert!(after_layer.position_keys.is_empty());

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
        ),
    );
    assert!(undone.accepted);
    let restored = undone
        .snapshot
        .expect("restored")
        .timeline
        .layers
        .into_iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert_eq!(restored.position_keys.len(), 1);
    assert_eq!(restored.position_keys[0].key_id, key_id);
    let _ = host_destroy_for_test(host);
}

#[test]
fn delete_layer_removes_timeline_row_and_undo_restores_id_and_name() {
    let _lock = test_lock();
    let host = create_host("delete-layer");
    let before = read_snapshot(host);
    let layer_id = before.layer_ids[0].clone();
    let display_name = before
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .display_name
        .clone();

    let deleted = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"delete_layer","#,
                r#""host_handle":"{host}","target":"{layer}"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(deleted.accepted);
    let after = deleted.snapshot.expect("deleted");
    assert!(!after
        .timeline
        .layers
        .iter()
        .any(|layer| layer.layer_id == layer_id));
    assert!(!after.layer_ids.iter().any(|id| id == &layer_id));

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
        ),
    );
    assert!(undone.accepted);
    let restored = undone.snapshot.expect("restored");
    let layer = restored
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("restored layer");
    assert_eq!(layer.display_name, display_name);
    assert!(restored.layer_ids.iter().any(|id| id == &layer_id));
    let _ = host_destroy_for_test(host);
}

#[test]
fn duplicate_layer_adds_timeline_row_and_undo_restores_count() {
    let _lock = test_lock();
    let host = create_host("duplicate-layer");
    let before = read_snapshot(host);
    let layer_id = before.layer_ids[0].clone();
    let before_count = before.layer_ids.len();
    assert_eq!(before.timeline.layers.len(), before_count);

    let duplicated = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"duplicate","#,
                r#""host_handle":"{host}","target":"{layer}"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(duplicated.accepted);
    let after = duplicated.snapshot.expect("duplicated");
    assert_eq!(after.layer_ids.len(), before_count + 1);
    assert_eq!(after.timeline.layers.len(), before_count + 1);
    assert!(after.layer_ids.iter().any(|id| id == &layer_id));
    assert_eq!(
        after.layer_ids.iter().filter(|id| *id != &layer_id).count(),
        1
    );

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
        ),
    );
    assert!(undone.accepted);
    let restored = undone.snapshot.expect("restored");
    assert_eq!(restored.layer_ids, before.layer_ids);
    assert_eq!(restored.timeline.layers.len(), before_count);
    let _ = host_destroy_for_test(host);
}

#[test]
fn duplicate_without_target_rejects_without_document_mutation() {
    let _lock = test_lock();
    let host = create_host("duplicate-missing-target");
    let baseline = read_snapshot(host);
    let rejected = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"duplicate","host_handle":"{host}"}}"#
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
    let after = read_snapshot(host);
    assert_eq!(after.revision, baseline.revision);
    assert_eq!(after.layer_ids, baseline.layer_ids);
    let _ = host_destroy_for_test(host);
}

#[test]
fn mute_and_solo_toggle_item_envelope_flags() {
    let _lock = test_lock();
    let host = create_host("mute-solo-envelope");
    let baseline = read_snapshot(host);
    let layer_id = baseline.layer_ids[0].clone();
    let seed = read_wire(host);
    let layer = seed
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert!(layer.visible);
    assert!(!layer.solo);

    let muted = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"mute","#,
                r#""host_handle":"{host}","target":"{layer}"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(muted.accepted);
    let after_mute = read_wire(host);
    let muted_layer = after_mute
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert!(!muted_layer.visible);
    assert!(!muted_layer.solo);

    let soloed = dispatch_raw_json(
        host,
        &format!(
            concat!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"solo","#,
                r#""host_handle":"{host}","target":"{layer}"}}"#
            ),
            host = host,
            layer = layer_id,
        ),
    );
    assert!(soloed.accepted);
    let after_solo = read_wire(host);
    let solo_layer = after_solo
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert!(!solo_layer.visible);
    assert!(solo_layer.solo);

    let undone = dispatch_raw_json(
        host,
        &format!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
        ),
    );
    assert!(undone.accepted);
    let after_undo = read_wire(host);
    let undone_layer = after_undo
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert!(!undone_layer.visible);
    assert!(!undone_layer.solo);
    let _ = host_destroy_for_test(host);
}

#[test]
fn mute_and_solo_without_target_reject_without_document_mutation() {
    let _lock = test_lock();
    let host = create_host("mute-solo-missing-target");
    let baseline = read_snapshot(host);
    for kind in ["mute", "solo"] {
        let rejected = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{host}"}}"#
            ),
        );
        assert!(!rejected.accepted, "{kind} must reject missing target");
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        let after = read_snapshot(host);
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.layer_ids, baseline.layer_ids);
    }
    let _ = host_destroy_for_test(host);
}

#[test]
fn missing_target_selection_and_delete_intents_reject_without_document_mutation() {
    let _lock = test_lock();
    let host = create_host("missing-target-reject");
    let baseline = read_snapshot(host);
    let missing = "999999";

    for kind in ["select_layer", "delete_layer"] {
        let rejected = dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","#,
                    r#""host_handle":"{host}","target":"{missing}"}}"#
                ),
                kind = kind,
                host = host,
                missing = missing,
            ),
        );
        assert!(!rejected.accepted, "{kind} must reject missing target");
        assert_eq!(rejected.reason, Some(RnHostReasonCode::InvalidIntent));
        let after = read_snapshot(host);
        assert_eq!(after.revision, baseline.revision);
        assert_eq!(after.projection_generation, baseline.projection_generation);
        assert_eq!(after.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(after.layer_ids, baseline.layer_ids);
    }
    let _ = host_destroy_for_test(host);
}
