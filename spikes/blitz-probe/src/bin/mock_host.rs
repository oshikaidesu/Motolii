//! P12: モックのHTML/CSSを、実際にマウスで触れる窓に載せる。
//!
//! P7(`texture_host`)との違いは**当たり判定の出どころ**。
//! P7は「レーン番号 = (y - ROWS_TOP)/ROW_H」のように器側が幾何を持っていた。
//! ここでは幾何を1つも持たず、掴む相手は `doc.hit()` が返した要素から決める。
//! trimハンドルの7pxも、行の高さも、CSSがすでに知っていることをそのまま使う。
//!
//! 構成(製品と同じ形):
//!   eframe(egui) が窓と wgpu29 デバイスを持つ
//!   Blitz は毎フレーム そのデバイス上のテクスチャへ HTML/CSS を描く
//!   ポインタは Motolii 側(このコード)が blitz-dom へ流す
//!   egui はテクスチャを画像として合成する
//!
//! **注意: これはUXの台本を触るための器であって、製品のTimelineではない。**
//! ここではDOMを直接書き換えて位置を動かしている。製品では意味の持ち主は
//! Document/D2 で、DOMは投影の出力でしかない。詳細は下の「同期」の注記。
//!
//! 使い方: mock_host <input.html> [W] [H]

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use atomic_refcell::AtomicRefCell;
use blitz_dom::{Document, DocumentConfig, EventDriver, NoopEventHandler, StyleThreading};
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
use std::time::Instant;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Blitz側の既知の穴をふさぐ最小のCSS。両方とも今日の実測で理由がある。
///
/// - `z-index` を持つ絶対配置は、包含ブロックが stacking context を作らないと
///   描画も hit も文書原点へ落ちる。だから包含ブロック側に `z-index:0` を置く。
/// - `transition` を持つプロパティはドラッグでポインタから遅れる。掴んでいる間の
///   位置は「今の指の位置」であるべきなので、bar/keyからは外す。
const BLITZ_FIXUP: &str = r#"<style>
.rowTrack,.overviewTrack,.keyTrack,.surfaceOverlay{z-index:0}
.objectBar,.key{transition:none!important}
</style>"#;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: mock_host <input.html> [W] [H]");
        std::process::exit(2);
    }
    let path = args[1].clone();
    let w: u32 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(1280);
    let h: u32 = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(500);

    eframe::run_native(
        "P12: モックのHTML/CSSを触る (host = eframe/egui, 面 = Blitz)",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([w as f32 + 40.0, h as f32 + 120.0]),
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(App::new(cc, &path, w, h)))),
    )
}

/// 掴んでいるもの。DOMのノードidで持つ — 器の側に行番号や列番号を持たない。
struct Grab {
    node: usize,
    mode: GrabMode,
    /// 掴んだ瞬間の、要素の親の中でのleft(px)
    start_left: f32,
    start_width: f32,
    /// 掴んだ瞬間のポインタx(面の座標)
    start_pointer: f32,
    label: String,
}

#[derive(Clone, Copy, PartialEq)]
enum GrabMode {
    Move,
    TrimStart,
    TrimEnd,
}

