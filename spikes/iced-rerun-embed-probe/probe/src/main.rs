//! iced master(0.15.0-dev, wgpu 29)の shader widget に、Rerun fork の `SpatialStage` が
//! offscreen で合成した frame を**同一 wgpu device のまま**埋め込めるかの隔離 probe。
//!
//! 測るのは3点。
//!
//! - **(a) 同一device成立**: iced のランタイム device で `re_renderer::RenderContext` を作り、
//!   アプリ全体が wgpu device 1つで動くか。2つ要るなら不成立。
//! - **(b) 絵の到達**: E0 と同じ4象限シーンが iced の窓の中に出るか。PNG を撮って
//!   E0 の期待色 oracle(四隅)にかける。
//! - **(c) 入力→Message**: shader widget 上の click が型付き Message になって
//!   `update` に届き、それを起点に `set_camera` で絵が変わるか。
//!
//! ## モード
//!
//! - `preflight <出力ディレクトリ>` — 窓を開かずに (a) を切り分ける。
//!   iced の device 記述と `re_renderer` の device 記述で同じシーンを回し、
//!   どちらで通るかを見る。子プロセスに分けてあるので、片方が panic しても
//!   もう片方の結果は残る。
//! - `gui <出力ディレクトリ>` — 本番。iced の窓を開き、(b) と (c) を撮って自分で終了する。
//! - `gui-interactive <出力ディレクトリ>` — 同じだが (c) の click を合成せず、
//!   **人が実際に窓を押すのを待つ**。winit → widget tree の配送まで含めて確かめたいとき用。
//!
//! ## この probe が測らないもの(発注時の明示的な範囲外)
//!
//! `iced_test`/Simulator の統合、AccessKit、IME、タイムライン widget。

mod app;
mod bridge;
mod bridge_app;
mod embed;
mod harness;
mod oracle;
mod scene;
mod stage;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use harness::{DeviceKind, Gpu, Offscreen};
use oracle::Frame;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("preflight") => match args.get(2) {
            Some(out_dir) => run_preflight(Path::new(out_dir)),
            None => usage(),
        },
        Some("preflight-case") => match (args.get(2), args.get(3)) {
            (Some(kind), Some(out_dir)) => run_preflight_case(kind, Path::new(out_dir)),
            _ => usage(),
        },
        Some("gui") => match args.get(2) {
            Some(out_dir) => run_gui(PathBuf::from(out_dir), false),
            None => usage(),
        },
        Some("gui-interactive") => match args.get(2) {
            Some(out_dir) => run_gui(PathBuf::from(out_dir), true),
            None => usage(),
        },
        Some("bridge") => match args.get(2) {
            Some(out_dir) => run_bridge(PathBuf::from(out_dir), false),
            None => usage(),
        },
        Some("bridge-interactive") => match args.get(2) {
            Some(out_dir) => run_bridge(PathBuf::from(out_dir), true),
            None => usage(),
        },
        Some("bridge-offscreen") => match (args.get(2), args.get(3)) {
            (Some(kind), Some(out_dir)) => run_bridge_offscreen(kind, Path::new(out_dir)),
            _ => usage(),
        },
        _ => usage(),
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         iced-rerun-embed-probe preflight <output-dir>\n  \
         iced-rerun-embed-probe preflight-case <iced-windowed|re_renderer> <output-dir>\n  \
         iced-rerun-embed-probe gui <output-dir>\n  \
         iced-rerun-embed-probe gui-interactive <output-dir>   # (c) waits for a real click\n  \
         iced-rerun-embed-probe bridge <output-dir>            # (d)(e)(f) scripted drag/wheel\n  \
         iced-rerun-embed-probe bridge-interactive <output-dir> # (d)(e)(f) driven by a human\n  \
         iced-rerun-embed-probe bridge-offscreen <iced-windowed|re_renderer> <output-dir>"
    );
    ExitCode::from(2)
}

// ---------------------------------------------------------------------------
// preflight — (a) の切り分け
// ---------------------------------------------------------------------------

