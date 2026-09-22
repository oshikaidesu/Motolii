use std::collections::BTreeMap;

use super::{
    GpuBackend, GpuOperationTable, GpuPassDesc, GpuPassKey, GpuPhysicalSlot,
    GpuResourceKey,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EngineGpuOperation {
    Resident,
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
            Some(EngineGpuOperation::Resident) => Ok(()),
            Some(EngineGpuOperation::Present | EngineGpuOperation::Readback) => {
                Err(EngineGpuBackendError::SinkRequiresEngineContext(key))
            }
            None => Err(EngineGpuBackendError::MissingOperation(key)),
        }
    }
}
