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
use crate::stable_id::EffectId;
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
        Soundtrack::try_new(raw.asset, raw.start_offset, raw.master_gain).map_err(de::Error::custom)
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
#[allow(clippy::large_enum_variant)] // Clip は VectorRecipe 込みで大きい。Box化は API 全域に波及するため v1 では許容。
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

fn default_false() -> bool {
    false
}

/// クリップ/グループ共通の項目エンベロープ(concept 2026-07-10)。
///
/// `visible`/`solo`/`lock` は B④ 3軸表(決定パック採択)。serde default で追加的。
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
    /// 自身の描画除外。依存先(parent/mask/LookAt)としては評価可(B④)。
    #[serde(default = "default_true")]
    pub visible: bool,
    /// 描画フィルタ。文書内に1つでも true があればソロ集合のみ描画(B④)。
    #[serde(default = "default_false")]
    pub solo: bool,
    /// 編集禁止のみ。評価・描画に影響しない(B④)。
    #[serde(default = "default_false")]
    pub lock: bool,
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
            visible: true,
            solo: false,
            lock: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct EffectInstance {
    /// document-local安定ID(A8)。不変・非再利用。複製時は新規採番(D2)。
    /// 旧形式(id無し)は拒否 — 変換はD1e(D1g/D1i-1と同型の方針)。
    pub id: EffectId,
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

/// 永続するmedia stream選択(kind + container内ordinal)。
///
/// content hashが同じAssetなら同じordinalを同じstreamとみなす。
/// 欠落時は別streamへfallbackせずtyped error(AG-1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, DeserializeDerive)]
pub struct StreamSelector {
    pub kind: StreamKind,
    pub ordinal: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum StreamKind {
    Video,
    Audio,
}

/// Asset Clipのvideo component(0または1)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, DeserializeDerive)]
pub struct VideoComponent {
    pub stream: StreamSelector,
}

impl VideoComponent {
    pub fn ordinal(ordinal: u32) -> Self {
        Self {
            stream: StreamSelector {
                kind: StreamKind::Video,
                ordinal,
            },
        }
    }
}

/// source範囲外の音声挙動。videoの`Freeze`/`Black`語彙は流用しない(AG-1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum AudioOutOfRange {
    #[default]
    Silence,
    Loop,
}

fn default_audio_gain() -> DocParam {
    DocParam::const_f64(1.0)
}

/// Asset Clipのaudio component(0以上)。
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct AudioComponent {
    pub stream: StreamSelector,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// linear・有限・0以上。上限は設けない(masteringしない — AG-1)。
    #[serde(default = "default_audio_gain")]
    pub gain: DocParam,
    #[serde(default)]
    pub out_of_range: AudioOutOfRange,
}

impl AudioComponent {
    pub fn ordinal(ordinal: u32) -> Self {
        Self {
            stream: StreamSelector {
                kind: StreamKind::Audio,
                ordinal,
            },
            enabled: true,
            gain: default_audio_gain(),
            out_of_range: AudioOutOfRange::Silence,
        }
    }
}

/// 旧`ClipSource::Asset { asset }`欠落時のdefault: video ordinal 0 / audioなし。
fn default_asset_video() -> Option<VideoComponent> {
    Some(VideoComponent::ordinal(0))
}

fn is_legacy_default_video(video: &Option<VideoComponent>) -> bool {
    matches!(
        video,
        Some(VideoComponent {
            stream: StreamSelector {
                kind: StreamKind::Video,
                ordinal: 0,
            },
        })
    )
}

/// 新しいnested fieldを含むAsset Clipか(旧readerでの再保存消失を防ぐ判定)。
pub fn asset_components_require_newer_reader(
    video: &Option<VideoComponent>,
    audio: &[AudioComponent],
) -> bool {
    !audio.is_empty() || !is_legacy_default_video(video)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ClipSource {
    /// raster / 汎用アセット。未知フィールド(recipe/modifiers等)は拒否(S6)。
    ///
    /// `video`/`audio`欠落は旧形式互換default(video ordinal 0 / audioなし)。
    Asset {
        asset: AssetId,
        #[serde(
            default = "default_asset_video",
            skip_serializing_if = "is_legacy_default_video"
        )]
        video: Option<VideoComponent>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        audio: Vec<AudioComponent>,
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
    /// ベクトルソース。modifiers はここにしか存在しない(S6 / D1i-1)。
    Vector { recipe: VectorRecipe },
}

impl ClipSource {
    /// 旧形式互換のAsset Clip(video ordinal 0 / audioなし)。
    pub fn asset_video_only(asset: AssetId) -> Self {
        Self::Asset {
            asset,
            video: default_asset_video(),
            audio: Vec::new(),
        }
    }
}

#[derive(DeserializeDerive)]
struct ClipSourceAssetDe {
    asset: AssetId,
    #[serde(default = "default_asset_video")]
    video: Option<VideoComponent>,
    #[serde(default)]
    audio: Vec<AudioComponent>,
}

#[derive(DeserializeDerive)]
struct ClipSourceVectorDe {
    recipe: VectorRecipe,
}

#[derive(DeserializeDerive)]
struct ClipSourcePluginDe {
    plugin_id: String,
    #[serde(default = "default_effect_version")]
    effect_version: u32,
    #[serde(default)]
    params: BTreeMap<String, DocParam>,
    #[serde(default, flatten)]
    extra: Map<String, JsonValue>,
}

impl<'de> Deserialize<'de> for ClipSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Plugin の flatten extra と Asset/Vector の厳格拒否を同居させるため、
        // tag を見てから variant ごとに Map を渡す(同一 enum への deny_unknown_fields+flatten は serde 不可)。
        let mut map = Map::<String, JsonValue>::deserialize(deserializer)?;
        let tag = map
            .remove("source")
            .ok_or_else(|| de::Error::missing_field("source"))?;
        let tag = tag
            .as_str()
            .ok_or_else(|| de::Error::custom("ClipSource.source must be a string"))?;
        match tag {
            "asset" => {
                reject_unknown_clip_source_fields(&map, &["asset", "video", "audio"])?;
                let de: ClipSourceAssetDe =
                    serde_json::from_value(JsonValue::Object(map)).map_err(de::Error::custom)?;
                Ok(Self::Asset {
                    asset: de.asset,
                    video: de.video,
                    audio: de.audio,
                })
            }
            "vector" => {
                // audio/video は Asset 専用。unknown field 拒否で組合せを弾く。
                reject_unknown_clip_source_fields(&map, &["recipe"])?;
                let de: ClipSourceVectorDe =
                    serde_json::from_value(JsonValue::Object(map)).map_err(de::Error::custom)?;
                Ok(Self::Vector { recipe: de.recipe })
            }
            "plugin" => {
                // flatten extra へ落として黙殺しない(AG-1: 不正source/component組合せ)。
                if map.contains_key("audio") || map.contains_key("video") {
                    return Err(de::Error::custom(
                        "ClipSource::Plugin cannot carry video/audio components; only Asset sources may",
                    ));
                }
                let de: ClipSourcePluginDe =
                    serde_json::from_value(JsonValue::Object(map)).map_err(de::Error::custom)?;
                Ok(Self::Plugin {
                    plugin_id: de.plugin_id,
                    effect_version: de.effect_version,
                    params: de.params,
                    extra: de.extra,
                })
            }
            other => Err(de::Error::unknown_variant(
                other,
                &["asset", "plugin", "vector"],
            )),
        }
    }
}

