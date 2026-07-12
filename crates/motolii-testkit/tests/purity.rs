//! INF-7f / M2E-9: 純関数契約。登録=検査対象への反転。

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime};
use motolii_eval::Value;
use motolii_gpu::{GpuCtx, PipelineCache};
use motolii_plugin::reference::{
    register_reference_plugins, CLEAR_FILTER, OPACITY_FILTER, SINE_PARAM_DRIVER, TINT_FILTER,
};
use motolii_plugin::{
    FilterPlugin, NodeDesc, ParamDriverContext, PluginError, PluginId, PluginKind, PluginRegistry,
    RenderCtx, ResolvedParams, TextureRef,
};
use motolii_testkit::purity::{
    assert_filter_pure, assert_param_driver_pure, assert_registry_pure, RegistryPurityProbe,
};
use motolii_testkit::{gpu_or_skip, TestkitError};

#[test]
fn clear_filter_is_pure() {
    let Some(gpu) = gpu_or_skip() else { return };
    let frame = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let input = vec![10u8; frame.data_size()];
    let mut params = ResolvedParams::new();
    params.insert("color", Value::Color([0.2, 0.4, 0.6, 1.0]));
    assert_filter_pure(
        "clear-pure",
        &gpu,
        &CLEAR_FILTER,
        RationalTime::ZERO,
        &params,
        frame,
        &input,
    )
    .unwrap();
}

#[test]
fn tint_filter_is_pure() {
    let Some(gpu) = gpu_or_skip() else { return };
    let frame = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let mut input = vec![0u8; frame.data_size()];
    for px in input.chunks_exact_mut(4) {
        px.copy_from_slice(&[128, 64, 32, 255]);
    }
    let mut params = ResolvedParams::new();
    params.insert("color", Value::Color([1.0, 0.5, 0.25, 1.0]));
    assert_filter_pure(
        "tint-pure",
        &gpu,
        &TINT_FILTER,
        RationalTime::from_seconds(1),
        &params,
        frame,
        &input,
    )
    .unwrap();
}

#[test]
fn opacity_filter_is_pure() {
    let Some(gpu) = gpu_or_skip() else { return };
    let frame = FrameDesc::packed(8, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let mut input = vec![0u8; frame.data_size()];
    for px in input.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 100, 50, 255]);
    }
    let mut params = ResolvedParams::new();
    params.insert("amount", Value::F64(0.5));
    assert_filter_pure(
        "opacity-pure",
        &gpu,
        &OPACITY_FILTER,
        RationalTime::ZERO,
        &params,
        frame,
        &input,
    )
    .unwrap();
}

#[test]
fn sine_param_driver_is_pure() {
    let mut params = ResolvedParams::new();
    params.insert("amplitude", Value::F64(1.0));
    params.insert("frequency_hz", Value::F64(2.0));
    params.insert("offset", Value::F64(0.0));
    assert_param_driver_pure(
        "sine-pure",
        &SINE_PARAM_DRIVER,
        ParamDriverContext {
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(1),
            sample_rate: Fps::try_new(8, 1).unwrap(),
        },
        &params,
    )
    .unwrap();
}

#[test]
fn reference_registry_is_pure() {
    let Some(gpu) = gpu_or_skip() else { return };
    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    // Filter×3 + ParamDriver + LayerSource + Composite が手書き列挙なしで検査される。
    assert!(registry.len(PluginKind::Filter) >= 1);
    assert!(registry.len(PluginKind::LayerSource) >= 1);
    assert!(registry.len(PluginKind::Composite) >= 1);
    assert!(registry.len(PluginKind::ParamDriver) >= 1);
    assert_registry_pure(&registry, &gpu, &RegistryPurityProbe::small()).unwrap();
}

