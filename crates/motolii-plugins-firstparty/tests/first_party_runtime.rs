use std::collections::BTreeSet;

use motolii_plugin::reference::{reference_catalog, register_reference_plugins};
use motolii_plugin::{MigrationOp, MigrationStep, PluginKind, PluginRegistry};
use motolii_plugin_radial_repeater::radial_repeater_contract;
use motolii_plugin_sine::sine_contract;
use motolii_plugins_firstparty::{first_party_catalog, first_party_registry, first_party_runtime};

const EXPECTED_FIRST_PARTY_IDS: &[&str] = &[
    "core.layer_source.clear",
    "core.layer_source.radial_repeater",
    "core.filter.clear",
    "core.filter.tint",
    "core.filter.opacity",
    "core.param.sine",
    "core.composite.clear",
];

#[test]
fn first_party_catalog_owns_sine_migration() {
    let catalog = first_party_catalog().unwrap();
    let sine = catalog.get("core.param.sine").unwrap();
    assert_eq!(sine.migrations, sine_contract().migrations);
    assert_eq!(
        sine.migrations,
        vec![MigrationStep {
            from_version: 1,
            to_version: 2,
            ops: vec![MigrationOp::RenameParam {
                from: "amp",
                to: "amplitude",
            }],
        }]
    );
}

#[test]
fn first_party_runtime_succeeds_with_required_opacity_capability() {
    let runtime = first_party_runtime().unwrap();
    assert!(runtime.catalog().get("core.filter.opacity").is_some());
    assert!(runtime
        .executors()
        .filter_by_name("core.filter.opacity")
        .is_some());
}

#[test]
fn first_party_catalog_exposes_fixed_id_set() {
    let first_party_ids: BTreeSet<_> = first_party_catalog()
        .unwrap()
        .iter()
        .map(|(id, _)| id.0)
        .collect();
    assert_eq!(
        first_party_ids,
        EXPECTED_FIRST_PARTY_IDS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn reference_catalog_omits_externalized_first_party_plugins() {
    let reference_ids: BTreeSet<_> = reference_catalog()
        .unwrap()
        .iter()
        .map(|(id, _)| id.0)
        .collect();
    assert!(!reference_ids.contains("core.filter.opacity"));
    assert!(!reference_ids.contains("core.param.sine"));
    assert!(!reference_ids.contains("core.layer_source.radial_repeater"));
    assert_eq!(reference_ids.len(), EXPECTED_FIRST_PARTY_IDS.len() - 3);
}

#[test]
fn reference_registry_omits_externalized_first_party_executors() {
    let mut legacy = PluginRegistry::new();
    register_reference_plugins(&mut legacy).unwrap();
    let legacy_ids = executor_id_set(&legacy);
    assert!(!legacy_ids.contains("core.filter.opacity"));
    assert!(!legacy_ids.contains("core.param.sine"));
    assert!(!legacy_ids.contains("core.layer_source.radial_repeater"));
    assert_eq!(legacy_ids.len(), EXPECTED_FIRST_PARTY_IDS.len() - 3);
}

#[test]
fn first_party_registry_exposes_fixed_executor_id_set() {
    let first_party_ids = executor_id_set(&first_party_registry().unwrap());
    assert_eq!(
        first_party_ids,
        EXPECTED_FIRST_PARTY_IDS
            .iter()
            .copied()
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
    );
}

fn executor_id_set(registry: &PluginRegistry) -> BTreeSet<String> {
    [
        PluginKind::LayerSource,
        PluginKind::Filter,
        PluginKind::ParamDriver,
        PluginKind::Composite,
    ]
    .into_iter()
    .flat_map(|kind| registry.iter(kind).map(|(id, _)| id.0.to_string()))
    .collect()
}

#[test]
fn p7_firstparty_radial_repeater_parity() {
    let contract = radial_repeater_contract();
    assert_eq!(contract.kind, PluginKind::LayerSource);
    assert_eq!(contract.node.id.0, "core.layer_source.radial_repeater");
    assert_eq!(contract.node.version, 1);

    let catalog = first_party_catalog().unwrap();
    let catalog_entry = catalog.get("core.layer_source.radial_repeater").unwrap();
    assert_eq!(catalog_entry.kind, PluginKind::LayerSource);
    assert_eq!(catalog_entry.node.id, contract.node.id);
    assert_eq!(catalog_entry.node.version, contract.node.version);
    assert_eq!(catalog_entry.node.display_name, contract.node.display_name);
    assert_eq!(catalog_entry.node.category, contract.node.category);
    assert_eq!(catalog_entry.node.params, contract.node.params);
    assert_eq!(catalog_entry.migrations, contract.migrations);

    let registry = first_party_registry().unwrap();
    let executor = registry
        .layer_source_by_name("core.layer_source.radial_repeater")
        .unwrap();
    assert_eq!(executor.desc().id, contract.node.id);
    assert_eq!(executor.desc().version, contract.node.version);
    assert_eq!(executor.desc().display_name, contract.node.display_name);
    assert_eq!(executor.desc().params, contract.node.params);
}
