//! 配置効果 — 素材を受けて配置の集合を返す効果。絵は返さない。
//! Repeater は 1 枚で、形はトグル(裁定 2026-09-06、reviews/2026-09-06-placement-effect.md)。
//! 配置は `(params, seed)` の純関数。

use crate::doc::core::{noise, RationalTime};
use crate::doc::eval::Value;
/// 欄は棚の共通契約(`kind.rs`)。配置効果は section(Shape / Each / Random)と modes を使う。
pub use crate::doc::store::kind::{Param as PlacementParam, ParamKind};

/// Inspector の格子の 1 行: 同じ属性の Each(番号順のずれ)と Random(種の散らし)。
pub struct GridRow {
    pub label: &'static str,
    pub unit: &'static str,
    pub each: Option<&'static str>,
    pub random: Option<&'static str>,
    /// Vec2 の param を 1 成分ずつ行にする時の軸。
    pub axis: Option<usize>,
    /// 触っても一見変化が無い欄。Inspector は Advanced に畳む(裁定 2026-09-07)。
    pub advanced: bool,
}

pub struct PlacementKind {
    pub plugin_id: &'static str,
    pub evaluate: fn(&crate::doc::store::kind::PlacementInput<'_>) -> Vec<crate::doc::store::kind::PlacementOutput>,
    pub label: &'static str,
    pub params: &'static [PlacementParam],
    /// 形の行に並ぶ param(形のトグルで出入りする物)。
    pub shape: &'static [&'static str],
    pub grid: &'static [GridRow],
}

const fn row(label: &'static str, unit: &'static str, each: Option<&'static str>, random: Option<&'static str>, axis: Option<usize>) -> GridRow {
    GridRow { label, unit, each, random, axis, advanced: false }
}

const fn advanced(label: &'static str, unit: &'static str, each: Option<&'static str>, random: Option<&'static str>) -> GridRow {
    GridRow { label, unit, each, random, axis: None, advanced: true }
}

pub const REPEAT: &str = "motolii.repeat";

pub const LINE: u8 = 0;
pub const CIRCLE: u8 = 1;
pub const GRID: u8 = 2;
pub const SHAPES: &[&str] = &["Line", "Circle", "Grid"];

/// 鏡映(Cavalry の Mirror filter・AE の Mirror 効果・C4D MoGraph Symmetry)。元と写しを配置で返す。
/// Radial は万華鏡(先例に名は無い: Cavalry の Mirror は 1 本の線、AE は CC Kaleida)。扇ごとに回し、隣は裏返す。
pub const MIRROR: &str = "motolii.mirror";
pub const HORIZONTAL: u8 = 0;
pub const VERTICAL: u8 = 1;
pub const BOTH: u8 = 2;
pub const RADIAL: u8 = 3;
pub const AXES: &[&str] = &["Horizontal", "Vertical", "Both", "Radial"];

/// グループに掛けた時、配置ごとにどの子を引くか。
pub const PICK_RANDOM: u8 = 0;
pub const PICK_ITERATE: u8 = 1;
pub const PICKS: &[&str] = &["Random", "Iterate"];
/// 層の変形(回転・大きさ)を複製 1 つずつに掛けるか、並び全体に掛けるか(2026-09-14 利用者裁定: 選べる)。
/// Whole は AE のシェイプ層の Repeater・Cavalry Duplicator・C4D Cloner と同じく、層を回すと並びごと回る。
pub const TRANSFORM_EACH: u8 = 0;
pub const TRANSFORM_WHOLE: u8 = 1;
pub const TRANSFORMS: &[&str] = &["Each", "Whole"];
/// 子ごとの重みの param 名。`share.<layer id>`。無ければ 100。
pub const SHARE_PREFIX: &str = "share.";
pub const SHARE_DEFAULT: f64 = 100.0;

const fn number(name: &'static str, label: &'static str, section: &'static str, default: f64, range: Option<(f64, f64)>, modes: Option<&'static [u8]>) -> PlacementParam {
    PlacementParam { name, label, section, kind: ParamKind::Number, default: [default, 0.0], range, modes }
}

