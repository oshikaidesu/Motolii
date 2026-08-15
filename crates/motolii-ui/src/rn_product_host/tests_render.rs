//! host_render_frame の試験。helper は tests。
use super::tests::*;
use super::*;

#[test]
fn host_render_frame_returns_texture_and_dirty_gates() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        // BLOCKED(GPU): sandboxにadapterが無い場合はsupervisorが実機で回す。
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let host = create_host("stage-frame-dirty");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let first = frame.as_ref().expect("frame");
    let draft = Quality::DRAFT
        .render_desc(frame_desc_from_composition(&Document::new_current()).expect("desc"));
    assert_eq!((first.width, first.height), (draft.width, draft.height));
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Unchanged
    );
    assert!(frame.is_some());

    let _ = host_destroy_for_test(host);
}

#[test]
fn host_render_frame_rerenders_when_caller_dropped_frame() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let host = create_host("stage-frame-dropped");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    assert!(frame.is_some());
    // 呼び手がframeを破棄(= renderer再生成後相当)。(rev,gen,time)一致でも再render。
    frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    assert!(frame.is_some());

    let _ = host_destroy_for_test(host);
}

#[test]
fn host_render_frame_rerenders_on_revision_and_time() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let host = create_host("stage-frame-rerender");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.1,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Unchanged
    );
    let rev1 = frame.as_ref().expect("f1").revision.clone();

    assert!(
        dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"{host}"}}"#
            ),
        )
        .accepted
    );
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let after_undo = frame.as_ref().expect("undo frame");
    assert_ne!(after_undo.revision, rev1);

    assert!(dispatch_raw_json(host, &set_time_json(host, "30")).accepted);
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let after_time = frame.as_ref().expect("time frame");
    assert_eq!(after_time.time, RationalTime::try_new(1, 1).expect("1/1"));
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Unchanged
    );

    let _ = host_destroy_for_test(host);
}

#[test]
fn host_render_frame_unknown_handle_is_false() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(9_999, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Failed
    );
    assert!(frame.is_none());
}

#[test]
fn host_render_frame_after_seed_place_has_non_uniform_pixels() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let host = create_host("stage-frame-readback");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);

    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let first = frame.take().expect("frame");
    let draft = Quality::DRAFT
        .render_desc(frame_desc_from_composition(&Document::new_current()).expect("desc"));
    assert_eq!((first.width, first.height), (draft.width, draft.height));

    let bytes = download_rgba(&gpu, &first.texture).expect("frame readback");
    assert_eq!(
        bytes.len(),
        (first.width as usize) * (first.height as usize) * 4
    );
    let center = pixel_at(&bytes, first.width, first.width / 2, first.height / 2);
    let background = pixel_at(&bytes, first.width, 0, 0);
    assert_ne!(center, background);
    assert!(has_non_background_pixel(
        &bytes,
        first.width,
        first.height,
        background
    ));

    let _ = host_destroy_for_test(host);
}

#[test]
fn host_render_frame_opacity_preview_changes_pixels() {
    let _lock = test_lock();
    let Some(gpu) = motolii_testkit::gpu_or_skip() else {
        return;
    };
    let mut session = RenderSession::new(&gpu);
    let host = create_host("stage-frame-opacity");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"place_rectangle","#,
                    r#""host_handle":"{host}","position":[0.0,0.0],"playhead":{{"num":0,"den":1}}}}"#
                ),
                host = host,
            ),
        )
        .accepted);
    let snapshot = read_snapshot(host);
    let layer_id = snapshot
        .primary_layer_id
        .clone()
        .or_else(|| snapshot.layer_ids.last().cloned())
        .expect("placed layer");
    assert!(dispatch_raw_json(
            host,
            &format!(
                concat!(
                    r#"{{"version":1,"direction":"rn-to-host","kind":"attach_effect","#,
                    r#""host_handle":"{host}","target":"{layer}","plugin_id":"core.filter.opacity"}}"#
                ),
                host = host,
                layer = layer_id,
            ),
        )
        .accepted);
    let effect_use_id = layer_effects(&read_wire(host), &layer_id)[0]
        .effect_use_id
        .clone();

    let mut frame = None;
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let opaque = frame.take().expect("opaque frame");
    let opaque_bytes = download_rgba(&gpu, &opaque.texture).expect("opaque readback");

    assert!(dispatch_raw_json(
            host,
            &format!(
                r#"{{"version":1,"direction":"rn-to-host","kind":"preview_effect_param","host_handle":"{host}","target":"{layer}","effect_use_id":"{effect}","param_id":"amount","value":0.25}}"#,
                host = host,
                layer = layer_id,
                effect = effect_use_id,
            ),
        )
        .accepted);
    assert_eq!(
        host_render_frame_for_app(host, &gpu, &mut session, &mut frame),
        HostRenderFrameResult::Rendered
    );
    let faded = frame.take().expect("faded frame");
    let faded_bytes = download_rgba(&gpu, &faded.texture).expect("faded readback");
    assert_eq!(opaque_bytes.len(), faded_bytes.len());
    assert_ne!(
        opaque_bytes, faded_bytes,
        "opacity preview must change the evaluated Stage frame"
    );

    let _ = host_destroy_for_test(host);
}
