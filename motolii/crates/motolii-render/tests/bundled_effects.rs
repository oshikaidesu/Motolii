//! 同梱の効果が、契約どおりに振る舞うか。効果はこの家の物なので、検査もここに置く
//! (コアは誰が何を実装しているか知らないので、コアの検査には書けない)。


use motolii_doc::store::{
    property, Composition, EffectScope, Fps, Interp, Keyframe, KeyframeTrack, PropertyId,
    StoreView, Value,
};

fn at(frame: i64) -> RationalTime {
    RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap()
}

fn position() -> PropertyId {
    PropertyId::new(property::POSITION).unwrap()
}

/// 同梱の効果を持つ作品。コアの既定は効果ゼロなので、効果の検査は自分で渡す。
fn document() -> Document {
    let mut doc = Document::new().with_programs(motolii_render::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 480,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    doc
}

fn layer(doc: &mut Document, id: u64, source: LayerSource) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(30, Some(90), 300) } },
    ])
    .unwrap();
    layer
}

fn put(doc: &mut Document, layer: LayerId, value: [f64; 2]) {
    doc.apply(Intent::SetConstant { layer, property: position(), value: Value::Vec2(value) }).unwrap();
}
use motolii_doc::store::{
    EffectId, EffectInstance, LayerId, LayerMeta, LayerSource, LayerTiming,
    Placement, RationalTime,
};
#[test]
fn motion_sampling_preserves_read_authority_and_effect_order() {
    use motolii_doc::store::{property, Interp, Keyframe, KeyframeTrack, PropertyId, Value};
    let mut doc = blank_project().with_programs(motolii_render::extensions::bundled());
    let layer = LayerId(1);
    let track = KeyframeTrack::try_from_keys(vec![
        Keyframe {
            t: RationalTime::ZERO,
            value: Value::Vec2([0.0, 0.0]),
            interp: Interp::Linear,
            spatial: None,
        },
        Keyframe {
            t: RationalTime::try_new(1, 1).unwrap(),
            value: Value::Vec2([300.0, 0.0]),
            interp: Interp::Linear,
            spatial: None,
        },
    ])
    .unwrap();
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: 0,
                timing: LayerTiming::place(0, None, 60),
            },
        },
        Intent::SetShapes {
            layer,
            shapes: vec![motolii_doc::store::rect_shape([255; 4], [10.0; 2])],
        },
        Intent::SetTrack {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            track,
        },
        Intent::SetEffects {
            layer,
            effects: vec![
                EffectInstance {
                    id: EffectId(1),
                    plugin_id: motolii_render::extensions::motion::MOTION_BLUR.into(),
                },
                EffectInstance {
                    id: EffectId(2),
                    plugin_id: "motolii.gain".into(),
                },
            ],
        },
    ])
    .unwrap();
    let revision = doc.revision();
    let history = doc.history_depth();
    let time = RationalTime::try_new(1, 2).unwrap();
    let samples = motolii_render::picture::resolve::resolved_layers(&doc.view(), time).unwrap();
    assert_eq!(samples.len(), 7);
    assert!(samples
        .iter()
        .all(|sample| sample.averaged == 7 && sample.effects.is_empty()));
    assert!(samples.iter().all(|sample| sample.after_effects.len() == 1
        && sample.after_effects[0].plugin_id == "motolii.gain"));
    assert!(
        (samples
            .iter()
            .map(|sample| sample.placement.opacity)
            .sum::<f32>()
            - 1.0)
            .abs()
            < 0.00001
    );
    assert_eq!(doc.revision(), revision);
    assert_eq!(doc.history_depth(), history);
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::effect_param(EffectId(1), "tune").unwrap(),
        value: Value::F64(0.0),
    })
    .unwrap();
    let still = motolii_render::picture::resolve::resolved_layers(&doc.view(), time).unwrap();
    assert_eq!(still.len(), 1);
    assert_eq!(still[0].averaged, 0);
    assert_eq!(still[0].placement.opacity, 1.0);
}

