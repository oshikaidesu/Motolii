use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneLayerValue, SceneValue};
use crate::render::compositor::LayerWithPasses;

/// Cross-contribution execution. All layers are already concrete contribution
/// outputs; this stage only applies explicit clip/matte edges and filters
/// consumed source contributions.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_cross_contributions(
        &mut self,
        scene: &SceneValue,
        mut layers: Vec<LayerWithPasses>,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<(Vec<LayerWithPasses>, Vec<LayerId>), crate::render::engine::EngineError> {
        #[derive(Clone, Copy)]
        struct Entry {
            layer: LayerId,
            matte: Option<crate::doc::store::Matte>,
            clip_to_below: bool,
            stencil: bool,
        }
        let entries: Vec<_> = scene.layers.iter().map(|source| Entry {
            layer: source.layer,
            matte: source.matte,
            clip_to_below: source.clip_to_below,
            stencil: source.blend.is_stencil(),
        }).collect();
        let by_id: std::collections::HashMap<_, _> = entries.iter().enumerate()
            .map(|(index, entry)| (entry.layer, index)).collect();
        let mut removed = vec![false; layers.len()];

        for index in 0..layers.len() {
            let entry = entries[index];
            if !entry.clip_to_below || entry.stencil { continue; }
            let Some(base_id) = entry.matte.map(|matte| matte.layer) else {
                removed[index] = true; continue;
            };
            let Some(base_index) = by_id.get(&base_id).copied() else {
                removed[index] = true; continue;
            };
            match self.clip_onto_base(
                layers[base_index].clone(),
                &layers[index].layer,
                &layers[index].passes,
            )? {
                Some(clipped) => layers[base_index] = clipped,
                None => self.layer_failures.push(format!(
                    "layer {} clips to a base without a texture (point cloud / model bases are not clippable)",
                    entry.layer.0
                )),
            }
            removed[index] = true;
        }

        let matte_sources: std::collections::HashSet<_> = scene.layers.iter()
            .filter(|entry| !entry.clip_to_below)
            .filter_map(|entry| entry.matte.map(|matte| matte.layer))
            .collect();
        for index in 0..layers.len() {
            if removed[index] || entries[index].clip_to_below { continue; }
            let Some(matte) = entries[index].matte else { continue; };
            let Some(source_index) = by_id.get(&matte.layer).copied() else {
                removed[index] = true; continue;
            };
            let semantic: &SceneLayerValue = &scene.layers[index];
            layers[index] = self.frame_graph_matte(
                semantic, &layers[index], &layers[source_index], comp, camera,
            )?;
        }

        let kept: Vec<_> = layers.into_iter().enumerate().filter(|(index, _)| {
            !removed[*index] && !entries[*index].stencil && !matte_sources.contains(&entries[*index].layer)
        }).map(|(index, layer)| (entries[index].layer, layer)).collect();
        let ids = kept.iter().map(|(id, _)| *id).collect();
        let layers = kept.into_iter().map(|(_, layer)| layer).collect();
        Ok((layers, ids))
    }
}
