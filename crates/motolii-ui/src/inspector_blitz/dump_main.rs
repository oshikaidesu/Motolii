//! C7 の判定材料: `inspector_blitz` が出したHTMLを実際に描いてPNGにする。
//!
//! ```text
//! cargo run -p motolii-ui --bin motolii-inspector-blitz-dump -- <out.png> [W] [H]
//! ```
//!
//! 経路は `spikes/blitz-probe/src/bin/c1_timeline_html.rs` と同じ(P4 offscreen)。
//! 違いはHTMLの出所が `inspector_blitz::inspector_html` であることと、
//! BMPではなくPNGを書くことだけ。
//!
//! 実測済みの罠:
//! - テクスチャformatは `Rgba8Unorm`。`Rgba8UnormSrgb` にすると色が浮く。
//! - 出力はプリマルチプライドアルファ。PNG化の前に unpremultiply する。
//! - `blitz_net::Provider` は Tokio reactor を要求する。ここは net を使わないが、
//!   同じ器の中で後から差し替えても壊れないよう `runtime.enter()` の中で回す。
//! - PNGのencoder依存は足さない(ALLOWLIST外)。deflateの **stored** ブロックで書く。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use motolii_ui::inspector_blitz::{inspector_html, SAMPLE};
use rustc_hash::FxHashMap;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: motolii-inspector-blitz-dump <out.png> [W] [H]");
        std::process::exit(2);
    }
    let out_path = args[1].clone();
    let w: u32 = args.get(2).map_or(340, |v| v.parse().expect("W"));
    let h: u32 = args.get(3).map_or(760, |v| v.parse().expect("H"));

    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _guard = runtime.enter();

    let html = inspector_html(&SAMPLE, w as f64, h as f64);
    if let Ok(path) = std::env::var("MOTOLII_INSPECTOR_HTML_OUT") {
        std::fs::write(&path, &html).expect("write html");
        println!("C7 HTML: {path}");
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = runtime
        .block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("no adapter");
    let (device, queue) = runtime
        .block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("c7-device"),
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
    // 2回呼ぶ。blitz-dom は hoisted paint child(= position付き かつ z-index≠0 の要素)の
    // 座標を flush_styles_to_layout で積むが、これは resolve_layout より前に走るため
    // (blitz-dom-0.3.0-beta.1 resolve.rs:86 → :90)、読む final_layout.location は
    // 1レイアウト前の値になる。初回は全部ゼロで、z-index付き要素が stacking context
    // ルートの原点へ落ちる。1枚しか描かない道具が定常状態を写すには2回要る。
    doc.resolve(0.0);
    doc.resolve(0.0);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("c7-inspector"),
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
        label: Some("c7-enc"),
    });
    let device_handle = DeviceHandle {
        instance: instance.clone(),
        adapter: adapter.clone(),
        device: device.clone(),
        queue: queue.clone(),
    };
    {
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
        label: Some("c7-readback"),
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

    let at = |x: u32, y: u32| -> [u8; 4] {
        let i = (y * bpr + x * 4) as usize;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    };
    println!("C7 左上 = {:?}", at(0, 0));
    println!("C7 中央 = {:?}", at(w / 2, h / 2));

    write_png(&out_path, w, h, bpr, &data);
    println!("C7 出力: {out_path}");
}

/// 8bit RGBA PNG。encoder依存を足さないため手で書く。
/// zlibは **stored**(非圧縮)ブロックだけを使う。仕様上これも正当なdeflateストリーム。
fn write_png(path: &str, w: u32, h: u32, bpr: u32, data: &[u8]) {
    // 行頭に filter byte 0 を足した生バイト列。プリマルチプライドを戻す。
    let mut raw = Vec::with_capacity(((w * 4 + 1) * h) as usize);
    for y in 0..h {
        raw.push(0u8);
        for x in 0..w {
            let i = (y * bpr + x * 4) as usize;
            let a = data[i + 3];
            let un = |c: u8| -> u8 {
                if a == 0 {
                    0
                } else {
                    ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8
                }
            };
            raw.push(un(data[i]));
            raw.push(un(data[i + 1]));
            raw.push(un(data[i + 2]));
            raw.push(a);
        }
    }

    let mut out = Vec::new();
    out.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit / truecolor+alpha
    push_chunk(&mut out, b"IHDR", &ihdr);
    push_chunk(&mut out, b"IDAT", &zlib_stored(&raw));
    push_chunk(&mut out, b"IEND", &[]);
    std::fs::write(path, out).expect("write png");
}

fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    let mut crc_input = Vec::with_capacity(4 + body.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(body);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01]; // deflate / 32K window / no dict
    let mut offset = 0usize;
    while offset < data.len() {
        let len = (data.len() - offset).min(0xffff);
        let last = if offset + len == data.len() { 1 } else { 0 };
        out.push(last);
        out.extend_from_slice(&(len as u16).to_le_bytes());
        out.extend_from_slice(&(!(len as u16)).to_le_bytes());
        out.extend_from_slice(&data[offset..offset + len]);
        offset += len;
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + *byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
