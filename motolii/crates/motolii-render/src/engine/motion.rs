//! 時間で重ねる描き方: Motion Blur の写しを 1 枚の板から作る、隣のコマの時刻、粒子の層の点をこのコマで解く。

#[allow(unused_imports)]
use crate::picture::resolved::{ResolvedEffect, ResolvedLayer, ResolvedMask};
use std::collections::HashMap;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{LayerId, LayerSource, RationalTime, ShapeNode, StoreView, TextDocument};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::translate::translate_effect_passes;
use crate::render::engine::{Engine, EngineError};

impl Engine {
    /// Motion Blur: 写しは置き場所だけが違う。素材座標の絵(Blur と同じ、余白は効果が宣言)を 1 回だけ組み、
    /// 同じ絵と同じ効果列を写しごとの置き場所で並べる(効果の列は同じ素材 × 同じ列を 1 回だけ流して配る)。足すのは呼び手の bake。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn motion_blur_copies(
        &mut self,
        previous_build: &mut Option<(LayerId, i64, Layer)>,
        copies: &[ResolvedLayer],
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        let Some(first) = copies.first() else { return Ok(Vec::new()) };
        let Some(built) = self.build_layer_shared(previous_build, first, text_documents, shape_documents, t, comp, camera, projection_camera, CompositeBlendMode::Normal)? else {
            return Ok(Vec::new());
        };
        let mut passes = translate_effect_passes(&first.effects);
        let screen = built.content.texture().is_none().then_some([comp.width, comp.height]);
        self.stamp_feedback(&mut passes, first.id, first.copy, 0, screen);
        Ok(copies.iter().map(|copy| {
            let mut placed = built.clone();
            placed.placement = copy.placement;
            LayerWithPasses { layer: placed, passes: passes.clone(), pass_sources: Vec::new(), padding: 0, cut: Vec::new() }
        }).collect())
    }

    /// このコマの粒子の層の点を書類から解いて置く(`texture_for_resolved` が読む)。
    pub(super) fn solve_particles(&mut self, view: &StoreView<'_>, resolved: &[ResolvedLayer], t: RationalTime) {
        self.particle_frames.clear();
        for layer in resolved.iter().filter(|l| l.source == LayerSource::Particles) {
            if self.particle_frames.contains_key(&layer.id) {
                continue;
            }
            match crate::doc::store::particles::particles_at(view, layer.id, t) {
                Ok((particles, turbulence, links)) => { self.particle_frames.insert(layer.id, super::ParticleFrame::from_particles(&particles, turbulence, links)); }
                Err(e) => self.layer_failures.push(format!("粒子の層 {} を解けない: {e}", layer.id.0)),
            }
        }
    }
}

/// t から整数コマずらした comp の時刻(TIME_OFFSET_FRAMES)。comp の前は 0。
pub(super) fn shifted_by_frames(view: &StoreView<'_>, t: RationalTime, frames: f32) -> RationalTime {
    view.composition().ok().flatten()
        .and_then(|c| t.try_to_frame_round(c.fps).ok().and_then(|now| RationalTime::try_from_frame((now + frames.round() as i64).max(0), c.fps).ok()))
        .unwrap_or(RationalTime::ZERO)
}
