//! motoly-eval: パラメータ評価エンジン。
//!
//! 「時刻t → 値」の純関数として評価する。キーフレーム補間と、
//! 解析結果(DataTrack)を参照するパラメータを同一機構で扱う(落とし穴C-3対策)。
//! Tracery的な「解析→生成」は、このDataTrack参照がパラメータを駆動することに他ならない。

mod bezier;
mod track;
mod value;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use bezier::cubic_bezier_ease;
pub use track::{DataTrack, Interp, Keyframe, KeyframeTrack};
pub use value::Value;

use motoly_core::RationalTime;

/// DataTrackの識別子(解析ノードの出力名など)。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataTrackId(pub String);

impl From<&str> for DataTrackId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 評価コンテキスト: 解析結果などの時系列データ列の集合。
#[derive(Debug, Default, Clone)]
pub struct DataTracks {
    tracks: HashMap<DataTrackId, DataTrack>,
}

impl DataTracks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: impl Into<DataTrackId>, track: DataTrack) {
        self.tracks.insert(id.into(), track);
    }

    pub fn get(&self, id: &DataTrackId) -> Option<&DataTrack> {
        self.tracks.get(id)
    }
}

/// パラメータの値の出どころ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParamSource {
    Const(Value),
    Keyframes(KeyframeTrack),
    /// 解析結果参照。トラックが存在しない場合はfallbackを返す
    Data {
        track: DataTrackId,
        fallback: Value,
    },
    /// スカラー2本からVec2を組み立てる(DataTrackのF64をVec2パラメータへ接続する最小経路)。
    Vec2Axes {
        x: Box<ParamSource>,
        y: Box<ParamSource>,
    },
}

impl ParamSource {
    /// 時刻tでのパラメータ値。純関数(同じ入力に対して常に同じ値)。
    pub fn eval(&self, t: RationalTime, ctx: &DataTracks) -> Value {
        match self {
            ParamSource::Const(v) => v.clone(),
            ParamSource::Keyframes(track) => track.eval(t),
            ParamSource::Data { track, fallback } => ctx
                .get(track)
                .map(|d| d.eval(t))
                .unwrap_or_else(|| fallback.clone()),
            ParamSource::Vec2Axes { x, y } => {
                Value::Vec2([eval_scalar_axis(x, t, ctx), eval_scalar_axis(y, t, ctx)])
            }
        }
    }
}

fn eval_scalar_axis(source: &ParamSource, t: RationalTime, ctx: &DataTracks) -> f64 {
    match source {
        ParamSource::Data { track, fallback } => {
            let v = ctx
                .get(track)
                .map(|d| d.eval(t))
                .unwrap_or_else(|| fallback.clone());
            v.as_f64().or_else(|| fallback.as_f64()).unwrap_or(0.0)
        }
        other => {
            let v = other.eval(t, ctx);
            v.as_f64().unwrap_or_else(|| axis_fallback_scalar(other))
        }
    }
}

fn axis_fallback_scalar(source: &ParamSource) -> f64 {
    match source {
        ParamSource::Const(v) => v.as_f64().unwrap_or(0.0),
        ParamSource::Data { fallback, .. } => fallback.as_f64().unwrap_or(0.0),
        ParamSource::Keyframes(_) | ParamSource::Vec2Axes { .. } => 0.0,
    }
}
