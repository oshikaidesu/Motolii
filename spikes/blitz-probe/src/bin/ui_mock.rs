//! P10: 現行Timeline UI を Blitz(HTML/CSS)で再現できるか。
//!
//! `crates/motolii-ui/src/timeline_egui/` の theme.rs / geometry.rs / ruler.rs / clip_band.rs
//! から実寸・実配色を写して組む。**見た目が持ってこれるか**の判定材料。
//!
//! CSS対応の検証も兼ねる:
//!   - ダイヤ(key)      → transform: rotate(45deg)
//!   - 開閉三角          → border trick
//!   - 選択枠            → inset box-shadow / border
//!   - 縦グリッド        → repeating-linear-gradient
//!
//! 構成は P7 と同じ: eframe が窓と wgpu29 を持ち、Blitz はテクスチャへ描くだけ。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::{DocumentConfig, EventDriver, NoopEventHandler, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use atomic_refcell::AtomicRefCell;
use keyboard_types::Modifiers;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

// ---- timeline_egui/theme.rs から写した実寸・実配色 ----
const W: u32 = 1000;
const H: u32 = 460;
const OVERVIEW_H: f64 = 22.0;
const RULER_H: f64 = 20.0;
const LOCATOR_H: f64 = 16.0;
const ROWS_TOP: f64 = OVERVIEW_H + RULER_H + LOCATOR_H + 1.0; // 59
const SIDEBAR: f64 = 204.0; // clamp(width*0.255, 140, 204)
const ROW_H: f64 = 22.0; // clamp((h-72)/rows, 20, 24)
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const PALETTE: [&str; 6] = [
    "#96aadb", "#6fb9c1", "#bfa973", "#89b992", "#d69a8b", "#c39bc5",
];

struct Row {
    label: &'static str,
    property: bool,
    start: f64,
    end: f64,
    keys: &'static [f64],
    slot: usize,
    selected: bool,
}

const ROWS: &[Row] = &[
    Row { label: "背景プレート", property: false, start: 0.02, end: 0.95, keys: &[], slot: 0, selected: false },
    Row { label: "Position",     property: true,  start: 0.0,  end: 0.0,  keys: &[0.08, 0.24, 0.41, 0.63, 0.88], slot: 0, selected: false },
    Row { label: "タイトル文字",  property: false, start: 0.12, end: 0.58, keys: &[0.16, 0.31, 0.49], slot: 1, selected: true },
    Row { label: "Scale",        property: true,  start: 0.0,  end: 0.0,  keys: &[0.14, 0.33, 0.52], slot: 1, selected: false },
    Row { label: "Opacity",      property: true,  start: 0.0,  end: 0.0,  keys: &[0.12, 0.20, 0.44, 0.57], slot: 1, selected: false },
    Row { label: "ロゴ",          property: false, start: 0.44, end: 0.86, keys: &[], slot: 2, selected: false },
    Row { label: "Rotation",     property: true,  start: 0.0,  end: 0.0,  keys: &[0.46, 0.61, 0.74, 0.85], slot: 2, selected: false },
    Row { label: "パーティクル",  property: false, start: 0.30, end: 1.0,  keys: &[], slot: 3, selected: false },
    Row { label: "光の帯",        property: false, start: 0.05, end: 0.40, keys: &[], slot: 4, selected: false },
    Row { label: "Position",     property: true,  start: 0.0,  end: 0.0,  keys: &[0.06, 0.18, 0.29, 0.37], slot: 4, selected: false },
    Row { label: "カラーグレード", property: false, start: 0.0, end: 1.0,  keys: &[], slot: 5, selected: false },
];

const PLAYHEAD: f64 = 0.37;

fn x_at(f: f64) -> f64 {
    SIDEBAR + f.clamp(0.0, 1.0) * (W as f64 - SIDEBAR)
}

