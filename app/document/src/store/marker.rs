
use serde::{Deserialize, Serialize};

use crate::doc::core::RationalTime;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub name: String,
    pub time: RationalTime,
    pub duration: RationalTime,
    /// 本文。Timeline は名前だけを見せ、机が全文を見せる。同じ物を別の長さで覗く。
    #[serde(default)]
    pub body: String,
}
