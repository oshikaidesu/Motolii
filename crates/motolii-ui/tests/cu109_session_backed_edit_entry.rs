//! CU-109 session-backed explicit-path shell entry and Apply roundtrip.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    journal_path_for_document, layer_names_for_item, motolii_dir_for_document,
    project_lock_path_for_document, Clip, ClipSource, Command as DocCommand, DocParam, Document,
    ItemEnvelope, ParentLocator, ProjectSession, RecoverySource, ResourceLimits,
    SaveProjectOptions, SessionError, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_gpu::GpuCtx;
use motolii_testkit::{interactive_window_or_skip, tmp_dir, unavailable_dep};

static WINDOW_SMOKE_LOCK: Mutex<()> = Mutex::new(());

fn production_limits() -> ResourceLimits {
    ResourceLimits::production()
}

fn smoke_fixture_document() -> Document {
    let mut document = Document::new_current();
    let layer = document.layers.allocate("static-preview").unwrap();
    let track = document.track_ids.allocate("static-preview").unwrap();
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([4.0, 4.0])),
                    ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    let survivor = document.layers.allocate("edit-survivor").unwrap();
    let survivor_item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(survivor),
        start: RationalTime::ZERO,
        duration: document.composition.duration,
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                ("size".into(), DocParam::const_vec2([4.0, 4.0])),
                ("color".into(), DocParam::const_color([0.0, 0.0, 1.0, 1.0])),
            ]),
            extra: Default::default(),
        },
    });
    document.tracks[0].items.push(survivor_item);
    document.validate().unwrap();
    document
}

fn initialize_project(path: &Path, document: &Document) {
    let limits = production_limits();
    let mut session = ProjectSession::acquire(path, &limits).expect("acquire");
    session
        .save_with_journal(
            document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .expect("checkpoint save");
}

fn document_after_remove_first_track_item(initial: &Document) -> Document {
    let mut expected = initial.clone();
    let track = expected.tracks[0].id;
    let item = expected.tracks[0].items[0].clone();
    DocCommand::RemoveTrackItem {
        parent: ParentLocator::Track(track),
        index: 0,
        layer_names: layer_names_for_item(&expected, &item).unwrap(),
        item,
    }
    .apply(&mut expected)
    .unwrap();
    expected
}

fn shell_executable() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_motolii_ui_shell"))
}

const SHELL_WAIT_TIMEOUT: Duration = Duration::from_secs(60);

fn launch_shell(args: &[&str], envs: &[(&str, &str)]) -> (std::process::Output, String) {
    let mut command = Command::new(shell_executable());
    for (key, value) in envs {
        command.env(key, value);
    }
    command.args(args);
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch product shell");
    let deadline = Instant::now() + SHELL_WAIT_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("wait on motolii_ui_shell failed: {error}");
            }
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "motolii_ui_shell timed out after {:?} args={args:?} envs={envs:?}",
                SHELL_WAIT_TIMEOUT
            );
        }
        thread::sleep(Duration::from_millis(50));
    }
    let output = child.wait_with_output().expect("reap product shell");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output, log)
}

fn interactive_gpu_or_skip() -> bool {
    let _guard = WINDOW_SMOKE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !interactive_window_or_skip(
        !linux_display_missing(),
        "DISPLAY and WAYLAND_DISPLAY are both unset",
    ) {
        return false;
    }
    if GpuCtx::new_for_ui().is_err() {
        unavailable_dep("GPU adapter", "new_for_ui failed");
        return false;
    }
    true
}

