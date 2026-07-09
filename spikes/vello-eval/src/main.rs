//! R8スパイク: Vello 0.9 + usvg 0.47 の採否評価。
//! 検証点:
//!  1. wgpu 29(本体固定メジャー)と同一デバイスでvelloが動くか
//!  2. 手続き図形(矩形/円)をwgpuテクスチャへレンダし読み戻せるか
//!  3. usvg→velloの自前変換(vello_svgはvello 0.7固定で使えない)が小さく書けるか
//!  4. 出力のアルファがpremultipliedかstraightか(本体はpremul正規形)
//!  5. Renderer::new(シェーダ初期化)と1フレームのおおよその時間

use std::num::NonZeroUsize;
use std::time::Instant;

use vello::kurbo::{Affine, BezPath, Circle, Point, Rect};
use vello::peniko::{Color, Fill};
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};

const W: u32 = 64;
const H: u32 = 48;

fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no adapter");
    println!("adapter: {}", adapter.get_info().name);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("vello-eval"),
        ..Default::default()
    }))
    .expect("no device");

    // 1+5: シェーダ初期化コスト(macOSはSome(1)推奨がvello側docに明記)
    let t0 = Instant::now();
    let mut renderer = Renderer::new(
        &device,
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: NonZeroUsize::new(1),
            pipeline_cache: None,
        },
    )
    .expect("renderer init failed");
    println!("Renderer::new: {:?}", t0.elapsed());

    // 2: 手続き図形。赤の不透明矩形と、半透明(0.5)青の円
    let mut scene = Scene::new();
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(255, 0, 0, 255),
        None,
        &Rect::new(4.0, 4.0, 20.0, 20.0),
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(0, 0, 255, 128),
        None,
        &Circle::new(Point::new(32.0, 36.0), 8.0),
    );

    // 3: usvg→vello自前変換の最小形(パス+単色fillのみ)
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="48">
        <path d="M40 4 L60 4 L50 20 Z" fill="#00ff00"/>
    </svg>"##;
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("svg parse");
    append_group(&mut scene, tree.root());

    // レンダターゲット(velloの要求: Rgba8Unorm + STORAGE_BINDING)
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("vello-target"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let t1 = Instant::now();
    renderer
        .render_to_texture(
            &device,
            &queue,
            &scene,
            &view,
            &RenderParams {
                base_color: Color::TRANSPARENT,
                width: W,
                height: H,
                antialiasing_method: AaConfig::Area,
            },
        )
        .expect("render failed");
    let rgba = read_back(&device, &queue, &texture);
    println!("render+readback: {:?}", t1.elapsed());

    // 4: 検証
    let px = |x: u32, y: u32| -> [u8; 4] {
        let i = ((y * W + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
    };
    let rect_c = px(12, 12); // 矩形内
    let bg = px(0, 47); // 全図形の外
    let circ = px(32, 36); // 円の中心(半透明青のみ)
    let svg_c = px(50, 8); // SVG三角形内

    println!("rect  (12,12) = {rect_c:?} (expect [255,0,0,255])");
    println!("bg    ( 0,47) = {bg:?} (expect [0,0,0,0])");
    println!("circle(32,36) = {circ:?} (premul: [0,0,~128,~128] / straight: [0,0,255,~128])");
    println!("svg   (50, 8) = {svg_c:?} (expect [0,255,0,255])");

    assert_eq!(rect_c, [255, 0, 0, 255], "opaque rect");
    assert_eq!(bg, [0, 0, 0, 0], "transparent background");
    assert_eq!(svg_c, [0, 255, 0, 255], "svg triangle via usvg->vello adapter");
    assert!(circ[3] > 100 && circ[3] < 156, "circle alpha ~0.5");
    let premultiplied = circ[2] < 200; // straightなら255のまま
    println!(
        "alpha semantics: {}",
        if premultiplied {
            "PREMULTIPLIED"
        } else {
            "STRAIGHT"
        }
    );

    println!("PASS: vello 0.9 + wgpu 29 + usvg 0.47 headless render OK");
}

/// usvgのグループを再帰的にvello Sceneへ流し込む(単色fillのパスのみ対応の最小形)。
fn append_group(scene: &mut Scene, group: &usvg::Group) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => append_group(scene, g),
            usvg::Node::Path(p) => {
                let Some(fill) = p.fill() else { continue };
                let usvg::Paint::Color(c) = fill.paint() else {
                    continue;
                };
                let alpha = (fill.opacity().get() * 255.0).round() as u8;
                let color = Color::from_rgba8(c.red, c.green, c.blue, alpha);
                let path = to_kurbo(p.data());
                let t = p.abs_transform();
                // tiny-skia Transform{sx,kx,ky,sy,tx,ty} → kurbo Affine[a,b,c,d,e,f]
                let affine = Affine::new([
                    t.sx as f64,
                    t.ky as f64,
                    t.kx as f64,
                    t.sy as f64,
                    t.tx as f64,
                    t.ty as f64,
                ]);
                scene.fill(Fill::NonZero, affine, color, None, &path);
            }
            _ => {}
        }
    }
}

fn to_kurbo(path: &usvg::tiny_skia_path::Path) -> BezPath {
    use usvg::tiny_skia_path::PathSegment;
    let mut out = BezPath::new();
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => out.move_to((p.x as f64, p.y as f64)),
            PathSegment::LineTo(p) => out.line_to((p.x as f64, p.y as f64)),
            PathSegment::QuadTo(p1, p2) => {
                out.quad_to((p1.x as f64, p1.y as f64), (p2.x as f64, p2.y as f64));
            }
            PathSegment::CubicTo(p1, p2, p3) => out.curve_to(
                (p1.x as f64, p1.y as f64),
                (p2.x as f64, p2.y as f64),
                (p3.x as f64, p3.y as f64),
            ),
            PathSegment::Close => out.close_path(),
        }
    }
    out
}

fn read_back(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let padded = unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (padded * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
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
    slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .unwrap();
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * H) as usize);
    for row in 0..H {
        let start = (row * padded) as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    out
}
