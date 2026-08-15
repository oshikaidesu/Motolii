//! 呼び出しごとのrender/eval文脈。

use motolii_core::{CompCamera, Fps, FrameDesc, Quality, RationalTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct TextureRef<'a> {
    pub texture: &'a wgpu::Texture,
    pub desc: FrameDesc,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDriverContext {
    /// サンプル列の開始時刻(タイムライン)。
    pub start: RationalTime,
    /// 総尺。半開区間 `[start, start+duration)` を覆う(M2E-17)。
    /// サンプル添字は `0..sample_count`（終端ちょうどは範囲外）。
    pub duration: RationalTime,
    pub sample_rate: Fps,
}

#[derive(Debug, Clone, Copy)]
pub struct LayerSourceContext {
    /// v1ではコンポ全体で共有される単一カメラ。
    pub camera: CompCamera,
}

/// 複製インスタンスの評価コンテキスト口(F-7予約。配線はM2以降)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceIndex {
    pub index: u32,
    pub count: u32,
}

/// 合体結果の別時刻参照(F-11予約。実装はM4後)。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompLookbehind {
    /// グループIDまたはコンポルートの安定文字列。
    pub target: String,
    /// 負のフレームオフセット列(例: [-1, -2])。
    pub offsets: Vec<i32>,
    /// 自己参照切断用のエフェクトID列。
    pub exclude: Vec<String>,
}

/// 前後フレーム/サブフレーム要求の静的宣言(F-12予約。解決はホスト)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TemporalFootprint {
    pub frames_before: u32,
    pub frames_after: u32,
    /// モーションブラー用。上限は`Quality::effect_samples`。
    pub subframe_samples: u32,
}

/// Filter/Composite の per-call 文脈(M2E-7)。
///
/// `#[non_exhaustive]` — Quality・予約口の追加で既存プラグインのシグネチャを壊さない。
/// 外部クレートは`RenderCtx::new`経由で構築する。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RenderCtx {
    pub t: RationalTime,
    /// Draft/Final 判別と effect_samples の口。解像度畳み込み後の TextureRef.desc だけでは読めない。
    pub quality: Quality,
    /// F-7 予約。Repeater 配線まで常に None。
    pub instance: Option<InstanceIndex>,
    /// F-11 予約。M4 配線まで常に None。
    pub lookbehind: Option<CompLookbehind>,
    /// F-12 予約。窓テクスチャの解決はホスト側(現状はデフォルト=ゼロ窓)。
    pub temporal_footprint: TemporalFootprint,
}

impl RenderCtx {
    pub fn new(t: RationalTime, quality: Quality) -> Self {
        Self {
            t,
            quality,
            instance: None,
            lookbehind: None,
            temporal_footprint: TemporalFootprint::default(),
        }
    }
}
