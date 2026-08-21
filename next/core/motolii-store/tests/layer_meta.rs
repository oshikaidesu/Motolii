//! layer-meta 束(裁定108 の宿題 + 地図17行)。
//!
//! ここで固定するのは:
//! - `hd`(Hidden)は present(削除)とは別に「今フレームは描かない」を作れる
//! - `parent` は循環参照を作れない
//! - `bm`(Blend Mode)/matte が resolve へ運ばれる
//! - `nm`(Name)/`ao`(Auto Orient)が attrs に載る
//! - `lv`(Level)/`tm`(Time Remap)は普通の property track として効く
//! - `sr`(Time Stretch)が comp フレームの進みを比でスケールする
//! - null layer は絵を持たない
//! - shape-layer / text-layer は `LayerSource` の variant として存在し、中身は別 component
//! - effect インスタンスの列は id で同一性を持つ(masks と同型)
//! - `Intent::SetMeta` は新規配置専用で、既存 layer には使えない(裁定108(c) の柵)

use motolii_store::{
    BlendMode, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerAttrsPatch,
    LayerId, LayerMeta, LayerSource, LayerTiming, Matte, MatteMode, PropertyId, RationalTime,
    Shape, Speed, Value, property,
};

fn t(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn solid() -> LayerSource {
    LayerSource::Solid {
        rgba: [255, 0, 0, 255],
        width: 64,
        height: 64,
    }
}

fn place(doc: &mut Document, layer: LayerId, source: LayerSource, start: i64, duration: i64) {
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source,
                order: 0,
                timing: LayerTiming {
                    start,
                    duration,
                    source_in: 0,
                    ..Default::default()
                },
            },
        },
    ])
    .unwrap();
}

// ---------------------------------------------------------------------------
// hd (Hidden)
// ---------------------------------------------------------------------------

#[test]
fn hidden_layer_does_not_resolve_but_still_exists() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    assert!(doc.view().resolve(layer, t(0)).unwrap().is_some());

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    // hidden ≠ present=false。layer 自体はまだ在る(一覧に出る)。
    assert!(doc.view().has_layer(layer), "hidden で layer 自体が消えてはいけない");
    assert!(
        doc.view().resolve(layer, t(0)).unwrap().is_none(),
        "hidden な layer が描かれてしまっている"
    );
}

// ---------------------------------------------------------------------------
// parent — 循環参照を作れない
// ---------------------------------------------------------------------------

#[test]
fn parent_cannot_point_at_itself() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    let result = doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            parent: Some(Some(layer)),
            ..Default::default()
        },
    });
    assert!(result.is_err(), "layer が自分自身の親になれてしまっている");
}

#[test]
fn parent_chain_cannot_form_a_cycle() {
    let mut doc = doc_with_comp(300);
    let (a, b, c) = (LayerId(1), LayerId(2), LayerId(3));
    place(&mut doc, a, solid(), 0, 100);
    place(&mut doc, b, solid(), 0, 100);
    place(&mut doc, c, solid(), 0, 100);

    // a ← b ← c(c の親が b、b の親が a)。ここまでは正常。
    doc.apply(Intent::SetAttrs {
        layer: b,
        patch: LayerAttrsPatch {
            parent: Some(Some(a)),
            ..Default::default()
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: c,
        patch: LayerAttrsPatch {
            parent: Some(Some(b)),
            ..Default::default()
        },
    })
    .unwrap();

    // a の親を c にすると a→c→b→a の循環になる。
    let result = doc.apply(Intent::SetAttrs {
        layer: a,
        patch: LayerAttrsPatch {
            parent: Some(Some(c)),
            ..Default::default()
        },
    });
    assert!(result.is_err(), "親鎖の循環が作れてしまっている");

    // 循環にならない再配置(a の親を持たないままにする)は通る。
    assert!(doc.view().attrs(a).unwrap().is_none());
}

// ---------------------------------------------------------------------------
// bm (Blend Mode) / matte — resolve へ運ばれる
// ---------------------------------------------------------------------------

#[test]
fn blend_mode_and_matte_reach_the_resolved_layer() {
    let mut doc = doc_with_comp(300);
    let (base, top) = (LayerId(1), LayerId(2));
    place(&mut doc, base, solid(), 0, 100);
    place(&mut doc, top, solid(), 0, 100);

    doc.apply(Intent::SetAttrs {
        layer: top,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Multiply),
            matte: Some(Some(Matte {
                layer: base,
                mode: MatteMode::Luma,
            })),
            ..Default::default()
        },
    })
    .unwrap();

    let resolved = doc.view().resolve(top, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.blend_mode, BlendMode::Multiply);
    assert_eq!(
        resolved.matte,
        Some(Matte {
            layer: base,
            mode: MatteMode::Luma
        })
    );

    // 属性を一度も書いていない layer は既定(Normal / matte 無し)。
    let base_resolved = doc.view().resolve(base, t(0)).unwrap().expect("居る");
    assert_eq!(base_resolved.blend_mode, BlendMode::Normal);
    assert_eq!(base_resolved.matte, None);
}

