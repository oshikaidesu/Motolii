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

/// `motolii.archetypes.Layer` の archetype 名。**正本はここ1箇所** — 以前は
/// `components.rs` の各 `descriptor_*` と `persist.rs::flattened()` の両方が
/// ベタ書きしており、名前を変える時に2箇所を揃える必要があった(2026-08-20 の
/// 敵対的レビュー、DRY の指摘)。
pub fn archetype_layer() -> &'static str {
    "motolii.archetypes.Layer"
}

/// `motolii.archetypes.Composition` の archetype 名。同上の理由でここへ寄せる。
pub fn archetype_composition() -> &'static str {
    "motolii.archetypes.Composition"
}

/// property ごとに別の `ComponentIdentifier` を割り当てる。
/// 1 layer = 1 entity、property は component として並ぶ(AE の property list と同じ形)。
pub fn descriptor_track(property: &crate::PropertyId) -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: property.component(),
        component_type: Some(TrackJson::name()),
    }
}

/// layer の素材と重ね順。track と同じく serde 表現を1つの文字列で持つ
/// (符号化の流儀を2つにしない)。
pub fn descriptor_meta() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:meta".into(),
        component_type: Some(TrackJson::name()),
    }
}

/// layer のマスク一覧(キーを打たない部分だけ)。
///
/// **`meta` の中へ入れない** — `SetMeta` は素材と重ね順を丸ごと差し替える口なので、
/// マスクを同居させると「素材を差し替えたらマスクが消えた」が作れる。
/// 形状と不透明度は普通の property track なので、ここには並びと重ね方だけが入る。
pub fn descriptor_masks() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:masks".into(),
        component_type: Some(TrackJson::name()),
    }
}

/// comp の設定。**layer と同じ JSON 経路を使い回す**(符号化の流儀を増やさない)。
pub fn descriptor_composition() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_composition().into()),
        component: "Composition:settings".into(),
        component_type: Some(TrackJson::name()),
    }
}

/// comp のマーカー一覧。**comp の設定とは別 component**にしてある —
/// `SetComposition` は解像度/fps/尺の意味の口であって、マーカーは別の編集操作
/// (`Intent::SetMarkers`)なので同居させない(`SetMeta` がマスクを巻き込まないのと
/// 同じ理由、裁定108(c))。
pub fn descriptor_markers() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_composition().into()),
        component: "Composition:markers".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub fn descriptor_present() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:present".into(),
        component_type: Some(LayerPresent::name()),
    }
}

/// layer の小さな非アニメーション属性(hidden / parent / blend mode / matte / name /
/// auto-orient)。**`meta` の外**(layer-meta 束、裁定108(c) の構造修正)。
pub fn descriptor_attrs() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:attrs".into(),
        component_type: Some(TrackJson::name()),
    }
}

/// layer が持つ effect インスタンスの列(id + plugin id のみ、`layer-meta` 束)。
pub fn descriptor_effects() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:effects".into(),
        component_type: Some(TrackJson::name()),
    }
}

/// shape-layer の図形列(`Vec<motolii_vector::Shape>`)。
pub fn descriptor_shapes() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:shapes".into(),
        component_type: Some(TrackJson::name()),
    }
}

/// text-layer の文字列内容(`layers/text-layer/t`)。範囲スタイル等は `text` 束の仕事。
pub fn descriptor_text() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:text".into(),
        component_type: Some(TrackJson::name()),
    }
}