fn run_preflight(out_dir: &Path) -> ExitCode {
    if let Err(error) = std::fs::create_dir_all(out_dir) {
        eprintln!("could not create {}: {error}", out_dir.display());
        return ExitCode::FAILURE;
    }
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            eprintln!("current_exe: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut transcript = String::from(
        "# preflight — does re_renderer run on the device iced would have made?\n\n\
         Each case is a separate child process so that a validation panic in one\n\
         does not take the other's result with it.\n",
    );
    let mut wrong_picture = Vec::new();
    let mut complained = Vec::new();

    for kind in [DeviceKind::IcedWindowed, DeviceKind::ReRenderer] {
        let label = kind.label();
        println!("\n=== preflight case: {label} ===");
        transcript.push_str(&format!("\n## case: {label}\n\n```\n"));

        let output = std::process::Command::new(&exe)
            .arg("preflight-case")
            .arg(label)
            .arg(out_dir)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                print!("{stdout}");
                if !stderr.trim().is_empty() {
                    print!("{stderr}");
                }
                transcript.push_str(&stdout);
                if !stderr.trim().is_empty() {
                    transcript.push_str("\n-- stderr --\n");
                    transcript.push_str(&stderr);
                }
                transcript.push_str(&format!("```\n\nexit: {:?}\n", output.status.code()));
                match output.status.code() {
                    Some(0) => {}
                    Some(3) => complained.push(label),
                    _ => wrong_picture.push(label),
                }
            }
            Err(error) => {
                let line = format!("could not run the child process: {error}\n");
                print!("{line}");
                transcript.push_str(&line);
                transcript.push_str("```\n");
                wrong_picture.push(label);
            }
        }
    }

    let iced = DeviceKind::IcedWindowed.label();
    let verdict = if wrong_picture.contains(&iced) {
        "(a) NOT ESTABLISHED: the picture the iced-shaped device produced is wrong — \
         see the case above"
    } else if complained.contains(&iced) {
        "(a) ESTABLISHED WITH A CAVEAT: one device serves both, and the picture matches the \
         re_renderer-shaped device exactly — but wgpu refused to build part of re_renderer \
         on iced's limits (see the case above). Whatever needs that part will be missing."
    } else {
        "(a) ESTABLISHED: the device iced makes is enough for re_renderer; one device serves both"
    };
    println!("\n{verdict}");
    transcript.push_str(&format!("\n## verdict\n\n{verdict}\n"));

    let path = out_dir.join("preflight.txt");
    if let Err(error) = std::fs::write(&path, transcript) {
        eprintln!("could not write {}: {error}", path.display());
    } else {
        println!("wrote {}", path.display());
    }

    if wrong_picture.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_preflight_case(kind: &str, out_dir: &Path) -> ExitCode {
    let kind = match kind {
        "iced-windowed" => DeviceKind::IcedWindowed,
        "re_renderer" => DeviceKind::ReRenderer,
        other => {
            eprintln!("unknown device kind: {other}");
            return ExitCode::from(2);
        }
    };
    match preflight_case(kind, out_dir) {
        Ok(Verdict::Clean(report)) => {
            println!("{report}\nPASS");
            ExitCode::SUCCESS
        }
        Ok(Verdict::PictureRightButComplained(report)) => {
            println!(
                "{report}\nPASS (picture) / COMPLAINED (wgpu) — the oracle is satisfied but wgpu \
                 refused to build part of re_renderer on this device"
            );
            ExitCode::from(3)
        }
        Err(report) => {
            println!("{report}\nFAIL");
            ExitCode::FAILURE
        }
    }
}

/// preflight の3値。「絵は正しいが wgpu が文句を言った」を潰さないために分ける。
enum Verdict {
    Clean(String),
    PictureRightButComplained(String),
}

/// E0 の (a)+(b) をそのまま、指定の device 記述の上で回す。
const PREFLIGHT_W: u32 = 640;
const PREFLIGHT_H: u32 = 480;

