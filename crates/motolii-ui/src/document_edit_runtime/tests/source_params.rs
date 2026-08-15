use super::super::*;
use super::fixtures::*;

#[test]
fn set_effect_param_commits_once_replay_is_noop_and_undo_restores_live_value() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let mut queue = DocumentEditQueue::default();
    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.opacity".into(),
    });
    let attached = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("opacity attach must publish");
    let effect_use_id = attached.created_effect_use.expect("effect use");
    let definition_id = attached
        .snapshot
        .find_effect_use(primary, effect_use_id)
        .expect("attached effect")
        .definition_id;
    let request = SetEffectParamRequest::new(
        primary,
        effect_use_id,
        definition_id,
        "core.filter.opacity".into(),
        1,
        "amount".into(),
        0.4,
    );
    queue.push_set_effect_param(request.clone());

    let changed = runtime
        .process_next(&mut queue, Some(primary), 1)
        .unwrap()
        .expect("changed release must publish once");
    assert_eq!(changed.kind, DocumentEditActionKind::SetEffectParam);
    assert_eq!(changed.revision, 2);
    assert_eq!(changed.projection_generation, 2);
    assert_eq!(runtime.history_lengths(), (2, 0));
    assert_eq!(
        changed
            .snapshot
            .effect_definition(definition_id)
            .unwrap()
            .params
            .get("amount"),
        Some(&motolii_doc::DocParam::const_f64(0.4))
    );

    let changed_json = serde_json::to_vec(&*changed.snapshot).unwrap();
    let journal_size = fs::metadata(&journal).unwrap().len();
    queue.push_set_effect_param(request);
    assert!(matches!(
        runtime.process_next(&mut queue, Some(primary), u64::MAX),
        Err(DocumentEditRuntimeError::PrepareRejected)
    ));
    assert_eq!(runtime.revision(), 2);
    assert_eq!(runtime.history_lengths(), (2, 0));
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        changed_json
    );
    assert_eq!(fs::metadata(&journal).unwrap().len(), journal_size);

    queue.push_undo();
    let undone = runtime
        .process_next(&mut queue, Some(primary), 2)
        .unwrap()
        .expect("one Undo must restore the live old value");
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    assert_eq!(runtime.history_lengths(), (1, 1));
    assert_eq!(
        undone
            .snapshot
            .effect_definition(definition_id)
            .unwrap()
            .params
            .get("amount"),
        Some(&motolii_doc::DocParam::const_f64(1.0))
    );
}

fn clip_source_param(document: &Document, layer: LayerId, param: &str) -> DocParam {
    for track in &document.tracks {
        for item in &track.items {
            if let TrackItem::Clip(clip) = item {
                if clip.envelope.layer_id == layer {
                    let ClipSource::Plugin { params, .. } = &clip.source else {
                        panic!("expected plugin source");
                    };
                    return params
                        .get(param)
                        .cloned()
                        .unwrap_or_else(|| panic!("missing param {param}"));
                }
            }
        }
    }
    panic!("layer not found");
}

#[test]
fn set_source_param_writes_f64_and_color_and_undo_restores() {
    let (document, _) = fixture();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_vism(PlaceVismRequest {
        plugin_id: "core.layer_source.radial_repeater".into(),
        position: [0.0, 0.0],
        playhead: RationalTime::ZERO,
    });
    let placed = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("place vism");
    let layer = placed.primary.expect("placed layer");
    assert_eq!(
        clip_source_param(&placed.snapshot, layer, "count"),
        DocParam::const_f64(12.0)
    );
    assert_eq!(
        clip_source_param(&placed.snapshot, layer, "color"),
        DocParam::const_color([1.0, 1.0, 1.0, 1.0])
    );

    queue.push_set_source_param(SetSourceParamRequest::new(
        layer,
        "count".into(),
        DocParam::const_f64(8.0),
    ));
    let counted = runtime
        .process_next(&mut queue, Some(layer), 1)
        .unwrap()
        .expect("count write");
    assert_eq!(counted.kind, DocumentEditActionKind::SetSourceParam);
    assert_eq!(
        clip_source_param(&counted.snapshot, layer, "count"),
        DocParam::const_f64(8.0)
    );
    assert_eq!(
        clip_source_param(&counted.snapshot, layer, "color"),
        DocParam::const_color([1.0, 1.0, 1.0, 1.0])
    );

    queue.push_set_source_param(SetSourceParamRequest::new(
        layer,
        "color".into(),
        DocParam::const_color([0.2, 0.4, 0.6, 1.0]),
    ));
    let colored = runtime
        .process_next(&mut queue, Some(layer), 2)
        .unwrap()
        .expect("color write");
    assert_eq!(
        clip_source_param(&colored.snapshot, layer, "color"),
        DocParam::const_color([0.2, 0.4, 0.6, 1.0])
    );
    assert_eq!(
        clip_source_param(&colored.snapshot, layer, "count"),
        DocParam::const_f64(8.0)
    );

    queue.push_undo();
    let undone_color = runtime
        .process_next(&mut queue, Some(layer), 3)
        .unwrap()
        .expect("undo color");
    assert_eq!(
        clip_source_param(&undone_color.snapshot, layer, "color"),
        DocParam::const_color([1.0, 1.0, 1.0, 1.0])
    );
    queue.push_undo();
    let undone_count = runtime
        .process_next(&mut queue, Some(layer), 4)
        .unwrap()
        .expect("undo count");
    assert_eq!(
        clip_source_param(&undone_count.snapshot, layer, "count"),
        DocParam::const_f64(12.0)
    );
}

