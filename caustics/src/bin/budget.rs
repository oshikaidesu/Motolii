//! Fixed-cost probe: what does one empty render/compute pass cost on this GPU, and what does a
//! full-screen additive RGBA16F overdraw cost? Measured with GPU timestamps, median of many runs.
use std::borrow::Cow;

const W: u32 = 1920;
const H: u32 = 1080;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    period_ms: f64,
}

fn color_target(g: &Gpu, w: u32, h: u32) -> wgpu::TextureView {
    g.device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

fn depth_target(g: &Gpu, w: u32, h: u32) -> wgpu::TextureView {
    g.device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

/// Runs `record(enc, qs)` which must write timestamps at indices 0..2*passes; returns median ms per pass.
fn measure(g: &Gpu, passes: u32, mut record: impl FnMut(&mut wgpu::CommandEncoder, &wgpu::QuerySet)) -> f64 {
    let count = passes * 2;
    let qs = g.device.create_query_set(&wgpu::QuerySetDescriptor { label: None, ty: wgpu::QueryType::Timestamp, count });
    let resolve = g.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: count as u64 * 8,
        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read = g.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: count as u64 * 8,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut per_pass: Vec<f64> = Vec::new();
    for run in 0..31 {
        let mut enc = g.device.create_command_encoder(&Default::default());
        record(&mut enc, &qs);
        enc.resolve_query_set(&qs, 0..count, &resolve, 0);
        enc.copy_buffer_to_buffer(&resolve, 0, &read, 0, count as u64 * 8);
        g.queue.submit([enc.finish()]);
        let slice = read.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
        g.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        {
            let data = slice.get_mapped_range();
            let t: &[u64] = bytemuck::cast_slice(&data);
            if run > 0 {
                let total: f64 = (0..passes as usize).map(|i| t[i * 2 + 1].saturating_sub(t[i * 2]) as f64).sum();
                per_pass.push(total * g.period_ms / passes as f64);
            }
        }
        read.unmap();
    }
    per_pass.sort_by(|a, b| a.partial_cmp(b).unwrap());
    per_pass[per_pass.len() / 2]
}


/// Amortised wall time: K identical passes in ONE submission, wait for the queue, divide by K.
fn amortised(g: &Gpu, k: u32, mut record: impl FnMut(&mut wgpu::CommandEncoder)) -> f64 {
    let mut runs: Vec<f64> = Vec::new();
    for _ in 0..15 {
        let mut enc = g.device.create_command_encoder(&Default::default());
        for _ in 0..k {
            record(&mut enc);
        }
        let t0 = std::time::Instant::now();
        g.queue.submit([enc.finish()]);
        g.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        runs.push(t0.elapsed().as_secs_f64() * 1000.0 / k as f64);
    }
    runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    runs[runs.len() / 2]
}

fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).expect("adapter");
    println!("adapter {} ({:?})", adapter.get_info().name, adapter.get_info().backend);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        ..Default::default()
    }))
    .expect("device");
    let period_ms = queue.get_timestamp_period() as f64 / 1.0e6;
    let g = Gpu { device, queue, period_ms };

    let small_c = color_target(&g, 256, 256);
    let small_d = depth_target(&g, 256, 256);
    let big_c = color_target(&g, W, H);

    // A: empty 256x256 colour+depth pass (clear and store, no draw) -> fixed cost of a capture pass
    for n in [1u32, 4, 8] {
        let ms = measure(&g, n, |enc, qs| {
            for i in 0..n {
                let _p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &small_c,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &small_d,
                        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                        query_set: qs,
                        beginning_of_pass_write_index: Some(i * 2),
                        end_of_pass_write_index: Some(i * 2 + 1),
                    }),
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
        });
        println!("A  empty 256x256 colour+depth render pass, {n} per frame : {ms:.3} ms each");
    }

    // B: empty 1920x1080 RGBA16F clear pass
    for n in [1u32, 4] {
        let ms = measure(&g, n, |enc, qs| {
            for i in 0..n {
                let _p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &big_c,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                        query_set: qs,
                        beginning_of_pass_write_index: Some(i * 2),
                        end_of_pass_write_index: Some(i * 2 + 1),
                    }),
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
        });
        println!("B  empty 1920x1080 RGBA16F clear pass, {n} per frame      : {ms:.3} ms each");
    }

    // C: trivial compute pass
    let cs = g.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed("@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3<u32>) {}")),
    });
    let cp = g.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    for n in [1u32, 4, 8] {
        let ms = measure(&g, n, |enc, qs| {
            for i in 0..n {
                let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                        query_set: qs,
                        beginning_of_pass_write_index: Some(i * 2),
                        end_of_pass_write_index: Some(i * 2 + 1),
                    }),
                });
                p.set_pipeline(&cp);
                p.dispatch_workgroups(1, 1, 1);
            }
        });
        println!("C  trivial compute pass, {n} per frame                      : {ms:.3} ms each");
    }

    // D: full-screen additive blend into RGBA16F, N layers of overdraw in one pass
    let fs = g.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
            "@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {\n  let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));\n  return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);\n}\n@fragment fn fs() -> @location(0) vec4<f32> { return vec4<f32>(0.01, 0.01, 0.01, 0.0); }",
        )),
    });
    let additive = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    let rp = g.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState { module: &fs, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &fs,
            entry_point: Some("fs"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba16Float,
                blend: Some(wgpu::BlendState { color: additive, alpha: additive }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    for layers in [1u32, 4, 16] {
        let ms = measure(&g, 1, |enc, qs| {
            let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &big_c,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                    query_set: qs,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&rp);
            for _ in 0..layers {
                p.draw(0..3, 0..1);
            }
        });
        println!("D  1080p additive RGBA16F full-screen overdraw x{layers:<2}          : {ms:.3} ms");
    }

    println!("--- amortised wall time (200 passes in one submission / 200) ---");
    let k = 200u32;
    let clear_small = |enc: &mut wgpu::CommandEncoder| {
        let _p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &small_c,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &small_d,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    };
    println!("A' 256x256 colour+depth clear pass      : {:.4} ms each", amortised(&g, k, clear_small));
    let clear_big = |enc: &mut wgpu::CommandEncoder| {
        let _p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &big_c,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    };
    println!("B' 1920x1080 RGBA16F clear pass         : {:.4} ms each", amortised(&g, k, clear_big));
    println!("C' trivial compute pass                 : {:.4} ms each", amortised(&g, k, |enc| {
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None, timestamp_writes: None });
        p.set_pipeline(&cp);
        p.dispatch_workgroups(1, 1, 1);
    }));
    for layers in [1u32, 4, 16] {
        let ms = amortised(&g, 20, |enc| {
            let mut p = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &big_c,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_pipeline(&rp);
            for _ in 0..layers {
                p.draw(0..3, 0..1);
            }
        });
        println!("D' 1080p additive RGBA16F overdraw x{layers:<2}      : {ms:.4} ms per pass");
    }
}
