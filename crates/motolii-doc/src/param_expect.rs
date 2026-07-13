//! DocParam受け口の期待型表(D1h / 第二監査S3・S4・S9)。
//!
//! スキーマ側の正本。validateがこれに照らして Const / Keyframes / Data.fallback /
//! Vec2Axes を検査する。DataTrack本体の実出力型照合はD3。

use crate::doc_value::DocValue;

/// パラメータ値の期待バリアント。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedValueType {
    F64,
    Vec2,
    Vec3,
    Color,
    AssetRef,
}

impl ExpectedValueType {
    pub fn name(self) -> &'static str {
        match self {
            Self::F64 => "F64",
            Self::Vec2 => "Vec2",
            Self::Vec3 => "Vec3",
            Self::Color => "Color",
            Self::AssetRef => "AssetRef",
        }
    }

    pub fn matches(self, value: &DocValue) -> bool {
        matches!(
            (self, value),
            (Self::F64, DocValue::F64(_))
                | (Self::Vec2, DocValue::Vec2(_))
                | (Self::Vec3, DocValue::Vec3(_))
                | (Self::Color, DocValue::Color(_))
                | (Self::AssetRef, DocValue::AssetRef(_))
        )
    }
}

/// 受け口ごとの制約。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamConstraints {
    pub expected: ExpectedValueType,
    /// LookAt / Follow を許すのは position のみ。
    pub allow_spatial_links: bool,
    /// スカラー成分を [0,1] に閉じる(opacity / Color 各成分)。
    pub unit_interval: bool,
}

impl ParamConstraints {
    pub const fn typed(expected: ExpectedValueType) -> Self {
        Self {
            expected,
            allow_spatial_links: false,
            unit_interval: false,
        }
    }

    pub const fn unit_f64() -> Self {
        Self {
            expected: ExpectedValueType::F64,
            allow_spatial_links: false,
            unit_interval: true,
        }
    }

    pub const fn color() -> Self {
        Self {
            expected: ExpectedValueType::Color,
            allow_spatial_links: false,
            unit_interval: true,
        }
    }

    pub const fn position() -> Self {
        Self {
            expected: ExpectedValueType::Vec2,
            allow_spatial_links: true,
            unit_interval: false,
        }
    }

    pub const fn scalar_f64() -> Self {
        Self::typed(ExpectedValueType::F64)
    }
}

/// Transform / envelope の固定スロット。
pub fn transform_position() -> ParamConstraints {
    ParamConstraints::position()
}

pub fn transform_anchor() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::Vec2)
}

pub fn transform_scale() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::Vec2)
}

pub fn transform_rotation() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::F64)
}

pub fn envelope_opacity() -> ParamConstraints {
    ParamConstraints::unit_f64()
}

/// PathOp の全 DocParam スロットは v1 で F64(値域の詳細は D1i-2)。
pub fn path_op_scalar() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::F64)
}

/// 既知ファーストパーティ effect / plugin / layer_source / composite / param_driver の期待型。
/// `register_reference_plugins` の NodeDesc 全件と一致させること(乖離テストあり)。
/// 未知 ID は呼び出し側が構造検査(有限性・AssetRef)のみ行う。
pub fn known_plugin_param(plugin_id: &str, param_id: &str) -> Option<ParamConstraints> {
    match (plugin_id, param_id) {
        // Filters
        ("core.filter.tint", "color")
        | ("core.filter.clear", "color")
        | ("core.layer_source.clear", "color")
        | ("core.composite.clear", "color") => Some(ParamConstraints::color()),
        ("core.filter.opacity", "amount") => Some(ParamConstraints::unit_f64()),
        // ParamDriver (sine v2)
        ("core.param.sine", "amplitude")
        | ("core.param.sine", "frequency_hz")
        | ("core.param.sine", "offset") => Some(ParamConstraints::scalar_f64()),
        _ => None,
    }
}

/// 既知表に載っている plugin_id の一覧(余剰エントリ検出用)。
pub fn known_plugin_ids() -> &'static [&'static str] {
    &[
        "core.filter.clear",
        "core.filter.tint",
        "core.filter.opacity",
        "core.layer_source.clear",
        "core.composite.clear",
        "core.param.sine",
    ]
}

/// Vec2Axes の各軸は常にスカラー。
pub fn vec2_axis() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::F64)
}
