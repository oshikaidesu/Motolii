//! RN app ↔ RnProductHost 接続の単一owner。
//!
//! processに最大1 host。ObjC/RNは薄いcarrierとしてここへ委譲する。

use std::path::Path;
use std::sync::{Mutex, OnceLock};

const MAX_JSON_BYTES: usize = 16_384;
const MAX_SNAPSHOT_JSON_BYTES: usize = 65_536;

// extern importではなくRust経由で呼ぶ。externで宣言すると同一crate graph内でも
// motolii-uiの該当objectがarchiveから引かれず、appのlinkで未解決symbolになる(実測)。
#[cfg(target_os = "macos")]
use motolii_ui::{
    motolii_rn_host_create, motolii_rn_host_dispatch_intent_json,
    motolii_rn_host_read_snapshot_json,
};

struct HostSlot {
    handle: u64,
}

fn host_slot() -> &'static Mutex<Option<HostSlot>> {
    static SLOT: OnceLock<Mutex<Option<HostSlot>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Host投影。revision変化時だけTimelineへ適用する。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineProjection {
    pub revision: String,
    pub primary_layer_id: Option<String>,
    pub bounds: Vec<(String, String)>,
    /// wire `timeline` がある時だけ。欠落時は旧host互換fallback。
    pub timeline_layers: Option<Vec<HostTimelineLayer>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineLayer {
    pub layer_id: String,
    pub display_name: String,
    pub start_secs: f64,
    pub duration_secs: f64,
    pub position_keys: Vec<HostTimelineKey>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostTimelineKey {
    pub key_id: u64,
    pub time_secs: f64,
}

/// 欠落documentを開ける最小projectでseedする。
/// `Document::new_current()` だけだと place_rectangle が process_next で落ちるため、
/// host test fixture と同型の1 layer documentを置く。
fn ensure_project_document(path: &Path) -> bool {
    if path.exists() {
        return true;
    }
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    use motolii_doc::{
        Clip, ClipSource, DocParam, Document, ItemEnvelope, ProjectSession, ResourceLimits,
        SaveProjectOptions, Track, TrackItem, RECT_LAYER_SOURCE,
    };
    use std::collections::BTreeMap;

    let mut document = Document::new_current();
    let Ok(layer) = document.layers.allocate("seed-layer") else {
        return false;
    };
    let Ok(track) = document.track_ids.allocate("seed-track") else {
        return false;
    };
    let duration = document.composition.duration;
    // RationalTime / TimeMap を crate依存に足さず、公開field型のメソッドだけでZERO/identityを得る。
    let Ok(start) = duration.try_sub(duration) else {
        return false;
    };
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start,
            duration,
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                    ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    if document.validate().is_err() {
        return false;
    }
    let limits = ResourceLimits::production();
    let Ok(mut session) = ProjectSession::acquire(path, &limits) else {
        return false;
    };
    session
        .save_with_journal(
            &document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .is_ok()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_host_ensure(path_utf8: *const u8, path_len: usize) -> bool {
    if path_utf8.is_null() || path_len == 0 || path_len > 4096 {
        return false;
    }
    let Ok(path) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(path_utf8, path_len) })
    else {
        return false;
    };
    let Ok(mut guard) = host_slot().lock() else {
        return false;
    };
    if guard.is_some() {
        return true;
    }
    if !ensure_project_document(Path::new(path)) {
        return false;
    }
    let path_bytes = path.as_bytes();
    let mut host_handle = 0u64;
    let mut out = [0u8; MAX_JSON_BYTES];
    let written = unsafe {
        motolii_rn_host_create(
            path_bytes.as_ptr(),
            path_bytes.len(),
            &mut host_handle,
            out.as_mut_ptr(),
            out.len(),
        )
    };
    if written <= 0 || host_handle == 0 {
        return false;
    }
    *guard = Some(HostSlot {
        handle: host_handle,
    });
    true
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_host_dispatch_json(
    in_utf8: *const u8,
    in_len: usize,
    out: *mut u8,
    out_cap: usize,
) -> i64 {
    if in_utf8.is_null() || out.is_null() || out_cap == 0 || in_len > MAX_JSON_BYTES {
        return -1;
    }
    let Ok(intent) = std::str::from_utf8(unsafe { std::slice::from_raw_parts(in_utf8, in_len) })
    else {
        return -1;
    };
    let Ok(guard) = host_slot().lock() else {
        return -1;
    };
    let Some(slot) = guard.as_ref() else {
        return -1;
    };
    let Ok(patched) = inject_host_handle(intent, slot.handle) else {
        return -1;
    };
    if patched.len() > MAX_JSON_BYTES {
        return -1;
    }
    unsafe {
        motolii_rn_host_dispatch_intent_json(
            slot.handle,
            patched.as_ptr(),
            patched.len(),
            out,
            out_cap,
        )
    }
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn motolii_rnapp_host_snapshot_json(out: *mut u8, out_cap: usize) -> i64 {
    if out.is_null() || out_cap == 0 {
        return -1;
    }
    let Ok(guard) = host_slot().lock() else {
        return -1;
    };
    let Some(slot) = guard.as_ref() else {
        return -1;
    };
    unsafe { motolii_rn_host_read_snapshot_json(slot.handle, out, out_cap) }
}

/// Hostが在る時だけsnapshotを読む。不在・失敗はNone。
pub(crate) fn try_read_timeline_projection() -> Option<HostTimelineProjection> {
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(guard) = host_slot().lock() else {
            return None;
        };
        let Some(slot) = guard.as_ref() else {
            return None;
        };
        let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written =
            unsafe { motolii_rn_host_read_snapshot_json(slot.handle, out.as_mut_ptr(), out.len()) };
        if written <= 0 {
            return None;
        }
        let Ok(json) = std::str::from_utf8(&out[..written as usize]) else {
            return None;
        };
        parse_timeline_projection(json)
    }
}

fn inject_host_handle(intent: &str, handle: u64) -> Result<String, ()> {
    const KEY: &str = "\"host_handle\"";
    let key_at = intent.find(KEY).ok_or(())?;
    let after_key = &intent[key_at + KEY.len()..];
    let colon = after_key.find(':').ok_or(())?;
    let after_colon = &after_key[colon + 1..];
    let value_start_rel = after_colon.find('"').ok_or(())?;
    let value_body = &after_colon[value_start_rel + 1..];
    let value_end_rel = value_body.find('"').ok_or(())?;
    let abs_value_start = key_at + KEY.len() + colon + 1 + value_start_rel + 1;
    let abs_value_end = abs_value_start + value_end_rel;
    let mut out = String::with_capacity(intent.len() + 24);
    out.push_str(&intent[..abs_value_start]);
    out.push_str(&handle.to_string());
    out.push_str(&intent[abs_value_end..]);
    Ok(out)
}

fn parse_timeline_projection(json: &str) -> Option<HostTimelineProjection> {
    let revision = json_string_value(json, "revision")?;
    let primary_layer_id = json_string_value(json, "primary_layer_id");
    let bounds = parse_bounds(json)?;
    let timeline_layers = parse_timeline_layers(json);
    Some(HostTimelineProjection {
        revision,
        primary_layer_id,
        bounds,
        timeline_layers,
    })
}

pub(crate) fn snapshot_layers_from_projection(
    projection: &HostTimelineProjection,
) -> Vec<crate::timeline_skia::SnapshotLayerInput> {
    use crate::timeline_skia::{SnapshotKeyInput, SnapshotLayerInput};
    if let Some(layers) = &projection.timeline_layers {
        return layers
            .iter()
            .map(|layer| SnapshotLayerInput {
                layer_id: layer.layer_id.clone(),
                display_name: layer.display_name.clone(),
                interval_secs: Some((layer.start_secs, layer.duration_secs)),
                keys: layer
                    .position_keys
                    .iter()
                    .map(|key| SnapshotKeyInput {
                        key_id: key.key_id,
                        time_secs: key.time_secs,
                    })
                    .collect(),
            })
            .collect();
    }
    projection
        .bounds
        .iter()
        .map(|(layer_id, display_name)| SnapshotLayerInput {
            layer_id: layer_id.clone(),
            display_name: display_name.clone(),
            interval_secs: None,
            keys: Vec::new(),
        })
        .collect()
}

fn json_string_value(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with("null") {
                return None;
            }
            if let Some(body) = rest.strip_prefix('"') {
                let (decoded, _) = scan_json_string(body)?;
                return Some(decoded);
            }
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn scan_json_string(input: &str) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut out = String::new();
    let mut i = 0usize;
    while i < len {
        let b = bytes[i];
        if b == b'\\' {
            if i + 1 >= len {
                return None;
            }
            let esc = bytes[i + 1];
            match esc {
                b'"' => {
                    out.push('"');
                    i += 2;
                }
                b'\\' => {
                    out.push('\\');
                    i += 2;
                }
                b'/' => {
                    out.push('/');
                    i += 2;
                }
                b'b' => {
                    out.push('\u{0008}');
                    i += 2;
                }
                b'f' => {
                    out.push('\u{000C}');
                    i += 2;
                }
                b'n' => {
                    out.push('\n');
                    i += 2;
                }
                b'r' => {
                    out.push('\r');
                    i += 2;
                }
                b't' => {
                    out.push('\t');
                    i += 2;
                }
                b'u' => {
                    if i + 6 > len {
                        return None;
                    }
                    let mut codepoint = parse_hex_u16(input, i + 2)? as u32;
                    i += 6;
                    if (0xD800..=0xDBFF).contains(&(codepoint as u16)) {
                        if i + 6 > len || bytes[i] != b'\\' || bytes[i + 1] != b'u' {
                            return None;
                        }
                        let next_codepoint = parse_hex_u16(input, i + 2)?;
                        if !(0xDC00..=0xDFFF).contains(&next_codepoint) {
                            return None;
                        }
                        let high = codepoint as u32 - 0xD800;
                        let low = next_codepoint as u32 - 0xDC00;
                        codepoint = 0x10000 + ((high << 10) | low);
                        i += 6;
                    }
                    let Some(ch) = std::char::from_u32(codepoint) else {
                        return None;
                    };
                    out.push(ch);
                }
                _ => return None,
            }
            continue;
        }
        if b == b'"' {
            return Some((out, i));
        }
        let next = input[i..].chars().next()?;
        out.push(next);
        i += next.len_utf8();
    }
    None
}

fn parse_hex_u16(input: &str, start: usize) -> Option<u16> {
    let end = start + 4;
    if start >= input.len() || end > input.len() {
        return None;
    }
    u16::from_str_radix(&input[start..end], 16).ok()
}

fn parse_bounds(json: &str) -> Option<Vec<(String, String)>> {
    let marker = "\"bounds\"";
    let at = json.find(marker)?;
    let after = json[at + marker.len()..].trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut bounds = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let layer_id = json_string_value(obj, "layer_id")?.to_owned();
        let display_name = json_string_value(obj, "display_name")?.to_owned();
        bounds.push((layer_id, display_name));
        rest = &rest[end + 1..];
    }
    Some(bounds)
}

fn parse_timeline_layers(json: &str) -> Option<Vec<HostTimelineLayer>> {
    let timeline = find_key_object(json, "timeline")?;
    let marker = "\"layers\"";
    let at = timeline.find(marker)?;
    let after = timeline[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut layers = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let layer_id = json_string_value(obj, "layer_id")?.to_owned();
        let display_name = json_string_value(obj, "display_name")?.to_owned();
        let (start_num, start_den) = json_rational(obj, "start")?;
        let (duration_num, duration_den) = json_rational(obj, "duration")?;
        if start_den == 0 || duration_den == 0 {
            return None;
        }
        let position_keys = parse_position_keys(obj)?;
        layers.push(HostTimelineLayer {
            layer_id,
            display_name,
            start_secs: start_num as f64 / start_den as f64,
            duration_secs: duration_num as f64 / duration_den as f64,
            position_keys,
        });
        rest = &rest[end + 1..];
    }
    Some(layers)
}

fn parse_position_keys(layer_obj: &str) -> Option<Vec<HostTimelineKey>> {
    let marker = "\"position_keys\"";
    let at = layer_obj.find(marker)?;
    let after = layer_obj[at + marker.len()..]
        .trim_start()
        .strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('[')?;
    let mut keys = Vec::new();
    let mut rest = after;
    loop {
        rest = rest.trim_start();
        if rest.starts_with(']') {
            break;
        }
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
            continue;
        }
        if !rest.starts_with('{') {
            return None;
        }
        let end = find_matching_brace(rest)?;
        let obj = &rest[..=end];
        let key_id = json_string_value(obj, "key_id")?.parse::<u64>().ok()?;
        let (time_num, time_den) = json_rational(obj, "time")?;
        if time_den == 0 {
            return None;
        }
        keys.push(HostTimelineKey {
            key_id,
            time_secs: time_num as f64 / time_den as f64,
        });
        rest = &rest[end + 1..];
    }
    Some(keys)
}

fn find_key_object<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with('{') {
                let end = find_matching_brace(rest)?;
                return Some(&rest[..=end]);
            }
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn json_rational(json: &str, key: &str) -> Option<(i64, i64)> {
    let obj = find_key_object(json, key)?;
    let num = json_i64_value(obj, "num")?;
    let den = json_i64_value(obj, "den")?;
    Some((num, den))
}

fn json_i64_value(json: &str, key: &str) -> Option<i64> {
    let needle = format!("\"{key}\"");
    let mut search = json;
    let mut abs = 0usize;
    while let Some(at) = search.find(&needle) {
        abs += at;
        let after = &json[abs + needle.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            let end = rest
                .find(|ch: char| !(ch.is_ascii_digit() || ch == '-' || ch == '+'))
                .unwrap_or(rest.len());
            if end == 0 {
                return None;
            }
            return rest[..end].parse().ok();
        }
        abs += needle.len();
        search = &json[abs..];
    }
    None
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, ch) in s.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::timeline_skia::TimelineScene;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Mutex, MutexGuard};

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
        let mut out = [0u8; MAX_JSON_BYTES];
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
        let response = std::str::from_utf8(&out[..written as usize]).expect("utf8");
        assert!(
            response.contains(r#""accepted":true"#),
            "expected accepted: {response}"
        );
    }

    #[test]
    fn place_then_undo_changes_from_snapshot_band_count() {
        let _lock = test_lock();
        let path = temp_project("place-undo");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
        let baseline_scene = TimelineScene::from_snapshot(
            &bounds_from_snapshot(&baseline),
            baseline.primary_layer_id.as_deref(),
        );
        assert_eq!(baseline_scene.band_count(), baseline.layer_ids.len());
        assert!(baseline_scene.real);

        dispatch_kind(
            host,
            "place_rectangle",
            r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
        );
        let placed_snap = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
        let placed_scene = TimelineScene::from_snapshot(
            &bounds_from_snapshot(&placed_snap),
            placed_snap.primary_layer_id.as_deref(),
        );
        assert_eq!(placed_scene.band_count(), baseline.layer_ids.len() + 1);

        dispatch_kind(host, "undo", "");
        let undone_snap = motolii_ui::host_read_snapshot_for_test(host).expect("undone");
        let undone_scene = TimelineScene::from_snapshot(
            &bounds_from_snapshot(&undone_snap),
            undone_snap.primary_layer_id.as_deref(),
        );
        assert_eq!(undone_scene.band_count(), baseline.layer_ids.len());

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

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
    fn parse_projection_reads_bounds_and_revision() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"L1",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"L1","display_name":"rect \"A\""},
                {"layer_id":"L2","display_name":"rect \n"}
            ]},
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert_eq!(proj.revision, "3");
        assert_eq!(proj.primary_layer_id.as_deref(), Some("L1"));
        assert_eq!(
            proj.bounds,
            vec![
                ("L1".into(), r#"rect "A""#.into()),
                ("L2".into(), "rect \n".into())
            ]
        );
    }

    #[test]
    fn parse_projection_from_host_snapshot_json_matches_wire_projection() {
        let _lock = test_lock();
        let path = temp_project("projection-json-wire");
        let host = motolii_ui::host_create_for_test(&path).expect("create host");
        let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
        let mut out = [0u8; MAX_JSON_BYTES];
        let written =
            unsafe { motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len()) };
        assert!(written > 0, "host snapshot json failed: {written}");
        let json = std::str::from_utf8(&out[..written as usize]).expect("snapshot json");
        let proj = parse_timeline_projection(json).expect("projection parse");
        assert_eq!(proj.revision, baseline.revision);
        assert_eq!(proj.primary_layer_id, baseline.primary_layer_id);
        assert_eq!(proj.bounds.len(), baseline.layer_ids.len());
        for (idx, layer_id) in baseline.layer_ids.iter().enumerate() {
            assert_eq!(proj.bounds[idx].0, *layer_id);
        }
        let timeline = proj.timeline_layers.expect("timeline from host");
        assert_eq!(timeline.len(), baseline.timeline.layers.len());
        assert_eq!(timeline[0].layer_id, baseline.timeline.layers[0].layer_id);

        motolii_ui::host_destroy_for_test(host).expect("destroy");
    }

    #[test]
    fn parse_projection_parses_layer_ids_and_falls_back_without_timeline_on_bad_key() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"9",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"L1",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"L1","display_name":"rect"}
            ]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[
                    {
                        "layer_id":"L1",
                        "display_name":"rect",
                        "start":{"num":0,"den":1},
                        "duration":{"num":10,"den":1},
                        "position_keys":[
                            {"key_id":"NaN","time":{"num":4,"den":1}}
                        ],
                        "keys_truncated":false
                    }
                ],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert_eq!(proj.bounds, vec![("L1".into(), "rect".into())]);
        assert!(proj.timeline_layers.is_none());
    }

    #[test]
    fn timeline_json_maps_to_scene_bars_and_keeps_key_id() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"7",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"11",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"11","display_name":"rect"}
            ]},
            "timeline":{
                "fps":{"num":30,"den":1},
                "layers":[{
                    "layer_id":"11",
                    "display_name":"rect",
                    "start":{"num":0,"den":1},
                    "duration":{"num":10,"den":1},
                    "position_keys":[{"key_id":"42","time":{"num":4,"den":1}}],
                    "keys_truncated":false
                }],
                "layers_truncated":false
            },
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        let layers = snapshot_layers_from_projection(&proj);
        let scene = TimelineScene::from_snapshot(&layers, proj.primary_layer_id.as_deref());
        let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert!((a - 0.0).abs() < 1e-6);
        assert!((b - 5.0).abs() < 1e-6); // 10s / 2
        assert_eq!(keys.len(), 1);
        assert!((keys[0].0 - 2.0).abs() < 1e-6); // 4s / 2
        assert_eq!(keys[0].1, 42);
        assert_eq!(scene.selected_flat, 0);
    }

    #[test]
    fn missing_timeline_falls_back_to_full_width_rows() {
        let json = r#"{
            "version":1,
            "direction":"host-to-rn",
            "role":"product",
            "host_handle":"1",
            "revision":"3",
            "projection_generation":"0",
            "current_time":{"num":0,"den":1},
            "primary_layer_id":"L1",
            "stage":{"selection":[],"bounds":[
                {"layer_id":"L1","display_name":"rect \"A\""},
                {"layer_id":"L2","display_name":"rect \n"}
            ]},
            "diagnostics":[]
        }"#;
        let proj = parse_timeline_projection(json).expect("parse");
        assert!(proj.timeline_layers.is_none());
        let layers = snapshot_layers_from_projection(&proj);
        let scene = TimelineScene::from_snapshot(&layers, proj.primary_layer_id.as_deref());
        let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
        assert!((a - 0.0).abs() < 1e-6);
        assert!((b - 96.0).abs() < 1e-6);
        assert!(keys.is_empty());
        assert_eq!(scene.band_count(), 2);
    }
}
