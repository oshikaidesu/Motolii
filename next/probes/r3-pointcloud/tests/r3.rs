//! R3: 点群の撮影検証(D12「rerunだからこそ3D」の実データ受入)。
//!
//! 利用者の実データ(PCL 出力の `fragment.ply`)を `re_renderer` の point_cloud
//! renderer で headless に撮り、**カメラ移動で視差が出る**ことを PNG の画素差分で縛る。
//!
//! ファイルは repo に無い(5.8MB・利用者のローカルパス)。無い環境では
//! `motolii-testkit` の欠落方針どおりスキップする(`MOTOLII_REQUIRE_DEPS=1` で落ちる)。

use std::path::{Path, PathBuf};
use std::time::Instant;

use r3_pointcloud::{load_binary_ply, PlyPointCloud};

/// 利用者のローカルパス。probe 内の定数に留め、無ければスキップする。
const FRAGMENT_PLY_PATH: &str =
    "/Users/member_ottoto/Downloads/livingroom_ply-RWP/fragment.ply";

const OUT_DIR: &str = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii--claude-worktrees-motolii-reset-handoff-bda7f3/1ac8f720-602a-48be-a910-7ba7c703d850/scratchpad/r3";

const RESOLUTION: [u32; 2] = [960, 540];
const BASE_VERTICAL_FOV_DEGREES: f32 = 60.0;

/// パン(center)5枚: カメラ位置を横へ truck する(視線の向きは変えない — 動くだけで
/// 深度に応じた視差が出る、`motolii-core::camera` と同じ意味論)。
const PAN_STEPS: [f32; 5] = [-2.0, -1.0, 0.0, 1.0, 2.0];
/// zoom 2枚: 画角だけ変える(カメラ位置・向きは pan の中央フレームと同じ)。
const ZOOM_FACTORS: [f32; 2] = [1.6, 2.4];

struct RenderedFrame {
    label: String,
    rgba: Vec<u8>,
}

