//! P7: 提案構成そのものを目で見て触る。
//!
//!   eframe(egui) が窓と wgpu29 デバイスを持つ   ← Motolii本体と同じ構成
//!   Blitz は毎フレーム、そのデバイス上のテクスチャへ HTML/CSS を描くだけ
//!   ポインタイベントは Motolii 側(このコード)が blitz-dom へルーティングする
//!   egui は そのテクスチャを画像として合成する
//!
//! 違和感の判定材料:
//!   - **白い丸(egui native描画)と、青→橙に変わるclip(Blitz描画)のズレ**
//!     丸に対して色替えが遅れて見えるなら、それがテクスチャ経路の遅延。
//!   - 文字と矩形の輪郭がぼやけていないか(テクスチャ経由の劣化)
//!   - clipを掴んで動かしたときの追従
//!
//! 注意: drag はHTMLを毎フレーム組み直して実現している(BlitzにJSが無いため)。
//! 実装ではDioxus等で差分更新するので、`rebuild` の時間は本番の値ではない。
//! 経路そのものの遅延は `render` の値と、hover の追従で見てほしい。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use atomic_refcell::AtomicRefCell;
use anyrender::{PaintScene, Scene as ARScene};
use blitz_dom::node::{ComputedStyles, Widget};
use blitz_dom::{DocumentConfig, EventDriver, NoopEventHandler, StyleThreading};
use peniko::kurbo::{Affine, Rect};
use peniko::{Color as PColor, Fill};
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

