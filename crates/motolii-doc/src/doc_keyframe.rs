//! ドキュメント永続用キーフレーム列(D1h)。
//!
//! 値は`DocValue`。評価層`KeyframeTrack`とは分離し、AssetId を型に載せる。

use serde::{Deserialize, Serialize};

use motolii_core::RationalTime;
use motolii_eval::Interp;

use crate::doc_value::DocValue;

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
