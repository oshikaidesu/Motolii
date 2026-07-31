use std::fs;

#[test]
fn product_stage_reprojects_the_published_snapshot_through_the_existing_gpu_slot() {
    let runtime = fs::read_to_string("src/product_runtime.rs").expect("product runtime");

    assert!(runtime.contains("let mut render_worker = RenderWorker::spawn(Arc::clone(&gpu))?;"));
    assert!(runtime.contains("self.submit_stage_projection()"));
    assert!(runtime.contains("Arc::clone(&self.current_document)"));
    assert!(runtime.contains("self.render_client.latest_accepted_generation()"));
    assert!(runtime.contains("self.preview.slot().copy(&self.gpu, &rendered.frame)?;"));
    assert!(runtime.contains("self.displayed_camera = rendered.camera;"));
    assert!(runtime.contains("render_worker.close();"));
    assert!(runtime.contains("render_worker.join()"));

    for forbidden in ["download_rgba", "read_buffer", "map_async"] {
        assert!(
            !runtime.contains(forbidden),
            "product Stage projection must remain GPU-resident: {forbidden}"
        );
    }
}

#[test]
fn product_inspector_preview_routes_through_bounded_wake_before_result_drain() {
    let runtime = fs::read_to_string("src/product_runtime.rs").expect("product runtime");
    let inspector_wake = runtime
        .find("fn process_inspector_gestures")
        .expect("inspector gesture drain");
    let drain_stage = runtime
        .find("fn drain_stage_projection")
        .expect("stage result drain");
    let handle_wake = runtime
        .find("ProductEvent::Wake => {")
        .expect("product wake handler");
    let commit_call = runtime[handle_wake..]
        .find("self.process_pending_inspector_commit(")
        .expect("pending inspector commit on wake");
    let process_call = runtime[handle_wake..]
        .find("self.process_inspector_gestures()")
        .expect("inspector gestures on wake");
    let drain_call = runtime[handle_wake..]
        .find("self.drain_stage_projection()")
        .expect("stage drain on wake");
    assert!(
        inspector_wake < drain_stage,
        "inspector gesture processing must be declared before stage result drain"
    );
    assert!(
        commit_call < process_call,
        "pending inspector commit must drain before gesture inbox on Wake"
    );
    assert!(
        process_call < drain_call,
        "inspector gestures must drain before completed render results on Wake"
    );
    for required in [
        "register_wake",
        "submit_preview",
        "submit_inspector_baseline",
        "pending_inspector_commit",
        "pending_inspector_commit: Option<InspectorGestureTerminal>",
        "take_pending_inspector_commit",
        "self.take_pending_inspector_commit()",
        "process_pending_inspector_commit",
        "into_set_effect_param_request",
        "push_set_effect_param",
        "inspector-opacity",
        "resolve_effect_param_preview_command",
        "InspectorGestureTerminalCause::Cancel",
    ] {
        assert!(
            runtime.contains(required),
            "product inspector preview route must retain {required}"
        );
    }
    assert!(
        !runtime.contains("PendingInspectorCommit"),
        "B2 must preserve the complete Inspector terminal for C, not a resolved snapshot command"
    );
}
