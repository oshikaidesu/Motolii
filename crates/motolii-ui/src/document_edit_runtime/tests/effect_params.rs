use super::super::*;
use super::fixtures::*;

#[test]
fn set_effect_param_stale_or_invalid_requests_write_nothing() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let (path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.opacity".into(),
    });
    let attached = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .unwrap();
    let effect_use_id = attached.created_effect_use.unwrap();
    let definition_id = attached
        .snapshot
        .find_effect_use(primary, effect_use_id)
        .unwrap()
        .definition_id;
    let initial_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    let initial_history = runtime.history_lengths();
    let initial_revision = runtime.revision();
    let journal = journal_path_for_document(&path);
    let initial_journal_size = fs::metadata(&journal).unwrap().len();

    for request in [
        SetEffectParamRequest::new(
            LayerId::from_raw(u64::MAX),
            effect_use_id,
            definition_id,
            "core.filter.opacity".into(),
            1,
            "amount".into(),
            0.3,
        ),
        SetEffectParamRequest::new(
            primary,
            effect_use_id,
            definition_id,
            "core.filter.opacity".into(),
            2,
            "amount".into(),
            0.3,
        ),
        SetEffectParamRequest::new(
            primary,
            effect_use_id,
            definition_id,
            "core.filter.opacity".into(),
            1,
            "amount".into(),
            f64::NAN,
        ),
    ] {
        queue.push_set_effect_param(request);
        assert!(matches!(
            runtime.process_next(&mut queue, Some(primary), u64::MAX),
            Err(DocumentEditRuntimeError::PrepareRejected)
        ));
        assert_eq!(runtime.revision(), initial_revision);
        assert_eq!(runtime.history_lengths(), initial_history);
        assert_eq!(
            serde_json::to_vec(&*runtime.snapshot()).unwrap(),
            initial_json
        );
        assert_eq!(fs::metadata(&journal).unwrap().len(), initial_journal_size);
    }
}

#[test]
fn set_effect_param_writes_amount_on_two_param_definition() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.opacity".into(),
    });
    let attached = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("opacity attach");
    let effect_use_id = attached.created_effect_use.expect("effect use");
    let definition_id = attached
        .snapshot
        .find_effect_use(primary, effect_use_id)
        .expect("attached effect")
        .definition_id;
    runtime.writer.edit(|document| {
        document
            .effect_definition_mut(definition_id)
            .expect("definition")
            .params
            .insert("mix".into(), DocParam::const_f64(0.5));
    });
    let request = SetEffectParamRequest::new(
        primary,
        effect_use_id,
        definition_id,
        "core.filter.opacity".into(),
        1,
        "amount".into(),
        0.4,
    );
    let command = prepare_set_effect_param_command(runtime.writer.snapshot().as_ref(), &request)
        .expect("2-param amount must prepare");
    match command {
        Command::SetProperty {
            property: ScalarPropertyId::EffectParam(use_id, param_id),
            new_value,
            ..
        } => {
            assert_eq!(use_id, effect_use_id);
            assert_eq!(param_id, "amount");
            assert_eq!(new_value, DocParam::const_f64(0.4));
        }
        other => panic!("expected EffectParam SetProperty, got {other:?}"),
    }
}

#[test]
fn set_effect_param_writes_color_const() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.tint".into(),
    });
    let attached = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("tint attach");
    let effect_use_id = attached.created_effect_use.expect("effect use");
    let definition_id = attached
        .snapshot
        .find_effect_use(primary, effect_use_id)
        .expect("attached effect")
        .definition_id;
    let f64_on_color = SetEffectParamRequest::new(
        primary,
        effect_use_id,
        definition_id,
        "core.filter.tint".into(),
        1,
        "color".into(),
        0.2,
    );
    assert!(
        prepare_set_effect_param_command(runtime.writer.snapshot().as_ref(), &f64_on_color)
            .is_none()
    );
    queue.push_set_effect_param(SetEffectParamRequest::with_param(
        primary,
        effect_use_id,
        definition_id,
        "core.filter.tint".into(),
        1,
        "color".into(),
        DocParam::const_color([0.2, 1.0, 1.0, 1.0]),
    ));
    let changed = runtime
        .process_next(&mut queue, Some(primary), attached.projection_generation)
        .unwrap()
        .expect("color effect param must write");
    assert_eq!(
        changed
            .snapshot
            .effect_definition(definition_id)
            .unwrap()
            .params
            .get("color"),
        Some(&DocParam::const_color([0.2, 1.0, 1.0, 1.0]))
    );
}

