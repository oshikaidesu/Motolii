//! 棚の 1 枚の宣言 — shader を持たない効果(配置)の契約。shader を持つ効果(pass・surface・field)は
//! `vism/*.wgsl` の manifest が同じ欄を宣言し、render の catalog が両方を 1 つの表にする(最小コア A、2026-09-07)。

use crate::doc::eval::Value;

pub enum ParamKind {
    Number,
    Vec2,
    /// 形のトグル。値は選択肢の番号(F64 で持つ)。
    Choice(&'static [&'static str]),
    /// 別の層を指す。値は LayerId(0 = 無し)。窓はカメラの target と同じ選択肢で描く。
    Layer,
    /// 色(非乗算 RGBA 0..1)。既定の色をここに持つ。
    Color([f64; 4]),
}

pub struct Param {
    pub name: &'static str,
    /// 窓に出る英語。
    pub label: &'static str,
    /// 欄の組。無ければ空文字。
    pub section: &'static str,
    pub kind: ParamKind,
    pub default: [f64; 2],
    pub range: Option<(f64, f64)>,
    /// この形の時だけ出る(配置効果の形のトグル)。None は全形。
    pub modes: Option<&'static [u8]>,
}

impl Param {
    pub const fn number(name: &'static str, label: &'static str, default: f64, range: Option<(f64, f64)>) -> Self {
        Self { name, label, section: "", kind: ParamKind::Number, default: [default, 0.0], range, modes: None }
    }
    pub const fn choice(name: &'static str, label: &'static str, choices: &'static [&'static str]) -> Self {
        Self { name, label, section: "", kind: ParamKind::Choice(choices), default: [0.0, 0.0], range: Some((0.0, (choices.len() - 1) as f64)), modes: None }
    }

    pub const fn layer(name: &'static str, label: &'static str) -> Self {
        Self { name, label, section: "", kind: ParamKind::Layer, default: [0.0, 0.0], range: None, modes: None }
    }

    pub fn shown(&self, mode: u8) -> bool {
        self.modes.is_none_or(|modes| modes.contains(&mode))
    }

    pub fn default_value(&self) -> Value {
        match self.kind {
            ParamKind::Vec2 => Value::Vec2(self.default),
            ParamKind::Layer => Value::LayerId(0),
            ParamKind::Color(c) => Value::Color(c),
            _ => Value::F64(self.default[0]),
        }
    }

    pub fn choices(&self) -> Option<&'static [&'static str]> {
        match self.kind {
            ParamKind::Choice(choices) => Some(choices),
            _ => None,
        }
    }
}

/// shader を持たない棚の 1 枚が何を返すか。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// 配置の集合。
    Placement,
    /// 形の層の輪郭。
    Path,
    /// 平らな素材から立体を起こす(押し出し・縁の丸み)。
    Solid,
    /// 文字の層の輪郭(文字を形にする段で効く)。
    Text,
    /// 出力(画面座標、AE の調整層): 下の合成を読んで、その上に描く(2026-09-13 効果の法の 4 札)。
    Output,
}

/// 棚に並ぶ 1 枚の見え方。
#[derive(Clone, Copy)]
pub struct Kind {
    pub plugin_id: &'static str,
    pub label: &'static str,
    pub params: &'static [Param],
    pub family: Family,
}

/// A stateless placement evaluator. It receives values, never editing authority.
#[derive(Clone, Copy)]
pub struct PlacementProgram {
    pub plugin_id: &'static str,
    pub needs_position: bool,
    pub evaluate: fn(&PlacementInput<'_>) -> Vec<PlacementOutput>,
}

pub struct PlacementInput<'a> {
    pub params: &'a [(String, Value)],
    pub layer: super::LayerId,
    pub time: RationalTime,
    pub position: [f32; 2],
    pub stretch_outline: bool,
    pub analysis: Option<&'a super::analysis::AnalysisInputs>,
}

pub struct PlacementOutput {
    pub placement: Placement,
    pub outline_stretch: [f32; 2],
}

impl From<Placement> for PlacementOutput {
    fn from(placement: Placement) -> Self {
        Self { placement, outline_stretch: [1.0; 2] }
    }
}

use crate::doc::core::RationalTime;

/// 配置 1 つ。値は層の**親の空間**で、回転と大きさは層の位置を中心にする。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub index: u32,
    pub offset: [f32; 2],
    pub rotation_degrees: f32,
    pub scale: f32,
    /// 奥行きのずれ。2.5D/3D の写しだけが受ける(2D の transform には無い)。
    pub offset_z: f32,
    pub opacity: f32,
    /// 正なら遅れて出る(この配置は `t - time_offset` の姿)。
    pub time_offset: RationalTime,
    /// 縦横の伸び(Blob Track が素材を箱に合わせる)。Repeater は [1, 1]。
    pub stretch: [f32; 2],
}

impl Placement {
    pub fn affine2(&self, pivot: glam::Vec2) -> glam::Affine2 {
        use glam::{Affine2, Vec2};
        Affine2::from_translation(Vec2::from(self.offset) + pivot)
            * Affine2::from_angle(self.rotation_degrees.to_radians())
            * Affine2::from_scale(Vec2::new(self.scale * self.stretch[0], self.scale * self.stretch[1]))
            * Affine2::from_translation(-pivot)
    }

    pub fn affine3(&self, pivot: glam::Vec3) -> glam::Affine3A {
        use glam::{Affine3A, Quat, Vec3};
        Affine3A::from_translation(Vec3::new(self.offset[0], self.offset[1], self.offset_z) + pivot)
            * Affine3A::from_quat(Quat::from_rotation_z(self.rotation_degrees.to_radians()))
            * Affine3A::from_scale(Vec3::new(self.scale * self.stretch[0], self.scale * self.stretch[1], 1.0))
            * Affine3A::from_translation(-pivot)
    }
}
