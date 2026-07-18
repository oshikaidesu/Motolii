#![allow(deprecated)]

//! D1f: 未知plugin_id・既知プラグインの未来版の「開く」側契約(F-9、実装ガード9、S13)。
//!
//! 完了条件: 未知plugin_idを含むJSONがロード成功+警告+roundtrip保持。既知プラグインの
//! 未来版も同じ契約(downgrade errorにしない)。plugin kindの違いは型付きエラー。

use std::collections::BTreeMap;

use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, DocParam, Document, DocumentPluginError, EffectDefinition,
    EffectDefinitionId, EffectId, EffectUse, ItemEnvelope, PluginDiagnosticReason, Track,
    TrackItem,
};
use motolii_plugins_firstparty::first_party_catalog;
use serde_json::{json, Map};

fn minimal_asset_clip_doc() -> (Document, motolii_doc::LayerId) {
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("layer").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    (doc, layer)
}

#[test]
fn unknown_effect_plugin_id_loads_warns_and_roundtrips() {
    let (mut doc, _layer) = minimal_asset_clip_doc();
    let mut extra = Map::new();
    extra.insert("vendor_flag".into(), json!(true));
    let def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        def_id,
        "vendor.filter.glow_deluxe",
        3,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        extra,
    ));
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        let eid = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
        clip.envelope.effects.push(EffectUse {
            id: eid,
            definition_id: def_id,
        });
    }

    // 1. load成功 = validateが通る(D1a/D1hの拒否対象ではない)
    doc.version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.min_reader_version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.validate()
        .expect("unknown plugin_id must not fail validate (open side)");

    // 2. 診断: 未知idとしてprepared解決に現れる
    let resolved = doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap();
    let warnings = resolved.diagnostics();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(warnings[0].plugin_id, "vendor.filter.glow_deluxe");
    assert!(matches!(
        warnings[0].reason,
        PluginDiagnosticReason::ContractMissing
    ));

    // 3. roundtrip保持: params/extraが無変更のまま残る(pass-through)
    let json = serde_json::to_string(&doc).unwrap();
    let back: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(back, doc);
    let def = back.effect_definition(def_id).expect("definition kept");
    assert_eq!(def.plugin_id, "vendor.filter.glow_deluxe");
    assert_eq!(def.effect_version, 3);
    assert_eq!(def.params.get("amount"), Some(&DocParam::const_f64(0.5)));
    assert_eq!(def.extra.get("vendor_flag"), Some(&json!(true)));
}

#[test]
fn unknown_clip_source_plugin_id_loads_warns_and_roundtrips() {
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("layer").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut extra = Map::new();
    extra.insert("future_field".into(), json!([1, 2, 3]));
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: "vendor.layer_source.particles".into(),
                effect_version: 1,
                params: BTreeMap::from([("seed".into(), DocParam::const_f64(42.0))]),
                extra,
            },
        })],
    });

    doc.validate().expect("unknown plugin source must open");
    let resolved = doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap();
    let warnings = resolved.diagnostics();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(matches!(
        warnings[0].reason,
        PluginDiagnosticReason::ContractMissing
    ));

    let back: Document = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
    assert_eq!(back, doc);
}

#[test]
fn known_plugin_future_version_is_degraded_not_a_downgrade_error() {
    // core.filter.opacity は現行 version 1(motolii-plugin-opacity 側 NodeDesc.version と同期)。
    // 未来版(2)を参照しても、migrate downgrade errorではなく未知プラグインと同じ契約になる(S13)。
    let (mut doc, _layer) = minimal_asset_clip_doc();
    let def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        def_id,
        "core.filter.opacity",
        2,
        true,
        // v1のパラメータ表(amount: F64)と食い違う値でも、degraded=構造検査のみなので通る。
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.9))]),
        Map::new(),
    ));
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        let eid = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
        clip.envelope.effects.push(EffectUse {
            id: eid,
            definition_id: def_id,
        });
    }

    doc.version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.min_reader_version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.validate()
        .expect("future effect_version must not be a hard error");
    let resolved = doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap();
    let warnings = resolved.diagnostics();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(warnings[0].plugin_id, "core.filter.opacity");
    assert_eq!(
        warnings[0].reason,
        PluginDiagnosticReason::FutureVersion {
            current_version: 1,
            saved_version: 2,
        }
    );
}

#[test]
fn known_plugin_current_version_has_no_warning() {
    let (mut doc, _layer) = minimal_asset_clip_doc();
    let def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        def_id,
        "core.filter.opacity",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.9))]),
        Map::new(),
    ));
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        let eid = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
        clip.envelope.effects.push(EffectUse {
            id: eid,
            definition_id: def_id,
        });
    }
    doc.version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.min_reader_version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.validate().unwrap();
    assert!(doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap()
        .diagnostics()
        .is_empty());
}

