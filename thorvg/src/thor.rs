use std::ffi::{c_char, c_void};
use std::time::Instant;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDevice, MTLTexture};

use objc2_metal::MTLTextureType;

#[repr(C)]
struct BNative {
    _private: [u8; 0],
}

extern "C" {
    fn b_native_open(width: u32, height: u32, error: *mut c_char, error_len: usize) -> *mut BNative;
    fn b_native_instance(native: *mut BNative) -> *mut c_void;
    fn b_native_device(native: *mut BNative) -> *mut c_void;
    fn b_native_texture(native: *mut BNative) -> *mut c_void;
    fn b_native_metal_device(native: *mut BNative) -> *mut c_void;
    fn b_native_metal_queue(native: *mut BNative) -> *mut c_void;

    fn tvg_engine_init(threads: u32) -> u32;
    fn tvg_wgcanvas_create(op: u32) -> *mut c_void;
    fn tvg_wgcanvas_set_target(
        canvas: *mut c_void,
        device: *mut c_void,
        instance: *mut c_void,
        target: *mut c_void,
        w: u32,
        h: u32,
        cs: u32,
        kind: i32,
    ) -> u32;
    fn tvg_canvas_add(canvas: *mut c_void, paint: *mut c_void) -> u32;
    fn tvg_canvas_remove(canvas: *mut c_void, paint: *mut c_void) -> u32;
    fn tvg_canvas_update(canvas: *mut c_void) -> u32;
    fn tvg_canvas_draw(canvas: *mut c_void, clear: bool) -> u32;
    fn tvg_canvas_sync(canvas: *mut c_void) -> u32;
    fn tvg_shape_new() -> *mut c_void;
    fn tvg_shape_append_rect(
        paint: *mut c_void,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
        cw: bool,
    ) -> u32;
    fn tvg_shape_set_fill_color(paint: *mut c_void, r: u8, g: u8, b: u8, a: u8) -> u32;
    fn tvg_paint_set_opacity(paint: *mut c_void, opacity: u8) -> u32;
    fn tvg_paint_translate(paint: *mut c_void, x: f32, y: f32) -> u32;
    fn tvg_font_load(path: *const i8) -> u32;
    fn tvg_text_new() -> *mut c_void;
    fn tvg_text_set_font(text: *mut c_void, name: *const i8) -> u32;
    fn tvg_text_set_size(text: *mut c_void, size: f32) -> u32;
    fn tvg_text_set_text(text: *mut c_void, utf8: *const i8) -> u32;
    fn tvg_text_set_color(text: *mut c_void, r: u8, g: u8, b: u8) -> u32;
    fn tvg_text_align(text: *mut c_void, x: f32, y: f32) -> u32;
}

const ABGR8888S: u32 = 2;
const ENGINE_DEFAULT: u32 = 1;

pub(super) struct Thor {
    #[allow(dead_code)]
    native: *mut BNative,
    canvas: *mut c_void,
    texture: wgpu::Texture,
    width: f32,
    height: f32,
}

pub(super) struct GlueRecord {
    pub same_device: bool,
    pub same_queue: bool,
    pub contents_survive: bool,
    pub straight: bool,
    pub white: [u8; 4],
}

