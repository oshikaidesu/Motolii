//! VSM-A0I-3: exportが内部でreference runtimeを再生成しない静的審判。

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn export_requires_caller_runtime_and_has_no_internal_reference_registry() {
    let root = workspace_root();
    let export = fs::read_to_string(root.join("crates/motolii-export/src/lib.rs")).unwrap();
    assert!(export.contains("pub runtime: &'a PluginRuntime"));
    assert!(!export.contains("register_reference_plugins"));
    assert!(!export.contains("PluginRegistry::new"));

    let graph = fs::read_to_string(root.join("crates/motolii-doc/src/graph.rs")).unwrap();
    let signature = graph
        .split("pub fn build_document_frame_graph")
        .nth(1)
        .unwrap()
        .split(") -> Result<DocumentFrameGraph")
        .next()
        .unwrap();
    assert!(signature.contains("runtime: &PluginRuntime"));
    assert!(!signature.contains("registry: &PluginRegistry"));
}

#[test]
fn product_document_export_uses_resolved_open_not_raw_load() {
    let source =
        fs::read_to_string(workspace_root().join("crates/motolii-cli/src/document_export.rs"))
            .unwrap();
    assert!(source.contains("open_project_resolved("));
    assert!(!source.contains("load_document("));
    assert!(!source.contains("open_project_with_limits("));
}
