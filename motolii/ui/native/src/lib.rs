#![recursion_limit = "256"]
pub use motolii_doc as doc;
pub use motolii_edit as edit;
pub use motolii_render as render;
#[allow(unused_imports)]
use crate::edit::{Animate, Document, Intent};
mod editor;
mod frames;
mod owners;
mod viewer;
mod port;
mod snapshot;
mod snapshot_cache;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use motolii_doc::store::{property, LayerId, PropertyId, RationalTime, Value};
use motolii_render::engine::{Engine, Window};
use viewer::View;
use objc2_io_surface::IOSurfaceRef;
use objc2_metal::{MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage};
use serde_json::json;

pub struct EditorRuntime {
    doc: Document,
    engine: Engine,
    viewer: viewer::ViewerState,
    clipboard: editor::clipboard::Clipboard,
    path: Option<String>,
    saved_signature: String,
    exporter: motolii_jobs::export::ExportController,
    freezer: motolii_jobs::freeze::FreezeController,
    device_id: u64,
    render_count: u64,
    render_ms: f64,
    reply: CString,
    error: Option<String>,
    preview: Option<(u64, Vec<Intent>)>,
    preview_tag: Option<String>,
    stage_drag: Option<editor::stage::DragSession>,
    pub(crate) full_status_revision: std::cell::RefCell<Option<String>>,
    pub(crate) flat_projection: crate::doc::store::LayerProjection,
    snapshot_cache: std::cell::RefCell<snapshot_cache::SnapshotCache>,
    pub(crate) history: editor::history::Ledger,
    effects_watch: Option<motolii_render::engine::CatalogWatcher>,
    last_script: Option<(String, i64, i64)>,
    /// The source the last script file ran with: a save that changes it reruns the script.
    last_script_source: Option<String>,
    /// How a file watcher wakes the window's thread (set with the effect watch).
    wake: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    script_watch: Option<notify::RecommendedWatcher>,
    /// 出したコマの列。再生中は「出して返る」ので、描き終わりはここが次の拍で拾う。
    frames: frames::Frames,
    /// 再生中の 1 コマを持ち主ごとに畳む。▶ で空にし、Ⅱ で 1 度だけ出す。
    owners: owners::FrameOwners,
    /// 面ごとに前回描いた窓(roi・寸法・投影)。同じ窓のまま時刻だけ動く道
    /// —— 時間帯のドラッグ —— は、絵と窓を揃えるために待つ必要がない。
    last_window: std::collections::HashMap<&'static str, Window>,
    /// host の再生 pulse が既に提出した時刻。Flutter の vsync ではなく native
    /// の時計がこれを進め、同じ作中コマを二度 CPU で解かない。
    playback_rendered_frame: Option<i64>,
}

pub use frames::FrameReady;

impl Drop for EditorRuntime {
    fn drop(&mut self) {
        self.frames.finish();
        let _ = self.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely());
    }
}

/// この編集機が扱う作品は、必ずここを通って生まれる。効果を渡すのはこの 1 箇所だけ
/// (コアの既定は効果ゼロなので、別の道で作った作品は効果が黙って効かない)。
pub(crate) fn work(path: Option<&str>) -> Result<Document, String> {
    let doc = match path {
        None => crate::edit::blank_project(),
        Some(path) => Document::load(path).map_err(|e| e.to_string())?,
    };
    Ok(doc.with_programs(render::extensions::bundled()).with_geometry(render::extensions::geometry()))
}

impl EditorRuntime {
    fn open(path: &str) -> Result<Self, String> {
        let doc = work(if path.is_empty() { None } else { Some(path) })?;
        if doc.view().composition().map_err(|e| e.to_string())?.is_none() {
            return Err("Saved document has no composition".into());
        }
        let mut engine = Engine::new().map_err(|e| e.to_string())?;
        engine.set_cache_root(Self::cache_root_for(if path.is_empty() { None } else { Some(path) }));
        let device_id = unsafe { engine.gpu_device().as_hal::<wgpu::hal::api::Metal>() }
            .ok_or("Engine did not create a Metal device")?.raw_device().registryID();
        let saved_signature = snapshot::authored_signature(&doc)?;
        let viewer = viewer::ViewerState::new(&doc.view(), doc.revision());
        let frames = frames::Frames::new(engine.gpu_device());
        let mut history = editor::history::Ledger::open(editor::history::default_file());
        history.record("open", if path.is_empty() { "New document".to_owned() } else { path.rsplit('/').next().unwrap_or(path).to_owned() }, Some(doc.edit_head()));
        Ok(Self {
            doc, engine, viewer, clipboard: Default::default(),
            path: if path.is_empty() { None } else { Some(path.into()) }, saved_signature,
            exporter: Default::default(), freezer: Default::default(), device_id,
            render_count: 0, render_ms: 0.0, reply: CString::new("{}").unwrap(), error: None,
            preview: None, preview_tag: None, stage_drag: None,
            snapshot_cache: Default::default(), full_status_revision: Default::default(),
            flat_projection: crate::doc::store::LayerProjection::TwoPointFiveD,
            history, effects_watch: None, last_script: None, last_script_source: None, wake: None, script_watch: None, frames,
            owners: Default::default(), last_window: Default::default(),
            playback_rendered_frame: None,
        })
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
        RationalTime::try_from_frame(self.viewer.frame, comp.fps).map_err(|e| e.to_string())
    }

