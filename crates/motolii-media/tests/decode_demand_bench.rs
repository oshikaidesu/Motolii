//! M4 decode需要matrix。閾値を持たない手動実機bench。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use motolii_media::{probe, FrameReader};
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
use serde::Serialize;

const FIXTURE_ENV: &str = "MOTOLII_DECODE_FIXTURE";
const OUTPUT_ENV: &str = "MOTOLII_DECODE_DEMAND_OUT";
const SEQUENTIAL_FRAMES: usize = 120;
const SEEK_TARGETS: &[i64] = &[0, 240, 30, 210, 60, 180, 90, 150, 120, 270, 15, 225];
const PARALLEL_TARGETS: &[i64] = &[0, 30, 60, 90, 120, 150, 180, 210];

#[derive(Debug, Serialize)]
struct DemandSample {
    target_frame: i64,
    elapsed_ms: f64,
}

#[derive(Debug, Serialize)]
struct DecodeDemandReport {
    schema_version: u32,
    fixture: String,
    generated_fixture: bool,
    width: u32,
    height: u32,
    fps_num: i64,
    fps_den: i64,
    sequential: Vec<DemandSample>,
    seeks: Vec<DemandSample>,
    parallel_wall_ms: f64,
    parallel: Vec<DemandSample>,
}

fn generated_fixture() -> PathBuf {
    let path = tmp_dir("m4-decode-demand").join("h264-720p30-10s.mp4");
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=1280x720:rate=30:duration=10",
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-g",
            "60",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&path)
        .status()
        .expect("spawn ffmpeg fixture generator");
    assert!(status.success(), "ffmpeg fixture generation failed");
    path
}

fn fixture_path() -> (PathBuf, bool) {
    match std::env::var_os(FIXTURE_ENV) {
        Some(path) => (PathBuf::from(path), false),
        None => (generated_fixture(), true),
    }
}

fn sequential_samples(path: &Path) -> Vec<DemandSample> {
    let info = probe(path).expect("probe sequential fixture");
    let mut reader = FrameReader::open(path, &info, 0).expect("open sequential reader");
    (0..SEQUENTIAL_FRAMES)
        .map(|target_frame| {
            let start = Instant::now();
            let frame = reader
                .next_frame()
                .expect("decode sequential frame")
                .expect("fixture has enough sequential frames");
            assert_eq!(
                frame.pts.try_to_frame_floor(info.fps).unwrap(),
                target_frame as i64
            );
            DemandSample {
                target_frame: target_frame as i64,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        })
        .collect()
}

fn seek_samples(path: &Path) -> Vec<DemandSample> {
    let info = probe(path).expect("probe seek fixture");
    SEEK_TARGETS
        .iter()
        .map(|&target_frame| {
            let start = Instant::now();
            let mut reader =
                FrameReader::open(path, &info, target_frame).expect("open seek reader");
            let frame = reader
                .next_frame()
                .expect("decode seek frame")
                .expect("seek target exists");
            assert_eq!(
                frame.pts.try_to_frame_floor(info.fps).unwrap(),
                target_frame
            );
            DemandSample {
                target_frame,
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        })
        .collect()
}

fn parallel_samples(path: &Path) -> (f64, Vec<DemandSample>) {
    let wall = Instant::now();
    let workers: Vec<_> = PARALLEL_TARGETS
        .iter()
        .map(|&target_frame| {
            let path = path.to_owned();
            std::thread::spawn(move || {
                let info = probe(&path).expect("probe parallel fixture");
                let start = Instant::now();
                let mut reader =
                    FrameReader::open(&path, &info, target_frame).expect("open parallel reader");
                let frame = reader
                    .next_frame()
                    .expect("decode parallel frame")
                    .expect("parallel target exists");
                assert_eq!(
                    frame.pts.try_to_frame_floor(info.fps).unwrap(),
                    target_frame
                );
                DemandSample {
                    target_frame,
                    elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                }
            })
        })
        .collect();
    let samples = workers
        .into_iter()
        .map(|worker| worker.join().expect("parallel decode worker"))
        .collect();
    (wall.elapsed().as_secs_f64() * 1000.0, samples)
}

#[test]
#[ignore = "manual hardware benchmark; run with --ignored --nocapture"]
fn record_decode_demand_matrix_without_thresholds() {
    if !ffmpeg_or_skip() {
        return;
    }
    let (path, generated_fixture) = fixture_path();
    let info = probe(&path).expect("probe demand fixture");
    let sequential = sequential_samples(&path);
    let seeks = seek_samples(&path);
    let (parallel_wall_ms, parallel) = parallel_samples(&path);
    let report = DecodeDemandReport {
        schema_version: 1,
        fixture: path.display().to_string(),
        generated_fixture,
        width: info.width,
        height: info.height,
        fps_num: info.fps.num(),
        fps_den: info.fps.den(),
        sequential,
        seeks,
        parallel_wall_ms,
        parallel,
    };
    let json = serde_json::to_string_pretty(&report).expect("serialize decode demand report");
    eprintln!("{json}");
    if let Some(path) = std::env::var_os(OUTPUT_ENV) {
        std::fs::write(&path, &json)
            .unwrap_or_else(|error| panic!("write {}: {error}", PathBuf::from(path).display()));
    }
}
