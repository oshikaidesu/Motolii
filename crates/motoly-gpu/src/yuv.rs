use motoly_core::{ColorSpace, CpuFrame, FrameDesc, PixelFormat};

use crate::GpuCtx;

/// YUV変換の係数・レンジ(FrameDesc.color_spaceから導出)。
/// シェーダとCPU参照実装が同じ値を共有する(落とし穴B-3対策)。
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorParams {
    pub y_off: f32,
    pub y_scale: f32,
    pub c_scale: f32,
    pub _pad: f32,
    pub crv: f32,
    pub cbu: f32,
    pub cgu: f32,
    pub cgv: f32,
}

impl ColorParams {
    /// 対応外の色空間(RGB系タグ)はNone。
    pub fn for_color_space(cs: ColorSpace) -> Option<Self> {
        // 係数: BT.709 kr=0.2126 kb=0.0722 / BT.601 kr=0.299 kb=0.114
        let (crv, cbu, cgu, cgv) = match cs {
            ColorSpace::Rec709Limited | ColorSpace::Rec709Full => {
                (1.5748, 1.8556, -0.187_324, -0.468_124)
            }
            ColorSpace::Rec601Limited => (1.402, 1.772, -0.344_136, -0.714_136),
            ColorSpace::LinearRgb | ColorSpace::Srgb => return None,
        };
        let (y_off, y_scale, c_scale) = match cs {
            ColorSpace::Rec709Full => (0.0, 1.0 / 255.0, 1.0 / 255.0),
            _ => (16.0, 1.0 / 219.0, 1.0 / 224.0),
        };
        Some(Self {
            y_off,
            y_scale,
            c_scale,
            _pad: 0.0,
            crv,
            cbu,
            cgu,
            cgv,
        })
    }
}

/// YUV420p → ガンマ保持RGBA8 変換パイプライン(M1-T3)。
/// wgpuにはネイティブYUVサンプラが無いためY/U/Vを別テクスチャに載せ、
/// fragmentシェーダで合成する。係数・レンジはFrameDesc.color_spaceに従う。
pub struct YuvToRgba {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    chroma_sampler: wgpu::Sampler,
    pool: Option<SizePool>,
}

/// 寸法ごとに使い回すGPUリソース一式(第3回レビュー#1: 毎フレーム確保の排除)。
/// performance-model 原則3「確保・解放を毎フレームやらない」の実装。
/// 出力は2枚のピンポン: 呼び出し側(UI表示)が1枚を保持している間に次を書ける。
/// 返したテクスチャを2回以上のconvertを跨いで保持する用途は想定しない。
struct SizePool {
    w: u32,
    h: u32,
    y_tex: wgpu::Texture,
    u_tex: wgpu::Texture,
    v_tex: wgpu::Texture,
    param_buf: wgpu::Buffer,
    bind: wgpu::BindGroup,
    outputs: [wgpu::Texture; 2],
    out_views: [wgpu::TextureView; 2],
    next: usize,
}

impl YuvToRgba {
    pub fn new(gpu: &GpuCtx) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("yuv"),
                source: wgpu::ShaderSource::Wgsl(include_str!("yuv.wgsl").into()),
            });
        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = (0..3)
            .map(|i| wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    // クロマのバイリニアサンプルのためfilterable(R8Unormは常時対応)
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            })
            .collect();
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        let layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("yuv-bgl"),
                entries: &entries,
            });
        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("yuv-pl"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
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
                multiview_mask: None,
                cache: None,
            });
        let chroma_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("yuv-chroma"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            chroma_sampler,
            pool: None,
        }
    }

    /// YUV420pのCpuFrameを変換し、ガンマ保持RGBA8テクスチャを返す。
    /// 係数はframe.desc.color_spaceから選択される。
    /// リソースは寸法別プールを使い回し、毎フレームの確保は行わない。
    pub fn convert(&mut self, gpu: &GpuCtx, frame: &CpuFrame) -> wgpu::Texture {
        assert_eq!(
            frame.desc.format,
            PixelFormat::Yuv420p,
            "YuvToRgba::convert expects Yuv420p"
        );
        let params = ColorParams::for_color_space(frame.desc.color_space)
            .unwrap_or_else(|| panic!("unsupported YUV color space {:?}", frame.desc.color_space));

        let (w, h) = (frame.desc.width, frame.desc.height);
        let (cw, ch) = (w / 2, h / 2);
        let y_size = (w * h) as usize;
        let c_size = (cw * ch) as usize;

        if self.pool.as_ref().map(|p| (p.w, p.h)) != Some((w, h)) {
            self.pool = Some(self.build_pool(gpu, w, h));
        }
        let pool = self.pool.as_mut().expect("pool built above");

        // 中身だけ更新(確保しない)
        write_plane(gpu, &pool.y_tex, w, h, &frame.data[..y_size]);
        write_plane(
            gpu,
            &pool.u_tex,
            cw,
            ch,
            &frame.data[y_size..y_size + c_size],
        );
        write_plane(
            gpu,
            &pool.v_tex,
            cw,
            ch,
            &frame.data[y_size + c_size..y_size + 2 * c_size],
        );
        gpu.queue
            .write_buffer(&pool.param_buf, 0, bytemuck::bytes_of(&params));

        let idx = pool.next;
        pool.next = (idx + 1) % 2;

        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("yuv-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pool.out_views[idx],
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
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &pool.bind, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([enc.finish()]);
        pool.outputs[idx].clone()
    }

    /// 寸法別リソースの一括確保(サイズ変更時のみ呼ばれる)
    fn build_pool(&self, gpu: &GpuCtx, w: u32, h: u32) -> SizePool {
        let (cw, ch) = (w / 2, h / 2);
        let y_tex = create_plane(gpu, w, h);
        let u_tex = create_plane(gpu, cw, ch);
        let v_tex = create_plane(gpu, cw, ch);

        let param_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("yuv-params"),
            size: std::mem::size_of::<ColorParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let make_out = || {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
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
                // TEXTURE_BINDING+RENDER_ATTACHMENT: Slint Image::try_fromの必須要件
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let outputs = [make_out(), make_out()];
        let out_views = [
            outputs[0].create_view(&Default::default()),
            outputs[1].create_view(&Default::default()),
        ];

        let views = [
            y_tex.create_view(&Default::default()),
            u_tex.create_view(&Default::default()),
            v_tex.create_view(&Default::default()),
        ];
        let mut bind_entries: Vec<wgpu::BindGroupEntry> = views
            .iter()
            .enumerate()
            .map(|(i, v)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::TextureView(v),
            })
            .collect();
        bind_entries.push(wgpu::BindGroupEntry {
            binding: 3,
            resource: param_buf.as_entire_binding(),
        });
        bind_entries.push(wgpu::BindGroupEntry {
            binding: 4,
            resource: wgpu::BindingResource::Sampler(&self.chroma_sampler),
        });
        let bind = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yuv-bind"),
            layout: &self.layout,
            entries: &bind_entries,
        });

        SizePool {
            w,
            h,
            y_tex,
            u_tex,
            v_tex,
            param_buf,
            bind,
            outputs,
            out_views,
            next: 0,
        }
    }
}

