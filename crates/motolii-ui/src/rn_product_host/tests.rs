//! 元 file の tests module。期待値は動かさない。

use std::collections::BTreeMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Clip, ClipSource, CompCameraDoc, DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document,
    EffectDefinition, EffectDefinitionId, EffectUse, ItemEnvelope, KeyframeId, LayerId,
    ProjectSession, ResourceLimits, SaveProjectOptions, Track, TrackItem, Transform2D,
    RECT_LAYER_SOURCE,
};
use motolii_gpu::download_rgba;
use motolii_render::RenderSession;
use motolii_testkit::tmp_dir;

use super::app_api::*;
use super::dispatch::*;
use super::error::*;
use super::gpu_draw::*;
use super::gpu_surface::*;
use super::host::*;
use super::projection::*;
use super::registry::*;
use super::surfaces::*;
use super::wire::*;
use super::wire_io::*;
use super::*;

static TEST_LOCK: Mutex<()> = Mutex::new(());
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(super) fn fixture_path(tag: &str) -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
    let mut document = Document::new_current();
    let layer = document.layers.allocate("r0-layer").expect("layer");
    let track = document.track_ids.allocate("r0-track").expect("track");
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
                    ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                    ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    document.validate().expect("valid fixture document");
    let limits = ResourceLimits::production();
    let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
    session
        .save_with_journal(
            &document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .expect("save fixture");
    path
}

pub(super) fn pixel_at(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let base = ((y * width + x) * 4) as usize;
    [
        bytes[base],
        bytes[base + 1],
        bytes[base + 2],
        bytes[base + 3],
    ]
}

pub(super) fn has_non_background_pixel(
    bytes: &[u8],
    width: u32,
    height: u32,
    background: [u8; 4],
) -> bool {
    for y in 0..height {
        for x in 0..width {
            if pixel_at(bytes, width, x, y) != background {
                return true;
            }
        }
    }
    false
}

pub(super) fn create_host(tag: &str) -> u64 {
    let path = fixture_path(tag);
    host_create_for_test(&path).expect("host")
}

pub(super) fn read_snapshot(host: u64) -> RnProductSnapshotForTest {
    host_read_snapshot_for_test(host).expect("snapshot")
}

pub(super) fn dispatch(host: u64, intent: RnHostTestIntent) -> RnHostTestResponse {
    host_dispatch_intent_for_test(host, intent).expect("dispatch")
}

pub(super) fn base_intent(kind: &str) -> RnHostTestIntent {
    RnHostTestIntent {
        kind: kind.to_owned(),
        stage_handle: None,
        projection_generation: None,
        width: None,
        height: None,
        scale_factor: None,
        focused: None,
    }
}

pub(super) fn pointer_intent(
    stage: u64,
    phase: &str,
    view_local_x: f64,
    view_local_y: f64,
    sequence: u64,
) -> WireIntentEnvelope {
    WireIntentEnvelope {
        version: WIRE_VERSION,
        direction: RN_TO_HOST.to_owned(),
        kind: "stage_pointer".to_owned(),
        host_handle: String::new(),
        stage_handle: Some(stage.to_string()),
        projection_generation: None,
        width: None,
        height: None,
        scale_factor: None,
        focused: None,
        phase: Some(phase.to_owned()),
        view_local_x: Some(view_local_x),
        view_local_y: Some(view_local_y),
        sequence: Some(sequence),
        frame: None,
        position: None,
        playhead: None,
        target: None,
        dest: None,
        key_id: None,
        property: None,
        time: None,
        new: None,
        interp: None,
        delta: None,
        plugin_id: None,
        item_id: None,
        effect_use_id: None,
        param_id: None,
        value: None,
        output_path: None,
        color: None,
    }
}

pub(super) fn set_time_json(host: u64, frame_json: &str) -> String {
    format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"set_time","#,
            r#""host_handle":"{host}","frame":{frame}}}"#
        ),
        host = host,
        frame = frame_json
    )
}

pub(super) fn host_kind_json(host: u64, kind: &str) -> String {
    format!(
        r#"{{"version":1,"direction":"rn-to-host","kind":"{kind}","host_handle":"{host}"}}"#,
        kind = kind,
        host = host,
    )
}