fn preflight_case(kind: DeviceKind, out_dir: &Path) -> Result<Verdict, String> {
    let (gpu, note) = Gpu::create(kind)?;
    let mut report = format!("{note}\n");

    let mut offscreen = Offscreen::new(gpu, PREFLIGHT_W, PREFLIGHT_H)
        .map_err(|error| format!("{report}\n{error}"))?;
    let mut spatial = stage::new_stage(&format!("preflight-{}", kind.label()))
        .map_err(|error| format!("{report}\n{error}"))?;
    stage::install_quadrants(&mut offscreen, &mut spatial)
        .map_err(|error| format!("{report}\n{error}"))?;

    for _ in 0..stage::WARMUP_FRAMES {
        if let Err(error) = offscreen.frame(&mut spatial) {
            return Err(format!("{report}\nframe: {error}"));
        }
    }
    spatial.set_camera(stage::document_camera(1.0));
    for _ in 0..stage::SETTLE_FRAMES {
        if let Err(error) = offscreen.frame(&mut spatial) {
            return Err(format!("{report}\nframe: {error}"));
        }
    }

    let rgba = offscreen
        .read_rgba()
        .map_err(|error| format!("{report}\nreadback: {error}"))?;
    let frame = Frame::new(rgba, PREFLIGHT_W, PREFLIGHT_H);
    let png = out_dir.join(format!("preflight-{}.png", kind.label()));
    frame
        .write_png(&png)
        .map_err(|error| format!("{report}\n{error}"))?;
    report.push_str(&format!(
        "PNG: {}\nfnv1a: {:016x}\n",
        png.display(),
        frame.digest()
    ));

    let (histogram, present_failures) = stage::check_all_quadrants_present(&frame);
    report.push_str(&histogram);
    let (corners, corner_failures) = stage::check_frame_corners(&frame);
    report.push_str(&corners);

    let validation = offscreen.take_errors();
    if validation.is_empty() {
        report.push_str("no wgpu validation errors\n");
    } else {
        report.push_str(&format!(
            "wgpu validation errors:\n  {}\n",
            validation.join("\n  ")
        ));
    }

    // 絵が違うのと、絵は合っているが wgpu が文句を言ったのとは、別の話である。
    let picture_failures: Vec<String> = present_failures
        .into_iter()
        .chain(corner_failures)
        .collect();
    if !picture_failures.is_empty() {
        return Err(format!("{report}\n{}", picture_failures.join("\n")));
    }
    if validation.is_empty() {
        Ok(Verdict::Clean(report))
    } else {
        Ok(Verdict::PictureRightButComplained(report))
    }
}

// ---------------------------------------------------------------------------
// gui — (b)(c)
// ---------------------------------------------------------------------------

fn run_gui(out_dir: PathBuf, interactive: bool) -> ExitCode {
    if let Err(error) = std::fs::create_dir_all(&out_dir) {
        eprintln!("could not create {}: {error}", out_dir.display());
        return ExitCode::FAILURE;
    }
    app::set_out_dir(out_dir);
    app::set_interactive(interactive);

    let result = iced::application(app::App::boot, app::App::update, app::App::view)
        .title(app::App::title)
        .subscription(app::App::subscription)
        .window_size((app::WINDOW_W, app::WINDOW_H))
        .resizable(false)
        .antialiasing(false)
        .run();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("iced failed: {error}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// bridge — (d)(e)(f)
// ---------------------------------------------------------------------------

fn run_bridge(out_dir: PathBuf, interactive: bool) -> ExitCode {
    if let Err(error) = std::fs::create_dir_all(&out_dir) {
        eprintln!("could not create {}: {error}", out_dir.display());
        return ExitCode::FAILURE;
    }
    match bridge_app::run(out_dir, interactive) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("iced failed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// 対照群。窓を開かずに同じ台本を回す。子プロセスに分けなくても片方ずつ走らせられる。
fn run_bridge_offscreen(kind: &str, out_dir: &Path) -> ExitCode {
    if let Err(error) = std::fs::create_dir_all(out_dir) {
        eprintln!("could not create {}: {error}", out_dir.display());
        return ExitCode::FAILURE;
    }
    let kind = match kind {
        "iced-windowed" => DeviceKind::IcedWindowed,
        "re_renderer" => DeviceKind::ReRenderer,
        other => {
            eprintln!("unknown device kind: {other}");
            return ExitCode::from(2);
        }
    };
    match bridge_app::run_offscreen(kind, out_dir) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bridge-offscreen failed: {error}");
            ExitCode::FAILURE
        }
    }
}