/// **BL2**: `Add`(裁定67 が後回しにしていた値、`next/core/motolii-store/src/attrs.rs`
/// の `BlendMode` doc 参照)も他の値と同じ経路で `resolve()` まで届く。
#[test]
fn add_blend_mode_reaches_the_resolved_layer() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Add),
            ..Default::default()
        },
    })
    .unwrap();

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る");
    assert_eq!(resolved.blend_mode, BlendMode::Add);
}

// ---------------------------------------------------------------------------
// nm (Name) / ao (Auto Orient)
// ---------------------------------------------------------------------------

#[test]
fn name_and_auto_orient_round_trip_through_attrs() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            name: Some("背景".to_owned()),
            auto_orient: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let attrs = doc.view().attrs(layer).unwrap().expect("attrs が無い");
    assert_eq!(attrs.name, "背景");
    assert!(attrs.auto_orient);
}

/// 属性を触っても `meta`(素材・重ね順・配置)は一切変わらない — これが
/// 裁定108(c) の構造修正の核心(component が分かれているので巻き込めない)。
#[test]
fn setting_attrs_never_touches_timing_or_source() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    let timing = LayerTiming {
        start: 42,
        duration: 17,
        source_in: 3,
        ..Default::default()
    };
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 5,
                timing,
            },
        },
    ])
    .unwrap();

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            name: Some("x".to_owned()),
            ..Default::default()
        },
    })
    .unwrap();

    let meta = doc.view().meta(layer).unwrap().expect("meta が無い");
    assert_eq!(meta.timing, timing, "SetAttrs が timing を巻き込んだ");
    assert_eq!(meta.order, 5, "SetAttrs が order を巻き込んだ");
    assert_eq!(meta.source, solid(), "SetAttrs が source を巻き込んだ");
}

/// 敵対的レビュー(2026-08-20)が実証した事故の再現: `SetAttrs` が `attrs` を丸ごと
/// 差し替えていた頃は `SetAttrs{ hidden: true, ..Default::default() }` を書くだけで
/// name/blend_mode/auto_orient が黙って既定値へ戻った。`LayerAttrsPatch` は
/// 触らないフィールドを `None`(=無変更)にできるので、1フィールドだけの更新が
/// 他のフィールドを巻き込まないことをここで固定する。
#[test]
fn patching_one_field_leaves_the_others_alone() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            name: Some("背景".to_owned()),
            blend_mode: Some(BlendMode::Multiply),
            auto_orient: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    // 別の1フィールドだけを触る呼び出し。`..Default::default()` を使っても
    // 上で立てた3フィールドは無変更のまま残らなければならない。
    doc.apply(Intent::SetAttrs {
        layer,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let attrs = doc.view().attrs(layer).unwrap().expect("attrs が無い");
    assert!(attrs.hidden, "hidden が反映されていない");
    assert_eq!(attrs.name, "背景", "SetAttrs の別フィールド更新で name が消えた");
    assert_eq!(
        attrs.blend_mode,
        BlendMode::Multiply,
        "SetAttrs の別フィールド更新で blend_mode が消えた"
    );
    assert!(attrs.auto_orient, "SetAttrs の別フィールド更新で auto_orient が消えた");
}

/// `parent` を触らない `SetAttrs` は、既存の parent を黙って外さない
/// (丸ごと差し替え時代は `parent` も struct-update の既定 `None` に巻き込まれていた)。
#[test]
fn patching_unrelated_fields_does_not_clear_an_existing_parent() {
    let mut doc = doc_with_comp(300);
    let (a, b) = (LayerId(1), LayerId(2));
    place(&mut doc, a, solid(), 0, 100);
    place(&mut doc, b, solid(), 0, 100);

    doc.apply(Intent::SetAttrs {
        layer: b,
        patch: LayerAttrsPatch {
            parent: Some(Some(a)),
            ..Default::default()
        },
    })
    .unwrap();

    doc.apply(Intent::SetAttrs {
        layer: b,
        patch: LayerAttrsPatch {
            hidden: Some(true),
            ..Default::default()
        },
    })
    .unwrap();

    let attrs = doc.view().attrs(b).unwrap().expect("attrs が無い");
    assert_eq!(attrs.parent, Some(a), "無関係な更新で parent が外れた");
}

// ---------------------------------------------------------------------------
// SetMeta は新規配置専用(裁定108(c))
// ---------------------------------------------------------------------------

#[test]
fn set_meta_refuses_to_replace_an_existing_layers_meta() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    let result = doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: solid(),
            order: 9,
            timing: LayerTiming::default(),
        },
    });
    assert!(
        result.is_err(),
        "既存 layer に対して SetMeta が丸ごと差し替えを許してしまっている"
    );
}