const fn vec2(name: &'static str, label: &'static str, section: &'static str, default: [f64; 2], modes: Option<&'static [u8]>) -> PlacementParam {
    PlacementParam { name, label, section, kind: ParamKind::Vec2, default, range: None, modes }
}

/// 欄は使う人が決める順: いくつ → どんな形 → 1 つずつどう変えるか → どう散らすか。
pub const KINDS: &[PlacementKind] = &[PlacementKind {
    evaluate: |input| placements(kind(REPEAT).unwrap(), input.params).into_iter().map(Into::into).collect(),
    plugin_id: REPEAT,
    label: "Repeater",
    params: &[
        number("count", "Count", "Shape", 3.0, Some((1.0, 1000.0)), None),
        PlacementParam { name: "mode", label: "Along", section: "Shape", kind: ParamKind::Choice(SHAPES), default: [0.0, 0.0], range: Some((0.0, 2.0)), modes: None },
        PlacementParam { name: "pick", label: "Pick", section: "Shape", kind: ParamKind::Choice(PICKS), default: [0.0, 0.0], range: Some((0.0, 1.0)), modes: None },
        PlacementParam { name: "transform", label: "Transform", section: "Shape", kind: ParamKind::Choice(TRANSFORMS), default: [0.0, 0.0], range: Some((0.0, 1.0)), modes: None },
        number("columns", "Columns", "Shape", 3.0, Some((1.0, 1000.0)), Some(&[GRID])),
        number("radius", "Radius", "Shape", 200.0, Some((0.0, f64::MAX)), Some(&[CIRCLE])),
        number("start_angle", "Start", "Shape", 0.0, None, Some(&[CIRCLE])),
        number("sweep", "Sweep", "Shape", 360.0, Some((0.0, 360.0)), Some(&[CIRCLE])),
        vec2("position_each", "Position", "Each", [100.0, 0.0], None),
        number("position_z_each", "Position Z", "Each", 0.0, None, None),
        number("rotation_each", "Rotation", "Each", 0.0, None, None),
        number("scale_each", "Scale", "Each", 0.0, Some((-100.0, 100.0)), None),
        number("opacity_each", "Opacity", "Each", 0.0, Some((-1.0, 1.0)), None),
        number("delay_each", "Delay", "Each", 0.0, None, None),
        vec2("position_random", "Position", "Random", [0.0, 0.0], None),
        number("position_z_random", "Position Z", "Random", 0.0, Some((0.0, f64::MAX)), None),
        number("rotation_random", "Rotation", "Random", 0.0, Some((0.0, 360.0)), None),
        number("scale_random", "Scale", "Random", 0.0, Some((0.0, 100.0)), None),
        number("opacity_random", "Opacity", "Random", 0.0, Some((0.0, 1.0)), None),
        number("delay_random", "Delay", "Random", 0.0, Some((0.0, f64::MAX)), None),
        number("seed", "Seed", "Random", 0.0, Some((0.0, 9999.0)), None),
    ],
    shape: &["columns", "radius", "start_angle", "sweep"],
    grid: &[
        row("Position X", "px", Some("position_each"), Some("position_random"), Some(0)),
        row("Position Y", "px", Some("position_each"), Some("position_random"), Some(1)),
        row("Position Z", "px", Some("position_z_each"), Some("position_z_random"), None),
        row("Rotation", "°", Some("rotation_each"), Some("rotation_random"), None),
        row("Scale", "%", Some("scale_each"), Some("scale_random"), None),
        row("Opacity", "", Some("opacity_each"), Some("opacity_random"), None),
        advanced("Delay", "s", Some("delay_each"), Some("delay_random")),
        advanced("Seed", "", None, Some("seed")),
    ],
}, PlacementKind {
    evaluate: |input| placements(kind(MIRROR).unwrap(), input.params).into_iter().map(Into::into).collect(),
    plugin_id: MIRROR,
    label: "Mirror",
    params: &[
        PlacementParam { name: "mode", label: "Axis", section: "Shape", kind: ParamKind::Choice(AXES), default: [0.0, 0.0], range: Some((0.0, 3.0)), modes: None },
        number("segments", "Segments", "Shape", 6.0, Some((2.0, 64.0)), Some(&[RADIAL])),
        vec2("centre", "Centre", "Shape", [0.0, 0.0], None),
    ],
    shape: &["segments", "centre"],
    grid: &[],
}];

