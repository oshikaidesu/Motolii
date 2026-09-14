//! 層の属性の英語名 — 窓の欄と、スクリプトが書ける名前は同じこの表から([スクリプトの口](../../../../../docs/reviews/2026-09-14-script-mouth.md))。
//! 名前は窓が既に出している語が先、無ければ AE の語(Mask Path・Tracking Amount)、字の軸は OpenType の登録名。
//! 効果の欄の名前は効果の表(`kind.rs` の `Param`・ISF manifest)が持つので、ここには無い。

use std::borrow::Cow;

use crate::doc::store::property as p;

/// 固定の名前を持つ属性: (property, 窓の名前)。Camera と粒子は各自の ROWS が持つ。
pub const FIXED: &[(&str, &str)] = &[
    (p::ANCHOR, "Anchor"),
    (p::POSITION, "Position"),
    (p::POSITION_X, "Position X"),
    (p::POSITION_Y, "Position Y"),
    (p::POSITION_Z, "Position Z"),
    (p::SCALE, "Scale"),
    (p::SCALE_Z, "Scale Z"),
    (p::ROTATION, "Rotation"),
    (p::ROTATION_X, "Tilt X"),
    (p::ROTATION_Y, "Tilt Y"),
    (p::OPACITY, "Opacity"),
    (p::DEPTH, "Depth"),
    (p::SKEW, "Skew"),
    (p::SKEW_AXIS, "Skew Axis"),
    (p::LEVEL, "Level"),
    (p::PAN, "Pan"),
    (p::FADE_IN, "Fade In"),
    (p::FADE_OUT, "Fade Out"),
    (p::TIME_REMAP, "Time Remap"),
    (p::SPEED, "Speed"),
    (TEXT_JUSTIFY, "Alignment"),
    (TEXT_CONTENT, "Content"),
    (p::SHAPE_POINTS, "Points"),
    (p::SHAPE_OUTER_RADIUS, "Outer Radius"),
    (p::SHAPE_INNER_RADIUS, "Inner Radius"),
    (p::SHAPE_SIZE, "Size"),
    (p::SHAPE_STROKE_WIDTH, "Stroke Width"),
    (p::SHAPE_FILL_COLOR, "Fill"),
    (p::SHAPE_LENGTH, "Length"),
    (p::FILL_ANGLE, "Angle"),
    (p::FILL_CENTER, "Center"),
    (p::FILL_SPREAD, "Spread"),
    (p::STAGE_MARGINS[0], "Left"),
    (p::STAGE_MARGINS[1], "Top"),
    (p::STAGE_MARGINS[2], "Right"),
    (p::STAGE_MARGINS[3], "Bottom"),
];

pub const TEXT_JUSTIFY: &str = "text_justify";
/// 文字の中身。property ではなく text document が持つが、名前は同じ表に置く。
pub const TEXT_CONTENT: &str = "content";

/// 番号付きの属性(`mask.<id>.`・`text_style.<id>.`・`text_range.<id>.`・`fill.stop.<id>.`)の末尾 → 名前。
const MASK: &[(&str, &str)] = &[("shape", "Mask Path"), ("opacity", "Mask Opacity"), ("expansion", "Mask Expansion")];
const TEXT_STYLE: &[(&str, &str)] = &[("size", "Size"), ("line_height", "Line height"), ("tracking", "Tracking"), ("fill_color", "Fill")];
const FILL_STOP: &[(&str, &str)] = &[("offset", "Stop Location"), ("color", "Stop Color")];
const TEXT_RANGE: &[(&str, &str)] = &[
    ("selector.start", "Start"),
    ("selector.end", "End"),
    ("selector.offset", "Offset"),
    ("selector.max_amount", "Amount"),
    ("style.fill_color", "Fill Color"),
    ("style.line_spacing", "Line Spacing"),
    ("style.tracking", "Tracking Amount"),
    ("transform.origin", "Anchor"),
    ("transform.opacity", "Opacity"),
    ("transform.position", "Position"),
    ("transform.rotation", "Rotation"),
    ("transform.scale", "Scale"),
];
/// OpenType の登録済み軸(`variation.<tag>`)。未登録の tag はそのまま名前にする(字の作者が付けた名前)。
const VARIATION_AXES: &[(&str, &str)] = &[("wght", "Weight"), ("wdth", "Width"), ("slnt", "Slant"), ("ital", "Italic"), ("opsz", "Optical Size")];

