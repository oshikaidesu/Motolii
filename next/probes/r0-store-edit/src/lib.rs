//! owns: R0 probe の測定器具。rerun の bench は「観測ログの追記」を測るもので、
//!       「編集ソフトが同じ property を何百回も書き換える」パターンを測る物は上流に無い。
//!
//! ここで測るのは1点だけ: **rerun store を Document の実体にできるか**。
//! 落ちたら軸(2026-08-20 リセット裁定)が立たないので、移植を1行も始める前に回す。
//!
//! 表現は裁定4のとおり `re_types_core` の custom component として **Motolii 側に**建てる
//! (`re_types` を fork しない)。この crate がその実証も兼ねる。

use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Float32Array, Int64Array};
use arrow::datatypes::DataType;
use re_byte_size::SizeBytes;
use re_chunk::{Chunk, RowId};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, StoreId, StoreKind, TimePoint, Timeline, TimelineName};
use re_types_core::{
    Component, ComponentDescriptor, ComponentType, DeserializationResult, Loggable,
    SerializationResult, SerializedComponentBatch,
};

// ---------------------------------------------------------------------------
// timeline 2本
// ---------------------------------------------------------------------------

/// 編集履歴。undo/redo はこの軸の latest-at 移動そのもの(裁定2)。
pub fn edit_timeline() -> Timeline {
    Timeline::new_sequence("edit")
}

/// 作品時間。**R0-A で「Document をこの軸に置くと undo が成立しない」ことを確かめる**ために
/// だけ使う。裁定後の設計では Document はこの軸に載らない(keyframe は値の側に持つ)。
pub fn comp_timeline() -> Timeline {
    Timeline::new_sequence("comp")
}

// ---------------------------------------------------------------------------
// Motolii 側で建てた custom component(裁定4の実証)
// ---------------------------------------------------------------------------

/// keyframe の打点(フレーム番号)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyFrame(pub i64);

/// keyframe の値。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyValue(pub f32);

impl SizeBytes for KeyFrame {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        0
    }
}

impl SizeBytes for KeyValue {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        0
    }
}

impl<'a> From<KeyFrame> for Cow<'a, KeyFrame> {
    fn from(v: KeyFrame) -> Self {
        Self::Owned(v)
    }
}

impl<'a> From<&'a KeyFrame> for Cow<'a, KeyFrame> {
    fn from(v: &'a KeyFrame) -> Self {
        Self::Borrowed(v)
    }
}

impl<'a> From<KeyValue> for Cow<'a, KeyValue> {
    fn from(v: KeyValue) -> Self {
        Self::Owned(v)
    }
}

impl<'a> From<&'a KeyValue> for Cow<'a, KeyValue> {
    fn from(v: &'a KeyValue) -> Self {
        Self::Borrowed(v)
    }
}

impl Loggable for KeyFrame {
    fn arrow_datatype() -> DataType {
        DataType::Int64
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<Cow<'a, Self>>>>,
    ) -> SerializationResult<ArrayRef>
    where
        Self: 'a,
    {
        let values: Vec<Option<i64>> = data
            .into_iter()
            .map(|v| v.map(|v| v.into().into_owned().0))
            .collect();
        Ok(Arc::new(Int64Array::from(values)))
    }

    fn from_arrow_opt(data: &dyn Array) -> DeserializationResult<Vec<Option<Self>>> {
        let array = data
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| re_types_core::DeserializationError::datatype_mismatch(
                DataType::Int64,
                data.data_type().clone(),
            ))?;
        Ok(array.iter().map(|v| v.map(Self)).collect())
    }
}

impl Loggable for KeyValue {
    fn arrow_datatype() -> DataType {
        DataType::Float32
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<Cow<'a, Self>>>>,
    ) -> SerializationResult<ArrayRef>
    where
        Self: 'a,
    {
        let values: Vec<Option<f32>> = data
            .into_iter()
            .map(|v| v.map(|v| v.into().into_owned().0))
            .collect();
        Ok(Arc::new(Float32Array::from(values)))
    }

    fn from_arrow_opt(data: &dyn Array) -> DeserializationResult<Vec<Option<Self>>> {
        let array = data
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or_else(|| re_types_core::DeserializationError::datatype_mismatch(
                DataType::Float32,
                data.data_type().clone(),
            ))?;
        Ok(array.iter().map(|v| v.map(Self)).collect())
    }
}

impl Component for KeyFrame {
    fn name() -> ComponentType {
        "motolii.KeyFrame".into()
    }
}

impl Component for KeyValue {
    fn name() -> ComponentType {
        "motolii.KeyValue".into()
    }
}

pub fn descriptor_frames() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some("motolii.archetypes.Track".into()),
        component: "Track:frames".into(),
        component_type: Some(KeyFrame::name()),
    }
}

pub fn descriptor_values() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some("motolii.archetypes.Track".into()),
        component: "Track:values".into(),
        component_type: Some(KeyValue::name()),
    }
}

// ---------------------------------------------------------------------------
// store
// ---------------------------------------------------------------------------

pub fn new_store() -> EntityDb {
    EntityDb::new(StoreId::random(StoreKind::Recording, "motolii-r0"))
}

