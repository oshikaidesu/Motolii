//! PV-1 振る舞い試験 — カウンタと状態遷移で禁止構造を審判する。
//! ソース文字列contains検査は行わない。

use motolii_gpu::GpuCtx;
use motolii_testkit::unavailable_dep;
use pv1_texture_lifecycle::{
    LifecycleEngine, LifecycleError, LifecycleEvent, LifecycleState, Pv1Manifest, Verdict,
    DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

fn gpu_for_ui_test() -> Option<GpuCtx> {
    match GpuCtx::new_for_ui() {
        Ok((gpu, _parts)) => Some(gpu),
        Err(e) => {
            unavailable_dep("GPU adapter", &e.to_string());
            None
        }
    }
}

fn bind_and_display(engine: &mut LifecycleEngine) {
    let generation = engine.texture_generation().unwrap();
    let _img = engine.bind_display_image().unwrap();
    engine.record_display_bound(generation).unwrap();
}

#[test]
fn skeleton_manifest_all_pending() {
    let m = Pv1Manifest::skeleton_template();
    assert_eq!(m.overall, Verdict::Pending);
    assert_eq!(m.ticket, "PV-1");
    for entry in &m.human_checks {
        assert_eq!(entry.verdict, Verdict::Pending);
    }
    for backend in &m.backends {
        assert_eq!(backend.verdict, Verdict::Pending);
    }
    assert_eq!(m.automation.lifecycle_behavior_tests, Verdict::Pending);
    assert_eq!(m.automation.release_build, Verdict::Pending);
}

#[test]
fn new_for_ui_and_image_try_from() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    assert_eq!(engine.counters().texture_create_count, 1);
    assert_eq!(engine.counters().image_try_from_count, 1);
    assert_eq!(engine.counters().ui_property_set_count, 1);
    assert_eq!(engine.state(), LifecycleState::Displaying);
}

#[test]
fn content_tick_does_not_recreate_texture() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let create_before = engine.counters().texture_create_count;
    let n = 8;
    for _ in 0..n {
        engine
            .apply_event(&gpu, LifecycleEvent::ContentTick)
            .unwrap();
    }
    assert_eq!(engine.counters().texture_create_count, create_before);
    // 初期 Clear 1 回 + ContentTick n 回
    assert_eq!(engine.counters().content_update_count, n + 1);
}

#[test]
fn resize_increments_texture_create_once() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let before = engine.counters().texture_create_count;
    let outcome = engine
        .apply_event(
            &gpu,
            LifecycleEvent::Resize {
                width: 480,
                height: 270,
            },
        )
        .unwrap();
    assert!(outcome.needs_image_rebind);
    assert_eq!(engine.counters().texture_create_count, before + 1);
    assert_eq!(engine.dimensions(), Some((480, 270)));
    assert_eq!(engine.texture_generation(), Some(2));
}

#[test]
fn hide_show_and_minimize_restore_do_not_recreate() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let create_before = engine.counters().texture_create_count;
    let gen_before = engine.texture_generation();

    engine.apply_event(&gpu, LifecycleEvent::Hide).unwrap();
    assert_eq!(engine.state(), LifecycleState::Hidden);
    engine.apply_event(&gpu, LifecycleEvent::Show).unwrap();
    assert_eq!(engine.state(), LifecycleState::Displaying);

    engine.apply_event(&gpu, LifecycleEvent::Minimize).unwrap();
    assert_eq!(engine.state(), LifecycleState::Minimized);
    engine.apply_event(&gpu, LifecycleEvent::Restore).unwrap();
    assert_eq!(engine.state(), LifecycleState::Restored);

    assert_eq!(engine.counters().texture_create_count, create_before);
    assert_eq!(engine.texture_generation(), gen_before);
}

