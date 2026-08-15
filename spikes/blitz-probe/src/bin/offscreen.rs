//! P4: Blitz を「Motolii が作った wgpu 29 デバイス」上のテクスチャへ描けるか。
//!
//! 再建計画の要石。成立すれば Timeline/パネルを HTML/CSS で書いたまま、
//! Rerun Stage と同じ窓へ合成できる。窓も dioxus も使わないヘッドレス検証。
//!
//! 版に関する重要な注:
//!   wgpu 29 系の描画実体は `vello_hybrid`(+ `anyrender_vello_hybrid`)。
//!   `anyrender_vello`(classic vello) は wgpu 26 なので Motolii と型が繋がらない。
//!   classic 側にある `ImageRenderer` / `CustomPaintSource` は hybrid 側に無く、
//!   代わりに `vello_hybrid::Renderer::render(.., view: &TextureView, ..)` が
//!   任意のテクスチャを受けるので、そこを直接使う。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::FxHashMap;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const W: u32 = 256;
const H: u32 = 128;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// 既知の色の矩形だけを置く。Timeline の clip と key を模す。
const HTML: &str = r#"
<html><head><style>
  html, body { margin: 0; padding: 0; background: rgb(36, 36, 36); }
  .clip { position: absolute; left: 32px; top: 24px;
          width: 96px; height: 40px; background: rgb(150, 170, 219); }
  .key  { position: absolute; left: 180px; top: 40px;
          width: 16px; height: 16px; background: rgb(255, 173, 86); }
</style></head>
<body><div class="clip"></div><div class="key"></div></body></html>
"#;

fn main() {
    // ---- 1. Motolii 側を模したデバイスを自分で作る ----
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("motolii-side-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .expect("no device");
    println!("P4/1 自前デバイス取得: backend={:?}", adapter.get_info().backend);

    // ---- 2. HTML/CSS を解決 ----
    let mut doc = HtmlDocument::from_html(HTML, DocumentConfig::default());
    doc.set_viewport(Viewport {
        window_size: (W, H),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);
    println!("P4/2 HTML/CSS 解決");

    // ---- 3. 自前テクスチャ ----
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("blitz-offscreen"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
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

    // ---- 4. DOM を vello_hybrid Scene へ、そして自前テクスチャへ描く ----
    let mut renderer = Renderer::new(
        &device,
        &RenderTargetConfig {
            format: FORMAT,
            width: W,
            height: H,
        },
    );
    let mut resources = Resources::new();
    let mut scene = Scene::new(W as u16, H as u16);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("blitz-offscreen-enc"),
    });
    let device_handle = DeviceHandle {
        instance: instance.clone(),
        adapter: adapter.clone(),
        device: device.clone(),
        queue: queue.clone(),
    };

    {
        let mut cache = FxHashMap::default();
        let mut texture_bindings_map = FxHashMap::default();
        let image_manager = ImageManager::new(
            &mut renderer,
            &mut resources,
            &device,
            &queue,
            &mut encoder,
            &mut cache,
        );
        let mut painter = VelloHybridScenePainter::new(
            &mut scene,
            image_manager,
            &mut texture_bindings_map,
            &device_handle,
        );
        paint_scene(&mut painter, &mut doc, 1.0, W, H, 0, 0);
    }
    println!("P4/3 DOM を Scene へ描画コマンド化");

    renderer
        .render(
            &scene,
            &mut resources,
            &device,
            &queue,
            &mut encoder,
            &RenderSize {
                width: W,
                height: H,
            },
            &view,
            &TextureBindings::default(),
        )
        .expect("render");
    println!("P4/4 自前 wgpu::Texture へ描画");

    // ---- 5. 読み戻して中身を確かめる ----
    let bpr = (W * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (bpr * H) as u64,
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
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    let data = slice.get_mapped_range().to_vec();

    let px = |x: u32, y: u32| -> [u8; 4] {
        let i = (y * bpr + x * 4) as usize;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    };

    let bg = px(4, 4);
    let clip = px(80, 44);
    let key = px(188, 48);

    println!("P4/5 bg   (36,36,36 期待)    = {bg:?}");
    println!("P4/5 clip (150,170,219 期待) = {clip:?}");
    println!("P4/5 key  (255,173,86 期待)  = {key:?}");

    let near = |got: [u8; 4], want: [u8; 3]| -> bool {
        (0..3).all(|i| (got[i] as i32 - want[i] as i32).abs() <= 8)
    };

    let mut ok = true;
    for (name, got, want) in [
        ("bg", bg, [36u8, 36, 36]),
        ("clip", clip, [150, 170, 219]),
        ("key", key, [255, 173, 86]),
    ] {
        if !near(got, want) {
            println!("NG: {name} の色が一致しない");
            ok = false;
        }
    }

    println!(
        "\nP4 RESULT: {}",
        if ok {
            "PASS — Motolii側が作った wgpu29 デバイスのテクスチャへ Blitz を描けた"
        } else {
            "FAIL"
        }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
