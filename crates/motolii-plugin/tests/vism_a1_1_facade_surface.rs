//! VSM-A1-1: 外部crate作者が `motolii-plugin` façade だけで A1S §2.1 型へ到達できる。

use motolii_plugin::{
    bytemuck, wgpu, CompCamera, DataTrack, Fps, FrameDesc, GpuCtx, PipelineCache, PipelineCacheKey,
    Quality, RationalTime, Value,
};

#[test]
fn external_crate_can_reach_facade_surface() {
    // コンパイルが主な審判。GPU 起動や pixel 検証は不要。
    let _ = std::mem::size_of::<DataTrack>();
    let _ = std::mem::size_of::<Value>();
    let _ = std::mem::size_of::<GpuCtx>();
    let _ = std::mem::size_of::<PipelineCache>();
    let _ = std::mem::size_of::<PipelineCacheKey>();
    let _ = std::mem::size_of::<CompCamera>();
    let _ = std::mem::size_of::<Fps>();
    let _ = std::mem::size_of::<FrameDesc>();
    let _ = std::mem::size_of::<Quality>();
    let _ = std::mem::size_of::<RationalTime>();
    let _ = std::mem::size_of::<wgpu::Texture>();
    let uniform = [1.0f32, 0.0, 0.0, 0.0];
    let _bytes = bytemuck::bytes_of(&uniform);
}
