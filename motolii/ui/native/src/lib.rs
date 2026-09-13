#![recursion_limit = "256"]
pub use motolii_doc as doc;
pub use motolii_render as render;
mod editor;
mod port;
mod snapshot;
mod snapshot_cache;
mod export_job;
mod freeze_job;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use motolii_doc::store::{property, Animate, Document, Intent, LayerId, PropertyId, RationalTime, Value};
use motolii_render::engine::{Engine, Window};
use snapshot::View;
use objc2_io_surface::IOSurfaceRef;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage};
use serde_json::json;

pub struct EditorRuntime {
    doc: Document,
    engine: Engine,
    selected: Option<LayerId>,
    selected_ids: Vec<LayerId>,
    /// 直前の Stage 描画で選ばれた層が描かれた画面上の範囲(絵そのものの籠)。
    /// view ごと、描いた画素から届いた選択の広がり(comp 画像の px)。
    selection_bounds: std::collections::HashMap<View, std::collections::HashMap<LayerId, [f32; 4]>>,
    selected_keys: Vec<editor::session::KeySel>,
    clipboard: editor::clipboard::Clipboard,
    path: Option<String>,
    saved_signature: String,
    color_target: Option<editor::session::ColorSlot>,
    exporter: export_job::ExportController,
    /// Freeze の裏仕事(層のコマを焼く)。
    freezer: freeze_job::FreezeController,
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
    /// Stage の上の pointer(comp 座標)と、画面 px / comp px。3D ギズモの見た目と掴みやすさに使う。
    stage_pointer: Option<[f64;2]>,
    stage_view_scale: f64,
    /// 押している P / R / S。3D ギズモをその 1 種に絞る。
    stage_held: Option<String>,
    /// Stage タブの窓(タブの画素寸法と関心域)。無ければ Stage は隠れていて描かない。
    stage_window: Option<Window>,
    /// 最後に pointer が乗った view。3D ギズモの hover はその view にだけ出る。
    stage_view: View,
    pub(crate) animate: Animate,
    /// 最後に全部入りの status を送った時の Document の版。同じ版で再生中なら生値だけ送る。
    pub(crate) full_status_revision: std::cell::RefCell<Option<String>>,
    user_camera: crate::doc::core::ResolvedCamera,
    /// 設定「New layers」: 新しく作る素材の投影。
    pub(crate) flat_projection: crate::doc::store::LayerProjection,
    snapshot_cache: std::cell::RefCell<snapshot_cache::SnapshotCache>,
    /// 履歴の一本線。編集の段と保存・異常の記録を同じ列に持つ。
    pub(crate) history: editor::history::Ledger,
    /// vism/ の見張り。file が変わると Swift の起こし口を叩き、窓が reloadEffects を送ってくる。
    effects_watch: Option<motolii_render::engine::CatalogWatcher>,
}

impl EditorRuntime {
    fn open(path: &str) -> Result<Self, String> {
        let doc = if path.is_empty() { doc::store::blank_project() } else { Document::load(path).map_err(|e| e.to_string())? };
        if doc.view().composition().map_err(|e| e.to_string())?.is_none() {
            return Err("Saved document has no composition".into());
        }
        let selected = doc.view().layers().first().copied();
        let mut engine = Engine::new().map_err(|e| e.to_string())?;
        engine.set_cache_root(Self::cache_root_for(if path.is_empty() { None } else { Some(path) }));
        let device_id = unsafe { engine.gpu_device().as_hal::<wgpu::hal::api::Metal>() }
            .ok_or("Engine did not create a Metal device")?.raw_device().registryID();
        let saved_signature = snapshot::authored_signature(&doc)?;
        let clock = editor::playback::Clock::from_document(&doc, 60.0);
        clock.sync_document(&doc);
        let clock_revision = doc.revision();
        let mut history = editor::history::Ledger::open(editor::history::default_file());
        history.record("open", if path.is_empty() { "New document".to_owned() } else { path.rsplit('/').next().unwrap_or(path).to_owned() }, Some(doc.edit_head()));
        Ok(Self { selected_ids: selected.into_iter().collect(), selection_bounds: Default::default(), selected_keys: Vec::new(), clipboard: Default::default(), path: if path.is_empty() { None } else { Some(path.into()) }, saved_signature, color_target: None, exporter: Default::default(), freezer: Default::default(), clock, clock_revision, doc, engine, selected, frame: 0, device_id, render_count: 0,
            render_ms: 0.0, picked_color: None, pick_serial: 0, reply: CString::new("{}").unwrap(), error: None, preview: None, preview_tag: None, stage_drag: None, stage_pointer: None, stage_view_scale: 1.0, stage_held: None, snapshot_cache: Default::default(), stage_window: None, stage_view: View::User, animate: Animate::Off, full_status_revision: Default::default(), user_camera: Default::default(), flat_projection: crate::doc::store::LayerProjection::TwoPointFiveD, history, effects_watch: None })
    }

