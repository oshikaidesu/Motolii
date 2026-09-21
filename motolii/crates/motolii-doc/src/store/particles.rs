//! 粒子の層 — 効果ではなく形(2026-09-13 効果の法: PointCloud の billboard)。取っ手は AE の Furikake を参考にした
//! (出す元の大きさ・率・寿命・向き・速さ・寿命に沿う大きさ / 不透明度 / 色・重力・風・跳ね返り・乱流・種)。
//! 動きは閉じた式(simulation-model.md §8 の L0: `(seed, 番号, 生まれた時刻, t)`)。状態を持たないので、
//! 飛んでも辿っても同じ絵。率だけは入点からコマごとに積む(率のキーで数が増減する)。

use crate::doc::core::{Fps, RationalTime};
use crate::doc::eval::Value;
use crate::doc::core::noise;
use crate::doc::store::{LayerId, PropertyId, StoreError, StoreView};

pub const RATE: &str = "particles.rate";
pub const LIFE: &str = "particles.life";
pub const LIFE_RANDOM: &str = "particles.life_random";
pub const EMITTER: &str = "particles.emitter";
pub const DIRECTION: &str = "particles.direction";
pub const SPREAD: &str = "particles.spread";
pub const SPEED: &str = "particles.speed";
pub const SPEED_RANDOM: &str = "particles.speed_random";
pub const SIZE: &str = "particles.size";
pub const SIZE_END: &str = "particles.size_end";
pub const OPACITY_END: &str = "particles.opacity_end";
pub const COLOR: &str = "particles.color";
pub const COLOR_END: &str = "particles.color_end";
pub const GRAVITY: &str = "particles.gravity";
pub const WIND: &str = "particles.wind";
pub const BOUNCE: &str = "particles.bounce";
pub const FLOOR: &str = "particles.floor";
pub const TURBULENCE: &str = "particles.turbulence";
pub const TURBULENCE_SIZE: &str = "particles.turbulence_size";
pub const SEED: &str = "particles.seed";
pub const CONNECT: &str = "particles.connect";
pub const LINE_WIDTH: &str = "particles.line_width";
pub const LINE_OPACITY: &str = "particles.line_opacity";

/// 粒子の層の欄: (property, label, 既定値, 範囲)。登録・既定・Inspector はこの 1 表から(Camera の表と同じ型)。
/// 長さは comp の px、時間は秒、角度は度(0 = 右、-90 = 上)。
pub const ROWS: &[(&str, &str, Value, Option<(f64, f64)>)] = &[
    (RATE, "Rate", Value::F64(40.0), Some((0.0, 10000.0))),
    (LIFE, "Life", Value::F64(2.0), Some((0.01, 600.0))),
    (LIFE_RANDOM, "Life Random", Value::F64(0.3), Some((0.0, 1.0))),
    (EMITTER, "Emitter Size", Value::Vec2([0.0, 0.0]), None),
    (DIRECTION, "Direction", Value::F64(-90.0), None),
    (SPREAD, "Spread", Value::F64(30.0), Some((0.0, 360.0))),
    (SPEED, "Speed", Value::F64(240.0), None),
    (SPEED_RANDOM, "Speed Random", Value::F64(0.3), Some((0.0, 1.0))),
    (SIZE, "Size", Value::F64(8.0), Some((0.0, 10000.0))),
    (SIZE_END, "Size at Death", Value::F64(2.0), Some((0.0, 10000.0))),
    (OPACITY_END, "Opacity at Death", Value::F64(0.0), Some((0.0, 1.0))),
    (COLOR, "Color", Value::Color([1.0, 1.0, 1.0, 1.0]), None),
    (COLOR_END, "Color at Death", Value::Color([1.0, 1.0, 1.0, 1.0]), None),
    (GRAVITY, "Gravity", Value::F64(0.0), None),
    (WIND, "Wind", Value::Vec2([0.0, 0.0]), None),
    (BOUNCE, "Bounce", Value::F64(0.0), Some((0.0, 1.0))),
    (FLOOR, "Floor", Value::F64(300.0), None),
    (TURBULENCE, "Turbulence", Value::F64(0.0), Some((0.0, 10000.0))),
    (TURBULENCE_SIZE, "Turbulence Size", Value::F64(120.0), Some((1.0, 100000.0))),
    (SEED, "Seed", Value::F64(0.0), Some((0.0, 9999.0))),
    // Plexus: この距離より近い粒どうしを線で結ぶ(0 は結ばない)。線は近いほど濃い。
    (CONNECT, "Connect Distance", Value::F64(0.0), Some((0.0, 10000.0))),
    (LINE_WIDTH, "Line Width", Value::F64(1.0), Some((0.0, 100.0))),
    (LINE_OPACITY, "Line Opacity", Value::F64(0.6), Some((0.0, 1.0))),
];