/// 形の行に出る欄の単位。格子の単位は `GridRow::unit`。
pub fn unit(name: &str) -> &'static str {
    match name {
        "radius" | "centre" => "px",
        "start_angle" | "sweep" => "°",
        _ => "",
    }
}

/// 形で変わる既定値。Circle の Position Each は 0 — 並べる向きの既定 [100, 0] が円を右へずらした螺旋にしないように
/// (2026-09-14 利用者裁定。Each を足せば螺旋にもできる)。窓の欄と配置が同じここを読む。
pub fn default_for(kind: &PlacementKind, name: &str, mode: u8) -> Option<Value> {
    let param = kind.params.iter().find(|p| p.name == name)?;
    if mode == CIRCLE && name == "position_each" {
        return Some(Value::Vec2([0.0, 0.0]));
    }
    Some(param.default_value())
}

pub fn kind(plugin_id: &str) -> Option<&'static PlacementKind> {
    KINDS.iter().find(|kind| kind.plugin_id == plugin_id)
}

pub fn program(plugin_id: &str) -> Option<crate::doc::store::kind::PlacementProgram> {
    kind(plugin_id).map(|kind| crate::doc::store::kind::PlacementProgram {
        plugin_id: kind.plugin_id,
        needs_position: false,
        evaluate: kind.evaluate,
        pick: picks,
        moves_whole: |params| params.iter().any(|(n, v)| n == "transform" && matches!(v, Value::F64(x) if x.round() == f64::from(TRANSFORM_WHOLE))),
    })
}

pub use crate::doc::store::Placement;

const TIME_DEN: i64 = 1_000_000;

pub fn placements(kind: &PlacementKind, params: &[(String, Value)]) -> Vec<Placement> {
    let param = |name: &str| kind.params.iter().find(|p| p.name == name);
    let value = |name: &str| params.iter().find(|(n, _)| n == name).map(|(_, v)| v.clone());
    let get = |name: &str| -> f64 {
        match value(name) {
            Some(Value::F64(v)) if v.is_finite() => v,
            _ => param(name).map_or(0.0, |p| p.default[0]),
        }
    };
    let choices = param("mode").and_then(|p| p.choices()).map_or(1, <[&str]>::len);
    let mode = get("mode").round().clamp(0.0, (choices - 1) as f64) as u8;
    let get2 = |name: &str| -> [f64; 2] {
        match (value(name), default_for(kind, name, mode)) {
            (Some(Value::Vec2(v)), _) if v.iter().all(|c| c.is_finite()) => v,
            (_, Some(Value::Vec2(v))) => v,
            _ => [0.0, 0.0],
        }
    };
    if kind.plugin_id == MIRROR {
        return mirrors(mode, get("segments").round().clamp(2.0, 64.0) as u32, get2("centre"));
    }
    let count = get("count").round().clamp(1.0, 1000.0) as u32;
    let seed = get("seed").round() as i64 as u64;
    let each = get2("position_each");
    let columns = get("columns").round().clamp(1.0, 1000.0) as u32;
    let (radius, start, sweep) = (get("radius"), get("start_angle"), get("sweep"));
    let random = get2("position_random");
    (0..count)
        .map(|index| {
            let i = f64::from(index);
            let u = |channel: u64| noise(seed, index, channel);
            let shape = match mode {
                CIRCLE => {
                    let full = sweep >= 360.0;
                    let share = if full { i / f64::from(count) } else { i / f64::from(count.max(2) - 1) };
                    let angle = (start + sweep * share).to_radians();
                    [radius * angle.cos(), radius * angle.sin()]
                }
                GRID => [each[0] * f64::from(index % columns), each[1] * f64::from(index / columns)],
                _ => [each[0] * i, each[1] * i],
            };
            // 円でも Position の Each は効く(番号順に流れる)。
            let shape = if mode == CIRCLE { [shape[0] + each[0] * i, shape[1] + each[1] * i] } else { shape };
            let seconds = get("delay_each") * i + get("delay_random") * u(5);
            Placement {
                index,
                offset: [
                    (shape[0] + random[0] * u(0)) as f32,
                    (shape[1] + random[1] * u(1)) as f32,
                ],
                rotation_degrees: (get("rotation_each") * i + get("rotation_random") * u(2)) as f32,
                scale: ((1.0 + get("scale_each") / 100.0).max(0.0).powf(i)
                    * (1.0 + get("scale_random") / 100.0 * u(3)).max(0.0)) as f32,
                offset_z: (get("position_z_each") * i + get("position_z_random") * u(7)) as f32,
                opacity: (1.0 + get("opacity_each") * i + get("opacity_random") * u(4)).clamp(0.0, 1.0) as f32,
                time_offset: RationalTime::try_new((seconds * TIME_DEN as f64).round() as i64, TIME_DEN)
                    .unwrap_or(RationalTime::ZERO),
                stretch: [1.0, 1.0],
            }
        })
        .collect()
}

