
use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, StringArray};
use arrow::datatypes::DataType;
use re_byte_size::SizeBytes;
use re_types_core::{
    Component, ComponentDescriptor, ComponentType, DeserializationResult, Loggable,
    SerializationResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackJson(pub String);

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

pub(crate) fn archetype_layer() -> &'static str {
    "motolii.archetypes.Layer"
}

pub(crate) fn archetype_composition() -> &'static str {
    "motolii.archetypes.Composition"
}

pub(crate) fn descriptor_track(property: &crate::doc::store::PropertyId) -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: property.component(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_meta() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:meta".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_masks() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:masks".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_composition() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_composition().into()),
        component: "Composition:settings".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_markers() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_composition().into()),
        component: "Composition:markers".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_slots() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_composition().into()),
        component: "Composition:slots".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_assets() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_composition().into()),
        component: "Composition:assets".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_present() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:present".into(),
        component_type: Some(LayerPresent::name()),
    }
}

pub(crate) fn descriptor_attrs() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:attrs".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_effects() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:effects".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_shapes() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:shapes".into(),
        component_type: Some(TrackJson::name()),
    }
}

pub(crate) fn descriptor_text() -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(archetype_layer().into()),
        component: "Layer:text".into(),
        component_type: Some(TrackJson::name()),
    }
}
