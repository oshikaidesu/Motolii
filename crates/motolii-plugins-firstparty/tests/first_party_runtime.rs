use std::collections::BTreeSet;

use motolii_plugin::reference::{reference_catalog, register_reference_plugins};
use motolii_plugin::{PluginKind, PluginRegistry};
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
fn first_party_runtime_succeeds_with_required_opacity_capability() {
    let runtime = first_party_runtime().unwrap();
    assert!(runtime.catalog().get("core.filter.opacity").is_some());
    assert!(runtime
        .executors()
        .filter_by_name("core.filter.opacity")
        .is_some());
}

#[test]
fn catalog_id_set_matches_reference_catalog() {
    let reference_ids: BTreeSet<_> = reference_catalog()
        .unwrap()
        .iter()
        .map(|(id, _)| id.0)
        .collect();
    let first_party_ids: BTreeSet<_> = first_party_catalog()
        .unwrap()
        .iter()
        .map(|(id, _)| id.0)
        .collect();
    assert_eq!(first_party_ids, reference_ids);
    assert_eq!(
        first_party_ids,
        EXPECTED_FIRST_PARTY_IDS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn executor_id_set_matches_register_reference_plugins() {
    let first_party_ids = executor_id_set(&first_party_registry().unwrap());
    let mut legacy = PluginRegistry::new();
    register_reference_plugins(&mut legacy).unwrap();
    let legacy_ids = executor_id_set(&legacy);
    assert_eq!(first_party_ids, legacy_ids);
    assert_eq!(
        first_party_ids,
        EXPECTED_FIRST_PARTY_IDS
            .iter()
            .copied()
            .map(str::to_string)
            .collect()
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
