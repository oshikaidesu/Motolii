use serde::Deserialize;

/// DTCG v2025.10 交換形の `$value`（color）。
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DtcgColorValue {
    #[serde(rename = "colorSpace")]
    pub color_space: String,
    pub components: [f32; 3],
    pub alpha: f32,
}

/// DTCG v2025.10 交換形の `$value`（dimension）。
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DtcgDimensionValue {
    pub value: f32,
    pub unit: String,
}