    fn position(&self, layer: LayerId) -> Result<[f64; 2], String> {
        self.position_in(&self.doc.view(), layer)
    }

    /// 層ごとに view を作り直さない形。status は 1 コマに層の数だけ呼ぶ。
    fn position_in(&self, view: &crate::doc::store::StoreView<'_>, layer: LayerId) -> Result<[f64; 2], String> {
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
        let layer = self.viewer.selected().ok_or("Select a layer")?;
        let x_property = PropertyId::new(property::POSITION_X).map_err(|e| e.to_string())?;
        let split = self.doc.view().property_source(layer, &x_property).map_err(|e| e.to_string())?.is_some();
        let (property, value) = if split {
            (x_property, Value::F64(value))
        } else {
            let mut point = self.position(layer)?;
            point[0] = value;
            (PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, Value::Vec2(point))
        };
        self.doc.place_checked(layer, &property, value, self.time()?, self.viewer.animate)
            .map(|edit| edit.into_iter().collect()).map_err(|e| e.to_string())
    }





    /// One app tick: every shown view, drawn from one prepared frame in one submission (Rerun's
    /// app frame). `Ok(false)` = a surface is still being drawn: nothing is drawn this tick.
    fn render_views(&mut self, targets: &[(u32, View)]) -> Result<bool, String> {
        // 前の拍で出した物のうち、GPU が終えている物をここで拾って鳴らす(待たない)。
        self.frames.collect();
        if !targets.iter().all(|(surface, _)| self.frames.claim(*surface)) { return Ok(false); }
        let mut drawn = Vec::with_capacity(targets.len());
        for &(surface_id, view) in targets {
            let window = self.window(view)?;
            drawn.push((surface_id, view, window, self.import_surface(surface_id, view, window)?));
        }
        self.draw_views(&drawn)
    }

    /// The host's IOSurface as this view's target (same device, no copy).
    fn import_surface(&self, surface_id: u32, view: View, window: Window) -> Result<wgpu::Texture, String> {
        let surface = IOSurfaceRef::lookup(surface_id).ok_or("IOSurface lookup failed")?;
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
        // the tick clears and writes it before this function publishes it.
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
        Ok(texture)
    }

    /// 窓を持たない道(試験)。合図の surface は 0。
    #[cfg(test)]
    fn render_into(&mut self, texture: &wgpu::Texture, view: View, window: Window) -> Result<(), String> {
        self.draw_views(&[(0, view, window, texture.clone())]).map(|_| ())
    }

