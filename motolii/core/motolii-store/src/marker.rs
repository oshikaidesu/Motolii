
use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub name: String,
    pub time: RationalTime,
    pub duration: RationalTime,
}
