//! U1a-1 / GR-UI-3: UI thread 規約と register-once 不変条件。

use std::path::PathBuf;
use std::process::Command;

use motolii_gpu::GpuCtx;
use motolii_testkit::unavailable_dep;

#[test]
fn shared_device_setup_does_not_use_sync_readback() {
    let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
        unavailable_dep("GPU adapter", "new_for_ui failed");
        return;
    };
    assert!(
        gpu.poll_wait(None).is_err(),
        "UiShared device must forbid sync readback on preview path"
    );
}

#[test]
fn visible_shell_can_launch_and_auto_close() {
    if GpuCtx::new_for_ui().is_err() {
        unavailable_dep("GPU adapter", "new_for_ui failed");
        return;
    }

    if linux_display_missing() {
        unavailable_dep("display/window", "DISPLAY and WAYLAND_DISPLAY unset");
        return;
    }

    let exe = PathBuf::from(env!("CARGO_BIN_EXE_motolii_ui_shell"));
    let output = Command::new(&exe)
        .args(["--auto-close-after-frames", "2"])
        .output()
        .unwrap_or_else(|err| panic!("spawn motolii_ui_shell: {err}"));

    if !output.status.success() {
        let detail = child_output(&output);
        panic!(
            "motolii_ui_shell failed with {:?}: {detail}",
            output.status.code()
        );
    }
}

fn child_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{stdout}{stderr}")
}

#[cfg(target_os = "linux")]
fn linux_display_missing() -> bool {
    std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
}

#[cfg(not(target_os = "linux"))]
fn linux_display_missing() -> bool {
    false
}
