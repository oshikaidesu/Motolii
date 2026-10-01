//! 解けた層 — その時刻に何をどう描くかの一覧。書類には無い、絵の側の答え。
//! (コアはこれを作りも読みもしない。定義だけ持っていたのを、使う家へ移した)

use crate::doc::core::{LayerPlacement, RationalTime};
use crate::doc::store::{
    BlendMode, EffectScope, LayerId, LayerProjection, LayerSource, MaskFrame,
    MaskMode, Matte,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedLayer {
    pub id: LayerId,
    pub source: LayerSource,
    pub placement: LayerPlacement,
    pub declared_size: [f32; 2],
    pub source_frame: i64,
    pub source_time: RationalTime,
    pub masks: Vec<ResolvedMask>,
    pub effects: Vec<ResolvedEffect>,
    pub blend_mode: BlendMode,
    pub matte: Option<Matte>,
    pub clip_to_below: bool,
    pub projection: LayerProjection,
    /// 3D の素材を平面へ収めるか。既定は収めない(裁定 2026-08-30)。
    pub flatten: bool,
    pub environment: bool,
    /// 押し出しの奥行き(px、素の値)。0 なら板。
    pub depth: f32,
    /// ゴースト(同じ層を遅れて見た姿)なら true。掴めない・枠に入らない(裁定 2026-09-07)。
    pub ghost: bool,
    /// 配置効果が増やした何番目か。増やしていなければ 0。
    pub copy: u32,
    /// 配置効果より**下**に積まれた効果、または板(`plate`)に掛かる効果。1 枚に合わせてから掛かる。
    pub after_effects: Vec<ResolvedEffect>,
    /// このグループの板の一部。Whole の効果を積んだグループの子孫は、同じ板の物を 1 枚に焼いてから
    /// `after_effects` を掛け、板の不透明度と混ぜ方はそのグループの物(裁定 2026-09-11)。
    pub plate: Option<LayerId>,
    /// Motion Blur の写しなら、足して平均する枚数(各写しの不透明度は 1/枚数)。0 なら普通に重ねる。
    pub averaged: u32,
    /// 形の輪郭を伸ばす倍率(Blob Track が形の素材を箱へ合わせる)。線の太さは伸ばさない。[1, 1] は素のまま。
    pub shape_stretch: [f32; 2],
    /// 文字の字ごとのずれ(組んだ順の字、素材座標)。折り返しが変わった時、字が前の場所から移る(間合いの法 4 の Transition)。
    pub glyph_offsets: Option<std::sync::Arc<Vec<[f32; 2]>>>,
    /// 折り返す文字が避ける物(CSS `shape-outside`)、文字の枠の座標。
    pub flow_around: Option<std::sync::Arc<Vec<crate::picture::text_frame::Obstacle>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedMask {
    pub mode: MaskMode,
    pub inverted: bool,
    pub opacity: f32,
    pub expansion: f64,
    pub shape: crate::doc::eval::Path,
    pub frame: MaskFrame,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedEffect {
    pub plugin_id: String,
    pub params: Vec<(String, crate::doc::store::Value)>,
    pub scope: EffectScope,
}