    fn draw_views(&mut self, views: &[(u32, View, Window, wgpu::Texture)]) -> Result<bool, String> {
        let started = Instant::now();
        let time = self.time()?;
        let playing = self.viewer.clock.playing();
        self.engine.set_realtime(playing);
        let requests: Vec<_> = views.iter().map(|(_, view, window, texture)| crate::render::engine::ViewRequest {
            target: texture,
            window: *window,
            // Camera は作品のカメラ(tick が準備した物)、Stage は利用者のカメラ。
            camera: (*view == View::User).then_some(self.viewer.user_camera),
            projection: match view { View::Camera => crate::render::frame_graph::ViewProjection::Camera, View::User => crate::render::frame_graph::ViewProjection::Stage },
            include_background: true,
        }).collect();
        self.engine.tick(&self.doc.view(), time, &requests).map_err(|e| e.to_string())?;
        drop(requests);
        // 出した束の番号。これが終われば、どの surface にも絵が入っている。
        let submission = self.engine.last_submission();
        let mut coherent = self.stage_drag.is_some();
        for (surface_id, view, window, _) in views {
            self.frames.submitted(submission.clone(), view.name(), *surface_id);
            // 絵と窓(roi)が同じコマで揃っていないといけない道 —— 掴む・伸ばす・Fit —— はここで待つ。
            // 窓が同じまま時刻だけ動く道(時間帯のドラッグ)・再生中は待たない。
            coherent |= self.last_window.insert(view.name(), *window) != Some(*window);
        }
        if !playing && coherent { self.frames.finish(); }
        // Playback changes no cage: the geometry cache is invalidated only for a still frame.
        if !playing {
            self.snapshot_cache.borrow_mut().invalidate_geometry();
        }
        self.render_count += 1;
        self.render_ms = started.elapsed().as_secs_f64() * 1000.0;
        // 再生中だけ畳む。止まっている 1 枚は待つ道なので、同じ物差しに混ぜない。
        if playing {
            let duration = self.doc.view().composition().ok().flatten().map(|c| c.duration_frames);
            if duration.is_some_and(|last| self.viewer.frame >= last) {
                self.owners.outside();
            } else {
                self.owners.push("tick", self.engine.frame_measurement());
                self.owners.push_inside("tick", self.engine.resolve_tally(), self.engine.resolve_worst());
            }
        }
        self.error = if self.engine.layer_failures().is_empty() { None } else { Some(self.engine.layer_failures().join("; ")) };
        Ok(true)
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
                // 描く窓の一覧。Camera はタブの見せる密度(無ければ出力そのもの)、Stage はタブが窓を置いている間だけ。
                let camera = probe.viewer.camera_window.map_or([c.width, c.height], |w| [w.width, w.height]);
                let mut views = vec![json!({"view":View::Camera.name(),"width":camera[0],"height":camera[1]})];
                if let Some(w) = probe.viewer.stage_window { views.push(json!({"view":View::User.name(),"width":w.width,"height":w.height})); }
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
        if value["op"] == "fontFacts" {
            model_reply = Some(Ok(json!({"facts": crate::render::picture::shaping::font_facts()})));
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
            let seen = probe.viewer.stage_view;
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
        // 静かな tick/seek でも「絵が動いたか」は返す。動いていなければ窓は描かない。
        let moved = before_image != probe.image_key();
        probe.reply = CString::new(format!("{{\"ok\":true,\"needsRender\":{moved}}}")).unwrap();
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

/// One app tick: draw every shown view, each into its IOSurface, from one prepared frame in one
/// submission. `surfaces[i]` / `views[i]` name the targets. 0 = drawn, 1 = a surface is still
/// being drawn (nothing was drawn; try again next pulse), negative = error.
#[no_mangle]
pub unsafe extern "C" fn motolii_probe_tick(ctx: *mut EditorRuntime, surfaces: *const u32, views: *const *const c_char, count: usize) -> i32 {
    if ctx.is_null() || (count > 0 && (surfaces.is_null() || views.is_null())) { return -1; }
    let probe = unsafe { &mut *ctx };
    let targets: Result<Vec<(u32, View)>, String> = (0..count).map(|i| {
        let view = unsafe { *views.add(i) };
        let name = (!view.is_null()).then(|| unsafe { CStr::from_ptr(view) }.to_str().ok()).flatten().ok_or("view name missing")?;
        Ok((unsafe { *surfaces.add(i) }, View::parse(name)?))
    }).collect();
    match catch_unwind(AssertUnwindSafe(|| targets.and_then(|targets| probe.render_views(&targets)))) {
        Ok(Ok(true)) => 0,
        Ok(Ok(false)) => 1,
        Ok(Err(error)) => { probe.error=Some(error); -1 },
        Err(_) => { probe.error=Some("Rust render panic".into()); -2 },
    }
}

/// Native host の再生 pulse。Dart の `Ticker` は時計もレンダラも駆動しない。
///
/// 新しい作中コマならその番号、同じコマ/停止中なら -1 を返す。host は番号を
/// 受けた時だけ、手元にある IOSurface へ `motolii_probe_tick` を出す。
#[no_mangle]
pub unsafe extern "C" fn motolii_probe_playback_tick(ctx: *mut EditorRuntime) -> i64 {
    if ctx.is_null() { return -2; }
    let probe = unsafe { &mut *ctx };
    if !probe.viewer.clock.playing() { return -1; }
    probe.frames.collect();
    probe.viewer.clock.poll_audio(&probe.doc.view());
    probe.clock_frame();
    let frame = probe.viewer.frame;
    if probe.playback_rendered_frame == Some(frame) { return -1; }
    probe.playback_rendered_frame = Some(frame);
    frame
}

/// Register `ready(user, view, surface_id)` for completed renders. It always runs
/// on the calling thread: a still frame is waited for inside `motolii_probe_tick`,
/// a playing one is picked up at the head of the next `motolii_probe_tick` (or by
/// `motolii_probe_finish_frames`), once the GPU has it.
/// Removing/replacing the registration cancels old signals. A callback must not
/// re-enter this setter; `user` must live until unregister returns.
#[no_mangle]
pub unsafe extern "C" fn motolii_probe_set_frame_ready(ctx: *mut EditorRuntime, ready: Option<FrameReady>, user: *mut std::ffi::c_void) -> i32 {
    if ctx.is_null() { return -1; }
    let probe = unsafe { &mut *ctx };
    probe.frames.set_ready(ready, user);
    0
}

/// 出した絵が GPU から出て来るまで待つ、同期の口。合図(`FrameReady`)を使わない道
/// —— channel 越しの `render`(毎回その場で作った IOSurface を返す)—— はこれを挟む。
#[no_mangle]
pub unsafe extern "C" fn motolii_probe_finish_frames(ctx: *mut EditorRuntime) -> i32 {
    if ctx.is_null() { return -1; }
    unsafe { &mut *ctx }.frames.finish();
    0
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
    let wake: std::sync::Arc<dyn Fn() + Send + Sync> = std::sync::Arc::new(move || unsafe { wake(user.pointer()) });
    probe.wake = Some(wake.clone());
    match motolii_render::engine::watch_effect_catalog(move || wake()) {
        Ok(watch) => { probe.effects_watch = Some(watch); 0 }
        Err(error) => { probe.error = Some(format!("Effect watch: {error}")); -2 }
    }
}

#[no_mangle]
pub unsafe extern "C" fn motolii_probe_close(ctx: *mut EditorRuntime) {
    if !ctx.is_null() { let _ = catch_unwind(AssertUnwindSafe(|| { let mut probe = unsafe { Box::from_raw(ctx) }; probe.history.close(); drop(probe); })); }
}

#[cfg(test)]
mod frame_ready_tests {
    use super::*;
    use objc2_core_foundation::{CFDictionary, CFNumber, CFString};

    unsafe extern "C" fn count(user: *mut std::ffi::c_void, view: *const c_char, surface: u32) {
        let seen = unsafe { &mut *(user as *mut Vec<(String, u32)>) };
        seen.push((unsafe { CStr::from_ptr(view) }.to_str().unwrap().to_owned(), surface));
    }

    /// 描くたびに合図が 1 回、描いた view と surface を持って、同じ thread で来る。外せば来ない。
    /// 止まっている 1 枚は描画の中で待つので、`finish` はその念押し(再生中の道と同じ形で書く)。
    #[test]
    fn the_frame_ready_signal_fires_once_per_render() {
        let mut rt = EditorRuntime::open("").unwrap();
        let comp = rt.doc.view().composition().unwrap().unwrap().spec();
        let keys: Vec<&CFString> = unsafe { vec![objc2_io_surface::kIOSurfaceWidth, objc2_io_surface::kIOSurfaceHeight, objc2_io_surface::kIOSurfaceBytesPerElement, objc2_io_surface::kIOSurfacePixelFormat] };
        let values = [CFNumber::new_i32(comp.width as i32), CFNumber::new_i32(comp.height as i32), CFNumber::new_i32(4), CFNumber::new_i32(u32::from_be_bytes(*b"BGRA") as i32)];
        let values: Vec<&CFNumber> = values.iter().map(|v| &**v).collect();
        let properties = CFDictionary::from_slices(&keys, &values);
        let surface = unsafe { IOSurfaceRef::new(properties.as_opaque()) }.expect("IOSurface");
        let mut seen: Vec<(String, u32)> = Vec::new();
        let user = &mut seen as *mut _ as *mut std::ffi::c_void;
        assert_eq!(unsafe { motolii_probe_set_frame_ready(&mut rt, Some(count), user) }, 0);
        for _ in 0..2 {
            assert_eq!(unsafe { motolii_probe_tick(&mut rt, &surface.id(), &c"Camera".as_ptr(), 1) }, 0, "{:?}", rt.error);
            rt.frames.finish();
        }
        assert_eq!(seen, vec![("Camera".to_owned(), surface.id()); 2]);
        // 失敗した描画は合図しない。
        assert_ne!(unsafe { motolii_probe_tick(&mut rt, &surface.id(), &c"Nowhere".as_ptr(), 1) }, 0);
        rt.frames.finish();
        assert_eq!(seen.len(), 2);
        assert_eq!(unsafe { motolii_probe_set_frame_ready(&mut rt, None, std::ptr::null_mut()) }, 0);
        assert_eq!(unsafe { motolii_probe_tick(&mut rt, &surface.id(), &c"Camera".as_ptr(), 1) }, 0);
        rt.frames.finish();
        assert_eq!(seen.len(), 2);
        // 番人: まだ合図の来ていない面には描かない(描画はこれを見て 1 を返す)。飛ばした数だけ増える。
        rt.frames.submitted(None, "Camera", surface.id());
        assert!(!rt.frames.claim(surface.id()), "同じ面には描かない");
        assert!(rt.frames.claim(surface.id() + 1), "もう 1 枚の面は空いている");
        assert_eq!(rt.frames.skipped(), 1);
        rt.frames.finish();
    }
}
