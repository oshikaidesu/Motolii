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
/// 物理の可視(提案 2026-09-16、利用者「画の引力やハンドルも可視できるモードがいるべきだ」)。
/// 箱は Layers と同じく下の層から取り、そこに触れ合っている線と、場の届く輪・向きを足す。
pub const PHYSICS_TRACE: &str = "motolii.physics_trace";
pub const METHODS: &[&str] = &["Motion Detection", "Key Color", "Layers"];
/// Grid の View Mode(Tracery 2 の Grid 節): Edge = 拾った物の箱の辺から格子を立てる、Cartesian = 等間隔の格子。
pub const GRID_MODES: &[&str] = &["Edge", "Cartesian"];
pub const BOX_SHAPES: &[&str] = &["Rectangle", "Square", "Ellipse", "Circle"];
pub const MARKER_TYPES: &[&str] = &["Dot", "Plus", "Cross", "Polygon"];
/// Labels の Display Mode(Tracery 2 の Coordinates / Dimensions と、Motolii の足しの Push = 押された px)。
pub const LABEL_MODES: &[&str] = &["Coordinates", "Dimensions", "Push", "Speed"];
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

const TRACK_OVERLAY_PARAMS: &[Param] = &[
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
        switch("links", "Contact Lines", "Physics", false),
        switch("contacts", "Contact Points", "Physics", false),
        switch("velocity", "Velocity", "Physics", false),
        color("velocity_color", "Velocity Color", "Physics", [0.55, 0.95, 1.0, 1.0]),
        switch("hull", "Outline", "Physics", false),
        color("links_color", "Contact Line Color", "Physics", [1.0, 0.85, 0.2, 1.0]),
        number("links_width", "Contact Line Width", "Physics", 2.0, (0.0, 100.0)),
        switch("well", "Field Reach", "Physics", false),
        color("well_color", "Field Color", "Physics", [0.4, 0.9, 1.0, 1.0]),
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
        // Motolii の足し(Layers の時): 間合いの法で押された物の、いたかった箱と、今の中心への矢印。
        switch("push", "Push Enabled", "Push", false),
        color("push_color", "Push Color", "Push", [0.88, 0.23, 0.18, 1.0]),
        number("push_opacity", "Push Opacity", "Push", 1.0, (0.0, 1.0)),
        number("push_thickness", "Push Thickness", "Push", 2.0, (0.0, 200.0)),
        switch("push_dash", "Push Dash", "Push", true),
        // Tracery 2 の Labels(Display Mode の一部と、Motolii の足しの Push)。
        switch("label", "Label Enabled", "Labels", false),
        choice("label_mode", "Display Mode", "Labels", LABEL_MODES, 0.0),
        color("label_color", "Label Color", "Labels", [0.62, 1.0, 0.24, 1.0]),
        number("label_opacity", "Label Opacity", "Labels", 0.85, (0.0, 1.0)),
        number("font_size", "Font Size", "Labels", 18.0, (1.0, 1000.0)),
        number("label_offset_x", "Offset X", "Labels", 0.0, (-10000.0, 10000.0)),
        number("label_offset_y", "Offset Y", "Labels", 10.0, (-10000.0, 10000.0)),
    ];

