use super::{
    read::{ReadOverlay, RecordCache, TrackCache},
    Revision, StoreError, StoreView, EDIT_TIMELINE,
};
use re_entity_db::EntityDb;
use re_log_types::Timeline;
use std::{cell::RefCell, collections::HashMap, path::Path};

/// An owned recording with no editing or Undo authority.
///
/// ```compile_fail
/// fn cannot_edit(recording: &mut motolii_doc::store::Recording) {
///     recording.undo();
/// }
/// ```
pub struct Recording {
    db: EntityDb,
    head: i64,
    transient: HashMap<super::read::TransientKey, super::Value>,
    overlay: ReadOverlay,
    tracks: RefCell<TrackCache>,
    records: RefCell<RecordCache>,
    layout: RefCell<super::layout::LayoutCache>,
}

impl Recording {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let db = load_db(path.as_ref())?;
        let head = edit_head(&db);
        Ok(Self::new(db, head))
    }

    pub(crate) fn new(db: EntityDb, head: i64) -> Self {
        Self {
            db,
            head,
            transient: Default::default(),
            overlay: Default::default(),
            tracks: Default::default(),
            records: Default::default(),
            layout: Default::default(),
        }
    }

    pub fn view(&self) -> StoreView<'_> {
        StoreView::new(
            &self.db,
            self.head,
            &self.transient,
            &self.overlay,
            Revision {
                store: self.db.generation(),
                head: self.head,
            },
            &self.tracks,
            &self.records,
            &self.layout,
            crate::doc::extensions::placement_program,
        )
    }
}

pub(crate) fn edit_head(db: &EntityDb) -> i64 {
    db.time_range_for(Timeline::new_sequence(EDIT_TIMELINE).name())
        .map(|range| range.max().as_i64())
        .unwrap_or(0)
}

pub(crate) fn load_db(path: &Path) -> Result<EntityDb, StoreError> {
    let file = std::fs::File::open(path).map_err(|e| StoreError::Io(e.to_string()))?;
    let decoder = re_log_encoding::rrd::DecoderApp::decode_eager(std::io::BufReader::new(file))
        .map_err(|e| StoreError::Io(e.to_string()))?;
    let mut out: Option<EntityDb> = None;
    for message in decoder {
        let message = message.map_err(|e| StoreError::Io(e.to_string()))?;
        let db = out.get_or_insert_with(|| EntityDb::new(message.store_id().clone()));
        db.add_log_msg(&message)
            .map_err(|e| StoreError::Ingest(e.to_string()))?;
    }
    out.ok_or_else(|| StoreError::Io("空の file(メッセージが1つも無い)".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{
        blank_project, property, Document, Intent, LayerId, PropertyId, RationalTime, Value,
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
