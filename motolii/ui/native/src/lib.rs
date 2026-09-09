#![recursion_limit = "256"]
pub use motolii_doc as doc;
pub use motolii_render as render;
mod editor;
mod port;
mod snapshot;
mod snapshot_cache;
mod export_job;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use motolii_doc::store::{property, Document, Intent, LayerId, PropertyId, RationalTime, Value};
use motolii_render::engine::Engine;
use objc2_io_surface::IOSurfaceRef;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage};
use serde_json::json;

pub struct EditorRuntime {
    doc: Document,
    engine: Engine,
    selected: Option<LayerId>,
    selected_ids: Vec<LayerId>,
    selected_keys: Vec<editor::session::KeySel>,
    clipboard: editor::clipboard::Clipboard,
    path: Option<String>,
    saved_signature: String,
    color_target: Option<editor::session::ColorSlot>,
    exporter: export_job::ExportController,
    clock: editor::playback::Clock,
    clock_revision: doc::store::Revision,
    frame: i64,
    device_id: u64,
    render_count: u64,
    render_ms: f64,
    picked_color: Option<[f64; 4]>,
    pick_serial: u64,
    reply: CString,
    error: Option<String>,
    preview: Option<(u64, Vec<Intent>)>,
    preview_tag: Option<String>,
    stage_drag: Option<editor::stage::DragSession>,
    user_stage: bool,
    /// Animate が入っている間、触った値は今の時刻のキーになる。
    pub(crate) animate: bool,
    /// 最後に全部入りの status を送った時の Document の版。同じ版で再生中なら生値だけ送る。
    pub(crate) full_status_revision: std::cell::RefCell<Option<String>>,
    user_camera: crate::doc::core::ResolvedCamera,
    snapshot_cache: std::cell::RefCell<snapshot_cache::SnapshotCache>,
}

impl EditorRuntime {
    fn open(path: &str) -> Result<Self, String> {
        let doc = if path.is_empty() { doc::store::blank_project() } else { Document::load(path).map_err(|e| e.to_string())? };
        if doc.view().composition().map_err(|e| e.to_string())?.is_none() {
            return Err("Saved document has no composition".into());
        }
        let selected = doc.view().layers().first().copied();
        let engine = Engine::new().map_err(|e| e.to_string())?;
        let device_id = unsafe { engine.gpu_device().as_hal::<wgpu::hal::api::Metal>() }
            .ok_or("Engine did not create a Metal device")?.raw_device().registryID();
        let saved_signature = snapshot::authored_signature(&doc)?;
        let clock = editor::playback::Clock::from_document(&doc, 60.0);
        clock.sync_document(&doc);
        let clock_revision = doc.revision();
        Ok(Self { selected_ids: selected.into_iter().collect(), selected_keys: Vec::new(), clipboard: Default::default(), path: if path.is_empty() { None } else { Some(path.into()) }, saved_signature, color_target: None, exporter: Default::default(), clock, clock_revision, doc, engine, selected, frame: 0, device_id, render_count: 0,
            render_ms: 0.0, picked_color: None, pick_serial: 0, reply: CString::new("{}").unwrap(), error: None, preview: None, preview_tag: None, stage_drag: None, snapshot_cache: Default::default(), user_stage: true, animate: false, full_status_revision: Default::default(), user_camera: Default::default() })
    }

    fn time(&self) -> Result<RationalTime, String> {
        let comp = self.doc.view().composition().map_err(|e| e.to_string())?.ok_or("No composition")?;
        RationalTime::try_from_frame(self.frame, comp.fps).map_err(|e| e.to_string())
    }

    fn position(&self, layer: LayerId) -> Result<[f64; 2], String> {
        let view = self.doc.view();
        let time = self.time()?;
        let id = PropertyId::new(property::POSITION).map_err(|e| e.to_string())?;
        let mut point = match view.value_at(layer, &id, time).map_err(|e| e.to_string())? {
            Some(Value::Vec2(p)) => p,
            _ => [0.0, 0.0],
        };
        for (axis, name) in [(0, property::POSITION_X), (1, property::POSITION_Y)] {
            if let Some(Value::F64(value)) = view.value_at(layer, &PropertyId::new(name).map_err(|e| e.to_string())?, time).map_err(|e| e.to_string())? {
                point[axis] = value;
            }
        }
        Ok(point)
    }

