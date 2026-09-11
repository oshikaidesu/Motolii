
use serde::{Deserialize, Serialize};

use crate::doc::store::LayerId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl BlendMode {
    pub fn to_enum_value(self) -> i64 {
        match self {
            BlendMode::Normal => 0,
            BlendMode::Add => 1,
            BlendMode::Multiply => 2,
            BlendMode::Screen => 3,
            BlendMode::Overlay => 4,
            BlendMode::Darken => 5,
            BlendMode::Lighten => 6,
            BlendMode::ColorDodge => 7,
            BlendMode::ColorBurn => 8,
            BlendMode::HardLight => 9,
            BlendMode::SoftLight => 10,
            BlendMode::Difference => 11,
            BlendMode::Exclusion => 12,
            BlendMode::Hue => 13,
            BlendMode::Saturation => 14,
            BlendMode::Color => 15,
            BlendMode::Luminosity => 16,
        }
    }

    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(BlendMode::Normal),
            1 => Some(BlendMode::Add),
            2 => Some(BlendMode::Multiply),
            3 => Some(BlendMode::Screen),
            4 => Some(BlendMode::Overlay),
            5 => Some(BlendMode::Darken),
            6 => Some(BlendMode::Lighten),
            7 => Some(BlendMode::ColorDodge),
            8 => Some(BlendMode::ColorBurn),
            9 => Some(BlendMode::HardLight),
            10 => Some(BlendMode::SoftLight),
            11 => Some(BlendMode::Difference),
            12 => Some(BlendMode::Exclusion),
            13 => Some(BlendMode::Hue),
            14 => Some(BlendMode::Saturation),
            15 => Some(BlendMode::Color),
            16 => Some(BlendMode::Luminosity),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatteMode {
    Alpha,
    InvertedAlpha,
    Luma,
    InvertedLuma,
}

impl MatteMode {
    pub fn to_enum_value(self) -> i64 {
        match self {
            MatteMode::Alpha => 0,
            MatteMode::InvertedAlpha => 1,
            MatteMode::Luma => 2,
            MatteMode::InvertedLuma => 3,
        }
    }

    pub fn from_enum_value(v: i64) -> Option<Self> {
        match v {
            0 => Some(MatteMode::Alpha),
            1 => Some(MatteMode::InvertedAlpha),
            2 => Some(MatteMode::Luma),
            3 => Some(MatteMode::InvertedLuma),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matte {
    pub layer: LayerId,
    pub mode: MatteMode,
}

/// 層の名札の色数。窓の palette と同じ数で回す。
pub const LABEL_PALETTE_LEN: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerProjection {
    #[serde(rename = "2D")]
    TwoD,
    #[serde(rename = "2.5D")]
    TwoPointFiveD,
    #[default]
    #[serde(rename = "3D")]
    ThreeD,
}

impl LayerProjection {
    pub const fn label(self) -> &'static str {
        match self { Self::TwoD => "2D", Self::TwoPointFiveD => "2.5D", Self::ThreeD => "3D" }
    }

    pub const fn next(self) -> Self {
        match self { Self::TwoD => Self::TwoPointFiveD, Self::TwoPointFiveD => Self::ThreeD, Self::ThreeD => Self::TwoD }
    }
}

fn read_projection<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<LayerProjection, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Stored { Projection(LayerProjection), Pinned(bool) }
    Ok(match Stored::deserialize(deserializer)? {
        Stored::Projection(p) => p,
        Stored::Pinned(true) => LayerProjection::TwoD,
        Stored::Pinned(false) => LayerProjection::ThreeD,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerAttrs {
    pub hidden: bool,
    pub parent: Option<LayerId>,
    pub blend_mode: BlendMode,
    pub matte: Option<Matte>,
    #[serde(default)]
    pub clip_to_below: bool,
    pub name: String,
    pub auto_orient: bool,
    #[serde(default, alias = "pinned", deserialize_with = "read_projection")]
    pub projection: LayerProjection,
    pub solo: bool,
    pub locked: bool,
    #[serde(default)]
    pub label_color: Option<u8>,
    #[serde(default)]
    pub frozen: bool,
    /// 3D の素材を平面へ収めるか。**既定は収めない**(AE と逆。裁定 2026-08-30)。
    #[serde(default)]
    pub flatten: bool,
    /// この画を comp の環境(空)にする。AE の Environment Layer。
    /// 3D の網はこの画で照らされ、背景にこの画が敷かれる。重ね順で一番上の 1 枚だけ効く。
    #[serde(default)]
    pub environment: bool,
    /// この層が光を遮る。光は環境(空の一番明るい方向)から来るものとして、影と、ガラスなら透過の色を
    /// 表面を持つ全ての層へ落とす。光源は置かない — 作者は光を奪うだけ(裁定 2026-09-10)。
    #[serde(default)]
    pub blocks_light: bool,
    /// ゴースト: この層を遅れ(フレーム、負なら先)だけずらして見た姿。**実物は 1 つ、ゴーストも 1 つ**。
    /// 複製が要るなら Delay の効果(Repeater)。行は増えず、Timeline に薄い帯として見え、
    /// Stage では掴めない(裁定 2026-09-07)。
    #[serde(default)]
    pub ghost: Option<i64>,
}

impl Default for LayerAttrs {
    fn default() -> Self {
        Self {
            hidden: false,
            parent: None,
            blend_mode: BlendMode::default(),
            matte: None,
            clip_to_below: false,
            name: String::new(),
            auto_orient: false,
            projection: LayerProjection::ThreeD,
            solo: false,
            locked: false,
            label_color: None,
            frozen: false,
            flatten: false,
            environment: false,
            blocks_light: false,
            ghost: None,
        }
    }
}

impl crate::doc::store::PropertyId {
    pub fn solo() -> Self {
        Self::new("solo").expect("`solo` は予約語でも空でもない")
    }

    pub fn hidden() -> Self {
        Self::new("hidden").expect("`hidden` は予約語でも空でもない")
    }

    pub fn blend_mode() -> Self {
        Self::new("blend_mode").expect("`blend_mode` は予約語でも空でもない")
    }

    pub fn matte_mode() -> Self {
        Self::new("matte_mode").expect("`matte_mode` は予約語でも空でもない")
    }
}

impl crate::doc::store::PropertyId {
    pub fn speed() -> Self {
        Self::new("speed").expect("`speed` は予約語でも空でもない")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerAttrsPatch {
    pub hidden: Option<bool>,
    pub parent: Option<Option<LayerId>>,
    pub blend_mode: Option<BlendMode>,
    pub matte: Option<Option<Matte>>,
    pub clip_to_below: Option<bool>,
    pub name: Option<String>,
    pub auto_orient: Option<bool>,
    pub projection: Option<LayerProjection>,
    pub solo: Option<bool>,
    pub locked: Option<bool>,
    pub label_color: Option<Option<u8>>,
    pub flatten: Option<bool>,
    pub environment: Option<bool>,
    pub blocks_light: Option<bool>,
    pub ghost: Option<Option<i64>>,
}

impl LayerAttrsPatch {
    pub(crate) fn apply_to(self, mut current: LayerAttrs) -> LayerAttrs {
        if let Some(v) = self.hidden {
            current.hidden = v;
        }
        if let Some(v) = self.parent {
            current.parent = v;
        }
        if let Some(v) = self.blend_mode {
            current.blend_mode = v;
        }
        if let Some(v) = self.matte {
            current.matte = v;
        }
        if let Some(v) = self.clip_to_below {
            current.clip_to_below = v;
        }
        if let Some(v) = self.name {
            current.name = v;
        }
        if let Some(v) = self.auto_orient {
            current.auto_orient = v;
        }
        if let Some(v) = self.projection {
            current.projection = v;
        }
        if let Some(v) = self.solo {
            current.solo = v;
        }
        if let Some(v) = self.locked {
            current.locked = v;
        }
        if let Some(v) = self.label_color {
            current.label_color = v;
        }
        if let Some(v) = self.flatten {
            current.flatten = v;
        }
        if let Some(v) = self.environment {
            current.environment = v;
        }
        if let Some(v) = self.blocks_light {
            current.blocks_light = v;
        }
        if let Some(v) = self.ghost {
            current.ghost = v;
        }
        current
    }
}

#[cfg(test)]
mod projection_storage_tests {
    use super::*;

    #[test]
    fn old_layer_attrs_default_to_unclipped() {
        let mut json = serde_json::to_value(LayerAttrs::default()).unwrap();
        json.as_object_mut().unwrap().remove("clip_to_below");
        let attrs: LayerAttrs = serde_json::from_value(json).unwrap();
        assert!(!attrs.clip_to_below);
    }

    #[test]
    fn old_pinned_and_absent_attrs_migrate_without_reframing() {
        for (legacy, expected) in [(Some(true), LayerProjection::TwoD), (Some(false), LayerProjection::ThreeD), (None, LayerProjection::ThreeD)] {
            let mut json = serde_json::to_value(LayerAttrs::default()).unwrap();
            json.as_object_mut().unwrap().remove("projection");
            if let Some(pinned) = legacy { json["pinned"] = pinned.into(); }
            let attrs: LayerAttrs = serde_json::from_value(json).unwrap();
            assert_eq!(attrs.projection, expected);
            let saved = serde_json::to_value(attrs).unwrap();
            assert!(saved.get("pinned").is_none());
            assert_eq!(saved["projection"], expected.label());
        }
    }
}
