#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};

use std::collections::HashSet;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{LayerId, LayerSource, RationalTime, StoreView};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses, Window,
};

use crate::render::engine::{Engine, EngineError};

/// 1 コマ分の層を建てて焼く。
pub(super) mod build;
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
        self.ledger.clear();
        self.purge_idle_video_players();
        self.export_frame(view, t, include_background, camera_override)
    }

    /// Freeze の 1 コマを焼く: 層を本物で組み、効果の列の出口(乗算済み線形)を読み戻して cache へ。
    /// 順に呼ぶ(feedback は 1 歩ずつ進む)。絵にならない層(網・点群)は false。
    pub fn freeze_bake_frame(&mut self, view: &StoreView<'_>, layer_id: LayerId, comp_frame: i64) -> Result<bool, EngineError> {
        let composition = view.composition()
            .map_err(|error| EngineError::Store(error.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let t = RationalTime::try_from_frame(comp_frame, composition.fps)
            .map_err(|error| EngineError::Time(error.to_string()))?;
        let (scene, camera, comp, fps) = self.evaluate_frame_graph_semantics(view, t)?;
        let Some(mut target) = scene.layers.into_iter().find(|layer| layer.layer == layer_id && layer.freeze_eligible && !layer.ghost) else {
            return Ok(false);
        };

        // Freeze stores the material result before external coverage/composite
        // semantics. Placement/opacity/blend/matte stay live when the cache is read.
        target.matte = None;
        target.clip_to_below = false;
        target.blend = crate::doc::store::BlendMode::Normal;

        self.freezing = Some(layer_id);
        let previous_clock = self.compositor.clock;
        let frame = t.try_to_frame_round(fps).unwrap_or(comp_frame) as f32;
        self.compositor.clock = Some([
            t.as_seconds_f64() as f32,
            fps.den() as f32 / fps.num() as f32,
            frame,
        ]);
        let prep = crate::render::engine::frame_graph_scene::Preparation::new(comp, Default::default(), crate::render::engine::frame_graph_scene::LegacyCameraSeam::new(camera));
        let prepared = self.prepare_gpu_pictures(
            &crate::frame_graph::SceneValue { layers: vec![target] },
            &prep,
            // A frozen frame is an evaluation at its own time: lit like a frame.
            true,
        );
        self.compositor.clock = previous_clock;
        self.freezing = None;

        let Some(layer) = prepared?.layers.into_iter().next() else { return Ok(false) };
        let Some(picture) = self.layer_with_passes_linear_picture(&layer)? else { return Ok(false) };
        let uploaded = self.compositor.upload_rgba16f(
            "motolii-frozen",
            picture.bytes.clone(),
            picture.width,
            picture.height,
        )?;
        let start = view.meta(layer_id)
            .map_err(|error| EngineError::Store(error.to_string()))?
            .map_or(0, |meta| meta.timing.start);
        let frozen = super::frozen::FrozenFrame {
            texture: uploaded,
            natural: picture.natural,
            padding: picture.padding,
            frame: picture.frame,
        };
        self.frozen.remember(
            layer_id,
            comp_frame - start,
            frozen,
            Some(&picture.bytes),
        ).map_err(|error| EngineError::Store(format!("Freeze の cache を書けない: {error}")))?;
        Ok(true)
    }

    /// Freeze の cache の置き場(書類の隣)。None なら GPU の中だけ。
    pub fn set_cache_root(&mut self, root: Option<std::path::PathBuf>) { self.frozen.root = root; }
    /// 書類の path から cache の置き場(`<name>.motolii-cache`)。
    pub fn cache_root_for(path: &std::path::Path) -> Option<std::path::PathBuf> { super::frozen::FrozenStore::root_for_document(Some(path)) }
    /// 焼いている最中は disk に新しいコマが増える: 「無い」と覚えた物を忘れて、また見に行く。
    pub fn refresh_frozen(&mut self) { self.frozen.forget_missing(); }
    /// Unfreeze: 層の cache を捨てる。
    pub fn forget_frozen(&mut self, layer: LayerId) { self.frozen.forget(layer); }
    /// 凍った層の、disk にあるコマの数。
    pub fn frozen_frames_on_disk(&self, layer: LayerId) -> usize { self.frozen.frames_on_disk(layer) }
    pub fn frozen_frames_resident(&self, layer: LayerId) -> usize { self.frozen.resident_count(layer) }

    /// Keys each persistent pass's history by its layer, copy and chain (the history store records
    /// which it read when a pass runs).
    pub(super) fn stamp_feedback(&self, passes: &mut [EffectPass], layer: LayerId, copy: u32, chain: u8, screen: Option<[u32; 3]>) {
        super::translate::stamp_feedback(passes, layer, copy, chain, screen, self.feedback_namespace);
    }




    /// The output at `t` into `target` (the composition's own window), as one tick with one view.
    pub fn render_frame_into(&mut self, view: &StoreView<'_>, t: RationalTime, target: &wgpu::Texture) -> Result<(), EngineError> {
        let comp = view.composition().map_err(|e| EngineError::Store(e.to_string()))?.ok_or(EngineError::NoComposition)?.spec();
        self.draw_one_view(view, t, target, None, true, &[], Window::output(comp))
    }

    /// The same world seen from an observation camera, with the authored layer projection kept.
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
        self.draw_one_view(view, t, target, Some(camera), include_background, outline, Window::output(comp))
    }

    /// The same world into a window of another size (a Stage tab: its size and region of interest).
    #[allow(clippy::too_many_arguments)]
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
        self.draw_one_view(view, t, target, Some(camera), include_background, outline, window)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_one_view(&mut self, view: &StoreView<'_>, t: RationalTime, target: &wgpu::Texture, camera: Option<ResolvedCamera>, include_background: bool, outline: &[LayerId], window: Window) -> Result<(), EngineError> {
        let request = super::frame_graph::tick::ViewRequest { target, window, camera, projection: crate::frame_graph::ViewProjection::Camera, include_background, outline };
        self.tick(view, t, &[request]).map(|_| ())
    }
}

// These adapters support the explicit `ResolvedLayer` oracle APIs below. The
// production render entry points evaluate a FrameGraph scene before lowering.

pub(crate) fn layer_size(layer: &ResolvedLayer, natural: [f32; 2]) -> [f32; 2] {
    [
        if layer.declared_size[0] > 0.0 { layer.declared_size[0] } else { natural[0] },
        if layer.declared_size[1] > 0.0 { layer.declared_size[1] } else { natural[1] },
    ]
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
