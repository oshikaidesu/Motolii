use skia_safe::{
    AlphaType, Color, ColorType, ImageInfo, Paint, PaintStyle, PathBuilder, Point, Rect, surfaces,
};
use std::time::{Duration, Instant};

const W: u32 = 3840;
const H: u32 = 2160;
const FRAMES: usize = 90;
const DIRTY_W: u32 = 2048;
const DIRTY_H: u32 = 1088;

fn percentile(v: &[Duration], p: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_unstable();
    s[((s.len() - 1) as f64 * p).round() as usize].as_secs_f64() * 1000.0
}

fn draw_overlay(bytes: &mut [u8], count: usize, frame: usize) {
    let info = ImageInfo::new(
        (W as i32, H as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface = surfaces::wrap_pixels(&info, bytes, Some(W as usize * 4), None).unwrap();
    let c = surface.canvas();
    c.clear(Color::TRANSPARENT);
    let mut p = Paint::default();
    p.set_anti_alias(true);

    // Adaptive composition grid and output frame.
    p.set_stroke_width(1.0);
    for x in (0..W).step_by(64) {
        p.set_color(if x % 256 == 0 {
            Color::from_argb(90, 110, 170, 225)
        } else {
            Color::from_argb(38, 110, 170, 225)
        });
        c.draw_line((x as f32, 0.0), (x as f32, H as f32), &p);
    }
    for y in (0..H).step_by(64) {
        p.set_color(if y % 256 == 0 {
            Color::from_argb(90, 110, 170, 225)
        } else {
            Color::from_argb(38, 110, 170, 225)
        });
        c.draw_line((0.0, y as f32), (W as f32, y as f32), &p);
    }
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(3.0);
    p.set_color(Color::from_argb(210, 238, 205, 90));
    c.draw_rect(Rect::from_xywh(260.0, 150.0, 3320.0, 1860.0), &p);

    let drag = (frame as f32 * 0.7) % 48.0;
    let cols = 25usize;
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        let x = 300.0 + col as f32 * 132.0 + if i % 13 == 0 { drag } else { 0.0 };
        let y = 190.0 + row as f32 * 86.0;
        let w = 72.0 + (i % 5) as f32 * 7.0;
        let h = 42.0 + (i % 4) as f32 * 6.0;
        let rect = Rect::from_xywh(x, y, w, h);
        p.set_style(PaintStyle::Stroke);
        p.set_stroke_width(if i % 13 == 0 { 2.0 } else { 1.0 });
        p.set_color(if i % 13 == 0 {
            Color::from_argb(235, 255, 206, 75)
        } else {
            Color::from_argb(145, 120, 205, 245)
        });
        c.draw_rect(rect, &p);

        // Eight scale handles plus rotate handle.
        let handles = [
            (rect.left, rect.top),
            (rect.center_x(), rect.top),
            (rect.right, rect.top),
            (rect.left, rect.center_y()),
            (rect.right, rect.center_y()),
            (rect.left, rect.bottom),
            (rect.center_x(), rect.bottom),
            (rect.right, rect.bottom),
        ];
        p.set_style(PaintStyle::Fill);
        p.set_color(Color::from_argb(230, 235, 238, 242));
        for &(hx, hy) in &handles {
            c.draw_circle((hx, hy), 3.5, &p);
        }
        let rotate = Point::new(rect.center_x(), rect.top - 18.0);
        c.draw_circle(rotate, 4.0, &p);
        p.set_style(PaintStyle::Stroke);
        c.draw_line((rect.center_x(), rect.top), rotate, &p);

        // Representative editable motion/path curve.
        let mut path = PathBuilder::new();
        path.move_to((rect.left, rect.center_y()));
        path.cubic_to(
            (rect.left + w * 0.25, rect.top - 20.0),
            (rect.left + w * 0.75, rect.bottom + 20.0),
            (rect.right, rect.center_y()),
        );
        p.set_color(Color::from_argb(155, 238, 120, 190));
        p.set_stroke_width(1.25);
        c.draw_path(&path.detach(), &p);
    }

    // Magnet/snap guides for actively dragged objects.
    p.set_color(Color::from_argb(230, 255, 92, 105));
    p.set_stroke_width(2.0);
    c.draw_line((W as f32 * 0.5, 0.0), (W as f32 * 0.5, H as f32), &p);
    c.draw_line((0.0, H as f32 * 0.5), (W as f32, H as f32 * 0.5), &p);
}

fn draw_layered_dirty(bytes: &mut [u8], count: usize, frame: usize) {
    let info = ImageInfo::new(
        (DIRTY_W as i32, DIRTY_H as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface =
        surfaces::wrap_pixels(&info, bytes, Some(DIRTY_W as usize * 4), None).unwrap();
    let c = surface.canvas();
    c.clear(Color::TRANSPARENT);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_style(PaintStyle::Stroke);
    let dx = (frame as f32 * 0.8) % 32.0;
    // Cached static grid is absent here. During group drag only lightweight object bounds move.
    for i in 0..count {
        let col = i % 25;
        let row = i / 25;
        let rect = Rect::from_xywh(
            24.0 + col as f32 * 79.0 + dx,
            28.0 + row as f32 * 50.0,
            55.0,
            30.0,
        );
        p.set_color(Color::from_argb(145, 115, 202, 244));
        p.set_stroke_width(1.0);
        c.draw_rect(rect, &p);
    }
    // One group gizmo, eight handles, rotate handle, and active magnet guides.
    let group = Rect::from_xywh(18.0 + dx, 18.0, 1980.0, 1020.0);
    p.set_color(Color::from_argb(240, 255, 206, 75));
    p.set_stroke_width(2.0);
    c.draw_rect(group, &p);
    p.set_style(PaintStyle::Fill);
    for &(x, y) in &[
        (group.left, group.top),
        (group.center_x(), group.top),
        (group.right, group.top),
        (group.left, group.center_y()),
        (group.right, group.center_y()),
        (group.left, group.bottom),
        (group.center_x(), group.bottom),
        (group.right, group.bottom),
    ] {
        c.draw_circle((x, y), 4.0, &p);
    }
    p.set_style(PaintStyle::Stroke);
    p.set_color(Color::from_argb(240, 255, 92, 105));
    c.draw_line(
        (DIRTY_W as f32 / 2.0, 0.0),
        (DIRTY_W as f32 / 2.0, DIRTY_H as f32),
        &p,
    );
    c.draw_line(
        (0.0, DIRTY_H as f32 / 2.0),
        (DIRTY_W as f32, DIRTY_H as f32 / 2.0),
        &p,
    );
}

fn main() {
    let count: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "500".into())
        .parse()
        .unwrap();
    let layered = std::env::args().nth(2).as_deref() == Some("layered");
    let (upload_w, upload_h) = if layered { (DIRTY_W, DIRTY_H) } else { (W, H) };
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("GPU adapter");
    let info = adapter.get_info();
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
    let overlay = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("skia-overlay"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let output = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("stage-output"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = overlay.create_view(&Default::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(r#"
        @group(0) @binding(0) var tex: texture_2d<f32>; @group(0) @binding(1) var samp: sampler;
        struct O { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
        @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.pos=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
        @fragment fn fs(i:O)->@location(0) vec4<f32> { let a=textureSample(tex,samp,i.uv); let base=vec3(.055+.14*i.uv.x,.06+.10*i.uv.y,.075+.08*i.uv.x); return vec4(a.rgb + base*(1.-a.a),1.); }
    "#.into()) });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });
    let pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
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
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let mut pixels = vec![0u8; upload_w as usize * upload_h as usize * 4];
    let mut raster = Vec::new();
    let mut upload = Vec::new();
    let total = Instant::now();
    for frame in 0..FRAMES {
        let t = Instant::now();
        if layered {
            draw_layered_dirty(&mut pixels, count, frame)
        } else {
            draw_overlay(&mut pixels, count, frame)
        };
        raster.push(t.elapsed());
        let t = Instant::now();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &overlay,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(upload_w * 4),
                rows_per_image: Some(upload_h),
            },
            wgpu::Extent3d {
                width: upload_w,
                height: upload_h,
                depth_or_array_layers: 1,
            },
        );
        upload.push(t.elapsed());
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let ov = output.create_view(&Default::default());
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ov,
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
            pass.set_pipeline(&pipe);
            pass.set_bind_group(0, &bg, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit([enc.finish()]);
    }
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let elapsed = total.elapsed().as_secs_f64();
    println!(
        "adapter={} backend={:?} mode={} 4K objects={} frames={} raster_p50={:.2}ms raster_p95={:.2}ms upload_call_p95={:.2}ms completed={:.1}fps total={:.2}s bytes/frame={:.1}MiB",
        info.name,
        info.backend,
        if layered { "layered-dirty" } else { "full" },
        count,
        FRAMES,
        percentile(&raster, 0.5),
        percentile(&raster, 0.95),
        percentile(&upload, 0.95),
        FRAMES as f64 / elapsed,
        elapsed,
        pixels.len() as f64 / 1048576.0
    );
}