#[test]
fn camera_motion_over_real_point_cloud_produces_parallax() {
    if !Path::new(FRAGMENT_PLY_PATH).exists() {
        motolii_testkit::unavailable_dep(
            "fragment.ply",
            &format!("{FRAGMENT_PLY_PATH} が無い(利用者のローカルデータ)"),
        );
        return;
    }
    std::fs::create_dir_all(OUT_DIR).expect("scratchpad dir");

    // --- PLY 読み込み ---------------------------------------------------
    let parse_start = Instant::now();
    let cloud = load_binary_ply(Path::new(FRAGMENT_PLY_PATH)).expect("load fragment.ply");
    let parse_elapsed = parse_start.elapsed();
    assert!(!cloud.is_empty(), "点群が空");

    let (bbox_min, bbox_max) = cloud.bounds();
    let extents = bbox_max - bbox_min;
    let centroid = cloud.centroid();
    let bounding_radius = cloud.bounding_radius(centroid).max(1e-3);

    println!(
        "R3 PLY 読み込み: {}点 / {:.3}ms / bbox {:?}..{:?} / centroid {:?} / 外接半径 {:.4}",
        cloud.len(),
        parse_elapsed.as_secs_f64() * 1000.0,
        bbox_min,
        bbox_max,
        centroid,
        bounding_radius
    );

    // --- CPU 側の点群バッファ(1回だけ作る。GPU アップロードは毎フレーム) ---
    let point_radius_units = point_radius(&extents, cloud.len());
    let (positions_and_radii, colors) = to_render_buffers(&cloud, point_radius_units);
    println!("R3 点の半径(world units): {point_radius_units:.5}");

    // --- カメラの土台: 重心を向く固定向き、外接球がだいたい画角に収まる距離 ---
    let base_fov_radians = BASE_VERTICAL_FOV_DEGREES.to_radians();
    let base_distance = bounding_radius / (base_fov_radians * 0.5).sin() * 1.15;
    let view_dir = choose_view_direction(&extents);
    let base_eye = centroid + view_dir * base_distance;
    let forward = (centroid - base_eye).normalize();
    let up = choose_up(forward);
    let right = forward.cross(up).normalize();

    let pan_step_units = bounding_radius * 0.12;

    // --- GPU headless ----------------------------------------------------
    let gpu = motolii_compositor::HeadlessGpu::new().expect("headless gpu");
    let mut ctx = re_renderer::RenderContext::new(
        &gpu.adapter,
        gpu.device,
        gpu.queue,
        re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
        |_caps| re_renderer::RenderConfig {
            msaa_mode: re_renderer::MsaaMode::Off,
        },
    )
    .expect("render context");

    let mut frames: Vec<RenderedFrame> = Vec::new();
    let mut view_id: u64 = 1;
    let mut total_render = std::time::Duration::ZERO;

    for (i, &step) in PAN_STEPS.iter().enumerate() {
        let eye = base_eye + right * (step * pan_step_units);
        let view_from_world = macaw::IsoTransform::look_at_rh(eye, eye + forward, up)
            .expect("degenerate look_at (pan)");

        let start = Instant::now();
        let rgba = render_frame(
            &mut ctx,
            RESOLUTION,
            &positions_and_radii,
            &colors,
            view_from_world,
            base_fov_radians,
            &mut view_id,
        );
        total_render += start.elapsed();

        let label = format!("r3_pan_{i:02}");
        write_png(OUT_DIR, &label, RESOLUTION, &rgba);
        frames.push(RenderedFrame { label, rgba });
    }

    // zoom フレームは pan の中央(step=0)と同じ eye/向きから、画角だけ変える。
    let center_view_from_world =
        macaw::IsoTransform::look_at_rh(base_eye, base_eye + forward, up)
            .expect("degenerate look_at (zoom)");
    let half_base_fov = base_fov_radians * 0.5;
    for (i, &zoom) in ZOOM_FACTORS.iter().enumerate() {
        let vertical_fov = 2.0 * (half_base_fov.tan() / zoom).atan();

        let start = Instant::now();
        let rgba = render_frame(
            &mut ctx,
            RESOLUTION,
            &positions_and_radii,
            &colors,
            center_view_from_world,
            vertical_fov,
            &mut view_id,
        );
        total_render += start.elapsed();

        let label = format!("r3_zoom_{i:02}");
        write_png(OUT_DIR, &label, RESOLUTION, &rgba);
        frames.push(RenderedFrame { label, rgba });
    }

    assert_eq!(frames.len(), 7, "PNG は7枚(pan5 + zoom2)のはず");
    println!(
        "R3 描画: {}枚 合計 {:.2}ms(平均 {:.2}ms/枚)",
        frames.len(),
        total_render.as_secs_f64() * 1000.0,
        total_render.as_secs_f64() * 1000.0 / frames.len() as f64
    );

    // --- 視差の機械検証: 隣接フレームの画素が実際に変化しているか ---------
    // pan は5枚とも同一の draw data・同一の画角で eye だけ動かしている。動いていれば
    // 隣接フレーム間で非自明な割合の画素が変わるはず(=視差が写っている証拠)。
    let mut min_pan_diff_ratio = f64::INFINITY;
    for w in 0..PAN_STEPS.len() - 1 {
        let (diff_ratio, mean_abs_diff) = pixel_diff(&frames[w].rgba, &frames[w + 1].rgba);
        println!(
            "R3 視差 {} -> {}: 差分画素比 {:.4} / 平均絶対差 {:.3}",
            frames[w].label,
            frames[w + 1].label,
            diff_ratio,
            mean_abs_diff
        );
        min_pan_diff_ratio = min_pan_diff_ratio.min(diff_ratio);
        assert!(
            diff_ratio > 0.01,
            "{} -> {} の差分画素比が {diff_ratio:.4} — カメラを動かしても絵がほぼ変わっていない",
            frames[w].label,
            frames[w + 1].label
        );
    }

    // zoom も同様に、隣接(中央pan → zoom0 → zoom1)で画が変わっていることを縛る。
    let zoom_pairs = [
        (2usize, 5usize), // pan中央 -> zoom0
        (5usize, 6usize), // zoom0 -> zoom1
    ];
    for (a, b) in zoom_pairs {
        let (diff_ratio, mean_abs_diff) = pixel_diff(&frames[a].rgba, &frames[b].rgba);
        println!(
            "R3 zoom差分 {} -> {}: 差分画素比 {:.4} / 平均絶対差 {:.3}",
            frames[a].label, frames[b].label, diff_ratio, mean_abs_diff
        );
        assert!(
            diff_ratio > 0.01,
            "{} -> {} の差分画素比が {diff_ratio:.4} — zoom を変えても絵がほぼ変わっていない",
            frames[a].label,
            frames[b].label
        );
    }

    println!("R3 pan 隣接差分の最小画素比: {min_pan_diff_ratio:.4}(視差が最も乏しかった組)");

    for path in &frames {
        let full = PathBuf::from(OUT_DIR).join(format!("{}.png", path.label));
        assert!(full.exists(), "PNG が実在しない: {full:?}");
    }
}

