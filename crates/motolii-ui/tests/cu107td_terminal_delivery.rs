//! CU-107TD: admitted terminalだけを単一下流境界へ配送する。

#[test]
fn production_admit_arm_delivers_only_to_pending_stage_drop() {
    let source = include_str!("../src/product_runtime.rs");
    let admit = source
        .split("if self.terminal_admission.admit(&terminal) {")
        .nth(1)
        .expect("production admission arm exists")
        .split('}')
        .next()
        .expect("admission arm closes");

    assert!(admit.contains("self.terminal_delivery.deliver(&terminal)"));
    assert!(admit.contains("self.pending_stage_drop ="));
    for forbidden in ["process_next", "apply_macro", "undo", "redo"] {
        assert!(
            !admit.contains(forbidden),
            "delivery reached D2: {forbidden}"
        );
    }
}
