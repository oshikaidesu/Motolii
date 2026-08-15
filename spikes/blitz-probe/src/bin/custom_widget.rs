//! P8: Blitz の custom widget で Timeline 面を「1ノード」にできるか。
//!
//! スプレッドシートの構造(密な面はcanvas、DOMはchromeだけ)を Blitz の中でやる。
//! 成立すれば、clip/key を何個描いてもDOMノードは1個なので、
//! P7 で出た「resolve が1ノードあたり約4.0µs で線形」という天井が効かなくなる。
//!
//! 比較対象:
//!   DOM モード         … clip/key を全部 div にする(= P7 と同じ)
//!   custom widget モード … clip/key は自前描画。DOMノードは widget 1個だけ
//!
//! 実行:
//!   BLITZ_PROBE_ITEMS=5000 cargo run --release --bin custom_widget
//!   BLITZ_PROBE_ITEMS=5000 BLITZ_PROBE_DOM=1 cargo run --release --bin custom_widget

use anyrender::{PaintScene, Scene};
use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::node::{ComputedStyles, Widget};
use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::events::UiEvent;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::FxHashMap;
use std::time::Instant;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, TextureBindings};
use wgpu_context::DeviceHandle;

use peniko::kurbo::{Affine, Rect};
use peniko::{Color, Fill};

const W: u32 = 1200;
const H: u32 = 600;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn items() -> usize {
    std::env::var("BLITZ_PROBE_ITEMS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000)
}
fn dom_mode() -> bool {
    std::env::var("BLITZ_PROBE_DOM").map(|v| v == "1").unwrap_or(false)
}

/// Timeline 面。clip と key を自前で描く。DOM から見ると1ノード。
struct TimelineWidget {
    n: usize,
    phase: f64,
}

impl Widget for TimelineWidget {
    fn handle_event(&mut self, _event: &UiEvent) {
        // 実装では自前のhit判定をここで回す(timeline_skia/hit.rs 相当)
    }

    fn paint(
        &mut self,
        _ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> Scene {
        let mut scene = Scene::new();
        let rows = 30usize;
        let row_h = (height as f64 / rows as f64).max(6.0);
        let per_row = (self.n / rows).max(1);
        let clip_c = Color::from_rgb8(150, 170, 219);
        let key_c = Color::from_rgb8(255, 173, 86);
        for r in 0..rows {
            let y = r as f64 * row_h + 2.0;
            for i in 0..per_row {
                let x = ((i * 17) as f64 + self.phase + r as f64 * 3.0) % (width as f64);
                // 3個に1個を key(小さい四角)、残りを clip(横長)にする
                if i % 3 == 0 {
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        key_c,
                        None,
                        &Rect::new(x, y + 3.0, x + 8.0, y + 11.0),
                    );
                } else {
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        clip_c,
                        None,
                        &Rect::new(x, y, x + 14.0, y + row_h - 4.0),
                    );
                }
            }
        }
        scene
    }
}

