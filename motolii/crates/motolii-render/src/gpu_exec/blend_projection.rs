use crate::doc::store::{BlendMode, LayerProjection};

use super::blend_backend::{resident_blend, ResidentBlend};
use super::graph::GpuResourceGraph;
use super::projection_backend::{resident_projection, ResidentProjection};
use super::resource_store::GpuResourceStore;
use super::types::GpuResourceKey;

/// Combined residency for the two per-contribution authoring pins that carry
/// no GPU buffer of their own (blend mode, projection). Bundled so call
/// sites pay one field/one call instead of duplicating the same
/// version-lookup/install/mark_resident boilerplate twice.
#[derive(Default)]
pub(crate) struct GpuBlendProjectionResidency {
    blend: GpuResourceStore<ResidentBlend>,
    projection: GpuResourceStore<ResidentProjection>,
}

impl GpuBlendProjectionResidency {
    pub fn install(
        &mut self,
        graph: &mut GpuResourceGraph,
        generation: u64,
        blend_key: GpuResourceKey,
        projection_key: GpuResourceKey,
        blend: BlendMode,
        projection: LayerProjection,
    ) -> Option<()> {
        let blend_version = graph.version(blend_key)?;
        resident_blend(&mut self.blend, blend_key, blend_version, generation, blend);
        graph.mark_resident(blend_key, blend_version, generation);
        let projection_version = graph.version(projection_key)?;
        resident_projection(&mut self.projection, projection_key, projection_version, generation, projection);
        graph.mark_resident(projection_key, projection_version, generation);
        Some(())
    }

    pub fn fetch(
        &self,
        graph: &GpuResourceGraph,
        blend_key: GpuResourceKey,
        projection_key: GpuResourceKey,
    ) -> Option<(ResidentBlend, ResidentProjection)> {
        let blend_version = graph.version(blend_key)?;
        let blend = self.blend.current(blend_key, blend_version).copied()?;
        let projection_version = graph.version(projection_key)?;
        let projection = self.projection.current(projection_key, projection_version).copied()?;
        Some((blend, projection))
    }
}
