//! Motolii 側で建てた component(裁定4: `re_types` を fork しない)。
//!
//! track は `KeyframeTrack` の serde 表現をそのまま1つの文字列として持つ。
//! 列指向へ割らないのは、`KeyframeTrack` が既に不変量(時刻昇順・bezier の可分割性)を
//! 型で持っており、**同じ意味を arrow schema として建て直すと正本が2つになる**ため。
//! 代償(サイズ)は `tests/storm.rs` が実測して予算で縛る。

use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, StringArray};
use arrow::datatypes::DataType;
use re_byte_size::SizeBytes;
use re_types_core::{
    Component, ComponentDescriptor, ComponentType, DeserializationResult, Loggable,
    SerializationResult,
};

/// track 1本の serde 表現。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackJson(pub String);

/// layer が今この edit 時点で存在するか。**削除は false の append**であって drop ではない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerPresent(pub bool);

impl SizeBytes for TrackJson {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.0.len() as u64
    }
}

impl SizeBytes for LayerPresent {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        0
    }
}

impl<'a> From<TrackJson> for Cow<'a, TrackJson> {
    fn from(v: TrackJson) -> Self {
        Self::Owned(v)
    }
}

impl<'a> From<&'a TrackJson> for Cow<'a, TrackJson> {
    fn from(v: &'a TrackJson) -> Self {
        Self::Borrowed(v)
    }
}

impl<'a> From<LayerPresent> for Cow<'a, LayerPresent> {
    fn from(v: LayerPresent) -> Self {
        Self::Owned(v)
    }
}

impl<'a> From<&'a LayerPresent> for Cow<'a, LayerPresent> {
    fn from(v: &'a LayerPresent) -> Self {
        Self::Borrowed(v)
    }
}

impl Loggable for TrackJson {
    fn arrow_datatype() -> DataType {
        DataType::Utf8
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<Cow<'a, Self>>>>,
    ) -> SerializationResult<ArrayRef>
    where
        Self: 'a,
    {
        let values: Vec<Option<String>> = data
            .into_iter()
            .map(|v| v.map(|v| v.into().into_owned().0))
            .collect();
        Ok(Arc::new(StringArray::from(values)))
    }

    fn from_arrow_opt(data: &dyn Array) -> DeserializationResult<Vec<Option<Self>>> {
        let array = data
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                re_types_core::DeserializationError::datatype_mismatch(
                    DataType::Utf8,
                    data.data_type().clone(),
                )
            })?;
        Ok(array
            .iter()
            .map(|v| v.map(|v| Self(v.to_owned())))
            .collect())
    }
}

impl Loggable for LayerPresent {
    fn arrow_datatype() -> DataType {
        DataType::Boolean
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<Cow<'a, Self>>>>,
    ) -> SerializationResult<ArrayRef>
    where
        Self: 'a,
    {
        let values: Vec<Option<bool>> = data
            .into_iter()
            .map(|v| v.map(|v| v.into().into_owned().0))
            .collect();
        Ok(Arc::new(BooleanArray::from(values)))
    }

    fn from_arrow_opt(data: &dyn Array) -> DeserializationResult<Vec<Option<Self>>> {
        let array = data
            .as_any()
            .downcast_ref::<BooleanArray>()
            .ok_or_else(|| {
                re_types_core::DeserializationError::datatype_mismatch(
                    DataType::Boolean,
                    data.data_type().clone(),
                )
            })?;
        Ok(array.iter().map(|v| v.map(Self)).collect())
    }
}

impl Component for TrackJson {
    fn name() -> ComponentType {
        "motolii.TrackJson".into()
    }
}

impl Component for LayerPresent {
    fn name() -> ComponentType {
        "motolii.LayerPresent".into()
    }
}

/// property ごとに別の `ComponentIdentifier` を割り当てる。
/// 1 layer = 1 entity、property は component として並ぶ(AE の property list と同じ形)。
pub fn descriptor_track(property: &crate::PropertyId) -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some("motolii.archetypes.Layer".into()),
        component: property.component(),
        component_type: Some(TrackJson::name()),
    }
}

/// layer の素材と重ね順。track と同じく serde 表現を1つの文字列で持つ
/// (符号化の流儀を2つにしない)。
pub fn descriptor_meta() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some("motolii.archetypes.Layer".into()),
        component: "Layer:meta".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub fn descriptor_present() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some("motolii.archetypes.Layer".into()),
        component: "Layer:present".into(),
        component_type: Some(LayerPresent::name()),
    }
}