/// bbox の一番薄い軸を「上」とみなす(部屋スキャンなら高さ方向が最も薄いことが多い)。
/// 完全な当てずっぽうではなく、Y軸と近すぎる/ほぼ0extentの場合のフォールバックも持つ。
fn choose_view_direction(extents: &glam::Vec3) -> glam::Vec3 {
    // 一番大きい2軸を含む面から斜め上に見下ろす向き。厳密な意味論を狙う数値ではなく、
    // 「点群が画角にだいたい収まる、かつ視線が単一軸に縮退しない」ための実用値。
    let thinnest = if extents.x <= extents.y && extents.x <= extents.z {
        glam::Vec3::X
    } else if extents.y <= extents.x && extents.y <= extents.z {
        glam::Vec3::Y
    } else {
        glam::Vec3::Z
    };
    (thinnest * 0.6 + glam::Vec3::new(0.55, 0.35, 0.75)).normalize()
}

fn choose_up(forward: glam::Vec3) -> glam::Vec3 {
    if forward.dot(glam::Vec3::Y).abs() > 0.98 {
        glam::Vec3::Z
    } else {
        glam::Vec3::Y
    }
}

/// 点間隔のだいたいの見積もりから、隙間が見え過ぎない半径を出す。
fn point_radius(extents: &glam::Vec3, point_count: usize) -> f32 {
    let volume = extents.x.max(1e-3) * extents.y.max(1e-3) * extents.z.max(1e-3);
    let avg_spacing = (volume / point_count.max(1) as f32).cbrt();
    (avg_spacing * 0.9).max(1e-4)
}

fn to_render_buffers(
    cloud: &PlyPointCloud,
    radius: f32,
) -> (Vec<re_renderer::PositionRadius>, Vec<re_renderer::Color32>) {
    let size = re_renderer::Size::new_scene_units(radius);
    let positions_and_radii = cloud
        .positions
        .iter()
        .map(|p| re_renderer::PositionRadius {
            pos: glam::Vec3::from(*p),
            radius: size,
        })
        .collect();
    let colors = cloud
        .colors
        .iter()
        .map(|c| re_renderer::Color32::from_rgb(c[0], c[1], c[2]))
        .collect();
    (positions_and_radii, colors)
}

