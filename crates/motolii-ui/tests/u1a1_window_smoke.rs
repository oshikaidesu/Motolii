//! 実windowを利用できる開発環境だけでresize/minimize/restoreを通す。

use std::path::PathBuf;
use std::process::Command;

use motolii_gpu::GpuCtx;
use motolii_testkit::{interactive_window_or_skip, unavailable_dep};

#[test]
fn product_window_resizes_minimizes_and_restores() {
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
    ] {
        assert!(log.contains(marker), "missing {marker}: {log}");
    }
    let register = identity_fields(line_with(&log, "U1A1_REGISTER"));
    for marker in [
        "U1A1_LIFECYCLE resize",
        "U1A1_LIFECYCLE minimize",
        "U1A1_LIFECYCLE restore",
        "U1A1_LIFECYCLE passed",
    ] {
        assert_eq!(
            identity_fields(line_with(&log, marker)),
            register,
            "{marker} changed slot, TextureId, registration, copy, or render identity: {log}"
        );
    }
    assert!(
        line_with(&log, "U1A1_LIFECYCLE passed").contains("paint_count="),
        "restore did not record a subsequent paint: {log}"
    );
}

fn line_with<'a>(log: &'a str, marker: &str) -> &'a str {
    log.lines()
        .find(|line| line.contains(marker))
        .unwrap_or_else(|| panic!("missing {marker}: {log}"))
}

fn identity_fields(line: &str) -> &str {
    let fields = line
        .split_once("slot=")
        .map(|(_, fields)| fields)
        .unwrap_or_else(|| panic!("identity fields missing: {line}"));
    fields
        .split_once(" paint_count=")
        .map_or(fields, |(identity, _)| identity)
}

#[cfg(target_os = "linux")]
fn linux_display_missing() -> bool {
    std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
}

#[cfg(not(target_os = "linux"))]
fn linux_display_missing() -> bool {
    false
}
