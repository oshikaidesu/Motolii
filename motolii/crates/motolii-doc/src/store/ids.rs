
use re_log_types::EntityPath;
use serde::{Deserialize, Serialize};

use crate::doc::store::StoreError;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct LayerId(pub u64);

impl LayerId {
    /// 層の置き場。`EntityPath` は文字列を組んで部品ごとに intern し hash まで取るので、
    /// 番号から作り直すと 1 コマに層の数 × 読む回数だけ同じ物を作ることになる
    /// (2026-09-21 の標本で、再生の render thread の 5%)。`EntityPath` は中が `Arc` なので、
    /// 一度作れば clone は安い。書類の層の数だけしか増えない。
    pub fn entity_path(self) -> EntityPath {
        thread_local! {
            static PATHS: std::cell::RefCell<std::collections::HashMap<u64, EntityPath>> =
                std::cell::RefCell::new(std::collections::HashMap::new());
        }
        PATHS.with(|paths| {
            if let Some(hit) = paths.borrow().get(&self.0) {
                return hit.clone();
            }
            let built = EntityPath::from(format!("/layer/{}", self.0));
            paths.borrow_mut().insert(self.0, built.clone());
            built
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropertyId {
    name: String,
    component: re_types_core::ComponentIdentifier,
}

impl PropertyId {
    /// 欄の名前。予約語の走査・`Layer:` を足した文字列・その intern は、名前が同じなら
    /// 毎回同じ答えになる。読む側は 1 コマに何百回もこれを作るので、作った物は取っておく
    /// (2026-09-21 の標本で、再生の render thread の 12%)。
    /// 増えるのは書類が持つ欄の名前の種類だけ(効果・マスクを足した分)。
    pub fn new(name: &str) -> Result<Self, StoreError> {
        thread_local! {
            static IDS: std::cell::RefCell<std::collections::HashMap<String, PropertyId>> =
                std::cell::RefCell::new(std::collections::HashMap::new());
        }
        if let Some(hit) = IDS.with(|ids| ids.borrow().get(name).cloned()) {
            return Ok(hit);
        }
        if crate::doc::store::property::RESERVED.contains(&name) {
            return Err(StoreError::Property(format!(
                "`{name}` は layer 自身の component 名なので property に使えない"
            )));
        }
        let component = re_types_core::ComponentIdentifier::try_new(format!("Layer:{name}"))
            .map_err(|e| StoreError::Property(e.to_string()))?;
        let built = Self { name: name.to_owned(), component };
        IDS.with(|ids| ids.borrow_mut().insert(name.to_owned(), built.clone()));
        Ok(built)
    }

    pub fn mask_shape(mask: crate::doc::store::MaskId) -> Self {
        Self::mask_property(mask, "shape")
    }

    pub fn mask_opacity(mask: crate::doc::store::MaskId) -> Self {
        Self::mask_property(mask, "opacity")
    }

    pub fn mask_expansion(mask: crate::doc::store::MaskId) -> Self {
        Self::mask_property(mask, "expansion")
    }

    fn mask_property(mask: crate::doc::store::MaskId, attr: &str) -> Self {
        let name = format!("{}{mask}.{attr}", crate::doc::store::property::MASK_PREFIX);
        Self::new(&name).expect("マスクの property 名は予約語でも空でもない")
    }

    pub fn effect_param(effect: crate::doc::store::EffectId, name: &str) -> Result<Self, StoreError> {
        let property_name = format!("{}{effect}.param.{name}", crate::doc::store::property::EFFECT_PREFIX);
        Self::new(&property_name)
    }

    pub fn effect_enabled(effect: crate::doc::store::EffectId) -> Self {
        let name = format!("{}{effect}.enabled", crate::doc::store::property::EFFECT_PREFIX);
        Self::new(&name).expect("effect の property 名は予約語でも空でもない")
    }

    /// グループに積んだ時の掛け先(`EffectScope` の enum 値)。
    pub fn effect_scope(effect: crate::doc::store::EffectId) -> Self {
        let name = format!("{}{effect}.scope", crate::doc::store::property::EFFECT_PREFIX);
        Self::new(&name).expect("effect の property 名は予約語でも空でもない")
    }

    pub fn text_range_selector_start(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "start")
    }

    pub fn text_range_selector_end(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "end")
    }

    pub fn text_range_selector_offset(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "offset")
    }

    pub fn text_range_selector_max_amount(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_selector_property(range, "max_amount")
    }

    fn text_range_selector_property(range: crate::doc::store::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.selector.{attr}",
            crate::doc::store::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_range_fill_color(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_style_property(range, "fill_color")
    }

    pub fn text_range_line_spacing(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_style_property(range, "line_spacing")
    }

    pub fn text_range_tracking(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_style_property(range, "tracking")
    }

    fn text_range_style_property(range: crate::doc::store::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.style.{attr}",
            crate::doc::store::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_range_origin(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "origin")
    }

    pub fn text_range_opacity(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "opacity")
    }

    pub fn text_range_position(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "position")
    }

    pub fn text_range_rotation(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "rotation")
    }

    pub fn text_range_scale(range: crate::doc::store::TextRangeId) -> Self {
        Self::text_range_transform_property(range, "scale")
    }

    fn text_range_transform_property(range: crate::doc::store::TextRangeId, attr: &str) -> Self {
        let name = format!(
            "{}{range}.transform.{attr}",
            crate::doc::store::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_range_variation_axis(range: crate::doc::store::TextRangeId, tag: &str) -> Self {
        let name = format!(
            "{}{range}.variation.{tag}",
            crate::doc::store::property::TEXT_RANGE_PREFIX
        );
        Self::new(&name).expect("text-range の property 名は予約語でも空でもない")
    }

    pub fn text_style_axis(style: crate::doc::store::TextStyleId, tag: &str) -> Self {
        let name = format!("{}{style}.axis.{tag}", crate::doc::store::property::TEXT_STYLE_PREFIX);
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

    pub fn component(&self) -> re_types_core::ComponentIdentifier {
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
