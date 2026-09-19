use motolii_doc::store::kind::{PlacementInput, PlacementOutput, PlacementProgram};
use motolii_doc::store::{
    blank_project, EffectId, EffectInstance, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    Placement, RationalTime,
};

#[test]
fn motion_sampling_preserves_read_authority_and_effect_order() {
    use motolii_doc::store::{property, Interp, Keyframe, KeyframeTrack, PropertyId, Value};
    let mut doc = blank_project();
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
                    plugin_id: motolii_doc::extensions::motion::MOTION_BLUR.into(),
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
    let samples = doc.view().resolved_layers(time).unwrap();
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
    let still = doc.view().resolved_layers(time).unwrap();
    assert_eq!(still.len(), 1);
    assert_eq!(still[0].averaged, 0);
    assert_eq!(still[0].placement.opacity, 1.0);
}

fn two_copies(input: &PlacementInput<'_>) -> Vec<PlacementOutput> {
    (0..2)
        .map(|index| Placement {
            index,
            offset: [index as f32 * 25.0 + input.position[0], 0.0],
            rotation_degrees: 0.0,
            scale: 1.0,
            offset_z: 0.0,
            opacity: 1.0,
            time_offset: RationalTime::ZERO,
            stretch: [1.0; 2],
        })
        .map(Into::into)
        .collect()
}

#[test]
fn an_external_placement_program_changes_only_the_read_projection() {
    let mut doc = blank_project();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetConstant {
            layer,
            property: motolii_doc::store::PropertyId::new(motolii_doc::store::property::POSITION)
                .unwrap(),
            value: motolii_doc::store::Value::Vec2([10.0, 0.0]),
        },
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Shape,
                order: 0,
                timing: LayerTiming::place(0, None, 10),
            },
        },
        Intent::SetShapes {
            layer,
            shapes: vec![motolii_doc::store::rect_shape([255; 4], [10.0; 2])],
        },
        Intent::SetEffects {
            layer,
            effects: vec![EffectInstance {
                id: EffectId(1),
                plugin_id: "test.external-placement".into(),
            }],
        },
    ])
    .unwrap();
    let revision = doc.revision();
    let head = doc.edit_head();
    let programs = [PlacementProgram {
        plugin_id: "test.external-placement",
        needs_position: false,
        evaluate: two_copies,
    }];
    let view = doc.view();
    let ordinary = view.resolved_layers(RationalTime::ZERO).unwrap();
    let supplied = view.clone().with_placement_programs(&programs);
    let placed = supplied.resolved_layers(RationalTime::ZERO).unwrap();
    assert_eq!(ordinary.len(), 1);
    assert_eq!(placed.len(), 2);
    assert_eq!(placed[0].id, layer);
    assert_eq!(placed[1].id, layer);
    assert!(
        (placed[1].placement.transform.translation.x
            - placed[0].placement.transform.translation.x
            - 25.0)
            .abs()
            < 0.001
    );
    assert_ne!(view.revision_key(), supplied.revision_key());
    let position_programs = [PlacementProgram {
        needs_position: true,
        ..programs[0]
    }];
    let positioned = view
        .clone()
        .with_placement_programs(&position_programs)
        .resolved_layers(RationalTime::ZERO)
        .unwrap();
    assert!(
        (positioned[0].placement.transform.translation.x
            - placed[0].placement.transform.translation.x
            - 10.0)
            .abs()
            < 0.001
    );
    assert_ne!(
        supplied.revision_key(),
        view.clone()
            .with_placement_programs(&position_programs)
            .revision_key(),
        "different evaluator input requirements must not share display identity"
    );
    assert_ne!(
        view.revision_key(),
        view.clone().with_placement_programs(&[]).revision_key()
    );
    assert_eq!(view.resolved_layers(RationalTime::ZERO).unwrap().len(), 1);
    assert_eq!(doc.revision(), revision);
    assert_eq!(doc.edit_head(), head);
}

#[test]
fn blob_group_uses_sparse_identity_without_drawing_unplaced_children() {
    use motolii_doc::store::{
        analysis::{AnalysisInputs, BlobMark},
        LayerAttrsPatch,
    };
    let mut doc = blank_project();
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
                plugin_id: motolii_doc::extensions::blob::BLOB_TRACK.into(),
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
    assert!(doc
        .view()
        .resolved_layers(RationalTime::ZERO)
        .unwrap()
        .is_empty());
    let placed = doc
        .view()
        .with_analysis(&analysis)
        .resolved_layers(RationalTime::ZERO)
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
    let disabled = doc
        .view()
        .with_analysis(&analysis)
        .resolved_layers(RationalTime::ZERO)
        .unwrap();
    let children: Vec<_> = disabled.iter().filter(|copy| copy.id == child).collect();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].copy, 0);
}
