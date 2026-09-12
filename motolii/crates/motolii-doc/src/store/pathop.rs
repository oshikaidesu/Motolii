//! パス効果 — 形の層の輪郭を受けて輪郭を返す効果。絵は返さない。
//! 語彙は Lottie の shape modifier(`vector::OpKind`)そのもの。棚の欄は配置効果と同じ契約(`kind.rs`)。
//! 値は層の property なので、他の効果と同じにキーが打てる。

use crate::doc::eval::Value;
use crate::doc::store::kind::Param;
use crate::doc::store::{ResolvedEffect, ShapeNode};
use crate::doc::vector::{LineJoin, OpKind, Point, PointType, ShapeOp, TrimMultiple};

pub struct PathOpKind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
}

pub const TRIM_PATHS: &str = "motolii.trim_paths";
pub const ROUNDED_CORNERS: &str = "motolii.rounded_corners";
pub const PUCKER_BLOAT: &str = "motolii.pucker_bloat";
pub const ZIG_ZAG: &str = "motolii.zig_zag";
pub const OFFSET_PATHS: &str = "motolii.offset_paths";
pub const TWIST: &str = "motolii.twist";
pub const WIGGLE_PATHS: &str = "motolii.wiggle_paths";
pub const SMOOTH: &str = "motolii.smooth";
pub const SUBDIVIDE: &str = "motolii.subdivide";
pub const REVERSE_PATH: &str = "motolii.reverse_path";
pub const EXTEND_PATHS: &str = "motolii.extend_paths";
pub const CHOP_PATH: &str = "motolii.chop_path";
pub const RESAMPLE_PATH: &str = "motolii.resample_path";
pub const BEND: &str = "motolii.bend";

const fn point(name: &'static str, label: &'static str) -> Param {
    Param { name, label, section: "", kind: crate::doc::store::kind::ParamKind::Vec2, default: [0.0, 0.0], range: None, modes: None }
}

const MULTIPLES: &[&str] = &["Simultaneously", "Individually"];
const POINTS: &[&str] = &["Corner", "Smooth"];
const JOINS: &[&str] = &["Miter", "Round", "Bevel"];

/// 欄と既定は AE の shape 層の「追加」メニューの既定に合わせる。量は % と px と °。
pub const KINDS: &[PathOpKind] = &[
    PathOpKind { plugin_id: TRIM_PATHS, label: "Trim Paths", params: &[
        Param::number("start", "Start", 0.0, Some((0.0, 100.0))),
        Param::number("end", "End", 100.0, Some((0.0, 100.0))),
        Param::number("offset", "Offset", 0.0, None),
        Param::choice("multiple", "Trim", MULTIPLES),
    ] },
    PathOpKind { plugin_id: ROUNDED_CORNERS, label: "Rounded Corners", params: &[
        Param::number("radius", "Radius", 10.0, Some((0.0, f64::MAX))),
    ] },
    PathOpKind { plugin_id: PUCKER_BLOAT, label: "Pucker & Bloat", params: &[
        Param::number("amount", "Amount", 0.0, Some((-100.0, 100.0))),
    ] },
    PathOpKind { plugin_id: ZIG_ZAG, label: "Zig Zag", params: &[
        Param::number("amplitude", "Size", 10.0, None),
        Param::number("frequency", "Ridges", 10.0, Some((0.0, f64::MAX))),
        Param::choice("point_type", "Points", POINTS),
    ] },
    PathOpKind { plugin_id: OFFSET_PATHS, label: "Offset Paths", params: &[
        Param::number("amount", "Amount", 10.0, None),
        Param::choice("join", "Join", JOINS),
        Param::number("miter_limit", "Miter Limit", 4.0, Some((1.0, f64::MAX))),
    ] },
    PathOpKind { plugin_id: TWIST, label: "Twist", params: &[
        Param::number("angle", "Angle", 0.0, None),
        point("center", "Center"),
    ] },
    // ここから Lottie の外。AE の Wiggle Paths と Cavalry の behaviour の欄と既定に合わせる。
    PathOpKind { plugin_id: WIGGLE_PATHS, label: "Wiggle Paths", params: &[
        Param::number("size", "Size", 10.0, Some((0.0, f64::MAX))),
        Param::number("detail", "Detail", 5.0, Some((1.0, 100.0))),
        Param::choice("point_type", "Points", POINTS),
        Param::number("phase", "Phase", 0.0, None),
        Param::number("seed", "Seed", 1.0, Some((0.0, f64::MAX))),
    ] },
    PathOpKind { plugin_id: SMOOTH, label: "Smooth", params: &[
        Param::number("strength", "Strength", 50.0, Some((0.0, 100.0))),
        Param::number("iterations", "Iterations", 1.0, Some((1.0, 50.0))),
    ] },
    PathOpKind { plugin_id: SUBDIVIDE, label: "Subdivide", params: &[
        Param::number("divisions", "Divisions", 1.0, Some((1.0, 64.0))),
    ] },
    PathOpKind { plugin_id: REVERSE_PATH, label: "Reverse Path", params: &[] },
    PathOpKind { plugin_id: EXTEND_PATHS, label: "Extend Paths", params: &[
        Param::number("start", "Start", 0.0, None),
        Param::number("end", "End", 0.0, None),
    ] },
    PathOpKind { plugin_id: CHOP_PATH, label: "Chop Path", params: &[
        Param::number("length", "Length", 50.0, Some((0.0, f64::MAX))),
        Param::number("gap", "Gap", 10.0, Some((0.0, f64::MAX))),
    ] },
    PathOpKind { plugin_id: RESAMPLE_PATH, label: "Resample", params: &[
        Param::number("spacing", "Spacing", 20.0, Some((0.0, f64::MAX))),
        Param::choice("point_type", "Points", POINTS),
    ] },
    PathOpKind { plugin_id: BEND, label: "Bend", params: &[
        Param::number("angle", "Angle", 0.0, Some((-360.0, 360.0))),
        point("center", "Center"),
    ] },
];

