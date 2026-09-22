//! Greenfield GPU control plane.
//!
//! Semantic FrameGraph decides meaning. This module owns the lower-level GPU
//! resource/execution graph and deliberately does not preserve the old
//! GpuScene/LayerWithPasses orchestration contract.

mod content_backend;
mod effect_backend;
mod executor;
mod graph;
mod image_source;
mod lowerer;
mod logical_lowerer;
mod planner;
mod placement_backend;
mod semantic_adapter;
mod snapshot_backend;
mod resource_store;
mod telemetry;
mod types;

pub(crate) use content_backend::ResidentContent;
pub(crate) use effect_backend::{effect_chain_key, resident_effect_chain, ResidentEffectChain};
pub(crate) use executor::{GpuBackend, GpuExecuteError, GpuExecutionStats, GpuExecutor};
pub(crate) use image_source::{lower_image_source, GpuImageSourceInput, GpuImageSourceResources};
pub(crate) use graph::{GpuGraphError, GpuGraphStats, GpuResourceDelta, GpuResourceGraph};
pub(crate) use logical_lowerer::{LogicalGpuLowerer, LogicalLowerError};
pub(crate) use lowerer::GpuLowerer;
pub(crate) use lowerer::{
    GpuContributionInput, GpuContributionResources, VersionedSemantic,
};
pub(crate) use resource_store::{GpuOperationTable, GpuResourceStore};
pub(crate) use snapshot_backend::ResidentSnapshot;
pub(crate) use semantic_adapter::{SemanticAdapterError, SemanticGpuAdapter};
pub(crate) use placement_backend::{resident_placement, ResidentPlacement};
pub(crate) use planner::{
    GpuExecutionPlan, GpuPlanError, GpuPlanner, GpuResourceInterval,
};
pub(crate) use telemetry::{GpuFrameCounters, GpuTelemetry};
pub(crate) use types::{
    GpuAliasClass, GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKey, GpuPassKind,
    GpuPhysicalSlot, GpuResourceClass, GpuResourceDesc, GpuResourceIdentity, GpuResourceKey,
    GpuResourceLifetime, GpuResourceVersion,
};
