//! Lottie の shape 語彙の焼き込み — shape 幾何は今の Document ではキーフレーム化
//! されないので、ここは全部静的 JSON を返す。`next/engine/motolii-export/src/lottie.rs`
//! から移送(SP-7、2026-08-23、中身は変えていない——移送のみ)。呼び手は
//! `super`(`build_layer`)。

use motolii_store::{RepeaterTransform, Shape as VecShape, ShapeGroup, ShapeNode};
use motolii_vector::{Brush, Contour, Dash, Fill, OpKind, PathSource, StarType, Stroke};

use super::enums::{
    composite_to_int, fill_rule_to_int, gradient_type_to_int, line_cap_to_int, line_join_to_int,
    point_type_to_int, star_type_to_int, trim_multiple_to_int,
};
// ---------------------------------------------------------------------------
// shapes(静的——shape 幾何は今の Document ではキーフレーム化されない)
// ---------------------------------------------------------------------------

pub(crate) fn shape_node_to_json(node: &ShapeNode) -> serde_json::Value {
    match node {
        ShapeNode::Leaf(shape) => shape_leaf_to_group_item(shape),
        ShapeNode::Group(group) => shape_group_to_json(group),
    }
}

fn shape_group_to_json(group: &ShapeGroup) -> serde_json::Value {
    let mut it: Vec<serde_json::Value> = group.children.iter().map(shape_node_to_json).collect();
    it.push(repeater_transform_shape_item(&group.transform, 1.0, 1.0));
    serde_json::json!({ "ty": "gr", "it": it })
}

fn shape_leaf_to_group_item(shape: &VecShape) -> serde_json::Value {
    let mut it = path_source_to_json_items(&shape.source);
    for op in &shape.ops {
        if op.hidden {
            continue;
        }
        if let Some(item) = op_kind_to_json(&op.kind) {
            it.push(item);
        }
    }
    if let Some(fill) = &shape.fill {
        if !fill.hidden {
            it.push(fill_to_json(fill));
        }
    }
    if let Some(stroke) = &shape.stroke {
        if !stroke.hidden {
            it.push(stroke_to_json(stroke));
        }
    }
    it.push(repeater_transform_shape_item(&RepeaterTransform::IDENTITY, 1.0, 1.0));
    serde_json::json!({ "ty": "gr", "it": it })
}

fn static_scalar(v: f64) -> serde_json::Value {
    serde_json::json!({ "a": 0, "k": v })
}

fn static_vec2(v: [f64; 2]) -> serde_json::Value {
    serde_json::json!({ "a": 0, "k": [v[0], v[1]] })
}

/// `motolii_vector::PathSource` → Lottie shape item(複数個返る場合がある)。
///
/// **`PathSource::Bezier` だけが1個とは限らない** — `motolii_vector::Path` は
/// `Vec<Contour>`(複数輪郭、`geom.rs` の型)であって `motolii_eval::Path`
/// (mask 形状が使う単一輪郭)とは別の型。Lottie の `sh`(Path)は1輪郭しか運べないので、
/// 複数輪郭は**同じ group 内に並ぶ複数の `sh` 要素**として書く——fill/stroke を1つだけ
/// 後ろに続ければ、複数の `sh` に対して同じ塗りが乗る(AE の「1つの shape に複数
/// サブパス」を Lottie が表す標準的な形)。
fn path_source_to_json_items(source: &PathSource) -> Vec<serde_json::Value> {
    match source {
        PathSource::Bezier(contours) => contours
            .iter()
            .map(contour_to_path_item)
            .collect(),
        PathSource::Rectangle { size } => vec![serde_json::json!({
            "ty": "rc",
            "p": static_vec2([0.0, 0.0]),
            "s": static_vec2([size.x, size.y]),
            "r": static_scalar(0.0),
        })],
        PathSource::Ellipse { size } => vec![serde_json::json!({
            "ty": "el",
            "p": static_vec2([0.0, 0.0]),
            "s": static_vec2([size.x, size.y]),
        })],
        PathSource::PolyStar {
            points,
            outer_radius,
            inner_radius,
            star_type,
        } => {
            let mut obj = serde_json::json!({
                "ty": "sr",
                "p": static_vec2([0.0, 0.0]),
                "r": static_scalar(0.0),
                "pt": static_scalar(*points),
                "or": static_scalar(*outer_radius),
                "os": static_scalar(0.0),
                "sy": star_type_to_int(*star_type),
            });
            if matches!(star_type, StarType::Star) {
                obj["ir"] = static_scalar(*inner_radius);
                obj["is"] = static_scalar(0.0);
            }
            vec![obj]
        }
    }
}

fn contour_to_path_item(contour: &Contour) -> serde_json::Value {
    let v: Vec<[f64; 2]> = contour.vertices.iter().map(|p| [p.point.x, p.point.y]).collect();
    let i: Vec<[f64; 2]> = contour
        .vertices
        .iter()
        .map(|p| [p.in_tangent.x, p.in_tangent.y])
        .collect();
    let o: Vec<[f64; 2]> = contour
        .vertices
        .iter()
        .map(|p| [p.out_tangent.x, p.out_tangent.y])
        .collect();
    serde_json::json!({
        "ty": "sh",
        "ks": { "a": 0, "k": { "c": contour.closed, "v": v, "i": i, "o": o } },
    })
}