fn build_html() -> String {
    let surface_w = W as f64 - SIDEBAR;
    let mut b = String::new();

    // ---- overview 帯 ----
    b.push_str(&format!(
        r#"<div class="ov" style="left:{SIDEBAR}px;width:{surface_w}px"></div>
           <div class="ovlabel">overview</div>
           <div class="ovborder" style="left:{}px;width:{}px"></div>"#,
        SIDEBAR + 1.0,
        surface_w - 3.0
    ));
    for (i, r) in ROWS.iter().enumerate() {
        if r.property || r.end <= r.start {
            continue;
        }
        b.push_str(&format!(
            r#"<div class="ovbar" style="left:{}px;top:{}px;width:{}px;background:{}"></div>"#,
            x_at(r.start),
            4.0 + i as f64 * 4.0,
            (x_at(r.end) - x_at(r.start)).max(1.0),
            PALETTE[r.slot]
        ));
    }

    // ---- ruler 帯 ----
    b.push_str(&format!(
        r#"<div class="ruler" style="top:{}px"></div><div class="rulerlabel" style="top:{}px">Inbox</div>"#,
        OVERVIEW_H + 1.0,
        OVERVIEW_H + 3.0
    ));
    for i in 0..=10 {
        let x = x_at(i as f64 / 10.0);
        b.push_str(&format!(
            r#"<div class="tick" style="left:{x}px;top:{}px"></div>
               <div class="ticklabel" style="left:{}px;top:{}px">{i}s</div>"#,
            OVERVIEW_H + RULER_H - 5.0,
            x + 3.0,
            OVERVIEW_H + 3.0
        ));
    }
    // ---- locator 帯 ----
    b.push_str(&format!(
        r#"<div class="locator" style="top:{}px"></div>"#,
        OVERVIEW_H + RULER_H + 1.0
    ));

    // ---- 行 ----
    for (i, r) in ROWS.iter().enumerate() {
        let y = ROWS_TOP + i as f64 * ROW_H;
        let cy = y + ROW_H * 0.5;
        b.push_str(&format!(
            r#"<div class="row{}" style="top:{y}px"></div>"#,
            if r.selected { " sel" } else { "" }
        ));
        // sidebar 側
        if r.property {
            b.push_str(&format!(
                r#"<div class="dot" style="top:{}px"></div>
                   <div class="plabel" style="top:{}px">{}</div>"#,
                cy - 1.0,
                cy - 5.0,
                r.label
            ));
        } else {
            b.push_str(&format!(
                r#"<div class="tri" style="top:{}px"></div>
                   <div class="llabel" style="top:{}px">{}</div>
                   <div class="tg" style="left:{}px;top:{}px">M</div>
                   <div class="tg" style="left:{}px;top:{}px">S</div>"#,
                cy - 4.0,
                cy - 5.0,
                r.label,
                SIDEBAR - 48.0,
                cy - 6.0,
                SIDEBAR - 31.0,
                cy - 6.0
            ));
        }
        // clip バー
        if !r.property && r.end > r.start {
            let x0 = x_at(r.start);
            let x1 = x_at(r.end);
            b.push_str(&format!(
                r#"<div class="bar{}" style="left:{x0}px;top:{}px;width:{}px;background:{}">{}</div>"#,
                if r.selected { " selbar" } else { "" },
                y + 1.0,
                (x1 - x0).max(1.0),
                PALETTE[r.slot],
                r.label
            ));
        }
        // key ダイヤ
        for (k, kx) in r.keys.iter().enumerate() {
            b.push_str(&format!(
                r#"<div class="key{}" style="left:{}px;top:{}px"></div>"#,
                if r.selected && k == 0 { " selkey" } else { "" },
                x_at(*kx) - 4.0,
                cy - 4.0
            ));
        }
    }

    // ---- surface 境界線と playhead ----
    b.push_str(&format!(
        r#"<div class="vsep" style="left:{SIDEBAR}px"></div>
           <div class="ph" style="left:{}px"></div>
           <div class="phhead" style="left:{}px"></div>"#,
        x_at(PLAYHEAD),
        x_at(PLAYHEAD) - 4.0
    ));

    format!(
        r#"<html><head><style>
  html,body {{ margin:0; padding:0; width:{W}px; height:{H}px;
              background:#2a2a2a; font-family:sans-serif; color:#d6d6d6; }}
  div {{ position:absolute; }}
  .ov {{ top:0; height:{OVERVIEW_H}px; background:#242424; }}
  .ovlabel {{ left:8px; top:4px; font-size:9px; color:#757575; }}
  .ovborder {{ top:1px; height:{}px; border:1px solid #d8d8d8; }}
  .ovbar {{ height:3px; }}
  .ruler {{ left:0; width:{W}px; height:{RULER_H}px; background:#464646; }}
  .rulerlabel {{ left:8px; font-size:9px; color:#c0c0c0; }}
  .tick {{ width:1px; height:5px; background:#6a6a6a; }}
  .ticklabel {{ font-size:8px; color:#919191; }}
  .locator {{ left:0; width:{W}px; height:{LOCATOR_H}px; background:#2f2f2f;
              border-bottom:1px solid #111111; }}
  .row {{ left:0; width:{W}px; height:{}px; background:#363636;
          border-bottom:1px solid #111111;
          background-image:repeating-linear-gradient(to right,
            #262626 0px, #262626 1px, transparent 1px, transparent {}px); }}
  .sel {{ background:#414141; }}
  .tri {{ left:10px; width:0; height:0; border-left:5px solid #929292;
          border-top:4px solid transparent; border-bottom:4px solid transparent; }}
  .dot {{ left:11px; width:2px; height:2px; background:#929292; }}
  .llabel {{ left:20px; font-size:9px; color:#d6d6d6; }}
  .plabel {{ left:31px; font-size:9px; color:#757575; }}
  .tg {{ width:13px; height:12px; background:#2f2f2f; color:#919191;
         font-size:8px; text-align:center; border:1px solid #464646; }}
  .bar {{ height:{}px; color:#141414; font-size:9px; padding-left:6px;
          overflow:hidden; white-space:nowrap; }}
  .selbar {{ border:1px solid #ffffff; }}
  .key {{ width:8px; height:8px; background:#d6d6d6; transform:rotate(45deg); }}
  .selkey {{ background:#ffad56; }}
  .vsep {{ top:0; width:1px; height:{H}px; background:#111111; }}
  .ph {{ top:{OVERVIEW_H}px; width:1px; height:{}px; background:#e7e7e7; }}
  .phhead {{ top:{}px; width:9px; height:6px; background:#e7e7e7; }}
</style></head><body>{b}</body></html>"#,
        OVERVIEW_H - 3.0,
        ROW_H - 1.0,
        (W as f64 - SIDEBAR) / 10.0,
        ROW_H - 4.0,
        H as f64 - OVERVIEW_H,
        OVERVIEW_H - 6.0,
    )
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "P10: 現行Timeline UI の Blitz 再現",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 560.0]),
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
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu");
        let texture = rs.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ui-mock"),
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
            wgpu::FilterMode::Nearest,
        );
        let mut doc = HtmlDocument::from_html(
            &build_html(),
            DocumentConfig { style_threading: StyleThreading::Sequential, ..Default::default() },
        );
        doc.set_viewport(Viewport {
            window_size: (W, H),
            hidpi_scale: 1.0,
            zoom: 1.0,
            color_scheme: ColorScheme::Dark,
        });
        doc.resolve(0.0);
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
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();
        ui.label(
            egui::RichText::new("P10: 現行Timeline UI を HTML/CSS で再現(実寸・実配色を timeline_egui から写した)")
                .color(egui::Color32::from_rgb(0xff, 0xad, 0x56)),
        );
        ui.label("hover で行/clip/key が反応すれば、CSS疑似クラスも効いている");
        ui.separator();

        let (rect, resp) = ui.allocate_exact_size(egui::vec2(W as f32, H as f32), egui::Sense::hover());
        if let Some(p) = resp.hover_pos() {
            let mut d = EventDriver::new(&mut *self.doc, NoopEventHandler);
            d.handle_ui_event(UiEvent::PointerMove(pointer_at(p.x - rect.min.x, p.y - rect.min.y)));
        }
        self.doc.resolve(0.0);

        let rs = frame.wgpu_render_state().unwrap();
        let (device, queue) = (rs.device.clone(), rs.queue.clone());
        let mut scene = Scene::new(W as u16, H as u16);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cache = FxHashMap::default();
            let mut bind = FxHashMap::default();
            let im = ImageManager::new(&mut self.r, &mut self.res, &device, &queue, &mut enc, &mut cache);
            let mut p = VelloHybridScenePainter::new(&mut scene, im, &mut bind, &self.dh);
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
