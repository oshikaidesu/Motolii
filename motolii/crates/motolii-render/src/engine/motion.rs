//! 時間で重ねる描き方: Motion Blur の写しを 1 枚の板から作る、隣のコマの時刻、粒子の層の点をこのコマで解く。

use std::collections::HashMap;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::{LayerId, LayerSource, RationalTime, ResolvedLayer, ShapeNode, StoreView, TextDocument};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};
use crate::render::engine::translate::translate_effect_passes;
use crate::render::engine::{Engine, EngineError};

impl Engine {
    /// Motion Blur: 写しは変換だけが違うので、1 枚目を comp 大の板に 1 回だけ焼き(上の効果もここで 1 回)、
    /// 板を写しごとのずれで置く(足すのは呼び手の bake)。形・文字は矩形でないと足す合成に乗らないので、必ず板にする。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn motion_blur_copies(
        &mut self,
        previous_build: &mut Option<(LayerId, i64, Layer)>,
        copies: &[ResolvedLayer],
        count: u32,
        text_documents: &HashMap<LayerId, TextDocument>,
        shape_documents: &HashMap<LayerId, Vec<ShapeNode>>,
        t: RationalTime,
        comp: CompSpec,
        camera: ResolvedCamera,
        projection_camera: ResolvedCamera,
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        let Some(first) = copies.first() else { return Ok(Vec::new()) };
        let Some(mut built) = self.build_layer_shared(previous_build, first, text_documents, shape_documents, t, comp, camera, projection_camera, CompositeBlendMode::Normal)? else {
            return Ok(Vec::new());
        };
        // 写しの不透明度(1/枚数)は置く時に 1 回だけ掛ける。板は層の不透明度で焼く。
        built.placement.opacity = (first.placement.opacity * count as f32).min(1.0);
        let mut passes = translate_effect_passes(&first.effects);
        let screen = built.content.texture().is_none().then_some([comp.width, comp.height]);
        self.stamp_feedback(&mut passes, first.id, first.copy, 0, screen);
        let mut plate = self.bake_isolated_layers(comp, camera, vec![LayerWithPasses { layer: built, passes, pass_sources: Vec::new(), padding: 0 }], CompositeBlendMode::Normal, first.placement, false)?;
        plate.placement.opacity = 1.0;
        let reference = first.placement.transform.inverse();
        Ok(copies.iter().map(|copy| {
            let mut moved = plate.clone();
            moved.placement.transform = copy.placement.transform * reference;
            moved.placement.opacity = 1.0 / count as f32;
            LayerWithPasses { layer: moved, passes: Vec::new(), pass_sources: Vec::new(), padding: 0 }
        }).collect())
    }

    /// このコマの粒子の層の点を書類から解いて置く(`texture_for_resolved` が読む)。
    pub(super) fn solve_particles(&mut self, view: &StoreView<'_>, resolved: &[ResolvedLayer], t: RationalTime) {
        self.particle_frames.clear();
        for layer in resolved.iter().filter(|l| l.source == LayerSource::Particles) {
            if self.particle_frames.contains_key(&layer.id) {
                continue;
            }
            match view.particles_at(layer.id, t) {
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