pub fn kind(plugin_id: &str) -> Option<&'static PathOpKind> {
    KINDS.iter().find(|k| k.plugin_id == plugin_id)
}

/// 効果 1 枚を演算 1 つへ。無い欄は表の既定。パス効果でない id は None。
pub fn op(effect: &ResolvedEffect) -> Option<OpKind> {
    let kind = kind(&effect.plugin_id)?;
    let param = |name: &str| kind.params.iter().find(|p| p.name == name);
    let value = |name: &str| effect.params.iter().find(|(n, _)| n == name).map(|(_, v)| v);
    let get = |name: &str| -> f64 {
        match value(name) {
            Some(Value::F64(v)) if v.is_finite() => *v,
            _ => param(name).map_or(0.0, |p| p.default[0]),
        }
    };
    let get2 = |name: &str| -> [f64; 2] {
        match value(name) {
            Some(Value::Vec2(v)) if v.iter().all(|c| c.is_finite()) => *v,
            _ => param(name).map_or([0.0, 0.0], |p| p.default),
        }
    };
    let choice = |name: &str| get(name).round().max(0.0) as usize;
    Some(match effect.plugin_id.as_str() {
        TRIM_PATHS => OpKind::TrimPath {
            start: get("start") / 100.0,
            end: get("end") / 100.0,
            offset: get("offset") / 360.0,
            multiple: if choice("multiple") == 1 { TrimMultiple::Individually } else { TrimMultiple::Simultaneously },
        },
        ROUNDED_CORNERS => OpKind::RoundedCorners { radius: get("radius") },
        PUCKER_BLOAT => OpKind::PuckerBloat { amount: get("amount") / 100.0 },
        ZIG_ZAG => OpKind::ZigZag {
            amplitude: get("amplitude"),
            frequency: get("frequency"),
            point_type: if choice("point_type") == 1 { PointType::Smooth } else { PointType::Corner },
        },
        OFFSET_PATHS => OpKind::OffsetPath {
            amount: get("amount"),
            join: match choice("join") { 1 => LineJoin::Round, 2 => LineJoin::Bevel, _ => LineJoin::Miter },
            miter_limit: get("miter_limit"),
        },
        TWIST => {
            let c = get2("center");
            OpKind::Twist { angle: get("angle"), center: Point { x: c[0], y: c[1] } }
        }
        WIGGLE_PATHS => OpKind::Wiggle {
            size: get("size"),
            detail: get("detail"),
            point_type: if choice("point_type") == 1 { PointType::Smooth } else { PointType::Corner },
            phase: get("phase"),
            seed: get("seed").round().max(0.0) as u64,
        },
        SMOOTH => OpKind::Smooth { strength: get("strength") / 100.0, iterations: get("iterations") },
        SUBDIVIDE => OpKind::Subdivide { divisions: get("divisions") },
        REVERSE_PATH => OpKind::Reverse,
        EXTEND_PATHS => OpKind::Extend { start: get("start"), end: get("end") },
        CHOP_PATH => OpKind::Chop { length: get("length"), gap: get("gap") },
        RESAMPLE_PATH => OpKind::Resample {
            spacing: get("spacing"),
            point_type: if choice("point_type") == 1 { PointType::Smooth } else { PointType::Corner },
        },
        BEND => {
            let c = get2("center");
            OpKind::Bend { angle: get("angle"), center: Point { x: c[0], y: c[1] } }
        }
        _ => return None,
    })
}

