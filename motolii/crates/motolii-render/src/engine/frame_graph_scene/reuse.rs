//! A frame that differs from the previous one only in where things are placed reuses its
//! prepared layers.

use crate::frame_graph::{SceneContentValue, SceneLayerValue, SceneValue};
use crate::render::engine::Engine;

use super::GpuSceneValue;

/// The scene a prepared frame was built from, kept to recognise a frame that differs only in
/// where things are placed.
pub(in crate::engine) struct PreparedFrame {
    scene: SceneValue,
    prepared: GpuSceneValue,
}

impl Engine {
    /// The previous frame's prepared layers with this frame's placements, when nothing but
    /// placement changed. No lowering, no preparation: the cost is one pass over the layers.
    pub(super) fn moved_only(&mut self, scene: &SceneValue) -> Option<GpuSceneValue> {
        let last = self.last_prepared.as_ref()?;
        if last.scene.layers.len() != scene.layers.len()
            || !last.scene.layers.iter().zip(&scene.layers).all(|(a, b)| same_but_placement(a, b))
        {
            return None;
        }
        let mut prepared = last.prepared.clone();
        for (layer, source) in prepared.layers.iter_mut().zip(&prepared.plain_sources) {
            let Some(index) = *source else { return None };
            let placed = &scene.layers[index];
            let placement = &mut layer.layer.placement;
            placement.transform = placed.transform.affine;
            placement.world_transform = Some(placed.transform.spatial);
            placement.z = placed.transform.spatial.translation.z;
            placement.opacity = placed.opacity;
            placement.order = i32::from(placed.order);
        }
        self.last_prepared = Some(PreparedFrame { scene: scene.clone(), prepared: prepared.clone() });
        Some(prepared)
    }

    /// Keeps a prepared frame whose layers depend only on their contributions' content, so a
    /// later frame that merely moves them can reuse it.
    pub(super) fn remember_prepared(&mut self, scene: &SceneValue, uncut: bool, prepared: &GpuSceneValue) {
        // Preparation that moved a layer itself (a planar warp's frame) is not a placement to replace.
        let placed_as_authored = prepared.layers.iter().zip(&prepared.plain_sources).all(|(layer, source)| {
            source.is_some_and(|index| {
                let authored = &scene.layers[index];
                layer.layer.placement.transform == authored.transform.affine
                    && layer.layer.placement.world_transform == Some(authored.transform.spatial)
            })
        });
        let reusable = uncut && placed_as_authored && scene.layers.iter().all(placement_independent);
        self.last_prepared = reusable.then(|| PreparedFrame { scene: scene.clone(), prepared: prepared.clone() });
    }

}
/// A contribution whose prepared picture does not depend on where it is placed or on the frame.
fn placement_independent(layer: &SceneLayerValue) -> bool {
    let content = match &layer.content {
        SceneContentValue::None | SceneContentValue::Shape(_) | SceneContentValue::Text(_) | SceneContentValue::Material(_) => true,
        SceneContentValue::Media { source, .. } => crate::render::media::is_still_image_path(&source.path),
        SceneContentValue::Particles(_) | SceneContentValue::Plate(_) => false,
    };
    content
        && layer.image_sources.is_empty()
        && layer.matte.is_none()
        && !layer.clip_to_below
        && !layer.flatten
        && !layer.freeze_eligible
        && !layer.effects.iter().chain(&layer.after_effects).any(|effect| crate::extensions::overlay::is_track_overlay(&effect.plugin_id))
}

/// Everything but placement (transform, opacity, order) is the same; content is compared by
/// identity of the shared evaluated value, never by walking it.
fn same_but_placement(a: &SceneLayerValue, b: &SceneLayerValue) -> bool {
    let content = match (&a.content, &b.content) {
        (SceneContentValue::None, SceneContentValue::None) => true,
        (SceneContentValue::Shape(x), SceneContentValue::Shape(y)) => std::sync::Arc::ptr_eq(x, y),
        (SceneContentValue::Text(x), SceneContentValue::Text(y)) => std::sync::Arc::ptr_eq(x, y),
        (SceneContentValue::Material(x), SceneContentValue::Material(y)) => x == y,
        (SceneContentValue::Media { source: x, time: tx }, SceneContentValue::Media { source: y, time: ty }) => {
            x == y && (tx == ty || crate::render::media::is_still_image_path(&x.path))
        }
        _ => false,
    };
    content
        && placement_independent(b)
        && a.layer == b.layer
        && a.instance == b.instance
        && a.source == b.source
        && a.content_key == b.content_key
        && a.effects == b.effects
        && a.after_effects == b.after_effects
        && a.masks == b.masks
        && a.environment == b.environment
        && a.ghost == b.ghost
        && a.timing_start == b.timing_start
        && a.projection == b.projection
        && a.blend == b.blend
        && a.shape_stretch == b.shape_stretch
        && a.depth == b.depth
}
