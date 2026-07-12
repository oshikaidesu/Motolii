//! ドキュメント側パラメータ(LayerId参照を含む)。
//!
//! evalの`ParamSource`にはLayerIdを出さない(M2E-15)。D3で解決済み値へ落とす。
//! serdeはsnake_case外部タグ。中の`Value`はeval由来でPascalCase — ProjectV1の
//! `ParamSource` JSONとは別名空間(意図的。DocumentはProjectV1を継承しない)。

use serde::{Deserialize, Serialize};

use motolii_eval::{DataTrackId, KeyframeTrack, Value};

use crate::LayerId;

/// LookAtの軸(concept: 型付きリンク)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LookAtAxis {
    PlusY,
    PlusX,
}

/// ドキュメントに保存するパラメータ出どころ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocParam {
    Const(Value),
    Keyframes(KeyframeTrack),
    Data {
        track: DataTrackId,
        fallback: Value,
    },
    Vec2Axes {
        x: Box<DocParam>,
        y: Box<DocParam>,
    },
    LookAt {
        target: LayerId,
        axis: LookAtAxis,
    },
    Follow {
        target: LayerId,
        offset: [f64; 2],
    },
}

impl DocParam {
    pub fn const_f64(v: f64) -> Self {
        Self::Const(Value::F64(v))
    }

    pub fn const_vec2(v: [f64; 2]) -> Self {
        Self::Const(Value::Vec2(v))
    }

    pub fn const_color(v: [f64; 4]) -> Self {
        Self::Const(Value::Color(v))
    }
}