/// 鏡映の配置。写し k は `x ↦ R(θ) S (x − c) + c`(c = 層の位置からの Centre、S = 裏返し)。
/// `Placement` は層の位置を中心に回す形なので、ずれは `c − R S c`。Radial は扇 `360 / segments` ごとに回し、奇数番を裏返す。
fn mirrors(axis: u8, segments: u32, centre: [f64; 2]) -> Vec<Placement> {
    let flips: Vec<(f64, [f32; 2])> = match axis {
        HORIZONTAL => vec![(0.0, [1.0, 1.0]), (0.0, [-1.0, 1.0])],
        VERTICAL => vec![(0.0, [1.0, 1.0]), (0.0, [1.0, -1.0])],
        BOTH => vec![(0.0, [1.0, 1.0]), (0.0, [-1.0, 1.0]), (0.0, [1.0, -1.0]), (0.0, [-1.0, -1.0])],
        _ => (0..segments).map(|k| (360.0 * f64::from(k) / f64::from(segments), if k % 2 == 1 { [-1.0, 1.0] } else { [1.0, 1.0] })).collect(),
    };
    let c = glam::DVec2::from(centre);
    flips
        .into_iter()
        .enumerate()
        .map(|(index, (degrees, stretch))| {
            let moved = glam::DMat2::from_angle(degrees.to_radians()) * (c * glam::DVec2::new(f64::from(stretch[0]), f64::from(stretch[1])));
            let offset = c - moved;
            Placement {
                index: index as u32,
                offset: [offset.x as f32, offset.y as f32],
                rotation_degrees: degrees as f32,
                scale: 1.0,
                offset_z: 0.0,
                opacity: 1.0,
                time_offset: RationalTime::ZERO,
                stretch,
            }
        })
        .collect()
}

/// グループ丸ごと増やすか(true)、配置ごとに子を 1 つ引くか(false)。
/// グループの子から、配置ごとに 1 つ引く。返すのは `children` の添字。
/// Random は `share.<id>` の重みと種で、Iterate は順番に回す。重みが全部 0 なら順番。
pub fn picks(params: &[(String, Value)], children: &[u64], count: usize) -> Vec<usize> {
    if children.is_empty() {
        return Vec::new();
    }
    let get = |name: &str| -> Option<f64> {
        params.iter().find_map(|(n, v)| match v {
            Value::F64(v) if n == name && v.is_finite() => Some(*v),
            _ => None,
        })
    };
    let pick = get("pick").unwrap_or(0.0).round() as u8;
    let seed = get("seed").unwrap_or(0.0).round() as i64 as u64;
    let weights: Vec<f64> = children
        .iter()
        .map(|id| get(&format!("{SHARE_PREFIX}{id}")).unwrap_or(SHARE_DEFAULT).max(0.0))
        .collect();
    let total: f64 = weights.iter().sum();
    (0..count)
        .map(|index| {
            if pick == PICK_ITERATE || total <= 0.0 {
                return index % children.len();
            }
            let u = (noise(seed, index as u32, 6) + 1.0) / 2.0 * total;
            let mut acc = 0.0;
            for (i, w) in weights.iter().enumerate() {
                acc += w;
                if u < acc {
                    return i;
                }
            }
            children.len() - 1
        })
        .collect()
}

