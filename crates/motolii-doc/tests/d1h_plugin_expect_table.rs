//! VSM-A0I-2: Document側へfirst-party plugin ID表を再導入しない審判。

use motolii_plugins_firstparty::first_party_catalog;

#[test]
fn plugin_contracts_live_in_catalog_not_param_expect() {
    let source = include_str!("../src/param_expect.rs");
    for forbidden in [
        "core.filter.clear",
        "core.filter.tint",
        "core.filter.opacity",
        "core.layer_source.clear",
        "core.composite.clear",
        "core.param.sine",
        "known_plugin_param",
        "known_plugin_info",
        "DocPluginKind",
    ] {
        assert!(
            !source.contains(forbidden),
            "Document-side plugin mirror returned: {forbidden}"
        );
    }

    let catalog = first_party_catalog().unwrap();
    assert_eq!(catalog.len(), 7);
    assert!(catalog.get("core.filter.opacity").is_some());
    assert!(catalog.get("core.param.sine").is_some());
    assert!(catalog.get("core.layer_source.radial_repeater").is_some());
}
