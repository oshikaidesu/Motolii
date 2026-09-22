#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ShapeNode, StoreView,
    TextDocument,
};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses, Window,
};

use crate::render::engine::translate::{
    translate_blend_mode, translate_cast_shadow, translate_clip, translate_effect_passes, translate_matte_mode, translate_point_displace,
};
use crate::render::compositor::effects::isf::TimeBase;
use crate::render::engine::{Engine, EngineError};

/// 先読みと、見えない層の捨て方。
mod warm;

impl Engine {
    pub fn render_with_camera_override(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        self.layer_failures.clear();
        self.purge_idle_video_players();
        self.render_frame_graph_pixels(view, t, include_background, camera_override)
    }

    /// 効果が宣言した時刻のずれごとに、**その時刻の層の絵**を用意する。
    ///
    /// 効果が自分で前フレームを覚えるのは恒久禁止(`docs/plugin-resources.md` §6) — 追跡できなくなり、
    /// 純関数契約・フレーム並列・スクラブが壊れるため。ここは逆で、ホストが時刻を決めて渡すので
    /// `render_frame(t)` は純関数のまま。
    ///
    /// 「時刻 t の層の姿」を作るのは Document の resolve 1 箇所だけ。ここでは引き直した姿を使う
    /// (`source_time` だけを手でずらすと、mask やキーフレームは t のままの継ぎ接ぎになる)。
    /// 別の時刻 `at` の**合成**を 1 枚に描く(下の合成 / 自分の群 / comp 全体)。自分は除く(非再帰)。

    pub(super) fn stamp_feedback(&mut self, passes: &mut [EffectPass], layer: LayerId, copy: u32, chain: u8, screen: Option<[u32; 2]>) {
        super::translate::stamp_feedback(passes, layer, copy, chain, screen, self.feedback_namespace);
        // 本番の鍵だけ辿り直しの対象(別の時刻の列は自分の列で進む)。
        if self.feedback_namespace == 0 {
            for key in passes.iter().filter_map(|p| p.feedback) { self.gpu_history.observe(key); }
        }
    }

    /// 層の組み立て + feedback の辿り直し。
    ///
    /// feedback は「入点を初期条件とする漸化式」(`docs/plugin-resources.md` §6-3)。組んだ後で、
    /// 初期条件から描かれてしまった状態(飛んで来た)があれば、直近の checkpoint(無ければ入点)から
    /// t の手前まで順に描いてから、t をもう一度組む。順再生と書き出しは 1 歩ずつなので何もしない。
    ///
    /// - 板の道(層の絵に焼く列)は**その層だけ**を辿り直す。


    pub fn render_frame_to_texture(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        self.render_frame_graph_to_texture_output(view, t, true)
    }

    pub fn render_frame_into(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
    ) -> Result<(), EngineError> {
        let comp = view.composition().map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?.spec();
        let camera = self.frame_graph_document_camera(view, t)?;
        self.render_frame_graph_into_window(
            view,
            t,
            target,
            camera,
            true,
            &[],
            Window::output(comp),
            crate::frame_graph::ViewProjection::Camera,
        )
    }

    /// Render from an observation camera while retaining authored layer projection.
    pub fn render_frame_into_with_camera(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
    ) -> Result<(), EngineError> {
        let comp = view.composition().map_err(|e| EngineError::Store(e.to_string()))?.ok_or(EngineError::NoComposition)?.spec();
        self.render_frame_into_window(view, t, target, camera, include_background, outline, Window::output(comp))
    }

    /// 同じ世界を、出力寸法以外の窓へ(Stage のタブ: 寸法と関心域は窓が言う)。
    pub fn render_frame_into_window(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
        window: Window,
    ) -> Result<(), EngineError> {
        self.render_frame_graph_into_window(
            view,
            t,
            target,
            camera,
            include_background,
            outline,
            window,
            crate::frame_graph::ViewProjection::Camera,
        )
    }
}

// These adapters support the explicit `ResolvedLayer` oracle APIs below. The
// production render entry points evaluate a FrameGraph scene before lowering.
pub(super) fn collect_text_documents(view: &StoreView<'_>, resolved: &[ResolvedLayer], t: RationalTime) -> Result<HashMap<LayerId, TextDocument>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Text {
            if let Some(document) = crate::picture::resolve::text::resolved_text_document(view, layer.id, t).map_err(|error| EngineError::Store(error.to_string()))? {
                documents.insert(layer.id, document);
            }
            let effects: Vec<_> = layer.effects.iter().chain(&layer.after_effects).cloned().collect();
            if let Some((target, _)) = crate::extensions::text::morph(&effects) {
                if !documents.contains_key(&target) {
                    if let Some(document) = crate::picture::resolve::text::resolved_text_document(view, target, t).map_err(|error| EngineError::Store(error.to_string()))? {
                        documents.insert(target, document);
                    }
                }
            }
        }
    }
    Ok(documents)
}

pub(super) fn collect_shape_documents(view: &StoreView<'_>, resolved: &[ResolvedLayer], t: RationalTime) -> Result<HashMap<LayerId, Vec<ShapeNode>>, EngineError> {
    let mut documents = HashMap::new();
    for layer in resolved {
        if layer.source == LayerSource::Shape {
            let shapes = crate::picture::shapes::shapes_at(view, layer.id, t).map_err(|error| EngineError::Store(error.to_string()))?;
            documents.insert(layer.id, shown_shapes(&shapes, layer));
        } else if layer.source == LayerSource::Group {
            if let Some(background) = crate::picture::boxes::background_shapes(view, layer.id, t).map_err(|error| EngineError::Store(error.to_string()))? {
                documents.insert(layer.id, background);
            }
        }
    }
    Ok(documents)
}

