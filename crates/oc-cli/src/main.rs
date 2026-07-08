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
            let report = oc_cli::export_project(&gpu, &args.project)?;
            println!(
                "wrote {} frames: {}x{} @ {}/{} fps -> {}",
                report.frames_written,
                report.desc.width,
                report.desc.height,
                report.fps.num,
                report.fps.den,
                args.project.display()
            );
        }
    }
    Ok(())
}

fn oc_media_tools_hint() {
    if !oc_media::tools_available() {
        eprintln!("warning: ffmpeg/ffprobe not found on PATH; export will fail");
    }
}
