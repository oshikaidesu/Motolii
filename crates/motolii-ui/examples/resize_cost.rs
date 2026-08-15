//! リサイズで `BlitzSurface` を作り直す費用と、`resize` で済ませる費用を並べて測る。
//!
//! ドッキングの splitter をドラッグすると面の寸法が毎フレーム変わる。そこで
//! `BlitzSurface::new` を呼んでいたのが `pane.rs` の作りだった。
//! `vello_hybrid::Renderer` は `RenderSize` を毎回受け取って自分で追随するので
//! 作り直す必要が無い、というのがこの計測で確かめたいこと。
//!
//! 実行: `cargo run --release -p motolii-ui --example resize_cost`

use std::time::Instant;

use motolii_ui::browser_blitz::render::BlitzSurface;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// splitter を動かしたときに実際に通る寸法の幅。ドラッグ1回ぶんを模す。
const SIZES: [(u32, u32); 8] = [
    (1024, 640),
    (992, 640),
    (960, 640),
    (928, 640),
    (896, 640),
    (864, 640),
    (832, 640),
    (800, 640),
];

fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("resize-cost"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .expect("no device");
    println!("backend={:?}", adapter.get_info().backend);

    // 温める(初回だけ走る初期化を計測へ混ぜない)。
    let mut warm = BlitzSurface::new(&device, &adapter, &queue, FORMAT, 1024, 640);
    warm.resize(800, 640);
    drop(warm);
    device.poll(wgpu::PollType::Wait).ok();

    // ---- 作り直す場合 ----
    let started = Instant::now();
    for (width, height) in SIZES {
        let surface = BlitzSurface::new(&device, &adapter, &queue, FORMAT, width, height);
        std::hint::black_box(&surface);
    }
    device.poll(wgpu::PollType::Wait).ok();
    let rebuild = started.elapsed();

    // ---- resize で済ませる場合 ----
    let mut surface = BlitzSurface::new(&device, &adapter, &queue, FORMAT, 1024, 640);
    device.poll(wgpu::PollType::Wait).ok();
    let started = Instant::now();
    for (width, height) in SIZES {
        surface.resize(width, height);
        std::hint::black_box(&surface);
    }
    device.poll(wgpu::PollType::Wait).ok();
    let resize = started.elapsed();

    let steps = SIZES.len() as u32;
    println!(
        "BlitzSurface::new x{steps}: {:?} ({:?}/回)",
        rebuild,
        rebuild / steps
    );
    println!(
        "BlitzSurface::resize x{steps}: {:?} ({:?}/回)",
        resize,
        resize / steps
    );
}
