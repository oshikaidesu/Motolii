//! P11: Browserパネル（外部フォルダ参照 + サムネイル + D&D）が Blitz で成立するか。
//!
//! 完成条件を塞いでいる `N-MEDIA-PICK` / `N-PROJECT-ENTRY`（ファイルを選ぶ入口が無い）に対し、
//! **native file dialog を待たずにフォルダ参照で代替できるか**を確かめる。
//!
//! 検証:
//!   1. `blitz-net` の `file` スキームで `<img src="file://...">` が実ファイルを読むか
//!   2. サムネイル格子が組めるか（自前テクスチャ不要か）
//!   3. 項目を掴んでドラッグできるか（Browser→Stage/Timeline の配置操作の前提）
//!
//! 実行:
//!   BLITZ_BROWSER_DIR=/path/to/folder cargo run --release --bin browser_panel

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
use rustc_hash::FxHashMap;
use std::sync::Arc;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const W: u32 = 900;
const H: u32 = 520;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const CELL_W: f64 = 132.0;
const CELL_H: f64 = 108.0;
const COLS: usize = 6;
const PAD: f64 = 10.0;
const TOP: f64 = 34.0;

fn max_items() -> usize {
    std::env::var("BLITZ_BROWSER_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(60)
}

fn scan_dir() -> (String, Vec<(String, String)>) {
    let dir = std::env::var("BLITZ_BROWSER_DIR").unwrap_or_else(|_| {
        // 既定はリポジトリのモック画像
        "/Users/member_ottoto/rust_ae/Motolii/docs/mocks".to_string()
    });
    let mut items = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let ext = p
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                items.push((name, p.to_string_lossy().to_string()));
            }
            if items.len() >= max_items() {
                break;
            }
        }
    }
    (dir, items)
}

