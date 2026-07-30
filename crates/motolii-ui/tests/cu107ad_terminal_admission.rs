//! CU-107AD: candidate terminal admissionを通常製品Host内へ閉じる。

#[test]
fn production_terminal_path_admits_without_downstream_delivery() {
    let source = include_str!("../src/product_runtime.rs");
    let terminal_path = source
        .split("Ok(Some(HostPointerCandidate::Released {")
        .nth(1)
        .expect("product Host handles Released")
        .split("Ok(None) =>")
        .next()
        .expect("terminal handling is bounded by idle polling");

    assert!(terminal_path.contains("self.terminal_admission.admit(&terminal)"));
    assert!(terminal_path.contains("self.admitted_terminal = Some(terminal.clone())"));
    assert!(
        !terminal_path.contains("self.pending_stage_drop = Some"),
        "admission must not perform accepted terminal delivery"
    );
    for forbidden in [
        "DocumentWriter",
        "DocumentEditQueue",
        "process_next",
        "apply_macro",
        "undo",
        "redo",
    ] {
        assert!(
            !terminal_path.contains(forbidden),
            "terminal admission gained delivery or Document work: {forbidden}"
        );
    }
}

#[test]
fn admission_state_is_private_and_retains_only_generation_watermarks() {
    let source = include_str!("../src/product_runtime.rs");
    let admission = source
        .split("struct PlaceTerminalAdmission {")
        .nth(1)
        .expect("private admission state exists")
        .split('}')
        .next()
        .expect("admission state closes");

    assert!(admission.contains("active_generation: Option<u64>"));
    assert!(admission.contains("retired_high_water: Option<u64>"));
    assert!(!source.contains("pub struct PlaceTerminalAdmission"));
    assert!(!admission.contains("BrowserPlaceIntent"));
    assert!(!admission.contains("layout_epoch"));
    assert!(!admission.contains("event_sequence"));
}