    /// Freeze の cache の置き場: 書類の隣。未保存の書類は temp(保存した時に引っ越さない — Freeze し直す)。
    fn cache_root_for(path: Option<&str>) -> Option<std::path::PathBuf> {
        match path {
            Some(path) => Engine::cache_root_for(std::path::Path::new(path)),
            None => Some(std::env::temp_dir().join(format!("motolii-cache-{}", std::process::id()))),
        }
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





    fn render(&mut self, surface_id: u32, view: View) -> Result<(), String> {
        let surface = IOSurfaceRef::lookup(surface_id).ok_or("IOSurface lookup failed")?;
        let window = self.window(view)?;
        if surface.width() != window.width as usize || surface.height() != window.height as usize {
            return Err(format!("IOSurface dimensions differ from the {} window", view.name()));
        }
        if surface.pixel_format() != u32::from_be_bytes(*b"BGRA") { return Err("IOSurface must be BGRA".into()); }
        let device = self.engine.gpu_device();
        let descriptor = MTLTextureDescriptor::new();
        unsafe {
            descriptor.setTextureType(MTLTextureType::Type2D);
            descriptor.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            descriptor.setWidth(window.width as usize);
            descriptor.setHeight(window.height as usize);
            descriptor.setMipmapLevelCount(1);
        }
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or("Metal device unavailable")?;
        let raw = hal.raw_device().newTextureWithDescriptor_iosurface_plane(&descriptor, &surface, 0)
            .ok_or("Metal could not bind IOSurface")?;
        let size = wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 };
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
        self.render_into(&texture, view, window)
    }

