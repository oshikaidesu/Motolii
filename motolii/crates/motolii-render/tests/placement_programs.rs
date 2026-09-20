use motolii_edit::{blank_project, Intent};
use motolii_doc::store::kind::{PlacementInput, PlacementOutput, PlacementProgram};
use motolii_doc::store::{
    EffectId, EffectInstance, LayerId, LayerMeta, LayerSource, LayerTiming,
    Placement, RationalTime,
};

#[test]
fn committed_and_transient_views_have_distinct_display_identity() {
    use motolii_doc::store::{property, PropertyId, Value};
    let mut doc = blank_project();
    let layer = LayerId(1);
    let position = PropertyId::new(property::POSITION).unwrap();
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetConstant {
            layer,
            property: position.clone(),
            value: Value::Vec2([10.0, 0.0]),
        },
    ])
    .unwrap();
    doc.set_transient(layer, position.clone(), Value::Vec2([20.0, 0.0]));
    let shown = doc.view();
    let committed = shown.clone().without_transients();
    assert_eq!(
        shown
            .value_at(layer, &position, RationalTime::ZERO)
            .unwrap(),
        Some(Value::Vec2([20.0, 0.0]))
    );
    assert_eq!(
        committed
            .value_at(layer, &position, RationalTime::ZERO)
            .unwrap(),
        Some(Value::Vec2([10.0, 0.0]))
    );
    assert_ne!(shown.revision_key(), committed.revision_key());
    assert_eq!(
        committed.revision_key(),
        committed.clone().without_transients().revision_key()
    );
    let key = committed.revision_key();
    doc.set_transient(layer, position.clone(), Value::Vec2([30.0, 0.0]));
    assert_eq!(doc.view().without_transients().revision_key(), key);
    let owner = doc.begin_preview();
    doc.preview_edits(
        owner,
        &[Intent::SetConstant {
            layer,
            property: position.clone(),
            value: Value::Vec2([50.0, 0.0]),
        }],
    )
    .unwrap();
    let committed = doc.view().without_transients();
    assert_eq!(committed.revision_key(), key);
    assert_eq!(
        committed
            .value_at(layer, &position, RationalTime::ZERO)
            .unwrap(),
        Some(Value::Vec2([10.0, 0.0]))
    );
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
        pick: motolii_doc::store::kind::pick_in_turn,
        moves_whole: motolii_doc::store::kind::never_moves_whole,
    }];
    let view = doc.view();
    let ordinary = motolii_render::picture::resolve::resolved_layers(&view, RationalTime::ZERO).unwrap();
    let supplied = view.clone().with_placement_programs(&programs);
    let placed = motolii_render::picture::resolve::resolved_layers(&supplied, RationalTime::ZERO).unwrap();
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
    let positioned = motolii_render::picture::resolve::resolved_layers(
        &view.clone().with_placement_programs(&position_programs),
        RationalTime::ZERO,
    )
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
    assert_eq!(motolii_render::picture::resolve::resolved_layers(&view, RationalTime::ZERO).unwrap().len(), 1);
    assert_eq!(doc.revision(), revision);
    assert_eq!(doc.edit_head(), head);
}

/// 作品に渡した効果は、平らにしても読み取り専用にしても付いて行く。
/// 落ちると絵から効果が黙って消えるだけで、どこも失敗しない(書き出しと Freeze がこの道)。
#[test]
fn a_work_keeps_its_effects_when_it_is_flattened_and_made_read_only() {
    use motolii_doc::store::kind::{pick_in_turn, never_moves_whole, PlacementProgram, Programs};

    const MARK: &str = "test.two-copies";
    fn program(plugin_id: &str) -> Option<PlacementProgram> {
        (plugin_id == MARK).then_some(PlacementProgram {
            plugin_id: MARK,
            needs_position: false,
            evaluate: two_copies,
            pick: pick_in_turn,
            moves_whole: never_moves_whole,
        })
    }

    let mut doc = blank_project().with_programs(Programs { placement: program, ..Programs::NONE });
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 300) } },
        Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(1), plugin_id: MARK.to_owned() }] },
    ])
    .unwrap();

    let copies = |view: &motolii_doc::store::StoreView<'_>| motolii_render::picture::resolve::resolved_layers(&view, RationalTime::ZERO).unwrap().len();
    assert_eq!(copies(&doc.view()), 2, "the work itself places two copies");

    let flat = doc.flattened().unwrap();
    assert_eq!(copies(&flat.view()), 2, "flattening must not drop the effects");

    let recording = flat.into_recording();
    assert_eq!(copies(&recording.view()), 2, "the read-only work exports with its effects");
}