fn create_plane(gpu: &GpuCtx, w: u32, h: u32) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
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
    })
}

fn write_plane(gpu: &GpuCtx, tex: &wgpu::Texture, w: u32, h: u32, plane: &[u8]) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
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
}

/// テクセル中心=+0.5の連続座標でのバイリニアサンプル(ClampToEdge)。
/// シェーダのsampler挙動を再現する。
fn sample_bilinear(plane: &[u8], w: usize, h: usize, pos_x: f32, pos_y: f32) -> f32 {
    let tx = pos_x - 0.5;
    let ty = pos_y - 0.5;
    let x0 = tx.floor();
    let y0 = ty.floor();
    let fx = tx - x0;
    let fy = ty - y0;
    let xi = |x: f32| (x.max(0.0) as usize).min(w - 1);
    let yi = |y: f32| (y.max(0.0) as usize).min(h - 1);
    let p = |x: usize, y: usize| plane[y * w + x] as f32;
    let (x0u, x1u) = (xi(x0), xi(x0 + 1.0));
    let (y0u, y1u) = (yi(y0), yi(y0 + 1.0));
    let top = p(x0u, y0u) * (1.0 - fx) + p(x1u, y0u) * fx;
    let bot = p(x0u, y1u) * (1.0 - fx) + p(x1u, y1u) * fx;
    top * (1.0 - fy) + bot * fy
}

/// CPU参照実装: シェーダと同一式・同一係数でYUV420pをRGBAへ変換する。
/// ゴールデンテストの理論値として使う(GPUと数値一致すべき)。
pub fn yuv_to_rgba_reference(frame: &CpuFrame) -> Vec<u8> {
    assert_eq!(frame.desc.format, PixelFormat::Yuv420p);
    let p = ColorParams::for_color_space(frame.desc.color_space).expect("yuv color space");
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
            // シェーダと同一のsiting位置(水平=左cosited、垂直=中間)でバイリニア
            let pos_x = col as f32 * 0.5 + 0.5;
            let pos_y = row as f32 * 0.5 + 0.25;
            let uv = sample_bilinear(u_plane, cw, ch, pos_x, pos_y);
            let vv = sample_bilinear(v_plane, cw, ch, pos_x, pos_y);

            let yl = (yv - p.y_off) * p.y_scale;
            let cb = (uv - 128.0) * p.c_scale;
            let cr = (vv - 128.0) * p.c_scale;
            let r = yl + p.crv * cr;
            let g = yl + p.cgu * cb + p.cgv * cr;
            let b = yl + p.cbu * cb;

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
pub fn solid_yuv420p(w: u32, h: u32, y: u8, u: u8, v: u8, cs: ColorSpace) -> CpuFrame {
    let desc = FrameDesc::yuv(w, h, PixelFormat::Yuv420p, cs);
    let mut data = vec![0u8; desc.data_size()];
    let y_size = (w * h) as usize;
    let c_size = ((w / 2) * (h / 2)) as usize;
    data[..y_size].fill(y);
    data[y_size..y_size + c_size].fill(u);
    data[y_size + c_size..y_size + 2 * c_size].fill(v);
    CpuFrame::new(desc, motoly_core::RationalTime::ZERO, data)
}
