//! VSM-A3-0: 外部 crate 作者が `motolii-plugin` façade だけで fullscreen_uniform16 へ到達できる。

use motolii_plugin::{GpuCtx, PipelineCache, PipelineCacheKey};

fn reach_fullscreen_uniform16(cache: &mut PipelineCache, gpu: &GpuCtx, key: PipelineCacheKey) {
    let _cached = cache.get_or_create_fullscreen_uniform16(gpu, key);
}

#[test]
fn external_crate_can_reach_fullscreen_uniform16() {
    // コンパイルが主な審判。GPU 起動や pixel 検証は不要。
    let _reach = reach_fullscreen_uniform16 as fn(&mut PipelineCache, &GpuCtx, PipelineCacheKey);
    let _ = std::mem::size_of_val(&_reach);
}
