use std::collections::BTreeMap;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, AudioComponent, Clip, ClipSource, DocParam, Document, EffectDefinition,
    EffectDefinitionId, EffectId, EffectUse, Group, ItemEnvelope, Soundtrack, StandardShape, Track,
    TrackItem, VectorContent, VectorRecipe, VideoComponent,
    MIN_READER_VERSION_FOR_ASSET_COMPONENTS, RECT_LAYER_SOURCE,
};

fn add_definition(doc: &mut Document, plugin_id: &str) -> EffectDefinitionId {
    let id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        id,
        plugin_id,
        1,
        true,
        BTreeMap::from([("opacity".into(), DocParam::const_f64(0.72))]),
        Default::default(),
    ));
    id
}

fn link_use(doc: &mut Document, envelope: &mut ItemEnvelope, definition_id: EffectDefinitionId) {
    envelope.effects.push(EffectUse {
        id: EffectId::from_raw(doc.next_stable_id.allocate().unwrap()),
        definition_id,
    });
}

fn plugin_shape(layer: motolii_doc::LayerId, center: [f64; 2]) -> Clip {
    Clip {
        envelope: ItemEnvelope::new(layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Plugin {
            plugin_id: RECT_LAYER_SOURCE.into(),
            effect_version: 1,
            params: BTreeMap::from([
                ("center".into(), DocParam::const_vec2(center)),
                ("size".into(), DocParam::const_vec2([0.24, 0.24])),
                (
                    "color".into(),
                    DocParam::const_color([0.65, 0.72, 0.86, 1.0]),
                ),
            ]),
            extra: Default::default(),
        },
    }
}

fn fixture_document() -> Document {
    let mut doc = Document::new_current();
    doc.min_reader_version = doc
        .min_reader_version
        .max(MIN_READER_VERSION_FOR_ASSET_COMPONENTS);

    let media_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: media_id,
            name: "Reference AV".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:reference-av".into(),
            path_absolute: None,
            path_project_relative: Some("media/reference-av.mp4".into()),
            file_name: Some("reference-av.mp4".into()),
            size_bytes: Some(1024),
            head_hash: Some("reference-head".into()),
            tail_hash: Some("reference-tail".into()),
        })
        .unwrap();
    let font_id = AssetId::from_raw(1);
    doc.assets
        .insert(Asset {
            id: font_id,
            name: "Reference Font".into(),
            asset_type: "font/ttf".into(),
            content_hash: "sha256:reference-font".into(),
            path_absolute: None,
            path_project_relative: Some("fonts/reference.ttf".into()),
            file_name: Some("reference.ttf".into()),
            size_bytes: Some(512),
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    doc.soundtrack = Some(Soundtrack::try_new(media_id, RationalTime::ZERO, 0.8).unwrap());

    let shared = add_definition(&mut doc, "core.filter.opacity");
    let left = doc.layers.allocate("Shared left").unwrap();
    let media = doc.layers.allocate("Video + audio").unwrap();
    let middle = doc.layers.allocate("Shared middle").unwrap();
    let text = doc.layers.allocate("Reference text").unwrap();
    let right = doc.layers.allocate("Shared right").unwrap();
    let group = doc.layers.allocate("Reference group").unwrap();
    let child = doc.layers.allocate("Group shape child").unwrap();
    let track_id = doc.track_ids.allocate("Reference timeline").unwrap();

    let mut left_clip = plugin_shape(left, [-0.55, 0.0]);
    link_use(&mut doc, &mut left_clip.envelope, shared);

    let media_clip = Clip {
        envelope: ItemEnvelope::new(media),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Asset {
            asset: media_id,
            video: Some(VideoComponent::ordinal(0)),
            audio: vec![AudioComponent::ordinal(0)],
        },
    };

    let mut middle_clip = plugin_shape(middle, [0.0, 0.0]);
    let local = add_definition(&mut doc, "core.filter.opacity");
    link_use(&mut doc, &mut middle_clip.envelope, local);
    link_use(&mut doc, &mut middle_clip.envelope, shared);

    let text_clip = Clip {
        envelope: ItemEnvelope::new(text),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Vector {
            recipe: VectorRecipe {
                content: VectorContent::TextPath {
                    text: "Motolii Reference".into(),
                    font_asset: font_id,
                },
                modifiers: vec![],
            },
        },
    };

    let mut right_clip = plugin_shape(right, [0.55, 0.0]);
    link_use(&mut doc, &mut right_clip.envelope, shared);
    link_use(&mut doc, &mut right_clip.envelope, local);

    let mut child_envelope = ItemEnvelope::new(child);
    child_envelope.transform.parent = Some(group);
    let child_clip = Clip {
        envelope: child_envelope,
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Vector {
            recipe: VectorRecipe {
                content: VectorContent::StandardShape {
                    shape: StandardShape::Ellipse {
                        width: DocParam::const_f64(0.25),
                        height: DocParam::const_f64(0.25),
                    },
                },
                modifiers: vec![],
            },
        },
    };
    let group_item = Group {
        envelope: ItemEnvelope::new(group),
        children: vec![TrackItem::Clip(child_clip)],
    };

    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(left_clip),
            TrackItem::Clip(media_clip),
            TrackItem::Clip(middle_clip),
            TrackItem::Clip(text_clip),
            TrackItem::Clip(right_clip),
            TrackItem::Group(group_item),
        ],
    });
    doc.validate().unwrap();
    doc
}

