//! 配置効果 — 素材を受けて配置の集合を返す効果。絵は返さない。
//! Repeater は 1 枚で、形はトグル(裁定 2026-09-06、reviews/2026-09-06-placement-effect.md)。
//! 配置は `(params, seed)` の純関数。

use crate::doc::core::RationalTime;
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

/// グループに掛けた時、配置ごとにどの子を引くか。
pub const PICK_RANDOM: u8 = 0;
pub const PICK_ITERATE: u8 = 1;
pub const PICKS: &[&str] = &["Random", "Iterate"];
/// グループに掛けた時、配置ごとに子を 1 つ引くか、グループ丸ごと増やすか。
pub const SUBJECT_ONE: u8 = 0;
pub const SUBJECT_WHOLE: u8 = 1;
pub const SUBJECTS: &[&str] = &["One child", "Whole group"];
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
    plugin_id: REPEAT,
    label: "Repeater",
    params: &[
        number("count", "Count", "Shape", 3.0, Some((1.0, 1000.0)), None),
        PlacementParam { name: "mode", label: "Along", section: "Shape", kind: ParamKind::Choice(SHAPES), default: [0.0, 0.0], range: Some((0.0, 2.0)), modes: None },
        PlacementParam { name: "subject", label: "Copies", section: "Shape", kind: ParamKind::Choice(SUBJECTS), default: [0.0, 0.0], range: Some((0.0, 1.0)), modes: None },
        PlacementParam { name: "pick", label: "Pick", section: "Shape", kind: ParamKind::Choice(PICKS), default: [0.0, 0.0], range: Some((0.0, 1.0)), modes: None },
        number("columns", "Columns", "Shape", 3.0, Some((1.0, 1000.0)), Some(&[GRID])),
        number("radius", "Radius", "Shape", 200.0, Some((0.0, f64::MAX)), Some(&[CIRCLE])),
        number("start_angle", "Start", "Shape", 0.0, None, Some(&[CIRCLE])),
        number("sweep", "Sweep", "Shape", 360.0, Some((0.0, 360.0)), Some(&[CIRCLE])),
        vec2("position_each", "Position", "Each", [100.0, 0.0], None),
        number("rotation_each", "Rotation", "Each", 0.0, None, None),
        number("scale_each", "Scale", "Each", 0.0, Some((-100.0, 100.0)), None),
        number("opacity_each", "Opacity", "Each", 0.0, Some((-1.0, 1.0)), None),
        number("delay_each", "Delay", "Each", 0.0, None, None),
        vec2("position_random", "Position", "Random", [0.0, 0.0], None),
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
        row("Rotation", "°", Some("rotation_each"), Some("rotation_random"), None),
        row("Scale", "%", Some("scale_each"), Some("scale_random"), None),
        row("Opacity", "", Some("opacity_each"), Some("opacity_random"), None),
        advanced("Delay", "s", Some("delay_each"), Some("delay_random")),
        advanced("Seed", "", None, Some("seed")),
    ],
}];

/// 形の行に出る欄の単位。格子の単位は `GridRow::unit`。
pub fn unit(name: &str) -> &'static str {
    match name {
        "radius" => "px",
        "start_angle" | "sweep" => "°",
        _ => "",
    }
}

pub fn kind(plugin_id: &str) -> Option<&'static PlacementKind> {
    KINDS.iter().find(|kind| kind.plugin_id == plugin_id)
}

/// 配置 1 つ。値は層の**親の空間**で、回転と大きさは層の位置を中心にする。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub index: u32,
    pub offset: [f32; 2],
    pub rotation_degrees: f32,
    pub scale: f32,
    pub opacity: f32,
    /// 正なら遅れて出る(この配置は `t - time_offset` の姿)。
    pub time_offset: RationalTime,
}

impl Placement {
    pub fn affine2(&self, pivot: glam::Vec2) -> glam::Affine2 {
        use glam::{Affine2, Vec2};
        Affine2::from_translation(Vec2::from(self.offset) + pivot)
            * Affine2::from_angle(self.rotation_degrees.to_radians())
            * Affine2::from_scale(Vec2::splat(self.scale))
            * Affine2::from_translation(-pivot)
    }

    pub fn affine3(&self, pivot: glam::Vec3) -> glam::Affine3A {
        use glam::{Affine3A, Quat, Vec3};
        Affine3A::from_translation(Vec3::new(self.offset[0], self.offset[1], 0.0) + pivot)
            * Affine3A::from_quat(Quat::from_rotation_z(self.rotation_degrees.to_radians()))
            * Affine3A::from_scale(Vec3::new(self.scale, self.scale, 1.0))
            * Affine3A::from_translation(-pivot)
    }
}

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
    let get2 = |name: &str| -> [f64; 2] {
        match value(name) {
            Some(Value::Vec2(v)) if v.iter().all(|c| c.is_finite()) => v,
            _ => param(name).map_or([0.0, 0.0], |p| p.default),
        }
    };
    let count = get("count").round().clamp(1.0, 1000.0) as u32;
    let mode = get("mode").round().clamp(0.0, (SHAPES.len() - 1) as f64) as u8;
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
                opacity: (1.0 + get("opacity_each") * i + get("opacity_random") * u(4)).clamp(0.0, 1.0) as f32,
                time_offset: RationalTime::try_new((seconds * TIME_DEN as f64).round() as i64, TIME_DEN)
                    .unwrap_or(RationalTime::ZERO),
            }
        })
        .collect()
}

/// グループ丸ごと増やすか(true)、配置ごとに子を 1 つ引くか(false)。
pub fn whole_group(params: &[(String, Value)]) -> bool {
    params.iter().any(|(n, v)| n == "subject" && matches!(v, Value::F64(v) if v.round() as u8 == SUBJECT_WHOLE))
}

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
fn noise(seed: u64, index: u32, channel: u64) -> f64 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(u64::from(index).wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .wrapping_add(channel.wrapping_mul(0x94D0_49BB_1331_11EB))
        .wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
}

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