#[test]
fn set_effect_param_unknown_param_id_is_prepare_rejected() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let (_path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.opacity".into(),
    });
    let attached = runtime
        .process_next(&mut queue, Some(primary), 0)
        .unwrap()
        .expect("opacity attach");
    let effect_use_id = attached.created_effect_use.expect("effect use");
    let definition_id = attached
        .snapshot
        .find_effect_use(primary, effect_use_id)
        .expect("attached effect")
        .definition_id;
    let before = serde_json::to_vec(&*runtime.snapshot()).unwrap();
    queue.push_set_effect_param(SetEffectParamRequest::new(
        primary,
        effect_use_id,
        definition_id,
        "core.filter.opacity".into(),
        1,
        "missing".into(),
        0.4,
    ));
    assert!(matches!(
        runtime.process_next(&mut queue, Some(primary), attached.projection_generation),
        Err(DocumentEditRuntimeError::PrepareRejected)
    ));
    assert_eq!(serde_json::to_vec(&*runtime.snapshot()).unwrap(), before);
}

#[test]
fn attach_effect_preflight_rejections_write_nothing() {
    fn assert_unchanged(
        runtime: &DocumentEditRuntime,
        queue: &DocumentEditQueue,
        initial_json: &[u8],
        initial_stable: u64,
        journal: &std::path::Path,
        journal_size: u64,
    ) {
        assert_preflight_rejection_invariants(runtime, queue, initial_json, 0, (0, 0));
        assert_eq!(
            runtime.snapshot().next_stable_id.peek_next(),
            initial_stable
        );
        assert_eq!(
            fs::metadata(journal).map(|meta| meta.len()).unwrap_or(0),
            journal_size
        );
    }

    for primary in [None, Some(LayerId::from_raw(u64::MAX))] {
        let (document, _) = fixture();
        let initial_json = serde_json::to_vec(&document).unwrap();
        let initial_stable = document.next_stable_id.peek_next();
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
        let mut queue = DocumentEditQueue::default();
        queue.push_attach_effect(AttachEffectRequest {
            plugin_id: "core.filter.opacity".into(),
        });

        let error = runtime
            .process_next(&mut queue, primary, u64::MAX)
            .expect_err("attach without a live primary must not silent-accept");
        match primary {
            None => assert!(matches!(
                error,
                DocumentEditRuntimeError::NoPrimarySelection
            )),
            Some(target) => assert!(matches!(
                error,
                DocumentEditRuntimeError::SelectionTargetNotFound(id) if id == target
            )),
        }
        assert_unchanged(
            &runtime,
            &queue,
            &initial_json,
            initial_stable,
            &journal,
            journal_size,
        );
    }

    for (plugin_id, expected) in [
        ("missing.filter", "missing"),
        ("core.layer_source.radial_repeater", "kind"),
    ] {
        let (document, _) = fixture();
        let primary = fixture_layer(&document);
        let initial_json = serde_json::to_vec(&document).unwrap();
        let initial_stable = document.next_stable_id.peek_next();
        let (path, mut runtime) = open_runtime(document);
        let journal = journal_path_for_document(&path);
        let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
        let mut queue = DocumentEditQueue::default();
        queue.push_attach_effect(AttachEffectRequest {
            plugin_id: plugin_id.into(),
        });

        let error = runtime
            .process_next(&mut queue, Some(primary), 0)
            .expect_err("invalid contract must fail before commit");
        match expected {
            "missing" => assert!(matches!(
                error,
                DocumentEditRuntimeError::DocumentPlugin(
                    DocumentPluginError::ContractMissing { .. }
                )
            )),
            "kind" => assert!(matches!(
                error,
                DocumentEditRuntimeError::DocumentPlugin(DocumentPluginError::KindMismatch { .. })
            )),
            _ => unreachable!(),
        }
        assert_unchanged(
            &runtime,
            &queue,
            &initial_json,
            initial_stable,
            &journal,
            journal_size,
        );
    }

    let (document, _) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let initial_stable = document.next_stable_id.peek_next();
    let (path, runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let journal_size = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
    let non_const = PreparedPluginRecipe {
        plugin_id: "core.filter.opacity".into(),
        saved_version: 1,
        current_version: 1,
        params: BTreeMap::from([(
            "amount".into(),
            motolii_doc::DocParam::Keyframes(motolii_doc::DocKeyframeTrack::new()),
        )]),
    };
    assert!(matches!(
        attach_effect_draft(non_const),
        Err(DocumentEditRuntimeError::AttachDefaultNotConst { ref param })
            if param == "amount"
    ));
    assert_unchanged(
        &runtime,
        &DocumentEditQueue::default(),
        &initial_json,
        initial_stable,
        &journal,
        journal_size,
    );
}
