
use serde::{Deserialize, Serialize};

use crate::doc::store::StoreError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EffectId(pub u32);

impl std::fmt::Display for EffectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectInstance {
    pub id: EffectId,
    pub plugin_id: String,
}

/// グループに積んだ効果をどこへ掛けるか。効果自身はこれを知らず、doc と render が解く
/// (裁定 2026-09-11: 効果製作者は「1 枚に掛ける」とだけ書く)。単層では意味を持たない。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectScope {
    /// 子それぞれに掛ける。グループは境界にならない(既定)。
    #[default]
    Each,
    /// 子を 1 枚に焼いてから掛ける。
    Whole,
}

impl EffectScope {
    pub const fn enum_value(self) -> i64 {
        match self {
            Self::Each => 0,
            Self::Whole => 1,
        }
    }

    pub fn from_enum_value(value: i64) -> Option<Self> {
        match value {
            0 => Some(Self::Each),
            1 => Some(Self::Whole),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedEffect {
    pub plugin_id: String,
    pub params: Vec<(String, crate::doc::store::Value)>,
    pub scope: EffectScope,
}

pub(crate) fn validate_unique_ids(effects: &[EffectInstance]) -> Result<(), StoreError> {
    for (i, effect) in effects.iter().enumerate() {
        if effects[..i].iter().any(|other| other.id == effect.id) {
            return Err(StoreError::Property(format!(
                "effect id {} が2枚ある",
                effect.id
            )));
        }
    }
    Ok(())
}
