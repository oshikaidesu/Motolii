//! CU-111: 製品CommandIdから既存durability/publish経路へUndo/Redoを一意配送する。

#[test]
fn product_history_shortcuts_reach_one_private_prepared_action_path() {
    let domain = include_str!("../src/domain_intent.rs");
    let registry = include_str!("../src/command_registry.rs");
    let capture = include_str!("../src/host_pointer_capture.rs");
    let browser = include_str!("../src/browser_host_runtime.rs");
    let product = include_str!("../src/product_runtime.rs");
    let runtime = include_str!("../src/document_edit_runtime.rs");

    assert!(domain.contains("Undo,\n    Redo,"));
    assert!(registry.contains("\"motolii.edit.undo\""));
    assert!(registry.contains("\"motolii.edit.redo\""));
    assert!(capture.contains("NSEventMask::KeyDown"));
    assert!(capture.contains("host_commands_enabled"));
    assert!(browser.contains("set_host_commands_enabled(target == BrowserFocusTarget::Parent)"));

    let product_path = product
        .split("fn handle_history_trigger(")
        .nth(1)
        .expect("production history consumer exists")
        .split("fn drain_stage_projection")
        .next()
        .expect("history consumer is bounded");
    assert!(product.contains("resolve_keymap("));
    assert!(product_path.contains("NormalizedInput::Command"));
    assert_eq!(
        product_path.matches(".push_prepared(output, None)").count(),
        1
    );
    assert_eq!(product_path.matches(".process_next(").count(), 1);
    assert!(product_path.contains("self.adopt_history_publish(event_loop, published)"));
    assert!(product_path.contains("ProductTimelineProjection::from_document"));
    assert!(product_path.contains("self.submit_stage_projection()"));
    assert!(product_path.contains("inspector.publish"));

    assert!(runtime.contains("struct PreparedHistoryAction"));
    assert!(runtime.contains("struct HistoryProjection"));
    assert!(runtime.contains("self.undo.len() != writer.undo_len()"));
    assert!(runtime.contains("self.redo.len() != writer.redo_len()"));
    assert_eq!(runtime.matches("fn commit_durable(").count(), 1);
    assert!(runtime.contains("self.commit_durable(&command)?"));
    assert!(runtime.contains("self.commit_durable(&action.durable_command)?"));

    for forbidden in [
        "pub struct PreparedHistoryAction",
        "pub struct HistoryProjection",
        "writer.undo_stack",
        "writer.redo_stack",
    ] {
        assert!(
            !runtime.contains(forbidden),
            "CU-111 exposed a stopped history boundary: {forbidden}"
        );
    }
}
