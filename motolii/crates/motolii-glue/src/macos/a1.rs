use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_core_foundation::{CFDictionary, CFRetained, CFString};
use objc2_core_video::{
    kCVPixelBufferHeightKey, kCVPixelBufferIOSurfacePropertiesKey, kCVPixelBufferMetalCompatibilityKey,
    kCVPixelBufferPixelFormatTypeKey, kCVPixelBufferPoolAllocationThresholdKey,
    kCVPixelBufferPoolMinimumBufferCountKey, kCVPixelBufferWidthKey, kCVPixelFormatType_32BGRA,
    kCVReturnSuccess, kCVReturnWouldExceedAllocationThreshold, CVPixelBuffer, CVPixelBufferGetIOSurface,
    CVPixelBufferPool,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString};
use objc2_io_surface::IOSurfaceID;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage};

use super::decode::VideoToolbox;
use super::import::Importer;
use super::space::World;
use super::thor::Thor;

static STOP: AtomicBool = AtomicBool::new(false);

pub type Publish = unsafe extern "C" fn(*mut c_void, *mut c_void);

struct Callback {
    publish: Publish,
    user: *mut c_void,
}

unsafe impl Send for Callback {}

/// Output ring. The texture cache is keyed by IOSurface and does not retain the CVPixelBuffer.
struct Ring {
    pool: CFRetained<CVPixelBufferPool>,
    cache: HashMap<IOSurfaceID, wgpu::Texture>,
    skipped: u64,
}

impl Ring {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let pool = pool(width, height)?;
        Ok(Self {
            pool,
            cache: HashMap::new(),
            skipped: 0,
        })
    }

    fn checkout(&mut self) -> Result<Option<CFRetained<CVPixelBuffer>>, String> {
        let mut raw: *mut CVPixelBuffer = std::ptr::null_mut();
        let status = unsafe {
            CVPixelBufferPool::create_pixel_buffer(None, &self.pool, NonNull::from(&mut raw))
        };
        if status == kCVReturnWouldExceedAllocationThreshold {
            self.skipped += 1;
            return Ok(None);
        }
        if status != kCVReturnSuccess {
            return Err(format!("pixel buffer pool {status}"));
        }
        let pixel = NonNull::new(raw).ok_or("pool returned a null buffer")?;
        Ok(Some(unsafe { CFRetained::from_raw(pixel) }))
    }

    fn view(&mut self, device: &wgpu::Device, pixel: &CVPixelBuffer) -> Result<wgpu::TextureView, String> {
        let surface = unsafe { CVPixelBufferGetIOSurface(Some(pixel)) }.ok_or("buffer has no IOSurface")?;
        let id = surface.id();
        if !self.cache.contains_key(&id) {
            let texture = bind(device, &surface, surface.width() as u32, surface.height() as u32)?;
            self.cache.insert(id, texture);
        }
        Ok(self.cache[&id].create_view(&wgpu::TextureViewDescriptor::default()))
    }
}

fn pool(width: u32, height: u32) -> Result<CFRetained<CVPixelBufferPool>, String> {
    let three = number(3);
    let cap = number(3);
    let format = number(kCVPixelFormatType_32BGRA as i32);
    let w = number(width as i32);
    let h = number(height as i32);
    let metal = number(1);
    let iosurface = NSDictionary::<NSString, AnyObject>::new();
    let limits = cf_dict(
        &[
            ns_key(unsafe { &*kCVPixelBufferPoolMinimumBufferCountKey }),
            ns_key(unsafe { &*kCVPixelBufferPoolAllocationThresholdKey }),
        ],
        &[three.as_ref(), cap.as_ref()],
    )?;
    let pixels = cf_dict(
        &[
            ns_key(unsafe { &*kCVPixelBufferPixelFormatTypeKey }),
            ns_key(unsafe { &*kCVPixelBufferWidthKey }),
            ns_key(unsafe { &*kCVPixelBufferHeightKey }),
            ns_key(unsafe { &*kCVPixelBufferMetalCompatibilityKey }),
            ns_key(unsafe { &*kCVPixelBufferIOSurfacePropertiesKey }),
        ],
        &[
            format.as_ref(),
            w.as_ref(),
            h.as_ref(),
            metal.as_ref(),
            iosurface.as_ref(),
        ],
    )?;
    let mut raw: *mut CVPixelBufferPool = std::ptr::null_mut();
    let status = unsafe {
        CVPixelBufferPool::create(None, Some(&limits), Some(&pixels), NonNull::from(&mut raw))
    };
    if status != kCVReturnSuccess {
        return Err(format!("CVPixelBufferPoolCreate {status}"));
    }
    let pool = NonNull::new(raw).ok_or("pool was null")?;
    Ok(unsafe { CFRetained::from_raw(pool) })
}

fn number(value: i32) -> Retained<NSNumber> {
    NSNumber::numberWithInt(value)
}

fn ns_key(key: &CFString) -> &NSString {
    unsafe { &*(key as *const CFString).cast::<NSString>() }
}

fn cf_dict(keys: &[&NSString], values: &[&AnyObject]) -> Result<CFRetained<CFDictionary>, String> {
    let ns = NSDictionary::from_slices(keys, values);
    let raw = Retained::into_raw(ns);
    let ptr = NonNull::new(raw.cast()).ok_or("dictionary was null")?;
    Ok(unsafe { CFRetained::from_raw(ptr) })
}

