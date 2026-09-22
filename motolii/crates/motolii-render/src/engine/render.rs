use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::HashMap;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{
    LayerId, LayerSource, RationalTime, ShapeNode, StoreView, TextDocument,
};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses, Window,
};

use crate::render::engine::translate::{
    translate_blend_mode, translate_cast_shadow, translate_clip, translate_effect_passes, translate_matte_mode, translate_point_displace,
};
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

