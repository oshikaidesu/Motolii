//! VSM-A0I-2: raw Documentと一時的なprepared recipeの境界。

use std::collections::BTreeMap;
pub mod common;
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::RationalTime;
use motolii_doc::{
    open_project_resolved, Clip, ClipSource, Command, CommandError, DocParam, Document,
    DocumentPluginError, DocumentWriter, EffectDefinition, EffectDefinitionId, EffectId, EffectUse,
    ItemEnvelope, PluginDiagnosticReason, PluginSlotId, ResourceLimits, ScalarPropertyId, Track,
    TrackItem,
};
use motolii_eval::Value;
use motolii_plugin::{
    MigrationOp, MigrationStep, NodeDesc, ParamDef, PluginCatalog, PluginCatalogBuilder,
    PluginContract, PluginId, PluginKind, ValueType,
};
use motolii_plugins_firstparty::first_party_catalog;

fn document_with_effect(
    plugin_id: &str,
    version: u32,
    params: BTreeMap<String, DocParam>,
) -> (Document, motolii_doc::LayerId, EffectId, EffectDefinitionId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("subject").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let definition_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        definition_id,
        plugin_id,
        version,
        true,
        params,
        Default::default(),
    ));
    let mut envelope = ItemEnvelope::new(layer);
    envelope.effects.push(EffectUse {
        id: use_id,
        definition_id,
    });
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc.validate().unwrap();
    (doc, layer, use_id, definition_id)
}

