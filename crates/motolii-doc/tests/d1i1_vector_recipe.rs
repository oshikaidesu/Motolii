#![allow(deprecated)]

//! D1i-1: VectorRecipe 構造移動と旧 path_ops 拒否。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Clip, ClipSource, DocParam, Document, DocumentError, ItemEnvelope, LineJoin, PathOp,
    StandardShape, Track, TrackItem, TrimMode, VectorContent, VectorRecipe,
};
use serde_json::json;

fn valid_minimal_raster() -> Document {
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("a").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc
}

#[test]
fn vector_recipe_roundtrip() {
    let recipe = VectorRecipe {
        content: VectorContent::StandardShape {
            shape: StandardShape::Ellipse {
                width: DocParam::const_f64(1.0),
                height: DocParam::const_f64(0.5),
            },
        },
        modifiers: vec![
            PathOp::Offset {
                distance: DocParam::const_f64(0.01),
                line_join: LineJoin::Miter,
                miter_limit: 4.0,
            },
            PathOp::Trim {
                start: DocParam::const_f64(0.0),
                end: DocParam::const_f64(1.0),
                offset: DocParam::const_f64(0.0),
                mode: TrimMode::Sequential,
            },
        ],
    };
    let json = serde_json::to_value(&recipe).unwrap();
    let back: VectorRecipe = serde_json::from_value(json).unwrap();
    assert_eq!(recipe, back);
}

#[test]
fn raster_clip_has_no_modifiers_field() {
    // 型レベル: Asset/Plugin に modifiers は無い。コンパイルできることが証拠。
    let _ = ClipSource::asset_video_only(motolii_doc::AssetId::from_raw(0));
    let doc = valid_minimal_raster();
    assert!(doc.validate().is_ok());
    let v = serde_json::to_value(&doc).unwrap();
    let clip = &v["tracks"][0]["items"][0];
    assert!(clip.get("path_ops").is_none());
    assert!(clip["source"].get("recipe").is_none());
}

#[test]
fn serde_rejects_legacy_path_ops() {
    let doc = valid_minimal_raster();
    let mut clip_json = serde_json::to_value(&doc.tracks[0].items[0]).unwrap();
    clip_json["path_ops"] = json!([
        {
            "op": "trim",
            "start": {"const": {"F64": 0.0}},
            "end": {"const": {"F64": 1.0}},
            "offset": {"const": {"F64": 0.0}}
        }
    ]);
    let err = serde_json::from_value::<Clip>(clip_json).unwrap_err();
    assert!(
        err.to_string().contains("path_ops"),
        "expected path_ops reject, got {err}"
    );
}

#[test]
fn serde_rejects_legacy_path_ops_null() {
    // Option<JsonValue> だと null が不在と同じになり拒否を迂回するため、presence で弾く
    let doc = valid_minimal_raster();
    let mut clip_json = serde_json::to_value(&doc.tracks[0].items[0]).unwrap();
    clip_json["path_ops"] = json!(null);
    let err = serde_json::from_value::<Clip>(clip_json).unwrap_err();
    assert!(
        err.to_string().contains("path_ops"),
        "expected path_ops:null reject, got {err}"
    );
}

#[test]
fn serde_rejects_recipe_on_asset_source() {
    let err = serde_json::from_value::<ClipSource>(json!({
        "source": "asset",
        "asset": 0,
        "recipe": {
            "content": {
                "kind": "standard_shape",
                "shape": "rect",
                "width": {"const": {"F64": 1.0}},
                "height": {"const": {"F64": 1.0}}
            },
            "modifiers": [{"op": "offset", "distance": {"const": {"F64": 0.1}}}]
        }
    }))
    .unwrap_err();
    assert!(
        err.to_string().contains("unknown field") || err.to_string().contains("recipe"),
        "expected unknown-field reject for Asset+recipe, got {err}"
    );
}

#[test]
fn serde_rejects_modifiers_on_asset_source() {
    let err = serde_json::from_value::<ClipSource>(json!({
        "source": "asset",
        "asset": 0,
        "modifiers": []
    }))
    .unwrap_err();
    assert!(
        err.to_string().contains("unknown field") || err.to_string().contains("modifiers"),
        "expected unknown-field reject for Asset+modifiers, got {err}"
    );
}

#[test]
fn validate_rejects_video_as_svg_asset() {
    let mut doc = Document::new_v1();
    let video = doc.assets.allocate("clip", "video/mp4", "h").unwrap();
    let layer = doc.layers.allocate("v").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::SvgAsset { asset: video },
                    modifiers: vec![PathOp::Offset {
                        distance: DocParam::const_f64(0.01),
                        line_join: LineJoin::Miter,
                        miter_limit: 4.0,
                    }],
                },
            },
        })],
    });
    match doc.validate() {
        Err(DocumentError::WrongAssetType { got, .. }) => assert_eq!(got, "video/mp4"),
        other => panic!("expected WrongAssetType, got {other:?}"),
    }
}

#[test]
fn validate_accepts_svg_asset_type() {
    let mut doc = Document::new_v1();
    let svg = doc.assets.allocate("icon", "image/svg+xml", "h").unwrap();
    let layer = doc.layers.allocate("v").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::SvgAsset { asset: svg },
                    modifiers: vec![],
                },
            },
        })],
    });
    assert!(doc.validate().is_ok());
}

#[test]
fn validate_rejects_non_font_for_text_path() {
    let mut doc = Document::new_v1();
    let video = doc.assets.allocate("clip", "video/mp4", "h").unwrap();
    let layer = doc.layers.allocate("v").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::TextPath {
                        text: "hi".into(),
                        font_asset: video,
                    },
                    modifiers: vec![],
                },
            },
        })],
    });
    match doc.validate() {
        Err(DocumentError::WrongAssetType { got, .. }) => assert_eq!(got, "video/mp4"),
        other => panic!("expected WrongAssetType, got {other:?}"),
    }
}

#[test]
fn validate_accepts_font_ttf_for_text_path() {
    let mut doc = Document::new_v1();
    let font = doc.assets.allocate("face", "font/ttf", "h").unwrap();
    let layer = doc.layers.allocate("v").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: tid,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::TextPath {
                        text: "hi".into(),
                        font_asset: font,
                    },
                    modifiers: vec![],
                },
            },
        })],
    });
    assert!(doc.validate().is_ok());
}
