//! CU-110PT: published snapshotを既存headless projectionからnative Timeline barへ描く。

#[test]
fn product_timeline_projects_the_adopted_snapshot_without_a_second_state_owner() {
    let source = include_str!("../src/product_runtime.rs");
    let publish = source
        .split("Ok(Some(published)) => {")
        .nth(1)
        .expect("product Host adopts one published snapshot")
        .split("Ok(None) =>")
        .next()
        .expect("publish arm is bounded");

    assert!(source.contains("project_timeline("));
    assert!(source.contains("TimelineViewport {"));
    assert!(source.contains("start: RationalTime::ZERO"));
    assert!(source.contains("end: document.composition.duration"));
    assert!(publish.contains("ProductTimelineProjection::from_document(&self.current_document)"));
    assert!(source.contains("timeline_projection.projection.bars()"));
    assert!(source.contains("timeline_bar_rect("));
    assert!(source.contains("&self.timeline_bar_pipeline"));

    for forbidden in [
        "TimelineHit::",
        "TimelineCandidate",
        "download_rgba",
        "read_buffer",
        "map_async",
    ] {
        assert!(
            !publish.contains(forbidden),
            "Timeline publish arm crossed a stopped boundary: {forbidden}"
        );
    }
}