#[test]
fn set_source_param_invalid_requests_write_nothing() {
    let (document, _) = fixture();
    let asset_layer = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_place_vism(PlaceVismRequest {
        plugin_id: "core.layer_source.radial_repeater".into(),
        position: [0.0, 0.0],
        playhead: RationalTime::ZERO,
    });
    let placed = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("place vism");
    let layer = placed.primary.expect("placed layer");
    let initial = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let history = runtime.history_lengths();
    let revision = runtime.revision();

    for request in [
        SetSourceParamRequest::new(asset_layer, "count".into(), DocParam::const_f64(8.0)),
        SetSourceParamRequest::new(layer, "missing".into(), DocParam::const_f64(8.0)),
        SetSourceParamRequest::new(layer, "count".into(), DocParam::const_f64(12.0)),
        SetSourceParamRequest::new(layer, "count".into(), DocParam::const_f64(f64::NAN)),
        SetSourceParamRequest::new(
            layer,
            "color".into(),
            DocParam::const_color([f64::INFINITY, 0.0, 0.0, 1.0]),
        ),
    ] {
        queue.push_set_source_param(request);
        assert!(matches!(
            runtime.process_next(&mut queue, Some(layer), revision),
            Err(DocumentEditRuntimeError::PrepareRejected)
        ));
    }
    assert_eq!(runtime.history_lengths(), history);
    assert_eq!(runtime.revision(), revision);
    assert_eq!(serde_json::to_vec(&*runtime.snapshot()).unwrap(), initial);
}

#[test]
fn no_primary_attach_effect_set_opacity_set_source_param_are_typed_errors() {
    let (document, _) = fixture();
    let layer = fixture_layer(&document);
    let initial_json = serde_json::to_vec(&document).unwrap();
    let initial_peek = document.layers.peek_next();
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();

    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.opacity".into(),
    });
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::NoPrimarySelection)
    ));

    queue.push_set_opacity(SetOpacityRequest {
        target: layer,
        value: 0.25,
    });
    assert!(matches!(
        runtime.process_next(&mut queue, None, 0),
        Err(DocumentEditRuntimeError::NoPrimarySelection)
    ));
    assert_eq!(runtime.snapshot().layers.peek_next(), initial_peek);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        initial_json
    );
    assert_eq!(runtime.revision(), 0);
    assert_eq!(runtime.history_lengths(), (0, 0));

    queue.push_place_vism(PlaceVismRequest {
        plugin_id: "core.layer_source.radial_repeater".into(),
        position: [0.0, 0.0],
        playhead: RationalTime::ZERO,
    });
    let placed = runtime
        .process_next(&mut queue, None, 0)
        .unwrap()
        .expect("place vism");
    let vism = placed.primary.expect("placed layer");
    let after_place = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let after_peek = runtime.snapshot().layers.peek_next();
    queue.push_set_source_param(SetSourceParamRequest::new(
        vism,
        "count".into(),
        DocParam::const_f64(8.0),
    ));
    assert!(matches!(
        runtime.process_next(&mut queue, None, placed.projection_generation),
        Err(DocumentEditRuntimeError::NoPrimarySelection)
    ));
    assert_eq!(runtime.snapshot().layers.peek_next(), after_peek);
    assert_eq!(
        serde_json::to_vec(&*runtime.snapshot()).unwrap(),
        after_place
    );
}
