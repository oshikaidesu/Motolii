//! M4 decode需要matrix。閾値を持たない手動実機bench。

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use motolii_media::{probe, FrameReader};
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
use serde::Serialize;
use sha2::{Digest, Sha256};

const FIXTURE_ENV: &str = "MOTOLII_DECODE_FIXTURE";
const OUTPUT_ENV: &str = "MOTOLII_DECODE_DEMAND_OUT";
const HWACCEL_ENV: &str = "MOTOLII_DECODE_HWACCEL";
const HW_OUTPUT_FORMAT_ENV: &str = "MOTOLII_DECODE_HW_OUTPUT_FORMAT";
const HW_SURFACE_FORMAT_ENV: &str = "MOTOLII_DECODE_HW_SURFACE_FORMAT";
const SEQUENTIAL_FRAMES: usize = 120;
const SEEK_TARGETS: &[i64] = &[0, 240, 30, 210, 60, 180, 90, 150, 120, 270, 15, 225];
const PARALLEL_TARGETS: &[i64] = &[0, 30, 60, 90, 120, 150, 180, 210];

#[derive(Debug, Serialize)]
struct DemandSample {
    target_frame: i64,
    elapsed_ms: f64,
}

#[derive(Debug, Serialize)]
struct CommandDemandSample {
    target_frame: i64,
    elapsed_ms: f64,
    output_bytes: u64,
}

#[derive(Debug, Serialize)]
struct CommandRouteReport {
    name: &'static str,
    hwaccel: Option<String>,
    hw_output_format: Option<String>,
    hw_surface_format: Option<String>,
    sequential: CommandDemandSample,
    seeks: Vec<CommandDemandSample>,
    parallel_wall_ms: f64,
    parallel: Vec<CommandDemandSample>,
}

#[derive(Debug, Serialize)]
struct ByteDiff {
    compared_bytes: usize,
    differing_bytes: usize,
    max_abs_diff: u8,
    mean_abs_diff: f64,
}

#[derive(Debug, Serialize)]
struct CommandRouteComparison {
    software: CommandRouteReport,
    hardware: CommandRouteReport,
    frame_zero_diff: ByteDiff,
}

#[derive(Debug, Serialize)]
struct DecodeDemandReport {
    schema_version: u32,
    fixture: String,
    fixture_bytes: u64,
    fixture_sha256: String,
    generated_fixture: bool,
    width: u32,
    height: u32,
    fps_num: i64,
    fps_den: i64,
    sequential: Vec<DemandSample>,
    seeks: Vec<DemandSample>,
    parallel_wall_ms: f64,
    parallel: Vec<DemandSample>,
    command_route_comparison: Option<CommandRouteComparison>,
}

#[derive(Clone)]
struct CommandRoute {
    name: &'static str,
    hwaccel: Option<String>,
    hw_output_format: Option<String>,
    hw_surface_format: Option<String>,
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

fn fixture_identity(path: &Path) -> (u64, String) {
    let mut file = File::open(path).expect("open decode fixture for digest");
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .expect("read decode fixture for digest");
        if read == 0 {
            break;
        }
        bytes = bytes
            .checked_add(read as u64)
            .expect("fixture size overflow");
        digest.update(&buffer[..read]);
    }
    (bytes, format!("{:x}", digest.finalize()))
}

#[test]
fn fixture_identity_depends_on_content_not_path() {
    let dir = tmp_dir("m4-decode-fixture-identity");
    let first = dir.join("first.mp4");
    let second = dir.join("second.mp4");
    std::fs::write(&first, b"same-fixture").unwrap();
    std::fs::write(&second, b"same-fixture").unwrap();

    assert_eq!(fixture_identity(&first), fixture_identity(&second));
    std::fs::write(&second, b"different-fixture").unwrap();
    assert_ne!(fixture_identity(&first), fixture_identity(&second));
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

fn hardware_route_from_env() -> Option<CommandRoute> {
    let hwaccel = std::env::var(HWACCEL_ENV).ok()?;
    let hw_output_format = std::env::var(HW_OUTPUT_FORMAT_ENV)
        .unwrap_or_else(|_| panic!("{HW_OUTPUT_FORMAT_ENV} is required when {HWACCEL_ENV} is set"));
    let hw_surface_format =
        std::env::var(HW_SURFACE_FORMAT_ENV).unwrap_or_else(|_| "nv12".to_owned());
    Some(CommandRoute {
        name: "hardware-download",
        hwaccel: Some(hwaccel),
        hw_output_format: Some(hw_output_format),
        hw_surface_format: Some(hw_surface_format),
    })
}

fn software_route() -> CommandRoute {
    CommandRoute {
        name: "software",
        hwaccel: None,
        hw_output_format: None,
        hw_surface_format: None,
    }
}

fn decode_command(
    path: &Path,
    info: &motolii_media::MediaInfo,
    route: &CommandRoute,
    target_frame: i64,
    frame_count: usize,
) -> Command {
    assert_eq!(
        info.rotation, 0,
        "command route comparison currently requires rotation=0"
    );
    let mut command = Command::new("ffmpeg");
    command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-noautorotate",
    ]);
    if let (Some(hwaccel), Some(output_format)) = (&route.hwaccel, &route.hw_output_format) {
        command
            .args(["-hwaccel", hwaccel])
            .args(["-hwaccel_output_format", output_format]);
    }
    if target_frame > 0 {
        let seek = motolii_core::format_ffmpeg_seek_before_frame(target_frame, info.fps)
            .expect("format command route seek");
        command.args(["-ss", &seek]);
    }
    command.arg("-i").arg(path).args(["-map", "0:v:0"]);
    if let Some(surface_format) = &route.hw_surface_format {
        command.args([
            "-vf",
            &format!("hwdownload,format={surface_format},format=yuv420p"),
        ]);
    }
    command
        .args(["-frames:v", &frame_count.to_string()])
        .args(["-f", "rawvideo", "-pix_fmt", "yuv420p", "-"]);
    command
}

