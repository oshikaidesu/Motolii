//! CU-107TC: candidate terminalの理由分類を通常製品Host内へ閉じる。

#[test]
fn production_terminal_path_classifies_before_admission_or_document_work() {
    let source = include_str!("../src/product_runtime.rs");
    let terminal_path = source
        .split("Ok(Some(HostPointerCandidate::Released {")
        .nth(1)
        .expect("product Host handles Released")
        .split("Ok(None) =>")
        .next()
        .expect("terminal handling is bounded by idle polling");

    assert!(terminal_path.contains("ClassifiedPlaceTerminal::released("));
    assert!(terminal_path.contains("ClassifiedPlaceTerminal::cancelled("));
    assert!(terminal_path.contains("HostPointerCandidate::Cancelled { generation, reason }"));
    for forbidden in [
        "DocumentWriter",
        "DocumentEditQueue",
        "process_next",
        "apply_macro",
        "undo",
        "redo",
        "admit",
    ] {
        assert!(
            !terminal_path.contains(forbidden),
            "terminal classification gained admission or Document work: {forbidden}"
        );
    }
}

#[test]
fn terminal_cause_is_a_private_closed_set() {
    let source = include_str!("../src/product_runtime.rs");
    let causes = source
        .split("enum PlaceTerminalCause {")
        .nth(1)
        .expect("private terminal cause exists")
        .split('}')
        .next()
        .expect("terminal cause enum closes");

    for expected in ["Escape", "OutsideStage", "CaptureLoss", "NoNonCommitCause"] {
        assert!(
            causes.contains(expected),
            "missing terminal cause: {expected}"
        );
    }
    assert_eq!(
        causes
            .lines()
            .filter(|line| line.trim().ends_with(','))
            .count(),
        4,
        "candidate terminal cause set changed"
    );
    assert!(!source.contains("pub enum PlaceTerminalCause"));
}
