use std::collections::BTreeSet;

use motolii_plugin::reference::{reference_catalog, register_reference_plugins};
use motolii_plugin::{MigrationOp, MigrationStep, PluginKind, PluginRegistry};
use motolii_plugin_sine::sine_contract;
use motolii_plugins_firstparty::{first_party_catalog, first_party_registry, first_party_runtime};

const EXPECTED_FIRST_PARTY_IDS: &[&str] = &[
    "core.layer_source.clear",
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
    assert_eq!(reference_ids.len(), EXPECTED_FIRST_PARTY_IDS.len() - 2);
}

#[test]
fn reference_registry_omits_externalized_first_party_executors() {
    let mut legacy = PluginRegistry::new();
    register_reference_plugins(&mut legacy).unwrap();
    let legacy_ids = executor_id_set(&legacy);
    assert!(!legacy_ids.contains("core.filter.opacity"));
    assert!(!legacy_ids.contains("core.param.sine"));
    assert_eq!(legacy_ids.len(), EXPECTED_FIRST_PARTY_IDS.len() - 2);
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
