//! Document 本体 — 書き口1本 + undo/redo。

use std::sync::Arc;

use re_chunk::{Chunk, RowId};
use re_entity_db::EntityDb;
use re_log_types::{
    AbsoluteTimeRange, EntityPath, StoreId, StoreKind, TimePoint, Timeline, TimelineName,
};
use re_types_core::SerializedComponentBatch;

use crate::components::{descriptor_composition, descriptor_meta, descriptor_present, descriptor_track, LayerPresent, TrackJson};
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
        if crate::property::RESERVED.contains(&name) {
            return Err(StoreError::Property(format!(
                "`{name}` は layer 自身の component 名なので property に使えない"
            )));
        }
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
    /// comp の設定(解像度・fps・尺)。**undo が効く**ので普通の編集と同じ経路。
    SetComposition(crate::Composition),
}

/// 「見えている Document が変わったか」の印。
///
/// store の世代だけでは undo/redo を捉えられないので、edit 位置と一組にしてある。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revision {
    store: re_chunk_store::ChunkStoreGeneration,
    head: i64,
}

pub struct Document {
    db: EntityDb,
    /// 現在の edit 位置。0 = 空の Document。
    head: i64,
    /// 到達済みの最大 edit 位置。redo の上限。
    tip: i64,
    /// undo の底。**ここより前へは戻れない**。
    ///
    /// 起動直後に置いた既定の comp や、project を開いた直後の状態は「編集」ではないので
    /// 戻せてはいけない。戻せると Stage が理由もなく空になる(実際に起きた)。
    floor: i64,
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
            floor: 0,
        }
    }

    /// comp 設定の置き場。layer(`/layer/{id}`)と混ざらない固定の path。
    pub(crate) fn composition_path() -> EntityPath {
        EntityPath::from("/composition")
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

    /// 今の状態を **undo の底**にする。
    ///
    /// 「新規作成した」「project を開いた」の直後に呼ぶ。ここより前は編集ではないので
    /// 戻せない。呼ばないと、起動時に置いた既定値を利用者が undo で消せてしまう。
    pub fn mark_undo_floor(&mut self) {
        self.floor = self.head;
    }

    pub fn can_undo(&self) -> bool {
        self.head > self.floor
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
            Intent::AddLayer(layer) => (layer.entity_path(), vec![serialize_present(true)?]),
            Intent::RemoveLayer(layer) => (layer.entity_path(), vec![serialize_present(false)?]),
            Intent::SetComposition(composition) => {
                let json = serde_json::to_string(&composition)?;
                (
                    Self::composition_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_composition(),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
            Intent::SetMeta { layer, meta } => {
                let json = serde_json::to_string(&meta)?;
                (
                    layer.entity_path(),
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
                    layer.entity_path(),
                    vec![SerializedComponentBatch {
                        descriptor: descriptor_track(&property),
                        array: <TrackJson as re_types_core::Loggable>::to_arrow([TrackJson(json)])
                            .map_err(|e| StoreError::Chunk(e.to_string()))?,
                    }],
                )
            }
        };

        let (path, batches) = batches;
        let chunk = Chunk::builder(path)
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

    /// 変化検出。front がこれを見れば「前回と同じか」が分かるので、
    /// **前回の値を自分で持つ必要が無い**。二重帳簿の入口を1つ塞ぐための口である。
    ///
    /// **上流の `EntityDb::generation` だけでは足りない**(2026-08-20 の敵対的レビュー):
    /// `undo`/`redo` は `head` を動かすだけで store に触らないので generation が変わらず、
    /// **undo しても front が再描画しない**。それでは front が `last_edit_head` を自分で
    /// 持つことになり、塞ぐと言った入口が逆に開く。よって **(store の世代, edit 位置)** を
    /// 一組で返す。
    pub fn revision(&self) -> Revision {
        Revision {
            store: self.db.generation(),
            head: self.head,
        }
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
