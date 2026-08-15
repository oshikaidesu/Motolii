//! P12: 透過合成（Blitz面を Stage の上へ重ねる）の実測。
//!
//! 採択時の未了4「透過合成が未検証」を潰す。P4/P6 で確定済みの
//! 「自前 wgpu29 テクスチャへ Blitz を描く」経路の上に乗り、その先だけを測る。
//!
//! 測るのは3点だけ:
//!   Q1 Blitzのテクスチャはアルファを保持するか（`transparent` / `rgba()` が抜けるか）
//!   Q2 そのテクスチャを別の絵の上へ重ねたとき、期待どおりの合成結果になるか
//!      （プリマルチプライドの扱い。色が濁る / 縁が暗くなる が出ないか）
//!   Q3 一部だけ透過（パネルは不透明、間は完全透過）が成立するか
//!
//! 「たぶん動く」を避けるため、全ピクセルを CPU で計算した期待値と突き合わせ、
//! 出力 PNG も残す。out/ を目で見て確かめられる。

use anyrender_vello_hybrid::{ImageManager, VelloHybridScenePainter};
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_paint::paint_scene;
use blitz_traits::shell::{ColorScheme, Viewport};
use rustc_hash::FxHashMap;
use std::io::Write;
use std::path::PathBuf;
use vello_hybrid::{RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings};
use wgpu_context::DeviceHandle;

const W: u32 = 320;
const H: u32 = 200;
const UI_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// 4区画。左上=不透明パネル / 右上=半透明赤 / 左下=角丸白(AA縁の検査用) /
/// 右下=薄い白。区画の間は**何も描かない** = 完全透過であるべき領域。
///
/// `html, body { background: transparent }` を明示する。
/// blitz-paint は html の背景が透明なら body を見に行き、body も透明なら
/// 「透明黒で塗る」= 実質何も乗らない、という実装（render.rs:127-160）。
const HTML: &str = r#"
<html><head><style>
  html, body { margin: 0; padding: 0; background: transparent; }
  div { position: absolute; }
  .panel { left: 16px;  top: 16px;  width: 112px; height: 72px; background: rgb(45, 45, 45); }
  .glass { left: 192px; top: 16px;  width: 112px; height: 72px; background: rgba(255, 0, 0, 0.5); }
  .round { left: 16px;  top: 112px; width: 112px; height: 72px;
           background: rgb(255, 255, 255); border-radius: 24px; }
  .semi  { left: 192px; top: 112px; width: 112px; height: 72px;
           background: rgba(255, 255, 255, 0.25); }
</style></head><body>
  <div class="panel"></div>
  <div class="glass"></div>
  <div class="round"></div>
  <div class="semi"></div>
</body></html>
"#;

/// 背景の指定を**一切書かない**場合。既定が透明なのか不透明なのかを実測する。
const HTML_NO_BG_RULE: &str = r#"
<html><head><style>
  html, body { margin: 0; padding: 0; }
  .panel { position: absolute; left: 16px; top: 16px; width: 112px; height: 72px;
           background: rgb(45, 45, 45); }
</style></head><body><div class="panel"></div></body></html>
"#;

/// `body` にだけ背景色が付いている場合。`html` が透明なら body を拾いに行く実装
/// (blitz-paint render.rs:127-160) なので、面全体が塗り潰されるはず — を確かめる。
const HTML_BODY_BG: &str = r#"
<html><head><style>
  html { margin: 0; padding: 0; background: transparent; }
  body { margin: 0; padding: 0; background: rgb(24, 24, 24); }
  .panel { position: absolute; left: 16px; top: 16px; width: 112px; height: 72px;
           background: rgb(45, 45, 45); }
</style></head><body><div class="panel"></div></body></html>
"#;

// 検査点（区画中心と、区画の間）
const P_PANEL: (u32, u32) = (72, 52);
const P_GLASS: (u32, u32) = (248, 52);
const P_ROUND: (u32, u32) = (72, 148);
const P_SEMI: (u32, u32) = (248, 148);
const P_GAP_H: (u32, u32) = (160, 52); // パネルとパネルの横の隙間
const P_GAP_V: (u32, u32) = (72, 100); // 縦の隙間
const P_CORNER: (u32, u32) = (2, 2); // 何も無い外周

