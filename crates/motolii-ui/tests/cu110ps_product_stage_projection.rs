use std::fs;

#[test]
fn product_stage_reprojects_the_published_snapshot_through_the_existing_gpu_slot() {
    let runtime = fs::read_to_string("src/product_runtime.rs").expect("product runtime");

    assert!(runtime.contains("let mut render_worker = RenderWorker::spawn(Arc::clone(&gpu))?;"));
    assert!(runtime.contains("self.submit_stage_projection()"));
    assert!(runtime.contains("document: Arc::clone(&self.current_document)"));
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
