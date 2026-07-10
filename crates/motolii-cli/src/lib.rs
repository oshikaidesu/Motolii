use std::path::PathBuf;

use motolii_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};

mod project;
mod verify_b4;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ExportOverlay(Box<ExportOverlayArgs>),
    ExportProject(ExportProjectArgs),
    VerifyB4(VerifyB4Args),
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportOverlayArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    pub start_frame: i64,
    pub frame_count: Option<usize>,
    pub overlay: ParamRectOverlay,
    pub qp0: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportProjectArgs {
    pub project: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifyB4Args {
    pub project: PathBuf,
    /// 検証前に書き出しを実行する。
    pub export_first: bool,
    pub tolerance: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{0}")]
    Usage(String),
}

pub const HELP: &str = "\
motolii-cli

Commands:
  export-overlay --input <mp4> --output <mp4> [options]
  export-project --project <json> [options]
  verify-b4 --project <json> [options]

Options:
  --start-frame <n>       First source frame to export (default: 0)
  --frame-count <n>       Number of frames to export (default: to end)
  --center <x> <y>        Overlay center in canonical coords (default: 0 0)
  --size <w> <h>          Overlay size in canonical coords (default: 0.25 0.25)
  --color <r> <g> <b> <a> Overlay straight RGBA, 0..1 (default: 1 0 0 0.5)
  --qp0                  Use near-lossless H.264 for verification
  --project <json>       Project file path (versioned JSON)
  --export               verify-b4: export before comparing (default: on)
  --no-export            verify-b4: compare existing output only
  --tolerance <n>        verify-b4: max per-channel diff (default: 8)
  --help                 Show this help
";

pub fn parse_args<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    match args.first().map(|s| s.as_str()) {
        None | Some("--help") | Some("-h") => Ok(Command::Help),
        Some("export-overlay") => {
            parse_export_overlay(&args[1..]).map(|args| Command::ExportOverlay(Box::new(args)))
        }
        Some("export-project") => parse_export_project(&args[1..]).map(Command::ExportProject),
        Some("verify-b4") => parse_verify_b4(&args[1..]).map(Command::VerifyB4),
        Some(other) => Err(CliError::Usage(format!(
            "unknown command: {other}\n\n{HELP}"
        ))),
    }
}

fn parse_export_project(args: &[String]) -> Result<ExportProjectArgs, CliError> {
    let mut project: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--project" => {
                project = Some(PathBuf::from(take_one(args, &mut i, "--project")?));
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown export-project option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    Ok(ExportProjectArgs {
        project: project.ok_or_else(|| CliError::Usage("--project is required".into()))?,
    })
}

fn parse_verify_b4(args: &[String]) -> Result<VerifyB4Args, CliError> {
    let mut project: Option<PathBuf> = None;
    let mut export_first = true;
    let mut tolerance = 8u32;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--project" => {
                project = Some(PathBuf::from(take_one(args, &mut i, "--project")?));
            }
            "--export" => {
                export_first = true;
                i += 1;
            }
            "--no-export" => {
                export_first = false;
                i += 1;
            }
            "--tolerance" => {
                tolerance = parse_one(args, &mut i, "--tolerance")?;
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown verify-b4 option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    Ok(VerifyB4Args {
        project: project.ok_or_else(|| CliError::Usage("--project is required".into()))?,
        export_first,
        tolerance,
    })
}

fn parse_export_overlay(args: &[String]) -> Result<ExportOverlayArgs, CliError> {
    let mut input = None;
    let mut output = None;
    let mut start_frame = 0i64;
    let mut frame_count = None;
    let mut center = CanonicalPoint::CENTER;
    let mut size = CanonicalSize {
        width: 0.25,
        height: 0.25,
    };
    let mut color = [1.0, 0.0, 0.0, 0.5];
    let mut qp0 = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--input" => {
                input = Some(PathBuf::from(take_one(args, &mut i, "--input")?));
            }
            "--output" => {
                output = Some(PathBuf::from(take_one(args, &mut i, "--output")?));
            }
            "--start-frame" => {
                start_frame = parse_one(args, &mut i, "--start-frame")?;
            }
            "--frame-count" => {
                frame_count = Some(parse_one(args, &mut i, "--frame-count")?);
            }
            "--center" => {
                center = CanonicalPoint {
                    x: parse_at(args, i + 1, "--center x")?,
                    y: parse_at(args, i + 2, "--center y")?,
                };
                i += 3;
            }
            "--size" => {
                size = CanonicalSize {
                    width: parse_at(args, i + 1, "--size width")?,
                    height: parse_at(args, i + 2, "--size height")?,
                };
                i += 3;
            }
            "--color" => {
                color = [
                    parse_at(args, i + 1, "--color r")?,
                    parse_at(args, i + 2, "--color g")?,
                    parse_at(args, i + 3, "--color b")?,
                    parse_at(args, i + 4, "--color a")?,
                ];
                i += 5;
            }
            "--qp0" => {
                qp0 = true;
                i += 1;
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown export-overlay option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    if start_frame < 0 {
        return Err(CliError::Usage("--start-frame must be >= 0".into()));
    }
    if size.width <= 0.0 || size.height <= 0.0 {
        return Err(CliError::Usage("--size values must be > 0".into()));
    }
    if color.iter().any(|v| !(0.0..=1.0).contains(v)) {
        return Err(CliError::Usage("--color values must be in 0..1".into()));
    }

    Ok(ExportOverlayArgs {
        input: input.ok_or_else(|| CliError::Usage("--input is required".into()))?,
        output: output.ok_or_else(|| CliError::Usage("--output is required".into()))?,
        start_frame,
        frame_count,
        overlay: ParamRectOverlay::constant(RectOverlay {
            center,
            size,
            color,
        }),
        qp0,
    })
}

