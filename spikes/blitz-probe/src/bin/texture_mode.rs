//! P6: 「Blitz をテクスチャレンダラとして使う」モードの端から端までの検証。
//!
//! P4 は静止画を1枚描いただけだった。本検証は採用予定の構成そのものを通す:
//!
//!   winit相当のイベント(ここでは合成) → 自前でルーティング → blitz-dom へ注入
//!     → DOM状態が変わる → 再レイアウト → 自前 wgpu29 テクスチャへ再描画
//!     → 読み戻して「実際に絵が変わったか」を確かめる
//!
//! これが通れば、窓モードで見つかった制約(キーがフォーム要素にしか届かない、
//! ピンチが届かない)は本構成に当てはまらないと言える。配り先を決めるのは
//! Motolii 側になるため。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::{EventDriver, NoopEventHandler};
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use atomic_refcell::AtomicRefCell;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use keyboard_types::Modifiers;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::FxHashMap;
use std::sync::Arc;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const W: u32 = 256;
const H: u32 = 128;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// `.clip` は既定で青、hover で橙。ポインタを載せたら色が変わることを見る。
const HTML: &str = r#"
<html><head><style>
  html, body { margin: 0; padding: 0; background: rgb(36, 36, 36); }
  .clip { position: absolute; left: 32px; top: 24px;
          width: 96px; height: 40px; background: rgb(150, 170, 219); }
  .clip:hover { background: rgb(255, 173, 86); }
</style></head>
<body><div class="clip"></div></body></html>
"#;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    renderer: Renderer,
    resources: Resources,
    device_handle: DeviceHandle,
}

fn main() {
    let mut gpu = init_gpu();

    let mut doc = HtmlDocument::from_html(HTML, DocumentConfig::default());
    doc.set_viewport(Viewport {
        window_size: (W, H),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);

    // ---- 1. 自前のヒット判定が使えるか ----
    // これが取れれば「今ポインタはBlitzパネルの上か、Stage/Timeline canvasの上か」を
    // Motolii 側で判定して振り分けられる。
    let inside = doc.hit(80.0, 44.0);
    let outside = doc.hit(8.0, 8.0);
    println!(
        "P6/1 hit(80,44)={} / hit(8,8)={}",
        inside.map(|h| h.node_id.to_string()).unwrap_or("None".into()),
        outside
            .map(|h| h.node_id.to_string())
            .unwrap_or("None".into())
    );

    // ---- 2. hover前の絵 ----
    let before = render_and_read(&mut gpu, &mut doc);
    let px_before = pick(&before, 80, 44);
    println!("P6/2 hover前 clip = {px_before:?} (150,170,219期待)");

    // ---- 3. 合成イベントを自分で注入する ----
    // winit から来たイベントを Motolii がルーティングして流す、という想定。
    {
        let mut driver = EventDriver::new(&mut *doc, NoopEventHandler);
        driver.handle_ui_event(UiEvent::PointerMove(pointer_at(80.0, 44.0)));
    }
    doc.resolve(0.0);
    println!("P6/3 PointerMove(80,44) を注入");

    // ---- 4. 絵が変わったか ----
    let after = render_and_read(&mut gpu, &mut doc);
    let px_after = pick(&after, 80, 44);
    println!("P6/4 hover後 clip = {px_after:?} (255,173,86期待)");

    // ---- 判定 ----
    let near = |g: [u8; 4], w: [u8; 3]| (0..3).all(|i| (g[i] as i32 - w[i] as i32).abs() <= 8);
    let mut ok = true;
    if inside.is_none() {
        println!("NG: clip上でhitが取れない");
        ok = false;
    }
    if !near(px_before, [150, 170, 219]) {
        println!("NG: hover前の色が違う");
        ok = false;
    }
    if !near(px_after, [255, 173, 86]) {
        println!("NG: 注入したイベントで絵が変わらなかった");
        ok = false;
    }

    println!(
        "\nP6 RESULT: {}",
        if ok {
            "PASS — 自前ルーティングのイベントがDOMを動かし、自前テクスチャの絵が変わった"
        } else {
            "FAIL"
        }
    );
    std::process::exit(if ok { 0 } else { 1 });
}

fn pointer_at(x: f32, y: f32) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: MouseEventButtons::None,
        mods: Modifiers::default(),
        details: PointerDetails {
            pressure: 0.0,
            tangential_pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
            altitude: 0.0,
            azimuth: 0.0,
        },
        element: Point { x, y },
        active_pointers: Arc::new(AtomicRefCell::new(Vec::new())),
    }
}

fn init_gpu() -> Gpu {
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

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("blitz-texture-mode"),
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
    let renderer = Renderer::new(
        &device,
        &RenderTargetConfig {
            format: FORMAT,
            width: W,
            height: H,
        },
    );
    let device_handle = DeviceHandle {
        instance,
        adapter,
        device: device.clone(),
        queue: queue.clone(),
    };
    Gpu {
        device,
        queue,
        texture,
        view,
        renderer,
        resources: Resources::new(),
        device_handle,
    }
}

/// 毎フレームこれを回す想定。Scene を作り直して自前テクスチャへ描き、読み戻す。
fn render_and_read(gpu: &mut Gpu, doc: &mut HtmlDocument) -> Vec<u8> {
    let mut scene = Scene::new(W as u16, H as u16);
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cache = FxHashMap::default();
        let mut bindings = FxHashMap::default();
        let im = ImageManager::new(
            &mut gpu.renderer,
            &mut gpu.resources,
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &mut cache,
        );
        let mut painter =
            VelloHybridScenePainter::new(&mut scene, im, &mut bindings, &gpu.device_handle);
        paint_scene(&mut painter, doc, 1.0, W, H, 0, 0);
    }
    gpu.renderer
        .render(
            &scene,
            &mut gpu.resources,
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &RenderSize {
                width: W,
                height: H,
            },
            &gpu.view,
            &TextureBindings::default(),
        )
        .expect("render");

    let bpr = (W * 4).next_multiple_of(256);
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bpr * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &gpu.texture,
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
    gpu.queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    slice.get_mapped_range().to_vec()
}

fn pick(data: &[u8], x: u32, y: u32) -> [u8; 4] {
    let bpr = (W * 4).next_multiple_of(256);
    let i = (y * bpr + x * 4) as usize;
    [data[i], data[i + 1], data[i + 2], data[i + 3]]
}
