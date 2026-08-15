// Ported from `egui_tiles` (https://github.com/rerun-io/egui_tiles) by rerun.io / Emil Ernerfeldt.
// Licensed under MIT OR Apache-2.0. Source: upstream commit fddb9fd (0.17.0 pre-release).
// Geometry helpers copied from `emath` (egui workspace, MIT OR Apache-2.0).
//
// なぜ: 移植元は `egui::{Pos2, Vec2, Rect, Rangef}` を使うが `dock/` は egui へ依存しない。
// C4 capsule は `kurbo` への置換を指すが、`kurbo` は `motolii-ui` の直接依存ではなく
// Cargo.toml は ALLOWLIST 外のため、ここでは同じ意味・同じメソッド名の最小型を置き、
// 移植した本体のコードを一字も変えずに済むようにしている。kurbo が依存に入ったら
// このファイルだけを差し替えれば足りる。

#![allow(dead_code)] // なぜ: 移植元のAPI面をそのまま保つため、未使用のヘルパも落とさない

/// A position in 2D space. Mirrors `egui::Pos2`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pos2 {
    pub x: f32,
    pub y: f32,
}

/// A vector in 2D space. Mirrors `egui::Vec2`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub const fn pos2(x: f32, y: f32) -> Pos2 {
    Pos2 { x, y }
}

#[inline]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Pos2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        self.distance_sq(other).sqrt()
    }

    #[inline]
    pub fn distance_sq(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
        }
    }

    /// Round to the nearest physical pixel, the way `egui`'s `GuiRounding` does.
    #[inline]
    pub fn round_to_pixels(self, pixels_per_point: f32) -> Self {
        Self {
            x: round_to_pixels(self.x, pixels_per_point),
            y: round_to_pixels(self.y, pixels_per_point),
        }
    }
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }
}

impl std::ops::Add<Vec2> for Pos2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vec2) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Vec2> for Pos2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vec2) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Sub<Self> for Pos2 {
    type Output = Vec2;

    #[inline]
    fn sub(self, rhs: Self) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

#[inline]
pub fn round_to_pixels(point: f32, pixels_per_point: f32) -> f32 {
    (point * pixels_per_point).round() / pixels_per_point
}

/// Copied from `emath::exponential_smooth_factor`.
#[inline]
pub fn exponential_smooth_factor(
    reach_this_fraction: f32,
    in_this_many_seconds: f32,
    dt: f32,
) -> f32 {
    1.0 - (1.0 - reach_this_fraction).powf(dt / in_this_many_seconds)
}

// ----------------------------------------------------------------------------

/// An inclusive 1D range. Mirrors `egui::emath::Rangef`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rangef {
    pub min: f32,
    pub max: f32,
}

impl Rangef {
    #[inline]
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn span(self) -> f32 {
        self.max - self.min
    }

    #[inline]
    pub fn center(self) -> f32 {
        0.5 * (self.min + self.max)
    }
}

impl Default for Rangef {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

// ----------------------------------------------------------------------------

/// An axis-aligned rectangle. Mirrors `egui::Rect`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub min: Pos2,
    pub max: Pos2,
}

impl Default for Rect {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Rect {
    pub const ZERO: Self = Self {
        min: Pos2::ZERO,
        max: Pos2::ZERO,
    };

    #[inline]
    pub const fn from_min_max(min: Pos2, max: Pos2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_min_size(min: Pos2, size: Vec2) -> Self {
        Self {
            min,
            max: pos2(min.x + size.x, min.y + size.y),
        }
    }

    #[inline]
    pub fn from_center_size(center: Pos2, size: Vec2) -> Self {
        Self {
            min: pos2(center.x - 0.5 * size.x, center.y - 0.5 * size.y),
            max: pos2(center.x + 0.5 * size.x, center.y + 0.5 * size.y),
        }
    }

    #[inline]
    pub fn from_x_y_ranges(x_range: Rangef, y_range: Rangef) -> Self {
        Self {
            min: pos2(x_range.min, y_range.min),
            max: pos2(x_range.max, y_range.max),
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        vec2(self.width(), self.height())
    }

    #[inline]
    pub fn set_width(&mut self, w: f32) {
        self.max.x = self.min.x + w;
    }

    #[inline]
    pub fn set_height(&mut self, h: f32) {
        self.max.y = self.min.y + h;
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.min.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.max.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.min.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.max.y
    }

    #[inline]
    pub fn center(&self) -> Pos2 {
        pos2(
            0.5 * (self.min.x + self.max.x),
            0.5 * (self.min.y + self.max.y),
        )
    }

    #[inline]
    pub fn x_range(&self) -> Rangef {
        Rangef::new(self.min.x, self.max.x)
    }

    #[inline]
    pub fn y_range(&self) -> Rangef {
        Rangef::new(self.min.y, self.max.y)
    }

    #[inline]
    pub fn left_top(&self) -> Pos2 {
        self.min
    }

    #[inline]
    pub fn right_top(&self) -> Pos2 {
        pos2(self.max.x, self.min.y)
    }

    #[inline]
    pub fn left_bottom(&self) -> Pos2 {
        pos2(self.min.x, self.max.y)
    }

    #[inline]
    pub fn right_bottom(&self) -> Pos2 {
        self.max
    }

    #[inline]
    pub fn left_center(&self) -> Pos2 {
        pos2(self.min.x, self.center().y)
    }

    #[inline]
    pub fn right_center(&self) -> Pos2 {
        pos2(self.max.x, self.center().y)
    }

    #[inline]
    pub fn center_top(&self) -> Pos2 {
        pos2(self.center().x, self.min.y)
    }

    #[inline]
    pub fn center_bottom(&self) -> Pos2 {
        pos2(self.center().x, self.max.y)
    }

    #[inline]
    pub fn contains(&self, p: Pos2) -> bool {
        self.min.x <= p.x && p.x <= self.max.x && self.min.y <= p.y && p.y <= self.max.y
    }

    #[inline]
    pub fn lerp_towards(&self, other: &Self, t: f32) -> Self {
        Self {
            min: self.min.lerp(other.min, t),
            max: self.max.lerp(other.max, t),
        }
    }

    pub fn split_left_right_at_fraction(&self, t: f32) -> (Self, Self) {
        self.split_left_right_at_x(lerp(self.min.x, self.max.x, t))
    }

    pub fn split_left_right_at_x(&self, split_x: f32) -> (Self, Self) {
        let left = Self::from_min_max(self.min, pos2(split_x, self.max.y));
        let right = Self::from_min_max(pos2(split_x, self.min.y), self.max);
        (left, right)
    }

    pub fn split_top_bottom_at_fraction(&self, t: f32) -> (Self, Self) {
        self.split_top_bottom_at_y(lerp(self.min.y, self.max.y, t))
    }

    pub fn split_top_bottom_at_y(&self, split_y: f32) -> (Self, Self) {
        let top = Self::from_min_max(self.min, pos2(self.max.x, split_y));
        let bottom = Self::from_min_max(pos2(self.min.x, split_y), self.max);
        (top, bottom)
    }
}
