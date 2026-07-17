//! composition rootへの集約で既存pluginが静かに欠落しないことを固定する。

use std::collections::BTreeSet;

use motolii_plugin::{
    reference::{reference_catalog, register_reference_plugins},
    PluginCatalog, PluginKind, PluginRegistry,
};
use motolii_plugins_firstparty::{
    first_party_catalog, first_party_registry, first_party_runtime, FirstPartyError,
};

fn catalog_ids(catalog: &PluginCatalog) -> BTreeSet<&'static str> {
    catalog.iter().map(|(id, _)| id.0).collect()
}

fn registry_ids(registry: &PluginRegistry, kind: PluginKind) -> BTreeSet<&'static str> {
    registry.iter(kind).map(|(id, _)| id.0).collect()
}

#[test]
fn first_party_apis_succeed() {
    let _ = first_party_catalog().unwrap();
    let _ = first_party_registry().unwrap();
    let _ = first_party_runtime().unwrap();
}

#[test]
fn catalog_id_parity_with_reference_catalog() {
    let reference = reference_catalog().unwrap();
    let first_party = first_party_catalog().unwrap();

    assert_eq!(reference.len(), first_party.len());
    assert_eq!(catalog_ids(&reference), catalog_ids(&first_party));
    assert!(first_party.get("core.filter.opacity").is_some());
}

#[test]
fn registry_id_parity_with_register_reference_plugins() {
    let mut reference = PluginRegistry::new();
    register_reference_plugins(&mut reference).unwrap();
    let first_party = first_party_registry().unwrap();

    for kind in [
        PluginKind::LayerSource,
        PluginKind::Filter,
        PluginKind::ParamDriver,
        PluginKind::Composite,
    ] {
        assert_eq!(
            reference.len(kind),
            first_party.len(kind),
            "len mismatch for {kind:?}"
        );
        assert_eq!(
            registry_ids(&reference, kind),
            registry_ids(&first_party, kind),
            "id mismatch for {kind:?}"
        );
    }
    assert!(first_party.filter_by_name("core.filter.opacity").is_some());
}

#[test]
fn first_party_error_preserves_distinct_variants() {
    let contract: FirstPartyError =
        motolii_plugin::PluginContractError::DuplicateContract { id: "test" }.into();
    let plugin: FirstPartyError = motolii_plugin::PluginError::Duplicate {
        kind: PluginKind::Filter,
        id: "test",
    }
    .into();
    let runtime: FirstPartyError = motolii_plugin::PluginRuntimeError::ExecutorContractMissing {
        id: "test",
        kind: PluginKind::Filter,
    }
    .into();

    assert!(matches!(contract, FirstPartyError::Contract(_)));
    assert!(matches!(plugin, FirstPartyError::Plugin(_)));
    assert!(matches!(runtime, FirstPartyError::Runtime(_)));
}
