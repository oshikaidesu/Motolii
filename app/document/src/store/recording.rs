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
    layout: RefCell<super::scratch::LayoutCache>,
    /// この作品で使える効果。読み込む側が渡す。
    programs: super::kind::Programs,
}

impl Recording {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let db = load_db(path.as_ref())?;
        let head = edit_head(&db);
        Ok(Self::new(db, head))
    }

    pub fn new(db: EntityDb, head: i64) -> Self {
        Self {
            db,
            head,
            transient: Default::default(),
            overlay: Default::default(),
            tracks: Default::default(),
            records: Default::default(),
            layout: Default::default(),
            programs: super::kind::Programs::NONE,
        }
    }

    /// 読み込む側が、この作品で使える効果の表を渡す。
    #[must_use]
    pub fn with_programs(mut self, programs: super::kind::Programs) -> Self {
        self.programs = programs;
        self
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
            self.programs,
        )
    }
}

pub fn edit_head(db: &EntityDb) -> i64 {
    db.time_range_for(Timeline::new_sequence(EDIT_TIMELINE).name())
        .map(|range| range.max().as_i64())
        .unwrap_or(0)
}

pub fn load_db(path: &Path) -> Result<EntityDb, StoreError> {
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
