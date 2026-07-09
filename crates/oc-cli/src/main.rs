use oc_cli::{parse_args, Command, HELP};
use oc_export::{export_overlay_video, ExportOverlayRequest};
use oc_gpu::GpuCtx;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match parse_args(std::env::args().skip(1))? {
        Command::Help => {
            print!("{HELP}");
        }
        Command::ExportOverlay(args) => {
            oc_media_tools_hint();
            let gpu = GpuCtx::new_headless()?;
            let report = export_overlay_video(
                &gpu,
                &ExportOverlayRequest {
                    input_path: &args.input,
                    output_path: &args.output,
                    start_frame: args.start_frame,
                    frame_count: args.frame_count,
                    overlay: args.overlay,
                    data_tracks: oc_eval::DataTracks::new(),
                    qp0: args.qp0,
                },
            )?;
            println!(
                "wrote {} frames: {}x{} @ {}/{} fps -> {}",
                report.frames_written,
                report.desc.width,
                report.desc.height,
                report.fps.num,
                report.fps.den,
                args.output.display()
            );
        }
        Command::ExportProject(args) => {
            oc_media_tools_hint();
            let gpu = GpuCtx::new_headless()?;
            let prepared = oc_cli::prepare_project_export(&args.project)?;
            let report = prepared.export(&gpu)?;
            println!(
                "wrote {} frames: {}x{} @ {}/{} fps -> {}",
                report.frames_written,
                report.desc.width,
                report.desc.height,
                report.fps.num,
                report.fps.den,
                prepared.output_path.display()
            );
        }
        Command::VerifyB4(args) => {
            oc_media_tools_hint();
            let gpu = GpuCtx::new_headless()?;
            match oc_cli::verify_b4_project_v1(
                &gpu,
                &args.project,
                args.tolerance,
                args.export_first,
            ) {
                Ok(report) => {
                    print_b4_report(&report, args.tolerance);
                    println!(
                        "B-4 verify passed: {}/{} frames within tolerance {}",
                        report.frames_passed, report.frames_checked, args.tolerance
                    );
                }
                Err(oc_cli::B4VerifyError::Mismatch { report, tolerance, .. }) => {
                    print_b4_report(&report, tolerance);
                    return Err(format!(
                        "B-4 verify failed: {}/{} frames exceeded tolerance {}",
                        report.frames_checked - report.frames_passed,
                        report.frames_checked,
                        tolerance
                    )
                    .into());
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
    Ok(())
}

fn oc_media_tools_hint() {
    if !oc_media::tools_available() {
        eprintln!("warning: ffmpeg/ffprobe not found on PATH; export will fail");
    }
}

fn print_b4_report(report: &oc_cli::B4VerifyReport, tolerance: u32) {
    for frame in &report.frame_results {
        println!(
            "frame {}: max_diff={} {} (tolerance {})",
            frame.export_index,
            frame.max_abs_diff,
            if frame.passed { "OK" } else { "FAIL" },
            tolerance
        );
    }
}