pub(crate) fn shown_shapes(shapes: &[ShapeNode], layer: &ResolvedLayer) -> Vec<ShapeNode> {
    let effects: Vec<_> = layer.effects.iter().chain(&layer.after_effects).cloned().collect();
    crate::extensions::pathop::with_effects(shapes, &effects)
}

pub(crate) fn layer_size(layer: &ResolvedLayer, natural: [f32; 2]) -> [f32; 2] {
    [
        if layer.declared_size[0] > 0.0 { layer.declared_size[0] } else { natural[0] },
        if layer.declared_size[1] > 0.0 { layer.declared_size[1] } else { natural[1] },
    ]
}

fn shifted_by_seconds(t: RationalTime, offset: f32) -> RationalTime {
    const DEN: i64 = 1000;
    let num = (offset as f64 * DEN as f64).round() as i64;
    let shifted = t.num().checked_mul(DEN)
        .and_then(|scaled| num.checked_mul(t.den()).map(|by| scaled + by))
        .zip(t.den().checked_mul(DEN))
        .and_then(|(num, den)| RationalTime::try_new(num, den).ok());
    match shifted {
        Some(at) if at.as_seconds_f64() >= 0.0 => at,
        _ => RationalTime::ZERO,
    }
}

/// 別の時刻ごとに引き直した「層の姿」。鍵はずれ(ミリ秒)。
type OtherTimes = BTreeMap<i64, (RationalTime, Vec<ResolvedLayer>, HashMap<LayerId, TextDocument>, HashMap<LayerId, Vec<ShapeNode>>)>;
/// 合体後の別時刻の写し: (時刻のずれ, 相手, 求めた層) → 絵。
type Composites = HashMap<(i64, crate::render::compositor::TimeSource, LayerId), crate::render::compositor::GpuTexture2D>;

/// 時刻のずれ(秒)を鍵にする — 同じずれは 1 回しか引かない。
/// 絶対時刻の鍵(ms)。復号の流れの名前空間に使う。
fn offset_key_of(at: RationalTime, _view: &StoreView<'_>) -> i64 { (at.as_seconds_f64() * 1000.0).round() as i64 }

fn offset_key(offset: f32) -> i64 {
    (offset as f64 * 1000.0).round() as i64
}

/// 別の時刻の鍵: ずれ(ms)か、層ごとの絶対時刻(入点からの ms に層の番号を混ぜ、上の bit で区別)。
fn time_key(offset: f32, base: TimeBase, layer: LayerId) -> i64 {
    match base {
        TimeBase::Offset => return offset_key(offset),
        // コマ数の鍵は秒の鍵と混ざらないよう上の bit で分ける(-1 コマと -0.001 秒は別の時刻)。
        TimeBase::Frames => return offset.round() as i64 | 1 << 60,
        TimeBase::At => {}
    }
    let mut hasher = std::hash::DefaultHasher::new();
    use std::hash::{Hash as _, Hasher as _};
    layer.0.hash(&mut hasher);
    offset_key(offset).hash(&mut hasher);
    (hasher.finish() >> 2) as i64 | 1 << 61
}

/// 層の入点(comp の時刻)。TIME_AT はここからの秒。
fn layer_in_point(view: &StoreView<'_>, layer: LayerId) -> RationalTime {
    let start = view.meta(layer).ok().flatten().map_or(0, |m| m.timing.start);
    view.composition().ok().flatten().and_then(|c| RationalTime::try_from_frame(start, c.fps).ok()).unwrap_or(RationalTime::ZERO)
}

/// 別の時刻を読むための層の番号。復号器の流れを本体と分けるためだけの物で、Document には無い。
fn lookbehind_layer_id(layer: LayerId, key: i64) -> LayerId {
    let mut hasher = std::hash::DefaultHasher::new();
    use std::hash::{Hash as _, Hasher as _};
    layer.0.hash(&mut hasher);
    key.hash(&mut hasher);
    LayerId(hasher.finish() | 1 << 63)
}

/// 別の時刻を読む効果は、**たどり着き方で絵が変わってはいけない**(実 GPU)。
///
/// 効果が自分で前フレームを覚える道を恒久禁止している理由がここ(`docs/plugin-resources.md` §6)。
/// ホストが時刻を渡す形なら、同じ時刻は何度描いても、どの順で描いても同じ絵になる。
#[cfg(test)]
mod time_reference_is_deterministic;

/// feedback(前のフレームを保つ効果)は**入点を初期条件とする漸化式**(`docs/plugin-resources.md` §6-3)。
/// 効果は覚えない — host が状態を持ち、飛んで来ても入点(か checkpoint)から辿り直すので、
/// 同じ時刻は何度描いても、どの順で描いても同じ絵(実 GPU)。
#[cfg(test)]
mod feedback_is_a_recurrence_from_the_in_point;

/// 合体後の別時刻(`SOURCE: below / comp`)。下の合成を t′ で読む効果は、ホストが t′ の下の層たちを
/// 描いて渡す。だから「下の合成の 0.5 秒前」は、下の層だけの書類を 0.5 秒前に描いた絵と同じで、
/// 飛んでも辿っても同じ(`docs/plugin-resources.md` §6-1 CompLookbehind、非再帰)。
#[cfg(test)]
mod composite_at_another_time;

/// Freeze(docs/freeze-and-flatten.md §2-6): 凍っても絵は変わらない、飛んでも辿っても同じ、Unfreeze で戻る。
#[cfg(test)]
mod freeze_keeps_the_picture;
