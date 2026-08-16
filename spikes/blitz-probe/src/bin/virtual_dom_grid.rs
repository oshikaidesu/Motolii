//! P10: 仮想スクロール(Googleスプレッドシート型)のDOMは、Timelineの
//!      横パン・横ズームに耐えるか。最悪どこまで行けるか。
//!
//! P8 は「全clipをDOMに置くと node 数に比例して resolve が伸びる」を示した。
//! P9 は「毎フレーム left/width を書き換えると resolve が再発する」を示した。
//! 本プローブが埋める穴は、その2つの間にある **窓の出入り(recycling)のコスト**。
//!
//! 表計算アプリは可視セルだけをDOMに置き、外へ出たノードを使い回す。
//! それを Blitz でやったとき、
//!   - 出入りのたびに tree を足し引きする(RECREATE)
//!   - ノードは常設して属性だけ差し替える(RECYCLE)
//! のどちらがいくらかかるか、そして 60fps(16.6ms) の壁がどの可視ノード数で来るか。
//!
//! 測る動き:
//!   PAN  … 横スクロール。幾何は平行移動だが、端で出入りが起きる
//!   ZOOM … 横ズーム。可視ノード全部の left/width が毎フレーム変わり、
//!          さらに可視集合そのものも変わる(Timeline固有の最悪ケース)
//!
//! 実行:
//!   P10_VISIBLE=400 cargo run --release --bin virtual_dom_grid

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::{Attribute, DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::{FxHashMap, FxHashSet};
use std::time::Instant;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const W: u32 = 1200;
const H: u32 = 600;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const ROW_H: f64 = 19.0;
const CLIP_H: f64 = 17.0;
const TOP: f64 = 28.0;
const FRAMES: usize = 120;
const WARMUP: usize = 15;

/// clip の色。sanity 検査で数える。
const CLIP_RGB: [u8; 3] = [150, 170, 219];

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// データ側の1 clip。DOMには可視分しか出さない。
#[derive(Clone, Copy)]
struct Item {
    row: usize,
    /// zoom=1 のときの開始位置(px)
    x0: f64,
    /// zoom=1 のときの幅(px)
    w0: f64,
}

fn main() {
    let rows = env_usize("P10_ROWS", 30).min(((H as f64 - TOP) / ROW_H) as usize);
    let visible_target = env_usize("P10_VISIBLE", 400);
    let total = env_usize("P10_TOTAL", 50_000);
    let with_text = std::env::var("P10_TEXT").is_ok();

    // 目標可視数から間隔を決める。1行あたりの可視 clip 数 = W / pitch。
    let per_row_visible = (visible_target as f64 / rows as f64).max(1.0);
    let pitch = W as f64 / per_row_visible;
    let width = pitch * 0.8;

    let per_row = (total / rows).max(4);
    let mut items = Vec::with_capacity(rows * per_row);
    for r in 0..rows {
        for c in 0..per_row {
            items.push(Item {
                row: r,
                // 行ごとに位相をずらす。全行が同じ列位置だと出入りが1フレームに
                // 集中し、churn の中央値が 0 になって実態を隠す。
                x0: c as f64 * pitch + (r as f64 / rows as f64) * pitch,
                w0: width,
            });
        }
    }
    let content_w = per_row as f64 * pitch;

    println!(
        "P10 setup: rows={rows} total_items={} pitch={pitch:.1}px width={width:.1}px text={with_text} (可視目標={visible_target})",
        items.len()
    );

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

    // (mode ラベル, 動きの種類, 戦略)
    let plan: [(&str, Motion, Strategy); 7] = [
        ("STATIC/mutate", Motion::None, Strategy::Recycle),
        ("PAN/recreate", Motion::Pan, Strategy::Recreate),
        ("PAN/recycle", Motion::Pan, Strategy::Recycle),
        ("ZOOM/recreate", Motion::Zoom, Strategy::Recreate),
        ("ZOOM/recycle", Motion::Zoom, Strategy::Recycle),
        ("FLING/recreate", Motion::Fling, Strategy::Recreate),
        ("FLING/recycle", Motion::Fling, Strategy::Recycle),
    ];

    for (label, motion, strategy) in plan {
        let mut doc = make_doc(rows);
        doc.resolve(0.0);
        let grid = find_by_class(&doc, "grid").expect("grid container");

        let mut live: FxHashMap<usize, usize> = FxHashMap::default(); // item idx -> node id
        let mut pool: Vec<usize> = Vec::new(); // RECYCLE 用の常設ノード
        let mut upd = Vec::new();
        let mut res = Vec::new();
        let mut rnd = Vec::new();
        let mut vis_counts = Vec::new();
        let mut churns = Vec::new();

        for f in 0..FRAMES {
            let (zoom, scroll) = motion.at(f, content_w);

            // ---- 可視集合を出す(これはデータ側の計算。DOMには触らない) ----
            let mut visible: Vec<usize> = Vec::new();
            for (i, it) in items.iter().enumerate() {
                let x = it.x0 * zoom - scroll;
                let w = it.w0 * zoom;
                if x + w > 0.0 && x < W as f64 {
                    visible.push(i);
                }
            }

            let t = Instant::now();
            let churn = match strategy {
                Strategy::Recreate => apply_recreate(
                    &mut doc, grid, &items, &visible, zoom, scroll, with_text, &mut live,
                ),
                Strategy::Recycle => apply_recycle(
                    &mut doc, grid, &items, &visible, zoom, scroll, with_text, &mut pool,
                ),
            };
            let update = t.elapsed().as_secs_f64() * 1000.0;

            let t = Instant::now();
            doc.resolve(0.0);
            let resolve = t.elapsed().as_secs_f64() * 1000.0;

            let render = render_once(&mut doc, &mut renderer, &mut resources);

            if f > WARMUP {
                upd.push(update);
                res.push(resolve);
                rnd.push(render);
                vis_counts.push(visible.len() as f64);
                churns.push(churn as f64);
            }
        }

        let painted = count_clip_pixels(&device, &queue, &texture);
        report(label, &mut vis_counts, &mut churns, &mut upd, &mut res, &mut rnd, painted);
    }
}

#[derive(Clone, Copy)]
enum Motion {
    None,
    Pan,
    Zoom,
    /// 最悪ケース。1フレームで1画面分飛ぶ = 可視ノードが毎フレーム総入れ替え
    Fling,
}

impl Motion {
    /// (zoom, scroll_x) を返す
    fn at(self, f: usize, content_w: f64) -> (f64, f64) {
        match self {
            Motion::None => (1.0, content_w * 0.25),
            // 毎フレーム 11px パン。端の出入りが定常的に起きる速度。
            Motion::Pan => (1.0, wrap(content_w * 0.25 + f as f64 * 11.0, content_w)),
            // P9 と同じ 1.0→2.18 の往復ズーム。
            Motion::Zoom => {
                let z = 1.0 + (f % 60) as f64 * 0.02;
                (z, content_w * 0.25 * z)
            }
            Motion::Fling => (
                1.0,
                wrap(content_w * 0.25 + f as f64 * W as f64, content_w),
            ),
        }
    }
}

/// content の中に収める。長時間パンしても可視が空にならないように。
fn wrap(x: f64, content_w: f64) -> f64 {
    let span = (content_w - W as f64).max(1.0);
    x.rem_euclid(span)
}

#[derive(Clone, Copy)]
enum Strategy {
    /// 出入りのたびに tree を足し引きする素朴な仮想化
    Recreate,
    /// ノードは常設し、属性だけ差し替える(表計算アプリの recycling)
    Recycle,
}

fn style_for(it: &Item, zoom: f64, scroll: f64) -> String {
    let x = it.x0 * zoom - scroll;
    let w = it.w0 * zoom;
    let y = TOP + it.row as f64 * ROW_H;
    format!("left:{x:.1}px; top:{y:.1}px; width:{w:.1}px; height:{CLIP_H}px")
}

/// 戻り値: このフレームの churn(追加+削除ノード数)
fn apply_recreate(
    doc: &mut HtmlDocument,
    grid: usize,
    items: &[Item],
    visible: &[usize],
    zoom: f64,
    scroll: f64,
    with_text: bool,
    live: &mut FxHashMap<usize, usize>,
) -> usize {
    let want: FxHashSet<usize> = visible.iter().copied().collect();
    let mut churn = 0;
    let mut m = doc.mutate();

    // 窓から出たものを外す
    let gone: Vec<usize> = live.keys().copied().filter(|i| !want.contains(i)).collect();
    for i in gone {
        if let Some(node) = live.remove(&i) {
            m.remove_and_drop_node(node);
            churn += 1;
        }
    }

    // 窓に入ったものを作る / 残っているものは属性だけ更新
    let mut created = Vec::new();
    for &i in visible {
        let style = style_for(&items[i], zoom, scroll);
        if let Some(&node) = live.get(&i) {
            m.set_attribute(node, blitz_dom::qual_name!("style"), &style);
        } else {
            let node = m.create_element(
                blitz_dom::qual_name!("div"),
                vec![
                    Attribute {
                        name: blitz_dom::qual_name!("class"),
                        value: "clip".into(),
                    },
                    Attribute {
                        name: blitz_dom::qual_name!("style"),
                        value: style,
                    },
                ],
            );
            if with_text {
                let t = m.create_text_node(&format!("clip {i}"));
                m.append_children(node, &[t]);
            }
            created.push(node);
            live.insert(i, node);
            churn += 1;
        }
    }
    if !created.is_empty() {
        m.append_children(grid, &created);
    }
    churn
}

fn apply_recycle(
    doc: &mut HtmlDocument,
    grid: usize,
    items: &[Item],
    visible: &[usize],
    zoom: f64,
    scroll: f64,
    with_text: bool,
    pool: &mut Vec<usize>,
) -> usize {
    let mut churn = 0;
    let mut m = doc.mutate();

    // 足りない分だけ pool を伸ばす(定常状態では0)
    let mut created = Vec::new();
    while pool.len() < visible.len() {
        let node = m.create_element(
            blitz_dom::qual_name!("div"),
            vec![
                Attribute {
                    name: blitz_dom::qual_name!("class"),
                    value: "clip".into(),
                },
                Attribute {
                    name: blitz_dom::qual_name!("style"),
                    value: "display:none".into(),
                },
            ],
        );
        if with_text {
            let t = m.create_text_node("clip");
            m.append_children(node, &[t]);
        }
        created.push(node);
        pool.push(node);
        churn += 1;
    }
    if !created.is_empty() {
        m.append_children(grid, &created);
    }

    for (slot, &i) in visible.iter().enumerate() {
        m.set_attribute(
            pool[slot],
            blitz_dom::qual_name!("style"),
            &style_for(&items[i], zoom, scroll),
        );
    }
    // 余った slot は隠す
    for &node in pool.iter().skip(visible.len()) {
        m.set_attribute(node, blitz_dom::qual_name!("style"), "display:none");
    }
    churn
}

fn report(
    label: &str,
    vis: &mut Vec<f64>,
    churn: &mut Vec<f64>,
    upd: &mut Vec<f64>,
    res: &mut Vec<f64>,
    rnd: &mut Vec<f64>,
    painted: usize,
) {
    for v in [&mut *vis, &mut *churn, &mut *upd, &mut *res, &mut *rnd] {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
    let p50 = |v: &Vec<f64>| v[v.len() / 2];
    let p95 = |v: &Vec<f64>| v[(v.len() * 95 / 100).min(v.len() - 1)];
    let total = p50(upd) + p50(res) + p50(rnd);
    let worst = |v: &Vec<f64>| *v.last().unwrap();
    println!(
        "P10 {label:>14}: visible={:.0} churn/frame(p50={:.0} max={:.0}) update={:.2} resolve={:.2} render={:.2} TOTAL={total:.2}ms (p95={:.2} 最悪={:.2}ms) [clip画素={painted}]",
        p50(vis),
        p50(churn),
        worst(churn),
        p50(upd),
        p50(res),
        p50(rnd),
        p95(upd) + p95(res) + p95(rnd),
        worst(upd) + worst(res) + worst(rnd),
    );
}

fn find_by_class(doc: &HtmlDocument, class: &str) -> Option<usize> {
    fn walk(doc: &HtmlDocument, id: usize, class: &str, out: &mut Option<usize>) {
        if out.is_some() {
            return;
        }
        if let Some(node) = doc.get_node(id) {
            if let Some(el) = node.element_data() {
                if el.attr(blitz_dom::local_name!("class")) == Some(class) {
                    *out = Some(id);
                    return;
                }
            }
            for c in node.children.iter() {
                walk(doc, *c, class, out);
            }
        }
    }
    let mut v = None;
    walk(doc, doc.root_element().id, class, &mut v);
    v
}

/// 描けているかの検査。clip 色の画素数を数える。0 なら測定は無意味。
fn count_clip_pixels(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> usize {
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
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let d = slice.get_mapped_range();
    let mut n = 0;
    for y in 0..H {
        for x in 0..W {
            let i = (y * bpr + x * 4) as usize;
            if d[i] == CLIP_RGB[0] && d[i + 1] == CLIP_RGB[1] && d[i + 2] == CLIP_RGB[2] {
                n += 1;
            }
        }
    }
    n
}

fn make_doc(rows: usize) -> HtmlDocument {
    let mut names = String::new();
    for r in 0..rows {
        let top = TOP + r as f64 * ROW_H;
        names.push_str(&format!(
            r#"<div class="tname" style="top:{top}px">layer {r}</div>"#
        ));
    }
    let html = format!(
        r#"<html><head><style>
      html,body {{ margin:0; padding:0; background:rgb(36,36,36);
                  font-family:sans-serif; font-size:11px; color:rgb(214,214,214); }}
      .tname {{ position:absolute; left:0; width:84px; height:17px; background:rgb(47,47,47); }}
      .grid {{ position:absolute; left:0; top:0; width:1200px; height:600px; }}
      .clip {{ position:absolute; background:rgb(150,170,219);
               color:rgb(20,20,20); font-size:10px; overflow:hidden; white-space:nowrap; }}
    </style></head><body>{names}<div class="grid"></div></body></html>"#
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