fn run_command_demand(
    path: &Path,
    info: &motolii_media::MediaInfo,
    route: &CommandRoute,
    target_frame: i64,
    frame_count: usize,
) -> CommandDemandSample {
    let start = Instant::now();
    let mut child = decode_command(path, info, route, target_frame, frame_count)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn command decode route");
    let output_bytes = io::copy(
        child.stdout.as_mut().expect("command decode stdout"),
        &mut io::sink(),
    )
    .expect("drain command decode route");
    let status = child.wait().expect("wait command decode route");
    assert!(
        status.success(),
        "{} decode route failed at frame {target_frame}",
        route.name
    );
    CommandDemandSample {
        target_frame,
        elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        output_bytes,
    }
}

fn capture_command_frame(
    path: &Path,
    info: &motolii_media::MediaInfo,
    route: &CommandRoute,
    target_frame: i64,
) -> Vec<u8> {
    let output = decode_command(path, info, route, target_frame, 1)
        .output()
        .expect("capture command decode frame");
    assert!(
        output.status.success(),
        "{} frame capture failed: {}",
        route.name,
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn command_route_report(
    path: &Path,
    info: &motolii_media::MediaInfo,
    route: CommandRoute,
) -> CommandRouteReport {
    let sequential = run_command_demand(path, info, &route, 0, SEQUENTIAL_FRAMES);
    let seeks = SEEK_TARGETS
        .iter()
        .map(|&target_frame| run_command_demand(path, info, &route, target_frame, 1))
        .collect();
    let wall = Instant::now();
    let workers: Vec<_> = PARALLEL_TARGETS
        .iter()
        .map(|&target_frame| {
            let path = path.to_owned();
            let info = info.clone();
            let route = route.clone();
            std::thread::spawn(move || run_command_demand(&path, &info, &route, target_frame, 1))
        })
        .collect();
    let parallel = workers
        .into_iter()
        .map(|worker| worker.join().expect("parallel command decode worker"))
        .collect();
    CommandRouteReport {
        name: route.name,
        hwaccel: route.hwaccel,
        hw_output_format: route.hw_output_format,
        hw_surface_format: route.hw_surface_format,
        sequential,
        seeks,
        parallel_wall_ms: wall.elapsed().as_secs_f64() * 1000.0,
        parallel,
    }
}

fn byte_diff(actual: &[u8], expected: &[u8]) -> ByteDiff {
    assert_eq!(actual.len(), expected.len(), "decoded frame size mismatch");
    let mut differing_bytes = 0;
    let mut max_abs_diff = 0;
    let mut sum_abs_diff = 0_u64;
    for (&actual, &expected) in actual.iter().zip(expected) {
        let diff = actual.abs_diff(expected);
        differing_bytes += usize::from(diff != 0);
        max_abs_diff = max_abs_diff.max(diff);
        sum_abs_diff += u64::from(diff);
    }
    ByteDiff {
        compared_bytes: actual.len(),
        differing_bytes,
        max_abs_diff,
        mean_abs_diff: sum_abs_diff as f64 / actual.len() as f64,
    }
}

fn command_route_comparison(
    path: &Path,
    info: &motolii_media::MediaInfo,
    hardware: CommandRoute,
) -> CommandRouteComparison {
    let software = software_route();
    let software_frame = capture_command_frame(path, info, &software, 0);
    let hardware_frame = capture_command_frame(path, info, &hardware, 0);
    let frame_zero_diff = byte_diff(&hardware_frame, &software_frame);
    CommandRouteComparison {
        software: command_route_report(path, info, software),
        hardware: command_route_report(path, info, hardware),
        frame_zero_diff,
    }
}

#[test]
#[ignore = "manual hardware benchmark; run with --ignored --nocapture"]
fn record_decode_demand_matrix_without_thresholds() {
    if !ffmpeg_or_skip() {
        return;
    }
    let (path, generated_fixture) = fixture_path();
    let (fixture_bytes, fixture_sha256) = fixture_identity(&path);
    let info = probe(&path).expect("probe demand fixture");
    let sequential = sequential_samples(&path);
    let seeks = seek_samples(&path);
    let (parallel_wall_ms, parallel) = parallel_samples(&path);
    let command_route_comparison =
        hardware_route_from_env().map(|hardware| command_route_comparison(&path, &info, hardware));
    let report = DecodeDemandReport {
        schema_version: 3,
        fixture: path.display().to_string(),
        fixture_bytes,
        fixture_sha256,
        generated_fixture,
        width: info.width,
        height: info.height,
        fps_num: info.fps.num(),
        fps_den: info.fps.den(),
        sequential,
        seeks,
        parallel_wall_ms,
        parallel,
        command_route_comparison,
    };
    let json = serde_json::to_string_pretty(&report).expect("serialize decode demand report");
    eprintln!("{json}");
    if let Some(path) = std::env::var_os(OUTPUT_ENV) {
        std::fs::write(&path, &json)
            .unwrap_or_else(|error| panic!("write {}: {error}", PathBuf::from(path).display()));
    }
}
