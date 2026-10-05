mod a1;
mod comp;
pub use a1::motolii_a1_start;
mod decode;
mod draw;
mod mosh;
mod import;
mod isf;
mod post;
mod shelf;
mod space;
mod thor;

use std::time::{Duration, Instant};

use objc2::{ClassType, MainThreadMarker, MainThreadOnly};
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSEventMask, NSWindow,
    NSWindowStyleMask,
};
use objc2_foundation::{ns_string, NSDate, NSPoint, NSRect, NSSize, NSString};
use objc2_metal::MTLPixelFormat;
use objc2_quartz_core::CAMetalLayer;

use decode::VideoToolbox;
use draw::{Drawer, Over};
use import::Importer;
use thor::{log_record, Thor};

pub(crate) fn run() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("usage: a0 <video>")?;
    let mtm = MainThreadMarker::new().ok_or("a0 must run on the main thread")?;
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::new(80.0, 80.0), NSSize::new(960.0, 540.0)),
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable
                | NSWindowStyleMask::Resizable,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    window.setTitle(ns_string!("Motolii B"));
    unsafe { window.setReleasedWhenClosed(false) };
    window.center();

    let view = window.contentView().ok_or("window has no content view")?;
    view.setWantsLayer(true);
    let layer = CAMetalLayer::layer();
    layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
    layer.setFramebufferOnly(true);
    let bounds = view.bounds();
    layer.setFrame(bounds);
    let scale = window.backingScaleFactor();
    layer.setDrawableSize(objc2_core_foundation::CGSize {
        width: bounds.size.width * scale,
        height: bounds.size.height * scale,
    });
    view.setLayer(Some(layer.as_super()));

    window.makeKeyAndOrderFront(None);
    app.activate();

    let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
    instance_desc.backends = wgpu::Backends::METAL;
    let instance = wgpu::Instance::new(instance_desc);
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
            Retained::as_ptr(&layer).cast::<std::ffi::c_void>().cast_mut(),
        ))
    }
    .map_err(|error| error.to_string())?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .map_err(|error| error.to_string())?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("a0"),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        ..Default::default()
    }))
    .map_err(|error| error.to_string())?;

    let drawable = layer.drawableSize();
    let width = drawable.width.round().max(1.0) as u32;
    let height = drawable.height.round().max(1.0) as u32;
    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|format| {
            matches!(
                format,
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
            )
        })
        .unwrap_or(caps.formats[0]);
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        },
    );

    let importer = Importer::new(&device)?;
    let drawer = Drawer::new(&device, format);
    let (mut thor, record) = Thor::open(&device, &queue, width, height)?;
    let over = Over::new(&device, format, record.straight);
    let mut video = VideoToolbox::open(&NSString::from_str(&path))?;
    let pace = Duration::from_secs_f64(1.0 / f64::from(video.fps));
    let mut logged = false;
    while window.isVisible() {
        pump(&app);
        let started = Instant::now();
        let Some(sample) = video.next_frame()? else {
            video = VideoToolbox::open(&NSString::from_str(&path))?;
            continue;
        };
        if !logged {
            eprintln!(
                "a0: {}x{} {:.2} fps. CVPixelBuffer stays until on_submitted_work_done",
                video.width, video.height, video.fps
            );
        }
        let imported = importer.import(&device, &sample)?;
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation => continue,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        drawer.draw(&device, &queue, &view, &imported.y, &imported.uv);
        let sync_ms = thor.draw()?;
        over.draw(&device, &queue, &view, thor.texture());
        if !logged {
            log_record(&record, sync_ms);
            logged = true;
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        queue.on_submitted_work_done(move || {
            let _ = sender.send(());
        });
        frame.present();
        receiver.recv().map_err(|_| "gpu completion closed".to_string())?;
        drop(imported);
        importer.flush();
        let spent = started.elapsed();
        if spent < pace {
            std::thread::sleep(pace - spent);
        }
    }
    Ok(())
}

fn pump(app: &NSApplication) {
    loop {
        let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
            NSEventMask::Any,
            Some(&NSDate::distantPast()),
            unsafe { NSDefaultRunLoopMode },
            true,
        ) else {
            break;
        };
        app.sendEvent(&event);
    }
}

use objc2_foundation::NSDefaultRunLoopMode;
