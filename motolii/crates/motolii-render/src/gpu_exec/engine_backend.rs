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


/// Transitional concrete executor for cross-contribution resources. It is
/// intentionally resource-keyed: all relationship discovery has already
/// happened in the semantic adapter/lowerer.
pub(crate) struct ResidentCrossExecutor<'a> {
    pub composites: &'a mut super::GpuResourceStore<super::ResidentCompositeLayer>,
    pub generation: u64,
}

impl EngineCrossExecutor for ResidentCrossExecutor<'_> {
    type Error = EngineGpuBackendError;

    fn execute_clip(&mut self, source: GpuResourceKey, base: GpuResourceKey, output: GpuResourceKey) -> Result<(), Self::Error> {
        // Concrete clip rendering is migrated separately; this owner only
        // accepts already-addressed resources and never scans semantic layers.
        let _ = self.composites.get_any(source).ok_or(EngineGpuBackendError::MissingCrossResource(source))?;
        let (_, base_value) = self.composites.get_any(base).ok_or(EngineGpuBackendError::MissingCrossResource(base))?;
        let value = base_value.clone();
        self.composites.install(output, super::GpuResourceVersion::STATIC, self.generation, value);
        Ok(())
    }

    fn execute_matte(&mut self, target: GpuResourceKey, source: GpuResourceKey, output: GpuResourceKey, _mode: crate::doc::store::MatteMode) -> Result<(), Self::Error> {
        let (_, target_value) = self.composites.get_any(target).ok_or(EngineGpuBackendError::MissingCrossResource(target))?;
        let _ = self.composites.get_any(source).ok_or(EngineGpuBackendError::MissingCrossResource(source))?;
        let value = target_value.clone();
        self.composites.install(output, super::GpuResourceVersion::STATIC, self.generation, value);
        Ok(())
    }
}
