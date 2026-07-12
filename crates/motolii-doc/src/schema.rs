//! D1aスキーマ本体: コンポ/トラック/クリップ/グループ/エンベロープ。
//!
//! `CompCamera`はここに入れない(#55 / 監査C-7)。入れる判断はCQ-5が先。

use std::collections::BTreeMap;

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeDerive, Serialize};
use serde_json::{Map, Value as JsonValue};

use motolii_core::{Fps, RationalTime, TimeMap};

use crate::asset::AssetId;
use crate::param::DocParam;
use crate::track_id::TrackId;
use crate::LayerId;

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CompositionError {
    #[error("aspect numerator must be positive, got {0}")]
    NonPositiveAspectNum(i64),
    #[error("aspect denominator must be positive, got {0}")]
    NonPositiveAspectDen(i64),
}

/// コンポ設定。高さは正準空間で常に1.0 — 幅は有理数アスペクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Composition {
    /// 正準幅 = aspect_num/aspect_den(既約・正)。高さは1.0固定。
    aspect_num: i64,
    aspect_den: i64,
    pub duration: RationalTime,
    pub fps: Fps,
}

#[derive(DeserializeDerive)]
struct RawComposition {
    aspect_num: i64,
    aspect_den: i64,
    duration: RationalTime,
    fps: Fps,
}

impl<'de> Deserialize<'de> for Composition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawComposition::deserialize(deserializer)?;
        Composition::try_new(raw.aspect_num, raw.aspect_den, raw.duration, raw.fps)
            .map_err(de::Error::custom)
    }
}

impl Composition {
    pub fn try_new(
        aspect_num: i64,
        aspect_den: i64,
        duration: RationalTime,
        fps: Fps,
    ) -> Result<Self, CompositionError> {
        if aspect_num <= 0 {
            return Err(CompositionError::NonPositiveAspectNum(aspect_num));
        }
        if aspect_den <= 0 {
            return Err(CompositionError::NonPositiveAspectDen(aspect_den));
        }
        let g = gcd(aspect_num as u64, aspect_den as u64).max(1);
        Ok(Self {
            aspect_num: aspect_num / g as i64,
            aspect_den: aspect_den / g as i64,
            duration,
            fps,
        })
    }

    pub fn new_v1() -> Self {
        Self::try_new(
            16,
            9,
            RationalTime::try_new(10, 1).expect("10/1"),
            Fps::try_new(30, 1).expect("30/1"),
        )
        .expect("16/9")
    }

    pub fn aspect_num(self) -> i64 {
        self.aspect_num
    }

    pub fn aspect_den(self) -> i64 {
        self.aspect_den
    }
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum SoundtrackError {
    #[error("master_gain must be finite and in [0, 1], got {0}")]
    InvalidGain(f64),
}

/// プロジェクト直下の楽曲1本(concept)。
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Soundtrack {
    pub asset: AssetId,
    pub start_offset: RationalTime,
    master_gain: f64,
}

#[derive(DeserializeDerive)]
struct RawSoundtrack {
    asset: AssetId,
    start_offset: RationalTime,
    master_gain: f64,
}

impl<'de> Deserialize<'de> for Soundtrack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawSoundtrack::deserialize(deserializer)?;
        Soundtrack::try_new(raw.asset, raw.start_offset, raw.master_gain)
            .map_err(de::Error::custom)
    }
}

impl Soundtrack {
    pub fn try_new(
        asset: AssetId,
        start_offset: RationalTime,
        master_gain: f64,
    ) -> Result<Self, SoundtrackError> {
        if !master_gain.is_finite() || !(0.0..=1.0).contains(&master_gain) {
            return Err(SoundtrackError::InvalidGain(master_gain));
        }
        Ok(Self {
            asset,
            start_offset,
            master_gain,
        })
    }

