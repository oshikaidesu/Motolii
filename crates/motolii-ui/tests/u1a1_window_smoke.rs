//! 実windowを利用できる開発環境だけでresize/minimize/restoreを通す。

use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use motolii_gpu::GpuCtx;
use motolii_testkit::{interactive_window_or_skip, unavailable_dep};

static WINDOW_SMOKE_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn product_window_resizes_minimizes_and_restores() {
    let _window_smoke_guard = WINDOW_SMOKE_LOCK.lock().expect("window smoke lock");
    if !interactive_window_or_skip(
        !linux_display_missing(),
        "DISPLAY and WAYLAND_DISPLAY are both unset",
    ) {
        return;
    }
    if GpuCtx::new_for_ui().is_err() {
        unavailable_dep("GPU adapter", "new_for_ui failed");
        return;
    }

    let executable = PathBuf::from(env!("CARGO_BIN_EXE_motolii_ui_shell"));
    let output = Command::new(executable)
        .env("MOTOLII_TEST_U1A1_LIFECYCLE", "1")
        .output()
        .expect("launch product shell");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "shell failed: {log}");
    for marker in [
        "U1A1_REGISTER",
        "U1A1_LIFECYCLE resize",
        "U1A1_LIFECYCLE minimize",
        "U1A1_LIFECYCLE restore",
        "U1A1_LIFECYCLE passed",
        "U1A2_LAYOUT",
    ] {
        assert!(log.contains(marker), "missing {marker}: {log}");
    }
    let register = stable_identity_fields(line_with(&log, "U1A1_REGISTER"));
    let lifecycle_copy_count =
        field_with_prefix(line_with(&log, "U1A1_LIFECYCLE resize"), "copies=");
    for marker in [
        "U1A1_LIFECYCLE resize",
        "U1A1_LIFECYCLE minimize",
        "U1A1_LIFECYCLE restore",
        "U1A1_LIFECYCLE passed",
    ] {
        assert_eq!(
            stable_identity_fields(line_with(&log, marker)),
            register,
            "{marker} changed slot, TextureId, registration, copy, or render identity: {log}"
        );
        assert_eq!(
            field_with_prefix(line_with(&log, marker), "copies="),
            lifecycle_copy_count,
            "{marker} changed copy count without a new render result: {log}"
        );
    }
    assert!(
        line_with(&log, "U1A1_LIFECYCLE passed").contains("paint_count="),
        "restore did not record a subsequent paint: {log}"
    );
    let layout = line_with(&log, "U1A2_LAYOUT");
    for surface in [
        "H[1:P:Browser,3:P:Stage,1:P:Inspector]",
        "1:P:Timeline",
        "status=Status",
    ] {
        assert!(
            layout.contains(surface),
            "five-surface layout evidence missing {surface}: {log}"
        );
    }
}

#[test]
fn latest_worker_result_reaches_the_product_event_loop() {
    let _window_smoke_guard = WINDOW_SMOKE_LOCK.lock().expect("window smoke lock");
    if !interactive_window_or_skip(
        !linux_display_missing(),
        "DISPLAY and WAYLAND_DISPLAY are both unset",
    ) {
        return;
    }
    if GpuCtx::new_for_ui().is_err() {
        unavailable_dep("GPU adapter", "new_for_ui failed");
        return;
    }

    let executable = PathBuf::from(env!("CARGO_BIN_EXE_motolii_ui_shell"));
    let output = Command::new(executable)
        .env("MOTOLII_TEST_U1B2_LATEST", "1")
        .output()
        .expect("launch product shell");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "shell failed: {log}");
    for marker in [
        "U1A1_REGISTER",
        "U1B2_LATEST passed",
        "U1B2_JOIN passed after_run_native=true",
    ] {
        assert!(log.contains(marker), "missing {marker}: {log}");
    }
    let latest = line_with(&log, "U1B2_LATEST passed");
    assert!(latest.contains("registrations=1"), "{log}");
    assert!(latest.contains("copies=2"), "{log}");
    assert!(latest.contains("generation=1"), "{log}");
}

fn line_with<'a>(log: &'a str, marker: &str) -> &'a str {
    log.lines()
        .find(|line| line.contains(marker))
        .unwrap_or_else(|| panic!("missing {marker}: {log}"))
}

fn stable_identity_fields(line: &str) -> Vec<&str> {
    line.split_whitespace()
        .filter(|field| {
            ["slot=", "texture=", "registrations=", "renders="]
                .iter()
                .any(|prefix| field.starts_with(prefix))
        })
        .collect()
}

fn field_with_prefix<'a>(line: &'a str, prefix: &str) -> &'a str {
    line.split_whitespace()
        .find(|field| field.starts_with(prefix))
        .unwrap_or_else(|| panic!("missing {prefix}: {line}"))
}

#[cfg(target_os = "linux")]
fn linux_display_missing() -> bool {
    std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
}

#[cfg(not(target_os = "linux"))]
fn linux_display_missing() -> bool {
    false
}