const TOL: i32 = 2;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    resources: Resources,
    device_handle: DeviceHandle,
}

fn main() {
    let out_dir = out_dir();
    std::fs::create_dir_all(&out_dir).expect("out dir");

    let mut gpu = init_gpu();
    let mut fail: Vec<String> = Vec::new();

    // ================= Q1: Blitz のテクスチャはアルファを保持するか =================
    let ui_tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("blitz-ui"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: UI_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let ui_view = ui_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let mut doc = HtmlDocument::from_html(HTML, DocumentConfig::default());
    doc.set_viewport(Viewport {
        window_size: (W, H),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);
    paint_to(&mut gpu, &mut doc, &ui_view);
    let ui = read_texture(&gpu.device, &gpu.queue, &ui_tex);

    println!("== Q1: アルファ保持 ==");
    for (name, p) in [
        ("外周(何も無い)", P_CORNER),
        ("横の隙間", P_GAP_H),
        ("縦の隙間", P_GAP_V),
        ("panel  rgb(45,45,45)", P_PANEL),
        ("glass  rgba(255,0,0,.5)", P_GLASS),
        ("round  rgb(255,255,255)", P_ROUND),
        ("semi   rgba(255,255,255,.25)", P_SEMI),
    ] {
        println!("  {:<30} {:?}", name, px(&ui, p.0, p.1));
    }

    // 透過が抜けているか
    for (name, p) in [
        ("外周", P_CORNER),
        ("横の隙間", P_GAP_H),
        ("縦の隙間", P_GAP_V),
    ] {
        let v = px(&ui, p.0, p.1);
        if v != [0, 0, 0, 0] {
            fail.push(format!("Q1: {name} が透過していない: {v:?}"));
        }
    }
    // 不透明部は α=255
    for (name, p) in [("panel", P_PANEL), ("round", P_ROUND)] {
        let v = px(&ui, p.0, p.1);
        if v[3] != 255 {
            fail.push(format!("Q1: {name} の α が 255 でない: {v:?}"));
        }
    }
    // 中間αが出るか
    let glass = px(&ui, P_GLASS.0, P_GLASS.1);
    let semi = px(&ui, P_SEMI.0, P_SEMI.1);
    if (glass[3] as i32 - 128).abs() > TOL {
        fail.push(format!("Q1: glass の α が 0.5 相当でない: {glass:?}"));
    }
    if (semi[3] as i32 - 64).abs() > TOL {
        fail.push(format!("Q1: semi の α が 0.25 相当でない: {semi:?}"));
    }

    // プリマルチプライドかストレートか。rgba(255,0,0,0.5) は
    //   premultiplied なら R≈128、straight なら R≈255。
    let premultiplied = (glass[0] as i32 - 128).abs() <= 8;
    let straight = (glass[0] as i32 - 255).abs() <= 8;
    println!(
        "  → アルファ形式: {}",
        if premultiplied {
            "premultiplied (R≈128)"
        } else if straight {
            "straight (R≈255)"
        } else {
            "不明"
        }
    );
    if !premultiplied {
        fail.push(format!(
            "Q1: premultiplied を前提にしていたが glass={glass:?}。合成式を見直す必要がある"
        ));
    }
    // premultiplied なら成分は α を超えない
    if premultiplied {
        for c in 0..3 {
            if glass[c] > glass[3] + TOL as u8 {
                fail.push(format!("Q1: premultiplied 不変条件を破る: {glass:?}"));
            }
        }
    }

    write_png(&out_dir.join("p12_ui_raw.png"), &ui, W, H);
    write_png(&out_dir.join("p12_ui_unpremul.png"), &unpremultiply(&ui), W, H);

    // ================= Q2/Q3: 別の絵の上へ重ねる =================
    // Stage 相当の下地。単色だと合成ミスが隠れるので勾配にする。
    let mut bg = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            bg[i] = (x * 255 / (W - 1)) as u8;
            bg[i + 1] = (y * 255 / (H - 1)) as u8;
            bg[i + 2] = 160;
            bg[i + 3] = 255;
        }
    }
    let bg_tex = upload(&gpu.device, &gpu.queue, &bg);
    write_png(&out_dir.join("p12_bg.png"), &bg, W, H);

    // CPU で計算した期待値（premultiplied over）
    let expected = compose_cpu(&ui, &bg);
    write_png(&out_dir.join("p12_expected.png"), &expected, W, H);

    let blitter = Blitter::new(&gpu.device, UI_FORMAT);

    // 正しい設定: PREMULTIPLIED_ALPHA_BLENDING
    let got_premul = blitter.compose(&gpu, &bg_tex, &ui_tex, BlendUnderTest::Premultiplied);
    write_png(&out_dir.join("p12_composite_premul.png"), &got_premul, W, H);

    // 誤った設定: ストレートα用のブレンド（対照）
    let got_straight = blitter.compose(&gpu, &bg_tex, &ui_tex, BlendUnderTest::Straight);
    write_png(&out_dir.join("p12_composite_straight.png"), &got_straight, W, H);

    let (max_p, bad_p, worst_p) = diff(&expected, &got_premul);
    let (max_s, bad_s, _) = diff(&expected, &got_straight);

    println!("\n== Q2: 合成結果 ==");
    println!(
        "  premultiplied blend: 最大誤差 {max_p} / 許容超え {bad_p} px（最悪座標 {worst_p:?}）"
    );
    println!("  straight blend(誤設定, 対照): 最大誤差 {max_s} / 許容超え {bad_s} px");
    for (name, p) in [
        ("外周(下地が素で出るはず)", P_CORNER),
        ("横の隙間", P_GAP_H),
        ("panel", P_PANEL),
        ("glass", P_GLASS),
        ("semi", P_SEMI),
    ] {
        println!(
            "  {:<26} 期待 {:?} / premul {:?} / straight {:?}",
            name,
            px(&expected, p.0, p.1),
            px(&got_premul, p.0, p.1),
            px(&got_straight, p.0, p.1)
        );
    }

    if max_p > TOL {
        fail.push(format!(
            "Q2: premultiplied 合成が期待値と一致しない（最大誤差 {max_p}, {bad_p}px）"
        ));
    }
    if max_s <= TOL {
        fail.push(
            "Q2: 誤設定(straight)でも同じ絵になった。判別力の無い試験になっている".into(),
        );
    }

    // 縁が暗くならないか。白の角丸を下地の上に置いているので、
    // 正しい合成なら AA 画素は必ず「下地 ≦ 出力 ≦ 白」に収まる。
    // 濁り/暗い縁が出るとこの下限を割る。
    let mut fringe: Option<(u32, u32, [u8; 4], [u8; 4])> = None;
    for y in 112..184 {
        for x in 16..128 {
            let g = px(&got_premul, x, y);
            let b = px(&bg, x, y);
            if (0..3).any(|c| (g[c] as i32) < b[c] as i32 - TOL) {
                fringe = Some((x, y, g, b));
                break;
            }
        }
        if fringe.is_some() {
            break;
        }
    }
    match fringe {
        None => println!("  角丸AA: 下地より暗い画素なし（縁が暗くなる現象は出ていない）"),
        Some((x, y, g, b)) => {
            println!("  角丸AA: ({x},{y}) 出力 {g:?} < 下地 {b:?}");
            fail.push("Q2: AA縁が下地より暗い（premultiplyの取り違え）".into());
        }
    }
    // 実際に AA 中間画素が存在することも確認する（境界が全部 0/1 なら試験になっていない）
    let aa_count = (112..184)
        .flat_map(|y| (16..128).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let a = px(&ui, x, y)[3];
            a > 4 && a < 251
        })
        .count();
    println!("  角丸AA: 中間α画素 {aa_count} 個");
    if aa_count < 32 {
        fail.push("Q2: AA画素がほぼ無い。縁の検査が成立していない".into());
    }

    // ================= Q3: 一部だけ透過 =================
    println!("\n== Q3: 一部だけ透過 ==");
    let mut q3 = true;
    for (name, p, want) in [
        ("外周", P_CORNER, px(&bg, P_CORNER.0, P_CORNER.1)),
        ("横の隙間", P_GAP_H, px(&bg, P_GAP_H.0, P_GAP_H.1)),
        ("縦の隙間", P_GAP_V, px(&bg, P_GAP_V.0, P_GAP_V.1)),
    ] {
        let g = px(&got_premul, p.0, p.1);
        let ok = (0..3).all(|c| (g[c] as i32 - want[c] as i32).abs() <= TOL);
        println!("  {name}: 出力 {g:?} / 下地 {want:?} {}", if ok { "一致" } else { "不一致" });
        if !ok {
            q3 = false;
        }
    }
    for (name, p, want) in [
        ("panel(不透明)", P_PANEL, [45u8, 45, 45]),
        ("round(不透明)", P_ROUND, [255u8, 255, 255]),
    ] {
        let g = px(&got_premul, p.0, p.1);
        let ok = (0..3).all(|c| (g[c] as i32 - want[c] as i32).abs() <= TOL);
        println!(
            "  {name}: 出力 {g:?} / 期待 {want:?} {}（下地が透けていないか）",
            if ok { "一致" } else { "不一致" }
        );
        if !ok {
            q3 = false;
        }
    }
    if !q3 {
        fail.push("Q3: 同一テクスチャ内で不透明/完全透過の使い分けが成立しない".into());
    }

    // ================= 参考: 出力先が sRGB フォーマットだったら =================
    // 本題ではないが「合成側の設定ミス」で最も起きやすいので測っておく。
    let srgb = blitter.compose_srgb(&gpu, &bg_tex, &ui_tex);
    write_png(&out_dir.join("p12_composite_srgb_target.png"), &srgb, W, H);
    let (max_srgb, _, _) = diff(&expected, &srgb);
    println!(
        "\n== 参考: 出力先を Rgba8UnormSrgb にした場合 ==\n  最大誤差 {max_srgb}（panel 出力 {:?} / 期待 {:?}）",
        px(&srgb, P_PANEL.0, P_PANEL.1),
        px(&expected, P_PANEL.0, P_PANEL.1)
    );

    // ================= 参考: 背景指定の書き方で透過が消えるか =================
    // 「CSSが効いているつもりで効いていない」を潰すため、書き方違いを実測する。
    println!("\n== 参考: 背景指定の書き方 ==");
    for (label, html) in [
        ("background 指定なし", HTML_NO_BG_RULE),
        ("body にだけ背景色", HTML_BODY_BG),
    ] {
        let mut d = HtmlDocument::from_html(html, DocumentConfig::default());
        d.set_viewport(Viewport {
            window_size: (W, H),
            hidpi_scale: 1.0,
            zoom: 1.0,
            color_scheme: ColorScheme::Dark,
        });
        d.resolve(0.0);
        paint_to(&mut gpu, &mut d, &ui_view);
        let v = read_texture(&gpu.device, &gpu.queue, &ui_tex);
        println!(
            "  {:<20} 外周 {:?} / panel {:?}",
            label,
            px(&v, P_CORNER.0, P_CORNER.1),
            px(&v, P_PANEL.0, P_PANEL.1)
        );
    }

    println!("\n出力: {}", out_dir.display());
    println!("\nP12 RESULT: {}", if fail.is_empty() { "PASS" } else { "FAIL" });
    for f in &fail {
        println!("  NG: {f}");
    }
    std::process::exit(if fail.is_empty() { 0 } else { 1 });
}

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("out")
}

