//! Document → Lottie JSON 書き出しの契約。
//!
//! 発注書の中心は「書けなかった物の一覧が空である」ことを検査できる形にすること
//! ([`motolii_export::UnsupportedForLottie`])——このファイルの各試験は「採用済の
//! この部分は書ける」ことを JSON の形で直接検査する(裁定189: 実行は任意、
//! `cargo check --tests` が検収線)。読み込み側(`next/` に存在しない)を経由した
//! 往復試験はできないので、JSON の形を schema(`next/reference/lottie.schema.json`)
//! の理解に照らして直接見る。

use motolii_core::{Fps, RationalTime};
use motolii_export::export_lottie;
use motolii_store::{
    property, Composition, Document, EffectId, EffectInstance, Intent, Interp, Keyframe,
    KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, Mask, MaskId,
    MaskMode, Matte, MatteMode, Path, PathVertex, PropertyId, PropertyLink, PropertySource,
    ShapeNode, SlotId, Value,
};
use motolii_vector::{Brush, Fill, FillRule, PathSource, Rgb, Shape as VecShape};

const W: u32 = 64;
const H: u32 = 48;
const FRAMES: i64 = 20;

fn fps() -> Fps {
    Fps::try_new(30, 1).unwrap()
}

fn base_document() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: fps(),
        duration_frames: FRAMES,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn add_layer(doc: &mut Document, layer: LayerId, source: LayerSource) {
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source,
            order: 0,
            timing: LayerTiming::place(0, None, FRAMES),
        },
    })
    .unwrap();
}

fn hold_track(pairs: &[(i64, f64)]) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    for (frame, value) in pairs {
        track.insert(Keyframe {
            t: RationalTime::try_from_frame(*frame, fps()).unwrap(),
            value: Value::F64(*value),
            interp: Interp::Hold,
            spatial: None,
        });
    }
    track
}

// ---------------------------------------------------------------------------
// composition + solid layer(基本の形)
// ---------------------------------------------------------------------------

#[test]
fn composition_and_solid_layer_export_with_no_unsupported_items() {
    let mut doc = base_document();
    let layer = LayerId(1);
    add_layer(
        &mut doc,
        layer,
        LayerSource::Solid {
            rgba: [255, 0, 128, 255],
            width: 16,
            height: 16,
        },
    );

    let out = export_lottie(&doc.view()).unwrap();
    assert!(
        out.unsupported.is_empty(),
        "採用済のみの Document なのに unsupported が出た: {:?}",
        out.unsupported
    );

    let json = out.json;
    assert_eq!(json["w"], W);
    assert_eq!(json["h"], H);
    assert_eq!(json["fr"], 30.0);
    assert_eq!(json["ip"], 0.0);
    assert_eq!(json["op"], FRAMES as f64);

    let layers = json["layers"].as_array().unwrap();
    assert_eq!(layers.len(), 1);
    let l0 = &layers[0];
    assert_eq!(l0["ty"], 1, "solid layer は ty=1");
    assert_eq!(l0["sw"], 16);
    assert_eq!(l0["sh"], 16);
    assert_eq!(l0["sc"], "#FF0080");
    assert_eq!(l0["ind"], 1);
    assert_eq!(l0["ip"], 0.0);
    assert_eq!(l0["op"], FRAMES as f64);

    // track を1本も打っていない transform は静的既定値になる(裁定20)。
    assert_eq!(l0["ks"]["o"], serde_json::json!({"a": 0, "k": 100.0}));
    assert_eq!(l0["ks"]["a"], serde_json::json!({"a": 0, "k": [0.0, 0.0]}));
    assert_eq!(l0["ks"]["s"], serde_json::json!({"a": 0, "k": [100.0, 100.0]}));
}

#[test]
fn animated_opacity_track_becomes_a_keyframed_lottie_property() {
    let mut doc = base_document();
    let layer = LayerId(1);
    add_layer(&mut doc, layer, LayerSource::Null);

    let property = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: hold_track(&[(0, 0.0), (5, 1.0)]),
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert!(out.unsupported.is_empty(), "{:?}", out.unsupported);

    let o = &out.json["layers"][0]["ks"]["o"];
    assert_eq!(o["a"], 1);
    let keys = o["k"].as_array().unwrap();
    assert_eq!(keys.len(), 2, "打った2キーがそのまま2キーになるはず: {keys:?}");
    assert_eq!(keys[0]["t"], 0.0);
    assert_eq!(keys[0]["s"], serde_json::json!([0.0]));
    assert_eq!(keys[0]["h"], 1, "Hold interp は h:1");
    assert_eq!(keys[1]["t"], 5.0);
    assert_eq!(keys[1]["s"], serde_json::json!([100.0]));
}

// ---------------------------------------------------------------------------
// link(裁定206 の実地検証) — 焼けば普通の track と区別が付かない
// ---------------------------------------------------------------------------

