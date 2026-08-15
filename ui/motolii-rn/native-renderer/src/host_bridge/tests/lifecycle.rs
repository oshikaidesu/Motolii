use super::super::terminal::inject_host_handle;

#[cfg(target_os = "macos")]
use super::{clear_slot, temp_project, test_lock};
#[cfg(target_os = "macos")]
use super::super::lifecycle::try_host_shutdown;
#[cfg(target_os = "macos")]
use super::super::slot::{host_slot, host_startup_reject, slice_from_written, try_host_handle};
#[cfg(target_os = "macos")]
use super::super::{
    motolii_rnapp_host_dispatch_json, motolii_rnapp_host_ensure, try_stage_mount, try_stage_unmount,
};
#[cfg(target_os = "macos")]
use motolii_ui::motolii_rn_host_read_snapshot_json;

#[test]
fn inject_host_handle_replaces_empty_value() {
    let patched = inject_host_handle(
        r#"{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":""}"#,
        42,
    )
    .expect("inject");
    assert!(patched.contains(r#""host_handle":"42""#));
}

#[test]
fn inject_host_handle_adds_missing_top_level_field() {
    let patched = inject_host_handle(
        r#"{"version":1,"direction":"rn-to-host","kind":"undo"}"#,
        42,
    )
    .expect("inject");
    assert_eq!(
        patched,
        r#"{"version":1,"direction":"rn-to-host","kind":"undo","host_handle":"42"}"#
    );
}

#[test]
fn inject_host_handle_does_not_rewrite_nested_only_field() {
    let patched = inject_host_handle(
        r#"{"version":1,"direction":"rn-to-host","kind":"undo","meta":{"host_handle":"nested"}}"#,
        42,
    )
    .expect("inject");
    assert!(patched.contains(r#""meta":{"host_handle":"nested"}"#));
    assert!(patched.ends_with(r#","host_handle":"42"}"#));
}

#[cfg(target_os = "macos")]
#[test]
fn dispatch_without_host_returns_the_typed_startup_reject() {
    let _lock = test_lock();
    clear_slot();
    let expected =
        r#"{"version":1,"accepted":false,"diagnostics":[{"reason":"project_already_open"}]}"#;
    *host_startup_reject().lock().expect("startup reject") = Some(expected.to_owned());
    let intent = br#"{"version":1,"direction":"rn-to-host","kind":"place_ellipse"}"#;
    let mut out = [0u8; 256];

    let written = unsafe {
        motolii_rnapp_host_dispatch_json(
            intent.as_ptr(),
            intent.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };

    assert!(written > 0);
    assert_eq!(&out[..written as usize], expected.as_bytes());
    *host_startup_reject().lock().expect("startup reject") = None;
}

#[cfg(target_os = "macos")]
#[test]
fn host_lifecycle_identity_reuses_same_path_and_tombstones_shutdown_handle() {
    let _lock = test_lock();
    clear_slot();
    *host_startup_reject().lock().expect("startup reject") = None;
    let first_path = temp_project("lifecycle-first");
    let first_bytes = first_path.to_string_lossy();

    assert!(unsafe { motolii_rnapp_host_ensure(first_bytes.as_ptr(), first_bytes.len()) });
    let first_host = try_host_handle().expect("first host");
    assert!(unsafe { motolii_rnapp_host_ensure(first_bytes.as_ptr(), first_bytes.len()) });
    assert_eq!(try_host_handle(), Some(first_host));

    assert!(try_stage_mount(1600.0, 900.0, 1.0));
    let first_stage = host_slot()
        .lock()
        .expect("slot")
        .as_ref()
        .and_then(|slot| slot.stage_handle)
        .expect("stage");
    assert!(try_stage_unmount());
    assert!(try_stage_mount(1600.0, 900.0, 1.0));
    {
        let remounted = host_slot().lock().expect("slot");
        let remounted = remounted.as_ref().expect("remounted slot");
        assert_eq!(remounted.handle, first_host);
        assert_eq!(remounted.stage_handle, Some(first_stage));
    }

    let second_path = temp_project("lifecycle-second");
    let second_bytes = second_path.to_string_lossy();
    assert!(!unsafe { motolii_rnapp_host_ensure(second_bytes.as_ptr(), second_bytes.len()) });
    assert_eq!(try_host_handle(), Some(first_host));

    assert!(try_host_shutdown());
    assert_eq!(try_host_handle(), None);
    assert!(try_host_shutdown(), "shutdown without a slot is idempotent");

    let mut out = [0u8; 1024];
    let written = motolii_rn_host_read_snapshot_json(first_host, out.as_mut_ptr(), out.len());
    let rejected = std::str::from_utf8(
        slice_from_written(&out, written).expect("destroyed host response"),
    )
    .expect("destroyed host utf8");
    assert!(rejected.contains(r#""accepted":false"#));
    assert!(rejected.contains(r#""reason":"destroyed_host_handle""#));

    assert!(unsafe { motolii_rnapp_host_ensure(second_bytes.as_ptr(), second_bytes.len()) });
    let second_host = try_host_handle().expect("second host");
    assert_ne!(second_host, first_host, "host handles stay monotonic");
    assert!(try_host_shutdown());
}

#[test]
fn inject_host_handle_rewrites_only_top_level_not_nested() {
    let patched = inject_host_handle(
        concat!(
            r#"{"version":1,"direction":"rn-to-host","kind":"set_effect_param","#,
            r#""host_handle":"","meta":{"host_handle":"nested"},"target":"1"}"#
        ),
        99,
    )
    .expect("inject");
    assert!(patched.contains(r#""host_handle":"99""#));
    assert!(patched.contains(r#""host_handle":"nested""#));
    // top-levelだけが99。nestedは維持。
    let top = patched.find(r#""host_handle":"99""#).expect("top");
    let nested = patched.find(r#""host_handle":"nested""#).expect("nested");
    assert!(top < nested);
}
