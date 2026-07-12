//! D1f: 未知`plugin_id`のロード警告と再保存でのパススルー(F-9 / ガード9の開く側)。

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    load_document, save_document, ClipSource, LoadWarning, PluginCatalog, PluginSlot, TrackItem,
};
use serde_json::{json, Value};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1f-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn unknown_plugin_json() -> Value {
    json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "layers": {
            "next": 2,
            "entries": [
                {"id": 0, "name": "src"},
                {"id": 1, "name": "fx"}
            ]
        },
        "track_ids": {
            "next": 1,
            "entries": [{"id": 0, "name": "V1"}]
        },
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "effects": [{
                        "plugin_id": "vendor.filter.echo",
                        "vendor_meta": {"rev": 3}
                    }],
                    "transform": {
                        "position": {"const": {"Vec2": [0.0, 0.0]}},
                        "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                        "scale": {"const": {"Vec2": [1.0, 1.0]}},
                        "rotation": {"const": {"F64": 0.0}}
                    }
                },
                "start": {"num": 0, "den": 1},
                "duration": {"num": 1, "den": 1},
                "source": {
                    "source": "plugin",
                    "plugin_id": "vendor.layer_source.procedural",
                    "future_flag": true,
                    "params": {
                        "seed": {"const": {"F64": 42.0}}
                    }
                }
            }]
        }]
    })
}

#[test]
fn unknown_plugin_ids_load_with_typed_warnings() {
    let catalog = PluginCatalog::reference_v1();
    let bytes = serde_json::to_vec(&unknown_plugin_json()).unwrap();
    let result = motolii_doc::load_document_bytes(&bytes, &catalog).unwrap();

    assert_eq!(result.warnings.len(), 2);
    assert_eq!(
        result.warnings[0],
        LoadWarning::UnknownEffectPlugin {
            plugin_id: "vendor.filter.echo".into(),
            layer_id: 0,
        }
    );
    assert_eq!(
        result.warnings[1],
        LoadWarning::UnknownLayerSourcePlugin {
            plugin_id: "vendor.layer_source.procedural".into(),
            layer_id: 0,
        }
    );
    assert!(result.document.validate().is_ok());
}

#[test]
fn known_reference_plugins_emit_no_warnings() {
    let input = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "layers": {
            "next": 1,
            "entries": [{"id": 0, "name": "src"}]
        },
        "track_ids": {
            "next": 1,
            "entries": [{"id": 0, "name": "V1"}]
        },
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "effects": [{
                        "plugin_id": "core.filter.tint"
                    }],
                    "transform": {
                        "position": {"const": {"Vec2": [0.0, 0.0]}},
                        "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                        "scale": {"const": {"Vec2": [1.0, 1.0]}},
                        "rotation": {"const": {"F64": 0.0}}
                    }
                },
                "start": {"num": 0, "den": 1},
                "duration": {"num": 1, "den": 1},
                "source": {
                    "source": "plugin",
                    "plugin_id": "core.layer_source.clear"
                }
            }]
        }]
    });
    let catalog = PluginCatalog::reference_v1();
    let bytes = serde_json::to_vec(&input).unwrap();
    let result = motolii_doc::load_document_bytes(&bytes, &catalog).unwrap();
    assert!(result.warnings.is_empty());
}

