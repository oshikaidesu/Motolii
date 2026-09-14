//! Track Overlay — 下の合成から塊を拾い、その上に FUI(箱・印)を描く出力の効果(2026-09-14 利用者「Tracery 2 の仕様を移す」)。
//! 先例は Dragoy の Tracery 2(調整層に掛ける、Keying / Box / Marker / Grid / Connection Lines / Labels)。欄の名前と並びは
//! 製品ページの画面から写した(docs/reviews/2026-09-14-tracery2-port-spec.md)。名前とコードは写さない。

use crate::doc::eval::Value;
use crate::doc::store::kind::{Param, ParamKind};

pub struct OverlayKind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
}

pub const TRACK_OVERLAY: &str = "motolii.track_overlay";
pub const METHODS: &[&str] = &["Motion Detection", "Key Color"];
pub const BOX_SHAPES: &[&str] = &["Rectangle", "Square", "Ellipse", "Circle"];
pub const MARKER_TYPES: &[&str] = &["Dot", "Plus", "Cross", "Polygon"];
const SWITCH: &[&str] = &["Off", "On"];

const fn number(name: &'static str, label: &'static str, section: &'static str, default: f64, range: (f64, f64)) -> Param {
    Param { name, label, section, kind: ParamKind::Number, default: [default, 0.0], range: Some(range), modes: None }
}
const fn choice(name: &'static str, label: &'static str, section: &'static str, choices: &'static [&'static str], default: f64) -> Param {
    Param { name, label, section, kind: ParamKind::Choice(choices), default: [default, 0.0], range: Some((0.0, (choices.len() - 1) as f64)), modes: None }
}
const fn switch(name: &'static str, label: &'static str, section: &'static str, on: bool) -> Param {
    choice(name, label, section, SWITCH, if on { 1.0 } else { 0.0 })
}
const fn color(name: &'static str, label: &'static str, section: &'static str, rgba: [f64; 4]) -> Param {
    Param { name, label, section, kind: ParamKind::Color(rgba), default: [0.0, 0.0], range: None, modes: None }
}

pub const KINDS: &[OverlayKind] = &[OverlayKind {
    plugin_id: TRACK_OVERLAY,
    label: "Track Overlay",
    params: &[
        choice("method", "Detection Method", "Keying", METHODS, 0.0),
        number("threshold", "Threshold", "Keying", 10.0, (0.0, 100.0)),
        number("blur", "Blur Strength", "Keying", 0.0, (0.0, 100.0)),
        number("min_region", "Min Region Pixels", "Keying", 200.0, (0.0, 1.0e9)),
        color("key_color", "Key Color", "Keying", [0.0, 1.0, 0.0, 1.0]),
        switch("show_mask", "Show Mask", "Keying", false),
        // Motolii の足し(Tracery 2 の画面に無い): 解析の細かさ、塊を切り離す削り、大きすぎる塊、ID の持続。
        number("detail", "Detail", "Keying", 960.0, (120.0, 3840.0)),
        number("separation", "Separation", "Keying", 2.0, (0.0, 200.0)),
        number("max_region", "Max Region Pixels", "Keying", 1.0e9, (0.0, 1.0e9)),
        switch("keep_ids", "Keep IDs", "Keying", true),
        switch("box", "Box Enabled", "Box", true),
        choice("box_shape", "Box Shape", "Box", BOX_SHAPES, 0.0),
        switch("custom_size", "Custom Size", "Box", false),
        number("custom_radius", "Custom Radius", "Box", 32.0, (0.0, 10000.0)),
        switch("box_stroke", "Box Stroke Enabled", "Box", true),
        color("box_stroke_color", "Color Box Stroke", "Box", [0.62, 1.0, 0.24, 1.0]),
        number("box_stroke_opacity", "Box Stroke Opacity", "Box", 1.0, (0.0, 1.0)),
        number("box_stroke_width", "Box Stroke", "Box", 2.0, (0.0, 200.0)),
        switch("box_gap", "Box Gap Enabled", "Box", false),
        number("box_gap_size", "Box Gap Size", "Box", 0.3, (0.0, 1.0)),
        switch("box_fill", "Box Fill Enabled", "Box", false),
        number("box_fill_opacity", "Box Fill Opacity", "Box", 0.25, (0.0, 1.0)),
        color("box_fill_color", "Box Fill Color", "Box", [0.62, 1.0, 0.24, 1.0]),
        switch("marker", "Marker Enabled", "Marker", false),
        choice("marker_type", "Marker Type", "Marker", MARKER_TYPES, 2.0),
        color("marker_color", "Marker Color", "Marker", [0.62, 1.0, 0.24, 1.0]),
        number("marker_opacity", "Marker Opacity", "Marker", 1.0, (0.0, 1.0)),
        number("marker_size", "Marker Size", "Marker", 16.0, (0.0, 10000.0)),
        number("marker_thickness", "Marker Line Thickness", "Marker", 3.0, (0.0, 200.0)),
        number("marker_rotation", "Marker Rotation", "Marker", 0.0, (-36000.0, 36000.0)),
        number("polygon_sides", "Polygon Sides", "Marker", 3.0, (3.0, 12.0)),
        switch("polygon_fill", "Polygon Fill", "Marker", false),
    ],
}];

pub fn is_track_overlay(plugin_id: &str) -> bool {
    plugin_id == TRACK_OVERLAY
}

/// 数・選択の欄の値。無い欄は宣言の既定。
pub fn number_of(params: &[(String, Value)], name: &str) -> f64 {
    let default = KINDS[0].params.iter().find(|p| p.name == name).map_or(0.0, |p| p.default[0]);
    match params.iter().find(|(n, _)| n == name).map(|(_, v)| v) {
        Some(Value::F64(v)) if v.is_finite() => *v,
        _ => default,
    }
}

pub fn switch_of(params: &[(String, Value)], name: &str) -> bool {
    number_of(params, name) >= 0.5
}

/// 色の欄の値(非乗算 RGBA)。無い欄は宣言の既定。
pub fn color_of(params: &[(String, Value)], name: &str) -> [f64; 4] {
    let default = KINDS[0].params.iter().find(|p| p.name == name).and_then(|p| match p.kind { ParamKind::Color(c) => Some(c), _ => None }).unwrap_or([1.0; 4]);
    match params.iter().find(|(n, _)| n == name).map(|(_, v)| v) {
        Some(Value::Color(c)) => *c,
        _ => default,
    }
}