/// 層の効果のうちパス効果を、積まれた順に演算へ。
pub fn ops(effects: &[ResolvedEffect]) -> Vec<ShapeOp> {
    effects.iter().filter_map(op).map(ShapeOp::new).collect()
}

/// 形の書類にパス効果を掛けた姿。描画と出力はこれを読む(書類そのものは書き換えない)。
pub fn with_effects(shapes: &[ShapeNode], effects: &[ResolvedEffect]) -> Vec<ShapeNode> {
    let ops = ops(effects);
    if ops.is_empty() {
        return shapes.to_vec();
    }
    fn push(node: &ShapeNode, ops: &[ShapeOp]) -> ShapeNode {
        match node {
            ShapeNode::Leaf(shape) => {
                let mut shape = shape.clone();
                shape.ops.extend(ops.iter().cloned());
                ShapeNode::Leaf(shape)
            }
            ShapeNode::Group(group) => {
                let mut group = group.clone();
                group.children = group.children.iter().map(|c| push(c, ops)).collect();
                ShapeNode::Group(group)
            }
        }
    }
    shapes.iter().map(|n| push(n, &ops)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::vector::{PathSource, Point, Shape};

    fn effect(id: &str, params: &[(&str, f64)]) -> ResolvedEffect {
        ResolvedEffect { plugin_id: id.into(), params: params.iter().map(|(n, v)| ((*n).to_owned(), Value::F64(*v))).collect(), ..Default::default() }
    }

    /// 全枚が演算になり、欄が無ければ表の既定、% は割合へ、選択肢は enum へ。点の欄は Vec2 のまま届く。
    #[test]
    fn every_card_becomes_an_op_with_table_defaults() {
        for k in KINDS {
            assert!(op(&effect(k.plugin_id, &[])).is_some(), "{} が演算にならない", k.plugin_id);
        }
        assert_eq!(op(&effect(PUCKER_BLOAT, &[("amount", 50.0)])), Some(OpKind::PuckerBloat { amount: 0.5 }));
        assert_eq!(op(&effect(ROUNDED_CORNERS, &[])), Some(OpKind::RoundedCorners { radius: 10.0 }));
        assert_eq!(
            op(&effect(TRIM_PATHS, &[("end", 25.0), ("multiple", 1.0)])),
            Some(OpKind::TrimPath { start: 0.0, end: 0.25, offset: 0.0, multiple: TrimMultiple::Individually })
        );
        assert!(matches!(op(&effect(OFFSET_PATHS, &[("join", 2.0)])), Some(OpKind::OffsetPath { join: LineJoin::Bevel, .. })));
        assert_eq!(op(&effect("motolii.blur", &[])), None);
        assert_eq!(op(&effect(SMOOTH, &[("strength", 25.0)])), Some(OpKind::Smooth { strength: 0.25, iterations: 1.0 }));
        assert_eq!(op(&effect(REVERSE_PATH, &[])), Some(OpKind::Reverse));
        let mut bend = effect(BEND, &[("angle", 90.0)]);
        bend.params.push(("center".into(), Value::Vec2([30.0, -5.0])));
        assert_eq!(op(&bend), Some(OpKind::Bend { angle: 90.0, center: Point { x: 30.0, y: -5.0 } }));
        assert!(matches!(op(&effect(WIGGLE_PATHS, &[("seed", 3.4)])), Some(OpKind::Wiggle { seed: 3, detail, .. }) if detail == 5.0));
    }

    /// 掛けた姿は葉ごとに演算が積まれ、群の中まで届き、元の書類は変わらない。
    #[test]
    fn effects_reach_every_leaf_without_touching_the_document() {
        let leaf = ShapeNode::Leaf(Shape::new(PathSource::Rectangle { size: Point { x: 10.0, y: 10.0 } }));
        let doc = vec![leaf.clone(), ShapeNode::Group(crate::doc::store::ShapeGroup { transform: crate::doc::vector::RepeaterTransform::IDENTITY, children: vec![leaf.clone()] })];
        let shown = with_effects(&doc, &[effect(PUCKER_BLOAT, &[("amount", 20.0)]), effect("motolii.blur", &[])]);
        let ops_of = |n: &ShapeNode| match n { ShapeNode::Leaf(s) => s.ops.len(), ShapeNode::Group(g) => ops_of_first(&g.children) };
        fn ops_of_first(c: &[ShapeNode]) -> usize { match &c[0] { ShapeNode::Leaf(s) => s.ops.len(), _ => 0 } }
        assert_eq!(shown.iter().map(ops_of).collect::<Vec<_>>(), vec![1, 1]);
        assert_eq!(with_effects(&doc, &[]), doc);
    }
}
