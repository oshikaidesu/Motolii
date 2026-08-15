use super::{clear_slot, dispatch_kind, install_slot, temp_project, test_lock};
use super::super::parse_wire::{layer_has_position_key, parse_timeline_projection};
use super::super::slot::{
    slice_from_written, MAX_SNAPSHOT_JSON_BYTES, TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT,
};
use super::super::try_dispatch_timeline_edit;
use super::super::{frame_from_scrub_bar, snapshot_layers_from_projection};
use crate::timeline_skia::TimelineScene;
use std::sync::atomic::Ordering;

#[cfg(target_os = "macos")]
use motolii_ui::motolii_rn_host_read_snapshot_json;

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
    assert_eq!(proj.host_handle.as_deref(), Some("1"));
    assert_eq!(proj.revision, "3");
    assert_eq!(proj.projection_generation, "0");
    assert_eq!(proj.current_time, (0, 1));
    assert!(proj.fps.is_none());
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
    let mut out = [0u8; MAX_SNAPSHOT_JSON_BYTES];
    let written =
        unsafe { motolii_rn_host_read_snapshot_json(host, out.as_mut_ptr(), out.len()) };
    assert!(written > 0, "host snapshot json failed: {written}");
    let json_bytes =
        slice_from_written(&out, written).expect("snapshot response within buffer");
    let json = std::str::from_utf8(json_bytes).expect("snapshot json");
    let proj = parse_timeline_projection(json).expect("projection parse");
    assert_eq!(proj.revision, baseline.revision);
    assert_eq!(proj.primary_layer_id, baseline.primary_layer_id);
    assert_eq!(proj.bounds.len(), baseline.layer_ids.len());
    for (idx, layer_id) in baseline.layer_ids.iter().enumerate() {
        assert_eq!(proj.bounds[idx].0, *layer_id);
    }
    let timeline = proj.timeline_layers.expect("timeline from host");
    assert_eq!(timeline.len(), baseline.timeline.layers.len());
    if !timeline.is_empty() {
        assert_eq!(timeline[0].layer_id, baseline.timeline.layers[0].layer_id);
    }
    assert_eq!(
        proj.fps,
        Some((baseline.timeline.fps.num(), baseline.timeline.fps.den()))
    );
    assert_eq!(
        proj.current_time,
        (baseline.current_time.num(), baseline.current_time.den())
    );

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
    assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
    let duration_secs = 10.0;
    let song_bars = duration_secs / crate::timeline_skia::SECONDS_PER_BAR;
    assert!((song_bars - duration_secs).abs() < 1e-12);
    let scene = TimelineScene::from_snapshot_with_song_bars(
        &layers,
        proj.primary_layer_id.as_deref(),
        song_bars as f32,
    );
    let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
    assert!((a - 0.0).abs() < 1e-6);
    assert!((f64::from(b) - duration_secs).abs() < 1e-6);
    assert_eq!(keys.len(), 1);
    assert!((keys[0].0 - 4.0).abs() < 1e-6);
    assert_eq!(keys[0].1, 42);
    assert_eq!(scene.selected_flat, 0);
    assert!((f64::from(scene.song_bars) - duration_secs).abs() < 1e-6);
    assert!((scene.view_a - 0.0).abs() < 1e-6);
    assert!((f64::from(scene.view_b) - duration_secs).abs() < 1e-6);
    assert_eq!(proj.fps, Some((30, 1)));
    assert_eq!(frame_from_scrub_bar(duration_secs, 30, 1), 300);
    assert_eq!(frame_from_scrub_bar(f64::from(keys[0].0), 30, 1), 120);
}

#[test]
fn position_keys_parse_optional_value_without_requiring_it() {
    let with_value = r#"{
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
                "position_keys":[
                    {"key_id":"42","time":{"num":4,"den":1},"value":[0.25,-0.5]},
                    {"key_id":"43","time":{"num":5,"den":1}}
                ],
                "keys_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(with_value).expect("parse");
    let layers = proj.timeline_layers.expect("layers");
    assert_eq!(layers[0].position_keys.len(), 2);
    assert_eq!(layers[0].position_keys[0].value, Some([0.25, -0.5]));
    assert_eq!(layers[0].position_keys[1].value, None);
    assert_eq!(layers[0].position_keys[0].key_id, 42);
    assert_eq!(layers[0].position_keys[1].key_id, 43);
}