pub(super) fn dispatch_raw_json(host: u64, intent_json: &str) -> RnHostTestResponse {
    #[cfg(target_os = "macos")]
    {
        let mut out = vec![0u8; MAX_SNAPSHOT_JSON_BYTES];
        let written = motolii_rn_host_dispatch_intent_json(
            host,
            intent_json.as_ptr(),
            intent_json.len(),
            out.as_mut_ptr(),
            out.len(),
        );
        assert!(
            written > 0,
            "motolii_rn_host_dispatch_intent_json failed: {written}"
        );
        let response: WireIntentResponse =
            serde_json::from_slice(&out[..written as usize]).expect("response json");
        response_for_test(response)
    }
    #[cfg(not(target_os = "macos"))]
    {
        with_registry(|registry| {
            let out = registry.dispatch_intent_json(host, intent_json)?;
            let response: WireIntentResponse =
                serde_json::from_str(&out).map_err(RnHostError::from)?;
            Ok(response_for_test(response))
        })
        .expect("dispatch raw json")
    }
}

pub(super) fn fixture_path_with_fps(tag: &str, fps: motolii_core::Fps) -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
    let mut document = Document::new_current();
    document.composition.fps = fps;
    let layer = document.layers.allocate("r0-layer").expect("layer");
    let track = document.track_ids.allocate("r0-track").expect("track");
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
                    ("size".into(), DocParam::const_vec2([1.0, 1.0])),
                    ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    document.validate().expect("valid fixture document");
    let limits = ResourceLimits::production();
    let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
    session
        .save_with_journal(
            &document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .expect("save fixture");
    path
}

pub(super) fn create_host_with_fps(tag: &str, fps: motolii_core::Fps) -> u64 {
    let path = fixture_path_with_fps(tag, fps);
    host_create_for_test(&path).expect("host")
}

pub(super) fn save_document_fixture(tag: &str, document: &Document) -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
    let limits = ResourceLimits::production();
    let mut session = ProjectSession::acquire(&path, &limits).expect("acquire fixture");
    session
        .save_with_journal(
            document,
            &SaveProjectOptions {
                limits,
                checkpoint: true,
                ..SaveProjectOptions::default()
            },
        )
        .expect("save fixture");
    path
}

pub(super) fn create_host_from_document(tag: &str, document: &Document) -> u64 {
    let path = save_document_fixture(tag, document);
    host_create_for_test(&path).expect("host")
}

struct Fixture {
    document: Document,
}

impl Fixture {
    pub(super) fn new() -> Self {
        Self {
            document: Document::new_current(),
        }
    }

    pub(super) fn push_rect_layer(
        &mut self,
        name: &str,
        center: [f64; 2],
        size: [f64; 2],
        transform: Transform2D,
    ) -> LayerId {
        if self.document.tracks.is_empty() {
            let track = self.document.track_ids.allocate("V1").expect("track");
            self.document.tracks.push(Track {
                id: track,
                items: vec![],
            });
        }
        let layer = self.document.layers.allocate(name).expect("layer");
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform = transform;
        self.document.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: self.document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params(center, size),
                extra: Default::default(),
            },
        }));
        layer
    }
}

pub(super) fn rect_params(center: [f64; 2], size: [f64; 2]) -> BTreeMap<String, DocParam> {
    BTreeMap::from([
        ("center".into(), DocParam::const_vec2(center)),
        ("size".into(), DocParam::const_vec2(size)),
        ("color".into(), DocParam::const_color([0.0, 1.0, 0.0, 1.0])),
    ])
}

pub(super) fn mount_and_resize(host: u64, stage: u64, width: u32, height: u32) {
    let mut mount = base_intent("stage_mount");
    mount.stage_handle = Some(stage);
    assert!(dispatch(host, mount).accepted);
    let mut resize = base_intent("stage_resize");
    resize.stage_handle = Some(stage);
    resize.width = Some(width);
    resize.height = Some(height);
    resize.scale_factor = Some(1.0);
    assert!(dispatch(host, resize).accepted);
}

pub(super) fn pointer_json(
    host: u64,
    stage: u64,
    phase: &str,
    view_local_x: f64,
    view_local_y: f64,
    sequence: u64,
) -> String {
    format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"stage_pointer","#,
            r#""host_handle":"{host}","stage_handle":"{stage}","phase":"{phase}","#,
            r#""view_local_x":{x},"view_local_y":{y},"sequence":{sequence}}}"#
        ),
        host = host,
        stage = stage,
        phase = phase,
        x = view_local_x,
        y = view_local_y,
        sequence = sequence
    )
}

pub(super) fn canonical_to_view_local(
    canonical_x: f64,
    canonical_y: f64,
    width: u32,
    height: u32,
) -> (f64, f64) {
    let w = f64::from(width);
    let h = f64::from(height);
    (w * 0.5 + canonical_x * h, h * 0.5 + canonical_y * h)
}

