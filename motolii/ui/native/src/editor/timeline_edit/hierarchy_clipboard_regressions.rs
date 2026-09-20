use crate::edit::{Animate, Document, Intent};
use super::*;
use crate::doc::store::{
    KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId,
    RationalTime,
};
use crate::editor::keyframe_edit::document_fps;

#[test]
fn copied_group_contains_descendants_through_non_group_parents() {
    let mut doc = crate::edit::blank_project().with_programs(crate::render::extensions::bundled());
    for (id, source, parent) in [
        (1, LayerSource::Group, None),
        (2, LayerSource::Null, Some(LayerId(1))),
        (3, LayerSource::Shape, Some(LayerId(2))),
    ] {
        doc.apply_all([
            Intent::AddLayer(LayerId(id)),
            Intent::SetMeta {
                layer: LayerId(id),
                meta: LayerMeta {
                    source,
                    order: id as i16,
                    timing: LayerTiming::place(0, None, 100),
                },
            },
            Intent::SetAttrs {
                layer: LayerId(id),
                patch: LayerAttrsPatch {
                    parent: Some(parent),
                    name: Some(format!("Layer {id}")),
                    ..Default::default()
                },
            },
        ])
        .unwrap();
    }
    let clipboard = copy_layers(&doc, &[LayerId(1), LayerId(3)]).unwrap();
    assert_eq!(clipboard.copies.len(), 3);
    assert_eq!(clipboard.roots.len(), 1);
    let before = doc.history_depth().0;
    let roots = paste_layers(&mut doc, &clipboard).unwrap();
    assert_eq!(doc.view().layers().len(), 6);
    assert_eq!(doc.history_depth().0, before + 1);
    let parent = doc
        .view()
        .layers()
        .into_iter()
        .find(|id| doc.view().attrs(*id).unwrap().unwrap().parent == Some(roots[0]))
        .unwrap();
    let child = doc
        .view()
        .layers()
        .into_iter()
        .find(|id| doc.view().attrs(*id).unwrap().unwrap().parent == Some(parent))
        .unwrap();
    assert_ne!(parent, LayerId(2));
    assert_ne!(child, LayerId(3));
    assert!(doc.undo());
    assert_eq!(doc.view().layers().len(), 3);
}

#[test]
fn content_selection_does_not_delete_or_move_other_property_keys() {
    let mut doc = crate::edit::fixture::build().doc;
    let layer = doc
        .view()
        .layers()
        .into_iter()
        .find(|id| doc.view().text_document(*id).unwrap().is_some())
        .unwrap();
    let fps = document_fps(&doc).unwrap();
    let mut text = doc.view().text_document(layer).unwrap().unwrap();
    text.content = crate::doc::store::ContentTrack::new();
    for (frame, content) in [(0, "a"), (10, "b")] {
        text.content.insert(crate::doc::store::ContentKeyframe {
            t: RationalTime::try_from_frame(frame, fps).unwrap(),
            content: content.into(),
        });
    }
    let property = PropertyId::new(crate::doc::store::property::POSITION).unwrap();
    let mut track = KeyframeTrack::new();
    track.insert(crate::doc::store::Keyframe {
        t: RationalTime::try_from_frame(10, fps).unwrap(),
        value: crate::doc::store::Value::Vec2([5.0, 6.0]),
        interp: crate::doc::store::Interp::Linear,
        spatial: None,
    });
    doc.apply_all([
        Intent::SetTextDocument {
            layer,
            document: text,
        },
        Intent::SetTrack {
            layer,
            property: property.clone(),
            track: track.clone(),
        },
    ])
    .unwrap();
    let selected = [(
        layer,
        Some(PropertyId::new("content").unwrap()),
        10.0 / fps.as_f64(),
    )];
    let moved =
        crate::editor::keyframe_edit::key_selection_move_intents(&doc, &selected, 5).unwrap();
    doc.apply_all(moved).unwrap();
    assert_eq!(doc.view().track(layer, &property).unwrap().unwrap(), track);
    assert_eq!(
        doc.view()
            .text_document(layer)
            .unwrap()
            .unwrap()
            .content
            .keys()[1]
            .t
            .try_to_frame_round(fps)
            .unwrap(),
        15
    );
    assert!(doc.undo());
    let deleted = delete_key_selection_intents(&doc, &selected, RationalTime::ZERO).unwrap();
    doc.apply_all(deleted).unwrap();
    assert_eq!(doc.view().track(layer, &property).unwrap().unwrap(), track);
    assert_eq!(
        doc.view()
            .text_document(layer)
            .unwrap()
            .unwrap()
            .content
            .keys()
            .len(),
        1
    );
}
