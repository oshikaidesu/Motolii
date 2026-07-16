//! M3実装ガード2: タイムライン1枚描画ベンチスパイク。
//!
//! クリップ1,000 + キーフレーム100,000 を wgpu テクスチャ1枚へ自前描画し、
//! パン/ズーム更新で 60fps を満たすかヘッドレス計測する。
//!
//! 実行:
//!   `cargo run --release`
//!   `cargo run --release -- --json`
//!   `TIMELINE_BENCH_EVIDENCE=../../docs/spikes/timeline-bench-evidence cargo run --release`

mod data;
mod renderer;

use std::time::{Duration, Instant};

use motolii_gpu::{download_rgba, GpuCtx};

use crate::data::{TimelineModel, ViewState};
use crate::renderer::TimelineRenderer;

const CLIP_COUNT: usize = 1_000;
const KEYFRAME_COUNT: usize = 100_000;
const TARGET_FPS: f64 = 60.0;
const FRAME_BUDGET: Duration = Duration::from_nanos((1_000_000_000.0 / TARGET_FPS) as u64);
const DEFAULT_WARMUP: u32 = 120;
const DEFAULT_MEASURE: u32 = 600;

#[derive(serde::Serialize)]
struct BenchReport {
    ticket: &'static str,
    adapter: String,
    backend: String,
    clips: usize,
    keyframes: usize,
    viewport: [u32; 2],
    warmup_frames: u32,
    measured_frames: u32,
    target_fps: f64,
    median_frame_ms: f64,
    p95_frame_ms: f64,
    fps_median: f64,
    fps_p95: f64,
    pass: bool,
    visible_clips_last: usize,
    visible_keyframes_last: usize,
    cpu_cull_upload_us_last: u64,
    measurement: &'static str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = std::env::args().any(|a| a == "--json");
    let warmup = env_u32("TIMELINE_BENCH_WARMUP", DEFAULT_WARMUP);
    let measure = env_u32("TIMELINE_BENCH_FRAMES", DEFAULT_MEASURE);

    let gpu = GpuCtx::new_headless()?;
    let adapter = gpu
        .adapter_info
        .as_ref()
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "unknown".into());
    let backend = gpu
        .adapter_info
        .as_ref()
        .map(|i| format!("{:?}", i.backend))
        .unwrap_or_else(|| "unknown".into());
    eprintln!("adapter: {adapter} ({backend})");

    let model = TimelineModel::generate(CLIP_COUNT, KEYFRAME_COUNT);
    eprintln!(
        "model: {} clips, {} keyframes, {} tracks",
        model.clips.len(),
        model.keyframes.len(),
        model.track_count
    );

    let mut renderer = TimelineRenderer::new(&gpu);
    let mut frame_times = Vec::with_capacity(measure as usize);
    let mut last_stats = renderer.draw_frame(&gpu, &model, ViewState::animate(0, &model));

    let total_frames = warmup + measure;
    for frame in 0..total_frames {
        let view = ViewState::animate(frame, &model);
        let t0 = Instant::now();
        last_stats = renderer.draw_frame(&gpu, &model, view);
        let elapsed = t0.elapsed();
        if frame >= warmup {
            frame_times.push(elapsed);
        }
    }

    frame_times.sort_by_key(|d| d.as_nanos());
    let median = frame_times[frame_times.len() / 2];
    let p95_idx = ((frame_times.len() as f64) * 0.95).floor() as usize;
    let p95 = frame_times[p95_idx.min(frame_times.len() - 1)];
    let median_ms = median.as_secs_f64() * 1000.0;
    let p95_ms = p95.as_secs_f64() * 1000.0;
    let fps_median = 1000.0 / median_ms;
    let fps_p95 = 1000.0 / p95_ms;
    let pass = p95 <= FRAME_BUDGET;

    if let Ok(dir) = std::env::var("TIMELINE_BENCH_EVIDENCE") {
        dump_evidence(&gpu, &renderer, &dir)?;
    }

    let report = BenchReport {
        ticket: "M3-guard-2",
        adapter,
        backend,
        clips: model.clips.len(),
        keyframes: model.keyframes.len(),
        viewport: [1920, 512],
        warmup_frames: warmup,
        measured_frames: measure,
        target_fps: TARGET_FPS,
        median_frame_ms: median_ms,
        p95_frame_ms: p95_ms,
        fps_median,
        fps_p95,
        pass,
        visible_clips_last: last_stats.visible_clips,
        visible_keyframes_last: last_stats.visible_keyframes,
        cpu_cull_upload_us_last: last_stats.cpu_cull_upload_us,
        measurement: "headless wgpu render_to_texture; each frame updates pan/zoom, CPU culls visible clips/keyframes, uploads instance buffer, single render pass; wall-clock frame time includes GPU submit+poll",
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("=== timeline-bench (M3 guard 2) ===");
        println!("clips: {}", report.clips);
        println!("keyframes: {}", report.keyframes);
        println!("warmup: {}  measured: {}", warmup, measure);
        println!("median: {median_ms:.3} ms ({fps_median:.1} fps)");
        println!("p95:    {p95_ms:.3} ms ({fps_p95:.1} fps)");
        println!(
            "visible (last frame): {} clips, {} keyframes",
            last_stats.visible_clips, last_stats.visible_keyframes
        );
        println!(
            "cpu cull+upload (last): {} µs",
            last_stats.cpu_cull_upload_us
        );
        println!("target: {TARGET_FPS} fps (p95 <= {:.3} ms)", FRAME_BUDGET.as_secs_f64() * 1000.0);
        println!("result: {}", if pass { "PASS" } else { "FAIL" });
    }

    if !pass {
        std::process::exit(1);
    }
    Ok(())
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn dump_evidence(
    gpu: &GpuCtx,
    renderer: &TimelineRenderer,
    dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;
    let rgba = download_rgba(gpu, renderer.texture())?;
    let path = std::path::Path::new(dir).join("frame-sample.png");
    image::save_buffer(
        &path,
        &rgba,
        1920,
        512,
        image::ColorType::Rgba8,
    )?;
    eprintln!("wrote evidence PNG: {}", path.display());
    Ok(())
}