fn bind(
    device: &wgpu::Device,
    surface: &objc2_io_surface::IOSurfaceRef,
    width: u32,
    height: u32,
) -> Result<wgpu::Texture, String> {
    let descriptor = MTLTextureDescriptor::new();
    unsafe {
        descriptor.setTextureType(MTLTextureType::Type2D);
        descriptor.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        descriptor.setWidth(width as usize);
        descriptor.setHeight(height as usize);
        descriptor.setMipmapLevelCount(1);
    }
    descriptor.setStorageMode(MTLStorageMode::Shared);
    descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
    let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or("device is not Metal")?;
    let raw = hal
        .raw_device()
        .newTextureWithDescriptor_iosurface_plane(&descriptor, surface, 0)
        .ok_or("Metal could not bind IOSurface")?;
    drop(hal);
    let hal_texture = unsafe {
        wgpu_hal::metal::Device::texture_from_raw(
            raw,
            wgpu::TextureFormat::Bgra8Unorm,
            MTLTextureType::Type2D,
            1,
            1,
            wgpu_hal::CopyExtent {
                width,
                height,
                depth: 1,
            },
        )
    };
    Ok(unsafe {
        device.create_texture_from_hal::<wgpu::hal::api::Metal>(
            hal_texture,
            &wgpu::TextureDescriptor {
                label: Some("a1"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
        )
    })
}

fn gpu() -> Result<(wgpu::Device, wgpu::Queue), String> {
    let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
    instance_desc.backends = wgpu::Backends::METAL;
    let instance = wgpu::Instance::new(instance_desc);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .map_err(|error| error.to_string())?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("a1"),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        ..Default::default()
    }))
    .map_err(|error| error.to_string())
}

fn session(path: &str, view_w: u32, view_h: u32, callback: Callback, ready: &mpsc::Sender<bool>) -> Result<(), String> {
    let (device, queue) = gpu()?;
    let material = match super::mosh::material(path) {
        Ok(mosh) => {
            eprintln!(
                "A1: datamosh kept {} IDR, dropped {}, repeated {} P slices",
                mosh.kept_idr, mosh.dropped_idr, mosh.repeated_p
            );
            mosh.path
        }
        Err(error) => {
            eprintln!("A1: datamosh unavailable ({error}), using the source clip");
            path.to_string()
        }
    };
    let mut video = VideoToolbox::open(&NSString::from_str(&material))?;
    let samples = video.drain()?;
    eprintln!("A1: {} video frames kept by time", samples.len());
    let video_w = video.width as u32;
    let video_h = video.height as u32;
    if video_w == 0 || video_h == 0 {
        return Err(format!("video size {video_w}x{video_h}"));
    }
    let width = if view_w > 0 { view_w } else { video_w };
    let height = if view_h > 0 { view_h } else { video_h };
    let importer = Importer::new(&device)?;
    let (thor, _record) = Thor::open(&device, &queue, width, height)?;
    let mut world = World::new(&device, &queue, width, height)?;
    let mut ring = Ring::new(width, height)?;
    let pace = Duration::from_secs_f64(1.0 / f64::from(video.fps));
    let mut logged = false;
    let mut frame_index = 0u32;
    let scrub = std::env::var_os("MOTOLII_SCRUB").is_some();
    eprintln!("A1: picture {width}x{height}, video {video_w}x{video_h}. cache key is IOSurface, not CVPixelBuffer");
    let _ = ready.send(true);
    while !STOP.load(Ordering::Relaxed) {
        let started = Instant::now();
        let t = if scrub && frame_index >= 40 { frame_index - 32 } else { frame_index };
        let needed = super::space::frames_needed(t);
        let mut held = Vec::new();
        for time in &needed {
            let sample = &samples[*time as usize % samples.len()];
            held.push((*time, importer.import(&device, sample)?));
        }
        let Some(pixel) = ring.checkout()? else {
            if ring.skipped == 1 {
                eprintln!("A1: pool full, frame skipped");
            }
            continue;
        };
        let ats: Vec<super::space::At<'_>> = held
            .iter()
            .map(|(time, imported)| super::space::At { time: *time, y: &imported.y, uv: &imported.uv })
            .collect();
        let view = ring.view(&device, &pixel)?;
        world.draw(&device, &queue, &view, &thor, t, &ats)?;
        frame_index = frame_index.wrapping_add(1);
        let (sender, receiver) = mpsc::channel();
        queue.on_submitted_work_done(move || {
            let _ = sender.send(());
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| error.to_string())?;
        receiver.recv().map_err(|_| "gpu completion closed".to_string())?;
        drop(held);
        importer.flush();
        unsafe { (callback.publish)(callback.user, CFRetained::as_ptr(&pixel).as_ptr().cast()) };
        drop(pixel);
        if !logged {
            eprintln!("A1: published one buffer after on_submitted_work_done");
            logged = true;
        }
        let spent = started.elapsed();
        if spent < pace {
            thread::sleep(pace - spent);
        }
    }
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn motolii_a1_start(
    path: *const c_char,
    view_w: u32,
    view_h: u32,
    publish: Publish,
    user: *mut c_void,
) -> i32 {
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    let callback = Callback { publish, user };
    STOP.store(false, Ordering::Relaxed);
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || match session(&path, view_w, view_h, callback, &sender) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("A1: {error}");
            let _ = sender.send(false);
        }
    });
    match receiver.recv() {
        Ok(true) => 0,
        _ => 1,
    }
}