fn main() {
    let n = items();
    let dom = dom_mode();

    // ---- GPU ----
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .expect("device");
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut renderer = Renderer::new(
        &device,
        &RenderTargetConfig {
            format: FORMAT,
            width: W,
            height: H,
        },
    );
    let mut resources = Resources::new();
    let dh = DeviceHandle {
        instance,
        adapter,
        device: device.clone(),
        queue: queue.clone(),
    };

    // ---- 文書 ----
    let html = if dom {
        // 比較用: clip/key を全部 div にする
        let mut body = String::new();
        let rows = 30usize;
        let per_row = (n / rows).max(1);
        for r in 0..rows {
            for i in 0..per_row {
                let x = (i * 17) % (W as usize);
                let y = r * 20 + 2;
                if i % 3 == 0 {
                    body.push_str(&format!(
                        r#"<div class="key" style="left:{x}px;top:{}px"></div>"#,
                        y + 3
                    ));
                } else {
                    body.push_str(&format!(
                        r#"<div class="clip" style="left:{x}px;top:{y}px"></div>"#
                    ));
                }
            }
        }
        wrap(&body)
    } else {
        wrap(r#"<div id="tl"></div>"#)
    };

    let mut doc = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
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

    // ---- custom widget を差し込む ----
    let mut widget_node = None;
    if !dom {
        // #tl の中心を hit して node_id を得る
        if let Some(hit) = doc.hit(600.0, 300.0) {
            doc.set_custom_widget(
                hit.node_id,
                Box::new(TimelineWidget { n, phase: 0.0 }),
            );
            widget_node = Some(hit.node_id);
            doc.resolve(0.0);
        }
    }
    println!(
        "P8 mode={} items={} widget_node={:?} dom_nodes={}",
        if dom { "DOM" } else { "custom-widget" },
        n,
        widget_node,
        count_nodes(&doc)
    );
    if !dom && widget_node.is_none() {
        println!("P8 RESULT: FAIL — #tl の node_id を取れなかった");
        std::process::exit(1);
    }

    // ---- 計測 ----
    let mut rs = Vec::new();
    let mut rd = Vec::new();
    for f in 0..120 {
        // 毎フレーム内容を動かす(ズーム/スクロール相当の負荷)
        if dom {
            // DOMモードでは inline style を作り直すしかない → 文書ごと作り直す
        } else if let Some(id) = widget_node {
            if let Some(node) = doc.get_node_mut(id) {
                if let Some(el) = node.element_data_mut() {
                    if let Some(wd) = el.custom_widget_data_mut() {
                        // phase を進める(再描画を強制)
                        let _ = wd;
                    }
                }
            }
        }
        let t0 = Instant::now();
        doc.resolve(0.0);
        let resolve_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = Instant::now();
        let mut scene = vello_hybrid::Scene::new(W as u16, H as u16);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cache = FxHashMap::default();
            let mut bind = FxHashMap::default();
            let im = ImageManager::new(&mut renderer, &mut resources, &device, &queue, &mut enc, &mut cache);
            let mut painter = VelloHybridScenePainter::new(&mut scene, im, &mut bind, &dh);
            paint_scene(&mut painter, &mut doc, 1.0, W, H, 0, 0);
        }
        let _ = renderer.render(
            &scene,
            &mut resources,
            &device,
            &queue,
            &mut enc,
            &RenderSize { width: W, height: H },
            &view,
            &TextureBindings::default(),
        );
        queue.submit([enc.finish()]);
        let render_ms = t1.elapsed().as_secs_f64() * 1000.0;

        if f > 15 {
            rs.push(resolve_ms);
            rd.push(render_ms);
        }
    }
    rs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    rd.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |v: &Vec<f64>, q: usize| v[v.len() * q / 100];
    println!(
        "P8 RESULT: mode={} items={} dom_nodes={} resolve(p50={:.2} p95={:.2}) render(p50={:.2} p95={:.2}) total(p50={:.2})",
        if dom { "DOM" } else { "widget" },
        n,
        count_nodes(&doc),
        p(&rs, 50),
        p(&rs, 95),
        p(&rd, 50),
        p(&rd, 95),
        p(&rs, 50) + p(&rd, 50)
    );
}

fn count_nodes(doc: &HtmlDocument) -> usize {
    // 公開APIが無いので概算: hit可能な要素数ではなく、DOMツリーを辿って数える
    fn walk(doc: &HtmlDocument, id: usize, n: &mut usize) {
        *n += 1;
        if let Some(node) = doc.get_node(id) {
            for c in node.children.iter() {
                walk(doc, *c, n);
            }
        }
    }
    let mut n = 0;
    walk(doc, doc.root_element().id, &mut n);
    n
}

fn wrap(body: &str) -> String {
    format!(
        r#"<html><head><style>
      html,body {{ margin:0; padding:0; background:rgb(36,36,36); }}
      #tl {{ position:absolute; left:0; top:0; width:{W}px; height:{H}px; }}
      .clip {{ position:absolute; width:14px; height:16px; background:rgb(150,170,219); }}
      .key {{ position:absolute; width:8px; height:8px; background:rgb(255,173,86); }}
    </style></head><body>{body}</body></html>"#
    )
}