/// 属性の窓の名前。効果の欄と、表に無い属性は `None`。
pub fn label(property: &str) -> Option<Cow<'static, str>> {
    let find = |table: &[(&'static str, &'static str)], key: &str| table.iter().find(|row| row.0 == key).map(|row| Cow::Borrowed(row.1));
    if let Some(name) = find(FIXED, property) {
        return Some(name);
    }
    if let Some(row) = p::CAMERA_ROWS.iter().find(|row| row.0 == property) {
        return Some(Cow::Borrowed(row.1));
    }
    if let Some(row) = crate::doc::store::particles::ROWS.iter().find(|row| row.0 == property) {
        return Some(Cow::Borrowed(row.1));
    }
    if let Some(row) = crate::doc::store::layout::row(property) {
        return Some(Cow::Borrowed(row.1));
    }
    if let Some(name) = crate::doc::store::layout::track_label(property) {
        return Some(Cow::Owned(name));
    }
    let numbered = |prefix: &str| property.strip_prefix(prefix).and_then(|rest| rest.split_once('.')).filter(|(id, _)| id.parse::<u64>().is_ok()).map(|(_, attr)| attr);
    if let Some(attr) = numbered(p::MASK_PREFIX) {
        return find(MASK, attr);
    }
    if let Some(attr) = numbered(p::TEXT_STYLE_PREFIX) {
        return find(TEXT_STYLE, attr);
    }
    if let Some(attr) = numbered(p::FILL_STOP_PREFIX) {
        return find(FILL_STOP, attr);
    }
    if let Some(attr) = numbered(p::TEXT_RANGE_PREFIX) {
        if let Some(tag) = attr.strip_prefix("variation.") {
            return Some(find(VARIATION_AXES, tag).unwrap_or_else(|| Cow::Owned(tag.to_owned())));
        }
        return find(TEXT_RANGE, attr);
    }
    None
}

/// 番号の無い属性の名前を全部(窓へ 1 回渡し、Inspector の見出しもここから引く)。
pub fn fixed() -> impl Iterator<Item = (&'static str, &'static str)> {
    FIXED.iter().copied().chain(p::CAMERA_ROWS.iter().map(|row| (row.0, row.1))).chain(crate::doc::store::particles::ROWS.iter().map(|row| (row.0, row.1)))
        .chain(crate::doc::store::layout::GROUP_ROWS.iter().chain(crate::doc::store::layout::ITEM_ROWS).chain(crate::doc::store::layout::SPACE_ROWS).chain(crate::doc::store::layout::CONNECT_ROWS).map(|row| (row.0, row.1)))
}

/// 表に載っているはずの名前。無ければ内部の id を窓に出してしまうので、呼ぶ側は test で塞ぐ。
pub fn label_or_id(property: &str) -> Cow<'_, str> {
    label(property).unwrap_or(Cow::Borrowed(property))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{MaskId, PropertyId, TextRangeId, TextStyleId};

    #[test]
    fn every_layer_property_has_an_english_name() {
        // property の表に定数を足したら、名前を付けるまでここが赤。接頭辞(`mask.` 等)は番号付きで見る。
        let source = include_str!("../store.rs");
        let module = &source[source.find("pub mod property").expect("property の表")..];
        let consts = module.lines().filter_map(|line| line.trim().strip_prefix("pub const ")?.split_once(": &str = \"")).map(|(_, rest)| rest.trim_end_matches("\";"))
            .filter(|id| !id.ends_with('.')).collect::<Vec<_>>();
        assert!(consts.len() > 30, "property の表を読めていない: {consts:?}");
        let particles = crate::doc::store::particles::ROWS.iter().map(|row| row.0);
        let mask = MaskId(3);
        let style = TextStyleId(2);
        let range = TextRangeId(5);
        let numbered = [
            PropertyId::mask_shape(mask), PropertyId::mask_opacity(mask), PropertyId::mask_expansion(mask),
            PropertyId::text_style_size(style), PropertyId::text_style_line_height(style), PropertyId::text_style_tracking(style), PropertyId::text_style_fill_color(style),
            PropertyId::text_range_selector_start(range), PropertyId::text_range_selector_end(range), PropertyId::text_range_selector_offset(range),
            PropertyId::text_range_selector_max_amount(range), PropertyId::text_range_fill_color(range), PropertyId::text_range_line_spacing(range),
            PropertyId::text_range_tracking(range), PropertyId::text_range_origin(range), PropertyId::text_range_opacity(range),
            PropertyId::text_range_position(range), PropertyId::text_range_rotation(range), PropertyId::text_range_scale(range),
            PropertyId::text_range_variation_axis(range, "wght"),
        ].map(|id| id.name().to_owned());
        let stops = ["offset", "color"].map(|attr| format!("{}7.{attr}", p::FILL_STOP_PREFIX));
        let missing: Vec<String> = consts.iter().map(|s| s.to_string()).chain(particles.map(str::to_owned)).chain(p::STAGE_MARGINS.map(str::to_owned))
            .chain(numbered).chain(stops).filter(|id| label(id).is_none()).collect();
        assert!(missing.is_empty(), "英語の名前が無い属性: {missing:?}");
    }

    /// p5 や AE の名前を書いても、表に無ければ名前にならない。
    #[test]
    fn names_outside_the_table_are_not_names() {
        for invented in ["fill", "stroke", "background", "frameCount", "effect.1.param.radius", "mask.x.shape", "text_style.0.weight"] {
            assert_eq!(label(invented), None, "{invented}");
        }
        assert_eq!(label("text_range.1.variation.GRAD").as_deref(), Some("GRAD"));
    }
}
