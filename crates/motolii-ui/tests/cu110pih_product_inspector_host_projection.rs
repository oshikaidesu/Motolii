//! CU-110PIH: adopted snapshot / primaryを第二のoffline Inspector islandへ投影する。

#[test]
fn product_host_projects_the_adopted_snapshot_without_a_second_owner_or_intent_path() {
    let product = include_str!("../src/product_runtime.rs");
    let inspector = include_str!("../src/inspector_host_runtime.rs");
    let web = include_str!("../../../ui/motolii-web/src/host/inspector-main.jsx");
    let publish = product
        .split("Ok(Some(published)) => {")
        .nth(1)
        .expect("product Host adopts one published snapshot")
        .split("Ok(None) =>")
        .next()
        .expect("publish arm is bounded");

    assert_eq!(product.matches("InspectorHostRuntime::new(").count(), 1);
    assert!(product.contains(
        "InspectorHostRuntime::new(\n            &window,\n            &self.current_document,\n            self.primary,\n            self.active_effect_use,\n        )"
    ));
    assert!(product.contains("inspector.set_bounds(layout.epoch, layout.inspector)"));
    assert_eq!(product.matches("inspector.publish(").count(), 3);
    assert!(!product.contains("inspector.publish(&self.current_document, self.primary)"));
    assert!(publish.contains("self.reconcile_active_effect_use(&published);"));
    let reconcile_pos = publish
        .find("self.reconcile_active_effect_use(&published);")
        .expect("publish arm reconciles active identity");
    let publish_pos = publish
        .find("inspector.publish(")
        .expect("publish arm forwards the adopted snapshot");
    assert!(reconcile_pos < publish_pos);
    assert!(publish.contains("self.active_effect_use"));
    assert!(inspector.contains("document,"));
    assert!(inspector.contains("target: InspectorTarget { layer_id: primary }"));
    assert!(inspector.contains("motolii_plugins_firstparty::first_party_catalog()"));
    assert!(inspector.contains("map_parameter_control(param)"));
    assert!(inspector.contains("fixture_revision: 1"));
    assert!(inspector.contains("active_effect_use_id: active_effect_use"));
    assert!(inspector.contains("snapshot_json(document, primary, active_effect_use)"));
    assert!(!inspector.contains("snapshot_json(document, primary, None)"));
    assert!(web.contains("decodeInspectorReadModel(raw)"));
    assert!(web.contains("<InspectorCandidate inspectorReadModel={inspectorReadModel} />"));

    for forbidden in [
        "docs/mocks-ui",
        "docs/mocks/",
        "onPlaceIntent",
        "postMessage",
        "DocumentWriter",
        "apply_macro",
    ] {
        assert!(
            !inspector.contains(forbidden) && !web.contains(forbidden),
            "Inspector Host crossed a stopped boundary: {forbidden}"
        );
    }
}
