//! M3E-4 / GR-2: GpuCtx起源タグと同期読み戻し拒否の審判。

use motolii_core::{ColorSpace, FrameDesc, PixelFormat};
use motolii_gpu::{
    download_rgba, upload_rgba, GpuCtx, GpuOrigin, GpuRuntimeError, DEFAULT_DOWNLOAD_TIMEOUT,
};

fn headless_gpu() -> Option<GpuCtx> {
    match GpuCtx::new_headless() {
        Ok(gpu) => Some(gpu),
        Err(e) => {
            eprintln!("GPU unavailable, skipping: {e}");
            None
        }
    }
}

#[test]
fn new_headless_tags_origin_headless() {
    let Some(gpu) = headless_gpu() else {
        return;
    };
    assert_eq!(gpu.origin(), GpuOrigin::Headless);
}

#[test]
fn new_for_ui_tags_origin_ui_shared() {
    let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
        eprintln!("GPU unavailable, skipping new_for_ui origin tag test");
        return;
    };
    assert_eq!(gpu.origin(), GpuOrigin::UiShared);
}

#[test]
fn headless_allows_download_rgba() {
    let Some(gpu) = headless_gpu() else {
        return;
    };
    let desc = FrameDesc::packed(2, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let data = vec![255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255];
    let texture = upload_rgba(&gpu, &desc, &data);
    let out = download_rgba(&gpu, &texture).expect("headless download should succeed");
    assert_eq!(out, data);
}

#[test]
fn ui_shared_rejects_download_rgba() {
    let Some(headless) = headless_gpu() else {
        return;
    };
    let desc = FrameDesc::packed(1, 1, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let texture = upload_rgba(&headless, &desc, &[0, 0, 0, 255]);

    let ui_shared = GpuCtx::from_device_queue_with_origin(
        headless.device.clone(),
        headless.queue.clone(),
        GpuOrigin::UiShared,
    );
    let err = download_rgba(&ui_shared, &texture).unwrap_err();
    assert!(matches!(err, GpuRuntimeError::SyncReadbackForbidden));
}

#[test]
fn ui_shared_rejects_poll_wait() {
    let Some(headless) = headless_gpu() else {
        return;
    };
    let ui_shared = GpuCtx::from_device_queue_with_origin(
        headless.device.clone(),
        headless.queue.clone(),
        GpuOrigin::UiShared,
    );
    let err = ui_shared
        .poll_wait(Some(DEFAULT_DOWNLOAD_TIMEOUT))
        .unwrap_err();
    assert!(matches!(err, GpuRuntimeError::SyncReadbackForbidden));
}

#[test]
fn headless_allows_poll_wait() {
    let Some(gpu) = headless_gpu() else {
        return;
    };
    gpu.poll_wait(Some(std::time::Duration::from_millis(1)))
        .expect("headless poll_wait should succeed");
}
