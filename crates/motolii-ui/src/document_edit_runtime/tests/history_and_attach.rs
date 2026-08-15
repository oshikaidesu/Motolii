use super::super::*;
use super::fixtures::*;

#[test]
fn undo_and_redo_commit_durably_and_publish_once_each() {
    let (document, request) = fixture();
    let initial_json = serde_json::to_vec(&document).unwrap();
    let (path, mut runtime) = open_runtime(document);
    let mut queue = DocumentEditQueue::default();
    queue.push_prepared(delete_output(), Some(request)).unwrap();
    let applied = runtime.process_next(&mut queue, None, 0).unwrap().unwrap();
    let post_apply_json = serde_json::to_vec(&*runtime.snapshot()).unwrap();

    queue.push_undo();
    let undone = runtime.process_next(&mut queue, None, 1).unwrap().unwrap();
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    assert_eq!(undone.revision, 2);
    assert_eq!(undone.projection_generation, 2);
    assert_eq!(serde_json::to_vec(&*undone.snapshot).unwrap(), initial_json);
    assert_eq!(runtime.history_lengths(), (0, 1));
    assert_eq!(queue.len(), 0);

    queue.push_redo();
    let redone = runtime.process_next(&mut queue, None, 2).unwrap().unwrap();
    assert_eq!(redone.kind, DocumentEditActionKind::Redo);
    assert_eq!(redone.revision, 3);
    assert_eq!(redone.projection_generation, 3);
    assert_eq!(
        serde_json::to_vec(&*redone.snapshot).unwrap(),
        post_apply_json
    );
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert_eq!(applied.revision, 1);

    drop(runtime);
    let limits = ResourceLimits::production();
    let (_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(
        serde_json::to_vec(&reopened.document).unwrap(),
        post_apply_json
    );
}

#[test]
fn attach_effect_commits_one_catalog_recipe_and_roundtrips_history() {
    let (document, _) = fixture();
    let primary = fixture_layer(&document);
    let initial_stable = document.next_stable_id.peek_next();
    let (path, mut runtime) = open_runtime(document);
    let journal = journal_path_for_document(&path);
    let journal_before = fs::metadata(&journal).map(|meta| meta.len()).unwrap_or(0);
    let mut queue = DocumentEditQueue::default();
    queue.push_attach_effect(AttachEffectRequest {
        plugin_id: "core.filter.opacity".into(),
    });

    let attached = runtime
        .process_next(&mut queue, Some(primary), 4)
        .unwrap()
        .expect("validated attach must publish once");
    let created = attached
        .created_effect_use
        .expect("attach publish must identify the created Effect Use");
    assert_eq!(attached.kind, DocumentEditActionKind::AttachEffect);
    assert_eq!(attached.revision, 1);
    assert_eq!(attached.primary, Some(primary));
    assert_eq!(attached.projection_generation, 5);
    assert_eq!(queue.len(), 0);
    assert_eq!(runtime.history_lengths(), (1, 0));
    assert!(fs::metadata(&journal).expect("journal").len() > journal_before);

    let envelope = runtime
        .writer
        .find_envelope(primary)
        .expect("primary envelope");
    assert_eq!(envelope.effects.len(), 1);
    assert_eq!(envelope.effects[0].id, created);
    let definition_id = envelope.effects[0].definition_id;
    let definition = attached
        .snapshot
        .effect_definition(definition_id)
        .expect("created definition");
    assert_eq!(definition.plugin_id, "core.filter.opacity");
    assert_eq!(definition.effect_version, 1);
    assert!(definition.enabled);
    assert_eq!(
        definition.params,
        BTreeMap::from([("amount".into(), motolii_doc::DocParam::const_f64(1.0))])
    );
    assert!(definition.extra.is_empty());
    assert_eq!(attached.snapshot.effect_definitions.len(), 1);
    assert_eq!(
        attached.snapshot.next_stable_id.peek_next(),
        initial_stable + 2
    );
    let attached_json = serde_json::to_vec(&*attached.snapshot).unwrap();

    queue.push_undo();
    let undone = runtime
        .process_next(&mut queue, Some(primary), 5)
        .unwrap()
        .expect("Undo must publish");
    assert_eq!(undone.kind, DocumentEditActionKind::Undo);
    assert_eq!(undone.created_effect_use, None);
    assert_eq!(undone.primary, Some(primary));
    assert!(runtime
        .writer
        .find_envelope(primary)
        .expect("primary after Undo")
        .effects
        .is_empty());
    assert!(undone.snapshot.effect_definition(definition_id).is_none());
    assert_eq!(runtime.history_lengths(), (0, 1));

    queue.push_redo();
    let redone = runtime
        .process_next(&mut queue, Some(primary), 6)
        .unwrap()
        .expect("Redo must publish");
    assert_eq!(redone.kind, DocumentEditActionKind::Redo);
    assert_eq!(redone.created_effect_use, Some(created));
    assert_eq!(redone.primary, Some(primary));
    let redone_envelope = runtime
        .writer
        .find_envelope(primary)
        .expect("primary after Redo");
    assert_eq!(redone_envelope.effects.len(), 1);
    assert_eq!(redone_envelope.effects[0].id, created);
    assert_eq!(redone_envelope.effects[0].definition_id, definition_id);
    assert_eq!(
        serde_json::to_vec(&*redone.snapshot).unwrap(),
        attached_json
    );
    assert_eq!(runtime.history_lengths(), (1, 0));

    queue.push_undo();
    let undone_for_foreign_primary = runtime
        .process_next(&mut queue, Some(primary), 7)
        .unwrap()
        .expect("Undo must publish before foreign-primary Redo");
    assert_eq!(undone_for_foreign_primary.created_effect_use, None);

    queue.push_redo();
    let foreign_primary_redo = runtime
        .process_next(&mut queue, None, 8)
        .unwrap()
        .expect("foreign-primary Redo must still publish the Document");
    assert_eq!(foreign_primary_redo.created_effect_use, None);
    assert_eq!(foreign_primary_redo.primary, None);

    let request = SetEffectParamRequest::new(
        primary,
        created,
        definition_id,
        "core.filter.opacity".into(),
        1,
        "amount".into(),
        0.4,
    );
    queue.push_set_effect_param(request);
    let changed = runtime
        .process_next(&mut queue, Some(primary), 9)
        .unwrap()
        .expect("SetEffectParam must publish");
    assert_eq!(changed.kind, DocumentEditActionKind::SetEffectParam);
    assert_eq!(changed.created_effect_use, None);

    queue.push_undo();
    let undone_param = runtime
        .process_next(&mut queue, Some(primary), 10)
        .unwrap()
        .expect("undo param must publish");
    assert_eq!(undone_param.kind, DocumentEditActionKind::Undo);
    assert_eq!(undone_param.created_effect_use, None);

    queue.push_redo();
    let redo_param = runtime
        .process_next(&mut queue, Some(primary), 11)
        .unwrap()
        .expect("redo param must publish");
    assert_eq!(redo_param.kind, DocumentEditActionKind::Redo);
    assert_eq!(redo_param.created_effect_use, None);

    queue.push_undo();
    let undo_param_restored = runtime
        .process_next(&mut queue, Some(primary), 12)
        .unwrap()
        .expect("undo param must publish");
    assert_eq!(undo_param_restored.kind, DocumentEditActionKind::Undo);
    assert_eq!(undo_param_restored.created_effect_use, None);

    drop(runtime);
    let limits = ResourceLimits::production();
    let (_session, reopened) = ProjectSession::open(&path, &limits).unwrap();
    assert_eq!(
        serde_json::to_vec(&reopened.document).unwrap(),
        attached_json
    );
}