// ---------------------------------------------------------------- 合成(CPU参照)

/// premultiplied over: out = src + dst * (1 - a_src)
fn compose_cpu(src: &[u8], dst: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; src.len()];
    for i in (0..src.len()).step_by(4) {
        let a = src[i + 3] as f32 / 255.0;
        for c in 0..4 {
            let v = src[i + c] as f32 + dst[i + c] as f32 * (1.0 - a);
            out[i + c] = v.round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

fn unpremultiply(src: &[u8]) -> Vec<u8> {
    let mut out = src.to_vec();
    for i in (0..src.len()).step_by(4) {
        let a = src[i + 3] as f32;
        if a > 0.0 {
            for c in 0..3 {
                out[i + c] = ((src[i + c] as f32 * 255.0 / a).round()).clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

fn diff(a: &[u8], b: &[u8]) -> (i32, usize, (u32, u32)) {
    let mut max = 0i32;
    let mut bad = 0usize;
    let mut worst = (0, 0);
    for i in (0..a.len()).step_by(4) {
        let mut d = 0i32;
        for c in 0..4 {
            d = d.max((a[i + c] as i32 - b[i + c] as i32).abs());
        }
        if d > max {
            max = d;
            let p = (i / 4) as u32;
            worst = (p % W, p / W);
        }
        if d > TOL {
            bad += 1;
        }
    }
    (max, bad, worst)
}

fn px(data: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [data[i], data[i + 1], data[i + 2], data[i + 3]]
}

// ---------------------------------------------------------------- wgpu

fn init_gpu() -> Gpu {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no adapter");
    println!("P12/0 backend={:?}", adapter.get_info().backend);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("motolii-side-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        trace: wgpu::Trace::Off,
    }))
    .expect("no device");
    let renderer = Renderer::new(
        &device,
        &RenderTargetConfig {
            format: UI_FORMAT,
            width: W,
            height: H,
        },
    );
    let device_handle = DeviceHandle {
        instance,
        adapter,
        device: device.clone(),
        queue: queue.clone(),
    };
    Gpu {
        device,
        queue,
        renderer,
        resources: Resources::new(),
        device_handle,
    }
}

/// P4/P6 で確定済みの経路。ここは検証対象ではなく前提。
fn paint_to(gpu: &mut Gpu, doc: &mut HtmlDocument, view: &wgpu::TextureView) {
    let mut scene = Scene::new(W as u16, H as u16);
    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cache = FxHashMap::default();
        let mut bindings = FxHashMap::default();
        let im = ImageManager::new(
            &mut gpu.renderer,
            &mut gpu.resources,
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &mut cache,
        );
        let mut painter =
            VelloHybridScenePainter::new(&mut scene, im, &mut bindings, &gpu.device_handle);
        paint_scene(&mut painter, doc, 1.0, W, H, 0, 0);
    }
    gpu.renderer
        .render(
            &scene,
            &mut gpu.resources,
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &RenderSize {
                width: W,
                height: H,
            },
            view,
            &TextureBindings::default(),
        )
        .expect("render");
    gpu.queue.submit([encoder.finish()]);
}

fn upload(device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) -> wgpu::Texture {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("stage-bg"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: UI_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(W * 4),
            rows_per_image: Some(H),
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    tex
}

#[derive(Clone, Copy)]
enum BlendUnderTest {
    /// 正しい設定（src はプリマルチプライド済み）
    Premultiplied,
    /// ストレートα用。プリマルチプライド済みデータに掛けると二重に α が乗る
    Straight,
}

const SHADER: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    var o: VOut;
    o.pos = vec4(p[i], 0.0, 1.0);
    o.uv = vec2((p[i].x + 1.0) * 0.5, (1.0 - p[i].y) * 0.5);
    return o;
}
@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;
@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> { return textureSample(t, s, in.uv); }
"#;

struct Blitter {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    replace: wgpu::RenderPipeline,
    premul: wgpu::RenderPipeline,
    straight: wgpu::RenderPipeline,
    replace_srgb: wgpu::RenderPipeline,
    premul_srgb: wgpu::RenderPipeline,
}

impl Blitter {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let make = |blend: Option<wgpu::BlendState>, fmt: wgpu::TextureFormat| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: fmt,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let straight_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let srgb = wgpu::TextureFormat::Rgba8UnormSrgb;
        Self {
            replace: make(Some(wgpu::BlendState::REPLACE), format),
            premul: make(Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), format),
            straight: make(Some(straight_blend), format),
            replace_srgb: make(Some(wgpu::BlendState::REPLACE), srgb),
            premul_srgb: make(Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), srgb),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            layout,
        }
    }

    fn bind(&self, device: &wgpu::Device, tex: &wgpu::Texture) -> wgpu::BindGroup {
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// 下地を素で敷き、その上へ UI テクスチャを重ねる。Stage(Rerun) の上に
    /// Blitz 面を載せる構成そのもの。
    fn compose(
        &self,
        gpu: &Gpu,
        bg: &wgpu::Texture,
        ui: &wgpu::Texture,
        mode: BlendUnderTest,
    ) -> Vec<u8> {
        let out = self.target(&gpu.device, UI_FORMAT);
        self.run(
            gpu,
            bg,
            ui,
            &out,
            &self.replace,
            match mode {
                BlendUnderTest::Premultiplied => &self.premul,
                BlendUnderTest::Straight => &self.straight,
            },
        );
        read_texture(&gpu.device, &gpu.queue, &out)
    }

    /// 参考: 出力先が sRGB フォーマットのとき何が起きるか。
    fn compose_srgb(&self, gpu: &Gpu, bg: &wgpu::Texture, ui: &wgpu::Texture) -> Vec<u8> {
        let out = self.target(&gpu.device, wgpu::TextureFormat::Rgba8UnormSrgb);
        self.run(gpu, bg, ui, &out, &self.replace_srgb, &self.premul_srgb);
        read_texture(&gpu.device, &gpu.queue, &out)
    }

    fn target(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("composite"),
            size: wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn run(
        &self,
        gpu: &Gpu,
        bg: &wgpu::Texture,
        ui: &wgpu::Texture,
        out: &wgpu::Texture,
        bg_pipe: &wgpu::RenderPipeline,
        ui_pipe: &wgpu::RenderPipeline,
    ) {
        let bg_bind = self.bind(&gpu.device, bg);
        let ui_bind = self.bind(&gpu.device, ui);
        let view = out.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(bg_pipe);
            pass.set_bind_group(0, &bg_bind, &[]);
            pass.draw(0..3, 0..1);
            pass.set_pipeline(ui_pipe);
            pass.set_bind_group(0, &ui_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([enc.finish()]);
    }
}

/// パディングを外して詰めた RGBA を返す。
fn read_texture(device: &wgpu::Device, queue: &wgpu::Queue, tex: &wgpu::Texture) -> Vec<u8> {
    let bpr = (W * 4).next_multiple_of(256);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (bpr * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
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
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    let padded = slice.get_mapped_range().to_vec();
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        let s = (y * bpr) as usize;
        out.extend_from_slice(&padded[s..s + (W * 4) as usize]);
    }
    out
}

// ---------------------------------------------------------------- PNG(依存追加なし)

/// 無圧縮 deflate(stored) の最小 PNG。目視確認できればよいので圧縮しない。
fn write_png(path: &std::path::Path, rgba: &[u8], w: u32, h: u32) {
    let mut raw = Vec::with_capacity(((w * 4 + 1) * h) as usize);
    for y in 0..h {
        raw.push(0u8); // filter: None
        let s = (y * w * 4) as usize;
        raw.extend_from_slice(&rgba[s..s + (w * 4) as usize]);
    }

    let mut z = vec![0x78u8, 0x01]; // zlib header, no compression
    let mut i = 0usize;
    while i < raw.len() {
        let n = (raw.len() - i).min(65535);
        let last = if i + n == raw.len() { 1u8 } else { 0u8 };
        z.push(last);
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[i..i + n]);
        i += n;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit RGBA
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);

    let mut f = std::fs::File::create(path).expect("png create");
    f.write_all(&png).expect("png write");
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
