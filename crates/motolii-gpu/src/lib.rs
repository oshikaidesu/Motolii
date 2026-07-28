//! motolii-gpu: wgpuコンテキスト・テクスチャ転送・YUV→RGB変換(M1-T3)。
//!
//! 性能方針(docs/performance-model.md): ピクセルはVRAM常駐が原則。
//! CPU⇔GPU転送はデコード直後のアップロードとテスト/書き出しのダウンロードのみに限る。

mod allocation;
mod ctx;
mod pipeline_cache;
mod resource_ledger;
mod transfer;
mod yuv;

pub use allocation::{
    estimate_buffer_bytes, estimate_texture_bytes, AllocationEstimateError,
    TextureAllocationAlignment,
};
pub use ctx::{drs_available, optional_features, GpuCtx, GpuError, GpuOrigin, GpuRuntimeError};
pub use pipeline_cache::{
    CachedFullscreenUniform16, CachedTexSampleUniform4, PipelineCache, PipelineCacheKey,
};
pub use resource_ledger::{
    BudgetCap, MemoryTier, OwnerUsage, ResourceAdmissionError, ResourceBudgets, ResourceGrant,
    ResourceLedger, ResourceOwner, ResourceRequest, ResourceSnapshot, TierUsage,
};
pub use transfer::{download_rgba, upload_rgba, RgbaDownloader, DEFAULT_DOWNLOAD_TIMEOUT};
pub use yuv::{solid_yuv420p, ColorParams, YuvError, YuvToRgba};
