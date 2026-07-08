//! oc-gpu: wgpuコンテキスト・テクスチャ転送・YUV→RGB変換(M1-T3)。
//!
//! 性能方針(docs/performance-model.md): ピクセルはVRAM常駐が原則。
//! CPU⇔GPU転送はデコード直後のアップロードとテスト/書き出しのダウンロードのみに限る。

mod ctx;
mod transfer;
mod yuv;

pub use ctx::{GpuCtx, GpuError, GpuRuntimeError};
pub use transfer::{download_rgba, upload_rgba, RgbaDownloader, DEFAULT_DOWNLOAD_TIMEOUT};
pub use yuv::{solid_yuv420p, yuv_to_rgba_reference, ColorParams, YuvToRgba};