#[test]
fn set_source_changes_only_the_source() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    let timing = LayerTiming::place(10, Some(50), 300);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 2,
                timing,
            },
        },
    ])
    .unwrap();

    let new_source = LayerSource::Solid {
        rgba: [0, 255, 0, 255],
        width: 64,
        height: 64,
    };
    doc.apply(Intent::SetSource {
        layer,
        source: new_source.clone(),
    })
    .unwrap();

    let meta = doc.view().meta(layer).unwrap().unwrap();
    assert_eq!(meta.source, new_source);
    assert_eq!(meta.timing, timing, "SetSource が timing を巻き込んだ");
    assert_eq!(meta.order, 2, "SetSource が order を巻き込んだ");
}

#[test]
fn set_order_changes_only_the_order() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    let timing = LayerTiming::place(0, Some(50), 300);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 2,
                timing,
            },
        },
    ])
    .unwrap();

    doc.apply(Intent::SetOrder { layer, order: 9 }).unwrap();

    let meta = doc.view().meta(layer).unwrap().unwrap();
    assert_eq!(meta.order, 9);
    assert_eq!(meta.timing, timing, "SetOrder が timing を巻き込んだ");
    assert_eq!(meta.source, solid(), "SetOrder が source を巻き込んだ");
}

// ---------------------------------------------------------------------------
// lv (Level) / tm (Time Remap) — 普通の property track
// ---------------------------------------------------------------------------

#[test]
fn level_is_a_plain_animatable_property() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    let property = PropertyId::new(property::LEVEL).unwrap();
    // track が無い間は「値が無い」— 何らかの既定(1.0)を Document が勝手に足さない。
    assert_eq!(doc.view().value_at(layer, &property, t(0)).unwrap(), None);

    let mut track = motolii_store::KeyframeTrack::new();
    track.insert(motolii_store::Keyframe {
        t: t(0),
        value: Value::F64(0.5),
        interp: motolii_store::Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: property.clone(),
        track,
    })
    .unwrap();

    assert_eq!(
        doc.view().value_at(layer, &property, t(0)).unwrap(),
        Some(Value::F64(0.5))
    );
}

#[test]
fn time_remap_overrides_the_normal_source_frame_mapping() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    // remap 無しなら通常どおり(covers している間は comp フレーム=素材フレーム)。
    let plain = doc.view().resolve(layer, t(10)).unwrap().expect("居る");
    assert_eq!(plain.source_frame, 10);

    let mut track = motolii_store::KeyframeTrack::new();
    track.insert(motolii_store::Keyframe {
        t: t(0),
        value: Value::F64(77.0),
        interp: motolii_store::Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer,
        property: PropertyId::new(property::TIME_REMAP).unwrap(),
        track,
    })
    .unwrap();

    let remapped = doc.view().resolve(layer, t(10)).unwrap().expect("居る");
    assert_eq!(remapped.source_frame, 77, "time_remap が効いていない");

    // ただし配置の外(covers していない時刻)は remap があっても描かない。
    assert!(doc.view().resolve(layer, t(500)).unwrap().is_none());
}

// ---------------------------------------------------------------------------
// sr (Time Stretch)
// ---------------------------------------------------------------------------

#[test]
fn speed_scales_how_fast_the_source_advances() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming {
                    start: 0,
                    duration: 100,
                    source_in: 0,
                    speed: Speed::try_new(2, 1).unwrap(),
                },
            },
        },
    ])
    .unwrap();

    // 2倍速 — comp が10進むと素材は20進む。
    let resolved = doc.view().resolve(layer, t(10)).unwrap().expect("居る");
    assert_eq!(resolved.source_frame, 20);
}

#[test]
fn speed_defaults_to_normal_and_matches_the_old_one_to_one_mapping() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);
    let resolved = doc.view().resolve(layer, t(10)).unwrap().expect("居る");
    assert_eq!(resolved.source_frame, 10);
}

