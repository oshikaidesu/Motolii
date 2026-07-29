//! CU-107PV: 通常製品HostのMoved配送を非terminal previewへ限定する。

#[test]
fn production_moved_path_delivers_preview_without_terminal_or_document_work() {
    let source = include_str!("../src/product_runtime.rs");
    let moved = source
        .split("Ok(Some(HostPointerCandidate::Moved {")
        .nth(1)
        .expect("product Host handles Moved")
        .split("Ok(Some(HostPointerCandidate::Released {")
        .next()
        .expect("Moved arm is bounded by Released");

    assert!(
        moved.contains(".deliver(source, generation, position, layout);"),
        "production Moved path no longer delivers Host Transient preview"
    );
    assert!(
        moved.contains("event_loop.set_control_flow(ControlFlow::Poll);"),
        "active nonterminal preview no longer keeps pointer polling alive"
    );
    for forbidden in [
        "pending_stage_drop",
        "active_place.take()",
        "DocumentWriter",
        "DocumentEditQueue",
        "process_next",
        "apply_macro",
        "undo",
        "redo",
    ] {
        assert!(
            !moved.contains(forbidden),
            "Moved preview path gained terminal or Document work: {forbidden}"
        );
    }
}

#[test]
fn preview_state_is_private_transient_product_state() {
    let source = include_str!("../src/product_runtime.rs");

    assert!(source.contains("struct PlacePreviewPhase"));
    assert!(source.contains("struct PlacePreviewProgress"));
    assert!(!source.contains("pub struct PlacePreviewPhase"));
    assert!(!source.contains("pub struct PlacePreviewProgress"));
    assert!(!source.contains("Serialize for PlacePreview"));
    assert!(!source.contains("Deserialize for PlacePreview"));
}
