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
    /// 同種のものの並び。keyframeはlist全体で1キーなので、この値がそのまま保存される。
    List(Vec<DocValue>),
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
            Self::List(items) => EvalValue::List(items.iter().map(Self::to_eval).collect()),
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::F64(_) => "F64",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Color(_) => "Color",
            Self::AssetRef(_) => "AssetRef",
            // 要素型は先頭要素から名乗る。空listは要素型を名乗れない。
            Self::List(items) => match items.first() {
                Some(Self::F64(_)) => "List<F64>",
                Some(Self::Vec2(_)) => "List<Vec2>",
                Some(Self::Vec3(_)) => "List<Vec3>",
                Some(Self::Color(_)) => "List<Color>",
                Some(Self::AssetRef(_)) => "List<AssetRef>",
                Some(Self::List(_)) => "List<List>",
                None => "List",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_survives_serde_roundtrip() {
        let value = DocValue::List(vec![
            DocValue::Color([0.0, 0.5, 1.0, 1.0]),
            DocValue::Color([1.0, 0.0, 0.0, 1.0]),
        ]);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<DocValue>(&json).unwrap(), value);
    }

    #[test]
    fn empty_list_survives_serde_roundtrip() {
        let value = DocValue::List(Vec::new());
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<DocValue>(&json).unwrap(), value);
    }

    /// 既存バリアントの表現は変わっていない(旧文書のバイト列を動かさない)。
    #[test]
    fn existing_variants_keep_their_representation() {
        assert_eq!(
            serde_json::to_string(&DocValue::F64(1.5)).unwrap(),
            r#"{"F64":1.5}"#
        );
        assert_eq!(
            serde_json::to_string(&DocValue::Vec2([0.0, 1.0])).unwrap(),
            r#"{"Vec2":[0.0,1.0]}"#
        );
    }

    #[test]
    fn list_lowers_to_eval_elementwise() {
        let value = DocValue::List(vec![DocValue::F64(1.0), DocValue::F64(2.0)]);
        assert_eq!(
            value.to_eval(),
            EvalValue::List(vec![EvalValue::F64(1.0), EvalValue::F64(2.0)])
        );
    }
}
