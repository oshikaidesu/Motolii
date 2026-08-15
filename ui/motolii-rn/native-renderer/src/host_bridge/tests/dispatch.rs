use super::{
    clear_slot, dispatch_kind, install_slot, read_host_projection, temp_project, test_lock,
};
use super::super::parse_wire::parse_timeline_projection;
use super::super::terminal::{parse_terminal_result, response_is_accepted};
use super::super::types::HostTerminalDiagnostic;
use super::super::{
    frame_from_scrub_bar, motolii_rnapp_host_key_event, playhead_from_current_time,
    try_dispatch_set_time, try_dispatch_timeline_edit,
};
use crate::timeline_skia::{TimelineEditCommit, TimelineScene};

#[test]
fn set_time_dispatch_requires_accepted_response() {
    assert!(response_is_accepted(r#"{"accepted":true}"#));
    assert!(!response_is_accepted(r#"{"accepted":false}"#));
    assert!(!response_is_accepted(r#"{"foo":true}"#));
    assert!(!response_is_accepted(r#"not-json"#));
}

#[test]
fn terminal_response_keeps_typed_diagnostic_and_authoritative_snapshot() {
    let response = r#"{
        "accepted":false,
        "snapshot":{
            "revision":"9",
            "projection_generation":"5",
            "current_time":{"num":3,"den":1},
            "stage":{"selection":[],"bounds":[]},
            "stage_geometry":{"layers":[],"layers_truncated":false},
            "timeline":{"duration":{"num":10,"den":1},"fps":{"num":30,"den":1},"layers":[],"layers_truncated":false},
            "diagnostics":[{"reason":"snapshot_only"}]
        },
        "diagnostics":[{
            "reason":"stale_projection_generation",
            "expected_projection_generation":"4",
            "actual_projection_generation":"5"
        }],
        "message":"stale terminal edit"
    }"#;
    let result = parse_terminal_result(response).expect("terminal response");
    assert!(!result.accepted);
    assert_eq!(result.feedback(), Some("stale terminal edit"));
    assert_eq!(result.stamp(), Some((9, 5)));
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0],
        HostTerminalDiagnostic {
            reason: "stale_projection_generation".into(),
            host_handle: None,
            stage_handle: None,
            timeline_handle: None,
            expected_projection_generation: Some("4".into()),
            actual_projection_generation: Some("5".into()),
        }
    );
    assert_eq!(result.projection.expect("snapshot").current_time, (3, 1));
}

#[test]
fn playhead_from_current_time_uses_seconds_not_bars() {
    assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
    // 2s / fixture曲長96秒 → 2/96。Ableton 1bar=2s なら 1/96 になる。
    let ph_fixture = playhead_from_current_time(2, 1, crate::timeline_skia::SONG_BARS);
    assert!((ph_fixture - 2.0 / 96.0).abs() < 1e-12);
    let ph_10s = playhead_from_current_time(2, 1, 10.0);
    assert!((ph_10s - 2.0 / 10.0).abs() < 1e-12);
    assert_eq!(playhead_from_current_time(0, 1, 10.0), 0.0);
}

#[test]
fn parse_timeline_projection_parses_timeline_duration() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"7",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "primary_layer_id":"11",
        "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"rect"}]},
        "timeline":{
            "duration":{"num":40,"den":1},
            "fps":{"num":30,"den":1},
            "layers":[{
                "layer_id":"11",
                "display_name":"rect",
                "start":{"num":0,"den":1},
                "duration":{"num":40,"den":1},
                "position_keys":[],
                "keys_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[ ]
    }"#;
    let projection = parse_timeline_projection(json).expect("parse");
    assert_eq!(projection.timeline_duration, Some((40, 1)));
}

#[test]
fn set_time_dispatch_moves_current_time_via_host_slot() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("set-time-scrub");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);

    let baseline = motolii_ui::host_read_snapshot_for_test(host).expect("baseline");
    let fps_num = baseline.timeline.fps.num();
    let fps_den = baseline.timeline.fps.den();
    assert_eq!((fps_num, fps_den), (30, 1));

    // 1s → frame 30。既定fpsで往復一致。
    let frame = frame_from_scrub_bar(1.0, fps_num, fps_den);
    assert_eq!(frame, 30);
    let terminal = try_dispatch_set_time(frame).expect("terminal");
    assert!(terminal.accepted);
    assert!(terminal.diagnostics.is_empty());
    assert_eq!(
        terminal.projection.as_ref().expect("snapshot").current_time,
        (1, 1)
    );
    assert_eq!(terminal.stamp(), Some((0, 1)));
    let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
    assert_eq!(after.current_time.num(), 1);
    assert_eq!(after.current_time.den(), 1);
    let ph =
        playhead_from_current_time(after.current_time.num(), after.current_time.den(), 10.0);
    assert!((ph - 1.0 / 10.0).abs() < 1e-12);

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn shuttle_reverse_keymap_steps_playhead_via_host_slot() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("shuttle-reverse-keymap");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    install_slot(host);

    assert!(try_dispatch_set_time(45).is_some_and(|result| result.accepted));
    let chars = b"j";
    let consumed = unsafe {
        motolii_rnapp_host_key_event(38, 0, chars.as_ptr(), chars.len(), false, false)
    };
    assert_eq!(
        consumed, 1,
        "J must reach host_key_event → try_dispatch_keymap"
    );
    let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
    assert_eq!(after.current_time.num(), 22);
    assert_eq!(after.current_time.den(), 15);

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn set_time_is_not_dispatched_without_host_slot() {
    let _lock = test_lock();
    clear_slot();
    assert!(try_dispatch_set_time(60).is_none());
}

#[test]
fn add_position_key_grows_wire_timeline_keys() {
    let _lock = test_lock();
    let path = temp_project("add-pos-key");
    let host = motolii_ui::host_create_for_test(&path).expect("create host");
    // placeでprimaryを載せ、add_position_keyはtarget+time必須(rn_product_host 718-747)。
    dispatch_kind(
        host,
        "place_rectangle",
        r#","position":[0.25,-0.125],"playhead":{"num":0,"den":1}"#,
    );
    let placed = motolii_ui::host_read_snapshot_for_test(host).expect("placed");
    let layer_id = placed.primary_layer_id.expect("primary after place");
    let before_keys = placed
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .map(|layer| layer.position_keys.len())
        .unwrap_or(0);

    dispatch_kind(
        host,
        "add_position_key",
        &format!(r#","target":"{layer_id}","time":{{"num":1,"den":1}}"#),
    );
    let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
    let layer = after
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("keyed layer");
    assert_eq!(layer.position_keys.len(), before_keys + 1);
    assert_eq!(layer.position_keys.last().unwrap().time.num(), 1);
    assert_eq!(layer.position_keys.last().unwrap().time.den(), 1);

    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

#[test]
fn timeline_edit_commit_remove_position_key_clears_wire_keys() {
    let _lock = test_lock();
    clear_slot();
    let path = temp_project("remove-pos-key-commit");
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
    let key_id = keyed
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
    let terminal = try_dispatch_timeline_edit(
        &crate::timeline_skia::TimelineEditCommit::RemovePositionKey {
            layer_id: layer_id.clone(),
            key_id,
        },
    )
    .expect("terminal");
    assert!(terminal.accepted);
    let terminal_layer = terminal
        .projection
        .as_ref()
        .and_then(|projection| projection.timeline_layers.as_ref())
        .and_then(|layers| layers.iter().find(|layer| layer.layer_id == layer_id))
        .expect("terminal layer");
    assert!(terminal_layer.position_keys.is_empty());
    let after = motolii_ui::host_read_snapshot_for_test(host).expect("after");
    let layer = after
        .timeline
        .layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .expect("layer");
    assert!(layer.position_keys.is_empty());

    clear_slot();
    motolii_ui::host_destroy_for_test(host).expect("destroy");
}

