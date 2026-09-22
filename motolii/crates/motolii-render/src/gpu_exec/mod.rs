//! Greenfield GPU control plane.
//!
//! Semantic FrameGraph decides meaning. This module owns the lower-level GPU
//! resource/execution graph and deliberately does not preserve the old
//! GpuScene/LayerWithPasses orchestration contract.

mod blend_backend;
mod blend_projection;
mod composite_backend;
mod content_backend;
mod contribution_backend;
mod effect_backend;
mod engine_backend;
mod executor;
mod graph;
mod history;
mod image_source;
mod lowerer;
mod logical_lowerer;
mod planner;
mod placement_backend;
mod projection_backend;
mod scene_root;
mod semantic_adapter;
mod snapshot_backend;
mod resource_store;
mod telemetry;
mod types;
mod visibility;

pub(crate) use blend_backend::ResidentBlend;
pub(crate) use blend_projection::GpuBlendProjectionResidency;
pub(crate) use composite_backend::ResidentCompositeLayer;
pub(crate) use content_backend::ResidentContent;
pub(crate) use engine_backend::{EngineCrossExecutor, EngineGpuBackend, EngineGpuBackendError, EngineGpuOperation, EngineCrossContext};
pub(crate) use effect_backend::{resident_effect_chain, ResidentEffectChain};
pub(crate) use executor::{GpuBackend, GpuExecuteError, GpuExecutionStats, GpuExecutor};
pub(crate) use history::{history_identity, GpuHistoryRegistry};
pub(crate) use image_source::{lower_image_source, snapshot_identity, snapshot_version, GpuImageSourceInput, GpuImageSourceResources};
pub(crate) use graph::{GpuGraphError, GpuGraphStats, GpuResourceDelta, GpuResourceGraph};
pub(crate) use logical_lowerer::{LogicalGpuLowerer, LogicalLowerError};
pub(crate) use lowerer::GpuLowerer;
pub(crate) use lowerer::{
    GpuContributionInput, GpuContributionResources, VersionedSemantic,
};
pub(crate) use resource_store::{GpuOperationTable, GpuResourceStore};
pub(crate) use snapshot_backend::ResidentSnapshot;
pub(crate) use scene_root::{lower_scene_root, lower_sink, GpuSceneRoot, GpuSinkKind, GpuSinkRoot};
pub(crate) use semantic_adapter::{SemanticAdapterError, SemanticGpuAdapter};
pub(crate) use placement_backend::{resident_placement, ResidentPlacement};
pub(crate) use projection_backend::ResidentProjection;
pub(crate) use planner::{
    GpuExecutionPlan, GpuPlanError, GpuPlanner, GpuResourceInterval,
};
pub(crate) use telemetry::{GpuFrameCounters, GpuTelemetry};
pub(crate) use types::{
    GpuAliasClass, GpuIdentitySource, GpuPassDesc, GpuPassIdentity, GpuPassKey, GpuPassKind,
    GpuPhysicalSlot, GpuResourceClass, GpuResourceDesc, GpuResourceIdentity, GpuResourceKey,
    GpuResourceLifetime, GpuResourceVersion,
};