/// 一度に生きていられる粒の上限(率 × 寿命がこれを超えたら、古い物から描かない)。
pub const MAX_ALIVE: usize = 100_000;

pub fn default_of(name: &str) -> Option<Value> {
    ROWS.iter().find(|row| row.0 == name).map(|row| row.2.clone())
}

/// 1 粒。位置は層の局所(出す元の中心が 0)、乱流は描く側が年齢で足す(`age`・`index`)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub index: u32,
    pub position: [f32; 3],
    /// 直径(px)。
    pub size: f32,
    /// 非乗算の色(0〜1)。
    pub color: [f32; 4],
    pub age: f32,
}

/// 乱流の取っ手(描く側が fbm で位置に足す)。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turbulence {
    pub amount: f32,
    pub size: f32,
    pub seed: f32,
}

/// Plexus の取っ手(描く側が近い粒どうしを線で結ぶ)。`distance` 0 は結ばない。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Links {
    pub distance: f32,
    pub width: f32,
    pub opacity: f32,
}

struct Params {
    life: f64,
    life_random: f64,
    emitter: [f64; 2],
    direction: f64,
    spread: f64,
    speed: f64,
    speed_random: f64,
    size: f64,
    size_end: f64,
    opacity_end: f64,
    color: [f64; 4],
    color_end: [f64; 4],
    gravity: f64,
    wind: [f64; 2],
    bounce: f64,
    floor: f64,
    seed: u64,
}

pub fn particle_value(view: &StoreView<'_>, layer: LayerId, name: &str, t: RationalTime) -> Result<Value, StoreError> {
    Ok(view.value_at(layer, &PropertyId::new(name)?, t)?.or_else(|| default_of(name)).unwrap_or(Value::F64(0.0)))
}

pub fn particle_number(view: &StoreView<'_>, layer: LayerId, name: &str, t: RationalTime) -> Result<f64, StoreError> {
    Ok(match particle_value(view, layer, name, t)? { Value::F64(v) if v.is_finite() => v, _ => 0.0 })
}

