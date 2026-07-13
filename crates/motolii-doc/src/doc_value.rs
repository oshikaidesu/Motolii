//! ドキュメント永続用の値型(D1h / S3)。
//!
//! 評価層の`motolii_eval::Value`とは分離する。特に`AssetRef`は doc 所有の
//! `AssetId`を載せ、cross-document 再写像を型に乗せる。D3で評価層へ落とす。

use serde::{Deserialize, Serialize};

use motolii_eval::Value as EvalValue;

use crate::asset::AssetId;

/// ドキュメントに保存するパラメータ値。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocValue {
    F64(f64),
    Vec2([f64; 2]),
    Vec3([f64; 3]),
    /// RGBA: 非線形sRGB・straight-alpha・各成分0.0–1.0(M2E-13)。
    Color([f64; 4]),
    /// 永続層のアセット参照。評価層へは D3 で解決済み値へ変換する。
    AssetRef(AssetId),
}

impl DocValue {
    /// D3 用: 評価層 `Value` へ落とす。AssetRef は生の AssetId を渡す。
    pub fn to_eval(&self) -> EvalValue {
        match self {
            Self::F64(v) => EvalValue::F64(*v),
            Self::Vec2(v) => EvalValue::Vec2(*v),
            Self::Vec3(v) => EvalValue::Vec3(*v),
            Self::Color(v) => EvalValue::Color(*v),
            Self::AssetRef(id) => EvalValue::AssetRef(id.get()),
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::F64(_) => "F64",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Color(_) => "Color",
            Self::AssetRef(_) => "AssetRef",
        }
    }
}