const W: u32 = 900;
const H: u32 = 420;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
fn tracks() -> usize {
    if let Some(t) = std::env::var("BLITZ_PROBE_TRACKS").ok().and_then(|v| v.parse::<usize>().ok()) {
        return t.max(1);
    }
    8 * std::env::var("BLITZ_PROBE_SCALE").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(1).max(1)
}
fn clips() -> usize {
    std::env::var("BLITZ_PROBE_CLIPS").ok().and_then(|v| v.parse().ok()).unwrap_or(3)
}
fn keys_per_track() -> usize {
    std::env::var("BLITZ_PROBE_KEYS").ok().and_then(|v| v.parse().ok()).unwrap_or(14)
}
/// 自動掃引: ズームを動かし続けて resolve/render の分位点を出す
fn auto_frames() -> usize {
    std::env::var("BLITZ_PROBE_AUTO").ok().and_then(|v| v.parse().ok()).unwrap_or(0)
}
const ROWS_TOP: f64 = 28.0;
const ROW_H: f64 = 30.0;
const LANE_LEFT: f64 = 90.0;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "P7: Blitz as texture renderer (host = eframe/egui)",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 560.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    doc: HtmlDocument,
    scene_renderer: Renderer,
    resources: Resources,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    egui_tex: egui::TextureId,
    device_handle: DeviceHandle,
    clips: Vec<Vec<(f64, f64)>>,
    drag: Option<(usize, usize, f64)>,
    dirty: bool,
    rebuild_ms: f64,
    render_ms: f64,
    resolve_ms: f64,
    zoom: f32,
    samples: Vec<(f64, f64, f64)>,
    frame_i: usize,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu backend");
        let device = &rs.device;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("blitz-panel"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            // egui へ渡すので TEXTURE_BINDING も要る
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let egui_tex = rs.renderer.write().register_native_texture(
            device,
            &view,
            wgpu::FilterMode::Nearest,
        );

        let clips: Vec<Vec<(f64, f64)>> = (0..tracks())
            .map(|tr| {
                (0..clips())
                    .map(|c| ((c * 190 + tr * 14) as f64, 150.0))
                    .collect::<Vec<(f64, f64)>>()
            })
            .collect();

        let mut app = Self {
            doc: build_doc(&clips, 1.0),
            scene_renderer: Renderer::new(
                device,
                &RenderTargetConfig {
                    format: FORMAT,
                    width: W,
                    height: H,
                },
            ),
            resources: Resources::new(),
            texture,
            view,
            egui_tex,
            device_handle: DeviceHandle {
                instance: rs.adapter.get_info().backend.to_string().parse().ok().map_or_else(
                    || wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
                    |_: u8| wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
                ),
                adapter: rs.adapter.clone(),
                device: rs.device.clone(),
                queue: rs.queue.clone(),
            },
            clips,
            drag: None,
            dirty: true,
            rebuild_ms: 0.0,
            render_ms: 0.0,
            resolve_ms: 0.0,
            zoom: 1.0,
            samples: Vec::new(),
            frame_i: 0,
        };
        app.doc.set_viewport(Viewport {
            window_size: (W, H),
            hidpi_scale: 1.0,
            zoom: 1.0,
            color_scheme: ColorScheme::Dark,
        });
        app.doc.resolve(0.0);
        app
    }

    /// Blitz を自前テクスチャへ描く。ここが「テクスチャレンダラとして使う」の実体。
    fn paint_blitz(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let t = Instant::now();
        let mut scene = Scene::new(W as u16, H as u16);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("blitz-panel-enc"),
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
            paint_scene(&mut painter, &mut self.doc, 1.0, W, H, 0, 0);
        }
        let _ = self.scene_renderer.render(
            &scene,
            &mut self.resources,
            device,
            queue,
            &mut enc,
            &RenderSize {
                width: W,
                height: H,
            },
            &self.view,
            &TextureBindings::default(),
        );
        queue.submit([enc.finish()]);
        self.render_ms = t.elapsed().as_secs_f64() * 1000.0;
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint(); // 連続描画にして遅延を見えるようにする
        {
            let ctx = &ctx;
            ui.label(
                egui::RichText::new("P7: Blitz をテクスチャとして egui が合成している")
                    .color(egui::Color32::from_rgb(0xff, 0xad, 0x56)),
            );
            ui.label(format!(
                "resolve(全ノード再スタイル+再レイアウト): {:.2} ms / render(Blitz→テクスチャ): {:.2} ms / rebuild(HTML再パース: probe都合): {:.2} ms",
                self.resolve_ms, self.render_ms, self.rebuild_ms
            ));
            let mut z = self.zoom;
            ui.horizontal(|ui| {
                ui.label("横方向ズーム(文字・行高・keyサイズは据え置き。left/widthを再計算):");
                ui.add(egui::Slider::new(&mut z, 0.3..=4.0).fixed_decimals(2));
            });
            if (z - self.zoom).abs() > 1e-6 {
                self.zoom = z;
                self.dirty = true;
            }
            ui.label("白い丸=egui native / clipの色替え=Blitz。ズレて見えたらそれが経路の遅延");
            ui.separator();

            let size = egui::vec2(W as f32, H as f32);
            let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

            // ---- 入力ルーティングは Motolii 側(ここ) ----
            let local = resp
                .hover_pos()
                .or_else(|| ctx.pointer_latest_pos())
                .map(|p| (p.x - rect.min.x, p.y - rect.min.y));

            if let Some((lx, ly)) = local {
                // hover を Blitz へ注入
                {
                    let mut d = EventDriver::new(&mut *self.doc, NoopEventHandler);
                    d.handle_ui_event(UiEvent::PointerMove(pointer_at(lx, ly)));
                }

                // drag は自前の状態で持つ(BlitzにJSが無いため)
                if resp.drag_started() {
                    let tr = ((ly as f64 - ROWS_TOP) / ROW_H).floor();
                    if tr >= 0.0 && (tr as usize) < tracks() {
                        let tr = tr as usize;
                        let wx = (lx as f64 - LANE_LEFT) / self.zoom as f64;
                        for (c, (s, w)) in self.clips[tr].iter().enumerate() {
                            if wx >= *s && wx <= s + w {
                                self.drag = Some((tr, c, wx - s));
                                break;
                            }
                        }
                    }
                }
                if resp.dragged() {
                    if let Some((tr, c, dx)) = self.drag {
                        // 横ズーム分を戻してから world 座標にする(ここが抜けていてズレていた)
                        let ns = ((lx as f64 - LANE_LEFT) / self.zoom as f64 - dx).max(0.0);
                        if (self.clips[tr][c].0 - ns).abs() > 0.01 {
                            self.clips[tr][c].0 = ns;
                            self.dirty = true;
                        }
                    }
                }
                if resp.drag_stopped() {
                    self.drag = None;
                }
            }

            if self.dirty {
                let t = Instant::now();
                self.doc = build_doc(&self.clips, self.zoom as f64);
                self.doc.set_viewport(Viewport {
                    window_size: (W, H),
                    hidpi_scale: 1.0,
                    zoom: 1.0,
                    color_scheme: ColorScheme::Dark,
                });
                if widget_mode() {
                    // hit() は resolve 済みを要求するので使わない。
                    // id 属性で辿れば resolve 前に付けられ、resolve を外側で正しく測れる。
                    if let Some(id) = find_by_id(&self.doc, "tl") {
                        self.doc.set_custom_widget(
                            id,
                            Box::new(TimelineWidget {
                                clips: self.clips.clone(),
                                keys: keys_per_track(),
                                xz: self.zoom as f64,
                            }),
                        );
                    }
                }
                self.rebuild_ms = t.elapsed().as_secs_f64() * 1000.0;
                self.dirty = false;
            }
            let t_resolve = Instant::now();
            self.doc.resolve(0.0);
            self.resolve_ms = t_resolve.elapsed().as_secs_f64() * 1000.0;

            // 自動掃引: ズームを動かし続けて上限を測る
            let auto = auto_frames();
            if auto > 0 {
                self.frame_i += 1;
                let z = 0.4 + ((self.frame_i % 200) as f32) * 0.018;
                if (z - self.zoom).abs() > 1e-6 { self.zoom = z; self.dirty = true; }
                if self.frame_i > 20 {
                    self.samples.push((self.resolve_ms, self.render_ms, self.rebuild_ms));
                }
                if self.frame_i >= auto {
                    report(&self.samples,
                        tracks() * (clips() * 2 + keys_per_track()) + 40);
                    std::process::exit(0);
                }
            }

            let rs = frame.wgpu_render_state().unwrap();
            let (device, queue) = (rs.device.clone(), rs.queue.clone());
            self.paint_blitz(&device, &queue);

            // ---- egui が Blitz のテクスチャを合成 ----
            ui.painter().image(
                self.egui_tex,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // ---- 遅延比較用の egui native マーカー ----
            if let Some(p) = ctx.pointer_latest_pos() {
                if rect.contains(p) {
                    ui.painter()
                        .circle_filled(p, 5.0, egui::Color32::from_white_alpha(220));
                }
            }
        }
    }
}

