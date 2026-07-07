//! oc-eval: パラメータ評価エンジン。
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

use oc_core::RationalTime;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamSource {
    Const(Value),
    Keyframes(KeyframeTrack),
    /// 解析結果参照。トラックが存在しない場合はfallbackを返す
    Data {
        track: DataTrackId,
        fallback: Value,
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
        }
    }
}
