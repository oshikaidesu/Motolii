
use serde::{Deserialize, Serialize};

use crate::StoreError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MaskId(pub u32);

impl std::fmt::Display for MaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskMode {
    Add,
    Subtract,
    Intersect,
    Lighten,
    Darken,
    Difference,
}

impl Default for MaskMode {
    fn default() -> Self {
        Self::Add
    }
}

impl MaskMode {
    pub fn to_enum_value(self) -> i64 {
        match self {
            MaskMode::Add => 0,
            MaskMode::Subtract => 1,
            MaskMode::Intersect => 2,
            MaskMode::Lighten => 3,
            MaskMode::Darken => 4,
            MaskMode::Difference => 5,
        }
    }

    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(MaskMode::Add),
            1 => Some(MaskMode::Subtract),
            2 => Some(MaskMode::Intersect),
            3 => Some(MaskMode::Lighten),
            4 => Some(MaskMode::Darken),
            5 => Some(MaskMode::Difference),
            _ => None,
        }
    }
}

impl crate::PropertyId {
    pub fn mask_mode(mask: MaskId) -> Self {
        Self::mask_attr_property(mask, "mode")
    }

    pub fn mask_inverted(mask: MaskId) -> Self {
        Self::mask_attr_property(mask, "inverted")
    }

    fn mask_attr_property(mask: MaskId, attr: &str) -> Self {
        let name = format!("{}{mask}.{attr}", crate::property::MASK_PREFIX);
        Self::new(&name).expect("マスクの property 名は予約語でも空でもない")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mask {
    pub id: MaskId,
    #[serde(default)]
    pub mode: MaskMode,
    pub inverted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedMask {
    pub mode: MaskMode,
    pub inverted: bool,
    pub opacity: f32,
    pub expansion: f64,
    pub shape: motolii_eval::Path,
}

pub(crate) fn validate_unique_ids(masks: &[Mask]) -> Result<(), StoreError> {
    for (i, mask) in masks.iter().enumerate() {
        if masks[..i].iter().any(|other| other.id == mask.id) {
            return Err(StoreError::Property(format!(
                "マスク id {} が2枚ある。形状トラック `mask.{}.shape` がどちらの物か決まらない",
                mask.id, mask.id
            )));
        }
    }
    Ok(())
}
