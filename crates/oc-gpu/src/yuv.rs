use oc_core::{ColorSpace, CpuFrame, FrameDesc, PixelFormat};

use crate::GpuCtx;

/// YUV420p(Rec.709 limited)→ sRGB RGBA8 変換パイプライン(M1-T3)。
/// wgpuにはネイティブYUVサンプラが無いためY/U/Vを別テクスチャに載せ、
/// fragmentシェーダで合成する(compute+storage textureより互換性が高い)。
pub struct YuvToRgba {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl YuvToRgba {
    pub fn new(gpu: &GpuCtx) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("yuv"),
                source: wgpu::ShaderSource::Wgsl(include_str!("yuv.wgsl").into()),
            });
        let layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("yuv-bgl"),
                entries: &(0..3)
                    .map(|i| wgpu::BindGroupLayoutEntry {
                        binding: i,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>(),
            });
        let pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("yuv-pl"),
                    bind_group_layouts: &[&layout],
                    push_constant_ranges: &[],
                });
        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("yuv-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        Self { pipeline, layout }
    }

    /// YUV420pのCpuFrameを変換し、sRGB RGBA8テクスチャを返す。
    pub fn convert(&self, gpu: &GpuCtx, frame: &CpuFrame) -> wgpu::Texture {
        assert_eq!(
            frame.desc.format,
            PixelFormat::Yuv420p,
            "YuvToRgba::convert expects Yuv420p"
        );
        let (w, h) = (frame.desc.width, frame.desc.height);
        let (cw, ch) = (w / 2, h / 2);
        let y_size = (w * h) as usize;
        let c_size = (cw * ch) as usize;

        let y_plane = &frame.data[..y_size];
        let u_plane = &frame.data[y_size..y_size + c_size];
        let v_plane = &frame.data[y_size + c_size..y_size + 2 * c_size];

        let y_tex = self.upload_plane(gpu, w, h, y_plane);
        let u_tex = self.upload_plane(gpu, cw, ch, u_plane);
        let v_tex = self.upload_plane(gpu, cw, ch, v_plane);

        let out = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("yuv-out"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let out_view = out.create_view(&Default::default());

        let views: Vec<wgpu::TextureView> = [&y_tex, &u_tex, &v_tex]
            .iter()
            .map(|t| t.create_view(&Default::default()))
            .collect();
        let bind = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yuv-bind"),
            layout: &self.layout,
            entries: &(0..3)
                .map(|i| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: wgpu::BindingResource::TextureView(&views[i]),
                })
                .collect::<Vec<_>>(),
        });

        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("yuv-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &out_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([enc.finish()]);
        out
    }

    fn upload_plane(&self, gpu: &GpuCtx, w: u32, h: u32, plane: &[u8]) -> wgpu::Texture {
        let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("yuv-plane"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            plane,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        tex
    }
}

/// CPU参照実装: シェーダと同一式でYUV420p1ピクセルをsRGB RGBAへ変換する。
/// ゴールデンテストの理論値として使う(GPUと数値一致すべき)。
pub fn yuv_to_rgba_reference(frame: &CpuFrame) -> Vec<u8> {
    assert_eq!(frame.desc.format, PixelFormat::Yuv420p);
    let (w, h) = (frame.desc.width as usize, frame.desc.height as usize);
    let (cw, ch) = (w / 2, h / 2);
    let y_size = w * h;
    let c_size = cw * ch;
    let y_plane = &frame.data[..y_size];
    let u_plane = &frame.data[y_size..y_size + c_size];
    let v_plane = &frame.data[y_size + c_size..y_size + 2 * c_size];

    let mut out = vec![0u8; w * h * 4];
    for row in 0..h {
        for col in 0..w {
            let yv = y_plane[row * w + col] as f32;
            let cidx = (row / 2) * cw + (col / 2);
            let uv = u_plane[cidx] as f32;
            let vv = v_plane[cidx] as f32;

            let yl = (yv - 16.0) / 219.0;
            let cb = (uv - 128.0) / 224.0;
            let cr = (vv - 128.0) / 224.0;
            let r = yl + 1.5748 * cr;
            let g = yl - 0.1873 * cb - 0.4681 * cr;
            let b = yl + 1.8556 * cb;

            let o = (row * w + col) * 4;
            out[o] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
            out[o + 1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
            out[o + 2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
            out[o + 3] = 255;
        }
    }
    out
}

/// テスト・ユーティリティ: べた塗りYUV420pフレームを作る。
pub fn solid_yuv420p(w: u32, h: u32, y: u8, u: u8, v: u8) -> CpuFrame {
    let desc = FrameDesc::yuv(w, h, PixelFormat::Yuv420p, ColorSpace::Rec709Limited);
    let mut data = vec![0u8; desc.data_size()];
    let y_size = (w * h) as usize;
    let c_size = ((w / 2) * (h / 2)) as usize;
    data[..y_size].fill(y);
    data[y_size..y_size + c_size].fill(u);
    data[y_size + c_size..y_size + 2 * c_size].fill(v);
    CpuFrame::new(desc, oc_core::RationalTime::ZERO, data)
}
