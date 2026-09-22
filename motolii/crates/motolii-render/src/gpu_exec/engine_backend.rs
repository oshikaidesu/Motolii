use std::collections::BTreeMap;

use super::{
    GpuBackend, GpuOperationTable, GpuPassDesc, GpuPassKey, GpuPhysicalSlot,
    GpuResourceKey,
};

/// Concrete cross-contribution execution is separated from planning. The
/// executor passes only resolved logical resource keys and immutable payload.
pub(crate) trait EngineCrossExecutor {
    type Error;
    fn execute_clip(&mut self, source: GpuResourceKey, base: GpuResourceKey, output: GpuResourceKey) -> Result<(), Self::Error>;
    fn execute_matte(&mut self, target: GpuResourceKey, source: GpuResourceKey, output: GpuResourceKey, mode: crate::doc::store::MatteMode) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EngineGpuOperation {
    /// Producer already materialized its backend payload while the concrete
    /// backends are being migrated behind the executor.
    Resident,
    /// Explicit cross-contribution clip operation. Resource identities are
    /// resolved by the semantic adapter/lowerer; the backend must not search
    /// SceneValue or LayerId to rediscover them.
    Clip {
        source: GpuResourceKey,
        base: GpuResourceKey,
        output: GpuResourceKey,
    },
    /// Explicit matte operation with its resolved mode carried as payload.
    Matte {
        target: GpuResourceKey,
        source: GpuResourceKey,
        output: GpuResourceKey,
        mode: crate::doc::store::MatteMode,
    },
    Present,
    Readback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EngineGpuBackendError {
    MissingOperation(GpuPassKey),
    MissingCrossResource(GpuResourceKey),
    SinkRequiresEngineContext(GpuPassKey),
}

pub(crate) struct EngineGpuBackend<'a, X> {
    pub operations: &'a GpuOperationTable<EngineGpuOperation>,
    pub cross: X,
}


impl<X> GpuBackend for EngineGpuBackend<'_, X>
where X: EngineCrossExecutor<Error = EngineGpuBackendError> {
    type Error = EngineGpuBackendError;

    fn execute_pass(
        &mut self,
        pass: &GpuPassDesc,
        _aliases: &BTreeMap<GpuResourceKey, GpuPhysicalSlot>,
    ) -> Result<(), Self::Error> {
        let key = pass.key();
        match self.operations.get(key).copied() {
            Some(EngineGpuOperation::Resident) => Ok(()),
            Some(EngineGpuOperation::Clip { source, base, output }) => self.cross.execute_clip(source, base, output),
            Some(EngineGpuOperation::Matte { target, source, output, mode }) => self.cross.execute_matte(target, source, output, mode),
            Some(EngineGpuOperation::Present | EngineGpuOperation::Readback) => {
                Err(EngineGpuBackendError::SinkRequiresEngineContext(key))
            }
            None => Err(EngineGpuBackendError::MissingOperation(key)),
        }
    }
}




/// Resource-keyed cross executor. Relationship discovery is already complete;
/// this adapter only invokes the proven compositor primitives.
pub(crate) struct CompositorCrossExecutor<'a> {
    pub engine: &'a mut crate::render::engine::Engine,
    pub resources: &'a mut super::GpuResourceStore<super::ResidentCompositeLayer>,
    pub versions: &'a super::GpuResourceGraph,
    pub generation: u64,
    pub comp: crate::doc::core::CompSpec,
    pub camera: crate::doc::core::ResolvedCamera,
}

impl EngineCrossExecutor for CompositorCrossExecutor<'_> {
    type Error = EngineGpuBackendError;

    fn execute_clip(&mut self, source: GpuResourceKey, base: GpuResourceKey, output: GpuResourceKey) -> Result<(), Self::Error> {
        let (_, source_value) = self.resources.get_any(source).ok_or(EngineGpuBackendError::MissingCrossResource(source))?;
        let (_, base_value) = self.resources.get_any(base).ok_or(EngineGpuBackendError::MissingCrossResource(base))?;
        let source_layer = source_value.layer.clone();
        let base_layer = base_value.layer.clone();
        let Some(result) = self.engine.clip_onto_base(base_layer, &source_layer.layer, &source_layer.passes)
            .map_err(|_| EngineGpuBackendError::MissingCrossResource(output))? else {
            return Err(EngineGpuBackendError::MissingCrossResource(output));
        };
        let version = self.versions.version(output).ok_or(EngineGpuBackendError::MissingCrossResource(output))?;
        self.resources.install(output, version, self.generation, super::ResidentCompositeLayer { layer: result });
        Ok(())
    }

    fn execute_matte(&mut self, target: GpuResourceKey, source: GpuResourceKey, output: GpuResourceKey, mode: crate::doc::store::MatteMode) -> Result<(), Self::Error> {
        let (_, target_value) = self.resources.get_any(target).ok_or(EngineGpuBackendError::MissingCrossResource(target))?;
        let (_, source_value) = self.resources.get_any(source).ok_or(EngineGpuBackendError::MissingCrossResource(source))?;
        let target_layer = target_value.layer.clone();
        let source_layer = source_value.layer.clone();
        let target_layer = self.engine.apply_effects_before_matte(self.comp, self.camera, target_layer.layer, &target_layer.passes)
            .map_err(|_| EngineGpuBackendError::MissingCrossResource(output))?;
        let source_layer = self.engine.apply_effects_before_matte(self.comp, self.camera, source_layer.layer, &source_layer.passes)
            .map_err(|_| EngineGpuBackendError::MissingCrossResource(output))?;
        let layer = self.engine.apply_matte(self.comp, self.camera, &target_layer, &source_layer, mode)
            .map_err(|_| EngineGpuBackendError::MissingCrossResource(output))?;
        let result = crate::render::compositor::LayerWithPasses { layer, passes: Vec::new(), padding: 0, pass_sources: Vec::new(), cut: Vec::new() };
        let version = self.versions.version(output).ok_or(EngineGpuBackendError::MissingCrossResource(output))?;
        self.resources.install(output, version, self.generation, super::ResidentCompositeLayer { layer: result });
        Ok(())
    }
}