pub(super) fn document_json_bytes(host: u64) -> Vec<u8> {
    with_registry(|registry| {
        let product = registry
            .hosts
            .get(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(serde_json::to_vec(product.runtime.snapshot().as_ref()).expect("document json"))
    })
    .expect("document bytes")
}

pub(super) fn dispatch_wire(host: u64, mut intent: WireIntentEnvelope) -> RnHostTestResponse {
    intent.host_handle = host.to_string();
    // JSON 経由だと非有限 f64 を運べないため、受理検証は envelope 直送で行う。
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        Ok(response_for_test(product.dispatch_intent(host, intent)))
    })
    .expect("dispatch wire")
}

pub(super) fn read_stage_pointer(stage: u64) -> Option<StagePointerTransient> {
    with_registry(|registry| {
        for host in registry.hosts.values() {
            if let Some(surface) = host.stages.get(&stage) {
                return Ok(surface.pointer.clone());
            }
        }
        Err(RnHostError::UnknownStage(stage))
    })
    .ok()
    .flatten()
}

pub(super) fn make_16_layers_64_keys_document() -> Document {
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("stress").expect("track");
    document.tracks.push(Track {
        id: track,
        items: vec![],
    });
    for layer_idx in 0_u64..16 {
        let layer = document
            .layers
            .allocate(&format!("layer-{layer_idx}"))
            .expect("layer");
        let mut keyframes = DocKeyframeTrack::new();
        for key_idx in 0_u64..64 {
            let key_id = document.next_stable_id.allocate().expect("key id");
            keyframes.insert(DocKeyframe {
                id: KeyframeId::from_raw(key_id),
                t: RationalTime::try_new(key_idx as i64, 1).expect("key time"),
                value: DocValue::Vec2([0.0, key_idx as f64]),
                interp: Interp::Linear,
            });
        }
        let mut envelope = ItemEnvelope::new(layer);
        envelope.transform.position = DocParam::Keyframes(keyframes);
        document.tracks[0].items.push(TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params([0.0, 0.0], [1.0, 1.0]),
                extra: Default::default(),
            },
        }));
    }
    document.validate().expect("valid");
    document
}

#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
pub(super) fn parse_wire_response(buf: &[u8], len: i64) -> WireIntentResponse {
    assert!(len > 0);
    let json = std::str::from_utf8(&buf[..len as usize]).expect("utf8");
    serde_json::from_str(json).expect("wire response")
}

#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
pub(super) fn read_projection_stamp(host: u64) -> (u64, u64) {
    let mut revision = 0u64;
    let mut generation = 0u64;
    assert!(
        motolii_rn_host_projection_stamp(host, &mut revision, &mut generation),
        "stamp ffi"
    );
    (revision, generation)
}

#[cfg(target_os = "macos")]
pub(super) fn read_snapshot_json_bytes(host: u64) -> Vec<u8> {
    let mut out = vec![0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written = motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len());
    assert!(written > 0, "snapshot read failed: {written}");
    out[..written as usize].to_vec()
}

/// F9: stampはsnapshot JSONが変わり得る全変更で必ず動く。no-opでは不変、stamp不変⇒snapshot不変。
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "macos")]
pub(super) fn mirror_signed_area(corners: &[[f64; 2]; 4]) -> f64 {
    let p0 = corners[0];
    let p1 = corners[1];
    let p2 = corners[2];
    let p3 = corners[3];
    0.5 * ((p0[0] * p1[1] - p1[0] * p0[1])
        + (p1[0] * p2[1] - p2[0] * p1[1])
        + (p2[0] * p3[1] - p3[0] * p2[1])
        + (p3[0] * p0[1] - p0[0] * p3[1]))
}

pub(super) fn keyed_scale_document() -> (Document, LayerId, KeyframeId) {
    let mut document = Document::new_current();
    let layer = document.layers.allocate("keyed-scale").expect("layer");
    let track = document.track_ids.allocate("track").expect("track");
    let key_id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key"));
    let mut keyframes = DocKeyframeTrack::new();
    keyframes.insert(DocKeyframe {
        id: key_id,
        t: RationalTime::try_new(1, 1).expect("1s"),
        value: DocValue::Vec2([1.0, 1.0]),
        interp: Interp::Linear,
    });
    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform.scale = DocParam::Keyframes(keyframes);
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: rect_params([0.0, 0.0], [1.0, 1.0]),
                extra: Default::default(),
            },
        })],
    });
    document.validate().expect("valid keyed scale");
    (document, layer, key_id)
}