/// Push Trace(2026-09-15 利用者「tracy が人気な理由は、簡単に関係性が可視化されているように見えるから」「ユーザーが触るのはひとつのオブジェクトだけ。旨みの最大値はそれ」):
/// Track Overlay と同じ欄で、既定だけが違う棚の 1 枚。下の層を拾い(Layers)、押された物ごとに跡・矢印・押された px を描く。
pub const PUSH_TRACE: &str = "motolii.push_trace";
/// 物理の可視の既定: 箱は Layers、枠は細く切れた線、触れ合いの線と場の輪を出す。
const PHYSICS_TRACE_DEFAULTS: &[(&str, [f64; 4])] = &[
    ("method", [2.0, 0.0, 0.0, 0.0]),
    // 線は全部細く、印は小さく、色は 1 つ(Tracery と Pinterest の計器の作法。太い線は 1 本も無い)。
    ("box", [0.0, 0.0, 0.0, 0.0]),
    ("hull", [1.0, 0.0, 0.0, 0.0]),
    ("box_stroke_color", [0.93, 0.98, 0.55, 1.0]),
    ("contacts", [1.0, 0.0, 0.0, 0.0]),
    ("links_color", [0.93, 0.98, 0.55, 1.0]),
    ("links_width", [1.0, 0.0, 0.0, 0.0]),
    ("velocity", [1.0, 0.0, 0.0, 0.0]),
    ("velocity_color", [0.93, 0.98, 0.55, 0.55]),
    ("well", [1.0, 0.0, 0.0, 0.0]),
    ("well_color", [0.93, 0.98, 0.55, 0.7]),
    ("marker", [1.0, 0.0, 0.0, 0.0]),
    ("marker_type", [1.0, 0.0, 0.0, 0.0]),
    ("marker_size", [5.0, 0.0, 0.0, 0.0]),
    ("marker_thickness", [1.0, 0.0, 0.0, 0.0]),
    ("marker_color", [0.93, 0.98, 0.55, 1.0]),
    ("marker_opacity", [0.9, 0.0, 0.0, 0.0]),
    ("label", [1.0, 0.0, 0.0, 0.0]),
    ("label_mode", [3.0, 0.0, 0.0, 0.0]),
    ("label_color", [0.93, 0.98, 0.55, 1.0]),
    ("label_opacity", [0.75, 0.0, 0.0, 0.0]),
    ("font_size", [10.0, 0.0, 0.0, 0.0]),
    ("label_offset_x", [0.0, 0.0, 0.0, 0.0]),
    ("label_offset_y", [6.0, 0.0, 0.0, 0.0]),
];

const PUSH_TRACE_DEFAULTS: &[(&str, [f64; 4])] = &[
    ("method", [2.0, 0.0, 0.0, 0.0]),
    ("box", [1.0, 0.0, 0.0, 0.0]),
    ("box_stroke_color", [0.95, 0.93, 0.89, 1.0]),
    ("box_stroke_opacity", [0.6, 0.0, 0.0, 0.0]),
    ("box_stroke_width", [1.0, 0.0, 0.0, 0.0]),
    ("box_gap", [1.0, 0.0, 0.0, 0.0]),
    ("box_gap_size", [0.8, 0.0, 0.0, 0.0]),
    ("push", [1.0, 0.0, 0.0, 0.0]),
    ("label", [1.0, 0.0, 0.0, 0.0]),
    ("label_mode", [2.0, 0.0, 0.0, 0.0]),
    ("label_color", [0.88, 0.23, 0.18, 1.0]),
    ("label_opacity", [1.0, 0.0, 0.0, 0.0]),
];

/// Found Grid(2026-09-15 利用者「かなりおもしろいね、Tracery 的にエフェクトとして足しておこう」): Track Overlay と同じ欄で、
/// 既定だけが違う棚の 1 枚。下の層の箱を拾い(Layers)、辺から格子を立てる(Grid の Edge)。見つけた線へ物を寄せるのは Snap Strength。
pub const FOUND_GRID: &str = "motolii.found_grid";
const FOUND_GRID_DEFAULTS: &[(&str, [f64; 4])] = &[
    ("method", [2.0, 0.0, 0.0, 0.0]),
    ("grid", [1.0, 0.0, 0.0, 0.0]),
    ("grid_color", [0.85, 0.33, 0.23, 1.0]),
    ("grid_opacity", [1.0, 0.0, 0.0, 0.0]),
    ("grid_thickness", [1.2, 0.0, 0.0, 0.0]),
    ("grid_merge", [40.0, 0.0, 0.0, 0.0]),
    ("box_stroke_color", [0.08, 0.08, 0.08, 1.0]),
    ("box_stroke_width", [1.0, 0.0, 0.0, 0.0]),
    ("box_gap", [1.0, 0.0, 0.0, 0.0]),
    ("box_gap_size", [0.7, 0.0, 0.0, 0.0]),
];

