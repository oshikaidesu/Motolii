//! CU-106P: native Timeline hitをprivate primary selection producerへ実配送する。

#[test]
fn production_timeline_click_reaches_typed_hit_and_one_selection_action() {
    let source = include_str!("../src/product_runtime.rs");
    let consumer = source
        .split("fn handle_timeline_click(")
        .nth(1)
        .expect("non-test product Timeline click consumer exists")
        .split("\n    fn ")
        .next()
        .expect("selection consumer is bounded");

    assert!(source.contains("browser.poll_host_click()"));
    assert!(source.contains("self.timeline_projection.hit_test(position, layout)"));
    assert!(consumer.contains("TimelineHit::Key { layer, .. } | TimelineHit::Bar { layer }"));
    assert_eq!(consumer.matches("push_replace_primary(").count(), 1);
    assert_eq!(consumer.matches("push_clear_primary()").count(), 1);
    assert_eq!(consumer.matches(".process_next(").count(), 1);
    assert!(consumer.contains("self.primary = published.primary"));
    assert!(consumer.contains("self.projection_generation = published.projection_generation"));

    for forbidden in [
        "DomainIntent",
        "apply_macro",
        "save_with_journal",
        "TimelineCandidate",
    ] {
        assert!(
            !consumer.contains(forbidden),
            "selection consumer crossed a stopped boundary: {forbidden}"
        );
    }
}