#[test]
fn invalid_resize_does_not_mutate_counters_or_generation() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let counters_before = engine.counters();
    let gen_before = engine.texture_generation();
    let dims_before = engine.dimensions();
    let state_before = engine.state();

    let err = engine
        .apply_event(
            &gpu,
            LifecycleEvent::Resize {
                width: 0,
                height: 0,
            },
        )
        .unwrap_err();
    assert!(matches!(err, LifecycleError::InvalidResize { .. }));

    assert_eq!(engine.counters(), counters_before);
    assert_eq!(engine.texture_generation(), gen_before);
    assert_eq!(engine.dimensions(), dims_before);
    assert_eq!(engine.state(), state_before);
    assert!(engine.dimensions().is_some());
    assert!(engine.texture_generation().is_some());
}

#[test]
fn failed_state_rejects_further_events() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    let counters_before = engine.counters();
    engine.mark_failed();
    assert_eq!(engine.state(), LifecycleState::Failed);
    assert!(engine
        .apply_event(&gpu, LifecycleEvent::ContentTick)
        .is_err());
    assert_eq!(engine.counters(), counters_before);
}

#[test]
fn regenerate_increments_create_and_generation() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let before = engine.counters().texture_create_count;
    let gen_before = engine.texture_generation().unwrap();
    let outcome = engine
        .apply_event(&gpu, LifecycleEvent::Regenerate)
        .unwrap();
    assert!(outcome.needs_image_rebind);
    assert_eq!(engine.counters().texture_create_count, before + 1);
    assert_eq!(engine.texture_generation(), Some(gen_before + 1));
}

#[test]
fn pipeline_and_shader_counters_stay_zero() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    for _ in 0..3 {
        engine
            .apply_event(&gpu, LifecycleEvent::ContentTick)
            .unwrap();
    }
    engine
        .apply_event(
            &gpu,
            LifecycleEvent::Resize {
                width: 512,
                height: 288,
            },
        )
        .unwrap();
    assert_eq!(engine.counters().pipeline_create_count, 0);
    assert_eq!(engine.counters().shader_module_create_count, 0);
}

#[test]
fn manifest_overall_never_auto_passes_from_skeleton() {
    let m = Pv1Manifest::skeleton_template();
    assert_ne!(m.overall, Verdict::Pass);
    assert_eq!(m.overall, Verdict::Pending);
}

#[test]
fn stale_record_display_bound_does_not_advance_counters_or_state() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let counters_before = engine.counters();
    let state_before = engine.state();

    engine.record_display_bound(0).unwrap();

    assert_eq!(engine.counters(), counters_before);
    assert_eq!(engine.state(), state_before);
}

#[test]
fn record_display_bound_advances_state_for_current_generation() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    let generation = engine.texture_generation().unwrap();
    let _ = engine.bind_display_image().unwrap();
    assert_eq!(engine.state(), LifecycleState::Ready);
    engine.record_display_bound(generation).unwrap();
    assert_eq!(engine.state(), LifecycleState::Displaying);
    assert_eq!(engine.counters().ui_property_set_count, 1);
    assert_eq!(engine.counters().image_try_from_count, 1);
}

#[test]
fn display_bind_failed_marks_failed_for_current_generation() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let generation = engine.texture_generation().unwrap();
    let counters_before = engine.counters();

    engine.record_display_bind_failed(generation);
    assert_eq!(engine.state(), LifecycleState::Failed);
    assert_eq!(engine.counters(), counters_before);
}

#[test]
fn display_bind_failed_stale_generation_is_no_op() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let state_before = engine.state();

    engine.record_display_bind_failed(0);
    assert_eq!(engine.state(), state_before);
}

#[test]
fn resize_failure_retains_texture_and_stays_recoverable() {
    let Some(gpu) = gpu_for_ui_test() else {
        return;
    };
    let mut engine = LifecycleEngine::new(&gpu, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
    engine.boot_to_ready();
    bind_and_display(&mut engine);
    let state_before = engine.state();

    let err = engine
        .apply_event(
            &gpu,
            LifecycleEvent::Resize {
                width: 0,
                height: 360,
            },
        )
        .unwrap_err();
    assert!(matches!(err, LifecycleError::InvalidResize { .. }));
    assert!(engine.dimensions().is_some());
    assert_ne!(engine.state(), LifecycleState::Failed);
    assert_eq!(engine.state(), state_before);
}
