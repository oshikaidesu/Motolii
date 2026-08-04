//! CU-110: 通常製品Hostのaccepted Placeを既存D2 runtimeへ接続する。

#[test]
fn pending_stage_drop_reaches_one_rectangle_action_and_one_runtime_process() {
    let source = include_str!("../src/product_runtime.rs");
    let delivery = source
        .split("if let Some(drop) = self.pending_stage_drop.take() {")
        .nth(1)
        .expect("product Host consumes the single downstream boundary")
        .split("pub(crate) fn handle_product_event")
        .next()
        .expect("Place commit arm is bounded");

    assert_eq!(delivery.matches("push_place_rectangle(").count(), 1);
    assert_eq!(delivery.matches(".process_next(").count(), 1);
    assert!(delivery.contains("canonical_drop_from_ndc(self.displayed_camera, drop.ndc)"));
    assert!(delivery.contains("playhead: RationalTime::ZERO"));
    assert!(delivery.contains("self.adopt_full_publish(event_loop, published, \"place\")"));
    assert!(source.contains("self.primary = published.primary"));
    assert!(source.contains("self.projection_generation = published.projection_generation"));
}

#[test]
fn cancel_arms_never_enqueue_document_work() {
    let source = include_str!("../src/product_runtime.rs");
    let cancel = source
        .split("Ok(Some(HostPointerCandidate::Cancelled { generation, reason })) =>")
        .nth(1)
        .expect("product Host handles cancellation")
        .split("Ok(None) =>")
        .next()
        .expect("cancel arm is bounded");

    for forbidden in ["push_place_rectangle", "process_next", "apply_macro"] {
        assert!(
            !cancel.contains(forbidden),
            "cancel reached D2: {forbidden}"
        );
    }
}