#[test]
fn param_keys_union_into_scene_keys() {
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
                "param_keys":[
                    {"property":"scale","key_id":"99","time":{"num":2,"den":1},"vec":[1.0,1.0]},
                    {"property":"opacity","key_id":"100","time":{"num":6,"den":1},"value":0.5}
                ],
                "keys_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    let host_layers = proj.timeline_layers.as_ref().expect("layers");
    assert_eq!(host_layers[0].position_keys.len(), 1);
    assert_eq!(host_layers[0].param_keys.len(), 2);
    assert_eq!(host_layers[0].param_keys[0].key_id, 99);
    assert_eq!(host_layers[0].param_keys[1].key_id, 100);
    assert!(layer_has_position_key(host_layers, "11", 42));
    assert!(!layer_has_position_key(host_layers, "11", 99));
    assert!(!layer_has_position_key(host_layers, "11", 100));
    let layers = snapshot_layers_from_projection(&proj);
    let scene = TimelineScene::from_snapshot_with_song_bars(
        &layers,
        proj.primary_layer_id.as_deref(),
        10.0,
    );
    let (_, _, keys) = scene.clip0_span_and_keys(0).expect("clip0");
    assert_eq!(keys.len(), 3);
    assert!(
        keys.iter()
            .any(|key| key.1 == 42 && (key.0 - 4.0).abs() < 1e-6)
    );
    assert!(
        keys.iter()
            .any(|key| key.1 == 99 && (key.0 - 2.0).abs() < 1e-6)
    );
    assert!(
        keys.iter()
            .any(|key| key.1 == 100 && (key.0 - 6.0).abs() < 1e-6)
    );
}

#[test]
fn param_key_id_does_not_dispatch_set_position_key_time() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("param-key-drag-noop");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);
    dispatch_kind(
        host,
        "place_rectangle",
        r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
    );
    let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
    let layer_id = placed.primary_layer_id.expect("primary after place");
    dispatch_kind(
        host,
        "add_position_key",
        &format!(r#","target":"{layer_id}","time":{{"num":1,"den":1}}"#),
    );
    let keyed = motolii_ui::host_read_snapshot_for_test(host).expect("keyed");
    let position_key_id = keyed
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("keyed layer")
        .position_keys
        .last()
        .expect("added key")
        .key_id
        .parse::<u64>()
        .expect("key id");
    let param_key_id = position_key_id.wrapping_add(1_000_003);
    TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.store(0, Ordering::SeqCst);
    assert!(
        try_dispatch_timeline_edit(
            &crate::timeline_skia::TimelineEditCommit::SetPositionKeyTime {
                layer_id: layer_id.clone(),
                key_id: param_key_id,
                bar: 2.0,
            }
        )
        .is_none()
    );
    assert_eq!(
        TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.load(Ordering::SeqCst),
        0
    );
    let _ = try_dispatch_timeline_edit(
        &crate::timeline_skia::TimelineEditCommit::SetPositionKeyTime {
            layer_id,
            key_id: position_key_id,
            bar: 2.0,
        },
    );
    assert_eq!(
        TEST_SET_POSITION_KEY_TIME_DISPATCH_COUNT.load(Ordering::SeqCst),
        1
    );

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
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
    let scene = TimelineScene::from_snapshot_with_song_bars(
        &layers,
        proj.primary_layer_id.as_deref(),
        (10.0 / crate::timeline_skia::SECONDS_PER_BAR) as f32,
    );
    let (a, b, keys) = scene.clip0_span_and_keys(0).expect("clip0");
    assert!((a - 0.0).abs() < 1e-6);
    assert!((b - 10.0).abs() < 1e-6);
    assert!(keys.is_empty());
    assert_eq!(scene.band_count(), 2);
}