/// `[-1, 1]` の一様乱数。同じ (seed, index, channel) は同じ値。

#[cfg(test)]
mod pure_function_contract {
    use super::*;

    fn repeat(params: &[(&str, Value)]) -> Vec<Placement> {
        let params: Vec<_> = params.iter().map(|(n, v)| ((*n).to_owned(), v.clone())).collect();
        placements(kind(REPEAT).unwrap(), &params)
    }
    fn n(v: f64) -> Value {
        Value::F64(v)
    }

    #[test]
    fn a_line_steps_each_copy_by_its_number() {
        let out = repeat(&[("count", n(4.0)), ("position_each", Value::Vec2([10.0, 5.0])), ("rotation_each", n(5.0)), ("scale_each", n(100.0)), ("delay_each", n(0.5))]);
        assert_eq!(out.len(), 4);
        assert_eq!(out[0].offset, [0.0, 0.0]);
        assert_eq!(out[3].offset, [30.0, 15.0]);
        assert_eq!(out[2].rotation_degrees, 10.0);
        assert_eq!(out[3].scale, 8.0);
        assert_eq!(out[1].time_offset, RationalTime::try_new(1, 2).unwrap());
        assert_eq!(out[0].time_offset, RationalTime::ZERO);
    }

    #[test]
    fn a_circle_shares_the_sweep_and_a_full_turn_does_not_double_the_first_copy() {
        let out = repeat(&[("mode", n(f64::from(CIRCLE))), ("count", n(4.0)), ("radius", n(10.0)), ("position_each", Value::Vec2([0.0, 0.0]))]);
        let rounded: Vec<[f32; 2]> = out.iter().map(|p| p.offset.map(|c| c.round())).collect();
        assert_eq!(rounded, [[10.0, 0.0], [0.0, 10.0], [-10.0, 0.0], [-0.0, -10.0]]);
        let half = repeat(&[("mode", n(f64::from(CIRCLE))), ("count", n(3.0)), ("radius", n(10.0)), ("sweep", n(180.0)), ("position_each", Value::Vec2([0.0, 0.0]))]);
        let rounded: Vec<[f32; 2]> = half.iter().map(|p| p.offset.map(|c| c.round())).collect();
        assert_eq!(rounded, [[10.0, 0.0], [0.0, 10.0], [-10.0, 0.0]]);
    }

    #[test]
    fn a_circle_left_at_its_defaults_closes_around_the_layer() {
        let out = repeat(&[("mode", n(f64::from(CIRCLE))), ("count", n(4.0)), ("radius", n(10.0))]);
        let centre = out.iter().fold([0.0f32; 2], |c, p| [c[0] + p.offset[0] / 4.0, c[1] + p.offset[1] / 4.0]);
        assert!(centre[0].abs() < 1e-4 && centre[1].abs() < 1e-4, "{centre:?}");
        assert_eq!(default_for(kind(REPEAT).unwrap(), "position_each", LINE), Some(Value::Vec2([100.0, 0.0])));
        let spiral = repeat(&[("mode", n(f64::from(CIRCLE))), ("count", n(4.0)), ("radius", n(10.0)), ("position_each", Value::Vec2([5.0, 0.0]))]);
        assert_eq!(spiral[3].offset.map(|c| c.round()), [15.0, -10.0], "an Each the user set still flows along the circle");
    }

    #[test]
    fn a_grid_wraps_at_the_column_count() {
        let out = repeat(&[("mode", n(f64::from(GRID))), ("count", n(5.0)), ("columns", n(2.0)), ("position_each", Value::Vec2([10.0, 20.0]))]);
        let offsets: Vec<[f32; 2]> = out.iter().map(|p| p.offset).collect();
        assert_eq!(offsets, [[0.0, 0.0], [10.0, 0.0], [0.0, 20.0], [10.0, 20.0], [0.0, 40.0]]);
    }