    fn cancel_preview(&mut self) {
        self.preview_tag = None;
        self.stage_drag = None;
        if let Some((owner, _)) = self.preview.take() { self.doc.clear_preview_edits(owner); }
    }

    fn x_edit(&self, value: f64) -> Result<Vec<Intent>, String> {
        if !value.is_finite() { return Err("X must be finite".into()); }
        let layer = self.selected.ok_or("Select a layer")?;
        let x_property = PropertyId::new(property::POSITION_X).map_err(|e| e.to_string())?;
        let split = self.doc.view().property_source(layer, &x_property).map_err(|e| e.to_string())?.is_some();
        let (property, value) = if split {
            (x_property, Value::F64(value))
        } else {
            let mut point = self.position(layer)?;
            point[0] = value;
            (PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, Value::Vec2(point))
        };
        self.doc.place_checked(layer, &property, value, self.time()?, self.animate)
            .map(|edit| edit.into_iter().collect()).map_err(|e| e.to_string())
    }





    fn render(&mut self, surface_id: u32) -> Result<(), String> {
        let started = Instant::now();
        let surface = IOSurfaceRef::lookup(surface_id).ok_or("IOSurface lookup failed")?;
        let comp = self.doc.view().composition().map_err(|e| e.to_string())?.ok_or("No composition")?;
        if surface.width() != comp.width as usize || surface.height() != comp.height as usize {
            return Err("IOSurface dimensions differ from composition".into());
        }
        if surface.pixel_format() != u32::from_be_bytes(*b"BGRA") { return Err("IOSurface must be BGRA".into()); }
        let device = self.engine.gpu_device();
        let descriptor = MTLTextureDescriptor::new();
        unsafe {
            descriptor.setTextureType(MTLTextureType::Type2D);
            descriptor.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            descriptor.setWidth(comp.width as usize);
            descriptor.setHeight(comp.height as usize);
            descriptor.setMipmapLevelCount(1);
        }
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or("Metal device unavailable")?;
        let raw = hal.raw_device().newTextureWithDescriptor_iosurface_plane(&descriptor, &surface, 0)
            .ok_or("Metal could not bind IOSurface")?;
        let size = wgpu::Extent3d { width: comp.width, height: comp.height, depth_or_array_layers: 1 };
        // Same-device imported attachment. The host exclusively owns a fresh surface;
        // the existing compositor clears and writes it before this function publishes it.
        let texture = unsafe {
            let raw = wgpu::hal::metal::Device::texture_from_raw(raw, wgpu::TextureFormat::Bgra8Unorm,
                MTLTextureType::Type2D, 1, 1, size.into());
            device.create_texture_from_hal::<wgpu::hal::api::Metal>(raw, &wgpu::TextureDescriptor {
                label: Some("Motolii direct IOSurface output"), size, mip_level_count: 1, sample_count: 1,
                dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING, view_formats: &[],
            })
        };
        drop(hal);
        let time = self.time()?;
        let view_camera = self.view_camera()?;
        self.engine.render_frame_into_with_camera(&self.doc.view(), time, &texture, view_camera, true).map_err(|e|e.to_string())?;
        self.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely()).map_err(|e|e.to_string())?;
        self.render_count += 1;
        self.render_ms = started.elapsed().as_secs_f64() * 1000.0;
        self.error = if self.engine.layer_failures().is_empty() { None } else { Some(self.engine.layer_failures().join("; ")) };
        Ok(())
    }
}