/// 状態から HTML を組む。本番は差分更新にするので、この全組み直しは probe 都合。
/// `xz` は横方向ズーム。**x位置と幅にだけ**掛ける。
/// 文字サイズ・行高・keyのグリフサイズには掛けない — Timelineの通例であり、
/// これが `transform: scaleX()` で代替できない理由でもある(文字が潰れる)。
fn build_doc(clips: &[Vec<(f64, f64)>], xz: f64) -> HtmlDocument {
    let mut body = String::new();
    for (tr, row) in clips.iter().enumerate() {
        let top = ROWS_TOP + tr as f64 * ROW_H;
        body.push_str(&format!(
            r#"<div class="tname" style="top:{top}px">layer {tr}</div>"#
        ));
        for (s, w) in row {
            body.push_str(&format!(
                r#"<div class="clip" style="left:{}px; top:{}px; width:{}px">素材 {}.mp4 — 夜明けの街</div>"#,
                LANE_LEFT + s * xz,
                top + 4.0,
                w * xz,
                tr
            ));
        }
        if widget_mode() { continue; }   // key だけ widget が描く
        for k in 0..keys_per_track() {
            body.push_str(&format!(
                r#"<div class="key" style="left:{}px; top:{}px"></div>"#,
                LANE_LEFT + ((k * 58) as f64 + tr as f64 * 6.0) * xz,
                top + 9.0
            ));
        }
    }
    // widget は最初に置く。後続の DOM(ruler/レイヤ名)がその上へ描かれる。
    let body = if widget_mode() {
        format!(
            r#"<div id="tl" style="position:absolute;left:{LANE_LEFT}px;top:{ROWS_TOP}px;width:{}px;height:{}px;z-index:0"></div>{body}"#,
            W as f64 - LANE_LEFT, H as f64 - ROWS_TOP
        )
    } else {
        body
    };
    let html = format!(
        r#"<html><head><style>
      html, body {{ margin:0; padding:0; background:rgb(36,36,36);
                   font-family:sans-serif; font-size:11px; color:rgb(214,214,214); }}
      .ruler {{ position:absolute; left:0; top:0; width:{W}px; height:26px;
                background:rgb(47,47,47); z-index:5; }}
      .tname {{ position:absolute; left:0; width:84px; height:29px; padding:8px 4px;
                background:rgb(47,47,47); color:rgb(145,145,145); z-index:5; }}
      .clip {{ position:absolute; height:21px; background:rgb(150,170,219);
               color:rgb(20,20,20); font-size:10px; padding:4px 5px; overflow:hidden;
               white-space:nowrap;
               border-left:2px solid rgb(111,140,196); border-right:2px solid rgb(111,140,196); }}
      .clip:hover {{ background:rgb(255,173,86); }}
      .key {{ position:absolute; width:11px; height:11px; background:rgb(255,173,86); }}
      .key:hover {{ background:rgb(231,231,231); }}
    </style></head><body>{body}<div class="ruler">Blitz が描画 — 日本語テキストと欧文 AVfi 123 の拡大品質を見る / hover で色が変わる</div></body></html>"#
    );
    {
        // 既定は Sequential(doc commentの "Defaults to Parallel" は誤り)。
        // BLITZ_PROBE_PAR=1 で Stylo の並列トラバーサルへ切り替えて比較する。
        let parallel = std::env::var("BLITZ_PROBE_PAR").map(|v| v == "1").unwrap_or(false);
        HtmlDocument::from_html(
            &html,
            DocumentConfig {
                style_threading: if parallel {
                    StyleThreading::Parallel
                } else {
                    StyleThreading::Sequential
                },
                ..Default::default()
            },
        )
    }
}

