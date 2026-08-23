//! P1-lyric-mv #87/#88: multi-selected text layers share one Inspector edit
//! entry and one undo step.

use motolii_core::{Fps, RationalTime};
use motolii_inspector_pane::{
    color::{commit_text_style_color_for_layers, ColorChannel, ColorFieldDraft, ColorTarget},
    commit_text_style_track_field_for_layers, TextStyleField, TextStyleTrackDraft,
};
use motolii_shell_state::Session;
use motolii_store::{
    Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId,
    Value,
};

fn two_text_layers() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();

    for (order, layer) in [LayerId(1), LayerId(2)].into_iter().enumerate() {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Text,
                    order: order as i16,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .unwrap();
    }
    doc
}

#[test]
fn multi_selected_size_writes_each_text_layer_in_one_undo() {
    let mut doc = two_text_layers();
    let layers = [LayerId(1), LayerId(2)];
    let mut draft = Some(TextStyleTrackDraft {
        style: motolii_store::TextStyleId(0),
        field: TextStyleField::Size,
        text: "48".to_owned(),
    });

    commit_text_style_track_field_for_layers(
        &mut doc,
        &mut draft,
        &layers,
        0,
        Fps::try_new(30, 1).unwrap(),
        motolii_store::TextStyleId(0),
        TextStyleField::Size,
    )
    .unwrap();
    assert!(draft.is_none());

    let property = PropertyId::text_style_size(motolii_store::TextStyleId(0));
    for layer in layers {
        assert_eq!(
            doc.view()
                .value_at(layer, &property, RationalTime::ZERO)
                .unwrap(),
            Some(Value::F64(48.0)),
            "layer {layer:?} に Size が反映されていない"
        );
    }

    assert!(doc.undo(), "一括 Size の undo が無い");
    for layer in layers {
        assert_eq!(
            doc.view()
                .value_at(layer, &property, RationalTime::ZERO)
                .unwrap(),
            None,
            "layer {layer:?} が1回の undo で戻っていない"
        );
    }
}

#[test]
fn multi_selected_fill_writes_each_text_layer_in_one_undo() {
    let mut doc = two_text_layers();
    let layers = [LayerId(1), LayerId(2)];
    let mut draft = Some(ColorFieldDraft {
        target: ColorTarget::Fill,
        channel: ColorChannel::R,
        text: "64".to_owned(),
    });

    commit_text_style_color_for_layers(
        &mut doc,
        &mut draft,
        &layers,
        ColorTarget::Fill,
        ColorChannel::R,
    )
    .unwrap();
    assert!(draft.is_none());

    for layer in layers {
        let document = doc.view().text_document(layer).unwrap().unwrap();
        assert_eq!(document.styles[0].fill[0], 64.0 / 255.0);
    }

    assert!(doc.undo(), "一括 Fill の undo が無い");
    for layer in layers {
        assert!(
            doc.view().text_document(layer).unwrap().is_none(),
            "layer {layer:?} が1回の undo で戻っていない"
        );
    }
}

#[test]
fn multi_selected_projection_uses_the_selection_set_and_counts_text_layers() {
    let doc = two_text_layers();
    let session = Session {
        selected_layers: vec![LayerId(1), LayerId(2)],
        ..Session::default()
    };

    let projection = motolii_inspector_pane::project(&doc.view(), &session)
        .unwrap()
        .unwrap();
    assert_eq!(projection.selection_count, 2);
    assert_eq!(projection.text_layer_count, 2);
    assert!(projection.text.is_some());
}