/// property track を **まるごと1行**として `edit` 軸へ書く(裁定後の形)。
///
/// keyframe 1本の編集でも track 全体を書き直す。これが store のメモリを食い潰さないか、
/// が R0-1 の問い。
pub fn write_track(db: &mut EntityDb, path: &str, edit_seq: i64, keys: &[(i64, f32)]) {
    let frames: Vec<KeyFrame> = keys.iter().map(|(f, _)| KeyFrame(*f)).collect();
    let values: Vec<KeyValue> = keys.iter().map(|(_, v)| KeyValue(*v)).collect();

    let chunk = Chunk::builder(EntityPath::from(path))
        .with_serialized_batches(
            RowId::new(),
            TimePoint::default().with(edit_timeline(), edit_seq),
            vec![
                SerializedComponentBatch {
                    descriptor: descriptor_frames(),
                    array: KeyFrame::to_arrow(frames.iter()).expect("frames to arrow"),
                },
                SerializedComponentBatch {
                    descriptor: descriptor_values(),
                    array: KeyValue::to_arrow(values.iter()).expect("values to arrow"),
                },
            ],
        )
        .build()
        .expect("build chunk");

    db.add_chunk(&Arc::new(chunk)).expect("add chunk");
}

/// `edit` 軸の指定時刻で track を読む。undo = ここへ渡す数を戻すだけ。
pub fn read_track(db: &EntityDb, path: &str, at_edit: i64) -> Option<Vec<(i64, f32)>> {
    let query = LatestAtQuery::new(*edit_timeline().name(), at_edit);
    let path = EntityPath::from(path);
    let results = db.latest_at(
        &query,
        &path,
        [descriptor_frames().component, descriptor_values().component],
    );

    let frames = results.component_batch::<KeyFrame>(descriptor_frames().component)?;
    let values = results.component_batch::<KeyValue>(descriptor_values().component)?;
    Some(
        frames
            .into_iter()
            .zip(values)
            .map(|(f, v)| (f.0, v.0))
            .collect(),
    )
}

/// R0-A 専用: 値を `comp` 軸(と `edit` 軸)の両方に打つ「素朴な形」。
pub fn write_scalar_at_comp(db: &mut EntityDb, path: &str, comp_frame: i64, edit_seq: i64, v: f32) {
    let chunk = Chunk::builder(EntityPath::from(path))
        .with_serialized_batches(
            RowId::new(),
            TimePoint::default()
                .with(comp_timeline(), comp_frame)
                .with(edit_timeline(), edit_seq),
            vec![SerializedComponentBatch {
                descriptor: descriptor_values(),
                array: KeyValue::to_arrow([KeyValue(v)]).expect("value to arrow"),
            }],
        )
        .build()
        .expect("build chunk");

    db.add_chunk(&Arc::new(chunk)).expect("add chunk");
}

/// R0-A 専用: `comp` 軸で読む。
pub fn read_scalar_on(db: &EntityDb, path: &str, timeline: Timeline, at: i64) -> Option<f32> {
    let query = LatestAtQuery::new(*timeline.name(), at);
    let path = EntityPath::from(path);
    let results = db.latest_at(&query, &path, [descriptor_values().component]);
    results
        .component_batch::<KeyValue>(descriptor_values().component)
        .and_then(|v| v.first().map(|v| v.0))
}

// ---------------------------------------------------------------------------
// 評価器
// ---------------------------------------------------------------------------

/// owns の実体: comp 時間 → 値。**rerun の latest-at は step 補間しか持たない**ので、
/// AE の keyframe 補間は Motolii 側にしか置けない。R0 では線形だけ測る。
pub fn eval_linear(track: &[(i64, f32)], frame: i64) -> f32 {
    if track.is_empty() {
        return 0.0;
    }
    if frame <= track[0].0 {
        return track[0].1;
    }
    let last = track[track.len() - 1];
    if frame >= last.0 {
        return last.1;
    }
    // 打点は昇順である前提(書き込み側の契約)。
    let index = track.partition_point(|(f, _)| *f <= frame);
    let (f0, v0) = track[index - 1];
    let (f1, v1) = track[index];
    if f1 == f0 {
        return v1;
    }
    let t = (frame - f0) as f32 / (f1 - f0) as f32;
    v0 + (v1 - v0) * t
}

/// 300 打点 × 指定本数の track を作る補助。
pub fn dense_track(keys: i64, seed: f32) -> Vec<(i64, f32)> {
    (0..keys).map(|k| (k, seed + k as f32)).collect()
}

pub fn timeline_names(db: &EntityDb) -> Vec<String> {
    let mut names: Vec<String> = db
        .timelines()
        .keys()
        .map(|name| name.as_str().to_owned())
        .collect();
    names.sort();
    names
}

pub fn store_bytes(db: &EntityDb) -> u64 {
    db.byte_size_of_physical_chunks()
}

pub fn store_chunks(db: &EntityDb) -> usize {
    db.num_physical_chunks()
}

#[allow(dead_code)]
fn _assert_timeline_name_is_used(_: TimelineName) {}
