use std::path::PathBuf;

use motolii_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};

mod document_debug;
pub mod document_edit;
mod document_export;
mod project;
mod verify_b4;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    ExportOverlay(Box<ExportOverlayArgs>),
    ExportProject(ExportProjectArgs),
    ExportDocument(ExportDocumentArgs),
    Dump(DumpDocumentArgs),
    Apply(ApplyDocumentArgs),
    New(NewDocumentArgs),
    ImportAsset(ImportAssetArgs),
    PlaceClip(PlaceClipArgs),
    SetSoundtrack(SetSoundtrackArgs),
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
pub struct ExportDocumentArgs {
    pub document: PathBuf,
    pub output: PathBuf,
    pub frame_count: Option<usize>,
    pub qp0: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DumpDocumentArgs {
    pub document: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewDocumentArgs {
    pub document: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportAssetArgs {
    pub project: PathBuf,
    pub media: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaceClipArgs {
    pub project: PathBuf,
    pub asset: u64,
    pub at_seconds: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetSoundtrackArgs {
    pub project: PathBuf,
    pub asset: u64,
    pub offset_seconds: f64,
    pub gain: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApplyDocumentArgs {
    pub document: PathBuf,
    pub command_json: String,
    pub out: Option<PathBuf>,
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
    #[error(transparent)]
    FirstParty(#[from] motolii_plugins_firstparty::FirstPartyError),
}

pub const HELP: &str = "\
motolii-cli

Commands:
  export-overlay --input <mp4> --output <mp4> [options]
  export-project --project <json> [options]
  export-document --document <json> --output <mp4> [options]
  dump --document <json>
  apply --document <json> --command <json-or-file> [--out <json>]
  new --document <json>
  import --project <json> --media <file>
  place --project <json> --asset <id> [--at <seconds>]
  set-soundtrack --project <json> --asset <id> [--offset <seconds>] [--gain <0..1>]
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
        Some("export-document") => parse_export_document(&args[1..]).map(Command::ExportDocument),
        Some("dump") => parse_dump(&args[1..]).map(Command::Dump),
        Some("apply") => parse_apply(&args[1..]).map(Command::Apply),
        Some("new") => parse_new(&args[1..]).map(Command::New),
        Some("import") => parse_import(&args[1..]).map(Command::ImportAsset),
        Some("place") => parse_place(&args[1..]).map(Command::PlaceClip),
        Some("set-soundtrack") => parse_set_soundtrack(&args[1..]).map(Command::SetSoundtrack),
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

fn parse_export_document(args: &[String]) -> Result<ExportDocumentArgs, CliError> {
    let mut document: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut frame_count = None;
    let mut qp0 = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--document" => {
                document = Some(PathBuf::from(take_one(args, &mut i, "--document")?));
            }
            "--output" => {
                output = Some(PathBuf::from(take_one(args, &mut i, "--output")?));
            }
            "--frame-count" => {
                frame_count = Some(parse_one(args, &mut i, "--frame-count")?);
            }
            "--qp0" => {
                qp0 = true;
                i += 1;
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown export-document option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    Ok(ExportDocumentArgs {
        document: document.ok_or_else(|| CliError::Usage("--document is required".into()))?,
        output: output.ok_or_else(|| CliError::Usage("--output is required".into()))?,
        frame_count,
        qp0,
    })
}

fn parse_dump(args: &[String]) -> Result<DumpDocumentArgs, CliError> {
    let mut document: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--document" => {
                document = Some(PathBuf::from(take_one(args, &mut i, "--document")?));
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown dump option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    Ok(DumpDocumentArgs {
        document: document.ok_or_else(|| CliError::Usage("--document is required".into()))?,
    })
}

fn parse_new(args: &[String]) -> Result<NewDocumentArgs, CliError> {
    let mut document: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--document" => {
                document = Some(PathBuf::from(take_one(args, &mut i, "--document")?));
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown new option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    Ok(NewDocumentArgs {
        document: document.ok_or_else(|| CliError::Usage("--document is required".into()))?,
    })
}

fn parse_import(args: &[String]) -> Result<ImportAssetArgs, CliError> {
    let mut project: Option<PathBuf> = None;
    let mut media: Option<PathBuf> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--project" => project = Some(PathBuf::from(take_one(args, &mut i, "--project")?)),
            "--media" => media = Some(PathBuf::from(take_one(args, &mut i, "--media")?)),
            other => {
                return Err(CliError::Usage(format!(
                    "unknown import option: {other}\n\n{HELP}"
                )))
            }
        }
    }
    Ok(ImportAssetArgs {
        project: project.ok_or_else(|| CliError::Usage("--project is required".into()))?,
        media: media.ok_or_else(|| CliError::Usage("--media is required".into()))?,
    })
}

fn parse_f64(raw: &str, flag: &str) -> Result<f64, CliError> {
    raw.parse()
        .map_err(|_| CliError::Usage(format!("{flag} expects a number, got: {raw}")))
}

fn parse_u64(raw: &str, flag: &str) -> Result<u64, CliError> {
    raw.parse()
        .map_err(|_| CliError::Usage(format!("{flag} expects an id, got: {raw}")))
}

fn parse_place(args: &[String]) -> Result<PlaceClipArgs, CliError> {
    let mut project: Option<PathBuf> = None;
    let mut asset: Option<u64> = None;
    let mut at_seconds = 0.0f64;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--project" => project = Some(PathBuf::from(take_one(args, &mut i, "--project")?)),
            "--asset" => asset = Some(parse_u64(&take_one(args, &mut i, "--asset")?, "--asset")?),
            "--at" => at_seconds = parse_f64(&take_one(args, &mut i, "--at")?, "--at")?,
            other => {
                return Err(CliError::Usage(format!(
                    "unknown place option: {other}\n\n{HELP}"
                )))
            }
        }
    }
    Ok(PlaceClipArgs {
        project: project.ok_or_else(|| CliError::Usage("--project is required".into()))?,
        asset: asset.ok_or_else(|| CliError::Usage("--asset is required".into()))?,
        at_seconds,
    })
}

fn parse_set_soundtrack(args: &[String]) -> Result<SetSoundtrackArgs, CliError> {
    let mut project: Option<PathBuf> = None;
    let mut asset: Option<u64> = None;
    let mut offset_seconds = 0.0f64;
    let mut gain = 1.0f64;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--project" => project = Some(PathBuf::from(take_one(args, &mut i, "--project")?)),
            "--asset" => asset = Some(parse_u64(&take_one(args, &mut i, "--asset")?, "--asset")?),
            "--offset" => {
                offset_seconds = parse_f64(&take_one(args, &mut i, "--offset")?, "--offset")?
            }
            "--gain" => gain = parse_f64(&take_one(args, &mut i, "--gain")?, "--gain")?,
            other => {
                return Err(CliError::Usage(format!(
                    "unknown set-soundtrack option: {other}\n\n{HELP}"
                )))
            }
        }
    }
    Ok(SetSoundtrackArgs {
        project: project.ok_or_else(|| CliError::Usage("--project is required".into()))?,
        asset: asset.ok_or_else(|| CliError::Usage("--asset is required".into()))?,
        offset_seconds,
        gain,
    })
}

fn parse_apply(args: &[String]) -> Result<ApplyDocumentArgs, CliError> {
    let mut document: Option<PathBuf> = None;
    let mut command_json: Option<String> = None;
    let mut out: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => return Err(CliError::Usage(HELP.to_string())),
            "--document" => {
                document = Some(PathBuf::from(take_one(args, &mut i, "--document")?));
            }
            "--command" => {
                command_json = Some(take_command_json(args, &mut i)?);
            }
            "--out" => {
                out = Some(PathBuf::from(take_one(args, &mut i, "--out")?));
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown apply option: {other}\n\n{HELP}"
                )))
            }
        }
    }

    Ok(ApplyDocumentArgs {
        document: document.ok_or_else(|| CliError::Usage("--document is required".into()))?,
        command_json: command_json
            .ok_or_else(|| CliError::Usage("--command is required".into()))?,
        out,
    })
}

fn take_command_json(args: &[String], i: &mut usize) -> Result<String, CliError> {
    let value = take_one(args, i, "--command")?;
    if value.starts_with('{') || value.starts_with('[') {
        Ok(value)
    } else {
        std::fs::read_to_string(&value).map_err(|e| CliError::Usage(e.to_string()))
    }
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

pub use document_debug::{apply_document, dump_document, new_document};
pub use document_export::export_document_file as export_document;
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
    match project::export_project_v1(gpu, project_path.as_ref()) {
        Ok(report) => Ok(report),
        Err(project::ProjectError::FirstParty(err)) => Err(CliError::FirstParty(err)),
        Err(other) => Err(CliError::Usage(other.to_string())),
    }
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