#[test]
fn unknown_plugin_parts_survive_save_reload_roundtrip() {
    let dir = unique_dir("roundtrip");
    let path = dir.join("doc.json");
    let catalog = PluginCatalog::reference_v1();

    fs::write(
        &path,
        serde_json::to_vec_pretty(&unknown_plugin_json()).unwrap(),
    )
    .unwrap();

    let first = load_document(&path, &catalog).unwrap();
    assert_eq!(first.warnings.len(), 2);

    save_document(&path, &first.document).unwrap();
    let second = load_document(&path, &catalog).unwrap();
    assert_eq!(second.warnings, first.warnings);

    let TrackItem::Clip(clip) = &second.document.tracks[0].items[0] else {
        panic!("expected clip");
    };
    assert_eq!(clip.envelope.effects[0].plugin_id, "vendor.filter.echo");
    assert_eq!(
        clip.envelope.effects[0].extra.get("vendor_meta"),
        Some(&json!({"rev": 3}))
    );
    let ClipSource::Plugin {
        plugin_id,
        extra,
        params,
        ..
    } = &clip.source
    else {
        panic!("expected plugin source");
    };
    assert_eq!(plugin_id, "vendor.layer_source.procedural");
    assert_eq!(extra.get("future_flag"), Some(&json!(true)));
    assert!(params.contains_key("seed"));

    let on_disk: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        on_disk["tracks"][0]["items"][0]["envelope"]["effects"][0]["plugin_id"],
        "vendor.filter.echo"
    );
    assert_eq!(
        on_disk["tracks"][0]["items"][0]["source"]["future_flag"],
        true
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn unknown_effect_in_nested_group_is_warned() {
    let input = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "layers": {
            "next": 2,
            "entries": [
                {"id": 0, "name": "grp"},
                {"id": 1, "name": "child"}
            ]
        },
        "track_ids": {
            "next": 1,
            "entries": [{"id": 0, "name": "V1"}]
        },
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "group",
                "envelope": {
                    "layer_id": 0,
                    "effects": [{"plugin_id": "vendor.filter.glow"}],
                    "transform": {
                        "position": {"const": {"Vec2": [0.0, 0.0]}},
                        "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                        "scale": {"const": {"Vec2": [1.0, 1.0]}},
                        "rotation": {"const": {"F64": 0.0}}
                    }
                },
                "children": [{
                    "kind": "clip",
                    "envelope": {
                        "layer_id": 1,
                        "transform": {
                            "position": {"const": {"Vec2": [0.0, 0.0]}},
                            "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                            "scale": {"const": {"Vec2": [1.0, 1.0]}},
                            "rotation": {"const": {"F64": 0.0}}
                        }
                    },
                    "start": {"num": 0, "den": 1},
                    "duration": {"num": 1, "den": 1},
                    "source": {"source": "plugin", "plugin_id": "core.layer_source.clear"}
                }]
            }]
        }]
    });
    let catalog = PluginCatalog::reference_v1();
    let doc: motolii_doc::Document = serde_json::from_value(input).unwrap();
    let warnings = motolii_doc::collect_plugin_warnings(&doc, &catalog);
    assert_eq!(
        warnings,
        vec![LoadWarning::UnknownEffectPlugin {
            plugin_id: "vendor.filter.glow".into(),
            layer_id: 0,
        }]
    );
}

#[test]
fn known_id_in_wrong_slot_is_wrong_kind_not_unknown() {
    // filter席にlayer_source既知ID → UnknownではなくWrongKind
    let input = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "layers": {
            "next": 1,
            "entries": [{"id": 0, "name": "src"}]
        },
        "track_ids": {
            "next": 1,
            "entries": [{"id": 0, "name": "V1"}]
        },
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "effects": [{
                        "plugin_id": "core.layer_source.clear"
                    }],
                    "transform": {
                        "position": {"const": {"Vec2": [0.0, 0.0]}},
                        "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                        "scale": {"const": {"Vec2": [1.0, 1.0]}},
                        "rotation": {"const": {"F64": 0.0}}
                    }
                },
                "start": {"num": 0, "den": 1},
                "duration": {"num": 1, "den": 1},
                "source": {
                    "source": "plugin",
                    "plugin_id": "core.filter.tint"
                }
            }]
        }]
    });
    let catalog = PluginCatalog::reference_v1();
    let result =
        motolii_doc::load_document_bytes(&serde_json::to_vec(&input).unwrap(), &catalog).unwrap();
    // ソートは (WrongKind, layer_id, plugin_id) — plugin_id辞書順
    assert_eq!(
        result.warnings,
        vec![
            LoadWarning::PluginIdWrongKind {
                plugin_id: "core.filter.tint".into(),
                layer_id: 0,
                expected: PluginSlot::LayerSource,
                actual: PluginSlot::Filter,
            },
            LoadWarning::PluginIdWrongKind {
                plugin_id: "core.layer_source.clear".into(),
                layer_id: 0,
                expected: PluginSlot::Filter,
                actual: PluginSlot::LayerSource,
            },
        ]
    );
}