pub(super) fn position_const_at(document: &Document, target: LayerId) -> Option<[f64; 2]> {
    let envelope = find_envelope_in_document(document, target)?;
    match &envelope.transform.position {
        DocParam::Const(DocValue::Vec2(value)) => Some(*value),
        _ => None,
    }
}

pub(super) fn seed_primary(host: u64, target: LayerId) {
    with_registry(|registry| {
        let product = registry
            .hosts
            .get_mut(&host)
            .ok_or(RnHostError::UnknownHost(host))?;
        let mut queue = DocumentEditQueue::default();
        queue.push_replace_primary(target);
        let published = product
            .runtime
            .process_next(&mut queue, product.primary, product.projection_generation)
            .expect("process")
            .expect("published");
        product.primary = published.primary;
        product.projection_generation = published.projection_generation;
        Ok(())
    })
    .expect("seed primary");
}

pub(super) fn read_wire(host: u64) -> WireProductSnapshot {
    with_registry(|registry| registry.read_snapshot(host)).expect("wire snapshot")
}

pub(super) fn layer_effects<'a>(
    wire: &'a WireProductSnapshot,
    layer_id: &str,
) -> &'a [WireTimelineEffect] {
    &wire
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer")
        .effects
}

/// RN `readSnapshot` と同じ `encode_snapshot_json` 経路。
pub(super) fn read_snapshot_json(host: u64) -> serde_json::Value {
    let json = encode_snapshot_json(&read_wire(host)).expect("snapshot json");
    serde_json::from_str(&json).expect("parse snapshot json")
}

/// App.tsx `hostSnapshotStateFromParsed` と同じ: primary の timeline layer effects だけ。
pub(super) fn inspector_selected_effects(snapshot: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(primary) = snapshot
        .get("primary_layer_id")
        .and_then(|value| value.as_str())
    else {
        return Vec::new();
    };
    snapshot
        .get("timeline")
        .and_then(|timeline| timeline.get("layers"))
        .and_then(|layers| layers.as_array())
        .and_then(|layers| {
            layers
                .iter()
                .find(|layer| layer.get("layer_id").and_then(|id| id.as_str()) == Some(primary))
        })
        .and_then(|layer| layer.get("effects"))
        .and_then(|effects| effects.as_array())
        .cloned()
        .unwrap_or_default()
}

pub(super) fn live_document(host: u64) -> Arc<Document> {
    with_registry(|registry| Ok(registry.hosts.get(&host).expect("host").runtime.snapshot()))
        .expect("document")
}

pub(super) fn document_clip(document: &Document, layer_id: LayerId) -> Option<&Clip> {
    document.tracks.iter().find_map(|track| {
        track.items.iter().find_map(|item| match item {
            TrackItem::Clip(clip) if clip.envelope.layer_id == layer_id => Some(clip),
            _ => None,
        })
    })
}

pub(super) fn create_empty_track_host(tag: &str) -> u64 {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = tmp_dir(&format!("rn-product-host-{tag}-{id}")).join("project.json");
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("seed-track").expect("track");
    document.tracks.push(Track {
        id: track,
        items: vec![],
    });
    document.validate().expect("valid empty track document");
    let limits = ResourceLimits::production();
    {
        let mut session = ProjectSession::acquire(&path, &limits).expect("acquire");
        session
            .save_with_journal(
                &document,
                &SaveProjectOptions {
                    limits,
                    checkpoint: true,
                    ..SaveProjectOptions::default()
                },
            )
            .expect("save");
    }
    host_create_for_test(&path).expect("host")
}

pub(super) fn place_vism_json(
    host: u64,
    plugin_id: &str,
    position: [f64; 2],
    playhead: &str,
) -> String {
    format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"place_vism","#,
            r#""host_handle":"{host}","plugin_id":"{plugin}","position":[{x},{y}],"playhead":{playhead}}}"#
        ),
        host = host,
        plugin = plugin_id,
        x = position[0],
        y = position[1],
        playhead = playhead,
    )
}

pub(super) fn place_media_json(
    host: u64,
    item_id: &str,
    position: [f64; 2],
    playhead: &str,
) -> String {
    format!(
        concat!(
            r#"{{"version":1,"direction":"rn-to-host","kind":"place_media","#,
            r#""host_handle":"{host}","item_id":"{item}","position":[{x},{y}],"playhead":{playhead}}}"#
        ),
        host = host,
        item = item_id,
        x = position[0],
        y = position[1],
        playhead = playhead,
    )
}