/// Timeline面を1ノードで描く custom widget。
/// **文字は描かない** — 文字はDOM側に置く。拡大しても伸びないことを見せるため。
struct TimelineWidget {
    clips: Vec<Vec<(f64, f64)>>,
    keys: usize,
    xz: f64,
}

impl Widget for TimelineWidget {
    fn handle_event(&mut self, _e: &UiEvent) {}
    fn paint(
        &mut self,
        _ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        _w: u32,
        _h: u32,
        _scale: f64,
    ) -> ARScene {
        let mut sc = ARScene::new();
        let clip_c = PColor::from_rgb8(150, 170, 219);
        let edge_c = PColor::from_rgb8(111, 140, 196);
        let key_c = PColor::from_rgb8(255, 173, 86);
        // 座標は widget 内ローカル(左上が lane の原点)
        let _ = (clip_c, edge_c);
        for (tr, _row) in self.clips.iter().enumerate() {
            let top = tr as f64 * ROW_H;
            for k in 0..self.keys {
                let x = ((k * 58) as f64 + tr as f64 * 6.0) * self.xz;
                sc.fill(Fill::NonZero, Affine::IDENTITY, key_c, None,
                        &Rect::new(x, top + 9.0, x + 11.0, top + 20.0));
            }
        }
        sc
    }
}

fn widget_mode() -> bool {
    std::env::var("BLITZ_PROBE_WIDGET").map(|v| v == "1").unwrap_or(false)
}

fn report(samples: &[(f64, f64, f64)], nodes: usize) {
    if samples.is_empty() { println!("P7 RESULT: no samples"); return; }
    let mut r: Vec<f64> = samples.iter().map(|s| s.0).collect();
    let mut d: Vec<f64> = samples.iter().map(|s| s.1).collect();
    let mut b: Vec<f64> = samples.iter().map(|s| s.2).collect();
    let mut t: Vec<f64> = samples.iter().map(|s| s.0 + s.1 + s.2).collect();
    for v in [&mut r, &mut d, &mut b, &mut t] { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); }
    let p = |v: &Vec<f64>, q: usize| v[v.len() * q / 100];
    println!(
        "P7 RESULT: par={} nodes~{} n={} resolve(p50={:.2}) render(p50={:.2}) rebuild(p50={:.2}) TOTAL(p50={:.2} p95={:.2})",
        std::env::var("BLITZ_PROBE_PAR").unwrap_or_else(|_| "0".into()), nodes, t.len(), p(&r,50), p(&d,50), p(&b,50), p(&t,50), p(&t,95)
    );
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

/// id 属性でノードを探す(resolve 不要)。
/// `hit()` は resolve 済みを要求するため、計測を歪めずに widget を付けるにはこちらを使う。
fn find_by_id(doc: &HtmlDocument, want: &str) -> Option<usize> {
    fn walk(doc: &HtmlDocument, id: usize, want: &str) -> Option<usize> {
        let node = doc.get_node(id)?;
        if let Some(el) = node.element_data() {
            if el.attr(blitz_dom::local_name!("id")) == Some(want) {
                return Some(id);
            }
        }
        for c in node.children.iter() {
            if let Some(f) = walk(doc, *c, want) {
                return Some(f);
            }
        }
        None
    }
    walk(doc, doc.root_element().id, want)
}