fn build_html(dir: &str, items: &[(String, String)], sel: Option<usize>, dragging: Option<usize>) -> String {
    let mut b = String::new();
    b.push_str(&format!(
        r#"<div class="hdr">Browser — {dir}　({}件)</div>"#,
        items.len()
    ));
    if items.is_empty() {
        b.push_str(
            r#"<div class="empty">画像が見つかりません。BLITZ_BROWSER_DIR で別フォルダを指定してください。</div>"#,
        );
    }
    for (i, (name, path)) in items.iter().enumerate() {
        let col = i % COLS;
        let row = i / COLS;
        let x = PAD + col as f64 * (CELL_W + PAD);
        let y = TOP + row as f64 * (CELL_H + PAD);
        let cls = if Some(i) == dragging {
            "card drag"
        } else if Some(i) == sel {
            "card sel"
        } else {
            "card"
        };
        b.push_str(&format!(
            r#"<div class="{cls}" style="left:{x}px;top:{y}px">
                 <img class="th" src="file://{path}" />
                 <div class="nm">{name}</div>
               </div>"#
        ));
    }
    format!(
        r#"<html><head><style>
  html,body {{ margin:0; padding:0; width:{W}px; height:{H}px;
              background:#2a2a2a; font-family:sans-serif; color:#d6d6d6; }}
  .hdr {{ position:absolute; left:0; top:0; width:{W}px; height:24px;
          background:#363636; color:#919191; font-size:11px; padding:7px 8px;
          border-bottom:1px solid #111111; overflow:hidden; white-space:nowrap; }}
  .empty {{ position:absolute; left:12px; top:48px; font-size:11px; color:#d69a8b; }}
  .card {{ position:absolute; width:{CELL_W}px; height:{CELL_H}px; background:#363636;
           border:1px solid #242424; }}
  .card:hover {{ background:#414141; border-color:#6a6a6a; }}
  .sel {{ border:1px solid #ffad56; }}
  .drag {{ background:#4a4a4a; border:1px solid #ffffff; }}
  .th {{ position:absolute; left:4px; top:4px; width:{}px; height:{}px;
         background:#242424; object-fit:contain; }}
  .nm {{ position:absolute; left:4px; top:{}px; width:{}px; font-size:9px;
         color:#d6d6d6; overflow:hidden; white-space:nowrap; }}
</style></head><body>{b}</body></html>"#,
        CELL_W - 8.0,
        CELL_H - 24.0,
        CELL_H - 18.0,
        CELL_W - 8.0
    )
}

fn main() -> eframe::Result<()> {
    // blitz_net::Provider は Tokio reactor を要求する。
    // 実装でも同じで、Blitzのファイル/ネットワーク取得を使うなら runtime を張る必要がある。
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _guard = rt.enter();
    eframe::run_native(
        "P11: Browserパネル（フォルダ参照 + サムネイル + D&D）",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([940.0, 640.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    doc: HtmlDocument,
    r: Renderer,
    res: Resources,
    view: wgpu::TextureView,
    tex: egui::TextureId,
    dh: DeviceHandle,
    dir: String,
    items: Vec<(String, String)>,
    sel: Option<usize>,
    dragging: Option<usize>,
    drop_note: String,
    dirty: bool,
    /// **フレームを跨いで保持する**。毎フレーム作り直すと画像が再確保され続け、
    /// 数秒で atlas を埋めて `AtlasLimitReached` で panic する(実測で踏んだ)。
    img_cache: FxHashMap<u64, vello_common::paint::ImageId>,
    tex_bindings: FxHashMap<anyrender::ResourceId, wgpu::TextureView>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu");
        let texture = rs.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("browser"),
            size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let tex = rs.renderer.write().register_native_texture(
            &rs.device,
            &view,
            wgpu::FilterMode::Linear,
        );
        let (dir, items) = scan_dir();
        eprintln!("P11 scan: {dir} → {} 件", items.len());
        let doc = make_doc(&dir, &items, None, None);
        Self {
            doc,
            r: Renderer::new(&rs.device, &RenderTargetConfig { format: FORMAT, width: W, height: H }),
            res: Resources::new(),
            view,
            tex,
            dh: DeviceHandle {
                instance: wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
                adapter: rs.adapter.clone(),
                device: rs.device.clone(),
                queue: rs.queue.clone(),
            },
            dir,
            items,
            sel: None,
            dragging: None,
            drop_note: String::new(),
            dirty: false,
            img_cache: FxHashMap::default(),
            tex_bindings: FxHashMap::default(),
        }
    }

    fn index_at(&self, x: f64, y: f64) -> Option<usize> {
        if y < TOP {
            return None;
        }
        let col = ((x - PAD) / (CELL_W + PAD)).floor();
        let row = ((y - TOP) / (CELL_H + PAD)).floor();
        if col < 0.0 || row < 0.0 || col as usize >= COLS {
            return None;
        }
        let i = row as usize * COLS + col as usize;
        (i < self.items.len()).then_some(i)
    }
}

fn make_doc(dir: &str, items: &[(String, String)], sel: Option<usize>, dragging: Option<usize>) -> HtmlDocument {
    let mut doc = HtmlDocument::from_html(
        &build_html(dir, items, sel, dragging),
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            net_provider: Some(blitz_net::Provider::shared(None)),
            ..Default::default()
        },
    );
    doc.set_viewport(Viewport {
        window_size: (W, H),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);
    doc
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.ctx().request_repaint();
        ui.label(
            egui::RichText::new("P11: 外部フォルダを参照してサムネイル表示 + 掴んでドラッグ")
                .color(egui::Color32::from_rgb(0xff, 0xad, 0x56)),
        );
        ui.label(format!("dir: {}　items: {}　{}", self.dir, self.items.len(), self.drop_note));
        ui.separator();

        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(W as f32, H as f32), egui::Sense::click_and_drag());
        if let Some(p) = resp.hover_pos().or_else(|| ui.ctx().pointer_latest_pos()) {
            let (lx, ly) = ((p.x - rect.min.x) as f64, (p.y - rect.min.y) as f64);
            {
                let mut d = EventDriver::new(&mut *self.doc, NoopEventHandler);
                d.handle_ui_event(UiEvent::PointerMove(pointer_at(lx as f32, ly as f32)));
            }
            if resp.drag_started() {
                if let Some(i) = self.index_at(lx, ly) {
                    self.dragging = Some(i);
                    self.sel = Some(i);
                    self.dirty = true;
                }
            }
            if resp.drag_stopped() {
                if let Some(i) = self.dragging.take() {
                    // 実装ではここが Browser→Stage/Timeline の配置intentになる
                    self.drop_note = format!("drop: {}", self.items[i].0);
                    self.dirty = true;
                }
            }
            if resp.clicked() {
                self.sel = self.index_at(lx, ly);
                self.dirty = true;
            }
        }

        if self.dirty {
            self.doc = make_doc(&self.dir, &self.items, self.sel, self.dragging);
            self.dirty = false;
        }
        self.doc.resolve(0.0);

        let rs = frame.wgpu_render_state().unwrap();
        let (device, queue) = (rs.device.clone(), rs.queue.clone());
        let mut scene = Scene::new(W as u16, H as u16);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let im = ImageManager::new(
                &mut self.r, &mut self.res, &device, &queue, &mut enc, &mut self.img_cache,
            );
            let mut p = VelloHybridScenePainter::new(&mut scene, im, &mut self.tex_bindings, &self.dh);
            paint_scene(&mut p, &mut self.doc, 1.0, W, H, 0, 0);
        }
        let _ = self.r.render(
            &scene, &mut self.res, &device, &queue, &mut enc,
            &RenderSize { width: W, height: H }, &self.view, &TextureBindings::default(),
        );
        queue.submit([enc.finish()]);

        ui.painter().image(
            self.tex,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        // ドラッグ中の見た目（ホスト側が絵を持つ = P6で確認した原則）
        if let (Some(i), Some(p)) = (self.dragging, ui.ctx().pointer_latest_pos()) {
            ui.painter().rect_filled(
                egui::Rect::from_center_size(p, egui::vec2(120.0, 22.0)),
                2.0,
                egui::Color32::from_black_alpha(200),
            );
            ui.painter().text(
                p,
                egui::Align2::CENTER_CENTER,
                &self.items[i].0,
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );
        }
    }
}

fn pointer_at(x: f32, y: f32) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords { page_x: x, page_y: y, screen_x: x, screen_y: y, client_x: x, client_y: y },
        button: MouseEventButton::Main,
        buttons: MouseEventButtons::None,
        mods: Modifiers::default(),
        details: PointerDetails {
            pressure: 0.0, tangential_pressure: 0.0, tilt_x: 0, tilt_y: 0,
            twist: 0, altitude: 0.0, azimuth: 0.0,
        },
        element: Point { x, y },
        active_pointers: Arc::new(AtomicRefCell::new(Vec::new())),
    }
}
