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
    /// グループの子が素材の袋になる時、`count` 個の写しがそれぞれどの子を引くか(子の並びの番号)。
    /// 引き方は効果の方針(順に・重みで・種から)。子が無ければ空。
    pub pick: fn(&[(String, Value)], &[u64], usize) -> Vec<usize>,
    /// この取っ手ではグループ全体が 1 つの物として動くか(子を 1 枚ずつ動かすのではなく)。
    pub moves_whole: fn(&[(String, Value)]) -> bool,
}

/// この書類で使える効果の一式。コアはこの 3 つの口しか知らず、中身が誰かは知らない。
/// 拡張を足すとは、この表に 1 行足すこと。
#[derive(Clone, Copy)]
pub struct Programs {
    pub placement: fn(&str) -> Option<PlacementProgram>,
    pub sampling: fn(&str) -> Option<SamplingProgram>,
    pub snap: fn(&str) -> Option<SnapProgram>,
}

impl Programs {
    /// 効果が 1 つも登録されていない書類。読むだけ・並べるだけなら足りる。
    pub const NONE: Self = Self { placement: |_| None, sampling: |_| None, snap: |_| None };
}

/// 「見えている所」を答える口。作品を編集する時、見た目を保つために必要になる
/// (束ねを解く・親を移す・札を変える)。解くのは絵の側で、コアは答えだけ受け取る。
#[derive(Clone, Copy)]
pub struct Geometry {
    /// 層の、親の空間での変換。
    pub local: fn(&super::StoreView<'_>, super::LayerId, RationalTime) -> Result<glam::Affine2, super::StoreError>,
    /// 層の、親の空間での変換(奥行き込み)。
    pub local3d: fn(&super::StoreView<'_>, super::LayerId, RationalTime) -> Result<glam::Affine3A, super::StoreError>,
    /// 層の、世界の変換(奥行き込み)。
    pub world: fn(&super::StoreView<'_>, super::LayerId, RationalTime) -> Result<glam::Affine3A, super::StoreError>,
    /// その時刻の全層の世界の変換。
    pub worlds: fn(&super::StoreView<'_>, RationalTime) -> Result<std::collections::HashMap<super::LayerId, glam::Affine3A>, super::StoreError>,
    /// その時刻に効いている観測の姿勢。
    pub camera: fn(&super::StoreView<'_>, RationalTime) -> Result<crate::doc::core::ResolvedCamera, super::StoreError>,
}

impl Geometry {
    /// 誰も答えない書類。見た目の補正は起きず、書いた値がそのまま残る。
    pub const NONE: Self = Self {
        local: |_, _, _| Ok(glam::Affine2::IDENTITY),
        local3d: |_, _, _| Ok(glam::Affine3A::IDENTITY),
        world: |_, _, _| Ok(glam::Affine3A::IDENTITY),
        worlds: |_, _| Ok(std::collections::HashMap::new()),
        camera: |_, _| Ok(crate::doc::core::ResolvedCamera::default()),
    };
}

/// 子を引かない効果の既定: `k` 番目の写しは `k` 番目の子。
pub fn pick_in_turn(_params: &[(String, Value)], children: &[u64], count: usize) -> Vec<usize> {
    if children.is_empty() { return Vec::new() }
    (0..count).map(|index| index % children.len()).collect()
}

/// 全体を動かす取っ手を持たない効果の既定。
pub fn never_moves_whole(_params: &[(String, Value)]) -> bool {
    false
}

/// 1 コマの中で層を取り直す効果(Motion Blur など)が返す取っ手。
/// どの欄を・どれだけの幅で・何枚に分けるかは効果の方針で、コアは道のりを測って枚数を訊き、
/// 言われた時刻の変換を取り直して平均の印を付けるだけ。
pub struct Shutter {
    /// 取り直す欄(位置・大きさ・角度)。
    pub channels: [bool; 3],
    /// コマ何個ぶんの幅で開くか。
    pub width_frames: f64,
    /// 層の縁が 1 コマで動いた道のり(px)から、写しの枚数。1 ならぼかさない。
    pub count: fn(f32) -> u32,
    /// `k` 枚目が、コマの中心からどれだけずれた所か(幅を 1 とした比、-0.5 から 0.5)。
    pub offset: fn(u32, u32) -> f64,
}

impl Shutter {
    /// コマの中心から幅の両端まで層がどれだけ動いたかを測り、効果に枚数を訊き、
    /// 各写しの時刻の変換を返す。動きの測り方と時刻の刻みはコアの仕事、枚数と刻み幅は効果の方針。
    pub fn deltas<E>(
        &self,
        time: RationalTime,
        frame_seconds: f64,
        size: [f32; 2],
        transform: glam::Affine2,
        parent: glam::Affine2,
        mut read_delta: impl FnMut(RationalTime) -> Result<glam::Affine2, E>,
    ) -> Result<Vec<Result<glam::Affine2, E>>, E> {
        const DEN: i64 = 1_000_000;
        let at = |share: f64| {
            RationalTime::try_new((share * self.width_frames * frame_seconds * DEN as f64).round() as i64, DEN)
                .ok()
                .and_then(|offset| time.try_add(offset).ok())
        };
        let (Some(first), Some(last)) = (at(-0.5), at(0.5)) else { return Ok(Vec::new()) };
        let inverse = parent.inverse();
        let early = parent * read_delta(first)? * inverse;
        let late = parent * read_delta(last)? * inverse;
        // 未確定の素材寸法は原点の周り 200px で回転の道のりを見積もる。
        let [w, h] = size;
        let (lo, hi) = if w > 0.0 && h > 0.0 { ([0.0, 0.0], [w, h]) } else { ([-100.0, -100.0], [100.0, 100.0]) };
        let travel = [[lo[0], lo[1]], [hi[0], lo[1]], [lo[0], hi[1]], [hi[0], hi[1]]]
            .map(|corner| {
                let p = transform.transform_point2(glam::Vec2::from(corner));
                early.transform_point2(p).distance(late.transform_point2(p))
            })
            .into_iter()
            .fold(0.0f32, f32::max);
        let count = (self.count)(travel);
        if count <= 1 {
            return Ok(Vec::new());
        }
        Ok((0..count).filter_map(|k| at((self.offset)(k, count))).map(read_delta).collect())
    }
}

/// 拾った物の箱の辺を格子の線へまとめ、そこへ寄せる効果が返す取っ手。
/// どの辺が同じ線に乗るかは効果の方針で、コアは箱を集めて渡し、返った線へ動かすだけ。
pub struct Snapping {
    /// 寄せる強さ(0 なら寄せない)。
    pub strength: f32,
    /// 線をまとめる度合い。効果の取っ手の値で、コアは意味を知らない。
    pub merge: f32,
    /// 辺の座標の列を、同じ長さの「その辺が乗る線」の列へ。
    pub lines: fn(&[f32], f32) -> Vec<f32>,
}

/// 取っ手の値から Snapping を読む効果。
pub struct SnapProgram {
    pub plugin_id: &'static str,
    pub snapping: fn(&str, &[(String, Value)]) -> Option<Snapping>,
}

/// 取っ手の値から Shutter を読む効果。`plugin_id` が合う効果に 1 つ。
pub struct SamplingProgram {
    pub plugin_id: &'static str,
    pub shutter: fn(&[(String, Value)]) -> Option<Shutter>,
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
