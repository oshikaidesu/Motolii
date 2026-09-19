use motolii_doc::store::kind::PlacementProgram;
use motolii_doc::store::{
    blank_project, EffectId, EffectInstance, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    Placement, RationalTime, Value,
};

fn two_copies(_: &[(String, Value)]) -> Vec<Placement> {
    (0..2)
        .map(|index| Placement {
            index,
            offset: [index as f32 * 25.0, 0.0],
            rotation_degrees: 0.0,
            scale: 1.0,
            offset_z: 0.0,
            opacity: 1.0,
            time_offset: RationalTime::ZERO,
            stretch: [1.0; 2],
        })
        .collect()
}

#[test]
fn an_external_placement_program_changes_only_the_read_projection() {
    let mut doc = blank_project();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
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
    assert_ne!(
        view.revision_key(),
        view.clone().with_placement_programs(&[]).revision_key()
    );
    assert_eq!(view.resolved_layers(RationalTime::ZERO).unwrap().len(), 1);
    assert_eq!(doc.revision(), revision);
    assert_eq!(doc.edit_head(), head);
}
