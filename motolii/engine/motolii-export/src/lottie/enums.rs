use motolii_store::{MaskMode, MatteMode, TextJustify};
use motolii_vector::{
    Composite, FillRule, GradientType, LineCap, LineJoin, PointType, StarType, TrimMultiple,
};

// ---------------------------------------------------------------------------
// 定数の数値化(`next/reference/lottie.schema.json` の `$defs/constants/*` を
// そのまま転記。裁定58/59/65/66/67 等が「発明の余地が無いのでそのまま採る」と
// 言っている語彙なので、ここも値を発明しない)
// ---------------------------------------------------------------------------

pub(crate) fn blend_mode_to_int(mode: motolii_store::BlendMode) -> i64 {
    use motolii_store::BlendMode::*;
    match mode {
        Normal => 0,
        Multiply => 1,
        Screen => 2,
        Overlay => 3,
        Darken => 4,
        Lighten => 5,
        ColorDodge => 6,
        ColorBurn => 7,
        HardLight => 8,
        SoftLight => 9,
        Difference => 10,
        Exclusion => 11,
        Hue => 12,
        Saturation => 13,
        Color => 14,
        Luminosity => 15,
        Add => 16,
    }
}

pub(crate) fn matte_mode_to_int(mode: MatteMode) -> i64 {
    match mode {
        MatteMode::Alpha => 1,
        MatteMode::InvertedAlpha => 2,
        MatteMode::Luma => 3,
        MatteMode::InvertedLuma => 4,
    }
}

pub(crate) fn mask_mode_to_str(mode: MaskMode) -> &'static str {
    match mode {
        MaskMode::Add => "a",
        MaskMode::Subtract => "s",
        MaskMode::Intersect => "i",
        MaskMode::Lighten => "l",
        MaskMode::Darken => "d",
        MaskMode::Difference => "f",
    }
}

pub(crate) fn fill_rule_to_int(rule: FillRule) -> i64 {
    match rule {
        FillRule::NonZero => 1,
        FillRule::EvenOdd => 2,
    }
}

pub(crate) fn line_cap_to_int(cap: LineCap) -> i64 {
    match cap {
        LineCap::Butt => 1,
        LineCap::Round => 2,
        LineCap::Square => 3,
    }
}

pub(crate) fn line_join_to_int(join: LineJoin) -> i64 {
    match join {
        LineJoin::Miter => 1,
        LineJoin::Round => 2,
        LineJoin::Bevel => 3,
    }
}

pub(crate) fn star_type_to_int(t: StarType) -> i64 {
    match t {
        StarType::Star => 1,
        StarType::Polygon => 2,
    }
}

pub(crate) fn point_type_to_int(t: PointType) -> i64 {
    match t {
        PointType::Corner => 1,
        PointType::Smooth => 2,
    }
}

pub(crate) fn composite_to_int(c: Composite) -> i64 {
    match c {
        Composite::Above => 2,
        Composite::Below => 1,
    }
}

pub(crate) fn trim_multiple_to_int(t: TrimMultiple) -> i64 {
    match t {
        TrimMultiple::Simultaneously => 1,
        TrimMultiple::Individually => 2,
    }
}

pub(crate) fn gradient_type_to_int(t: GradientType) -> i64 {
    match t {
        GradientType::Linear => 1,
        GradientType::Radial => 2,
    }
}

pub(crate) fn text_justify_to_int(j: TextJustify) -> i64 {
    match j {
        TextJustify::Left => 0,
        TextJustify::Right => 1,
        TextJustify::Center => 2,
    }
}
