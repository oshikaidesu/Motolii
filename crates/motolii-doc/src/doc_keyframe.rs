//! ドキュメント永続用キーフレーム列(D1h)。
//!
//! 値は`DocValue`。評価層`KeyframeTrack`とは分離し、AssetId を型に載せる。

use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;
use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value as EvalValue};

use crate::doc_value::DocValue;
use crate::stable_id::KeyframeId;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum DocKeyframeError {
    #[error("Bezier control point x1/x2 must be in [0,1], got x1={x1} x2={x2}")]
    InvalidBezier { x1: f64, x2: f64 },
    #[error("Bezier control points must be finite")]
    NonFiniteBezier,
    #[error("keyframes must be sorted by strictly increasing time without duplicates")]
    UnsortedOrDuplicateKeys,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocKeyframe {
    /// document-local安定ID(A8)。時刻編集で不変、複製時は新規採番(D2)。
    /// 旧形式(id無し)は拒否 — 変換はD1e(D1g/D1i-1と同型の方針)。
    pub id: KeyframeId,
    pub t: RationalTime,
    pub value: DocValue,
    pub interp: Interp,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "DocKeyframeTrackDe")]
pub struct DocKeyframeTrack {
    keys: Vec<DocKeyframe>,
}

#[derive(Deserialize)]
struct DocKeyframeTrackDe {
    keys: Vec<DocKeyframe>,
}

impl TryFrom<DocKeyframeTrackDe> for DocKeyframeTrack {
    type Error = DocKeyframeError;

    fn try_from(value: DocKeyframeTrackDe) -> Result<Self, Self::Error> {
        let track = Self { keys: value.keys };
        track.validate()?;
        Ok(track)
    }
}

impl DocKeyframeTrack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: DocKeyframe) {
        match self.keys.binary_search_by(|k| k.t.cmp(&key.t)) {
            Ok(i) => self.keys[i] = key,
            Err(i) => self.keys.insert(i, key),
        }
    }

    pub fn keys(&self) -> &[DocKeyframe] {
        &self.keys
    }

    pub fn get_by_id(&self, id: KeyframeId) -> Option<&DocKeyframe> {
        self.keys.iter().find(|k| k.id == id)
    }

    /// idで1件削除する(コマンド層のキーフレーム削除で使用。時刻は不変条件維持に無関係)。
    pub fn remove_by_id(&mut self, id: KeyframeId) -> Option<DocKeyframe> {
        let idx = self.keys.iter().position(|k| k.id == id)?;
        Some(self.keys.remove(idx))
    }

    pub fn validate(&self) -> Result<(), DocKeyframeError> {
        for window in self.keys.windows(2) {
            if window[0].t >= window[1].t {
                return Err(DocKeyframeError::UnsortedOrDuplicateKeys);
            }
        }
        for key in &self.keys {
            validate_interp(&key.interp)?;
        }
        Ok(())
    }

    /// D3: 評価層へ落として補間する(恒久面は DocValue のまま)。
    pub fn eval(&self, t: RationalTime) -> EvalValue {
        let mut track = KeyframeTrack::new();
        for key in &self.keys {
            track.insert(Keyframe {
                t: key.t,
                value: key.value.to_eval(),
                interp: key.interp,
            });
        }
        track.eval(t)
    }
}

pub fn validate_interp(interp: &Interp) -> Result<(), DocKeyframeError> {
    match *interp {
        Interp::Hold | Interp::Linear => Ok(()),
        Interp::Bezier { x1, y1, x2, y2 } => {
            if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
                return Err(DocKeyframeError::NonFiniteBezier);
            }
            if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
                return Err(DocKeyframeError::InvalidBezier { x1, x2 });
            }
            Ok(())
        }
    }
}
