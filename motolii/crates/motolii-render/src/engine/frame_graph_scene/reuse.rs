//! Per-contribution preparation: a contribution that is unchanged but for where it is placed
//! keeps its lowered work and prepared picture from the previous frame; only changed
//! contributions are lowered and prepared again.

use crate::doc::core::LayerPlacement;
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneLayerValue, SceneValue};
use crate::render::compositor::LayerWithPasses;
use crate::render::engine::{Engine, EngineError};
use crate::render_graph::LayerWork;

use super::GpuSceneValue;

/// What a contribution was last prepared from, and what it produced.
pub(in crate::engine) struct CachedContribution {
    scene: SceneLayerValue,
    clip_base: bool,
    /// The shelf it was prepared with: a saved Vism changes what the same effects mean.
    catalog: u64,
    work: LayerWork,
    prepared: Option<LayerWithPasses>,
}

pub(in crate::engine) type ContributionCache = std::collections::HashMap<(LayerId, u32), CachedContribution>;

impl Engine {
    /// The production frame: contributions unchanged since the last frame but for placement are
    /// reused with their new placement; the rest are lowered and prepared.
    pub(super) fn prepare_gpu_scene_incremental(&mut self, scene: &SceneValue, prep: &mut super::Preparation) -> Result<GpuSceneValue, EngineError> {
        self.compositor.refresh_catalog_programs();
        let catalog = self.compositor.catalog.clone();
        let mut reused: Vec<Option<Option<LayerWithPasses>>> = vec![None; scene.layers.len()];
        let cache = &self.contributions;
        let (graph, bases) = crate::render_lowering::lower_scene_reusing(scene, &catalog, &mut |index, clip_base| {
            let current = &scene.layers[index];
            let hit = cache.get(&(current.layer, current.instance))
                .filter(|cached| cached.clip_base == clip_base && cached.catalog == catalog.generation && same_but_placement(&cached.scene, current))?;
            let mut work = hit.work.clone();
            place(&mut work.placement, current);
            let mut prepared = hit.prepared.clone();
            if let Some(prepared) = &mut prepared {
                place(&mut prepared.layer.placement, current);
            }
            reused[index] = Some(prepared);
            Some(work)
        }).map_err(|error| EngineError::Store(error.to_string()))?;
        self.adopt_world_environment(&graph)?;

        let kept: Vec<bool> = reused.iter().map(Option::is_some).collect();
        let mut prepared = Vec::with_capacity(graph.layers.len());
        for (index, work) in graph.layers.iter().enumerate() {
            prepared.push(match reused[index].take() {
                Some(kept) => kept,
                None => {
                    self.prepared_contributions += 1;
                    let current = &scene.layers[index];
                    let why = why_prepared(self.contributions.get(&(current.layer, current.instance)), current, bases[index], catalog.generation);
                    let started = std::time::Instant::now();
                    let layer = self.execute_layer(work, prep)?;
                    self.ledger.claim("prepare", format!("layer {}", current.layer.0), why, started.elapsed());
                    layer
                }
            });
        }
        let result = self.compose_prepared(&graph, &prepared, prep)?;

        let mut previous = std::mem::take(&mut self.contributions);
        let mut next = ContributionCache::with_capacity(scene.layers.len());
        for (index, (current, work)) in scene.layers.iter().zip(&graph.layers).enumerate() {
            let key = (current.layer, current.instance);
            // A reused contribution's record still describes it: move it, copy nothing.
            if kept[index] {
                if let Some(record) = previous.remove(&key) {
                    next.insert(key, record);
                }
                continue;
            }
            let layer = &prepared[index];
            // Preparation that moved the picture itself (a planar warp's frame) is not a placement to replace.
            let placed_as_authored = layer.as_ref().is_none_or(|layer| {
                layer.layer.placement.transform == current.transform.affine
                    && layer.layer.placement.world_transform == Some(current.transform.spatial)
            });
            // A pass with history is a recurrence over frames; its picture is never the previous one.
            let has_history = layer.as_ref().is_some_and(|layer| layer.passes.iter().any(|pass| pass.feedback.is_some()));
            if placement_independent(current) && placed_as_authored && !has_history {
                next.insert(key, CachedContribution {
                    scene: current.clone(),
                    clip_base: bases[index],
                    catalog: catalog.generation,
                    work: work.clone(),
                    prepared: layer.clone(),
                });
            }
        }
        self.contributions = next;
        Ok(result)
    }
}

/// Why a contribution could not keep its previous preparation.
fn why_prepared(previous: Option<&CachedContribution>, current: &SceneLayerValue, clip_base: bool, catalog: u64) -> &'static str {
    if !placement_independent(current) {
        return match &current.content {
            SceneContentValue::Media { .. } => "a video frame changes every frame",
            SceneContentValue::Particles(_) => "particles move every frame",
            SceneContentValue::Plate(_) => "a plate is composed every frame",
            _ if current.reads_other_pictures() => "it reads another layer or another time",
            _ if current.flatten => "flattened in composition space",
            _ if current.freeze_eligible => "freezable",
            _ => "its picture depends on the frame",
        };
    }
    let Some(previous) = previous else { return "new this frame" };
    if previous.catalog != catalog { return "a Vism on the shelf was saved"; }
    let a = &previous.scene;
    let content = match (&a.content, &current.content) {
        (SceneContentValue::Shape(x), SceneContentValue::Shape(y)) => std::sync::Arc::ptr_eq(x, y),
        (SceneContentValue::Text(x), SceneContentValue::Text(y)) => std::sync::Arc::ptr_eq(x, y),
        (x, y) => x == y,
    };
    if !content { "its content changed" }
    else if a.effects != current.effects || a.after_effects != current.after_effects { "an effect value changed" }
    else if a.masks != current.masks { "a mask changed" }
    else if a.matte != current.matte || a.clip_to_below != current.clip_to_below || previous.clip_base != clip_base { "its matte or clip changed" }
    else { "an attribute changed" }
}

fn place(placement: &mut LayerPlacement, layer: &SceneLayerValue) {
    placement.transform = layer.transform.affine;
    placement.world_transform = Some(layer.transform.spatial);
    placement.z = layer.transform.spatial.translation.z;
    placement.opacity = layer.opacity;
    placement.order = i32::from(layer.order);
}

/// A contribution whose prepared picture does not depend on where it is placed or on the frame.
/// Clip groups and mattes are composed after preparation, so they do not matter here.
fn placement_independent(layer: &SceneLayerValue) -> bool {
    let content = match &layer.content {
        SceneContentValue::None | SceneContentValue::Shape(_) | SceneContentValue::Text(_) | SceneContentValue::Material(_) => true,
        SceneContentValue::Media { source, .. } => crate::render::media::is_still_image_path(&source.path),
        SceneContentValue::Particles(_) | SceneContentValue::Plate(_) => false,
    };
    content
        && !layer.reads_other_pictures()
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
        && a.matte == b.matte
        && a.clip_to_below == b.clip_to_below
        && a.environment == b.environment
        && a.ghost == b.ghost
        && a.timing_start == b.timing_start
        && a.projection == b.projection
        && a.blend == b.blend
        && a.shape_stretch == b.shape_stretch
        && a.depth == b.depth
}