#[test]
fn property_link_bakes_into_a_normal_keyframed_property() {
    let mut doc = base_document();
    let source_layer = LayerId(1);
    let target_layer = LayerId(2);
    add_layer(&mut doc, source_layer, LayerSource::Null);
    add_layer(&mut doc, target_layer, LayerSource::Null);

    let opacity = PropertyId::new(property::OPACITY).unwrap();
    doc.apply(Intent::SetTrack {
        layer: source_layer,
        property: opacity.clone(),
        track: hold_track(&[(0, 0.0), (5, 1.0)]),
    })
    .unwrap();

    doc.apply(Intent::SetPropertyLink {
        layer: target_layer,
        property: opacity.clone(),
        link: PropertyLink {
            source_layer,
            source_property: opacity,
            time_offset: RationalTime::ZERO,
            plugin_id: "motolii.link.identity".to_owned(),
            params: Vec::new(),
        },
    })
    .unwrap();

    // 焼く前提の確認: `property_source` は Link のまま(track ではない)。
    let view = doc.view();
    match view
        .property_source(target_layer, &PropertyId::new(property::OPACITY).unwrap())
        .unwrap()
    {
        Some(PropertySource::Link(_)) => {}
        other => panic!("SetPropertyLink 直後は Link のはず: {other:?}"),
    }

    let out = export_lottie(&view).unwrap();
    assert!(
        out.unsupported.is_empty(),
        "link は焼けるので unsupported に積まれないはず: {:?}",
        out.unsupported
    );

    let target_json = &out.json["layers"][1];
    assert_eq!(target_json["ind"], 2);
    let o = &target_json["ks"]["o"];
    assert_eq!(o["a"], 1, "link を焼いた結果はキーフレーム化された普通の property");
    let keys = o["k"].as_array().unwrap();
    // 値が変わった時だけキーを打つ焼き方なので、frame0(0%)と frame5(100%)の2本。
    assert_eq!(keys.len(), 2, "焼いたキー列: {keys:?}");
    assert_eq!(keys[0]["t"], 0.0);
    assert_eq!(keys[0]["s"], serde_json::json!([0.0]));
    assert_eq!(keys[1]["t"], 5.0);
    assert_eq!(keys[1]["s"], serde_json::json!([100.0]));

    // 焼いた値は source をそのままサンプルしたのと一致する(identity link なので)。
    for frame in [0i64, 3, 5, 10, FRAMES - 1] {
        let t = RationalTime::try_from_frame(frame, fps()).unwrap();
        let expected = view.value_at(source_layer, &PropertyId::new(property::OPACITY).unwrap(), t)
            .unwrap()
            .and_then(|v| v.as_f64())
            .unwrap();
        let sampled = view.value_at(target_layer, &PropertyId::new(property::OPACITY).unwrap(), t)
            .unwrap()
            .and_then(|v| v.as_f64())
            .unwrap();
        assert_eq!(expected, sampled, "frame {frame} で link の評価値が source と一致しない");
    }
}

// ---------------------------------------------------------------------------
// matte(裁定66 → 裁定206 と同じ形: 内部は1フィールド、書き出しは tt/tp/td へ明示展開)
// ---------------------------------------------------------------------------