#[test]
fn blob_group_uses_sparse_identity_without_drawing_unplaced_children() {
    use motolii_doc::store::{
        analysis::{AnalysisInputs, BlobMark},
        LayerAttrsPatch,
    };
    let mut doc = blank_project().with_programs(motolii_render::extensions::bundled());
    let group = LayerId(1);
    let child = LayerId(2);
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta {
            layer: group,
            meta: LayerMeta {
                source: LayerSource::Group,
                order: 0,
                timing: LayerTiming::place(0, None, 10),
            },
        },
        Intent::AddLayer(child),
        Intent::SetMeta {
            layer: child,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: 1,
                timing: LayerTiming::place(0, None, 10),
            },
        },
        Intent::SetShapes {
            layer: child,
            shapes: vec![motolii_doc::store::rect_shape([255; 4], [10.0; 2])],
        },
        Intent::SetAttrs {
            layer: child,
            patch: LayerAttrsPatch {
                parent: Some(Some(group)),
                ..Default::default()
            },
        },
        Intent::SetEffects {
            layer: group,
            effects: vec![EffectInstance {
                id: EffectId(1),
                plugin_id: motolii_render::extensions::blob::BLOB_TRACK.into(),
            }],
        },
    ])
    .unwrap();
    let revision = doc.revision();
    let mut analysis = AnalysisInputs::default();
    analysis.set_blobs(
        group,
        EffectId(0),
        RationalTime::ZERO,
        vec![
            BlobMark {
                id: 7,
                center: [100.0, 100.0],
                size: [20.0; 2],
                age: 0,
            },
            BlobMark {
                id: 42,
                center: [200.0, 100.0],
                size: [20.0; 2],
                age: 1,
            },
        ],
    );
    assert!(motolii_render::picture::resolve::resolved_layers(&doc.view(), RationalTime::ZERO)
        .unwrap()
        .is_empty());
    let placed = motolii_render::picture::resolve::resolved_layers(&doc.view().with_analysis(&analysis), RationalTime::ZERO)
        .unwrap();
    assert_eq!(placed.len(), 2);
    assert!(placed.iter().all(|copy| copy.id == child));
    assert_eq!(
        placed.iter().map(|copy| copy.copy).collect::<Vec<_>>(),
        [7, 42]
    );
    assert_eq!(doc.revision(), revision);
    doc.apply(Intent::SetConstant {
        layer: group,
        property: motolii_doc::store::PropertyId::effect_enabled(EffectId(1)),
        value: motolii_doc::store::Value::Bool(false),
    })
    .unwrap();
    let disabled = motolii_render::picture::resolve::resolved_layers(&doc.view().with_analysis(&analysis), RationalTime::ZERO)
        .unwrap();
    let children: Vec<_> = disabled.iter().filter(|copy| copy.id == child).collect();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].copy, 0);
}

#[test]
fn a_placement_effect_multiplies_the_layer_and_delays_later_copies() {
    use motolii_render::extensions::placement;
    use motolii_doc::store::{EffectId, EffectInstance};
    let mut doc = document();
    let paper = layer(&mut doc, 1, LayerSource::Shape);
    put(&mut doc, paper, [100.0, 50.0]);
    let repeat = EffectId(0);
    doc.apply_all([
        Intent::SetEffects {
            layer: paper,
            effects: vec![
                EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() },
                EffectInstance { id: EffectId(1), plugin_id: "motolii.blur".to_owned() },
            ],
        },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(3.0) },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([20.0, 0.0]) },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "opacity_each").unwrap(), value: Value::F64(-0.25) },
        Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "delay_each").unwrap(), value: Value::F64(1.0) },
    ])
    .unwrap();

    // 層は 30f から居る。90f では全部の配置が出ていて、番号順に右へ 20 ずつずれる。
    let copies = motolii_render::picture::resolve::resolved_layers(&doc.view(), at(90)).unwrap();
    assert_eq!(copies.iter().map(|c| (c.id, c.copy)).collect::<Vec<_>>(), [(paper, 0), (paper, 1), (paper, 2)]);
    let origins: Vec<[f32; 2]> = copies.iter().map(|c| c.placement.transform.translation.to_array()).collect();
    assert_eq!(origins, [[100.0, 50.0], [120.0, 50.0], [140.0, 50.0]]);
    assert_eq!(copies.iter().map(|c| c.placement.opacity).collect::<Vec<_>>(), [1.0, 0.75, 0.5]);
    // 配置効果より上には何も無く、下の blur は全体に掛かる側へ残る。
    assert!(copies.iter().all(|c| c.effects.is_empty()));
    assert!(copies.iter().all(|c| c.after_effects.iter().map(|e| e.plugin_id.as_str()).eq(["motolii.blur"])));

    // 45f では 2 番目(1 秒遅れ)はまだ 15f の姿 = 層の始まる前なので出ない。
    let early = motolii_render::picture::resolve::resolved_layers(&doc.view(), at(45)).unwrap();
    assert_eq!(early.iter().map(|c| c.copy).collect::<Vec<_>>(), [0]);
}

