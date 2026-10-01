use crate::edit::{Animate, Document, Intent};
use super::*;
use crate::doc::store::*;

fn document() -> Document {
    let mut doc = crate::edit::blank_project().with_programs(crate::render::extensions::bundled());
    for id in [1, 2] {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(id as f64 * 0.1),
            interp: Interp::Linear,
            spatial: None,
        });
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
            Intent::SetTrack {
                layer: LayerId(id),
                property: PropertyId::new(property::OPACITY).unwrap(),
                track,
            },
        ])
        .unwrap();
    }
    doc
}

fn key(layer: u64) -> KeySel {
    KeySel {
        layer: LayerId(layer),
        property: Some(PropertyId::new(property::OPACITY).unwrap()),
        at_sec: 0.0,
    }
}

#[test]
fn matching_layer_numbers_in_another_document_require_explicit_destination() {
    let source = document();
    let mut destination = document();
    assert_ne!(source.identity(), destination.identity());
    let clipboard = Clipboard::default();
    clipboard.copy_keys(&source, &[key(1)]).unwrap();
    let history = destination.history_depth();
    assert!(clipboard.paste(&mut destination, None, 30).is_err());
    assert_eq!(destination.history_depth(), history);
    let PasteResult::Keys(keys) = clipboard
        .paste(&mut destination, Some(LayerId(2)), 30)
        .unwrap()
    else {
        panic!("expected keys")
    };
    assert_eq!(keys[0].layer, LayerId(2));
    let property = PropertyId::new(property::OPACITY).unwrap();
    assert_eq!(
        destination
            .view()
            .track(LayerId(1), &property)
            .unwrap()
            .unwrap()
            .keys()
            .len(),
        1
    );
    assert_eq!(
        destination
            .view()
            .track(LayerId(2), &property)
            .unwrap()
            .unwrap()
            .keys()
            .len(),
        2
    );
    assert!(destination.undo());
    assert_eq!(
        destination
            .view()
            .track(LayerId(2), &property)
            .unwrap()
            .unwrap()
            .keys()
            .len(),
        1
    );
}

#[test]
fn matching_multi_layer_numbers_in_another_document_are_not_a_mapping() {
    let source = document();
    let mut destination = document();
    let clipboard = Clipboard::default();
    clipboard.copy_keys(&source, &[key(1), key(2)]).unwrap();
    let history = destination.history_depth();
    assert!(clipboard
        .paste(&mut destination, Some(LayerId(2)), 30)
        .is_err());
    assert_eq!(destination.history_depth(), history);
    let property = PropertyId::new(property::OPACITY).unwrap();
    for id in [1, 2] {
        assert_eq!(
            destination
                .view()
                .track(LayerId(id), &property)
                .unwrap()
                .unwrap()
                .keys()
                .len(),
            1
        );
    }
}