#[test]
fn matte_expands_into_explicit_tt_tp_td_fields() {
    let mut doc = base_document();
    let source_layer = LayerId(1);
    let target_layer = LayerId(2);
    add_layer(
        &mut doc,
        source_layer,
        LayerSource::Solid { rgba: [255, 255, 255, 255], width: 8, height: 8 },
    );
    add_layer(&mut doc, target_layer, LayerSource::Null);

    doc.apply(Intent::SetAttrs {
        layer: target_layer,
        patch: LayerAttrsPatch {
            matte: Some(Some(Matte {
                layer: source_layer,
                mode: MatteMode::Alpha,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert!(out.unsupported.is_empty(), "{:?}", out.unsupported);

    let source_json = &out.json["layers"][0];
    let target_json = &out.json["layers"][1];
    assert_eq!(target_json["tt"], 1, "MatteMode::Alpha は Lottie の 1");
    assert_eq!(target_json["tp"], source_json["ind"], "tp は明示的に source の ind を指す");
    assert_eq!(source_json["td"], 1, "matte の source 側は td:1");
    assert!(target_json.get("td").is_none(), "source でない側に td を立てない");
}

// ---------------------------------------------------------------------------
// masks
// ---------------------------------------------------------------------------

#[test]
fn mask_shape_and_opacity_export_as_bezier_and_scalar_properties() {
    let mut doc = base_document();
    let layer = LayerId(1);
    add_layer(&mut doc, layer, LayerSource::Shape);

    let mask_id = MaskId(1);
    let shape = Path {
        vertices: vec![
            PathVertex { point: [0.0, 0.0], in_tangent: [0.0, 0.0], out_tangent: [0.0, 0.0] },
            PathVertex { point: [10.0, 0.0], in_tangent: [0.0, 0.0], out_tangent: [0.0, 0.0] },
            PathVertex { point: [10.0, 10.0], in_tangent: [0.0, 0.0], out_tangent: [0.0, 0.0] },
        ],
        closed: true,
    };
    let mut shape_track = KeyframeTrack::new();
    shape_track.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::Path(shape),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::AddMask {
        layer,
        mask: Mask { id: mask_id, mode: MaskMode::Subtract, inverted: true },
        shape: shape_track,
    })
    .unwrap();
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::mask_opacity(mask_id),
        track: hold_track(&[(0, 0.5)]),
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert!(out.unsupported.is_empty(), "{:?}", out.unsupported);

    let masks = out.json["layers"][0]["masksProperties"].as_array().unwrap();
    assert_eq!(masks.len(), 1);
    let m = &masks[0];
    assert_eq!(m["mode"], "s", "MaskMode::Subtract は Lottie の \"s\"");
    assert_eq!(m["inv"], true);
    assert_eq!(m["o"], serde_json::json!({"a": 0, "k": 50.0}), "0.5(比)→50(%)");
    let pt = &m["pt"]["k"];
    assert_eq!(pt["c"], true);
    assert_eq!(pt["v"], serde_json::json!([[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]));
}

// ---------------------------------------------------------------------------
// shapes(静的 — fill/stroke/trim を1つの group にまとめて書けているか)
// ---------------------------------------------------------------------------

#[test]
fn shape_layer_exports_fill_and_geometry() {
    let mut doc = base_document();
    let layer = LayerId(1);
    add_layer(&mut doc, layer, LayerSource::Shape);

    let mut shape = VecShape::new(PathSource::Ellipse { size: motolii_vector::Point { x: 20.0, y: 20.0 } });
    shape.fill = Some(Fill {
        brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }),
        rule: FillRule::NonZero,
        opacity: 0.5,
        hidden: false,
    });
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![ShapeNode::Leaf(shape)],
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert!(out.unsupported.is_empty(), "{:?}", out.unsupported);

    let shapes = out.json["layers"][0]["shapes"].as_array().unwrap();
    assert_eq!(shapes.len(), 1);
    let items = shapes[0]["it"].as_array().unwrap();
    let ellipse = items.iter().find(|i| i["ty"] == "el").expect("ellipse item が無い");
    assert_eq!(ellipse["s"]["k"], serde_json::json!([20.0, 20.0]));
    let fill = items.iter().find(|i| i["ty"] == "fl").expect("fill item が無い");
    assert_eq!(fill["c"]["k"], serde_json::json!([1.0, 0.0, 0.0]));
    assert_eq!(fill["o"]["k"], 50.0, "0.5 → 50%");
}

// ---------------------------------------------------------------------------
// unsupported の報告(黙って落とさない)
// ---------------------------------------------------------------------------

#[test]
fn effect_instance_is_reported_as_unsupported_not_silently_dropped() {
    let mut doc = base_document();
    let layer = LayerId(1);
    add_layer(&mut doc, layer, LayerSource::Null);
    doc.apply(Intent::SetEffects {
        layer,
        effects: vec![EffectInstance {
            id: EffectId(1),
            plugin_id: "motolii.glow".to_owned(),
            enabled: true,
        }],
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert_eq!(out.unsupported.len(), 1);
    assert_eq!(out.unsupported[0].category, "effect");
    assert_eq!(out.unsupported[0].layer, Some(layer));
    assert!(out.unsupported[0].detail.contains("motolii.glow"));
    // JSON 自体は壊れずに出る(effect が無いことにして黙って落とすのではなく、
    // layer 自体は書いた上で unsupported を別途報告する)。
    assert_eq!(out.json["layers"].as_array().unwrap().len(), 1);
}

#[test]
fn camera_usage_is_reported_as_unsupported() {
    let mut doc = base_document();
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::F64(2.0),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetCameraTrack {
        property: PropertyId::camera(property::CAMERA_ZOOM).unwrap(),
        track,
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert_eq!(out.unsupported.len(), 1);
    assert_eq!(out.unsupported[0].category, "camera");
    assert_eq!(out.unsupported[0].layer, None);
}

#[test]
fn slot_referenced_property_exports_as_sid_reference() {
    let mut doc = base_document();
    let layer = LayerId(1);
    add_layer(&mut doc, layer, LayerSource::Null);

    doc.apply(Intent::SetSlots {
        slots: vec![motolii_store::Slot {
            id: SlotId("primary_opacity".to_owned()),
            track: hold_track(&[(0, 0.75)]),
        }],
    })
    .unwrap();
    doc.apply(Intent::SetPropertySlot {
        layer,
        property: PropertyId::new(property::OPACITY).unwrap(),
        slot: SlotId("primary_opacity".to_owned()),
    })
    .unwrap();

    let out = export_lottie(&doc.view()).unwrap();
    assert!(out.unsupported.is_empty(), "{:?}", out.unsupported);
    assert_eq!(
        out.json["layers"][0]["ks"]["o"],
        serde_json::json!({"sid": "primary_opacity"})
    );
    assert_eq!(
        out.json["slots"]["primary_opacity"]["p"],
        serde_json::json!({"a": 0, "k": 0.75})
    );
}
