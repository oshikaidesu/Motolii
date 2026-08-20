//! Document 本体 — 書き口1本 + undo/redo。

use std::sync::Arc;

use re_chunk::{Chunk, RowId};
use re_entity_db::EntityDb;
use re_log_types::{
    AbsoluteTimeRange, EntityPath, StoreId, StoreKind, TimePoint, Timeline, TimelineName,
};
use re_types_core::SerializedComponentBatch;

use crate::components::{descriptor_meta, descriptor_present, descriptor_track, LayerPresent, TrackJson};
use crate::view::StoreView;
use crate::{StoreError, EDIT_TIMELINE};

/// layer の安定 ID。entity path はこれ1つから決まる。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(pub u64);

impl LayerId {
    pub fn entity_path(self) -> EntityPath {
        EntityPath::from(format!("/layer/{}", self.0))
    }
}

/// property の名前。AE の property list の1行に相当する。
///
/// 構築時に component 識別子まで解決しておく。`ComponentIdentifier` は空文字を拒む
/// interned 型なので、**検証を境界で1回だけ**行い、以降は失敗し得ない形にする。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropertyId {
    name: String,
    component: re_types_core::ComponentIdentifier,
}

impl PropertyId {
    pub fn new(name: &str) -> Result<Self, StoreError> {
        let component = re_types_core::ComponentIdentifier::try_new(format!("Layer:{name}"))
            .map_err(|e| StoreError::Property(e.to_string()))?;
        Ok(Self {
            name: name.to_owned(),
            component,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn component(&self) -> re_types_core::ComponentIdentifier {
        self.component
    }
}

/// 編集の意図。**Document を書き換える道はこれだけ**。
#[derive(Clone, Debug)]
pub enum Intent {
    AddLayer(LayerId),
    /// 墓標を立てるだけで、chunk は落とさない(落とすと undo で戻せない)。
    RemoveLayer(LayerId),
    SetTrack {
        layer: LayerId,
        property: PropertyId,
        track: motolii_eval::KeyframeTrack,
    },
    /// 素材と重ね順。アニメーションしない属性はこちら。
    SetMeta {
        layer: LayerId,
        meta: crate::LayerMeta,
    },
}

pub struct Document {
    db: EntityDb,
    /// 現在の edit 位置。0 = 空の Document。
    head: i64,
    /// 到達済みの最大 edit 位置。redo の上限。
    tip: i64,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            db: EntityDb::new(StoreId::random(StoreKind::Recording, "motolii")),
            head: 0,
            tip: 0,
        }
    }

    fn timeline() -> Timeline {
        Timeline::new_sequence(EDIT_TIMELINE)
    }

    fn timeline_name() -> TimelineName {
        *Self::timeline().name()
    }

    /// 読み手が受け取る唯一の物。可変ハンドルは外へ出さない。
    pub fn view(&self) -> StoreView<'_> {
        StoreView::new(&self.db, self.head)
    }

    pub fn edit_head(&self) -> i64 {
        self.head
    }

    pub fn can_undo(&self) -> bool {
        self.head > 0
    }

    pub fn can_redo(&self) -> bool {
        self.head < self.tip
    }

    /// **時間を戻すだけ**。store からは何も失われない。
    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.head -= 1;
            true
        } else {
            false
        }
    }

    /// **時間を進めるだけ**。
    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.head += 1;
            true
        } else {
            false
        }
    }

    /// 唯一の書き口。
    ///
    /// undo 後に新しい編集をしたら redo 空間を落とす — rerun blueprint と同じ規則
    /// (`re_viewer_context/src/undo.rs`: "When editing, we first drop all data after
    /// the current time.")。
    pub fn apply(&mut self, intent: Intent) -> Result<(), StoreError> {
        if self.head < self.tip {
            self.db.drop_time_range(
                &Self::timeline_name(),
                AbsoluteTimeRange::new(self.head + 1, self.tip),
                // 上流が undo/redo スタック操作のために用意している変種をそのまま使う。
                re_chunk_store::ChunkDeletionReason::ExplicitDrop,
            );
            self.tip = self.head;
        }

        let at = self.head + 1;
        let batches = match intent {
            Intent::AddLayer(layer) => (
                layer,
                vec![serialize_present(true)?],
            ),
            Intent::RemoveLayer(layer) => (layer, vec![serialize_present(false)?]),
            Intent::SetMeta { layer, meta } => {
                let json = serde_json::to_string(&meta)?;
                (
                    layer,
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_meta(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetTrack {
                layer,
                property,
                track,
            } => {
                let json = serde_json::to_string(&track)?;
                (
                    layer,
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
        };

        let (layer, batches) = batches;
        let chunk = Chunk::builder(layer.entity_path())
            .with_serialized_batches(
                RowId::new(),
                TimePoint::default().with(Self::timeline(), at),
                batches,
            )
            .build()
            .map_err(|e| StoreError::Chunk(e.to_string()))?;

        self.db
            .add_chunk(&Arc::new(chunk))
            .map_err(|e| StoreError::Ingest(e.to_string()))?;

        self.head = at;
        self.tip = at;
        Ok(())
    }

    /// 実測用。製品経路ではない。
    pub fn store_bytes(&self) -> u64 {
        self.db.byte_size_of_physical_chunks()
    }

    /// 実測用。製品経路ではない。
    pub fn store_chunks(&self) -> usize {
        self.db.num_physical_chunks()
    }
}

fn serialize_present(present: bool) -> Result<SerializedComponentBatch, StoreError> {
    Ok(SerializedComponentBatch {
        descriptor: descriptor_present(),
        array: <LayerPresent as re_types_core::Loggable>::to_arrow([LayerPresent(present)])
            .map_err(|e| StoreError::Chunk(e.to_string()))?,
    })
}
