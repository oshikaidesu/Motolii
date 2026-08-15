//! place_vism と source catalog の試験。helper は tests。
use super::tests::*;
use super::*;

pub(super) fn live_document(host: u64) -> Arc<Document> {
    with_registry(|registry| Ok(registry.hosts.get(&host).expect("host").runtime.snapshot()))
        .expect("document")
}

pub(super) fn document_clip(document: &Document, layer_id: LayerId) -> Option<&Clip> {
    document.tracks.iter().find_map(|track| {
        track.items.iter().find_map(|item| match item {
            TrackItem::Clip(clip) if clip.envelope.layer_id == layer_id => Some(clip),
            _ => None,
        })
    })
}

pub(super) fn create_empty_track_host(tag: &str) -> u64 {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("seed-track").expect("track");
    document.tracks.push(Track {
        id: track,
        items: vec![],
    });
    document.validate().expect("valid empty track document");
    let limits = ResourceLimits::production();
    {
        let mut session = ProjectSession::acquire(&path, &limits).expect("acquire");
        session
            .save_with_journal(
                &document,
                &SaveProjectOptions {
                    limits,
                    checkpoint: true,
                    ..SaveProjectOptions::default()
                },
            )
            .expect("save");
    }
    host_create_for_test(&path).expect("host")
}

pub(super) fn place_vism_json(
    host: u64,
    plugin_id: &str,
    position: [f64; 2],
    playhead: &str,
) -> String {
    format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"place_vism","#,
            r#""host_handle":"{host}","plugin_id":"{plugin}","position":[{x},{y}],"playhead":{playhead}}}"#
        ),
        host = host,
        plugin = plugin_id,
        x = position[0],
        y = position[1],
        playhead = playhead,
    )
}

pub(super) fn place_media_json(
    host: u64,
    item_id: &str,
    position: [f64; 2],
    playhead: &str,
) -> String {
    format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"place_media","#,
            r#""host_handle":"{host}","item_id":"{item}","position":[{x},{y}],"playhead":{playhead}}}"#
        ),
        host = host,
        item = item_id,
        x = position[0],
        y = position[1],
        playhead = playhead,
    )
}

#[test]
fn snapshot_catalog_sources_lists_radial_repeater_and_all_place_vism() {
    let _lock = test_lock();
    let host = create_empty_track_host("catalog-sources-place-loop");
    let wire = read_wire(host);
    assert!(wire
        .catalog
        .sources
        .iter()
        .any(|source| source.plugin_id == "core.layer_source.radial_repeater"));
    assert!(!wire
        .catalog
        .sources
        .iter()
        .any(|source| source.plugin_id == "core.layer_source.clear"));
    assert!(wire.timeline.layers.is_empty());

    for source in wire.catalog.sources.clone() {
        let response = dispatch_raw_json(
            host,
            &place_vism_json(host, &source.plugin_id, [0.0, 0.0], r#"{"num":0,"den":1}"#),
        );
        assert!(
            response.accepted,
            "place_vism failed for {}: accepted={} reason={:?}",
            source.plugin_id, response.accepted, response.reason,
        );
    }
    let after_place = read_wire(host);
    assert_eq!(
        after_place.timeline.layers.len(),
        wire.catalog.sources.len()
    );
    let placed_ids: Vec<String> = after_place
        .timeline
        .layers
        .iter()
        .map(|layer| layer.layer_id.clone())
        .collect();
    assert!(
        after_place
            .primary_layer_id
            .as_ref()
            .is_some_and(|id| placed_ids.contains(id)),
        "place_vism should select a live layer"
    );

    for _ in 0..wire.catalog.sources.len() {
        let before_undo = read_wire(host);
        let removed = before_undo.primary_layer_id.clone();
        let undone = dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#,
                host = host,
            ),
        );
        assert!(undone.accepted);
        let after_undo = read_wire(host);
        if let Some(removed_id) = removed {
            assert_ne!(
                after_undo.primary_layer_id.as_deref(),
                Some(removed_id.as_str()),
                "undo must not keep deleted LayerId as primary"
            );
            assert!(
                after_undo
                    .primary_layer_id
                    .as_ref()
                    .map(|id| after_undo
                        .timeline
                        .layers
                        .iter()
                        .any(|layer| layer.layer_id == *id))
                    .unwrap_or(true),
                "primary_layer_id must be absent or live"
            );
        }
    }
    assert!(read_wire(host).timeline.layers.is_empty());
    assert!(read_wire(host).primary_layer_id.is_none());
    let _ = host_destroy_for_test(host);
}

#[test]
fn place_vism_projects_source_params_defaults_on_timeline() {
    let _lock = test_lock();
    let host = create_empty_track_host("place-vism-source-params");
    assert!(
        dispatch_raw_json(
            host,
            &place_vism_json(
                host,
                "core.layer_source.radial_repeater",
                [0.0, 0.0],
                r#"{"num":0,"den":1}"#
            ),
        )
        .accepted
    );
    let wire = read_wire(host);
    assert_eq!(wire.timeline.layers.len(), 1);
    let layer = &wire.timeline.layers[0];
    assert_eq!(layer.display_name, "Radial Repeater");
    assert!(!layer.source_params_truncated);
    let by_id: BTreeMap<_, _> = layer
        .source_params
        .iter()
        .map(|param| (param.param_id.as_str(), param))
        .collect();
    assert_eq!(by_id.get("count").map(|param| param.value), Some(12.0));
    assert_eq!(by_id.get("radius").map(|param| param.value), Some(0.30));
    assert_eq!(by_id.get("dot_radius").map(|param| param.value), Some(0.04));
    assert_eq!(by_id.get("phase").map(|param| param.value), Some(0.0));
    assert_eq!(
        by_id.get("angular_speed").map(|param| param.value),
        Some(0.0)
    );
    let color = by_id.get("color").expect("color is a product source param");
    assert_eq!(color.value, 0.0);
    assert_eq!(color.color, Some([1.0, 1.0, 1.0, 1.0]));
    let _ = host_destroy_for_test(host);
}