#[test]
fn plugin_kind_mismatch_in_effect_slot_is_typed_error() {
    // core.layer_source.clear は LayerSource 種別。effects(Filter専用スロット)に置くのは
    // degradeで救う「未知/未来版」ではなく構造上のバグ — 型付きエラーで拒否する。
    let (mut doc, _layer) = minimal_asset_clip_doc();
    let def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        def_id,
        "core.layer_source.clear",
        1,
        true,
        BTreeMap::new(),
        Map::new(),
    ));
    if let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] {
        let eid = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
        clip.envelope.effects.push(EffectUse {
            id: eid,
            definition_id: def_id,
        });
    }
    doc.version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.min_reader_version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.validate()
        .expect("intrinsic validation does not know plugin kinds");
    let err = doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap_err();
    assert!(
        matches!(
            err,
            DocumentPluginError::KindMismatch {
                expected: motolii_plugin::PluginKind::Filter,
                actual: motolii_plugin::PluginKind::LayerSource,
                ..
            }
        ),
        "{err:?}"
    );
    // kind不一致はwarningの対象ではない(validateが先にエラーで拒否する)。
}

#[test]
fn plugin_kind_mismatch_in_clip_source_slot_is_typed_error() {
    // core.filter.opacity は Filter 種別。ClipSource::Plugin(LayerSourceスロット)は誤り。
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("layer").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Plugin {
                plugin_id: "core.filter.opacity".into(),
                effect_version: 1,
                params: BTreeMap::new(),
                extra: Map::new(),
            },
        })],
    });
    doc.validate()
        .expect("intrinsic validation does not know plugin kinds");
    let err = doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap_err();
    assert!(
        matches!(
            err,
            DocumentPluginError::KindMismatch {
                expected: motolii_plugin::PluginKind::LayerSource,
                actual: motolii_plugin::PluginKind::Filter,
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn raw_json_with_unknown_plugin_id_and_future_version_loads_and_preserves_extra() {
    // 実際のJSON経由(load_document_bytes相当)でも同じ契約が成立することを固定する。
    let input = json!({
        "version": 4,
        "min_reader_version": 4,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "layers": {"next": 1, "entries": [{"id": 0, "name": "L"}]},
        "track_ids": {"next": 1, "entries": [{"id": 0, "name": "V1"}]},
        "next_stable_id": 4,
        "effect_definitions": [
            {
                "id": 2,
                "plugin_id": "vendor.filter.mystery",
                "effect_version": 7,
                "params": {
                    "strength": {"const": {"F64": 0.25}}
                },
                "vendor_only_field": {"nested": [1, 2]}
            },
            {
                "id": 3,
                "plugin_id": "core.filter.opacity",
                "effect_version": 99
            }
        ],
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "effects": [
                        {"id": 0, "definition_id": 2},
                        {"id": 1, "definition_id": 3}
                    ],
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
                    "plugin_id": "vendor.layer_source.mystery_gen",
                    "effect_version": 1
                }
            }]
        }]
    });

    let doc: Document = serde_json::from_value(input).expect("load must succeed (open side)");
    doc.validate()
        .expect("validate must accept unknown plugin_id/kind combos here");

    let resolved = doc
        .prepare_plugins(&first_party_catalog().unwrap())
        .unwrap();
    let warnings = resolved.diagnostics();
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert!(warnings
        .iter()
        .any(|w| w.plugin_id == "vendor.filter.mystery"
            && matches!(w.reason, PluginDiagnosticReason::ContractMissing)));
    assert!(warnings.iter().any(|w| w.plugin_id == "core.filter.opacity"
        && w.reason
            == PluginDiagnosticReason::FutureVersion {
                current_version: 1,
                saved_version: 99,
            }));
    assert!(warnings
        .iter()
        .any(|w| w.plugin_id == "vendor.layer_source.mystery_gen"
            && matches!(w.reason, PluginDiagnosticReason::ContractMissing)));

    // 再保存(pass-through評価・無変更保持): 未知フィールドを含め完全一致する。
    let roundtrip: Document = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
    assert_eq!(roundtrip, doc);
    let def = roundtrip
        .effect_definition(EffectDefinitionId::from_raw(2))
        .expect("definition kept");
    assert_eq!(
        def.extra.get("vendor_only_field"),
        Some(&json!({"nested": [1, 2]}))
    );
}
