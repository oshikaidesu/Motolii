use std::path::PathBuf;

use oc_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};

mod project;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ExportOverlay(ExportOverlayArgs),
    ExportProject(ExportProjectArgs),
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

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("{0}")]
    Usage(String),
}

pub const HELP: &str = "\
oc-cli

Commands:
  export-overlay --input <mp4> --output <mp4> [options]
  export-project --project <json> [options]

Options:
  --start-frame <n>       First source frame to export (default: 0)
  --frame-count <n>       Number of frames to export (default: to end)
  --center <x> <y>        Overlay center in canonical coords (default: 0 0)
  --size <w> <h>          Overlay size in canonical coords (default: 0.25 0.25)
  --color <r> <g> <b> <a> Overlay straight RGBA, 0..1 (default: 1 0 0 0.5)
  --qp0                  Use near-lossless H.264 for verification
  --project <json>       Project file path (versioned JSON)
  --help                 Show this help
";

pub fn parse_args<I, S>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args: Vec<String> = args.into_iter().map(Into::into).collect();
    if args
        .first()
        .map(|s| {
            !matches!(
                s.as_str(),
                "export-overlay" | "export-project" | "--help" | "-h"
            )
        })
        .unwrap_or(false)
        && args.len() > 1
    {
        args.remove(0);
    }
    match args.first().map(|s| s.as_str()) {
        None | Some("--help") | Some("-h") => Ok(Command::Help),
        Some("export-overlay") => parse_export_overlay(&args[1..]).map(Command::ExportOverlay),
        Some("export-project") => parse_export_project(&args[1..]).map(Command::ExportProject),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_export_overlay_command() {
        let cmd = parse_args([
            "oc-cli",
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
    fn accepts_real_binary_path_as_argv0() {
        assert_eq!(
            parse_args(["target/debug/oc-cli", "--help"]).unwrap(),
            Command::Help
        );
    }

    #[test]
    fn parses_export_project_command() {
        let cmd = parse_args(["oc-cli", "export-project", "--project", "proj.json"]).unwrap();

        let Command::ExportProject(args) = cmd else {
            panic!("expected export-project command");
        };
        assert_eq!(args.project, PathBuf::from("proj.json"));
    }
}

pub use project::{load_project_v1, load_project_v1_from_str, ProjectV1, RectOverlayParamV1};

pub fn export_project(
    gpu: &oc_gpu::GpuCtx,
    project_path: impl AsRef<std::path::Path>,
) -> Result<oc_export::ExportReport, CliError> {
    project::export_project_v1(gpu, project_path.as_ref())
        .map_err(|e| CliError::Usage(e.to_string()))
}
