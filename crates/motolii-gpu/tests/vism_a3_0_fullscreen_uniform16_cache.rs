//! VSM-A3-0: 0-input LayerSource 向け fullscreen_uniform16 定型のレイアウトと cache 審判。

use motolii_gpu::{PipelineCache, PipelineCacheKey};
use motolii_testkit::gpu_or_skip;

const TEST_WGSL: &str = r#"
struct Params {
    data: array<vec4<f32>, 4>,
};

@group(0) @binding(0) var<uniform> params: Params;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(params.data[0].xyz, 1.0);
}
"#;

#[test]
fn fullscreen_uniform16_layout_is_binding0_only_64_bytes() {
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let key = PipelineCacheKey {
        id: "vism.a3_0.layout",
        wgsl: TEST_WGSL,
    };
    let mut cache = PipelineCache::new();
    let cached = cache.get_or_create_fullscreen_uniform16(&gpu, key);

    assert_eq!(cached.uniform_buffer.size(), 64);
    assert!(cached
        .uniform_buffer
        .usage()
        .contains(wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST));

    // binding 0 のみの bind group が成立することで texture/sampler 無しを固定する。
    let _bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("vism.a3_0.layout"),
        layout: &cached.bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: cached.uniform_buffer.as_entire_binding(),
        }],
    });
}

#[test]
fn fullscreen_uniform16_reuses_entry_on_same_key() {
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let key = PipelineCacheKey {
        id: "vism.a3_0.cache",
        wgsl: TEST_WGSL,
    };
    let mut cache = PipelineCache::new();

    let first = cache.get_or_create_fullscreen_uniform16(&gpu, key.clone());
    let first_pipeline = &first.pipeline as *const wgpu::RenderPipeline;
    let first_buffer = &first.uniform_buffer as *const wgpu::Buffer;

    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.hits(), 0);

    let (same_pipeline, same_buffer) = {
        let second = cache.get_or_create_fullscreen_uniform16(&gpu, key);
        (
            std::ptr::eq(&second.pipeline, first_pipeline),
            std::ptr::eq(&second.uniform_buffer, first_buffer),
        )
    };
    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.hits(), 1);
    assert!(same_pipeline);
    assert!(same_buffer);
}
