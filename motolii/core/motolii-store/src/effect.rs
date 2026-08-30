
use serde::{Deserialize, Serialize};

use crate::StoreError;

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

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedEffect {
    pub plugin_id: String,
    pub params: Vec<(String, crate::Value)>,
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