    fn render_into(&mut self, texture: &wgpu::Texture, view: View, window: Window) -> Result<(), String> {
        let started = Instant::now();
        let time = self.time()?;
        let view_camera = self.view_camera(view)?;
        self.engine.set_realtime(self.clock.playing());
        // 再生中はギズモを出さないので、選択の mask も焼かない。
        let outline: &[LayerId] = if self.clock.playing() { &[] } else { &self.selected_ids };
        self.engine.render_frame_into_window(&self.doc.view(), time, texture, view_camera, true, outline, window).map_err(|e|e.to_string())?;
        if self.clock.playing() { let _ = self.engine.warm_upcoming(&self.doc.view(), time); }
        self.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely()).map_err(|e|e.to_string())?;
        self.take_selection_bounds(view, window);
        self.snapshot_cache.borrow_mut().invalidate_geometry();
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
    let head_before = probe.doc.edit_head();
    let op = (!request.is_null()).then(|| unsafe { CStr::from_ptr(request) }.to_str().ok()).flatten()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .map_or(String::new(), |value| value["op"].as_str().unwrap_or_default().to_owned());
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
                let c = comp.ok_or("No composition")?;
                // 描く窓の一覧。Camera は出力そのもの、Stage はタブが窓を置いている間だけ。
                let mut views = vec![json!({"view":View::Camera.name(),"width":c.width,"height":c.height})];
                if let Some(w) = probe.stage_window { views.push(json!({"view":View::User.name(),"width":w.width,"height":w.height})); }
                Ok(json!({"width":c.width,"height":c.height,"views":views}))
            }));
            return Ok(());
        }
        if value["op"] == "stageWindow" {
            model_reply = Some(probe.set_stage_window(&value).map(|changed| json!({"needsRender": changed})));
            return Ok(());
        }
        if value["op"] == "visualSample" {
            let at = probe.time()?;
            model_reply = Some(if value["kind"] == "effect" { editor::effect_sample::reply(&value) } else { editor::visual_samples::reply(&probe.doc, at, &value) });
            return Ok(());
        }
        if value["op"] == "easeModel" {
            model_reply = Some(editor::ease_kinds::model(&value));
            return Ok(());
        }
        quiet = value["quiet"] == true && (value["op"] == "seek" || value["op"] == "tick");
        // hover の時は、ギズモの絵だけ返す。status 全体を組み直さない。
        let hover = value["op"] == "stageGesture" && value["phase"] == "hover";
        probe.request(value)?;
        if hover {
            let seen = probe.stage_view;
            model_reply = Some(probe.spatial_gizmo(seen).map(|gizmo| json!({(if seen == View::User { "stageSpatialGizmo" } else { "spatialGizmo" }): gizmo, "needsRender": false})));
        }
        Ok(())
    }));
    match outcome {
        Ok(Ok(())) => probe.history.note(&op, head_before, probe.doc.edit_head()),
        Ok(Err(e)) => probe.error=Some(e),
        Err(_) => { let message = "Rust request panic"; probe.history.record("error", format!("{message} in {op}"), None); probe.error=Some(message.into()) }
    }
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
pub unsafe extern "C" fn motolii_probe_render(ctx: *mut EditorRuntime, surface_id: u32, view: *const c_char) -> i32 {
    if ctx.is_null() { return -1; }
    let probe = unsafe { &mut *ctx };
    let view = (!view.is_null()).then(|| unsafe { CStr::from_ptr(view) }.to_str().ok()).flatten().unwrap_or("Camera");
    match catch_unwind(AssertUnwindSafe(|| View::parse(view).and_then(|view| probe.render(surface_id, view)))) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => { probe.error=Some(error); -1 },
        Err(_) => { probe.error=Some("Rust render panic".into()); -2 },
    }
}

/// 効果の棚(vism/)の見張りを立てる。file が変わる度に `wake(user)` が別 thread から呼ばれる。
/// 呼ばれた側は自分の thread へ戻してから `{"op":"reloadEffects"}` を送る。
/// 焼き込み build(load_shaders_from_disk 無し)では見張りは空で、0 を返すだけ。
#[no_mangle]
pub unsafe extern "C" fn motolii_probe_watch_effects(ctx: *mut EditorRuntime, wake: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>, user: *mut std::ffi::c_void) -> i32 {
    if ctx.is_null() { return -1; }
    let probe = unsafe { &mut *ctx };
    let Some(wake) = wake else { return -1; };
    // closure は欄ごとに掴む(edition 2021)ので、method 越しに丸ごと掴ませる。
    struct User(*mut std::ffi::c_void);
    unsafe impl Send for User {}
    unsafe impl Sync for User {}
    impl User { fn pointer(&self) -> *mut std::ffi::c_void { self.0 } }
    let user = User(user);
    match motolii_render::engine::watch_effect_catalog(move || unsafe { wake(user.pointer()) }) {
        Ok(watch) => { probe.effects_watch = Some(watch); 0 }
        Err(error) => { probe.error = Some(format!("Effect watch: {error}")); -2 }
    }
}

#[no_mangle]
pub unsafe extern "C" fn motolii_probe_close(ctx: *mut EditorRuntime) {
    if !ctx.is_null() { let _ = catch_unwind(AssertUnwindSafe(|| { let mut probe = unsafe { Box::from_raw(ctx) }; probe.history.close(); drop(probe); })); }
}
