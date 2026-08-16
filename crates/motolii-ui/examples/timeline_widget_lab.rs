//! 現行 `timeline_blitz` を native window で確認するための開発用Lab。
//!
//! P7の eframe → Blitz texture → egui image 経路を使う。ただし Timeline の
//! HTML/CSS、投影、clip/key custom widgetは製品moduleをそのまま呼ぶ。
//! ダミーのTimeline状態を置かないので、C2入力が未配線なことも隠さない。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use atomic_refcell::AtomicRefCell;
use blitz_dom::{DocumentConfig, EventDriver, NoopEventHandler, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use keyboard_types::Modifiers;
use motolii_core::RationalTime;
use motolii_doc::{DocumentWriter, GestureId, KeyframeId, LayerId};
use motolii_ui::timeline_blitz::{
    attach_surface_interactive, project_for_blitz, timeline_html, timeline_html_dom_prototype, TimelineInput,
    TimelinePointerEvent, TimelinePointerPhase, TimelineSurfaceHit,
};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const LOGICAL_W: u32 = 1000;
const LOGICAL_H: u32 = 460;
/// UI合成より2倍細かく描く。高密度画面でも文字とダイヤを潰さない。
const SUPERSAMPLE: f64 = 2.0;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
fn main() -> eframe::Result<()> {
    let Some(project_path) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: timeline_widget_lab <existing-project.json>");
        std::process::exit(2);
    };
    eframe::run_native(
        "Motolii · Timeline widget lab",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([LOGICAL_W as f32 + 24.0, LOGICAL_H as f32 + 84.0]),
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(App::new(cc, &project_path)))),
    )
}

struct App {
    // Lab終了までlockを保持する。fixtureや一時Documentには戻さない。
    _session: motolii_doc::ProjectSession,
    writer: DocumentWriter,
    input: TimelineInput,
    drag: Option<Drag>,
    primary: Option<LayerId>,
    saved_gesture: bool,
    status: String,
    document: HtmlDocument,
    renderer: Renderer,
    resources: Resources,
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    texture_id: egui::TextureId,
    device_handle: DeviceHandle,
    width: u32,
    height: u32,
    scale: f64,
    resolve_ms: f64,
    render_ms: f64,
}

