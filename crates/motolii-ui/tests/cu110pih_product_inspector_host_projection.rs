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

    assert!(product.contains("InspectorHostRuntime::new("));
    assert!(product.contains("inspector.set_bounds(layout.epoch, layout.inspector)"));
    assert!(publish.contains("inspector.publish(&self.current_document, self.primary)"));
    assert!(inspector.contains("\"document\": document"));
    assert!(inspector.contains("\"target\": { \"layer_id\": primary }"));
    assert!(inspector.contains("\"nodes\": []"));
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