struct App {
    doc: HtmlDocument,
    scene_renderer: Renderer,
    resources: Resources,
    view: wgpu::TextureView,
    egui_tex: egui::TextureId,
    device_handle: DeviceHandle,
    w: u32,
    h: u32,
    grab: Option<Grab>,
    under_cursor: String,
    last_action: String,
    resolve_ms: f64,
    render_ms: f64,
    frame: u32,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, path: &str, w: u32, h: u32) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu backend");
        let device = &rs.device;

        let raw = std::fs::read_to_string(path).expect("input html");
        let html = inline_and_fixup(path, &raw);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mock-panel"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
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
        let egui_tex =
            rs.renderer
                .write()
                .register_native_texture(device, &view, wgpu::FilterMode::Nearest);

        let mut doc = HtmlDocument::from_html(
            &html,
            DocumentConfig {
                style_threading: StyleThreading::Sequential,
                ..Default::default()
            },
        );
        doc.set_viewport(Viewport {
            window_size: (w, h),
            hidpi_scale: 1.0,
            zoom: 1.0,
            color_scheme: ColorScheme::Dark,
        });
        // hit() は stacking context を見る。1回目のresolveでは z-index を持つ
        // 絶対配置がまだ揃わず、親が当たる。最初から触れるように2回回す。
        doc.resolve(0.0);
        doc.resolve(0.0);

        Self {
            doc,
            scene_renderer: Renderer::new(
                device,
                &RenderTargetConfig {
                    format: FORMAT,
                    width: w,
                    height: h,
                },
            ),
            resources: Resources::new(),
            view,
            egui_tex,
            device_handle: DeviceHandle {
                instance: wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
                adapter: rs.adapter.clone(),
                device: rs.device.clone(),
                queue: rs.queue.clone(),
            },
            w,
            h,
            grab: None,
            under_cursor: "-".into(),
            last_action: "clip を掴んで動かす / 端7pxでトリム / key もつまめる".into(),
            resolve_ms: 0.0,
            render_ms: 0.0,
            frame: 0,
        }
    }

    fn paint_blitz(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let t = Instant::now();
        let mut scene = Scene::new(self.w as u16, self.h as u16);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mock-panel-enc"),
        });
        {
            let mut cache = FxHashMap::default();
            let mut bindings = FxHashMap::default();
            let im = ImageManager::new(
                &mut self.scene_renderer,
                &mut self.resources,
                device,
                queue,
                &mut enc,
                &mut cache,
            );
            let mut painter =
                VelloHybridScenePainter::new(&mut scene, im, &mut bindings, &self.device_handle);
            paint_scene(&mut painter, &mut self.doc, 1.0, self.w, self.h, 0, 0);
        }
        let _ = self.scene_renderer.render(
            &scene,
            &mut self.resources,
            device,
            queue,
            &mut enc,
            &RenderSize {
                width: self.w,
                height: self.h,
            },
            &self.view,
            &TextureBindings::default(),
        );
        queue.submit([enc.finish()]);
        self.render_ms = t.elapsed().as_secs_f64() * 1000.0;
    }

    /// 掴む相手を決める。器は幾何を持たず、当たった要素の素性だけを見る。
    fn begin_grab(&mut self, lx: f32, ly: f32) {
        let Some(hit) = self.doc.hit(lx, ly) else {
            return;
        };
        let mut node = hit.node_id;
        let mut mode = GrabMode::Move;

        // trimハンドルに当たったら、幅を変える意図として読む。
        // 「端から7px」を器が判定しない — CSSがすでにその矩形を持っている。
        if let Some(cls) = class_of(&self.doc, node) {
            if cls.contains("trimStart") {
                mode = GrabMode::TrimStart;
            } else if cls.contains("trimEnd") {
                mode = GrabMode::TrimEnd;
            }
        }

        // 掴める先祖(.objectBar か .key)まで登る
        let target = loop {
            let cls = class_of(&self.doc, node).unwrap_or_default();
            if cls.split_whitespace().any(|c| c == "objectBar" || c == "key") {
                break Some(node);
            }
            match self.doc.get_node(node).and_then(|n| n.parent) {
                Some(p) => node = p,
                None => break None,
            }
        };
        let Some(target) = target else {
            self.last_action = "掴めるものではない".into();
            return;
        };

        let n = self.doc.get_node(target).unwrap();
        let label = attr(&self.doc, target, "data-owner")
            .or_else(|| attr(&self.doc, target, "aria-label"))
            .unwrap_or_else(|| class_of(&self.doc, target).unwrap_or_default());

        self.grab = Some(Grab {
            node: target,
            mode,
            start_left: n.final_layout.location.x,
            start_width: n.final_layout.size.width,
            start_pointer: lx,
            label: label.clone(),
        });
        self.last_action = match mode {
            GrabMode::Move => format!("掴んだ: {label}"),
            GrabMode::TrimStart => format!("左トリム: {label}"),
            GrabMode::TrimEnd => format!("右トリム: {label}"),
        };
    }

    fn drag_to(&mut self, lx: f32) {
        let Some(grab) = &self.grab else { return };
        let dx = lx - grab.start_pointer;
        let (left, width) = match grab.mode {
            GrabMode::Move => ((grab.start_left + dx).max(0.0), grab.start_width),
            GrabMode::TrimStart => {
                let l = (grab.start_left + dx).max(0.0);
                let w = (grab.start_left + grab.start_width - l).max(8.0);
                (l, w)
            }
            GrabMode::TrimEnd => (grab.start_left, (grab.start_width + dx).max(8.0)),
        };
        let node = grab.node;
        let style = format!("left:{left:.1}px;width:{width:.1}px");
        {
            let mut m = self.doc.mutate();
            m.set_attribute(node, blitz_dom::qual_name!("style"), &style);
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();

        ui.label(
            egui::RichText::new("P12: HTML/CSSのモックを、DOMの当たり判定だけで触る")
                .color(egui::Color32::from_rgb(0xff, 0xad, 0x56)),
        );
        ui.label(format!(
            "resolve {:.2} ms / render {:.2} ms   —   カーソルの下: {}",
            self.resolve_ms, self.render_ms, self.under_cursor
        ));
        ui.label(&self.last_action);
        ui.separator();

        let size = egui::vec2(self.w as f32, self.h as f32);
        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

        let local = resp
            .hover_pos()
            .or_else(|| ctx.pointer_latest_pos())
            .map(|p| (p.x - rect.min.x, p.y - rect.min.y));

        if let Some((lx, ly)) = local {
            let inside = lx >= 0.0 && ly >= 0.0 && lx < self.w as f32 && ly < self.h as f32;

            if inside {
                // hover は Blitz へ渡すだけ。:hover のCSSはBlitzが自分で当てる。
                {
                    let mut d = EventDriver::new(&mut self.doc as &mut dyn Document, NoopEventHandler);
                    d.handle_ui_event(UiEvent::PointerMove(pointer_at(lx, ly)));
                }
                self.under_cursor = match self.doc.hit(lx, ly) {
                    Some(hit) => {
                        let cls = class_of(&self.doc, hit.node_id).unwrap_or_else(|| "text".into());
                        match attr(&self.doc, hit.node_id, "data-owner") {
                            Some(o) => format!("{cls}  (owner={o})"),
                            None => cls,
                        }
                    }
                    None => "-".into(),
                };
            }

            if resp.drag_started() && inside {
                self.begin_grab(lx, ly);
            }
            if resp.dragged() {
                self.drag_to(lx);
            }
            if resp.drag_stopped() {
                if let Some(g) = &self.grab {
                    // 製品ではここが「意図をDocumentへ出す」場所。
                    self.last_action = format!("離した: {} — 製品ならここでintentを1本出す", g.label);
                }
                self.grab = None;
            }
        }

        self.frame = self.frame.wrapping_add(1);
        let t = Instant::now();
        self.doc.resolve(self.frame as f64 * 16.0);
        self.resolve_ms = t.elapsed().as_secs_f64() * 1000.0;

        let rs = _frame.wgpu_render_state().expect("wgpu").clone();
        self.paint_blitz(&rs.device, &rs.queue);

        ui.painter().image(
            self.egui_tex,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}

/// `<link rel=stylesheet>` を実体に置き換え、Blitzの穴をふさぐCSSを足す。
/// Blitzに `file://` の取得経路を持たせないための前処理で、意味は変えない。
fn inline_and_fixup(path: &str, html: &str) -> String {
    let base = std::path::Path::new(path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let mut out = String::with_capacity(html.len() + 4096);
    let mut rest = html;
    while let Some(start) = rest.find("<link") {
        let Some(end) = rest[start..].find('>') else {
            break;
        };
        let tag = &rest[start..start + end + 1];
        out.push_str(&rest[..start]);
        if tag.contains("stylesheet") {
            if let Some(href) = attr_value(tag, "href") {
                let p = base.join(&href);
                match std::fs::read_to_string(&p) {
                    Ok(css) => {
                        out.push_str("<style>");
                        out.push_str(&css);
                        out.push_str("</style>");
                    }
                    Err(_) => out.push_str(&format!("<!-- missing {href} -->")),
                }
            }
        } else {
            out.push_str(tag);
        }
        rest = &rest[start + end + 1..];
    }
    out.push_str(rest);

    match out.find("</head>") {
        Some(i) => {
            let mut s = out.clone();
            s.insert_str(i, BLITZ_FIXUP);
            s
        }
        None => format!("{BLITZ_FIXUP}{out}"),
    }
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = tag.find(&key)? + key.len();
    let j = tag[i..].find('"')? + i;
    Some(tag[i..j].to_string())
}

fn class_of(doc: &HtmlDocument, id: usize) -> Option<String> {
    doc.get_node(id)?
        .element_data()?
        .attr(blitz_dom::local_name!("class"))
        .map(|c| c.to_string())
}

fn attr(doc: &HtmlDocument, id: usize, name: &str) -> Option<String> {
    let el = doc.get_node(id)?.element_data()?;
    el.attrs
        .iter()
        .find(|a| &*a.name.local == name)
        .map(|a| a.value.clone())
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