/// Repeater の Transform(2026-09-14 利用者裁定: 選べる)。Each は層を回すと複製が 1 つずつ回り、並びは動かない。
/// Whole は並びごと層のアンカーを中心に回る(AE のシェイプ層の Repeater・Cavalry Duplicator・C4D Cloner)。
#[test]
fn a_repeaters_transform_turns_each_copy_or_the_whole_arrangement() {
    use motolii_render::extensions::placement;
    use motolii_doc::store::{property, EffectId, EffectInstance};
    let origins = |choice: f64| {
        let mut doc = document();
        let paper = layer(&mut doc, 1, LayerSource::Shape);
        put(&mut doc, paper, [100.0, 50.0]);
        let repeat = EffectId(0);
        doc.apply_all([
            Intent::SetEffects { layer: paper, effects: vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }] },
            Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(2.0) },
            Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([20.0, 0.0]) },
            Intent::SetConstant { layer: paper, property: PropertyId::effect_param(repeat, "transform").unwrap(), value: Value::F64(choice) },
            Intent::SetConstant { layer: paper, property: PropertyId::new(property::ROTATION).unwrap(), value: Value::F64(90.0) },
        ])
        .unwrap();
        motolii_render::picture::resolve::resolved_layers(&doc.view(), at(90)).unwrap().iter().map(|c| c.placement.transform.translation.to_array().map(|v| v.round())).collect::<Vec<_>>()
    };
    assert_eq!(origins(f64::from(placement::TRANSFORM_EACH)), [[100.0, 50.0], [120.0, 50.0]], "each copy turns in place; the row stays");
    assert_eq!(origins(f64::from(placement::TRANSFORM_WHOLE)), [[100.0, 50.0], [100.0, 70.0]], "the row turns with the layer");
}

#[test]
fn a_repeater_on_a_group_hands_out_the_children_instead_of_the_group() {
    use motolii_render::extensions::placement;
    use motolii_doc::store::{EffectId, EffectInstance, LayerAttrsPatch};
    let mut doc = document();
    let group = layer(&mut doc, 1, LayerSource::Group);
    let circle = layer(&mut doc, 2, LayerSource::Shape);
    let square = layer(&mut doc, 3, LayerSource::Shape);
    for child in [circle, square] {
        doc.apply(Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..Default::default() } }).unwrap();
    }
    put(&mut doc, group, [100.0, 100.0]);
    put(&mut doc, circle, [10.0, 0.0]);
    put(&mut doc, square, [0.0, 10.0]);
    let repeat = EffectId(0);
    doc.apply_all([
        Intent::SetEffects { layer: group, effects: vec![EffectInstance { id: repeat, plugin_id: placement::REPEAT.to_owned() }] },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(4.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "pick").unwrap(), value: Value::F64(1.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([50.0, 0.0]) },
    ])
    .unwrap();
    // Iterate: 4 placements alternate circle, square, circle, square. The children are not drawn on their own.
    let out = motolii_render::picture::resolve::resolved_layers(&doc.view(), at(60)).unwrap();
    let mut seen: Vec<(LayerId, u32, [f32; 2])> = out.iter().map(|c| (c.id, c.copy, c.placement.transform.translation.to_array())).collect();
    seen.sort_by_key(|(id, copy, _)| (id.0, *copy));
    assert_eq!(seen, [
        (circle, 0, [110.0, 100.0]), (circle, 2, [210.0, 100.0]),
        (square, 1, [150.0, 110.0]), (square, 3, [250.0, 110.0]),
    ]);
    assert!(out.iter().all(|c| c.id != group));
    // Random with all the weight on the square: every placement is a square.
    doc.apply_all([
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "pick").unwrap(), value: Value::F64(0.0) },
        Intent::SetConstant { layer: group, property: PropertyId::effect_param(repeat, "share.2").unwrap(), value: Value::F64(0.0) },
    ])
    .unwrap();
    let out = motolii_render::picture::resolve::resolved_layers(&doc.view(), at(60)).unwrap();
    assert_eq!(out.len(), 4);
    assert!(out.iter().all(|c| c.id == square));
    // Whole group: every placement carries both children.
    doc.apply(Intent::SetConstant { layer: group, property: PropertyId::effect_scope(repeat), value: Value::Enum(EffectScope::Whole.enum_value()) }).unwrap();
    let out = motolii_render::picture::resolve::resolved_layers(&doc.view(), at(60)).unwrap();
    assert_eq!(out.len(), 8);
    assert_eq!(out.iter().filter(|c| c.id == circle).count(), 4);
    assert_eq!(out.iter().filter(|c| c.id == square && c.copy == 3).map(|c| c.placement.transform.translation.to_array()).next(), Some([250.0, 110.0]));
}