fn take_one(args: &[String], i: &mut usize, name: &str) -> Result<String, CliError> {
    let value = args
        .get(*i + 1)
        .ok_or_else(|| CliError::Usage(format!("{name} requires a value")))?
        .clone();
    *i += 2;
    Ok(value)
}

fn parse_one<T>(args: &[String], i: &mut usize, name: &str) -> Result<T, CliError>
where
    T: std::str::FromStr,
{
    let raw = take_one(args, i, name)?;
    parse_raw(&raw, name)
}

fn parse_at<T>(args: &[String], index: usize, name: &str) -> Result<T, CliError>
where
    T: std::str::FromStr,
{
    let raw = args
        .get(index)
        .ok_or_else(|| CliError::Usage(format!("{name} requires a value")))?;
    parse_raw(raw, name)
}

fn parse_raw<T>(raw: &str, name: &str) -> Result<T, CliError>
where
    T: std::str::FromStr,
{
    raw.parse()
        .map_err(|_| CliError::Usage(format!("{name} has invalid value: {raw}")))
}

pub use project::{
    build_data_tracks, load_project_v1, load_project_v1_from_str, prepare_project_export,
    ParamDriverV1, PreparedProject, ProjectV1, RectOverlayParamV1,
};
pub use verify_b4::{
    verify_b4_project_v1, verify_prepared_b4, B4FrameResult, B4VerifyError, B4VerifyReport,
};

pub fn export_project(
    gpu: &motolii_gpu::GpuCtx,
    project_path: impl AsRef<std::path::Path>,
) -> Result<motolii_export::ExportReport, CliError> {
    project::export_project_v1(gpu, project_path.as_ref())
        .map_err(|e| CliError::Usage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_export_overlay_command() {
        let cmd = parse_args([
            "export-overlay",
            "--input",
            "in.mp4",
            "--output",
            "out.mp4",
            "--start-frame",
            "3",
            "--frame-count",
            "12",
            "--center",
            "0.1",
            "-0.2",
            "--size",
            "0.3",
            "0.4",
            "--color",
            "1",
            "0.5",
            "0",
            "0.75",
            "--qp0",
        ])
        .unwrap();

        let Command::ExportOverlay(args) = cmd else {
            panic!("expected export command");
        };
        assert_eq!(args.input, PathBuf::from("in.mp4"));
        assert_eq!(args.output, PathBuf::from("out.mp4"));
        assert_eq!(args.start_frame, 3);
        assert_eq!(args.frame_count, Some(12));
        assert_eq!(
            args.overlay,
            ParamRectOverlay::constant(RectOverlay {
                center: CanonicalPoint { x: 0.1, y: -0.2 },
                size: CanonicalSize {
                    width: 0.3,
                    height: 0.4
                },
                color: [1.0, 0.5, 0.0, 0.75],
            })
        );
        assert!(args.qp0);
    }

    #[test]
    fn rejects_missing_required_paths() {
        assert!(parse_args(["export-overlay", "--input", "in.mp4"]).is_err());
    }

    #[test]
    fn shows_help_when_no_args() {
        assert_eq!(parse_args(Vec::<String>::new()).unwrap(), Command::Help);
    }

    #[test]
    fn parses_export_project_command() {
        let cmd = parse_args(["export-project", "--project", "proj.json"]).unwrap();

        let Command::ExportProject(args) = cmd else {
            panic!("expected export-project command");
        };
        assert_eq!(args.project, PathBuf::from("proj.json"));
    }
}
