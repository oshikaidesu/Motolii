//! P9: 差分更新は再パースを置き換えられるか。
//!
//! これまでの測定で最大の数字だった `rebuild`(HTML全再パース、25〜33ms)を
//! 「実装では差分更新になるので消える」と扱ってきたが、それは**仮定**だった。
//! 本プローブはその仮定を検証する。
//!
//! 比較:
//!   REPARSE … 毎フレーム HTML を組み直して `from_html` で作り直す(= これまでの probe)
//!   MUTATE  … 文書は1回だけ作り、毎フレーム `set_style_property` で left/width だけ書き換える
//!
//! 実行:
//!   BLITZ_PROBE_CLIPS=900 cargo run --release --bin diff_update

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::FxHashMap;
use std::time::Instant;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const W: u32 = 1200;
const H: u32 = 600;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const ROWS: usize = 30;

fn n_clips() -> usize {
    std::env::var("BLITZ_PROBE_CLIPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(900)
}
const FRAMES: usize = 120;

fn main() {
    let n = n_clips();

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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
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

    let mut render_once = |doc: &mut HtmlDocument,
                           renderer: &mut Renderer,
                           resources: &mut Resources|
     -> f64 {
        let t = Instant::now();
        let mut scene = Scene::new(W as u16, H as u16);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cache = FxHashMap::default();
            let mut bind = FxHashMap::default();
            let im = ImageManager::new(renderer, resources, &device, &queue, &mut enc, &mut cache);
            let mut p = VelloHybridScenePainter::new(&mut scene, im, &mut bind, &dh);
            paint_scene(&mut p, doc, 1.0, W, H, 0, 0);
        }
        let _ = renderer.render(
            &scene,
            resources,
            &device,
            &queue,
            &mut enc,
            &RenderSize {
                width: W,
                height: H,
            },
            &view,
            &TextureBindings::default(),
        );
        queue.submit([enc.finish()]);
        t.elapsed().as_secs_f64() * 1000.0
    };

    // ================= REPARSE =================
    {
        let mut upd = Vec::new();
        let mut res = Vec::new();
        let mut rnd = Vec::new();
        let mut doc = make_doc(n, 1.0);
        for f in 0..FRAMES {
            let xz = 1.0 + (f % 60) as f64 * 0.02;
            let t = Instant::now();
            doc = make_doc(n, xz);
            let update = t.elapsed().as_secs_f64() * 1000.0;
            let t = Instant::now();
            doc.resolve(0.0);
            let resolve = t.elapsed().as_secs_f64() * 1000.0;
            let render = render_once(&mut doc, &mut renderer, &mut resources);
            if f > 15 {
                upd.push(update);
                res.push(resolve);
                rnd.push(render);
            }
        }
        report("REPARSE", n, &mut upd, &mut res, &mut rnd);
    }

    // ================= MUTATE =================
    {
        let mut doc = make_doc(n, 1.0);
        doc.resolve(0.0);
        let ids = collect_clip_ids(&doc);
        let mut upd = Vec::new();
        let mut res = Vec::new();
        let mut rnd = Vec::new();
        for f in 0..FRAMES {
            let xz = 1.0 + (f % 60) as f64 * 0.02;
            let t = Instant::now();
            {
                let mut m = doc.mutate();
                for (i, id) in ids.iter().enumerate() {
                    let base = ((i % (n / ROWS).max(1)) * 190) as f64;
                    let top = 28.0 + (i / (n / ROWS).max(1)) as f64 * 19.0;
                    m.set_attribute(
                        *id,
                        blitz_dom::qual_name!("style"),
                        &format!("left:{}px; top:{top}px; width:{}px", base * xz, 150.0 * xz),
                    );
                }
            }
            let update = t.elapsed().as_secs_f64() * 1000.0;
            let t = Instant::now();
            doc.resolve(0.0);
            let resolve = t.elapsed().as_secs_f64() * 1000.0;
            let render = render_once(&mut doc, &mut renderer, &mut resources);
            if f > 15 {
                upd.push(update);
                res.push(resolve);
                rnd.push(render);
            }
        }
        println!("(MUTATE 対象 clip ノード数 = {})", ids.len());
        report("MUTATE", n, &mut upd, &mut res, &mut rnd);

        // --- 健全性検査: mutate が本当にレイアウトへ効いているか ---
        // 1本目のclipを xz=1.0 と xz=3.0 で描き、右端の位置が変わることを見る。
        // xz=1.0 では clip0(0-150) と clip1(190-340) の隙間 = 背景色。
        // xz=3.0 では clip0 が 0-450 まで伸びるので clip 色になるはず。
        let probe_x = 165u32;
        let probe_y = 30u32;
        for (label, xz) in [("xz=1.0", 1.0f64), ("xz=3.0", 3.0f64)] {
            {
                let mut m = doc.mutate();
                for (i, id) in ids.iter().enumerate() {
                    let base = ((i % (n / ROWS).max(1)) * 190) as f64;
                    let top = 28.0 + (i / (n / ROWS).max(1)) as f64 * 19.0;
                    m.set_attribute(
                        *id,
                        blitz_dom::qual_name!("style"),
                        &format!("left:{}px; top:{top}px; width:{}px", base * xz, 150.0 * xz),
                    );
                }
            }
            doc.resolve(0.0);
            let _ = render_once(&mut doc, &mut renderer, &mut resources);
            let px = readback(&device, &queue, &texture, probe_x, probe_y);
            println!("P9 SANITY {label}: pixel({probe_x},{probe_y}) = {px:?}");
        }
    }
}

