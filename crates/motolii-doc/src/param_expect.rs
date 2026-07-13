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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamConstraints {
    pub expected: ExpectedValueType,
    /// LookAt / Follow を許すのは position のみ。
    pub allow_spatial_links: bool,
    /// スカラー成分を [0,1] に閉じる(opacity / Color 各成分)。
    pub unit_interval: bool,
    /// F64の下限(含む)。PathOp意味論表の`≥0`等の拒否項目用(D1i-2)。
    pub min: Option<f64>,
    /// F64の上限(含む)。PathOp意味論表の`∈[-1,1]`等の拒否項目用(D1i-2)。
    pub max: Option<f64>,
    /// F64が整数(端数なし)であること。Repeater.copies等(Lottie整数スロット)。
    pub integer: bool,
}

impl ParamConstraints {
    pub const fn typed(expected: ExpectedValueType) -> Self {
        Self {
            expected,
            allow_spatial_links: false,
            unit_interval: false,
            min: None,
            max: None,
            integer: false,
        }
    }

    pub const fn unit_f64() -> Self {
        Self {
            expected: ExpectedValueType::F64,
            allow_spatial_links: false,
            unit_interval: true,
            min: None,
            max: None,
            integer: false,
        }
    }

    pub const fn color() -> Self {
        Self {
            expected: ExpectedValueType::Color,
            allow_spatial_links: false,
            unit_interval: true,
            min: None,
            max: None,
            integer: false,
        }
    }

    pub const fn position() -> Self {
        Self {
            expected: ExpectedValueType::Vec2,
            allow_spatial_links: true,
            unit_interval: false,
            min: None,
            max: None,
            integer: false,
        }
    }

    pub const fn scalar_f64() -> Self {
        Self::typed(ExpectedValueType::F64)
    }

    /// F64を`[min, max]`(両端含む)に閉じる(例: pucker_bloat.amount∈[-1,1])。
    pub const fn ranged_f64(min: f64, max: f64) -> Self {
        Self {
            expected: ExpectedValueType::F64,
            allow_spatial_links: false,
            unit_interval: false,
            min: Some(min),
            max: Some(max),
            integer: false,
        }
    }

    /// F64を`[min, +inf)`に閉じる(例: zig_zag.amount≥0)。
    pub const fn min_f64(min: f64) -> Self {
        Self {
            expected: ExpectedValueType::F64,
            allow_spatial_links: false,
            unit_interval: false,
            min: Some(min),
            max: None,
            integer: false,
        }
    }

    /// F64を`[min, +inf)`かつ整数に閉じる(例: repeater.copies — Lottie整数スロット)。
    pub const fn non_negative_integer_f64() -> Self {
        Self {
            expected: ExpectedValueType::F64,
            allow_spatial_links: false,
            unit_interval: false,
            min: Some(0.0),
            max: None,
            integer: true,
        }
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

/// PathOp の無制限スカラー(角度・オフセット・距離等。表が範囲を固定しない席)。
pub fn path_op_scalar() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::F64)
}

/// PathOp の無制限Vec2(twist.center等。LookAt/Followは許可しない — 表が未決)。
pub fn path_op_vec2() -> ParamConstraints {
    ParamConstraints::typed(ExpectedValueType::Vec2)
}

/// pucker_bloat.amount ∈ [-1, 1](PathOp意味論表)。
pub fn path_op_pucker_bloat_amount() -> ParamConstraints {
    ParamConstraints::ranged_f64(-1.0, 1.0)
}

/// zig_zag.amount / ridges, round_corners.radius ≥ 0(PathOp意味論表)。
pub fn path_op_non_negative() -> ParamConstraints {
    ParamConstraints::min_f64(0.0)
}

/// repeater.copies: 非負整数(Lottie/AE Repeater。fractional offsetとは別スロット)。
pub fn path_op_non_negative_integer() -> ParamConstraints {
    ParamConstraints::non_negative_integer_f64()
}

/// trim.start / trim.end ∈ [0, 1](PathOp意味論表)。
pub fn path_op_unit_interval() -> ParamConstraints {
    ParamConstraints::unit_f64()
}

/// repeater.start_opacity / end_opacity ∈ [0, 1](envelope.opacityと同型)。
pub fn path_op_opacity() -> ParamConstraints {
    ParamConstraints::unit_f64()
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
