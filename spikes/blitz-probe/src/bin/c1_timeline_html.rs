//! C1: `crates/motolii-ui/src/timeline_blitz/` が出したHTMLを実際に描いて絵にする。
//!
//! 移植先のCSSがBlitzで本当に効いているかを確かめるためだけの器。
//! 判定材料はP4(offscreen.rs)と同じ経路で、HTMLだけを外から差し替える。
//!
//! 使い方: `cargo run --bin c1_timeline_html -- <input.html> <output.bmp> <W> <H>`
//!
//! 出力が BMP なのは blitz-probe に画像encoderの依存が無いため。
//! テクスチャformatは Rgba8Unorm。Srgbにすると合成で色が浮く。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::FxHashMap;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("usage: c1_timeline_html <input.html> <output.bmp> <W> <H>");
        std::process::exit(2);
    }
    let html = std::fs::read_to_string(&args[1]).expect("input html");
    let out_path = args[2].clone();
    let w: u32 = args[3].parse().expect("W");
    let h: u32 = args[4].parse().expect("H");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("c1-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .expect("no device");

    let mut doc = HtmlDocument::from_html(&html, DocumentConfig::default());
    doc.set_viewport(Viewport {
        window_size: (w, h),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("c1-timeline"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut renderer = Renderer::new(
        &device,
        &RenderTargetConfig {
            format: FORMAT,
            width: w,
            height: h,
        },
    );
    let mut resources = Resources::new();
    let mut scene = Scene::new(w as u16, h as u16);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("c1-enc"),
    });
    let device_handle = DeviceHandle {
        instance: instance.clone(),
        adapter: adapter.clone(),
        device: device.clone(),
        queue: queue.clone(),
    };
    {
        // 画像は使わないが、キャッシュはフレームを跨いで持つのが正。
        let mut cache = FxHashMap::default();
        let mut bindings = FxHashMap::default();
        let image_manager = ImageManager::new(
            &mut renderer,
            &mut resources,
            &device,
            &queue,
            &mut encoder,
            &mut cache,
        );
        let mut painter =
            VelloHybridScenePainter::new(&mut scene, image_manager, &mut bindings, &device_handle);
        paint_scene(&mut painter, &mut doc, 1.0, w, h, 0, 0);
    }
    renderer
        .render(
            &scene,
            &mut resources,
            &device,
            &queue,
            &mut encoder,
            &RenderSize {
                width: w,
                height: h,
            },
            &view,
            &TextureBindings::default(),
        )
        .expect("render");

    let bpr = (w * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("c1-readback"),
        size: (bpr * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let data = slice.get_mapped_range().to_vec();

    // 外周が透明のままか（罠(b)の回帰確認）。
    let at = |x: u32, y: u32| -> [u8; 4] {
        let i = (y * bpr + x * 4) as usize;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    };
    println!("C1 左上 = {:?}", at(0, 0));
    println!("C1 中央 = {:?}", at(w / 2, h / 2));

    write_bmp(&out_path, w, h, bpr, &data);
    println!("C1 出力: {out_path}");
}

/// 24bit BMP。encoder依存を足さないため手で書く。
fn write_bmp(path: &str, w: u32, h: u32, bpr: u32, data: &[u8]) {
    let row_bytes = (w * 3).next_multiple_of(4);
    let pixels = (row_bytes * h) as usize;
    let mut out = Vec::with_capacity(54 + pixels);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54u32 + pixels as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..6 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    // BMPは下から上。
    for y in (0..h).rev() {
        let mut written = 0;
        for x in 0..w {
            let i = (y * bpr + x * 4) as usize;
            out.push(data[i + 2]);
            out.push(data[i + 1]);
            out.push(data[i]);
            written += 3;
        }
        while written < row_bytes {
            out.push(0);
            written += 1;
        }
    }
    std::fs::write(path, out).expect("write bmp");
}
