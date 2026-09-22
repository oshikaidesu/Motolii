//! コアの読みを、編集で組んだ作品で確かめる。組むのに編集が要るのでこの家に置く。

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use motolii_doc::store::*;
    use motolii_edit::{blank_project, Animate, Document, Intent};
    use super::*;
    use motolii_doc::store::{
        property, LayerId, PropertyId, RationalTime, Value,
    };

    #[test]
    fn consuming_edit_authority_keeps_the_undo_head() {
        let mut doc = blank_project();
        let original = doc.view().composition().unwrap().unwrap();
        let mut edited = original.clone();
        edited.width += 100;
        doc.apply(Intent::SetComposition(edited)).unwrap();
        assert!(doc.undo());
        let recording = doc.into_recording();
        assert_eq!(
            recording.view().composition().unwrap().unwrap().width,
            original.width
        );
    }

    #[test]
    fn save_round_trips_a_typed_anchor_without_renderer_or_grid() {
        let path = std::env::temp_dir().join(format!(
            "motolii-anchor-save-{}-{}.rrd",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut doc = blank_project();
        let layer = LayerId(7);
        let anchor = PropertyId::new(property::ANCHOR).unwrap();
        let exact = [12.345_678_901_234_5, -3.141_592_653_589_79];
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetConstant {
                layer,
                property: anchor.clone(),
                value: Value::Vec2(exact),
            },
        ])
        .unwrap();

        // This exercises flattened()/save() only. No renderer, Grid, layout
        // solver, or FrameGraph is needed for the failure that regressed here.
        doc.save(&path).unwrap();
        let recording = Recording::load(&path).unwrap();
        let editor = Document::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(
            recording
                .view()
                .value_at(layer, &anchor, RationalTime::ZERO)
                .unwrap(),
            Some(Value::Vec2(exact))
        );
        assert_eq!(
            editor
                .view()
                .value_at(layer, &anchor, RationalTime::ZERO)
                .unwrap(),
            Some(Value::Vec2(exact))
        );
    }

    #[test]
    fn saved_recording_and_editor_load_the_same_values() {
        let path = std::env::temp_dir().join(format!(
            "motolii-recording-{}-{}.rrd",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut doc = blank_project();
        let layer = LayerId(7);
        let opacity = PropertyId::new(property::OPACITY).unwrap();
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetConstant {
            layer,
            property: opacity.clone(),
            value: Value::F64(0.7),
        })
        .unwrap();
        doc.set_transient(layer, opacity.clone(), Value::F64(0.2));
        doc.save(&path).unwrap();
        let recording = Recording::load(&path).unwrap();
        let editor = Document::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(recording.view().layers(), editor.view().layers());
        assert_eq!(recording.view().layers(), vec![layer]);
        assert_eq!(
            recording
                .view()
                .value_at(layer, &opacity, RationalTime::ZERO)
                .unwrap(),
            Some(Value::F64(0.7))
        );
        assert_eq!(
            doc.into_recording()
                .view()
                .value_at(layer, &opacity, RationalTime::ZERO)
                .unwrap(),
            Some(Value::F64(0.7))
        );
        assert_eq!(
            serde_json::to_value(recording.view().composition().unwrap()).unwrap(),
            serde_json::to_value(editor.view().composition().unwrap()).unwrap()
        );
        fn requires_send<T: Send>(_: &T) {}
        requires_send(&recording);
    }
}