#[test]
fn committed_reference_document_is_current_valid_and_byte_canonical() {
    let bytes = include_bytes!("../../../docs/mocks-ui/fixtures/reference-document.json");
    let before = bytes.to_vec();
    let loaded = motolii_doc::load_document_bytes(bytes).unwrap();
    let expected = fixture_document();
    assert_eq!(loaded, expected);
    assert_eq!(
        bytes,
        format!("{}\n", serde_json::to_string_pretty(&loaded).unwrap()).as_bytes()
    );
    assert_eq!(bytes, before.as_slice());

    let shared = expected.effect_definitions[0].id;
    let positions: Vec<usize> = expected.tracks[0]
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let envelope = match item {
                TrackItem::Clip(clip) => &clip.envelope,
                TrackItem::Group(group) => &group.envelope,
            };
            envelope
                .effects
                .iter()
                .any(|effect| effect.definition_id == shared)
                .then_some(index)
        })
        .collect();
    assert_eq!(positions, [0, 2, 4]);
    let stack_positions: Vec<usize> = [0usize, 2, 4]
        .into_iter()
        .map(|index| {
            let TrackItem::Clip(clip) = &expected.tracks[0].items[index] else {
                panic!("shared use must remain on a clip");
            };
            clip.envelope
                .effects
                .iter()
                .position(|effect| effect.definition_id == shared)
                .unwrap()
        })
        .collect();
    assert_eq!(stack_positions, [0, 1, 0]);

    let TrackItem::Clip(media) = &expected.tracks[0].items[1] else {
        panic!("reference media item must remain a clip");
    };
    assert!(matches!(
        &media.source,
        ClipSource::Asset {
            video: Some(_),
            audio,
            ..
        } if audio.len() == 1
    ));
    assert!(matches!(
        &expected.tracks[0].items[3],
        TrackItem::Clip(Clip {
            source: ClipSource::Vector {
                recipe: VectorRecipe {
                    content: VectorContent::TextPath { .. },
                    ..
                }
            },
            ..
        })
    ));
    assert!(matches!(
        &expected.tracks[0].items[5],
        TrackItem::Group(Group {
            children,
            ..
        }) if matches!(
            children.as_slice(),
            [TrackItem::Clip(Clip {
                source: ClipSource::Vector {
                    recipe: VectorRecipe {
                        content: VectorContent::StandardShape {
                            shape: StandardShape::Ellipse { .. }
                        },
                        ..
                    }
                },
                ..
            })]
        )
    ));
}