#[test]
fn apply_roundtrip_through_session_backed_shell_entry() {
    if !interactive_gpu_or_skip() {
        return;
    }
    let dir = tmp_dir("cu109-p2");
    let path = dir.join("proj.json");
    let initial = smoke_fixture_document();
    initialize_project(&path, &initial);

    let (output, log) = launch_shell(
        &[path.to_str().expect("utf8 path")],
        &[("MOTOLII_TEST_U2B1_DOCUMENT", "1")],
    );
    assert!(output.status.success(), "shell failed: {log}");
    for marker in [
        "U1A1_REGISTER",
        "U2B1_DOCUMENT passed",
        "U1B2_JOIN passed after_run_native=true",
    ] {
        assert!(log.contains(marker), "missing {marker}: {log}");
    }
    let pass = log
        .lines()
        .find(|line| line.contains("U2B1_DOCUMENT passed"))
        .expect("pass line");
    for expected in ["registrations=1", "generation=2", "revisions=1"] {
        assert!(
            pass.split_whitespace().any(|field| field == expected),
            "missing exact {expected}: {log}"
        );
    }

    let limits = production_limits();
    let (_session, opened) = ProjectSession::open(&path, &limits).expect("reopen");
    let expected = document_after_remove_first_track_item(&initial);
    assert_ne!(opened.document, initial);
    assert_eq!(opened.document, expected);
    assert_ne!(opened.source, RecoverySource::MainFile);
}

#[test]
fn zero_argv_lifecycle_and_latest_smokes_still_pass() {
    if !interactive_gpu_or_skip() {
        return;
    }
    for (env_key, marker) in [
        ("MOTOLII_TEST_U1A1_LIFECYCLE", "U1A1_LIFECYCLE passed"),
        ("MOTOLII_TEST_U1B2_LATEST", "U1B2_LATEST passed"),
    ] {
        let (output, log) = launch_shell(&[], &[(env_key, "1")]);
        assert!(output.status.success(), "{env_key} failed: {log}");
        assert!(log.contains(marker), "missing {marker}: {log}");
    }
}

#[test]
fn main_reaches_run_shell_with_project() {
    let source = include_str!("../src/bin/motolii_ui_shell.rs");
    assert!(source.contains("run_shell_with_project"));
    let lib_source = include_str!("../src/shell.rs");
    assert!(lib_source.contains("pub fn run_shell_with_project"));
}

#[test]
fn library_never_reads_process_argv() {
    for (name, source) in [
        ("shell.rs", include_str!("../src/shell.rs")),
        ("app.rs", include_str!("../src/app.rs")),
        ("lib.rs", include_str!("../src/lib.rs")),
    ] {
        assert!(
            !source.contains("args_os") && !source.contains("env::args"),
            "{name} must not read argv"
        );
    }
}

#[test]
fn document_edit_smoke_env_is_presence_only_in_shell() {
    let source = include_str!("../src/shell.rs");
    assert!(source.contains("var_os(DOCUMENT_EDIT_SMOKE_ENV)"));
    assert!(!source.contains("var(DOCUMENT_EDIT_SMOKE_ENV)"));
}

#[test]
fn argv_usage_rejects_before_gpu() {
    let cases: &[&[&str]] = &[
        &["a.json", "b.json"],
        &["-x"],
        &["--project"],
        &["-"],
        &["--"],
    ];
    for args in cases {
        let (output, log) = launch_shell(args, &[]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "expected usage reject for {args:?}: {log}"
        );
        assert!(
            log.contains("MOTOLII_USAGE_REJECT"),
            "missing usage marker: {log}"
        );
    }
}

#[test]
fn missing_project_path_fails_without_sidecar() {
    let dir = tmp_dir("cu109-n9");
    let path = dir.join("missing.json");
    let (output, log) = launch_shell(&[path.to_str().unwrap()], &[]);
    assert!(!output.status.success());
    assert!(!path.exists());
    assert!(!motolii_dir_for_document(&path).exists());
    assert!(!log.contains("U1A1_REGISTER"), "{log}");
    assert!(!log.contains("U2B1_DOCUMENT passed"), "{log}");
}

#[test]
fn uninitialized_project_path_fails_without_journal() {
    let dir = tmp_dir("cu109-n10");
    let path = dir.join("empty.json");
    fs::write(&path, b"{}").unwrap();
    let main_before = fs::read(&path).unwrap();
    let (output, log) = launch_shell(&[path.to_str().unwrap()], &[]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&path).unwrap(), main_before);
    assert!(!motolii_dir_for_document(&path).exists());
    assert!(!journal_path_for_document(&path).exists());
    assert!(!log.contains("U1A1_REGISTER"), "{log}");
    assert!(!log.contains("U2B1_DOCUMENT passed"), "{log}");
}