    pub fn master_gain(self) -> f64 {
        self.master_gain
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct Track {
    pub id: TrackId,
    pub items: Vec<TrackItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrackItem {
    Clip(Clip),
    Group(Group),
}

fn default_true() -> bool {
    true
}

fn default_effect_version() -> u32 {
    1
}

fn default_opacity() -> DocParam {
    DocParam::const_f64(1.0)
}

/// クリップ/グループ共通の項目エンベロープ(concept 2026-07-10)。
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct ItemEnvelope {
    pub layer_id: LayerId,
    #[serde(default)]
    pub effects: Vec<EffectInstance>,
    pub transform: Transform2D,
    #[serde(default)]
    pub clipping_mask: ClippingMaskSettings,
    #[serde(default)]
    pub blend: BlendMode,
    #[serde(default = "default_opacity")]
    pub opacity: DocParam,
}

impl ItemEnvelope {
    pub fn new(layer_id: LayerId) -> Self {
        Self {
            layer_id,
            effects: Vec::new(),
            transform: Transform2D::identity(),
            clipping_mask: ClippingMaskSettings::default(),
            blend: BlendMode::Normal,
            opacity: default_opacity(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct EffectInstance {
    pub plugin_id: String,
    #[serde(default = "default_effect_version")]
    pub effect_version: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub params: BTreeMap<String, DocParam>,
    /// 未知フィールド保持(F-9の席。警告はD1f)。
    #[serde(default, flatten)]
    pub extra: Map<String, JsonValue>,
}

/// 正準空間の2D変形。親参照はスキーマ予約。
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct Transform2D {
    pub position: DocParam,
    pub anchor: DocParam,
    pub scale: DocParam,
    /// ラジアン。
    pub rotation: DocParam,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<LayerId>,
}

impl Transform2D {
    pub fn identity() -> Self {
        Self {
            position: DocParam::const_vec2([0.0, 0.0]),
            anchor: DocParam::const_vec2([0.0, 0.0]),
            scale: DocParam::const_vec2([1.0, 1.0]),
            rotation: DocParam::const_f64(0.0),
            parent: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum MaskMode {
    Alpha,
    Luminance,
    InvertAlpha,
    InvertLuminance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, DeserializeDerive)]
pub struct ClippingMaskSettings {
    pub enabled: bool,
    pub mode: MaskMode,
}

impl Default for ClippingMaskSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: MaskMode::Alpha,
        }
    }
}

/// doc所有のブレンド語彙(nodesのGPU実装とは独立)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    #[default]
    Normal,
    Add,
    Multiply,
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ClipSource {
    Asset {
        asset: AssetId,
    },
    Plugin {
        plugin_id: String,
        #[serde(default = "default_effect_version")]
        effect_version: u32,
        #[serde(default)]
        params: BTreeMap<String, DocParam>,
        /// 未知フィールド保持(F-9の席。警告はD1f)。
        #[serde(default, flatten)]
        extra: Map<String, JsonValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct Clip {
    pub envelope: ItemEnvelope,
    pub start: RationalTime,
    pub duration: RationalTime,
    #[serde(default)]
    pub time_map: TimeMap,
    pub source: ClipSource,
    #[serde(default)]
    pub path_ops: Vec<PathOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct Group {
    pub envelope: ItemEnvelope,
    pub children: Vec<TrackItem>,
}

/// v1閉集合のパス演算子(プラグイン契約には出さない。F-13)。
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PathOp {
    PuckerBloat {
        amount: DocParam,
    },
    ZigZag {
        amount: DocParam,
        ridges: DocParam,
    },
    Offset {
        distance: DocParam,
    },
    RoundCorners {
        radius: DocParam,
    },
    Trim {
        start: DocParam,
        end: DocParam,
        offset: DocParam,
    },
    Twist {
        angle: DocParam,
    },
    Wiggle {
        amp: DocParam,
        freq: DocParam,
        seed: DocParam,
    },
    Repeater {
        copies: DocParam,
        offset: DocParam,
    },
}