/// 欄の表から、既定だけを差し替えた写し(欄の名前・並び・範囲は同じ 1 つの表から)。
fn with_overrides(base: &[Param], overrides: &[(&str, [f64; 4])]) -> &'static [Param] {
    let params: Vec<Param> = base.iter().map(|p| {
        let over = overrides.iter().find(|(n, _)| *n == p.name).map(|(_, v)| *v);
        let kind = match &p.kind {
            ParamKind::Number => ParamKind::Number,
            ParamKind::Vec2 => ParamKind::Vec2,
            ParamKind::Choice(c) => ParamKind::Choice(c),
            ParamKind::Layer => ParamKind::Layer,
            ParamKind::Color(c) => ParamKind::Color(over.unwrap_or(*c)),
        };
        let default = match (&p.kind, over) { (ParamKind::Color(_), _) | (_, None) => p.default, (_, Some(v)) => [v[0], v[1]] };
        Param { name: p.name, label: p.label, section: p.section, kind, default, range: p.range, modes: p.modes }
    }).collect();
    Box::leak(params.into_boxed_slice())
}

pub static KINDS: std::sync::LazyLock<Vec<OverlayKind>> = std::sync::LazyLock::new(|| vec![
    OverlayKind { plugin_id: TRACK_OVERLAY, label: "Track Overlay", params: TRACK_OVERLAY_PARAMS },
    OverlayKind { plugin_id: FOUND_GRID, label: "Found Grid", params: with_overrides(TRACK_OVERLAY_PARAMS, FOUND_GRID_DEFAULTS) },
    OverlayKind { plugin_id: PUSH_TRACE, label: "Push Trace", params: with_overrides(TRACK_OVERLAY_PARAMS, PUSH_TRACE_DEFAULTS) },
    OverlayKind { plugin_id: PHYSICS_TRACE, label: "Physics Trace", params: with_overrides(TRACK_OVERLAY_PARAMS, PHYSICS_TRACE_DEFAULTS) },
]);

/// 効果の値に、その種類の既定を足す(書いていない欄も種類ごとの既定で読めるように)。
pub fn with_defaults(plugin_id: &str, params: &[(String, Value)]) -> Vec<(String, Value)> {
    let mut out = params.to_vec();
    if let Some(kind) = KINDS.iter().find(|k| k.plugin_id == plugin_id) {
        for p in kind.params {
            if !out.iter().any(|(n, _)| n == p.name) {
                out.push((p.name.to_owned(), match p.kind { ParamKind::Color(c) => Value::Color(c), _ => Value::F64(p.default[0]) }));
            }
        }
    }
    out
}

pub fn is_track_overlay(plugin_id: &str) -> bool {
    plugin_id == TRACK_OVERLAY || plugin_id == FOUND_GRID || plugin_id == PUSH_TRACE || plugin_id == PHYSICS_TRACE
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
    fn found_grid_shares_the_rows_and_only_changes_defaults() {
        let (track, found) = (&KINDS[0], &KINDS[1]);
        assert_eq!(track.params.iter().map(|p| p.name).collect::<Vec<_>>(), found.params.iter().map(|p| p.name).collect::<Vec<_>>(), "one table of rows");
        let found_values = with_defaults(FOUND_GRID, &[]);
        assert_eq!(number_of(&found_values, "method"), 2.0, "Found Grid reads layers");
        assert!(switch_of(&found_values, "grid"), "and draws the grid");
        assert_eq!(number_of(&with_defaults(TRACK_OVERLAY, &[]), "method"), 0.0, "Track Overlay keeps its own defaults");
        assert_eq!(number_of(&with_defaults(FOUND_GRID, &[("method".into(), Value::F64(1.0))]), "method"), 1.0, "a written value wins");
    }

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
