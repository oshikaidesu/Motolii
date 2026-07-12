//! 既知plugin期待型表と reference registry の乖離検出(D1h)。

use motolii_doc::param_expect::{known_plugin_ids, known_plugin_param, ExpectedValueType};
use motolii_plugin::{
    reference::register_reference_plugins, PluginKind, PluginRegistry, ValueType,
};

fn value_type_to_expected(vt: ValueType) -> ExpectedValueType {
    match vt {
        ValueType::F64 => ExpectedValueType::F64,
        ValueType::Vec2 => ExpectedValueType::Vec2,
        ValueType::Vec3 => ExpectedValueType::Vec3,
        ValueType::Color => ExpectedValueType::Color,
        ValueType::AssetRef => ExpectedValueType::AssetRef,
    }
}

#[test]
fn known_plugin_param_table_covers_reference_registry() {
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();

    let kinds = [
        PluginKind::LayerSource,
        PluginKind::Filter,
        PluginKind::ParamDriver,
        PluginKind::Composite,
    ];
    let mut seen_ids = Vec::new();
    for kind in kinds {
        for (id, plugin) in registry.iter(kind) {
            seen_ids.push(id.0);
            let desc = plugin.desc();
            for param in &desc.params {
                let constraints = known_plugin_param(id.0, param.id).unwrap_or_else(|| {
                    panic!("missing known_plugin_param entry for {}.{}", id.0, param.id)
                });
                assert_eq!(
                    constraints.expected,
                    value_type_to_expected(param.value_type),
                    "type mismatch for {}.{}",
                    id.0,
                    param.id
                );
            }
        }
    }

    // 表の余剰 plugin_id がレジストリに無いことも検出
    for &table_id in known_plugin_ids() {
        assert!(
            seen_ids.contains(&table_id),
            "known_plugin_ids has orphan entry {table_id} not in reference registry"
        );
    }
}
