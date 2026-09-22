use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::SceneValue;
use crate::render::compositor::LayerWithPasses;

#[derive(Clone)]
pub(crate) struct ExecutableScene {
    pub layers: Vec<LayerWithPasses>,
    pub layer_ids: Vec<LayerId>,
}

/// Concrete scene dispatcher. Semantic meaning is already resolved; this only
/// joins resident producer outputs into executable contributions.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_executable_scene(
        &mut self,
        scene: &SceneValue,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<ExecutableScene, crate::render::engine::EngineError> {
        let clip_bases: std::collections::HashSet<_> = scene.layers.iter()
            .filter(|layer| layer.clip_to_below)
            .filter_map(|layer| layer.matte.map(|matte| matte.layer))
            .collect();
        let mut layers = Vec::with_capacity(scene.layers.len());

        for source in &scene.layers {
            let solid = crate::render::engine::translate::translate_solid(&source.effects)
                .map(|solid| if solid.depth > 0.0 { solid } else {
                    crate::render::compositor::extrude::Solid { depth: source.depth, ..solid }
                })
                .unwrap_or(crate::render::compositor::extrude::Solid { depth: source.depth, bevel: None });
            let force_picture = clip_bases.contains(&source.layer) || solid.extent() > 0.0;
            let resident = self.frame_graph_resident_content(source);
            let Some((resident, padding, frame, frozen)) = self.gpu_special_content(
                source, resident, force_picture, comp, camera,
            )? else { continue; };
            let placement = self.frame_graph_resident_placement(source)
                .unwrap_or_else(|| crate::gpu_exec::ResidentPlacement::from_scene(source));
            let effects = self.frame_graph_resident_effects(source).unwrap_or_else(|| {
                let mut passes = crate::render::engine::translate::translate_effect_passes(&source.effects);
                let mut plate_passes = crate::render::engine::translate::translate_plate_passes(&source.after_effects);
                crate::render::engine::translate::stamp_feedback(&mut passes, source.layer, source.instance, 0, None, 0);
                crate::render::engine::translate::stamp_feedback(&mut plate_passes, source.layer, source.instance, 1, None, 0);
                crate::gpu_exec::ResidentEffectChain { passes, plate_passes }
            });
            let pass_sources = if frozen { Vec::new() } else {
                self.frame_graph_snapshot_rows(source, comp, camera)?
            };
            let mut contribution = self.gpu_contribution_layer(
                source, resident, placement, effects, pass_sources, comp, camera,
            )?;
            contribution.padding = padding;
            contribution.layer.frame = frame;
            layers.push(contribution);
        }

        let (layers, layer_ids) = self.gpu_cross_contributions(scene, layers, comp, camera)?;
        Ok(ExecutableScene { layers, layer_ids })
    }
}