fn reject_unknown_clip_source_fields<E: de::Error>(
    map: &Map<String, JsonValue>,
    allowed: &'static [&'static str],
) -> Result<(), E> {
    for key in map.keys() {
        if !allowed.iter().any(|a| *a == key) {
            return Err(E::unknown_field(key, allowed));
        }
    }
    Ok(())
}

/// Vector系ソースのレシピ。modifiers は root の全パス集合に index 0 から順に作用。
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct VectorRecipe {
    pub content: VectorContent,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<PathOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VectorContent {
    StandardShape {
        #[serde(flatten)]
        shape: StandardShape,
    },
    SvgAsset {
        asset: AssetId,
    },
    TextPath {
        text: String,
        font_asset: AssetId,
    },
    /// パス合成用ネスト(タイムライン`TrackItem::Group`とは別概念)。
    Group {
        children: Vec<VectorContent>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum StandardShape {
    Rect { width: DocParam, height: DocParam },
    Ellipse { width: DocParam, height: DocParam },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Clip {
    pub envelope: ItemEnvelope,
    pub start: RationalTime,
    pub duration: RationalTime,
    #[serde(default)]
    pub time_map: TimeMap,
    pub source: ClipSource,
}

#[derive(DeserializeDerive)]
struct ClipDe {
    envelope: ItemEnvelope,
    start: RationalTime,
    duration: RationalTime,
    #[serde(default)]
    time_map: TimeMap,
    source: ClipSource,
    /// 旧形式。キーが存在するだけで拒否(null 含む。不在との区別が必要 — D1i-1 follow-up)。
    #[serde(default)]
    path_ops: LegacyPathOpsField,
}

/// `Option<JsonValue>` だと `"path_ops": null` が不在と同じ `None` になり拒否を迂回するため、
/// キー存在を保持する(値自体は見ない)。
#[derive(Debug, Clone, Default)]
enum LegacyPathOpsField {
    #[default]
    Absent,
    Present,
}

impl<'de> Deserialize<'de> for LegacyPathOpsField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // null / 配列 / オブジェクトいずれも「キーが在る」ことだけが拒否条件
        let _ = JsonValue::deserialize(deserializer)?;
        Ok(Self::Present)
    }
}

impl<'de> Deserialize<'de> for Clip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = ClipDe::deserialize(deserializer)?;
        if !matches!(raw.path_ops, LegacyPathOpsField::Absent) {
            return Err(serde::de::Error::custom(
                "legacy field `path_ops` is not supported; use ClipSource::Vector { recipe.modifiers } (D1i-1). Migration is D1e",
            ));
        }
        Ok(Self {
            envelope: raw.envelope,
            start: raw.start,
            duration: raw.duration,
            time_map: raw.time_map,
            source: raw.source,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
pub struct Group {
    pub envelope: ItemEnvelope,
    pub children: Vec<TrackItem>,
}

/// Trim の適用モード(Lottie parallel/sequential。意味論は D1i-2)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    #[default]
    Parallel,
    Sequential,
}

/// ZigZag の頂点形状(D1i-2 PathOp意味論表)。デフォルトは表が固定しないため
/// 「Zig Zag」の字面どおり鋭角側を既定にする(便利デフォルトの発明ではなく命名からの素直な選択)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum PointType {
    #[default]
    Corner,
    Smooth,
}

/// Offset の線結合スタイル(Clipper2 offset準拠。D1i-2 PathOp意味論表)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

fn default_miter_limit() -> f64 {
    4.0
}

/// Repeaterのコピー合成順(Lottie `rp.m`: 1=Above/2=Below)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, DeserializeDerive)]
#[serde(rename_all = "snake_case")]
pub enum CompositeOrder {
    #[default]
    Above,
    Below,
}

fn default_repeater_transform() -> Transform2D {
    Transform2D::identity()
}

fn default_full_opacity() -> DocParam {
    DocParam::const_f64(1.0)
}

/// v1閉集合のパス演算子(プラグイン契約には出さない。F-13)。
/// 意味・単位・範囲の正本は docs/specs/M2-document-model.md「PathOp意味論表」。
/// 【決定】2026-07-13(Lottie/AE採択)。
///
/// `line_join`/`miter_limit`/`point_type`は`TrimMode`と同格の非キーフレーム様式席(生の値)、
/// それ以外の数値・空間パラメータは通常のDocParam(キーフレーム/リンク駆動)。
/// `Wiggle.seed`は再現性のための固定`u64`であり、時間駆動のDocParamにはしない(意味論表)。
/// `Twist.center`は表が「必須」と定めるため`default`を持たない — 旧JSON(center無し)は
/// 型付き拒否になり、変換はD1e migrationの担当(D1g/D1i-1と同じ「拒否→D1e変換」の型)。
#[derive(Debug, Clone, PartialEq, Serialize, DeserializeDerive)]
#[serde(tag = "op", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Repeaterは完全なTransform2D+opacity2本を持ち大きい。TrackItemと同様、v1ではBox化しない。
pub enum PathOp {
    PuckerBloat {
        amount: DocParam,
    },
    ZigZag {
        amount: DocParam,
        ridges: DocParam,
        #[serde(default)]
        point_type: PointType,
    },
    Offset {
        distance: DocParam,
        #[serde(default)]
        line_join: LineJoin,
        #[serde(default = "default_miter_limit")]
        miter_limit: f64,
    },
    RoundCorners {
        radius: DocParam,
    },
    Trim {
        start: DocParam,
        end: DocParam,
        offset: DocParam,
        #[serde(default)]
        mode: TrimMode,
    },
    Twist {
        angle: DocParam,
        center: DocParam,
    },
    Wiggle {
        amp: DocParam,
        freq: DocParam,
        seed: u64,
    },
    Repeater {
        copies: DocParam,
        offset: DocParam,
        #[serde(default = "default_repeater_transform")]
        transform: Transform2D,
        #[serde(default)]
        composite: CompositeOrder,
        #[serde(default = "default_full_opacity")]
        start_opacity: DocParam,
        #[serde(default = "default_full_opacity")]
        end_opacity: DocParam,
    },
}
