use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneValue, SolverPlanValue};

impl crate::render::engine::Engine {
    /// Conservative visibility planning belongs to GPU execution planning.
    /// Semantic SceneValue remains untouched and cacheable.
    pub(crate) fn gpu_plan_visible_scene(
        &self,
        scene: &SceneValue,
        solver: &SolverPlanValue,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Option<SceneValue> {
        if scene.layers.iter().any(|layer| !layer.image_sources.is_empty()) { return None; }
        let block_ids: std::collections::HashSet<&str> = self.compositor.catalog.definitions.iter()
            .filter(|definition| definition.manifest.stage == crate::render::compositor::effects::isf::IsfStage::Block)
            .map(|definition| definition.plugin_id()).collect();
        if scene.layers.iter().any(|layer| layer.effects.iter().any(|effect| block_ids.contains(effect.plugin_id.as_str()))) { return None; }
        if solver.layers.values().any(|value| value.relation != crate::frame_graph::RelationValue::default()) { return None; }

        let matte_sources: std::collections::HashSet<LayerId> = scene.layers.iter()
            .filter_map(|layer| layer.matte.map(|matte| matte.layer)).collect();
        let screen = [0.0f32, 0.0, comp.width as f32, comp.height as f32];
        let mut covers: Vec<[f32; 4]> = Vec::new();
        let mut omitted = std::collections::HashSet::new();

        for (index, layer) in scene.layers.iter().enumerate().rev() {
            let feeds_others = matte_sources.contains(&layer.layer) || layer.matte.is_some() || layer.clip_to_below;
            let SceneContentValue::Media { source, .. } = &layer.content else { continue };
            if crate::render::media::is_still_image_path(&source.path)
                || crate::render::media::is_audio_path(&source.path)
                || crate::render::media::is_mesh_path(&source.path)
                || crate::render::media::is_point_cloud_path(&source.path) { continue; }
            let Some(info) = self.probes.get(&source.path) else { continue };
            if info.rotation != 0 { continue; }
            let size = [info.width as f32, info.height as f32];
            let corners = crate::doc::core::projected_screen_corners(
                comp, camera, camera, layer.projection, layer.transform.spatial,
                [0.0, 0.0, 0.0], [size[0], size[1], 0.0],
            );
            if corners.iter().any(|corner| !corner.is_finite()) { continue; }
            let rect = corners.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |rect, corner| {
                [rect[0].min(corner.x), rect[1].min(corner.y), rect[2].max(corner.x), rect[3].max(corner.y)]
            });
            let epsilon = 0.5;
            let axis_aligned = corners.iter().all(|corner| {
                ((corner.x - rect[0]).abs() < epsilon || (corner.x - rect[2]).abs() < epsilon)
                    && ((corner.y - rect[1]).abs() < epsilon || (corner.y - rect[3]).abs() < epsilon)
            });
            let plain = layer.effects.is_empty() && layer.after_effects.is_empty()
                && layer.masks.is_empty() && !layer.flatten;
            let offscreen = rect[2] < screen[0] || rect[3] < screen[1] || rect[0] > screen[2] || rect[1] > screen[3];
            let covered = covers.iter().any(|cover| rect[0] >= cover[0] && rect[1] >= cover[1] && rect[2] <= cover[2] && rect[3] <= cover[3]);
            if !feeds_others && plain && (offscreen || covered) {
                omitted.insert(index);
                continue;
            }
            let opaque = axis_aligned && plain && layer.opacity >= 1.0
                && layer.blend == crate::doc::store::BlendMode::Normal
                && layer.matte.is_none() && !layer.clip_to_below && !layer.ghost;
            if opaque { covers.push(rect); }
        }

        (!omitted.is_empty()).then(|| SceneValue {
            layers: scene.layers.iter().enumerate()
                .filter(|(index, _)| !omitted.contains(index))
                .map(|(_, layer)| layer.clone()).collect(),
        })
    }
}
