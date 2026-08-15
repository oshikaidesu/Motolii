use super::super::{frame_from_scrub_bar, rational_time_parts_from_bar};
use super::super::parse_wire::parse_timeline_projection;

#[test]
fn frame_from_scrub_bar_rounds_at_fps_30_and_24_boundaries() {
    assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
    // 1s → fps30 frame 30、fps24 frame 24。
    assert_eq!(frame_from_scrub_bar(1.0, 30, 1), 30);
    assert_eq!(frame_from_scrub_bar(1.0, 24, 1), 24);
    // 0.5s → fps30 frame 15、fps24 frame 12。
    assert_eq!(frame_from_scrub_bar(0.5, 30, 1), 15);
    assert_eq!(frame_from_scrub_bar(0.5, 24, 1), 12);
    // 0.5 frame は最近傍で 1（24fps も同じ。µs先丸めだと 0 になっていた）
    assert_eq!(frame_from_scrub_bar(0.5 / 30.0, 30, 1), 1);
    assert_eq!(frame_from_scrub_bar(0.5 / 24.0, 24, 1), 1);
    // 直前は0
    assert_eq!(frame_from_scrub_bar(0.49 / 30.0, 30, 1), 0);
    assert_eq!(frame_from_scrub_bar(0.49 / 24.0, 24, 1), 0);
    assert_eq!(frame_from_scrub_bar(0.5, 30, 0), 0);
    assert_eq!(frame_from_scrub_bar(0.5, 0, 1), 0);
}

#[test]
fn rational_time_parts_from_bar_emits_seconds_not_ableton_bars() {
    assert_eq!(crate::timeline_skia::SECONDS_PER_BAR, 1.0);
    assert_eq!(rational_time_parts_from_bar(0.0), (0, 1_000_000));
    assert_eq!(rational_time_parts_from_bar(1.0), (1_000_000, 1_000_000));
    assert_eq!(rational_time_parts_from_bar(0.5), (500_000, 1_000_000));
    assert_eq!(rational_time_parts_from_bar(10.0), (10_000_000, 1_000_000));
}

#[test]
fn parse_stage_geometry_reads_corners() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
        "stage_geometry":{
            "layers":[{
                "layer_id":"1",
                "corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5]]
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    let geom = proj.stage_geometry.expect("stage_geometry");
    assert!(!geom.layers_truncated);
    assert_eq!(geom.layers.len(), 1);
    assert_eq!(geom.layers[0].layer_id, "1");
    assert_eq!(
        geom.layers[0].corners,
        [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
    );
}

#[test]
fn parse_stage_geometry_falls_back_to_none_when_layers_truncated_missing() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
        "stage_geometry":{
            "layers":[
                {"layer_id":"1","corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5]]}
            ]
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    assert!(proj.stage_geometry.is_none());
}

#[test]
fn parse_stage_geometry_falls_back_to_none_when_layers_truncated_is_not_bool() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
        "stage_geometry":{
            "layers":[
                {"layer_id":"1","corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5],[-0.5,0.5]]}
            ],
            "layers_truncated":"false"
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    assert!(proj.stage_geometry.is_none());
}

#[test]
fn parse_stage_geometry_falls_back_to_none_on_three_point_corners() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
        "stage_geometry":{
            "layers":[{
                "layer_id":"1",
                "corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,0.5]]
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    assert!(proj.stage_geometry.is_none());
}

#[test]
fn parse_stage_geometry_falls_back_to_none_with_infinite_corner() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
        "stage_geometry":{
            "layers":[{
                "layer_id":"1",
                "corners":[[-0.5,-0.5],[0.5,-0.5],[0.5,"inf"],[-0.5,0.5]]
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    assert!(proj.stage_geometry.is_none());
}

#[test]
fn parse_stage_geometry_falls_back_to_none_on_broken_corners() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"1","display_name":"r"}]},
        "stage_geometry":{
            "layers":[{
                "layer_id":"1",
                "corners":[[-0.5,-0.5],[0.5,-0.5]]
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    assert!(proj.stage_geometry.is_none());
    assert_eq!(proj.revision, "3");
}