/// テクスチャの1ピクセルを読み戻す
fn readback(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    x: u32,
    y: u32,
) -> [u8; 4] {
    let bpr = (W * 4).next_multiple_of(256);
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bpr * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
    let d = slice.get_mapped_range();
    let i = (y * bpr + x * 4) as usize;
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

fn report(label: &str, n: usize, upd: &mut Vec<f64>, res: &mut Vec<f64>, rnd: &mut Vec<f64>) {
    for v in [&mut *upd, &mut *res, &mut *rnd] {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
    let p = |v: &Vec<f64>| v[v.len() / 2];
    println!(
        "P9 {label:>8}: clips={n} update(p50={:.2}ms) resolve(p50={:.2}ms) render(p50={:.2}ms) TOTAL={:.2}ms",
        p(upd),
        p(res),
        p(rnd),
        p(upd) + p(res) + p(rnd)
    );
}

fn collect_clip_ids(doc: &HtmlDocument) -> Vec<usize> {
    fn walk(doc: &HtmlDocument, id: usize, out: &mut Vec<usize>) {
        if let Some(node) = doc.get_node(id) {
            if let Some(el) = node.element_data() {
                if el.attr(blitz_dom::local_name!("class")) == Some("clip") {
                    out.push(id);
                }
            }
            for c in node.children.iter() {
                walk(doc, *c, out);
            }
        }
    }
    let mut v = Vec::new();
    walk(doc, doc.root_element().id, &mut v);
    v
}

fn make_doc(n: usize, xz: f64) -> HtmlDocument {
    let per_row = (n / ROWS).max(1);
    let mut body = String::new();
    for r in 0..ROWS {
        let top = 28.0 + r as f64 * 19.0;
        body.push_str(&format!(
            r#"<div class="tname" style="top:{top}px">layer {r}</div>"#
        ));
        for c in 0..per_row {
            body.push_str(&format!(
                r#"<div class="clip" style="left:{}px; top:{top}px; width:{}px">素材 {c}.mp4</div>"#,
                (c * 190) as f64 * xz,
                150.0 * xz
            ));
        }
    }
    let html = format!(
        r#"<html><head><style>
      html,body {{ margin:0; padding:0; background:rgb(36,36,36);
                  font-family:sans-serif; font-size:11px; color:rgb(214,214,214); }}
      .tname {{ position:absolute; left:0; width:84px; height:17px; background:rgb(47,47,47); }}
      .clip {{ position:absolute; height:17px; background:rgb(150,170,219);
               color:rgb(20,20,20); font-size:10px; overflow:hidden; white-space:nowrap; }}
    </style></head><body>{body}</body></html>"#
    );
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
    doc
}
