//! oc-gpu: wgpuコンテキスト・テクスチャ転送・YUV→RGB変換(M1-T3)。
//!
//! 性能方針(docs/performance-model.md): ピクセルはVRAM常駐が原則。
//! CPU⇔GPU転送はデコード直後のアップロードとテスト/書き出しのダウンロードのみに限る。

mod ctx;
mod transfer;
mod yuv;

pub use ctx::{GpuCtx, GpuError};
pub use transfer::{download_rgba, upload_rgba};
pub use yuv::YuvToRgba;