    #[test]
    fn the_same_seed_scatters_the_same_way_and_another_seed_differently() {
        let scatter = |seed: f64| repeat(&[("count", n(5.0)), ("position_each", Value::Vec2([0.0, 0.0])), ("position_random", Value::Vec2([20.0, 0.0])), ("seed", n(seed))]);
        let (a, b, c) = (scatter(7.0), scatter(7.0), scatter(8.0));
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.iter().all(|p| p.offset[0].abs() <= 20.0 && p.offset[1] == 0.0));
        assert!(a.iter().any(|p| p.offset[0] != 0.0));
    }

    #[test]
    fn the_first_copy_of_a_line_is_the_layer_itself() {
        let out = repeat(&[("count", n(3.0)), ("scale_each", n(50.0)), ("opacity_each", n(-0.25))]);
        assert_eq!(out[0].affine2(glam::Vec2::new(3.0, 4.0)), glam::Affine2::IDENTITY);
        assert_eq!(out[0].opacity, 1.0);
        assert_eq!(out[2].opacity, 0.5);
    }

    #[test]
    fn rotation_and_scale_turn_about_the_pivot() {
        let out = repeat(&[("count", n(2.0)), ("position_each", Value::Vec2([0.0, 0.0])), ("rotation_each", n(90.0)), ("scale_each", n(100.0))]);
        let pivot = glam::Vec2::new(10.0, 10.0);
        let m = out[1].affine2(pivot);
        assert!(m.transform_point2(pivot).abs_diff_eq(pivot, 1e-4));
        assert!(m.transform_point2(glam::Vec2::new(11.0, 10.0)).abs_diff_eq(glam::Vec2::new(10.0, 12.0), 1e-4));
    }

    #[test]
    fn a_group_hands_out_its_children_by_share_or_in_turn() {
        let children = [11u64, 12, 13];
        let p = |pairs: &[(&str, f64)]| pairs.iter().map(|(n, v)| ((*n).to_owned(), Value::F64(*v))).collect::<Vec<_>>();
        let turn = picks(&p(&[("pick", 1.0)]), &children, 5);
        assert_eq!(turn, [0, 1, 2, 0, 1]);
        let only_second = picks(&p(&[("share.11", 0.0), ("share.13", 0.0)]), &children, 6);
        assert!(only_second.iter().all(|&i| i == 1));
        let mixed = picks(&p(&[("share.11", 300.0), ("share.12", 100.0), ("share.13", 0.0), ("seed", 3.0)]), &children, 400);
        let first = mixed.iter().filter(|&&i| i == 0).count();
        assert!(mixed.iter().all(|&i| i != 2));
        assert!((250..=350).contains(&first), "about three quarters go to the heavy child, got {first}");
        assert_eq!(mixed, picks(&p(&[("share.11", 300.0), ("share.12", 100.0), ("share.13", 0.0), ("seed", 3.0)]), &children, 400));
        assert!(picks(&p(&[]), &[], 3).is_empty());
    }

    #[test]
    fn a_line_can_step_into_depth_for_the_2_5d_copies() {
        let out = repeat(&[("count", n(3.0)), ("position_z_each", n(10.0))]);
        assert_eq!(out.iter().map(|p| p.offset_z).collect::<Vec<_>>(), [0.0, 10.0, 20.0]);
        let m = out[2].affine3(glam::Vec3::ZERO);
        assert!(m.transform_point3(glam::Vec3::ZERO).abs_diff_eq(glam::Vec3::new(200.0, 0.0, 20.0), 1e-4));
        assert_eq!(out[1].affine2(glam::Vec2::ZERO), glam::Affine2::from_translation(glam::Vec2::new(100.0, 0.0)), "2D ignores depth");
        let scattered = repeat(&[("count", n(4.0)), ("position_z_random", n(5.0)), ("seed", n(2.0))]);
        assert!(scattered.iter().all(|p| p.offset_z.abs() <= 5.0) && scattered.iter().any(|p| p.offset_z != 0.0));
    }

    fn mirror(params: &[(&str, Value)]) -> Vec<Placement> {
        let params: Vec<_> = params.iter().map(|(n, v)| ((*n).to_owned(), v.clone())).collect();
        placements(kind(MIRROR).unwrap(), &params)
    }

    #[test]
    fn a_horizontal_mirror_reflects_across_the_centre_line_and_keeps_the_original() {
        let pivot = glam::Vec2::new(100.0, 50.0);
        let out = mirror(&[("centre", Value::Vec2([10.0, 0.0]))]);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].affine2(pivot), glam::Affine2::IDENTITY);
        let m = out[1].affine2(pivot);
        assert!(m.transform_point2(glam::Vec2::new(112.0, 53.0)).abs_diff_eq(glam::Vec2::new(108.0, 53.0), 1e-4), "x mirrors about pivot.x + 10, y stays");
        assert!(m.transform_point2(pivot + glam::Vec2::new(10.0, 0.0)).abs_diff_eq(pivot + glam::Vec2::new(10.0, 0.0), 1e-4), "the axis is fixed");
        let vertical = mirror(&[("mode", n(f64::from(VERTICAL)))]);
        assert!(vertical[1].affine2(pivot).transform_point2(glam::Vec2::new(100.0, 57.0)).abs_diff_eq(glam::Vec2::new(100.0, 43.0), 1e-4));
    }

    #[test]
    fn both_gives_the_four_quadrants_and_radial_alternates_the_flip_around_the_centre() {
        let both = mirror(&[("mode", n(f64::from(BOTH)))]);
        assert_eq!(both.iter().map(|p| p.stretch).collect::<Vec<_>>(), [[1.0, 1.0], [-1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]]);
        let out = mirror(&[("mode", n(f64::from(RADIAL))), ("segments", n(6.0)), ("centre", Value::Vec2([0.0, -30.0]))]);
        assert_eq!(out.len(), 6);
        assert_eq!(out.iter().map(|p| p.rotation_degrees).collect::<Vec<_>>(), [0.0, 60.0, 120.0, 180.0, 240.0, 300.0]);
        assert!(out.iter().enumerate().all(|(k, p)| p.stretch == if k % 2 == 1 { [-1.0, 1.0] } else { [1.0, 1.0] }));
        let pivot = glam::Vec2::new(7.0, 9.0);
        let centre = pivot + glam::Vec2::new(0.0, -30.0);
        for p in &out {
            assert!(p.affine2(pivot).transform_point2(centre).abs_diff_eq(centre, 1e-3), "every wedge turns about the centre: {p:?}");
        }
        let unit = glam::Vec2::new(1.0, 0.0);
        let turned = out[1].affine2(pivot).transform_point2(centre + unit) - centre;
        assert!(turned.abs_diff_eq(glam::Vec2::from_angle(60f32.to_radians()).rotate(-unit), 1e-3), "wedge 1 is flipped then turned 60°");
        let kind = kind(MIRROR).unwrap();
        assert!(kind.params.iter().find(|p| p.name == "segments").unwrap().shown(RADIAL));
        assert!(!kind.params.iter().find(|p| p.name == "segments").unwrap().shown(HORIZONTAL));
        assert_eq!(mirror(&[]), mirror(&[]), "a pure function of its params");
    }

    #[test]
    fn only_the_chosen_shape_shows_its_fields() {
        let kind = kind(REPEAT).unwrap();
        let shown = |mode: u8| kind.params.iter().filter(|p| p.shown(mode)).map(|p| p.name).collect::<Vec<_>>();
        assert!(shown(LINE).contains(&"position_each") && !shown(LINE).contains(&"radius"));
        assert!(shown(CIRCLE).contains(&"radius") && !shown(CIRCLE).contains(&"columns"));
        assert!(shown(GRID).contains(&"columns"));
        let grid = kind.grid.iter().flat_map(|r| [r.each, r.random]).flatten();
        assert!(grid.clone().all(|name| kind.params.iter().any(|p| p.name == name)), "every grid cell names a real param");
    }
}
