use super::lifecycle::ensure_project_document;
use super::parse_wire::parse_timeline_projection;
use super::slot::{host_slot, slice_from_written, HostSlot, MAX_SNAPSHOT_JSON_BYTES};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

#[cfg(target_os = "macos")]
use motolii_ui::{motolii_rn_host_dispatch_intent_json, motolii_rn_host_read_snapshot_json};

use super::types::HostTimelineProjection;

mod catalog;
mod dispatch;
mod keymap;
mod lifecycle;
mod parse_geometry;
mod parse_projection;
mod place;
mod stage;

static TEST_LOCK: Mutex<()> = Mutex::new(());
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock().expect("host bridge test lock")
}

fn temp_project(tag: &str) -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("motolii-rnapp-host-{tag}-{id}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("project.json");
    assert!(ensure_project_document(&path));
    path
}

fn bounds_from_snapshot(
    snap: &motolii_ui::RnProductSnapshotForTest,
) -> Vec<crate::timeline_skia::SnapshotLayerInput> {
    use crate::timeline_skia::{SnapshotKeyInput, SnapshotLayerInput};
    snap.timeline
        .layers
        .iter()
        .map(|layer| SnapshotLayerInput {
            layer_id: layer.layer_id.clone(),
            display_name: layer.display_name.clone(),
            interval_secs: Some((
                layer.start.as_seconds_f64(),
                layer.duration.as_seconds_f64(),
            )),
            keys: layer
                .position_keys
                .iter()
                .filter_map(|key| {
                    Some(SnapshotKeyInput {
                        key_id: key.key_id.parse().ok()?,
                        time_secs: key.time.as_seconds_f64(),
                    })
                })
                .collect(),
        })
        .collect()
}

fn dispatch_kind(host: u64, kind: &str, extra: &str) {
    let intent = format!(
        r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{host}"{extra}}}"#
    );
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = unsafe {
        motolii_rn_host_dispatch_intent_json(
            host,
            intent.as_ptr(),
            intent.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };
    assert!(written > 0, "dispatch {kind} failed: {written}");
    let response_bytes =
        slice_from_written(&out, written).expect("dispatch response within buffer");
    let response = std::str::from_utf8(response_bytes).expect("utf8");
    assert!(
        response.contains(r#""accepted":true"#),
        "expected accepted: {response}"
    );
}

fn install_slot(host: u64) {
    let mut guard = host_slot().lock().expect("slot");
    *guard = Some(HostSlot {
        handle: host,
        project_path: PathBuf::new(),
        stage_handle: None,
        stage_pointer_active: false,
        stage_mounted: false,
        pointer_sequence: 0,
        stage_logical_width: 0.0,
        stage_logical_height: 0.0,
    });
}

fn clear_slot() {
    let mut guard = host_slot().lock().expect("slot");
    *guard = None;
}

fn read_host_projection(host: u64) -> HostTimelineProjection {
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written =
        unsafe { motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len()) };
    assert!(written > 0, "host snapshot json failed: {written}");
    let json_bytes =
        slice_from_written(&out, written).expect("snapshot response within buffer");
    let json = std::str::from_utf8(json_bytes).expect("snapshot json");
    parse_timeline_projection(json).expect("projection parse")
}
