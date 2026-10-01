use crate::edit::{Animate, Document, Intent};
use super::*;
use crate::doc::store::*;

#[test]
fn content_only_copy_and_paste_keep_explicit_content_identity() {
    let mut doc = crate::edit::fixture::build().doc;
    let layer = doc
        .view()
        .layers()
        .into_iter()
        .find(|id| doc.view().text_document(*id).unwrap().is_some())
        .unwrap();
    let fps = crate::editor::keyframe_edit::document_fps(&doc).unwrap();
    let mut document = doc.view().text_document(layer).unwrap().unwrap();
    document.content = ContentTrack::new();
    document.content.insert(ContentKeyframe {
        t: RationalTime::ZERO,
        content: "a".into(),
    });
    let property = PropertyId::new(property::POSITION).unwrap();
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::Vec2([1.0, 2.0]),
        interp: Interp::Linear,
        spatial: None,
    });
    doc.apply_all([
        Intent::SetTextDocument { layer, document },
        Intent::SetTrack {
            layer,
            property: property.clone(),
            track: track.clone(),
        },
    ])
    .unwrap();
    let clipboard = Clipboard::default();
    assert_eq!(
        clipboard
            .copy_keys(
                &doc,
                &[KeySel {
                    layer,
                    property: Some(PropertyId::new("content").unwrap()),
                    at_sec: 0.0
                }]
            )
            .unwrap(),
        1
    );
    let PasteResult::Keys(selected) = clipboard.paste(&mut doc, Some(layer), 20).unwrap()
    else {
        panic!("expected keys")
    };
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].property.as_ref().unwrap().name(), "content");
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
        20
    );
}

#[test]
fn missing_multi_layer_sources_cannot_merge_onto_one_fallback() {
    let mut doc = crate::edit::blank_project().with_programs(crate::render::extensions::bundled());
    for id in [1, 2, 3] {
        doc.apply_all([
            Intent::AddLayer(LayerId(id)),
            Intent::SetMeta {
                layer: LayerId(id),
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: id as i16,
                    timing: LayerTiming::place(0, None, 100),
                },
            },
        ])
        .unwrap();
    }
    let property = PropertyId::new(property::OPACITY).unwrap();
    let copied: Vec<_> = [1, 2]
        .into_iter()
        .map(|id| CopiedKey {
            layer: LayerId(id),
            offset_frames: 0,
            value: CopiedKeyValue::Property(
                property.clone(),
                Keyframe {
                    t: RationalTime::ZERO,
                    value: Value::F64(id as f64 * 0.1),
                    interp: Interp::Linear,
                    spatial: None,
                },
            ),
        })
        .collect();
    doc.apply(Intent::RemoveLayer(LayerId(1))).unwrap();
    let history = doc.history_depth();
    assert!(paste_keys(&mut doc, &copied, true, Some(LayerId(3)), 10).is_err());
    assert_eq!(doc.history_depth(), history);
    assert!(doc.view().track(LayerId(3), &property).unwrap().is_none());
}