/// 時刻 t に生きている粒と、乱流・Plexus の取っ手。入点より前は空。
pub fn particles_at(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<(Vec<Particle>, Turbulence, Links), StoreError> {
    let empty = || (Vec::new(), Turbulence { amount: 0.0, size: 120.0, seed: 0.0 }, Links { distance: 0.0, width: 1.0, opacity: 0.6 });
    let Some(fps) = view.composition()?.map(|c| c.fps) else { return Ok(empty()) };
    let Some(meta) = view.meta(layer)? else { return Ok(empty()) };
    let mut values = std::collections::BTreeMap::new();
    for &(name, _, _, _) in ROWS {
        values.insert(name.to_owned(), particle_value(view, layer, name, t)?);
    }
    let births = births(view, layer, meta.timing.start, t, fps)?;
    Ok(particles_from_values_and_births(&values, &births, t.as_seconds_f64()))
}

pub fn particles_from_values_and_births(
    values: &std::collections::BTreeMap<String, Value>,
    births: &[(u32, f64)],
    now: f64,
) -> (Vec<Particle>, Turbulence, Links) {
    let value = |name: &str| values.get(name).cloned().or_else(|| default_of(name)).unwrap_or(Value::F64(0.0));
    let number = |name: &str| match value(name) { Value::F64(v) if v.is_finite() => v, _ => 0.0 };
    let vec2 = |name: &str| match value(name) { Value::Vec2(v) => v, _ => [0.0, 0.0] };
    let color = |name: &str| match value(name) { Value::Color(v) => v, _ => [1.0; 4] };
    let turbulence = Turbulence {
        amount: number(TURBULENCE) as f32,
        size: number(TURBULENCE_SIZE).max(1.0) as f32,
        seed: number(SEED) as f32,
    };
    let links = Links {
        distance: number(CONNECT).max(0.0) as f32,
        width: number(LINE_WIDTH).max(0.0) as f32,
        opacity: number(LINE_OPACITY).clamp(0.0, 1.0) as f32,
    };
    let p = Params {
        life: number(LIFE).max(0.01),
        life_random: number(LIFE_RANDOM).clamp(0.0, 1.0),
        emitter: vec2(EMITTER),
        direction: number(DIRECTION),
        spread: number(SPREAD).clamp(0.0, 360.0),
        speed: number(SPEED),
        speed_random: number(SPEED_RANDOM).clamp(0.0, 1.0),
        size: number(SIZE).max(0.0),
        size_end: number(SIZE_END).max(0.0),
        opacity_end: number(OPACITY_END).clamp(0.0, 1.0),
        color: color(COLOR),
        color_end: color(COLOR_END),
        gravity: number(GRAVITY),
        wind: vec2(WIND),
        bounce: number(BOUNCE).clamp(0.0, 1.0),
        floor: number(FLOOR),
        seed: number(SEED).round().max(0.0) as u64,
    };
    let mut out = Vec::new();
    for &(index, born) in births.iter().rev() {
        let age = now - born;
        if age > p.life { break; }
        if out.len() >= MAX_ALIVE { break; }
        if let Some(particle) = p.particle(index, age) {
            out.push(particle);
        }
    }
    out.reverse();
    (out, turbulence, links)
}

/// 入点から t までに生まれた粒: (番号, 生まれた comp の秒)。率はコマごとにその時刻の値で積む。
pub fn births(view: &StoreView<'_>, layer: LayerId, start_frame: i64, t: RationalTime, fps: Fps) -> Result<Vec<(u32, f64)>, StoreError> {
    let frame_seconds = fps.den() as f64 / fps.num() as f64;
    let Ok(now_frame) = t.try_to_frame_floor(fps) else { return Ok(Vec::new()) };
    let now = t.as_seconds_f64();
    let mut births = Vec::new();
    let mut accumulated = 0.0f64;
    let mut index = 0u32;
    for frame in start_frame..=now_frame {
        let Ok(at) = RationalTime::try_from_frame(frame, fps) else { continue };
        let rate = particle_number(view, layer, RATE, at)?.max(0.0);
        let before = accumulated;
        accumulated += rate * frame_seconds;
        let count = accumulated.floor() as u64 - before.floor() as u64;
        for k in 0..count {
            // コマの中で等間隔に生まれる(同じコマの粒が 1 点に固まらない)。
            let born = at.as_seconds_f64() + frame_seconds * (k as f64 + 0.5) / count as f64;
            if born <= now {
                births.push((index, born));
            }
            index = index.wrapping_add(1);
        }
    }
    Ok(births)
}

impl Params {
    fn particle(&self, index: u32, age: f64) -> Option<Particle> {
        let u = |channel: u64| noise(self.seed, index, channel);
        let life = self.life * (1.0 - self.life_random * (u(0) * 0.5 + 0.5));
        if age < 0.0 || age >= life {
            return None;
        }
        let angle = (self.direction + self.spread * 0.5 * u(1)).to_radians();
        let speed = self.speed * (1.0 + self.speed_random * u(2));
        let start = [self.emitter[0] * 0.5 * u(3), self.emitter[1] * 0.5 * u(4)];
        let velocity = [angle.cos() * speed, angle.sin() * speed];
        let accel = [self.wind[0], self.wind[1] + self.gravity];
        let [x, y] = ballistic(start, velocity, accel, age, self.bounce, self.floor);
        let life_share = age / life;
        let lerp = |a: f64, b: f64| a + (b - a) * life_share;
        let color = std::array::from_fn(|i| lerp(self.color[i], self.color_end[i]) as f32);
        let mut color: [f32; 4] = color;
        color[3] *= lerp(1.0, self.opacity_end) as f32;
        Some(Particle { index, position: [x as f32, y as f32, 0.0], size: lerp(self.size, self.size_end) as f32, color, age: age as f32 })
    }
}

/// 等加速度の放物線。`bounce > 0` なら床(y = floor、下が +)で縦の速さを `bounce` 倍にして跳ね返る。
/// 跳ねるたびに次の着地を解くので、状態を持たずに年齢だけで位置が決まる。
fn ballistic(start: [f64; 2], velocity: [f64; 2], accel: [f64; 2], age: f64, bounce: f64, floor: f64) -> [f64; 2] {
    let at = |p: [f64; 2], v: [f64; 2], dt: f64| [p[0] + v[0] * dt + 0.5 * accel[0] * dt * dt, p[1] + v[1] * dt + 0.5 * accel[1] * dt * dt];
    if bounce <= 0.0 || start[1] > floor {
        return at(start, velocity, age);
    }
    let (mut p, mut v, mut left) = (start, velocity, age);
    for _ in 0..32 {
        // y(τ) = floor の正の解(下向きに着く方)。
        let (a, b, c) = (0.5 * accel[1], v[1], p[1] - floor);
        let hit = if a.abs() < 1e-9 {
            if b > 1e-9 { -c / b } else { f64::INFINITY }
        } else {
            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 { f64::INFINITY } else {
                let r = disc.sqrt();
                [(-b - r) / (2.0 * a), (-b + r) / (2.0 * a)].into_iter().filter(|&x| x > 1e-6).fold(f64::INFINITY, f64::min)
            }
        };
        if hit >= left {
            return at(p, v, left);
        }
        p = at(p, v, hit);
        p[1] = floor;
        v = [v[0] + accel[0] * hit, -(v[1] + accel[1] * hit) * bounce];
        left -= hit;
        if v[1].abs() < 1.0 {
            // 跳ねが尽きたら床を滑る。
            return [p[0] + v[0] * left + 0.5 * accel[0] * left * left, floor];
        }
    }
    [p[0], floor]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ball_bounces_back_up_off_the_floor() {
        // 真下へ落ちる: 床 100、重力 1000。着地 ≈ 0.447 秒、跳ね返り 0.5 で上へ戻る。
        let before = ballistic([0.0, 0.0], [0.0, 0.0], [0.0, 1000.0], 0.4, 0.5, 100.0);
        let after = ballistic([0.0, 0.0], [0.0, 0.0], [0.0, 1000.0], 0.5, 0.5, 100.0);
        assert!(before[1] < 100.0 && before[1] > 70.0, "{before:?}");
        assert!(after[1] < 100.0, "床を突き抜けない: {after:?}");
        assert!(ballistic([0.0, 0.0], [0.0, 0.0], [0.0, 1000.0], 5.0, 0.5, 100.0)[1] <= 100.0 + 1e-6, "何度跳ねても床の上");
        assert!(ballistic([0.0, 0.0], [0.0, 0.0], [0.0, 1000.0], 0.5, 0.0, 100.0)[1] > 100.0, "跳ね返り 0 は床が無い");
    }
}