#[test]
fn locked_project_rejects_second_open() {
    let dir = tmp_dir("cu109-n11");
    let path = dir.join("proj.json");
    initialize_project(&path, &smoke_fixture_document());
    let main_before = fs::read(&path).unwrap();
    let journal = journal_path_for_document(&path);
    let journal_before = fs::read(&journal).unwrap();
    let limits = production_limits();
    let _held = ProjectSession::open(&path, &limits).expect("hold lock");
    let (output, log) = launch_shell(&[path.to_str().unwrap()], &[]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&path).unwrap(), main_before);
    assert_eq!(fs::read(&journal).unwrap(), journal_before);
    assert!(!log.contains("U1A1_REGISTER"), "{log}");
    assert!(!log.contains("U2B1_DOCUMENT passed"), "{log}");
}

#[test]
fn corrupted_sidecar_family_rejects_open() {
    let dir = tmp_dir("cu109-n12");
    let path = dir.join("proj.json");
    initialize_project(&path, &smoke_fixture_document());
    let main_before = fs::read(&path).unwrap();
    let sidecar = motolii_dir_for_document(&path);
    let journal = sidecar.join("journal.wal");
    fs::write(&journal, b"not-a-valid-journal").unwrap();
    let journal_before = fs::read(&journal).unwrap();
    let (output, log) = launch_shell(&[path.to_str().unwrap()], &[]);
    assert!(!output.status.success());
    assert_eq!(fs::read(&path).unwrap(), main_before);
    assert_eq!(fs::read(&journal).unwrap(), journal_before);
    assert!(!log.contains("U1A1_REGISTER"), "{log}");
    assert!(!log.contains("U2B1_DOCUMENT passed"), "{log}");
}

#[test]
fn zero_argv_document_edit_flag_is_not_product_edit_path() {
    if !interactive_gpu_or_skip() {
        return;
    }
    let (output, log) = launch_shell(
        &[],
        &[
            ("MOTOLII_TEST_U2B1_DOCUMENT", "1"),
            ("MOTOLII_TEST_U1A1_LIFECYCLE", "1"),
        ],
    );
    assert!(output.status.success(), "{log}");
    assert!(log.contains("U1A1_LIFECYCLE passed"), "{log}");
    assert!(!log.contains("U2B1_DOCUMENT passed"), "{log}");
}

#[test]
fn project_lock_is_the_only_pre_edit_artifact_on_missing_open() {
    let dir = tmp_dir("cu109-lock-only");
    let path = dir.join("missing.json");
    let limits = production_limits();
    let err = ProjectSession::open(&path, &limits).unwrap_err();
    assert!(matches!(
        err,
        SessionError::Recovery(_) | SessionError::Io(_)
    ));
    let lock = project_lock_path_for_document(&path);
    if lock.exists() {
        let _ = fs::remove_file(lock);
    }
}

#[test]
fn argv_is_sole_product_path_carrier_in_ui_src() {
    for (name, source) in [
        ("app.rs", include_str!("../src/app.rs")),
        ("shell.rs", include_str!("../src/shell.rs")),
        ("lib.rs", include_str!("../src/lib.rs")),
        (
            "document_edit_runtime.rs",
            include_str!("../src/document_edit_runtime.rs"),
        ),
        (
            "static_preview.rs",
            include_str!("../src/static_preview.rs"),
        ),
    ] {
        for forbidden in [
            "current_dir(",
            "home_dir(",
            "recent-project",
            "default_project",
            "default-project",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} must not use {forbidden} as a project path source"
            );
        }
    }
}

#[test]
fn shell_smoke_request_is_consumed_once() {
    let source = include_str!("../src/app.rs");
    assert!(
        source.contains("smoke.request.take()"),
        "document edit smoke must take the prepared request so a second dispatch is impossible"
    );
}

#[cfg(target_os = "linux")]
fn linux_display_missing() -> bool {
    std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none()
}

#[cfg(not(target_os = "linux"))]
fn linux_display_missing() -> bool {
    false
}