/// 未クランプの記録(2026-08-20 の敵対的レビュー、`LayerTiming::source_frame` の
/// doc コメント参照)。負の `speed`(逆再生)で `source_in` が小さいと
/// `source_frame` は**負のまま**返る — store は 0 へ丸めない。この値をどう扱うか
/// (0 でクランプ / ループ / エラー)は engine 側の判断で store の仕事ではないが、
/// **黙った穴のままにはしない** — この試験が現状の(意図的に未クランプな)挙動を
/// 固定し、doc コメントの説明と食い違わないようにする。
#[test]
fn negative_speed_can_make_source_frame_go_negative_and_this_is_not_clamped_by_the_store() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: solid(),
                order: 0,
                timing: LayerTiming {
                    start: 0,
                    duration: 100,
                    source_in: 0,
                    speed: Speed::try_new(-1, 1).unwrap(),
                },
            },
        },
    ])
    .unwrap();

    let resolved = doc.view().resolve(layer, t(10)).unwrap().expect("居る");
    assert_eq!(
        resolved.source_frame, -10,
        "store が未クランプのままなら -10 のはず(engine 側の対処はここでは行わない)"
    );
}

// ---------------------------------------------------------------------------
// null-layer
// ---------------------------------------------------------------------------

#[test]
fn null_layer_resolves_but_has_no_declared_size() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, LayerSource::Null, 0, 100);

    let resolved = doc.view().resolve(layer, t(0)).unwrap().expect("居る(絵は無くても解決はする)");
    assert_eq!(resolved.source, LayerSource::Null);
    assert_eq!(resolved.declared_size, [0.0, 0.0]);
}

// ---------------------------------------------------------------------------
// shape-layer / text-layer
// ---------------------------------------------------------------------------

#[test]
fn shape_layer_holds_an_ordered_list_of_shapes() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, LayerSource::Shape, 0, 100);

    assert!(doc.view().shapes(layer).unwrap().is_empty());

    // `Shape`/`PathSource` の語彙は shape-1/2/3 が既に決めた `motolii-vector` の正本
    // (裁定10)。layer-meta 束はこれを Document へ乗せる線だけを足す。
    let shape = Shape::new(motolii_vector::PathSource::Rectangle {
        size: motolii_vector::Point { x: 10.0, y: 10.0 },
    });
    doc.apply(Intent::SetShapes {
        layer,
        shapes: vec![shape.clone()],
    })
    .unwrap();

    let shapes = doc.view().shapes(layer).unwrap();
    assert_eq!(shapes, vec![shape]);
}

#[test]
fn text_layer_exists_as_a_layer_source_variant_and_holds_no_document_until_set() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, LayerSource::Text, 0, 100);

    // `Layer:text` の中身(裁定112(k) の後継)は `text` 発注単位(未着手時点)の仕事。
    // ここでは layer-meta 束の関心である「`LayerSource::Text` という variant が存在し、
    // 中身は別 component」までを固定する。中身の意味(content/style/font)は
    // `tests/text.rs` が縛る。
    assert_eq!(doc.view().text_document(layer).unwrap(), None);
}

// ---------------------------------------------------------------------------
// ef (Effects) — id で同一性を持つ列
// ---------------------------------------------------------------------------

#[test]
fn effects_are_an_ordered_list_identified_by_id_not_index() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    assert!(doc.view().effects(layer).unwrap().is_empty());

    let effects = vec![
        EffectInstance {
            id: EffectId(1),
            plugin_id: "motolii.gaussian-blur".to_owned(),
            enabled: true,
        },
        EffectInstance {
            id: EffectId(2),
            plugin_id: "motolii.drop-shadow".to_owned(),
            enabled: false,
        },
    ];
    doc.apply(Intent::SetEffects {
        layer,
        effects: effects.clone(),
    })
    .unwrap();

    assert_eq!(doc.view().effects(layer).unwrap(), effects);
}

#[test]
fn duplicate_effect_ids_are_rejected() {
    let mut doc = doc_with_comp(300);
    let layer = LayerId(1);
    place(&mut doc, layer, solid(), 0, 100);

    let result = doc.apply(Intent::SetEffects {
        layer,
        effects: vec![
            EffectInstance {
                id: EffectId(1),
                plugin_id: "a".to_owned(),
                enabled: true,
            },
            EffectInstance {
                id: EffectId(1),
                plugin_id: "b".to_owned(),
                enabled: true,
            },
        ],
    });
    assert!(result.is_err(), "同じ id の effect が2枚置けてしまっている");
}