#[no_mangle]
pub unsafe extern "C" fn motolii_probe_open(path: *const c_char) -> *mut EditorRuntime {
    match catch_unwind(AssertUnwindSafe(|| {
        if path.is_null() { return Err("Missing project path".to_string()); }
        let path = unsafe { CStr::from_ptr(path) }.to_str().map_err(|e|e.to_string())?;
        EditorRuntime::open(path).map(|probe| Box::into_raw(Box::new(probe)))
    })) {
        Ok(Ok(probe)) => probe,
        result => { eprintln!("Motolii probe open failed: {}", match result { Ok(Err(e)) => e, _ => "panic".into() }); std::ptr::null_mut() }
    }
}

#[no_mangle]
pub unsafe extern "C" fn motolii_probe_request(ctx: *mut EditorRuntime, request: *const c_char) -> *const c_char {
    if ctx.is_null() { return c"{\"error\":\"No probe context\"}".as_ptr(); }
    let probe = unsafe { &mut *ctx };
    let before_image = probe.image_key();
    let mut known = None;
    let mut known_references = None;
    let mut defer_snapshot = false;
    let mut quiet = false;
    let mut model_reply = None;
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        if request.is_null() { return Err("Missing request".into()); }
        let bytes = unsafe { CStr::from_ptr(request) }.to_bytes();
        let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e|e.to_string())?;
        known = value["knownSnapshotId"].as_u64();
        known_references = value["knownReferenceId"].as_u64();
        defer_snapshot = value["deferSnapshot"] == true;
        if value["bootstrap"] == true { *probe.full_status_revision.borrow_mut() = None; }
        if value["op"] == "renderInfo" {
            model_reply = Some(probe.doc.view().composition().map_err(|e|e.to_string()).and_then(|comp| {
                comp.map(|c|json!({"width":c.width,"height":c.height})).ok_or("No composition".into())
            }));
            return Ok(());
        }
        if value["op"] == "visualSample" {
            model_reply = Some(editor::visual_samples::reply(&probe.doc, probe.time()?, &value));
            return Ok(());
        }
        if value["op"] == "easeModel" {
            model_reply = Some(editor::ease_kinds::model(&value));
            return Ok(());
        }
        quiet = value["quiet"] == true && (value["op"] == "seek" || value["op"] == "tick");
        probe.request(value)
    }));
    match outcome { Ok(Ok(())) => {}, Ok(Err(e)) => probe.error=Some(e), Err(_) => probe.error=Some("Rust request panic".into()) }
    if let Some(model) = model_reply {
        let value = model.unwrap_or_else(|error| json!({"error":error}));
        probe.reply = CString::new(value.to_string()).unwrap();
        return probe.reply.as_ptr();
    }
    if quiet && probe.error.is_none() {
        probe.reply = CString::new("{\"ok\":true}").unwrap();
        return probe.reply.as_ptr();
    }
    let needs_render = before_image != probe.image_key();
    if defer_snapshot && needs_render && probe.error.is_none() {
        probe.reply = CString::new("{\"needsRender\":true}").unwrap();
        return probe.reply.as_ptr();
    }
    let status = catch_unwind(AssertUnwindSafe(|| probe.status_response(known, known_references)));
    let mut value = match status { Ok(Ok(v)) => v, Ok(Err(e)) => json!({"error":e}), Err(_) => json!({"error":"Rust status panic"}) };
    value["needsRender"] = json!(needs_render);
    probe.reply = CString::new(value.to_string()).unwrap_or_else(|_| CString::new("{\"error\":\"Invalid reply\"}").unwrap());
    probe.reply.as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn motolii_probe_render(ctx: *mut EditorRuntime, surface_id: u32) -> i32 {
    if ctx.is_null() { return -1; }
    let probe = unsafe { &mut *ctx };
    match catch_unwind(AssertUnwindSafe(|| probe.render(surface_id))) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => { probe.error=Some(error); -1 },
        Err(_) => { probe.error=Some("Rust render panic".into()); -2 },
    }
}

#[no_mangle]
pub unsafe extern "C" fn motolii_probe_close(ctx: *mut EditorRuntime) {
    if !ctx.is_null() { let _ = catch_unwind(AssertUnwindSafe(|| drop(unsafe { Box::from_raw(ctx) }))); }
}
