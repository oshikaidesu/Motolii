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
pub const METHODS: &[&str] = &["Motion Detection", "Key Color", "Layers"];
/// Grid の View Mode(Tracery 2 の Grid 節): Edge = 拾った物の箱の辺から格子を立てる、Cartesian = 等間隔の格子。
pub const GRID_MODES: &[&str] = &["Edge", "Cartesian"];
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
        switch("grid", "Grid Enabled", "Grid", false),
        choice("grid_mode", "View Mode", "Grid", GRID_MODES, 0.0),
        number("grid_columns", "Grid Columns", "Grid", 12.0, (1.0, 400.0)),
        number("grid_rows", "Grid Rows", "Grid", 8.0, (1.0, 400.0)),
        color("grid_color", "Grid Color", "Grid", [0.62, 1.0, 0.24, 1.0]),
        number("grid_opacity", "Grid Opacity", "Grid", 0.85, (0.0, 1.0)),
        number("grid_thickness", "Line Thickness", "Grid", 1.0, (0.0, 200.0)),
        // Motolii の足し: 近い辺を 1 本にまとめる幅(辺の分布の山の広がり、px)と、物をその線へ引き寄せる強さ(Layers の時)。
        number("grid_merge", "Merge Distance", "Grid", 12.0, (0.0, 2000.0)),
        number("grid_snap", "Snap Strength", "Grid", 0.0, (0.0, 1.0)),
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

/// 箱の辺の位置(1 つの軸)から格子の線を立てる: 辺ごとに 1 本、近くの辺の重み付き平均へ寄せた位置(mean shift を 3 回)。
/// 重みは幅 `merge` のガウス。近い辺同士の線は同じ所に重なって 1 本に見え、離れれば分かれる。分布の山を拾う方法と違い
/// 枝分かれの点が無いので、辺が動いても線は滑らかに動く。濃さは 1 / (近くの辺の数) で、重なった線が 1 本分の濃さに近づく。
/// 戻り値は辺と同じ順の (線の位置, 濃さ)。`merge` が 0 なら辺そのもの。
pub fn edge_lines(edges: &[f32], merge: f32) -> Vec<(f32, f32)> {
    if merge <= 1e-3 {
        return edges.iter().map(|e| (*e, 1.0)).collect();
    }
    let kernel = |d: f32| (-(d * d) / (2.0 * merge * merge)).exp();
    let mut positions: Vec<f32> = edges.to_vec();
    for _ in 0..3 {
        positions = positions.iter().map(|x| {
            let (mut sum, mut total) = (0.0f32, 0.0f32);
            for e in edges {
                let w = kernel(x - e);
                sum += e * w;
                total += w;
            }
            if total > 1e-9 { sum / total } else { *x }
        }).collect();
    }
    positions.iter().map(|x| (*x, 1.0 / edges.iter().map(|e| kernel(x - e)).sum::<f32>().max(1.0))).collect()
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn near_edges_meet_on_one_line_and_lines_move_smoothly_as_edges_part() {
        let lines = edge_lines(&[100.0, 104.0, 300.0], 12.0);
        assert!((lines[0].0 - lines[1].0).abs() < 0.5 && (lines[0].0 - 102.0).abs() < 1.0, "edges 4 px apart share a line between them: {lines:?}");
        assert!((lines[2].0 - 300.0).abs() < 0.1, "a far edge keeps its own line");
        let mut previous: Option<Vec<(f32, f32)>> = None;
        for k in 0..200 {
            let d = k as f32 * 0.25;
            let now = edge_lines(&[200.0 - d * 0.5, 200.0 + d * 0.5], 12.0);
            if let Some(prev) = &previous {
                for (a, b) in now.iter().zip(prev) {
                    assert!((a.0 - b.0).abs() < 1.0, "each edge's line moves a little per step: {a:?} vs {b:?} at {d}");
                }
            }
            previous = Some(now);
        }
    }
}