/// 1フレーム描く。`motolii-compositor::Compositor::render_with_timing` と同じ
/// begin_frame → draw → submit → poll → begin_frame(×2) の段取り(headless の読み戻し
/// はこの順でしか揃わない、compositor 側の doc コメント参照)。ここは
/// `RectangleDrawData` の代わりに `PointCloudDrawData` を積むだけの違い。
#[allow(clippy::too_many_arguments)]
fn render_frame(
    ctx: &mut re_renderer::RenderContext,
    resolution: [u32; 2],
    positions_and_radii: &[re_renderer::PositionRadius],
    colors: &[re_renderer::Color32],
    view_from_world: macaw::IsoTransform,
    vertical_fov_radians: f32,
    next_id: &mut u64,
) -> Vec<u8> {
    let id = *next_id;
    *next_id += 1;

    ctx.begin_frame();

    let mut builder = re_renderer::PointCloudBuilder::new(ctx);
    builder
        .reserve(positions_and_radii.len())
        .expect("reserve points");
    builder
        .batch("fragment.ply")
        .add_points(positions_and_radii, colors, &[]);
    let draw_data = builder.into_draw_data().expect("point cloud draw data");

    let aspect_ratio = resolution[0] as f32 / resolution[1] as f32;
    let mut view_builder = re_renderer::ViewBuilder::new(
        ctx,
        re_renderer::view_builder::TargetConfiguration {
            name: "r3-pointcloud".into(),
            render_mode: re_renderer::RenderMode::Deterministic,
            resolution_in_pixel: resolution,
            view_from_world,
            projection_from_view: re_renderer::view_builder::Projection::Perspective {
                vertical_fov: vertical_fov_radians,
                near_plane_distance: 0.01,
                aspect_ratio,
            },
            pixels_per_point: 1.0,
            ..Default::default()
        },
        re_renderer::ViewBuilderId::new(id),
    )
    .expect("view builder");

    view_builder.queue_draw(ctx, draw_data);
    view_builder
        .schedule_screenshot(ctx, id, ())
        .expect("schedule screenshot");
    let command_buffer = view_builder
        .draw(ctx, re_renderer::Rgba::TRANSPARENT)
        .expect("draw");

    ctx.before_submit();
    ctx.queue.submit([command_buffer]);

    // headless には窓が無いので、compositor と同じく同一呼び出しの中でフレームを
    // 2回進めて読み戻す(1回目で map_async 開始、poll で完了待ち、2回目で受け取り)。
    ctx.begin_frame();
    ctx.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");

    ctx.begin_frame();
    let mut out: Option<Vec<u8>> = None;
    re_renderer::ScreenshotProcessor::next_readback_result::<()>(ctx, id, |data, _extent, ()| {
        out = Some(data.to_vec());
    });
    out.expect("screenshot readback missing")
}

fn write_png(dir: &str, label: &str, resolution: [u32; 2], rgba: &[u8]) {
    let path = PathBuf::from(dir).join(format!("{label}.png"));
    image::save_buffer(
        &path,
        rgba,
        resolution[0],
        resolution[1],
        image::ColorType::Rgba8,
    )
    .unwrap_or_else(|e| panic!("PNG 書き出し失敗 {path:?}: {e}"));
}

/// 2枚の RGBA8 フレームの間で、有意に変わった画素の比率と平均絶対差を返す。
/// 「視差の存在」の機械検証はこの数値の閾値で行う(目視ではなく)。
fn pixel_diff(a: &[u8], b: &[u8]) -> (f64, f64) {
    assert_eq!(a.len(), b.len(), "フレームサイズが揃っていない");
    let mut changed_pixels = 0usize;
    let mut abs_diff_sum: u64 = 0;
    let pixel_count = a.len() / 4;
    for i in 0..pixel_count {
        let base = i * 4;
        let mut pixel_abs_diff: u32 = 0;
        for c in 0..4 {
            pixel_abs_diff += (a[base + c] as i32 - b[base + c] as i32).unsigned_abs();
        }
        abs_diff_sum += pixel_abs_diff as u64;
        // チャンネル合計差が小さい画素は GPU の丸め誤差・アンチエイリアス相当のノイズ
        // として無視する(この probe は MSAA off だが、点の縁で数値が数階調揺れうる)。
        if pixel_abs_diff > 12 {
            changed_pixels += 1;
        }
    }
    let diff_ratio = changed_pixels as f64 / pixel_count.max(1) as f64;
    let mean_abs_diff = abs_diff_sum as f64 / (pixel_count.max(1) as f64 * 4.0);
    (diff_ratio, mean_abs_diff)
}