fn op_kind_to_json(kind: &OpKind) -> Option<serde_json::Value> {
    Some(match kind {
        OpKind::TrimPath { start, end, offset, multiple } => serde_json::json!({
            "ty": "tm",
            "s": static_scalar(start * 100.0),
            "e": static_scalar(end * 100.0),
            "o": static_scalar(*offset),
            "m": trim_multiple_to_int(*multiple),
        }),
        OpKind::Repeater {
            copies,
            offset,
            transform,
            composite,
            start_opacity,
            end_opacity,
        } => serde_json::json!({
            "ty": "rp",
            "c": static_scalar(*copies),
            "o": static_scalar(*offset),
            "m": composite_to_int(*composite),
            "tr": repeater_transform_shape_item(transform, *start_opacity, *end_opacity),
        }),
        OpKind::RoundedCorners { radius } => serde_json::json!({
            "ty": "rd",
            "r": static_scalar(*radius),
        }),
        OpKind::PuckerBloat { amount } => serde_json::json!({
            "ty": "pb",
            "a": static_scalar(amount * 100.0),
        }),
        OpKind::ZigZag { amplitude, frequency, point_type } => serde_json::json!({
            "ty": "zz",
            "s": static_scalar(*amplitude),
            "r": static_scalar(*frequency),
            "pt": static_scalar(point_type_to_int(*point_type) as f64),
        }),
        OpKind::OffsetPath { amount, join, miter_limit } => serde_json::json!({
            "ty": "op",
            "a": static_scalar(*amount),
            "lj": line_join_to_int(*join),
            "ml": static_scalar(*miter_limit),
        }),
        OpKind::Twist { angle, center } => serde_json::json!({
            "ty": "tw",
            "a": static_scalar(*angle),
            "c": static_vec2([center.x, center.y]),
        }),
    })
}

/// `repeater.tr`(`shapes/repeater-transform`)。単体 shape の恒等変換にも、
/// `ShapeGroup::transform`/`OpKind::Repeater::transform` にも使う共通口。
fn repeater_transform_shape_item(
    t: &RepeaterTransform,
    start_opacity: f64,
    end_opacity: f64,
) -> serde_json::Value {
    serde_json::json!({
        "ty": "tr",
        "a": static_vec2([t.anchor.x, t.anchor.y]),
        "p": static_vec2([t.position.x, t.position.y]),
        "s": static_vec2([t.scale.x * 100.0, t.scale.y * 100.0]),
        "r": static_scalar(t.rotation),
        "o": static_scalar(100.0),
        "so": static_scalar(start_opacity * 100.0),
        "eo": static_scalar(end_opacity * 100.0),
    })
}

fn fill_to_json(fill: &Fill) -> serde_json::Value {
    match &fill.brush {
        Brush::Solid(rgb) => serde_json::json!({
            "ty": "fl",
            "c": static_vec3([rgb.r, rgb.g, rgb.b]),
            "o": static_scalar(fill.opacity * 100.0),
            "r": fill_rule_to_int(fill.rule),
        }),
        Brush::Gradient(g) => serde_json::json!({
            "ty": "gf",
            "o": static_scalar(fill.opacity * 100.0),
            "s": static_vec2([g.start.x, g.start.y]),
            "e": static_vec2([g.end.x, g.end.y]),
            "t": gradient_type_to_int(g.kind),
            "g": gradient_colors_json(&g.stops),
            "r": fill_rule_to_int(fill.rule),
        }),
    }
}

fn stroke_to_json(stroke: &Stroke) -> serde_json::Value {
    let mut obj = match &stroke.brush {
        Brush::Solid(rgb) => serde_json::json!({
            "ty": "st",
            "c": static_vec3([rgb.r, rgb.g, rgb.b]),
        }),
        Brush::Gradient(g) => serde_json::json!({
            "ty": "gs",
            "s": static_vec2([g.start.x, g.start.y]),
            "e": static_vec2([g.end.x, g.end.y]),
            "t": gradient_type_to_int(g.kind),
            "g": gradient_colors_json(&g.stops),
        }),
    };
    obj["o"] = static_scalar(stroke.opacity * 100.0);
    obj["w"] = static_scalar(stroke.width);
    obj["lc"] = serde_json::json!(line_cap_to_int(stroke.cap));
    obj["lj"] = serde_json::json!(line_join_to_int(stroke.join));
    obj["ml2"] = static_scalar(stroke.miter_limit);
    if let Some(dash) = &stroke.dash {
        obj["d"] = dash_to_json(dash);
    }
    obj
}

fn dash_to_json(dash: &Dash) -> serde_json::Value {
    let mut out = Vec::new();
    for (i, len) in dash.pattern.iter().enumerate() {
        let tag = if i % 2 == 0 { "d" } else { "g" };
        out.push(serde_json::json!({ "n": tag, "v": static_scalar(*len) }));
    }
    out.push(serde_json::json!({ "n": "o", "v": static_scalar(dash.offset) }));
    serde_json::Value::Array(out)
}

fn static_vec3(v: [f64; 3]) -> serde_json::Value {
    serde_json::json!({ "a": 0, "k": [v[0], v[1], v[2]] })
}

fn gradient_colors_json(stops: &[motolii_vector::GradientStop]) -> serde_json::Value {
    let mut flat = Vec::with_capacity(stops.len() * 4);
    for stop in stops {
        flat.push(stop.offset);
        flat.push(stop.color.r);
        flat.push(stop.color.g);
        flat.push(stop.color.b);
    }
    serde_json::json!({ "p": stops.len(), "k": { "a": 0, "k": flat } })
}