enum Drag {
    Move { layer: LayerId, offset: RationalTime, gesture: GestureId },
    TrimIn { layer: LayerId, gesture: GestureId },
    TrimOut { layer: LayerId, gesture: GestureId },
    Key { layer: LayerId, key: KeyframeId, offset: RationalTime, gesture: GestureId },
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, project_path: &std::path::Path) -> Self {
        let state = cc.wgpu_render_state.as_ref().expect("wgpu backend");
        // eguiはpoint、Blitzのtextureは物理px。Retinaで等倍に合成する。
        let scale = f64::from(cc.egui_ctx.pixels_per_point().max(1.0)) * SUPERSAMPLE;
        let width = (LOGICAL_W as f64 * scale).round() as u32;
        let height = (LOGICAL_H as f64 * scale).round() as u32;
        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("timeline-widget-lab"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_id = state.renderer.write().register_native_texture(
            &state.device,
            &view,
            wgpu::FilterMode::Nearest,
        );

        let catalog = Arc::new(
            motolii_plugin::reference::reference_catalog()
                .expect("reference plugin catalog must be available"),
        );
        let opened = motolii_doc::open_project_resolved(
            project_path,
            &motolii_doc::ResourceLimits::production(),
            &catalog,
        )
        .unwrap_or_else(|error| panic!("timeline lab cannot open {}: {error}", project_path.display()));
        let writer = DocumentWriter::new(opened.recovered.document, catalog)
            .expect("opened project must create its DocumentWriter");
        let (document, input) = timeline_document(&writer.snapshot(), None, scale, width, height);

        Self {
            _session: opened.session,
            writer,
            input,
            drag: None,
            primary: None,
            saved_gesture: false,
            status: "drag clip body / edge / Position key · release persists".to_owned(),
            document,
            renderer: Renderer::new(
                &state.device,
                &RenderTargetConfig {
                    format: FORMAT,
                    width,
                    height,
                },
            ),
            resources: Resources::new(),
            _texture: texture,
            view,
            texture_id,
            width,
            height,
            scale,
            device_handle: DeviceHandle {
                instance: state.adapter.get_info().backend.to_string().parse().ok().map_or_else(
                    || wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
                    |_: u8| wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
                ),
                adapter: state.adapter.clone(),
                device: state.device.clone(),
                queue: state.queue.clone(),
            },
            resolve_ms: 0.0,
            render_ms: 0.0,
        }
    }

    fn rebuild_document(&mut self, scale: f64) {
        let (document, input) = timeline_document(
            &self.writer.snapshot(),
            self.primary,
            scale,
            self.width,
            self.height,
        );
        self.document = document;
        self.input = input;
    }

    fn time_at(&self, fraction: f64) -> Option<RationalTime> {
        let seconds = self.writer.snapshot().composition.duration.as_seconds_f64() * fraction;
        let scaled = seconds * 1_000_000.0;
        RationalTime::try_new(scaled.round() as i64, 1_000_000).ok()
    }

    fn receive_timeline_event(&mut self, event: TimelinePointerEvent, scale: f64) {
        let Some(pointer_time) = self.time_at(event.time_fraction) else { return };
        match event.phase {
            TimelinePointerPhase::Down => self.begin_drag(event.hit, pointer_time),
            TimelinePointerPhase::Move => self.apply_drag(pointer_time, scale),
            TimelinePointerPhase::Up => {
                self.apply_drag(pointer_time, scale);
                if self.saved_gesture {
                    let snapshot = self.writer.snapshot();
                    match self.session_save(&snapshot) {
                        Ok(()) => self.status = "saved to ProjectSession journal".to_owned(),
                        Err(error) => self.status = format!("save failed: {error}"),
                    }
                }
                self.drag = None;
                self.saved_gesture = false;
            }
        }
    }

    fn begin_drag(&mut self, hit: TimelineSurfaceHit, pointer_time: RationalTime) {
        let snapshot = self.writer.snapshot();
        let projection = match project_for_blitz(&snapshot) { Ok(value) => value, Err(_) => return };
        let gesture = self.writer.begin_gesture();
        self.drag = match hit {
            TimelineSurfaceHit::ClipBody { layer } => projection.bars().iter().find(|bar| bar.layer == layer)
                .and_then(|bar| bar.start.try_sub(pointer_time).ok())
                .map(|offset| Drag::Move { layer, offset, gesture }),
            TimelineSurfaceHit::ClipLeft { layer } => Some(Drag::TrimIn { layer, gesture }),
            TimelineSurfaceHit::ClipRight { layer } => Some(Drag::TrimOut { layer, gesture }),
            TimelineSurfaceHit::PositionKey { layer, key } => projection.keys().iter()
                .find(|item| item.layer == layer && item.key == key)
                .and_then(|item| item.t.try_sub(pointer_time).ok())
                .map(|offset| Drag::Key { layer, key, offset, gesture }),
            TimelineSurfaceHit::None => None,
        };
        self.primary = match hit {
            TimelineSurfaceHit::ClipBody { layer }
            | TimelineSurfaceHit::ClipLeft { layer }
            | TimelineSurfaceHit::ClipRight { layer }
            | TimelineSurfaceHit::PositionKey { layer, .. } => Some(layer),
            TimelineSurfaceHit::None => self.primary,
        };
    }

    fn apply_drag(&mut self, pointer_time: RationalTime, scale: f64) {
        let Some(drag) = self.drag.as_ref() else { return };
        let prepared = match *drag {
            Drag::Move { layer, offset, .. } => offset.try_add(pointer_time)
                .ok().map_or_else(|| Err(motolii_doc::CommandError::LayerNotFound(layer.get())), |time| self.writer.prepare_set_clip_start(layer, time)),
            Drag::TrimIn { layer, .. } => self.writer.prepare_trim_clip_in(layer, pointer_time),
            Drag::TrimOut { layer, .. } => self.writer.prepare_trim_clip_out(layer, pointer_time),
            Drag::Key { layer, key, offset, .. } => offset.try_add(pointer_time)
                .ok().map_or_else(|| Err(motolii_doc::CommandError::LayerNotFound(layer.get())), |time| self.writer.prepare_set_position_key_time(layer, key, time)),
        };
        let gesture = match *drag {
            Drag::Move { gesture, .. } | Drag::TrimIn { gesture, .. } | Drag::TrimOut { gesture, .. } | Drag::Key { gesture, .. } => gesture,
        };
        match prepared {
            Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                Ok(()) => {
                    self.saved_gesture = true;
                    self.rebuild_document(scale);
                }
                Err(error) => self.status = format!("edit rejected: {error}"),
            },
            Ok(None) => {}
            Err(error) => self.status = format!("edit rejected: {error}"),
        }
    }

    fn session_save(&mut self, snapshot: &motolii_doc::Document) -> Result<(), Box<motolii_doc::ProjectError>> {
        self._session.save_with_journal(snapshot, &motolii_doc::SaveProjectOptions::default())
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let started = Instant::now();
        let mut scene = Scene::new(self.width as u16, self.height as u16);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("timeline-widget-lab"),
        });
        {
            let mut cache = FxHashMap::default();
            let mut bindings = FxHashMap::default();
            let images = ImageManager::new(
                &mut self.renderer,
                &mut self.resources,
                device,
                queue,
                &mut encoder,
                &mut cache,
            );
            let mut painter = VelloHybridScenePainter::new(
                &mut scene,
                images,
                &mut bindings,
                &self.device_handle,
            );
            paint_scene(
                &mut painter,
                &mut self.document,
                1.0,
                self.width,
                self.height,
                0,
                0,
            );
        }
        let _ = self.renderer.render(
            &scene,
            &mut self.resources,
            device,
            queue,
            &mut encoder,
            &RenderSize {
                width: self.width,
                height: self.height,
            },
            &self.view,
            &TextureBindings::default(),
        );
        queue.submit([encoder.finish()]);
        self.render_ms = started.elapsed().as_secs_f64() * 1000.0;
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.label(egui::RichText::new("Timeline widget lab · current timeline_blitz · ProjectSession").strong());
        ui.label(format!(
            "resolve {:.2} ms · render {:.2} ms · {}",
            self.resolve_ms, self.render_ms, self.status
        ));
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(LOGICAL_W as f32, LOGICAL_H as f32),
            egui::Sense::click_and_drag(),
        );
        let pointer = response.interact_pointer_pos().or_else(|| ui.ctx().pointer_latest_pos());
        if let Some(pointer) = pointer {
            if rect.contains(pointer) || response.dragged() || response.drag_stopped() {
                let local = || pointer_at(pointer.x - rect.min.x, pointer.y - rect.min.y);
                let mut driver = EventDriver::new(&mut *self.document, NoopEventHandler);
                if response.drag_started() {
                    driver.handle_ui_event(UiEvent::PointerDown(local()));
                }
                if response.dragged() {
                    driver.handle_ui_event(UiEvent::PointerMove(local()));
                }
                if response.drag_stopped() {
                    driver.handle_ui_event(UiEvent::PointerUp(local()));
                }
            }
        }
        for event in self.input.drain() {
            self.receive_timeline_event(event, self.scale);
        }

        let resolved = Instant::now();
        self.document.resolve(0.0);
        self.resolve_ms = resolved.elapsed().as_secs_f64() * 1000.0;
        let state = frame.wgpu_render_state().expect("wgpu backend");
        self.render(&state.device, &state.queue);
        ui.painter().image(
            self.texture_id,
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        ui.ctx().request_repaint();
    }
}

fn timeline_document(
    source: &motolii_doc::Document,
    primary: Option<LayerId>,
    scale: f64,
    width: u32,
    height: u32,
) -> (HtmlDocument, TimelineInput) {
    let projection = project_for_blitz(source).expect("opened project timeline");
    let dom_prototype = std::env::var_os("MOTOLII_TIMELINE_DOM_PROTOTYPE").is_some();
    let html = if dom_prototype {
        timeline_html_dom_prototype(source, Some(&projection), primary, RationalTime::ZERO)
    } else {
        timeline_html(source, Some(&projection), primary, RationalTime::ZERO)
    };
    let mut document = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            ..Default::default()
        },
    );
    document.set_viewport(Viewport {
        window_size: (width, height),
        hidpi_scale: scale as f32,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    let input = if dom_prototype {
        TimelineInput::default()
    } else {
        attach_surface_interactive(&mut document, source, Some(&projection), primary)
            .expect("timeline HTML must expose #tl-surface")
    };
    document.resolve(0.0);
    (document, input)
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
