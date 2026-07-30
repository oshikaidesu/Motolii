//! CU-110PT: published snapshotを既存headless projectionからnative Timeline sceneへ描く。

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
    assert!(source.contains("native_timeline_renderer"));
    assert!(source.contains("&timeline_projection.projection"));
    assert!(source.contains("native_timeline_renderer.composite"));

    let renderer = include_str!("../src/native_timeline_renderer.rs");
    assert!(renderer.contains("for bar in projection.bars()"));
    assert!(renderer.contains("for key in projection.keys()"));
    assert!(renderer.contains("document.layers.display_name"));
    assert!(renderer.contains("use_cpu: false"));

    for forbidden in [
        "TimelineHit::",
        "TimelineCandidate",
        "download_rgba",
        "read_buffer",
        "map_async",
        "TimelineToolsHostRuntime::new",
    ] {
        assert!(
            !publish.contains(forbidden),
            "Timeline publish arm crossed a stopped boundary: {forbidden}"
        );
    }
}
