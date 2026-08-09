use motolii_core::RationalTime;
use motolii_doc::{
    resolve_document_spaces, Clip, ClipSource, DocParam, Document, EvaluationTime, Group,
    ItemEnvelope, LayerId, Track, TrackItem, Transform2D, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_ui::{project_stage_geometry, StageLayerProjection};
use std::collections::BTreeMap;

fn rect(center: [f64; 2], size: [f64; 2]) -> ClipSource {
    ClipSource::Plugin {
        plugin_id: RECT_LAYER_SOURCE.into(),
        effect_version: 1,
        params: BTreeMap::from([
            ("center".into(), DocParam::const_vec2(center)),
            ("size".into(), DocParam::const_vec2(size)),
            ("color".into(), DocParam::const_color([1.0, 1.0, 1.0, 1.0])),
        ]),
        extra: Default::default(),
    }
}

fn clip(layer: LayerId, transform: Transform2D) -> TrackItem {
    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform = transform;
    TrackItem::Clip(Clip {
        envelope,
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: Default::default(),
        source: rect([0.0, 0.0], [0.5, 0.5]),
    })
}

#[test]
fn stage_projection_uses_group_world_map_for_nested_children_and_preserves_other_worlds() {
    let mut document = Document::new_current();
    let track = document.track_ids.allocate("V1").unwrap();
    document.tracks.push(Track {
        id: track,
        items: Vec::new(),
    });

    let outer = document.layers.allocate("outer").unwrap();
    let inner = document.layers.allocate("inner").unwrap();
    let grouped = document.layers.allocate("grouped").unwrap();
    let outside_parent = document.layers.allocate("outside-parent").unwrap();
    let outside_child = document.layers.allocate("outside-child").unwrap();

    let mut outer_envelope = ItemEnvelope::new(outer);
    outer_envelope.transform.position = DocParam::const_vec2([0.8, -0.2]);
    outer_envelope.transform.rotation = DocParam::const_f64(0.25);

    let mut inner_envelope = ItemEnvelope::new(inner);
    inner_envelope.transform.position = DocParam::const_vec2([-0.3, 0.4]);
    inner_envelope.transform.scale = DocParam::const_vec2([1.2, 0.7]);

    let mut parent_envelope = ItemEnvelope::new(outside_parent);
    parent_envelope.transform.position = DocParam::const_vec2([0.6, 0.1]);

    document.tracks[0].items = vec![
        TrackItem::Group(Group {
            envelope: outer_envelope,
            children: vec![TrackItem::Group(Group {
                envelope: inner_envelope,
                children: vec![clip(grouped, Transform2D::identity())],
            })],
        }),
        TrackItem::Clip(Clip {
            envelope: parent_envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(10, 1).unwrap(),
            time_map: Default::default(),
            source: rect([0.0, 0.0], [0.5, 0.5]),
        }),
        clip(
            outside_child,
            Transform2D {
                position: DocParam::const_vec2([0.2, 0.3]),
                parent: Some(outside_parent),
                ..Transform2D::identity()
            },
        ),
    ];

    document.validate().unwrap();
    let tracks = DataTracks::new();
    let t = RationalTime::ZERO;
    let (_, world_map) = resolve_document_spaces(&document, t, &tracks).unwrap();
    let projection = project_stage_geometry(&document, EvaluationTime::new(t), &tracks).unwrap();

    let available = |layer| match projection.get(layer).unwrap() {
        StageLayerProjection::Available(geometry) => geometry,
        other => panic!("expected available projection, got {other:?}"),
    };

    assert_eq!(available(grouped).world, world_map[&grouped.get()]);
    assert_eq!(
        available(outside_child).world,
        world_map[&outside_child.get()]
    );
    let canonical =
        (available(grouped).camera_view * available(grouped).world).transform_point(0.0, 0.0);
    let local = (available(grouped).camera_view * available(grouped).world)
        .try_invert()
        .unwrap()
        .transform_point(canonical[0], canonical[1]);
    assert!(local[0].abs() <= 0.25 && local[1].abs() <= 0.25);
}
