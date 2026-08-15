use super::super::parse_catalog::parse_catalog_projection;
use super::super::parse_wire::parse_timeline_projection;

#[test]
fn parse_catalog_and_layer_effects_from_wire_json() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "primary_layer_id":"11",
        "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"L"}]},
        "catalog":{
            "effects":[
                {"plugin_id":"core.filter.opacity","name":"Opacity","effect_version":1},
                {"plugin_id":"core.param.sine","name":"Sine","effect_version":2}
            ],
            "sources":[
                {"plugin_id":"core.layer_source.radial_repeater","name":"Radial Repeater","effect_version":1}
            ]
        },
        "timeline":{
            "fps":{"num":30,"den":1},
            "layers":[{
                "layer_id":"11",
                "display_name":"L",
                "start":{"num":0,"den":1},
                "duration":{"num":10,"den":1},
                "position_keys":[],
                "keys_truncated":false,
                "effects":[{
                    "effect_use_id":"7",
                    "plugin_id":"core.filter.opacity",
                    "params":[{"param_id":"amount","value":0.5}]
                }],
                "effects_truncated":false,
                "source_params":[{"param_id":"count","value":12.0}],
                "source_params_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let catalog = parse_catalog_projection(json).expect("catalog");
    assert_eq!(catalog.effects.len(), 2);
    assert_eq!(catalog.effects[0].plugin_id, "core.filter.opacity");
    assert_eq!(catalog.effects[0].name, "Opacity");
    assert_eq!(catalog.effects[0].effect_version, 1);
    assert_eq!(catalog.sources.len(), 1);
    assert_eq!(
        catalog.sources[0].plugin_id,
        "core.layer_source.radial_repeater"
    );
    assert_eq!(catalog.sources[0].name, "Radial Repeater");
    let proj = parse_timeline_projection(json).expect("parse");
    let layers = proj.timeline_layers.expect("layers");
    assert_eq!(layers[0].effects.len(), 1);
    assert_eq!(layers[0].effects[0].effect_use_id, "7");
    assert_eq!(layers[0].effects[0].params[0].value, 0.5);
    assert!(!layers[0].effects_truncated);
    assert_eq!(layers[0].source_params.len(), 1);
    assert_eq!(layers[0].source_params[0].param_id, "count");
    assert_eq!(layers[0].source_params[0].value, 12.0);
    assert!(!layers[0].source_params_truncated);
}

#[test]
fn parse_timeline_effects_respects_truncated_flag_after_eight_entries() {
    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"L"}]},
        "timeline":{
            "fps":{"num":30,"den":1},
            "layers":[{
                "layer_id":"11",
                "display_name":"L",
                "start":{"num":0,"den":1},
                "duration":{"num":10,"den":1},
                "position_keys":[],
                "keys_truncated":false,
                "effects":[
                    {"effect_use_id":"e0","plugin_id":"p0","params":[]},
                    {"effect_use_id":"e1","plugin_id":"p1","params":[]},
                    {"effect_use_id":"e2","plugin_id":"p2","params":[]},
                    {"effect_use_id":"e3","plugin_id":"p3","params":[]},
                    {"effect_use_id":"e4","plugin_id":"p4","params":[]},
                    {"effect_use_id":"e5","plugin_id":"p5","params":[]},
                    {"effect_use_id":"e6","plugin_id":"p6","params":[]},
                    {"effect_use_id":"e7","plugin_id":"p7","params":[]}
                ],
                "effects_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    let layers = proj.timeline_layers.expect("layers");
    assert_eq!(layers[0].effects.len(), 8);
    assert_eq!(layers[0].effects[7].effect_use_id, "e7");
    assert!(!layers[0].effects_truncated);

    let json = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[{"layer_id":"11","display_name":"L"}]},
        "timeline":{
            "fps":{"num":30,"den":1},
            "layers":[{
                "layer_id":"11",
                "display_name":"L",
                "start":{"num":0,"den":1},
                "duration":{"num":10,"den":1},
                "position_keys":[],
                "keys_truncated":false,
                "effects":[
                    {"effect_use_id":"e0","plugin_id":"p0","params":[]},
                    {"effect_use_id":"e1","plugin_id":"p1","params":[]},
                    {"effect_use_id":"e2","plugin_id":"p2","params":[]},
                    {"effect_use_id":"e3","plugin_id":"p3","params":[]},
                    {"effect_use_id":"e4","plugin_id":"p4","params":[]},
                    {"effect_use_id":"e5","plugin_id":"p5","params":[]},
                    {"effect_use_id":"e6","plugin_id":"p6","params":[]},
                    {"effect_use_id":"e7","plugin_id":"p7","params":[]},
                    {"effect_use_id":"e8","plugin_id":"p8","params":[]}
                ],
                "effects_truncated":true
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(json).expect("parse");
    let layers = proj.timeline_layers.expect("layers");
    assert_eq!(layers[0].effects.len(), 9);
    assert_eq!(layers[0].effects[7].effect_use_id, "e7");
    assert_eq!(layers[0].effects[8].effect_use_id, "e8");
    assert!(layers[0].effects_truncated);
}

#[test]
fn parse_catalog_and_effects_fall_back_on_broken_values() {
    let broken_catalog = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[]},
        "catalog":{"effects":[{"plugin_id":1}]},
        "timeline":{"fps":{"num":30,"den":1},"layers":[],"layers_truncated":false},
        "diagnostics":[]
    }"#;
    assert!(parse_catalog_projection(broken_catalog).is_none());
    assert!(parse_timeline_projection(broken_catalog).is_some());

    let broken_effects = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[]},
        "timeline":{
            "fps":{"num":30,"den":1},
            "layers":[{
                "layer_id":"11",
                "display_name":"L",
                "start":{"num":0,"den":1},
                "duration":{"num":10,"den":1},
                "position_keys":[],
                "keys_truncated":false,
                "effects":[{"effect_use_id":"7","plugin_id":"x","params":"bad"}],
                "effects_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(broken_effects).expect("parse");
    let layers = proj.timeline_layers.expect("layers kept");
    assert!(layers[0].effects.is_empty());
    assert!(!layers[0].effects_truncated);

    let broken_sources = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[]},
        "catalog":{
            "effects":[{"plugin_id":"core.filter.opacity","name":"Opacity","effect_version":1}],
            "sources":[{"plugin_id":1}]
        },
        "timeline":{"fps":{"num":30,"den":1},"layers":[],"layers_truncated":false},
        "diagnostics":[]
    }"#;
    let catalog = parse_catalog_projection(broken_sources).expect("catalog kept");
    assert_eq!(catalog.effects.len(), 1);
    assert_eq!(catalog.effects[0].plugin_id, "core.filter.opacity");
    assert!(catalog.sources.is_empty());

    let broken_source_params = r#"{
        "version":1,
        "direction":"host-to-rn",
        "role":"product",
        "host_handle":"1",
        "revision":"3",
        "projection_generation":"0",
        "current_time":{"num":0,"den":1},
        "stage":{"selection":[],"bounds":[]},
        "timeline":{
            "fps":{"num":30,"den":1},
            "layers":[{
                "layer_id":"11",
                "display_name":"L",
                "start":{"num":0,"den":1},
                "duration":{"num":10,"den":1},
                "position_keys":[],
                "keys_truncated":false,
                "effects":[],
                "effects_truncated":false,
                "source_params":"bad",
                "source_params_truncated":false
            }],
            "layers_truncated":false
        },
        "diagnostics":[]
    }"#;
    let proj = parse_timeline_projection(broken_source_params).expect("parse");
    let layers = proj.timeline_layers.expect("layers kept");
    assert!(layers[0].source_params.is_empty());
    assert!(!layers[0].source_params_truncated);
}