impl Thor {
    pub(super) fn open(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Result<(Self, GlueRecord), String> {
        let mut error = [0u8; 256];
        let native = unsafe {
            b_native_open(
                width,
                height,
                error.as_mut_ptr().cast(),
                error.len(),
            )
        };
        if native.is_null() {
            let text = std::ffi::CStr::from_bytes_until_nul(&error)
                .ok()
                .and_then(|text| text.to_str().ok())
                .unwrap_or("native device failed");
            return Err(text.to_string());
        }
        if unsafe { tvg_engine_init(0) } != 0 {
            return Err("tvg_engine_init failed".into());
        }
        let canvas = unsafe { tvg_wgcanvas_create(ENGINE_DEFAULT) };
        if canvas.is_null() {
            return Err("tvg_wgcanvas_create failed".into());
        }
        let status = unsafe {
            tvg_wgcanvas_set_target(
                canvas,
                b_native_device(native),
                b_native_instance(native),
                b_native_texture(native),
                width,
                height,
                ABGR8888S,
                1,
            )
        };
        if status != 0 {
            return Err(format!("tvg_wgcanvas_set_target {status}"));
        }

        let metal = unsafe { b_native_metal_device(native) };
        let metal_device = unsafe {
            Retained::retain(metal.cast::<ProtocolObject<dyn MTLDevice>>())
                .ok_or("native MTLDevice was null")?
        };
        let rust_device = rust_metal_device(device)?;
        let same_device = metal_device.registryID() == rust_device.registryID();
        if !same_device {
            return Err(format!(
                "candidate 2 failed: registry {} vs {}",
                metal_device.registryID(),
                rust_device.registryID()
            ));
        }
        let texture = import_l(device, native, width, height)?;
        let contents_survive = contents_survive(canvas, device, queue, &texture)?;
        let (straight, white) = straight_white(canvas, device, queue, &texture)?;
        add_live(canvas)?;
        let metal_queue = unsafe { b_native_metal_queue(native) };
        let rust_queue = rust_metal_queue(queue)?;
        let same_queue = metal_queue == rust_queue;
        Ok((
            Self {
                native,
                canvas,
                texture,
                width: width as f32,
                height: height as f32,
            },
            GlueRecord {
                same_device,
                same_queue,
                contents_survive,
                straight,
                white,
            },
        ))
    }

    pub(super) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    /// Vector mark and text, into L. Same canvas as the shapes. Premultiplied, so the compositor does not multiply again.
    pub(super) fn lettering(&self) -> Result<(), String> {
        clear_paints(self.canvas)?;
        let sx = self.width / 640.0;
        let sy = self.height / 360.0;
        add_rect(self.canvas, 196.0 * sx, 28.0 * sy, 10.0 * sx, 72.0 * sy, 232, 92, 44, 255, 255)?;
        static LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !LOADED.load(std::sync::atomic::Ordering::Relaxed) {
            let path = std::ffi::CString::new("/System/Library/Fonts/Supplemental/Arial.ttf")
                .map_err(|_| "font path".to_string())?;
            check(unsafe { tvg_font_load(path.as_ptr()) }, "font")?;
            LOADED.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        let text = unsafe { tvg_text_new() };
        if text.is_null() {
            return Err("tvg_text_new failed".into());
        }
        let face = std::ffi::CString::new("Arial").map_err(|_| "font name".to_string())?;
        let words = std::ffi::CString::new("Motolii").map_err(|_| "text".to_string())?;
        check(unsafe { tvg_text_set_font(text, face.as_ptr()) }, "face")?;
        check(unsafe { tvg_text_set_size(text, 64.0 * sy) }, "size")?;
        check(unsafe { tvg_text_set_text(text, words.as_ptr()) }, "words")?;
        check(unsafe { tvg_text_set_color(text, 244, 236, 220) }, "ink")?;
        check(unsafe { tvg_paint_set_opacity(text, 255) }, "opacity")?;
        check(unsafe { tvg_text_align(text, 0.0, 1.0) }, "align")?;
        check(unsafe { tvg_paint_translate(text, 214.0 * sx, 92.0 * sy) }, "place")?;
        check(unsafe { tvg_canvas_add(self.canvas, text) }, "add text")?;
        paint(self.canvas, true)
    }

    pub(super) fn draw(&mut self) -> Result<f64, String> {
        check(unsafe { tvg_canvas_update(self.canvas) }, "update")?;
        check(unsafe { tvg_canvas_draw(self.canvas, true) }, "draw")?;
        let started = Instant::now();
        check(unsafe { tvg_canvas_sync(self.canvas) }, "sync")?;
        Ok(started.elapsed().as_secs_f64() * 1000.0)
    }
}

fn import_l(
    device: &wgpu::Device,
    native: *mut BNative,
    width: u32,
    height: u32,
) -> Result<wgpu::Texture, String> {
    let metal = unsafe { wgpu_texture_metal(b_native_texture(native)) };
    let retained = unsafe { Retained::retain(metal) }.ok_or("L has no MTLTexture")?;
    let hal = unsafe {
        wgpu_hal::metal::Device::texture_from_raw(
            retained,
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
            hal,
            &wgpu::TextureDescriptor {
                label: Some("L"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
        )
    })
}

unsafe fn wgpu_texture_metal(texture: *mut c_void) -> *mut ProtocolObject<dyn MTLTexture> {
    extern "C" {
        fn wgpuTextureGetNativeMetalTexture(texture: *mut c_void) -> *mut c_void;
    }
    wgpuTextureGetNativeMetalTexture(texture).cast()
}

fn contents_survive(
    canvas: *mut c_void,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<bool, String> {
    clear_paints(canvas)?;
    add_rect(canvas, 0.0, 0.0, 4000.0, 4000.0, 255, 0, 0, 255, 255)?;
    paint(canvas, true)?;
    clear_paints(canvas)?;
    add_rect(canvas, 400.0, 400.0, 32.0, 32.0, 0, 0, 255, 255, 255)?;
    paint(canvas, false)?;
    let pixel = read_pixel(device, queue, texture, 8, 8)?;
    Ok(pixel[0] > 200 && pixel[2] < 40)
}

fn straight_white(
    canvas: *mut c_void,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<(bool, [u8; 4]), String> {
    clear_paints(canvas)?;
    add_rect(canvas, 0.0, 0.0, 64.0, 64.0, 255, 255, 255, 255, 128)?;
    paint(canvas, true)?;
    let pixel = read_pixel(device, queue, texture, 8, 8)?;
    let straight = pixel[0] > 240 && pixel[1] > 240 && pixel[2] > 240 && (100..160).contains(&pixel[3]);
    Ok((straight, pixel))
}

fn add_live(canvas: *mut c_void) -> Result<(), String> {
    clear_paints(canvas)?;
    add_rect(canvas, 48.0, 48.0, 220.0, 140.0, 255, 140, 40, 255, 255)?;
    add_rect(canvas, 300.0, 80.0, 280.0, 180.0, 255, 255, 255, 255, 128)?;
    Ok(())
}

fn add_rect(
    canvas: *mut c_void,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    opacity: u8,
) -> Result<(), String> {
    let paint = unsafe { tvg_shape_new() };
    if paint.is_null() {
        return Err("tvg_shape_new failed".into());
    }
    check(unsafe { tvg_shape_append_rect(paint, x, y, w, h, 0.0, 0.0, true) }, "rect")?;
    check(unsafe { tvg_shape_set_fill_color(paint, r, g, b, a) }, "fill")?;
    check(unsafe { tvg_paint_set_opacity(paint, opacity) }, "opacity")?;
    check(unsafe { tvg_canvas_add(canvas, paint) }, "add")?;
    Ok(())
}

fn clear_paints(canvas: *mut c_void) -> Result<(), String> {
    check(unsafe { tvg_canvas_remove(canvas, std::ptr::null_mut()) }, "remove")
}

fn paint(canvas: *mut c_void, clear: bool) -> Result<(), String> {
    check(unsafe { tvg_canvas_update(canvas) }, "update")?;
    check(unsafe { tvg_canvas_draw(canvas, clear) }, "draw")?;
    check(unsafe { tvg_canvas_sync(canvas) }, "sync")
}

fn check(status: u32, what: &str) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("thorvg {what} {status}"))
    }
}

fn read_pixel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    x: u32,
    y: u32,
) -> Result<[u8; 4], String> {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("b-probe"),
        size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("b-probe"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|error| error.to_string())?;
    receiver.recv().map_err(|_| "pixel map closed".to_string())?.map_err(|error| error.to_string())?;
    let view = buffer.slice(..).get_mapped_range();
    let pixel = [view[0], view[1], view[2], view[3]];
    drop(view);
    buffer.unmap();
    Ok(pixel)
}

fn rust_metal_device(
    device: &wgpu::Device,
) -> Result<Retained<ProtocolObject<dyn MTLDevice>>, String> {
    let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or("device is not Metal")?;
    Ok(hal.raw_device().clone())
}

fn rust_metal_queue(queue: &wgpu::Queue) -> Result<*mut c_void, String> {
    let hal = unsafe { queue.as_hal::<wgpu::hal::api::Metal>() }.ok_or("queue is not Metal")?;
    Ok(std::ptr::from_ref(hal.as_raw()).cast_mut().cast())
}

pub(super) fn log_record(record: &GlueRecord, sync_ms: f64) {
    eprintln!(
        "B: candidate {} (same MTLDevice registry). queues {}. wait tvg_canvas_sync {:.3} ms. L 1. contents {}. format ABGR8888S stored {}. white rgba {} {} {} {}",
        if record.same_device { "2" } else { "not-2" },
        if record.same_queue { "same" } else { "different" },
        sync_ms,
        if record.contents_survive { "survive" } else { "cleared" },
        if record.straight { "straight" } else { "not-straight" },
        record.white[0],
        record.white[1],
        record.white[2],
        record.white[3],
    );
}
