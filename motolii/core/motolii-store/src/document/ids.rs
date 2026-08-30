
use re_log_types::EntityPath;
use serde::{Deserialize, Serialize};

use crate::StoreError;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LayerId(pub u64);

impl LayerId {
    pub fn entity_path(self) -> EntityPath {
        EntityPath::from(format!("/layer/{}", self.0))
    }
}

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

    pub fn mask_shape(mask: crate::MaskId) -> Self {
        Self::mask_property(mask, "shape")
    }

    pub fn mask_opacity(mask: crate::MaskId) -> Self {
        Self::mask_property(mask, "opacity")
    }

    pub fn mask_expansion(mask: crate::MaskId) -> Self {
        Self::mask_property(mask, "expansion")
    }

    fn mask_property(mask: crate::MaskId, attr: &str) -> Self {
        let name = format!("{}{mask}.{attr}", crate::property::MASK_PREFIX);
        Self::new(&name).expect("マスクの property 名は予約語でも空でもない")
    }

    pub fn effect_param(effect: crate::EffectId, name: &str) -> Result<Self, StoreError> {
        let property_name = format!("{}{effect}.param.{name}", crate::property::EFFECT_PREFIX);
        Self::new(&property_name)
    }

    pub fn effect_enabled(effect: crate::EffectId) -> Self {
        let name = format!("{}{effect}.enabled", crate::property::EFFECT_PREFIX);
        Self::new(&name).expect("effect の property 名は予約語でも空でもない")
    }

    pub fn text_range_selector_start(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "start")
    }

    pub fn text_range_selector_end(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "end")
    }

    pub fn text_range_selector_offset(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "offset")
    }

    pub fn text_range_selector_max_amount(range: crate::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "max_amount")
    }

    fn text_range_selector_property(range: crate::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.selector.{attr}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_range_fill_color(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "fill_color")
    }

    pub fn text_range_stroke_color(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "stroke_color")
    }

    pub fn text_range_stroke_width(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "stroke_width")
    }

    pub fn text_range_line_spacing(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "line_spacing")
    }

    pub fn text_range_tracking(range: crate::TextRangeId) -> Self {
        Self::text_range_style_property(range, "tracking")
    }

    fn text_range_style_property(range: crate::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.style.{attr}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_range_origin(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "origin")
    }

    pub fn text_range_opacity(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "opacity")
    }

    pub fn text_range_position(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "position")
    }

    pub fn text_range_rotation(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "rotation")
    }

    pub fn text_range_scale(range: crate::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "scale")
    }

    fn text_range_transform_property(range: crate::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.transform.{attr}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_range_variation_axis(range: crate::TextRangeId, tag: &str) -> Self {
        let name = format!(
            "{}{range}.variation.{tag}",
            crate::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_style_axis(style: crate::TextStyleId, tag: &str) -> Self {
        let name = format!("{}{style}.axis.{tag}", crate::property::TEXT_STYLE_PREFIX);
        Self::new(&name).expect("text-style の property 名は予約語でも空でもない")
    }

    pub fn camera(name: &str) -> Result<Self, StoreError> {
        let component = re_types_core::ComponentIdentifier::try_new(format!("Composition:{name}"))
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

impl Serialize for PropertyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.name)
    }
}

impl<'de> Deserialize<'de> for PropertyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = <String as Deserialize>::deserialize(deserializer)?;
        PropertyId::new(&name).map_err(serde::de::Error::custom)
    }
}