/// レジストリに載せた瞬間に検査対象になること(opt-in列挙の抜けを許さない)。
#[test]
fn registering_stateful_plugin_fails_registry_purity() {
    let Some(gpu) = gpu_or_skip() else { return };

    struct StatefulClear;
    impl FilterPlugin for StatefulClear {
        fn desc(&self) -> &NodeDesc {
            static DESC: OnceLock<NodeDesc> = OnceLock::new();
            DESC.get_or_init(|| NodeDesc {
                id: PluginId("test.filter.stateful"),
                version: 1,
                display_name: "Stateful",
                category: "Utility",
                tags: &["test"],
                params: vec![],
                min_inputs: 1,
                max_inputs: 1,
            })
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            _pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            _params: &ResolvedParams,
            _input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            static CALLS: AtomicU32 = AtomicU32::new(0);
            let n = CALLS.fetch_add(1, Ordering::Relaxed);
            let g = if n == 0 { 0.0 } else { 1.0 };
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("stateful-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            Ok(())
        }
    }
    static STATEFUL: StatefulClear = StatefulClear;

    let mut registry = PluginRegistry::new();
    register_reference_plugins(&mut registry).unwrap();
    registry.register_filter(&STATEFUL).unwrap();

    let err = assert_registry_pure(&registry, &gpu, &RegistryPurityProbe::small()).unwrap_err();
    assert!(
        matches!(
            err,
            TestkitError::PurityViolation { ref label } if label == "test.filter.stateful"
        ),
        "{err:?}"
    );
}

/// 共有 uniform を `queue.write_buffer` で更新する状態付き Filter は偽陰性にならないこと。
/// 1 submit に2描画を積むと両パスが2回目の uniform を読み、違反を見逃せる(M2E-9 follow-up)。
#[test]
fn shared_uniform_stateful_filter_fails_purity_check() {
    let Some(gpu) = gpu_or_skip() else { return };

    const STATEFUL_UNIFORM_WGSL: &str = r#"
struct AmountUniform {
    amount: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<uniform> u: AmountUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
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
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let tin = textureSample(input_tex, tex_sampler, in.uv);
    return tin * u.amount.x;
}
"#;

    struct StatefulUniformFilter;
    impl FilterPlugin for StatefulUniformFilter {
        fn desc(&self) -> &NodeDesc {
            static DESC: OnceLock<NodeDesc> = OnceLock::new();
            DESC.get_or_init(|| NodeDesc {
                id: PluginId("test.filter.stateful_uniform"),
                version: 1,
                display_name: "Stateful Uniform",
                category: "Utility",
                tags: &["test"],
                params: vec![],
                min_inputs: 1,
                max_inputs: 1,
            })
        }

        fn render(
            &self,
            gpu: &GpuCtx,
            pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            _params: &ResolvedParams,
            input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            use motolii_gpu::PipelineCacheKey;

            static CALLS: AtomicU32 = AtomicU32::new(0);
            let n = CALLS.fetch_add(1, Ordering::Relaxed);
            // 1回目は黒(0)、2回目は入力そのまま(1) — submit を分ければ出力が食い違う。
            let amount = if n == 0 { 0.0f32 } else { 1.0 };
            let cached = pipelines.get_or_create_tex_sample_uniform4(
                gpu,
                PipelineCacheKey {
                    id: "test.filter.stateful_uniform",
                    wgsl: STATEFUL_UNIFORM_WGSL,
                },
            );
            let mut uniform_bytes = [0u8; 16];
            uniform_bytes[0..4].copy_from_slice(&amount.to_ne_bytes());
            gpu.queue
                .write_buffer(&cached.uniform_buffer, 0, &uniform_bytes);

            let input_view = input
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let output_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("stateful-uniform.bg"),
                layout: &cached.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&cached.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cached.uniform_buffer.as_entire_binding(),
                    },
                ],
            });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("stateful-uniform.pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    multiview_mask: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&cached.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            Ok(())
        }
    }
    static STATEFUL: StatefulUniformFilter = StatefulUniformFilter;

    let frame = FrameDesc::packed(4, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let mut input = vec![0u8; frame.data_size()];
    for px in input.chunks_exact_mut(4) {
        px.copy_from_slice(&[200, 100, 50, 255]);
    }
    let err = assert_filter_pure(
        "stateful-uniform-should-fail",
        &gpu,
        &STATEFUL,
        RationalTime::ZERO,
        &ResolvedParams::new(),
        frame,
        &input,
    )
    .unwrap_err();
    assert!(
        matches!(err, TestkitError::PurityViolation { .. }),
        "{err:?}"
    );
}

/// 隠れた呼び出し回数状態を持つ偽Filterは純関数検査で落ちる(検出器の負例)。
#[test]
fn stateful_filter_fails_purity_check() {
    let Some(gpu) = gpu_or_skip() else { return };

    struct StatefulClear;
    impl FilterPlugin for StatefulClear {
        fn desc(&self) -> &NodeDesc {
            static DESC: OnceLock<NodeDesc> = OnceLock::new();
            DESC.get_or_init(|| NodeDesc {
                id: PluginId("test.filter.stateful_direct"),
                version: 1,
                display_name: "Stateful Direct",
                category: "Utility",
                tags: &["test"],
                params: vec![],
                min_inputs: 1,
                max_inputs: 1,
            })
        }

        fn render(
            &self,
            _gpu: &GpuCtx,
            _pipelines: &mut PipelineCache,
            encoder: &mut wgpu::CommandEncoder,
            _ctx: &RenderCtx,
            _params: &ResolvedParams,
            _input: TextureRef<'_>,
            output: TextureRef<'_>,
        ) -> Result<(), PluginError> {
            static CALLS: AtomicU32 = AtomicU32::new(0);
            let n = CALLS.fetch_add(1, Ordering::Relaxed);
            let g = if n == 0 { 0.0 } else { 1.0 };
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("stateful-clear-direct"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            Ok(())
        }
    }
    static STATEFUL: StatefulClear = StatefulClear;

    let frame = FrameDesc::packed(4, 4, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let input = vec![0u8; frame.data_size()];
    let err = assert_filter_pure(
        "stateful-should-fail",
        &gpu,
        &STATEFUL,
        RationalTime::ZERO,
        &ResolvedParams::new(),
        frame,
        &input,
    )
    .unwrap_err();
    assert!(
        matches!(err, TestkitError::PurityViolation { .. }),
        "{err:?}"
    );
}
