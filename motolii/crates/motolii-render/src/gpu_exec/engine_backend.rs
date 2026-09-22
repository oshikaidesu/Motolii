use std::collections::BTreeMap;

use super::{
    GpuBackend, GpuOperationTable, GpuPassDesc, GpuPassKey, GpuPhysicalSlot,
    GpuResourceKey,
};

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
    SinkRequiresEngineContext(GpuPassKey),
}

pub(crate) struct EngineGpuBackend<'a> {
    pub operations: &'a GpuOperationTable<EngineGpuOperation>,
}

impl GpuBackend for EngineGpuBackend<'_> {
    type Error = EngineGpuBackendError;

    fn execute_pass(
        &mut self,
        pass: &GpuPassDesc,
        _aliases: &BTreeMap<GpuResourceKey, GpuPhysicalSlot>,
    ) -> Result<(), Self::Error> {
        let key = pass.key();
        match self.operations.get(key).copied() {
            Some(EngineGpuOperation::Resident
                | EngineGpuOperation::Clip { .. }
                | EngineGpuOperation::Matte { .. }) => Ok(()),
            Some(EngineGpuOperation::Present | EngineGpuOperation::Readback) => {
                Err(EngineGpuBackendError::SinkRequiresEngineContext(key))
            }
            None => Err(EngineGpuBackendError::MissingOperation(key)),
        }
    }
}
