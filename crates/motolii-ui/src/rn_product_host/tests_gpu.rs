//! Stage GPU binding の試験。helper は tests。
use super::tests::*;
use super::*;

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_binding_starts_detached_with_zero_epoch() {
    let binding = StageGpuBinding::detached();
    assert_eq!(binding.surface_epoch, 0);
    assert!(!binding.is_attached());
    assert_eq!(binding.physical_width, 0);
    assert_eq!(binding.physical_height, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_detach_increments_epoch_and_clears_binding_markers() {
    let mut stage = RnStageSurface {
        host_handle: 1,
        mounted: true,
        destroyed: false,
        width: 100,
        height: 50,
        scale_factor: 2.0,
        focused: false,
        pointer: None,
        gpu: StageGpuBinding {
            surface_epoch: 2,
            last_presented_epoch: Some(2),
            physical_width: 200,
            physical_height: 100,
            layer_ptr: 0xdead_beef,
            surface: None,
            needs_reconfigure: false,
            poisoned: false,
            overlay: None,
            overlay_upload_key: None,
        },
    };
    assert!(stage.gpu.is_attached());
    stage.gpu_detach_surface();
    assert!(!stage.gpu.is_attached());
    assert_eq!(stage.gpu.surface_epoch, 3);
    assert_eq!(stage.gpu.physical_width, 0);
    assert_eq!(stage.gpu.physical_height, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_resize_unknown_stage_is_rejected_without_registry_mutation() {
    let _lock = test_lock();
    let host = create_host("gpu-unknown-stage");
    let before = read_snapshot(host);
    let outcome = run_stage_gpu_op(host, 99_999, |_, _| Ok(()));
    assert_eq!(outcome, Err(RnHostReasonCode::UnknownStageHandle));
    let after = read_snapshot(host);
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.projection_generation, before.projection_generation);
    let _ = host_destroy_for_test(host);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_attach_validation_rejects_null_layer_without_state_change() {
    let binding = StageGpuBinding::detached();
    assert_eq!(
        binding.validate_attach(0),
        Err(RnHostReasonCode::InvalidIntent)
    );
    assert_eq!(binding.surface_epoch, 0);
    assert!(!binding.is_attached());
    assert!(!binding.has_surface());
    assert!(!binding.needs_reconfigure);
    assert!(!binding.poisoned);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_unmount_detaches_gpu_binding_markers_without_real_layer() {
    let _lock = test_lock();
    let host = create_host("gpu-unmount-detach");
    let stage = host_register_stage_for_test(host).expect("stage");
    stage_gpu_mark_attached_for_test(stage, 0xfeed_face).expect("mark attached");
    let (epoch, attached, _, _) = stage_gpu_state_for_test(stage).expect("state");
    assert!(attached);
    assert_eq!(epoch, 1);

    let mut mount = base_intent("stage_mount");
    mount.stage_handle = Some(stage);
    assert!(dispatch(host, mount).accepted);

    let mut unmount = base_intent("stage_unmount");
    unmount.stage_handle = Some(stage);
    assert!(dispatch(host, unmount).accepted);

    let (epoch_after, attached_after, width, height) =
        stage_gpu_state_for_test(stage).expect("state");
    assert!(!attached_after);
    assert_eq!(epoch_after, epoch + 1);
    assert_eq!(width, 0);
    assert_eq!(height, 0);

    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_duplicate_attach_is_rejected_without_replacing_binding() {
    let mut binding = StageGpuBinding::detached();
    binding.layer_ptr = 0xfeed_face;
    binding.surface_epoch = 4;
    assert_eq!(
        binding.validate_attach(0xdead_beef),
        Err(RnHostReasonCode::InvalidIntent)
    );
    assert_eq!(binding.layer_ptr, 0xfeed_face);
    assert_eq!(binding.surface_epoch, 4);
    assert!(!binding.has_surface());
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_surface_state_transitions_are_epoch_bounded() {
    let mut binding = StageGpuBinding::detached();
    binding.layer_ptr = 1;
    binding.needs_reconfigure = true;

    binding.configured(640, 360);
    assert_eq!(binding.surface_epoch, 1);
    assert!(!binding.needs_reconfigure);
    assert_eq!(binding.last_presented_epoch, None);

    binding.presented(false);
    assert_eq!(binding.last_presented_epoch, Some(1));
    assert!(!binding.needs_reconfigure);

    binding.acquisition_deferred();
    assert_eq!(binding.surface_epoch, 1);
    assert_eq!(binding.last_presented_epoch, Some(1));
    assert!(!binding.needs_reconfigure);

    binding.presented(true);
    assert_eq!(binding.last_presented_epoch, Some(1));
    assert!(binding.needs_reconfigure);
    binding.configured(640, 360);
    assert_eq!(binding.surface_epoch, 2);
    assert!(!binding.needs_reconfigure);

    binding.outdated();
    assert!(binding.needs_reconfigure);
    binding.configured(640, 360);
    assert_eq!(binding.surface_epoch, 3);
    assert!(!binding.needs_reconfigure);

    binding.lost();
    assert_eq!(binding.surface_epoch, 4);
    assert!(!binding.is_attached());
    assert!(!binding.has_surface());
    assert!(!binding.needs_reconfigure);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_size_change_invalidates_overlay_upload() {
    let mut binding = StageGpuBinding::detached();
    binding.configured(640, 360);
    let key = OverlayUploadKey {
        selected: Some(LayerId::from_raw(1)),
        projection_generation: 2,
    };
    binding.overlay_upload_key = Some(key);

    binding.configured(640, 360);
    assert_eq!(binding.overlay_upload_key, Some(key));

    binding.configured(1280, 720);
    assert_eq!(binding.overlay_upload_key, None);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_validation_poison_recovers_only_through_detach() {
    let mut stage = RnStageSurface {
        host_handle: 1,
        mounted: true,
        destroyed: false,
        width: 100,
        height: 50,
        scale_factor: 2.0,
        focused: false,
        pointer: None,
        gpu: StageGpuBinding {
            surface_epoch: 7,
            last_presented_epoch: Some(7),
            physical_width: 200,
            physical_height: 100,
            layer_ptr: 1,
            surface: None,
            needs_reconfigure: false,
            poisoned: false,
            overlay: None,
            overlay_upload_key: None,
        },
    };

    stage.gpu.validation_failed();
    assert_eq!(stage.gpu.surface_epoch, 8);
    assert!(stage.gpu.poisoned);
    // draw／resizeは同じpoison gateを通り、attachは重複bindingも併せて拒否する。
    assert_eq!(
        stage.gpu.reject_if_poisoned(),
        Err(RnHostReasonCode::InvalidIntent)
    );
    assert_eq!(
        stage.gpu.validate_attach(2),
        Err(RnHostReasonCode::InvalidIntent)
    );

    stage.gpu_detach_surface();
    assert_eq!(stage.gpu.surface_epoch, 9);
    assert!(!stage.gpu.is_attached());
    assert!(!stage.gpu.has_surface());
    assert!(!stage.gpu.poisoned);
    assert_eq!(stage.gpu.last_presented_epoch, None);
    assert_eq!(stage.gpu.physical_width, 0);
    assert_eq!(stage.gpu.physical_height, 0);
    assert_eq!(stage.gpu.validate_attach(2), Ok(()));
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_host_stage_pair_mismatch_is_rejected_without_snapshot_write() {
    let _lock = test_lock();
    let host = create_host("gpu-pair-mismatch");
    let stage = host_register_stage_for_test(host).expect("stage");
    let before = read_snapshot(host);
    let outcome = run_stage_gpu_op(host + 100, stage, |_, _| Ok(()));
    assert_eq!(outcome, Err(RnHostReasonCode::UnknownHostHandle));
    let after = read_snapshot(host);
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.projection_generation, before.projection_generation);
    let _ = host_destroy_stage_for_test(stage);
    let _ = host_destroy_for_test(host);
}

#[cfg(target_os = "macos")]
#[test]
fn stage_gpu_abi_rejects_off_main_before_zero_handle_validation() {
    let (written, output) = std::thread::spawn(|| {
        let mut output = vec![0_u8; MAX_JSON_BYTES];
        let written = motolii_rn_stage_draw(0, 0, output.as_mut_ptr(), output.len());
        (written, output)
    })
    .join()
    .expect("off-main gpu call");
    assert!(written > 0);
    let response = parse_wire_response(&output, written);
    assert!(!response.accepted);
    assert_eq!(
        response.diagnostics[0].reason,
        RnHostReasonCode::InvalidIntent
    );
}