fn incomplete_chain_catalog() -> PluginCatalog {
    let mut builder = PluginCatalogBuilder::new();
    builder
        .register(PluginContract {
            kind: PluginKind::Filter,
            node: NodeDesc {
                id: PluginId("vendor.filter.chain_gap"),
                version: 3,
                display_name: "Chain gap",
                category: "Test",
                tags: &["test"],
                params: vec![ParamDef {
                    id: "amount",
                    value_type: ValueType::F64,
                    default: Value::F64(0.0),
                    f64_domain: None,
                }],
                min_inputs: 1,
                max_inputs: 1,
            },
            migrations: vec![MigrationStep {
                from_version: 1,
                to_version: 2,
                ops: vec![MigrationOp::RenameParam {
                    from: "old_amount",
                    to: "amount",
                }],
            }],
        })
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn sine_rename_changes_only_the_prepared_clone() {
    let catalog = first_party_catalog().unwrap();
    let raw = BTreeMap::from([
        ("amp".into(), DocParam::const_f64(0.25)),
        ("frequency_hz".into(), DocParam::const_f64(2.0)),
        ("offset".into(), DocParam::const_f64(0.5)),
    ]);
    let before = raw.clone();

    let prepared = motolii_doc::prepare_plugin_recipe(
        "core.param.sine",
        PluginKind::ParamDriver,
        1,
        &raw,
        &catalog,
    )
    .unwrap();

    assert_eq!(raw, before);
    assert!(!prepared.params.contains_key("amp"));
    assert_eq!(
        prepared.params.get("amplitude"),
        Some(&DocParam::const_f64(0.25))
    );
    assert_eq!(prepared.saved_version, 1);
    assert_eq!(prepared.current_version, 2);
}

#[test]
fn migration_conflict_and_old_shape_mismatch_are_distinct_hard_errors() {
    let catalog = first_party_catalog().unwrap();
    let conflict = BTreeMap::from([
        ("amp".into(), DocParam::const_f64(0.25)),
        ("amplitude".into(), DocParam::const_f64(0.5)),
    ]);
    assert!(matches!(
        motolii_doc::prepare_plugin_recipe(
            "core.param.sine",
            PluginKind::ParamDriver,
            1,
            &conflict,
            &catalog,
        ),
        Err(DocumentPluginError::MigrationConflict { .. })
    ));

    let wrong_shape = BTreeMap::from([("frequency_hz".into(), DocParam::const_f64(1.0))]);
    assert!(matches!(
        motolii_doc::prepare_plugin_recipe(
            "core.param.sine",
            PluginKind::ParamDriver,
            1,
            &wrong_shape,
            &catalog,
        ),
        Err(DocumentPluginError::ContractViolation { param, .. }) if param == "amp"
    ));
}

#[test]
fn missing_contract_future_version_and_chain_gap_remain_separate_diagnostics() {
    let reference = first_party_catalog().unwrap();
    let (unknown, _, _, unknown_definition) = document_with_effect(
        "vendor.filter.absent",
        1,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    let unknown_prepared = unknown.prepare_plugins(&reference).unwrap();
    assert_eq!(
        unknown_prepared.diagnostics()[0].reason,
        PluginDiagnosticReason::ContractMissing
    );
    assert!(unknown_prepared
        .get(&PluginSlotId::EffectDefinition(unknown_definition))
        .is_none());

    let (future, _, _, _) = document_with_effect(
        "core.filter.opacity",
        2,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    assert!(matches!(
        future.prepare_plugins(&reference).unwrap().diagnostics()[0].reason,
        PluginDiagnosticReason::FutureVersion {
            current_version: 1,
            saved_version: 2
        }
    ));

    let (gap, _, _, _) = document_with_effect(
        "vendor.filter.chain_gap",
        2,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    assert!(matches!(
        gap.prepare_plugins(&incomplete_chain_catalog())
            .unwrap()
            .diagnostics()[0]
            .reason,
        PluginDiagnosticReason::MigrationStepMissing { from_version: 2 }
    ));
}

#[test]
fn kind_mismatch_is_catalog_typed_not_intrinsic() {
    let catalog = first_party_catalog().unwrap();
    let (doc, _, _, _) = document_with_effect(
        "core.layer_source.clear",
        1,
        BTreeMap::from([("color".into(), DocParam::const_color([0.0, 0.0, 0.0, 1.0]))]),
    );
    doc.validate().unwrap();
    assert!(matches!(
        doc.prepare_plugins(&catalog),
        Err(DocumentPluginError::KindMismatch {
            expected: PluginKind::Filter,
            actual: PluginKind::LayerSource,
            ..
        })
    ));
}

#[test]
fn opacity_domain_is_owned_by_contract_and_checks_both_boundaries() {
    let catalog = first_party_catalog().unwrap();
    for amount in [-0.01, 1.01] {
        let (doc, _, _, _) = document_with_effect(
            "core.filter.opacity",
            1,
            BTreeMap::from([("amount".into(), DocParam::const_f64(amount))]),
        );
        doc.validate().unwrap();
        assert!(matches!(
            doc.prepare_plugins(&catalog),
            Err(DocumentPluginError::ContractViolation { param, .. }) if param == "amount"
        ));
    }
}

#[test]
fn failed_plugin_edit_restores_raw_document_revision_and_undo() {
    let catalog = Arc::new(first_party_catalog().unwrap());
    let (doc, layer, effect, _) = document_with_effect(
        "core.filter.opacity",
        1,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    let mut writer = DocumentWriter::new(doc, catalog).unwrap();
    let before = writer.snapshot();
    let revision = writer.revision;
    let undo_len = writer.undo_len();
    let gesture = writer.begin_gesture();

    let error = writer
        .apply_command(
            gesture,
            Command::SetProperty {
                target: layer,
                property: ScalarPropertyId::EffectParam(effect, "amount".into()),
                old_value: DocParam::const_f64(0.5),
                new_value: DocParam::const_f64(1.01),
            },
        )
        .unwrap_err();

    assert!(matches!(error, CommandError::Plugin(_)));
    assert_eq!(&*writer.snapshot(), &*before);
    assert_eq!(writer.revision, revision);
    assert_eq!(writer.undo_len(), undo_len);
}

#[test]
fn unknown_plugin_roundtrips_and_does_not_block_unrelated_edit() {
    let catalog = Arc::new(first_party_catalog().unwrap());
    let (doc, layer, _, definition) = document_with_effect(
        "vendor.filter.absent",
        1,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    let raw_before = serde_json::to_vec(&doc).unwrap();
    let prepared = doc.prepare_plugins(&catalog).unwrap();
    assert!(!prepared.is_fully_prepared());
    assert_eq!(serde_json::to_vec(&doc).unwrap(), raw_before);

    let mut writer = DocumentWriter::new(doc, catalog.clone()).unwrap();
    let gesture = writer.begin_gesture();
    writer
        .apply_command(
            gesture,
            Command::SetProperty {
                target: layer,
                property: ScalarPropertyId::Position,
                old_value: DocParam::const_vec2([0.0, 0.0]),
                new_value: DocParam::const_vec2([0.25, -0.25]),
            },
        )
        .unwrap();

    let snapshot = writer.snapshot();
    assert_eq!(
        snapshot
            .effect_definition(definition)
            .unwrap()
            .params
            .get("amount"),
        Some(&DocParam::const_f64(0.5))
    );
    assert_eq!(
        snapshot.prepare_plugins(&catalog).unwrap().diagnostics()[0].reason,
        PluginDiagnosticReason::ContractMissing
    );
}

#[test]
fn resolved_project_open_returns_raw_document_with_prepared_diagnostics() {
    let catalog = first_party_catalog().unwrap();
    let (doc, _, _, definition) = document_with_effect(
        "vendor.filter.absent",
        1,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
    );
    let raw = serde_json::to_vec(&doc).unwrap();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-a0i2-resolved-open-{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("project.json");
    common::session::save_document_via_session(&path, &doc);
    let opened = open_project_resolved(&path, &ResourceLimits::production(), &catalog).unwrap();
    let _session = opened.session;

    assert_eq!(serde_json::to_vec(&opened.recovered.document).unwrap(), raw);
    assert_eq!(
        opened.plugins.diagnostics()[0].reason,
        PluginDiagnosticReason::ContractMissing
    );
    assert!(opened
        .plugins
        .get(&PluginSlotId::EffectDefinition(definition))
        .is_none());
    fs::remove_dir_all(dir).ok();
}
