//! composition rootへの集約で既存pluginが静かに欠落しないことを固定する。

use std::collections::BTreeSet;

use motolii_plugin::{F64Domain, PluginCatalog, PluginKind, PluginRegistry};
use motolii_plugins_firstparty::{
    first_party_catalog, first_party_registry, first_party_runtime, FirstPartyError,
};

fn catalog_ids(catalog: &PluginCatalog) -> BTreeSet<&'static str> {
    catalog.iter().map(|(id, _)| id.0).collect()
}

fn registry_ids(registry: &PluginRegistry, kind: PluginKind) -> BTreeSet<&'static str> {
    registry.iter(kind).map(|(id, _)| id.0).collect()
}

const EXPECTED_CATALOG_IDS: &[&str] = &[
    "core.layer_source.clear",
    "core.filter.clear",
    "core.filter.tint",
    "core.filter.opacity",
    "core.param.sine",
    "core.composite.clear",
];

#[test]
fn first_party_apis_succeed() {
    let _ = first_party_catalog().unwrap();
    let _ = first_party_registry().unwrap();
    let _ = first_party_runtime().unwrap();
}

#[test]
fn first_party_catalog_has_six_contracts_including_opacity() {
    let catalog = first_party_catalog().unwrap();
    assert_eq!(catalog.len(), 6);
    assert_eq!(
        catalog_ids(&catalog),
        EXPECTED_CATALOG_IDS.iter().copied().collect()
    );
    assert!(catalog.get("core.filter.opacity").is_some());
}

#[test]
fn opacity_domain_is_owned_by_first_party_catalog() {
    let catalog = first_party_catalog().unwrap();
    let opacity = catalog.get("core.filter.opacity").unwrap();
    assert_eq!(opacity.kind, PluginKind::Filter);
    assert_eq!(opacity.node.params[0].f64_domain, Some(F64Domain::unit()));
}

#[test]
fn first_party_registry_has_three_filters_including_opacity() {
    let registry = first_party_registry().unwrap();
    assert_eq!(registry.len(PluginKind::LayerSource), 1);
    assert_eq!(registry.len(PluginKind::Filter), 3);
    assert_eq!(registry.len(PluginKind::ParamDriver), 1);
    assert_eq!(registry.len(PluginKind::Composite), 1);
    let filter_ids = registry_ids(&registry, PluginKind::Filter);
    assert!(filter_ids.contains("core.filter.clear"));
    assert!(filter_ids.contains("core.filter.tint"));
    assert!(filter_ids.contains("core.filter.opacity"));
    assert!(registry.filter_by_name("core.filter.opacity").is_some());
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
    let missing = FirstPartyError::MissingRequiredCapability {
        id: "core.filter.opacity",
    };

    assert!(matches!(contract, FirstPartyError::Contract(_)));
    assert!(matches!(plugin, FirstPartyError::Plugin(_)));
    assert!(matches!(runtime, FirstPartyError::Runtime(_)));
    assert!(matches!(
        missing,
        FirstPartyError::MissingRequiredCapability { .. }
    ));
}
